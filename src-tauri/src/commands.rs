use crate::{
    db::{insert_operation_log, load_settings, open_connection, persist_settings, upsert_setting},
    models::*,
    state::{resolve_path, AppState},
    utils::{
        append_text_log, current_month, ensure_parent_dir, file_sha256, generate_pinyin, make_business_no,
        month_range, normalize_optional_text, now_string, sha256_hex,
    },
};
use anyhow::{anyhow, Context, Result};
use rusqlite::{params, Connection, OptionalExtension, Row, Transaction};
use serde_json::json;
use std::{
    collections::HashSet,
    fs,
    path::{PathBuf},
};

#[tauri::command]
pub fn member_search(
    state: tauri::State<'_, AppState>,
    keyword: String,
    page_no: i64,
    page_size: i64,
) -> Result<PagedResult<MemberSummary>, String> {
    member_search_impl(&state, keyword, page_no, page_size)
        .map_err(|err| map_command_error(&state, "member_search", err))
}

#[tauri::command]
pub fn member_get_detail(
    state: tauri::State<'_, AppState>,
    member_id: i64,
) -> Result<MemberDetailData, String> {
    member_get_detail_impl(&state, member_id)
        .map_err(|err| map_command_error(&state, "member_get_detail", err))
}

#[tauri::command]
pub fn member_create(
    state: tauri::State<'_, AppState>,
    payload: MemberFormPayload,
) -> Result<CommandResult, String> {
    member_create_impl(&state, payload).map_err(|err| map_command_error(&state, "member_create", err))
}

#[tauri::command]
pub fn member_update(
    state: tauri::State<'_, AppState>,
    member_id: i64,
    payload: MemberFormPayload,
) -> Result<CommandResult, String> {
    member_update_impl(&state, member_id, payload)
        .map_err(|err| map_command_error(&state, "member_update", err))
}

#[tauri::command]
pub fn member_disable(
    state: tauri::State<'_, AppState>,
    member_id: i64,
) -> Result<CommandResult, String> {
    member_disable_impl(&state, member_id)
        .map_err(|err| map_command_error(&state, "member_disable", err))
}

#[tauri::command]
pub fn consume_create(
    state: tauri::State<'_, AppState>,
    payload: ConsumePayload,
) -> Result<CommandResult, String> {
    consume_create_impl(&state, payload).map_err(|err| map_command_error(&state, "consume_create", err))
}

#[tauri::command]
pub fn gift_list(state: tauri::State<'_, AppState>) -> Result<Vec<GiftRecord>, String> {
    gift_list_impl(&state).map_err(|err| map_command_error(&state, "gift_list", err))
}

#[tauri::command]
pub fn gift_save(
    state: tauri::State<'_, AppState>,
    payload: GiftRecord,
) -> Result<CommandResult, String> {
    gift_save_impl(&state, payload).map_err(|err| map_command_error(&state, "gift_save", err))
}

#[tauri::command]
pub fn gift_redeem(
    state: tauri::State<'_, AppState>,
    payload: RedeemPayload,
) -> Result<CommandResult, String> {
    gift_redeem_impl(&state, payload).map_err(|err| map_command_error(&state, "gift_redeem", err))
}

#[tauri::command]
pub fn settings_get(state: tauri::State<'_, AppState>) -> Result<SettingsData, String> {
    settings_get_impl(&state).map_err(|err| map_command_error(&state, "settings_get", err))
}

#[tauri::command]
pub fn settings_save(
    state: tauri::State<'_, AppState>,
    payload: SettingsData,
) -> Result<CommandResult, String> {
    settings_save_impl(&state, payload).map_err(|err| map_command_error(&state, "settings_save", err))
}

#[tauri::command]
pub fn report_dashboard(
    state: tauri::State<'_, AppState>,
    month: String,
) -> Result<DashboardData, String> {
    report_dashboard_impl(&state, month).map_err(|err| map_command_error(&state, "report_dashboard", err))
}

#[tauri::command]
pub fn backup_create(
    state: tauri::State<'_, AppState>,
    target_path: Option<String>,
) -> Result<BackupResult, String> {
    backup_create_impl(&state, target_path).map_err(|err| map_command_error(&state, "backup_create", err))
}

#[tauri::command]
pub fn backup_restore(
    state: tauri::State<'_, AppState>,
    file_path: String,
) -> Result<CommandResult, String> {
    backup_restore_impl(&state, file_path).map_err(|err| map_command_error(&state, "backup_restore", err))
}

#[tauri::command]
pub fn migration_precheck(
    state: tauri::State<'_, AppState>,
    source_path: String,
    import_scope: String,
) -> Result<MigrationPrecheckResult, String> {
    migration_precheck_impl(&state, source_path, import_scope)
        .map_err(|err| map_command_error(&state, "migration_precheck", err))
}

#[tauri::command]
pub fn migration_execute(
    state: tauri::State<'_, AppState>,
    batch_no: String,
) -> Result<CommandResult, String> {
    migration_execute_impl(&state, batch_no).map_err(|err| map_command_error(&state, "migration_execute", err))
}

#[tauri::command]
pub fn migration_get_report(
    state: tauri::State<'_, AppState>,
    batch_no: String,
) -> Result<MigrationReport, String> {
    migration_get_report_impl(&state, batch_no)
        .map_err(|err| map_command_error(&state, "migration_get_report", err))
}

#[tauri::command]
pub fn operation_logs_query(
    state: tauri::State<'_, AppState>,
    filter: OperationLogFilter,
) -> Result<Vec<OperationLogItem>, String> {
    operation_logs_query_impl(&state, filter)
        .map_err(|err| map_command_error(&state, "operation_logs_query", err))
}

fn member_search_impl(
    state: &AppState,
    keyword: String,
    page_no: i64,
    page_size: i64,
) -> Result<PagedResult<MemberSummary>> {
    let conn = open_connection(&state.db_path)?;
    let page_no = page_no.max(1);
    let page_size = page_size.clamp(1, 100);
    let offset = (page_no - 1) * page_size;
    let keyword = keyword.trim().to_string();
    let like = format!("%{}%", keyword);
    let initials_like = format!("{}%", keyword.to_lowercase());

    let total: i64 = conn.query_row(
        "SELECT COUNT(1)
         FROM members
         WHERE ?1 = ''
            OR CAST(member_no AS TEXT) LIKE ?2
            OR IFNULL(mobile, '') LIKE ?2
            OR name LIKE ?2
            OR IFNULL(name_initials, '') LIKE ?3
            OR IFNULL(name_pinyin, '') LIKE ?2",
        params![keyword, like, initials_like],
        |row| row.get(0),
    )?;

    let mut stmt = conn.prepare(
        "SELECT id, member_no, name, mobile, points_balance, total_spent, last_consume_at, status
         FROM members
         WHERE ?1 = ''
            OR CAST(member_no AS TEXT) LIKE ?2
            OR IFNULL(mobile, '') LIKE ?2
            OR name LIKE ?2
            OR IFNULL(name_initials, '') LIKE ?3
            OR IFNULL(name_pinyin, '') LIKE ?2
         ORDER BY status = 'ACTIVE' DESC, updated_at DESC, id DESC
         LIMIT ?4 OFFSET ?5",
    )?;

    let rows = stmt.query_map(params![keyword, like, initials_like, page_size, offset], |row| {
        Ok(MemberSummary {
            id: row.get(0)?,
            member_no: row.get(1)?,
            name: row.get(2)?,
            mobile: row.get(3)?,
            points_balance: row.get(4)?,
            total_spent: row.get(5)?,
            last_consume_at: row.get(6)?,
            status: row.get(7)?,
        })
    })?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }

    Ok(PagedResult {
        items,
        total,
        page_no,
        page_size,
    })
}

fn member_get_detail_impl(state: &AppState, member_id: i64) -> Result<MemberDetailData> {
    let conn = open_connection(&state.db_path)?;
    let member = conn
        .query_row(
            "SELECT id, member_no, name, gender, birth_month, birth_day, mobile, name_pinyin, name_initials,
                    points_balance, total_spent, last_consume_at, status, remark, legacy_member_id, created_at, updated_at
             FROM members WHERE id = ?1",
            params![member_id],
            map_member_record,
        )
        .optional()?
        .ok_or_else(|| anyhow!("会员不存在"))?;

    let recent_consumptions = query_recent_consumptions(&conn, member_id)?;
    let recent_redemptions = query_recent_redemptions(&conn, member_id)?;
    let recent_ledger = query_recent_ledger(&conn, member_id)?;
    let redeemed_gift_ids = query_redeemed_gift_ids(&conn, member_id)?;

    Ok(MemberDetailData {
        member,
        recent_consumptions,
        recent_redemptions,
        recent_ledger,
        redeemed_gift_ids,
    })
}

fn member_create_impl(state: &AppState, payload: MemberFormPayload) -> Result<CommandResult> {
    let name = payload.name.trim().to_string();
    if name.is_empty() {
        return Err(anyhow!("会员姓名不能为空"));
    }
    let birth_month = validate_member_birth_part("生日月", payload.birth_month.clone(), 1, 12)?;
    let birth_day = validate_member_birth_part("生日日", payload.birth_day.clone(), 1, 31)?;

    let mut conn = open_connection(&state.db_path)?;
    let tx = conn.transaction()?;
    let member_no = match payload.member_no {
        Some(no) if no > 0 => no,
        _ => next_member_no(&tx)?,
    };

    let exists: i64 = tx.query_row(
        "SELECT COUNT(1) FROM members WHERE member_no = ?1",
        params![member_no],
        |row| row.get(0),
    )?;
    if exists > 0 {
        return Err(anyhow!("会员编号已存在"));
    }

    let now = now_string();
    let (name_pinyin, name_initials) = generate_pinyin(&name);
    tx.execute(
        "INSERT INTO members(member_no, name, gender, birth_month, birth_day, mobile, name_pinyin, name_initials, points_balance, total_spent, status, remark, created_at, updated_at)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, 0, 'ACTIVE', ?9, ?10, ?10)",
        params![
            member_no,
            name,
            normalize_optional_text(payload.gender),
            birth_month,
            birth_day,
            normalize_optional_text(payload.mobile),
            name_pinyin,
            name_initials,
            normalize_optional_text(payload.remark),
            now
        ],
    )?;
    let member_id = tx.last_insert_rowid();

    let summary = serde_json::to_string(&json!({
        "memberNo": member_no,
        "name": payload.name
    }))?;
    insert_operation_log(
        &tx,
        "管理员",
        "member",
        "create",
        Some("member"),
        Some(&member_id.to_string()),
        Some(&summary),
        "SUCCESS",
        None,
    )?;

    tx.commit()?;
    append_text_log(&state.log_dir, "app.log", &format!("创建会员成功: {}", member_no))?;

    Ok(CommandResult {
        success: true,
        message: "会员已创建".to_string(),
        target_id: Some(member_id.to_string()),
    })
}

fn member_update_impl(state: &AppState, member_id: i64, payload: MemberFormPayload) -> Result<CommandResult> {
    let name = payload.name.trim().to_string();
    if name.is_empty() {
        return Err(anyhow!("会员姓名不能为空"));
    }
    let birth_month = validate_member_birth_part("生日月", payload.birth_month.clone(), 1, 12)?;
    let birth_day = validate_member_birth_part("生日日", payload.birth_day.clone(), 1, 31)?;

    let mut conn = open_connection(&state.db_path)?;
    let tx = conn.transaction()?;
    let existing_no: i64 = tx
        .query_row(
            "SELECT member_no FROM members WHERE id = ?1",
            params![member_id],
            |row| row.get(0),
        )
        .optional()?
        .ok_or_else(|| anyhow!("会员不存在"))?;
    let member_no = payload.member_no.unwrap_or(existing_no);
    let duplicate: i64 = tx.query_row(
        "SELECT COUNT(1) FROM members WHERE member_no = ?1 AND id <> ?2",
        params![member_no, member_id],
        |row| row.get(0),
    )?;
    if duplicate > 0 {
        return Err(anyhow!("会员编号已被其他会员占用"));
    }

    let (name_pinyin, name_initials) = generate_pinyin(&name);
    tx.execute(
        "UPDATE members
         SET member_no = ?1, name = ?2, gender = ?3, birth_month = ?4, birth_day = ?5, mobile = ?6,
             name_pinyin = ?7, name_initials = ?8, remark = ?9, updated_at = ?10
         WHERE id = ?11",
        params![
            member_no,
            name,
            normalize_optional_text(payload.gender),
            birth_month,
            birth_day,
            normalize_optional_text(payload.mobile),
            name_pinyin,
            name_initials,
            normalize_optional_text(payload.remark),
            now_string(),
            member_id
        ],
    )?;
    let summary = serde_json::to_string(&json!({
        "memberId": member_id,
        "memberNo": member_no,
        "name": payload.name
    }))?;
    insert_operation_log(
        &tx,
        "管理员",
        "member",
        "update",
        Some("member"),
        Some(&member_id.to_string()),
        Some(&summary),
        "SUCCESS",
        None,
    )?;
    tx.commit()?;

    Ok(CommandResult {
        success: true,
        message: "会员资料已更新".to_string(),
        target_id: Some(member_id.to_string()),
    })
}

fn validate_member_birth_part(
    label: &str,
    value: Option<String>,
    min: i64,
    max: i64,
) -> Result<Option<String>> {
    let Some(text) = normalize_optional_text(value) else {
        return Ok(None);
    };
    if !text.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(anyhow!("{label}必须为数字"));
    }
    let parsed: i64 = text
        .parse()
        .map_err(|_| anyhow!("{label}格式不正确"))?;
    if parsed < min || parsed > max {
        return Err(anyhow!("{label}必须在 {min} 到 {max} 之间"));
    }
    Ok(Some(format!("{parsed:02}")))
}

fn member_disable_impl(state: &AppState, member_id: i64) -> Result<CommandResult> {
    let mut conn = open_connection(&state.db_path)?;
    let tx = conn.transaction()?;
    let affected = tx.execute(
        "UPDATE members SET status = 'INACTIVE', updated_at = ?1 WHERE id = ?2",
        params![now_string(), member_id],
    )?;
    if affected == 0 {
        return Err(anyhow!("会员不存在"));
    }
    insert_operation_log(
        &tx,
        "管理员",
        "member",
        "disable",
        Some("member"),
        Some(&member_id.to_string()),
        None,
        "SUCCESS",
        None,
    )?;
    tx.commit()?;

    Ok(CommandResult {
        success: true,
        message: "会员已停用".to_string(),
        target_id: Some(member_id.to_string()),
    })
}

fn consume_create_impl(state: &AppState, payload: ConsumePayload) -> Result<CommandResult> {
    if payload.amount <= 0.0 {
        return Err(anyhow!("消费金额必须大于 0"));
    }

    let mut conn = open_connection(&state.db_path)?;
    let settings = load_settings(&conn, state)?;
    let rule_amount = settings.points_rule_amount.max(1) as f64;
    let tx = conn.transaction()?;
    let remark = normalize_optional_text(payload.remark.clone());

    let member = tx
        .query_row(
            "SELECT id, points_balance, total_spent, status FROM members WHERE id = ?1",
            params![payload.member_id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, f64>(2)?,
                    row.get::<_, String>(3)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| anyhow!("会员不存在"))?;

    if member.3 != "ACTIVE" {
        return Err(anyhow!("停用会员不能登记消费"));
    }

    let points_added = (payload.amount / rule_amount).floor() as i64;
    let record_no = make_business_no("C");
    let now = now_string();
    tx.execute(
        "INSERT INTO consumption_records(record_no, member_id, amount, points_added, operator_name, remark, created_at)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            record_no,
            payload.member_id,
            payload.amount,
            points_added,
            payload.operator_name.trim(),
            remark.clone(),
            now
        ],
    )?;

    let new_balance = member.1 + points_added;
    let total_spent = member.2 + payload.amount;
    tx.execute(
        "UPDATE members
         SET points_balance = ?1, total_spent = ?2, last_consume_at = ?3, updated_at = ?3
         WHERE id = ?4",
        params![new_balance, total_spent, now, payload.member_id],
    )?;

    tx.execute(
        "INSERT INTO points_ledger(member_id, change_type, points_delta, balance_after, source_type, source_id, operator_name, remark, created_at)
         VALUES(?1, 'ADD', ?2, ?3, 'CONSUME', ?4, ?5, ?6, ?7)",
        params![
            payload.member_id,
            points_added,
            new_balance,
            record_no,
            payload.operator_name.trim(),
            remark,
            now
        ],
    )?;

    let summary = serde_json::to_string(&json!({
        "memberId": payload.member_id,
        "amount": payload.amount,
        "pointsAdded": points_added
    }))?;
    insert_operation_log(
        &tx,
        payload.operator_name.trim(),
        "consume",
        "create",
        Some("consumption"),
        Some(&record_no),
        Some(&summary),
        "SUCCESS",
        None,
    )?;
    tx.commit()?;

    Ok(CommandResult {
        success: true,
        message: format!("消费登记成功，本次增加 {} 积分", points_added),
        target_id: Some(record_no),
    })
}

fn gift_list_impl(state: &AppState) -> Result<Vec<GiftRecord>> {
    let conn = open_connection(&state.db_path)?;
    let mut stmt = conn.prepare(
        "SELECT id, gift_name, points_cost, stock_qty, status, unique_per_member, remark, created_at, updated_at
         FROM gifts
         ORDER BY status = 'ACTIVE' DESC, updated_at DESC, id DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(GiftRecord {
            id: Some(row.get(0)?),
            gift_name: row.get(1)?,
            points_cost: row.get(2)?,
            stock_qty: row.get(3)?,
            status: Some(row.get(4)?),
            unique_per_member: Some(row.get::<_, i64>(5)? != 0),
            remark: row.get(6)?,
            created_at: Some(row.get(7)?),
            updated_at: Some(row.get(8)?),
        })
    })?;
    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}

fn gift_save_impl(state: &AppState, payload: GiftRecord) -> Result<CommandResult> {
    if payload.gift_name.trim().is_empty() {
        return Err(anyhow!("礼品名称不能为空"));
    }
    if payload.points_cost < 0 {
        return Err(anyhow!("礼品积分不能为负数"));
    }

    let mut conn = open_connection(&state.db_path)?;
    let tx = conn.transaction()?;
    let now = now_string();
    let gift_id = if let Some(id) = payload.id {
        tx.execute(
            "UPDATE gifts
             SET gift_name = ?1, points_cost = ?2, stock_qty = ?3, status = ?4, unique_per_member = ?5, remark = ?6, updated_at = ?7
             WHERE id = ?8",
            params![
                payload.gift_name.trim(),
                payload.points_cost,
                payload.stock_qty,
                payload.status.clone().unwrap_or_else(|| "ACTIVE".to_string()),
                if payload.unique_per_member.unwrap_or(false) { 1 } else { 0 },
                normalize_optional_text(payload.remark.clone()),
                now,
                id
            ],
        )?;
        id
    } else {
        tx.execute(
            "INSERT INTO gifts(gift_name, points_cost, stock_qty, status, unique_per_member, remark, created_at, updated_at)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
            params![
                payload.gift_name.trim(),
                payload.points_cost,
                payload.stock_qty,
                payload.status.clone().unwrap_or_else(|| "ACTIVE".to_string()),
                if payload.unique_per_member.unwrap_or(false) { 1 } else { 0 },
                normalize_optional_text(payload.remark.clone()),
                now
            ],
        )?;
        tx.last_insert_rowid()
    };

    insert_operation_log(
        &tx,
        "管理员",
        "gift",
        "save",
        Some("gift"),
        Some(&gift_id.to_string()),
        Some(&serde_json::to_string(&payload)?),
        "SUCCESS",
        None,
    )?;
    tx.commit()?;

    Ok(CommandResult {
        success: true,
        message: "礼品信息已保存".to_string(),
        target_id: Some(gift_id.to_string()),
    })
}

fn gift_redeem_impl(state: &AppState, payload: RedeemPayload) -> Result<CommandResult> {
    if payload.qty <= 0 {
        return Err(anyhow!("兑换数量必须大于 0"));
    }

    let mut conn = open_connection(&state.db_path)?;
    let tx = conn.transaction()?;

    let member = tx
        .query_row(
            "SELECT points_balance, status FROM members WHERE id = ?1",
            params![payload.member_id],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()?
        .ok_or_else(|| anyhow!("会员不存在"))?;
    if member.1 != "ACTIVE" {
        return Err(anyhow!("停用会员不能兑换礼品"));
    }

    let gift = tx
        .query_row(
            "SELECT gift_name, points_cost, status, unique_per_member FROM gifts WHERE id = ?1",
            params![payload.gift_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)? != 0,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| anyhow!("礼品不存在"))?;
    if gift.2 != "ACTIVE" {
        return Err(anyhow!("礼品已禁用，无法兑换"));
    }
    if gift.3 {
        if payload.qty != 1 {
            return Err(anyhow!("兑换一次礼品每位会员仅可兑换 1 件"));
        }
        let redeemed_count: i64 = tx.query_row(
            "SELECT COUNT(1) FROM gift_redemptions WHERE member_id = ?1 AND gift_id = ?2",
            params![payload.member_id, payload.gift_id],
            |row| row.get(0),
        )?;
        if redeemed_count > 0 {
            return Err(anyhow!("该礼品为兑换一次，当前会员已兑换过"));
        }
    }

    let points_used = gift.1 * payload.qty;
    if member.0 < points_used {
        return Err(anyhow!("会员积分不足"));
    }
    let redeem_no = make_business_no("R");
    let now = now_string();
    tx.execute(
        "INSERT INTO gift_redemptions(redeem_no, member_id, gift_id, gift_name_snapshot, qty, points_used, operator_name, remark, created_at)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            redeem_no,
            payload.member_id,
            payload.gift_id,
            gift.0,
            payload.qty,
            points_used,
            payload.operator_name.trim(),
            normalize_optional_text(payload.remark),
            now
        ],
    )?;

    let new_balance = member.0 - points_used;
    tx.execute(
        "UPDATE members SET points_balance = ?1, updated_at = ?2 WHERE id = ?3",
        params![new_balance, now, payload.member_id],
    )?;
    tx.execute(
        "INSERT INTO points_ledger(member_id, change_type, points_delta, balance_after, source_type, source_id, operator_name, remark, created_at)
         VALUES(?1, 'DEDUCT', ?2, ?3, 'GIFT', ?4, ?5, ?6, ?7)",
        params![
            payload.member_id,
            -points_used,
            new_balance,
            redeem_no,
            payload.operator_name.trim(),
            None::<String>,
            now
        ],
    )?;

    let summary = serde_json::to_string(&json!({
        "memberId": payload.member_id,
        "giftId": payload.gift_id,
        "qty": payload.qty,
        "pointsUsed": points_used
    }))?;
    insert_operation_log(
        &tx,
        payload.operator_name.trim(),
        "gift",
        "redeem",
        Some("giftRedemption"),
        Some(&redeem_no),
        Some(&summary),
        "SUCCESS",
        None,
    )?;
    tx.commit()?;

    Ok(CommandResult {
        success: true,
        message: format!("兑换成功，扣减 {} 积分", points_used),
        target_id: Some(redeem_no),
    })
}

fn settings_get_impl(state: &AppState) -> Result<SettingsData> {
    let conn = open_connection(&state.db_path)?;
    load_settings(&conn, state)
}

fn settings_save_impl(state: &AppState, payload: SettingsData) -> Result<CommandResult> {
    if payload.points_rule_amount <= 0 {
        return Err(anyhow!("积分规则必须大于 0"));
    }

    let mut conn = open_connection(&state.db_path)?;
    let tx = conn.transaction()?;
    persist_settings(&tx, state, &payload)?;
    let backup_dir = resolve_path(&state.root_dir, &payload.backup_path);
    fs::create_dir_all(&backup_dir)?;
    insert_operation_log(
        &tx,
        "管理员",
        "settings",
        "save",
        Some("settings"),
        None,
        Some(&serde_json::to_string(&payload)?),
        "SUCCESS",
        None,
    )?;
    tx.commit()?;

    Ok(CommandResult {
        success: true,
        message: "系统设置已保存".to_string(),
        target_id: None,
    })
}

fn report_dashboard_impl(state: &AppState, month: String) -> Result<DashboardData> {
    let conn = open_connection(&state.db_path)?;
    let month = if month.trim().is_empty() {
        current_month()
    } else {
        month
    };
    let (start, end) = month_range(&month)?;

    let member_total: i64 = conn.query_row("SELECT COUNT(1) FROM members", [], |row| row.get(0))?;
    let new_members_this_month: i64 = conn.query_row(
        "SELECT COUNT(1) FROM members WHERE created_at >= ?1 AND created_at < ?2",
        params![start, end],
        |row| row.get(0),
    )?;
    let consume_amount_this_month: f64 = conn.query_row(
        "SELECT COALESCE(SUM(amount), 0) FROM consumption_records WHERE created_at >= ?1 AND created_at < ?2",
        params![start, end],
        |row| row.get(0),
    )?;
    let points_added_this_month: i64 = conn.query_row(
        "SELECT COALESCE(SUM(CASE WHEN points_delta > 0 THEN points_delta ELSE 0 END), 0)
         FROM points_ledger WHERE created_at >= ?1 AND created_at < ?2",
        params![start, end],
        |row| row.get(0),
    )?;
    let redemption_count_this_month: i64 = conn.query_row(
        "SELECT COUNT(1) FROM gift_redemptions WHERE created_at >= ?1 AND created_at < ?2",
        params![start, end],
        |row| row.get(0),
    )?;

    let mut stmt = conn.prepare(
        "SELECT m.id, m.member_no, m.name, SUM(c.amount) AS total_amount
         FROM consumption_records c
         INNER JOIN members m ON m.id = c.member_id
         WHERE c.created_at >= ?1 AND c.created_at < ?2
         GROUP BY m.id, m.member_no, m.name
         ORDER BY total_amount DESC
         LIMIT 5",
    )?;
    let rows = stmt.query_map(params![start, end], |row| {
        Ok(MemberRankingItem {
            member_id: row.get(0)?,
            member_no: row.get(1)?,
            name: row.get(2)?,
            total_amount: row.get(3)?,
        })
    })?;
    let mut top_consumers = Vec::new();
    for row in rows {
        top_consumers.push(row?);
    }

    Ok(DashboardData {
        month,
        member_total,
        new_members_this_month,
        consume_amount_this_month,
        points_added_this_month,
        redemption_count_this_month,
        top_consumers,
    })
}

fn backup_create_impl(state: &AppState, target_path: Option<String>) -> Result<BackupResult> {
    let conn = open_connection(&state.db_path)?;
    let settings = load_settings(&conn, state)?;
    let target_dir = match normalize_optional_text(target_path) {
        Some(path) => resolve_path(&state.root_dir, &path),
        None => resolve_path(&state.root_dir, &settings.backup_path),
    };
    fs::create_dir_all(&target_dir)?;
    let backup_file = target_dir.join(format!(
        "club_backup_{}.db",
        chrono::Local::now().format("%Y%m%d_%H%M%S")
    ));
    let sql = format!(
        "VACUUM INTO '{}';",
        backup_file.to_string_lossy().replace('\'', "''")
    );
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
    conn.execute_batch(&sql)?;
    insert_operation_log(
        &conn,
        "管理员",
        "backup",
        "create",
        Some("backup"),
        Some(&backup_file.to_string_lossy()),
        None,
        "SUCCESS",
        None,
    )?;
    append_text_log(
        &state.log_dir,
        "app.log",
        &format!("数据库备份成功: {}", backup_file.display()),
    )?;

    Ok(BackupResult {
        file_path: backup_file.to_string_lossy().to_string(),
        created_at: now_string(),
        message: "备份已创建".to_string(),
    })
}

fn backup_restore_impl(state: &AppState, file_path: String) -> Result<CommandResult> {
    let source = PathBuf::from(file_path.trim());
    if !source.exists() {
        return Err(anyhow!("备份文件不存在"));
    }
    let validation = open_connection(&source)?;
    let _: String = validation.query_row(
        "SELECT name FROM sqlite_master WHERE type = 'table' LIMIT 1",
        [],
        |row| row.get(0),
    )?;
    drop(validation);

    let protect_backup = backup_create_impl(state, None)?;
    let wal_path = PathBuf::from(format!("{}-wal", state.db_path.to_string_lossy()));
    let shm_path = PathBuf::from(format!("{}-shm", state.db_path.to_string_lossy()));
    if wal_path.exists() {
        fs::remove_file(&wal_path)?;
    }
    if shm_path.exists() {
        fs::remove_file(&shm_path)?;
    }
    fs::copy(&source, &state.db_path)?;

    let conn = open_connection(&state.db_path)?;
    insert_operation_log(
        &conn,
        "管理员",
        "backup",
        "restore",
        Some("backup"),
        Some(&source.to_string_lossy()),
        Some(&serde_json::to_string(&json!({ "protectBackup": protect_backup.file_path }))?),
        "SUCCESS",
        None,
    )?;
    append_text_log(
        &state.log_dir,
        "app.log",
        &format!("数据库恢复成功: {}", source.display()),
    )?;

    Ok(CommandResult {
        success: true,
        message: format!("数据已恢复，恢复前备份保存在 {}", protect_backup.file_path),
        target_id: Some(source.to_string_lossy().to_string()),
    })
}

fn migration_precheck_impl(
    state: &AppState,
    source_path: String,
    import_scope: String,
) -> Result<MigrationPrecheckResult> {
    let source = PathBuf::from(source_path.trim());
    if !source.exists() {
        return Err(anyhow!("导入文件不存在"));
    }
    let content = fs::read_to_string(&source)
        .with_context(|| format!("读取导入文件失败: {}", source.display()))?;
    let source_data: ImportBundle = serde_json::from_str(&content)
        .with_context(|| "当前仅支持导入 JSON 格式的迁移文件")?;

    let source_file_hash = file_sha256(&source)?;
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    validate_import_bundle(&source_data, &mut errors, &mut warnings);

    let import_mode = if source_data.consumptions.is_empty() && source_data.redemptions.is_empty() {
        "BALANCE_ONLY"
    } else {
        "FULL_HISTORY"
    }
    .to_string();

    let member_keys = source_data
        .members
        .iter()
        .map(|item| item.legacy_pk.clone())
        .collect::<Vec<_>>()
        .join("|");
    let consumption_keys = source_data
        .consumptions
        .iter()
        .map(|item| item.legacy_pk.clone())
        .collect::<Vec<_>>()
        .join("|");
    let redemption_keys = source_data
        .redemptions
        .iter()
        .map(|item| item.legacy_pk.clone())
        .collect::<Vec<_>>()
        .join("|");
    let digest_source = format!(
        "{}|{}|{}|{}|{}|{}|{}|{}",
        source_file_hash,
        import_scope,
        source_data.members.len(),
        source_data.consumptions.len(),
        source_data.redemptions.len(),
        source_data.gifts.len(),
        member_keys,
        sha256_hex(format!("{consumption_keys}|{redemption_keys}").as_bytes())
    );
    let batch_fingerprint = sha256_hex(digest_source.as_bytes());

    let conn = open_connection(&state.db_path)?;
    let duplicate_batch = conn
        .query_row(
            "SELECT COUNT(1) FROM migration_batches WHERE batch_fingerprint = ?1 AND status = 'SUCCESS'",
            params![batch_fingerprint],
            |row| row.get::<_, i64>(0),
        )?
        > 0;
    if duplicate_batch {
        warnings.push("系统检测到同指纹批次已成功导入，本次预检结果仅供查看".to_string());
    }

    let batch_no = make_business_no("BATCH");
    let precheck = MigrationPrecheckResult {
        batch_no: batch_no.clone(),
        source_file: source.to_string_lossy().to_string(),
        source_version: source_data.source_version.clone(),
        import_scope,
        batch_fingerprint,
        import_mode,
        member_count: source_data.members.len() as i64,
        consumption_count: source_data.consumptions.len() as i64,
        redemption_count: source_data.redemptions.len() as i64,
        gift_count: source_data.gifts.len() as i64,
        duplicate_batch,
        can_execute: errors.is_empty() && !duplicate_batch,
        warnings,
        errors: errors.clone(),
    };

    let cache = MigrationPrecheckCache {
        precheck: precheck.clone(),
        source_file_hash,
        source: source_data,
    };
    let cache_file = precheck_cache_file(state, &batch_no);
    ensure_parent_dir(&cache_file)?;
    fs::write(&cache_file, serde_json::to_string_pretty(&cache)?)?;

    Ok(precheck)
}

fn migration_execute_impl(state: &AppState, batch_no: String) -> Result<CommandResult> {
    let cache_file = precheck_cache_file(state, &batch_no);
    if !cache_file.exists() {
        return Err(anyhow!("未找到该批次的预检缓存，请重新执行预检"));
    }
    let content = fs::read_to_string(&cache_file)?;
    let cache: MigrationPrecheckCache = serde_json::from_str(&content)?;
    if !cache.precheck.can_execute {
        return Err(anyhow!("当前批次存在预检错误或已重复导入，不能执行正式导入"));
    }

    let source_file = PathBuf::from(&cache.precheck.source_file);
    if source_file.exists() {
        let current_hash = file_sha256(&source_file)?;
        if current_hash != cache.source_file_hash {
            return Err(anyhow!("源文件在预检后发生变化，请重新预检"));
        }
    }

    let mut conn = open_connection(&state.db_path)?;
    let already_success = conn
        .query_row(
            "SELECT COUNT(1) FROM migration_batches WHERE batch_fingerprint = ?1 AND status = 'SUCCESS'",
            params![cache.precheck.batch_fingerprint],
            |row| row.get::<_, i64>(0),
        )?
        > 0;
    if already_success {
        return Err(anyhow!("该数据批次已成功导入，系统已阻止重复入库"));
    }

    conn.execute(
        "INSERT INTO migration_batches(batch_no, source_file, source_file_hash, source_version, import_scope, batch_fingerprint, status, created_at)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, 'PROCESSING', ?7)
         ON CONFLICT(batch_no) DO UPDATE SET status = 'PROCESSING', error_message = NULL, completed_at = NULL",
        params![
            cache.precheck.batch_no,
            cache.precheck.source_file,
            cache.source_file_hash,
            cache.precheck.source_version,
            cache.precheck.import_scope,
            cache.precheck.batch_fingerprint,
            now_string()
        ],
    )?;

    let tx = conn.transaction()?;
    tx.execute("DELETE FROM migration_errors WHERE batch_no = ?1", params![cache.precheck.batch_no])?;

    let mut ctx = ImportContext::default();
    let mut next_no = next_member_no(&tx)?;
    let mut success_count = 0_i64;
    let mut failed_count = 0_i64;

    if let Some(settings) = &cache.source.settings {
        apply_import_settings(&tx, settings)?;
        success_count += 1;
    }

    for gift in &cache.source.gifts {
        if let Err(err) = import_gift(&tx, &cache.precheck.batch_no, &mut ctx, gift) {
            record_import_error(&tx, &cache.precheck.batch_no, "GIFT", gift.legacy_pk.clone(), &err, gift)?;
            failed_count += 1;
        } else {
            success_count += 1;
        }
    }

    for member in &cache.source.members {
        if let Err(err) = import_member(
            &tx,
            &cache.precheck.batch_no,
            &cache.precheck.import_mode,
            &mut ctx,
            member,
            &mut next_no,
        ) {
            record_import_error(&tx, &cache.precheck.batch_no, "MEMBER", Some(member.legacy_pk.clone()), &err, member)?;
            failed_count += 1;
        } else {
            success_count += 1;
        }
    }

    let settings = load_settings(&tx, state)?;
    for consume in &cache.source.consumptions {
        if let Err(err) = import_consumption(
            &tx,
            &cache.precheck.batch_no,
            &ctx,
            consume,
            settings.points_rule_amount,
        ) {
            record_import_error(
                &tx,
                &cache.precheck.batch_no,
                "CONSUMPTION",
                Some(consume.legacy_pk.clone()),
                &err,
                consume,
            )?;
            failed_count += 1;
        } else {
            success_count += 1;
        }
    }

    for redemption in &cache.source.redemptions {
        if let Err(err) = import_redemption(&tx, &cache.precheck.batch_no, &ctx, redemption) {
            record_import_error(
                &tx,
                &cache.precheck.batch_no,
                "REDEMPTION",
                Some(redemption.legacy_pk.clone()),
                &err,
                redemption,
            )?;
            failed_count += 1;
        } else {
            success_count += 1;
        }
    }

    if cache.precheck.import_mode == "FULL_HISTORY" {
        reconcile_import_balances(&tx, &ctx, &cache.source.members)?;
    }

    let final_status = if failed_count == 0 { "SUCCESS" } else { "FAILED" };
    let final_error = if failed_count == 0 {
        None
    } else {
        Some("部分数据导入失败，请查看导入报告".to_string())
    };

    tx.execute(
        "UPDATE migration_batches
         SET status = ?1, success_count = ?2, failed_count = ?3, error_message = ?4, completed_at = ?5
         WHERE batch_no = ?6",
        params![
            final_status,
            success_count,
            failed_count,
            final_error,
            now_string(),
            cache.precheck.batch_no
        ],
    )?;
    insert_operation_log(
        &tx,
        "管理员",
        "migration",
        "execute",
        Some("migrationBatch"),
        Some(&cache.precheck.batch_no),
        Some(&serde_json::to_string(&json!({
            "successCount": success_count,
            "failedCount": failed_count,
            "importMode": cache.precheck.import_mode
        }))?),
        final_status,
        final_error.as_deref(),
    )?;
    tx.commit()?;

    Ok(CommandResult {
        success: failed_count == 0,
        message: if failed_count == 0 {
            format!("导入完成，共成功处理 {} 条数据", success_count)
        } else {
            format!("导入完成，成功 {} 条，失败 {} 条", success_count, failed_count)
        },
        target_id: Some(cache.precheck.batch_no),
    })
}

fn migration_get_report_impl(state: &AppState, batch_no: String) -> Result<MigrationReport> {
    let conn = open_connection(&state.db_path)?;
    let batch = conn
        .query_row(
            "SELECT batch_no, source_file, source_version, import_scope, status, success_count, failed_count, error_message, created_at, completed_at
             FROM migration_batches WHERE batch_no = ?1",
            params![batch_no],
            |row| {
                Ok(MigrationBatchInfo {
                    batch_no: row.get(0)?,
                    source_file: row.get(1)?,
                    source_version: row.get(2)?,
                    import_scope: row.get(3)?,
                    status: row.get(4)?,
                    success_count: row.get(5)?,
                    failed_count: row.get(6)?,
                    error_message: row.get(7)?,
                    created_at: row.get(8)?,
                    completed_at: row.get(9)?,
                })
            },
        )
        .optional()?
        .ok_or_else(|| anyhow!("导入批次不存在"))?;

    let mut stmt = conn.prepare(
        "SELECT entity_type, legacy_pk, error_code, error_message FROM migration_errors WHERE batch_no = ?1 ORDER BY id ASC",
    )?;
    let rows = stmt.query_map(params![batch_no], |row| {
        Ok(MigrationErrorItem {
            entity_type: row.get(0)?,
            legacy_pk: row.get(1)?,
            error_code: row.get(2)?,
            error_message: row.get(3)?,
        })
    })?;
    let mut errors = Vec::new();
    for row in rows {
        errors.push(row?);
    }

    Ok(MigrationReport { batch, errors })
}

fn operation_logs_query_impl(state: &AppState, filter: OperationLogFilter) -> Result<Vec<OperationLogItem>> {
    let conn = open_connection(&state.db_path)?;
    let keyword = filter.keyword.unwrap_or_default().trim().to_string();
    let keyword_like = format!("%{}%", keyword);
    let module = filter.module_name.unwrap_or_default();
    let limit = filter.limit.unwrap_or(80).clamp(1, 500);

    let mut stmt = conn.prepare(
        "SELECT id, operator_name, module_name, action_name, target_type, target_id, request_summary, result_status, error_message, created_at
         FROM operation_logs
         WHERE (?1 = '' OR module_name = ?1)
           AND (
             ?2 = ''
             OR operator_name LIKE ?3
             OR action_name LIKE ?3
             OR IFNULL(target_id, '') LIKE ?3
             OR IFNULL(request_summary, '') LIKE ?3
             OR IFNULL(error_message, '') LIKE ?3
           )
         ORDER BY id DESC
         LIMIT ?4",
    )?;
    let rows = stmt.query_map(params![module, keyword, keyword_like, limit], |row| {
        Ok(OperationLogItem {
            id: row.get(0)?,
            operator_name: row.get(1)?,
            module_name: row.get(2)?,
            action_name: row.get(3)?,
            target_type: row.get(4)?,
            target_id: row.get(5)?,
            request_summary: row.get(6)?,
            result_status: row.get(7)?,
            error_message: row.get(8)?,
            created_at: row.get(9)?,
        })
    })?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}

fn query_recent_consumptions(conn: &Connection, member_id: i64) -> Result<Vec<ConsumptionRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, record_no, member_id, amount, points_added, operator_name, remark, legacy_record_id, created_at
         FROM consumption_records
         WHERE member_id = ?1
         ORDER BY created_at DESC, id DESC
         LIMIT 8",
    )?;
    let rows = stmt.query_map(params![member_id], map_consumption_record)?;
    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}

fn query_recent_redemptions(conn: &Connection, member_id: i64) -> Result<Vec<GiftRedemption>> {
    let mut stmt = conn.prepare(
        "SELECT id, redeem_no, member_id, gift_id, gift_name_snapshot, qty, points_used, operator_name, remark, legacy_redemption_id, created_at
         FROM gift_redemptions
         WHERE member_id = ?1
         ORDER BY created_at DESC, id DESC
         LIMIT 8",
    )?;
    let rows = stmt.query_map(params![member_id], map_redemption_record)?;
    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}

fn query_redeemed_gift_ids(conn: &Connection, member_id: i64) -> Result<Vec<i64>> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT gift_id
         FROM gift_redemptions
         WHERE member_id = ?1 AND gift_id IS NOT NULL",
    )?;
    let rows = stmt.query_map(params![member_id], |row| row.get(0))?;
    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}

fn query_recent_ledger(conn: &Connection, member_id: i64) -> Result<Vec<PointsLedgerEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, member_id, change_type, points_delta, balance_after, source_type, source_id, operator_name, remark, created_at
         FROM points_ledger
         WHERE member_id = ?1
         ORDER BY created_at DESC, id DESC
         LIMIT 12",
    )?;
    let rows = stmt.query_map(params![member_id], |row| {
        Ok(PointsLedgerEntry {
            id: row.get(0)?,
            member_id: row.get(1)?,
            change_type: row.get(2)?,
            points_delta: row.get(3)?,
            balance_after: row.get(4)?,
            source_type: row.get(5)?,
            source_id: row.get(6)?,
            operator_name: row.get(7)?,
            remark: row.get(8)?,
            created_at: row.get(9)?,
        })
    })?;
    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}

fn map_member_record(row: &Row<'_>) -> rusqlite::Result<MemberRecord> {
    Ok(MemberRecord {
        id: row.get(0)?,
        member_no: row.get(1)?,
        name: row.get(2)?,
        gender: row.get(3)?,
        birth_month: row.get(4)?,
        birth_day: row.get(5)?,
        mobile: row.get(6)?,
        name_pinyin: row.get(7)?,
        name_initials: row.get(8)?,
        points_balance: row.get(9)?,
        total_spent: row.get(10)?,
        last_consume_at: row.get(11)?,
        status: row.get(12)?,
        remark: row.get(13)?,
        legacy_member_id: row.get(14)?,
        created_at: row.get(15)?,
        updated_at: row.get(16)?,
    })
}

fn map_consumption_record(row: &Row<'_>) -> rusqlite::Result<ConsumptionRecord> {
    Ok(ConsumptionRecord {
        id: row.get(0)?,
        record_no: row.get(1)?,
        member_id: row.get(2)?,
        amount: row.get(3)?,
        points_added: row.get(4)?,
        operator_name: row.get(5)?,
        remark: row.get(6)?,
        legacy_record_id: row.get(7)?,
        created_at: row.get(8)?,
    })
}

fn map_redemption_record(row: &Row<'_>) -> rusqlite::Result<GiftRedemption> {
    Ok(GiftRedemption {
        id: row.get(0)?,
        redeem_no: row.get(1)?,
        member_id: row.get(2)?,
        gift_id: row.get(3)?,
        gift_name_snapshot: row.get(4)?,
        qty: row.get(5)?,
        points_used: row.get(6)?,
        operator_name: row.get(7)?,
        remark: row.get(8)?,
        legacy_redemption_id: row.get(9)?,
        created_at: row.get(10)?,
    })
}

fn next_member_no(conn: &Connection) -> Result<i64> {
    Ok(conn.query_row(
        "SELECT COALESCE(MAX(member_no), 100000) + 1 FROM members",
        [],
        |row| row.get(0),
    )?)
}

fn map_command_error(state: &AppState, module: &str, err: anyhow::Error) -> String {
    let message = err.to_string();
    let _ = append_text_log(
        &state.log_dir,
        "error.log",
        &format!("{}: {}", module, message),
    );
    message
}

fn validate_import_bundle(
    source: &ImportBundle,
    errors: &mut Vec<MigrationErrorItem>,
    warnings: &mut Vec<String>,
) {
    let mut member_keys = HashSet::new();
    let mut member_nos = HashSet::new();
    for member in &source.members {
        if member.legacy_pk.trim().is_empty() {
            errors.push(MigrationErrorItem {
                entity_type: "MEMBER".to_string(),
                legacy_pk: None,
                error_code: "EMPTY_LEGACY_PK".to_string(),
                error_message: "会员 legacyPk 不能为空".to_string(),
            });
        }
        if !member_keys.insert(member.legacy_pk.clone()) {
            errors.push(MigrationErrorItem {
                entity_type: "MEMBER".to_string(),
                legacy_pk: Some(member.legacy_pk.clone()),
                error_code: "DUPLICATE_LEGACY_PK".to_string(),
                error_message: "会员 legacyPk 在导入文件中重复".to_string(),
            });
        }
        if member.name.trim().is_empty() {
            errors.push(MigrationErrorItem {
                entity_type: "MEMBER".to_string(),
                legacy_pk: Some(member.legacy_pk.clone()),
                error_code: "EMPTY_NAME".to_string(),
                error_message: "会员姓名不能为空".to_string(),
            });
        }
        if let Some(no) = member.member_no {
            if no <= 0 {
                errors.push(MigrationErrorItem {
                    entity_type: "MEMBER".to_string(),
                    legacy_pk: Some(member.legacy_pk.clone()),
                    error_code: "INVALID_MEMBER_NO".to_string(),
                    error_message: "会员编号必须大于 0".to_string(),
                });
            }
            if !member_nos.insert(no) {
                errors.push(MigrationErrorItem {
                    entity_type: "MEMBER".to_string(),
                    legacy_pk: Some(member.legacy_pk.clone()),
                    error_code: "DUPLICATE_MEMBER_NO".to_string(),
                    error_message: "会员编号在导入文件中重复".to_string(),
                });
            }
        }
        if member.points_balance.unwrap_or(0) < 0 {
            errors.push(MigrationErrorItem {
                entity_type: "MEMBER".to_string(),
                legacy_pk: Some(member.legacy_pk.clone()),
                error_code: "NEGATIVE_POINTS".to_string(),
                error_message: "会员当前积分不能为负数".to_string(),
            });
        }
        if let Some(mobile) = &member.mobile {
            if !mobile.trim().is_empty() && mobile.trim().len() < 7 {
                warnings.push(format!("会员 {} 的手机号长度偏短，请人工确认", member.legacy_pk));
            }
        }
    }

    let member_key_set = source
        .members
        .iter()
        .map(|item| item.legacy_pk.clone())
        .collect::<HashSet<_>>();
    let mut consume_keys = HashSet::new();
    for consume in &source.consumptions {
        if !consume_keys.insert(consume.legacy_pk.clone()) {
            errors.push(MigrationErrorItem {
                entity_type: "CONSUMPTION".to_string(),
                legacy_pk: Some(consume.legacy_pk.clone()),
                error_code: "DUPLICATE_LEGACY_PK".to_string(),
                error_message: "消费记录 legacyPk 在导入文件中重复".to_string(),
            });
        }
        if consume.amount <= 0.0 {
            errors.push(MigrationErrorItem {
                entity_type: "CONSUMPTION".to_string(),
                legacy_pk: Some(consume.legacy_pk.clone()),
                error_code: "INVALID_AMOUNT".to_string(),
                error_message: "消费金额必须大于 0".to_string(),
            });
        }
        if let Some(points_added) = consume.points_added {
            if points_added < 0 {
                errors.push(MigrationErrorItem {
                    entity_type: "CONSUMPTION".to_string(),
                    legacy_pk: Some(consume.legacy_pk.clone()),
                    error_code: "INVALID_POINTS".to_string(),
                    error_message: "消费积分不能为负数".to_string(),
                });
            }
        }
        if !member_key_set.contains(&consume.member_legacy_pk) {
            errors.push(MigrationErrorItem {
                entity_type: "CONSUMPTION".to_string(),
                legacy_pk: Some(consume.legacy_pk.clone()),
                error_code: "MISSING_MEMBER".to_string(),
                error_message: "消费记录关联的会员不存在".to_string(),
            });
        }
    }

    let mut gift_keys = HashSet::new();
    for gift in &source.gifts {
        let key = gift_identity(gift);
        if !gift_keys.insert(key.clone()) {
            errors.push(MigrationErrorItem {
                entity_type: "GIFT".to_string(),
                legacy_pk: Some(key),
                error_code: "DUPLICATE_GIFT".to_string(),
                error_message: "礼品主键或名称在导入文件中重复".to_string(),
            });
        }
        if gift.gift_name.trim().is_empty() {
            errors.push(MigrationErrorItem {
                entity_type: "GIFT".to_string(),
                legacy_pk: gift.legacy_pk.clone(),
                error_code: "EMPTY_NAME".to_string(),
                error_message: "礼品名称不能为空".to_string(),
            });
        }
    }

    let mut redemption_keys = HashSet::new();
    for redemption in &source.redemptions {
        if !redemption_keys.insert(redemption.legacy_pk.clone()) {
            errors.push(MigrationErrorItem {
                entity_type: "REDEMPTION".to_string(),
                legacy_pk: Some(redemption.legacy_pk.clone()),
                error_code: "DUPLICATE_LEGACY_PK".to_string(),
                error_message: "兑换记录 legacyPk 在导入文件中重复".to_string(),
            });
        }
        if !member_key_set.contains(&redemption.member_legacy_pk) {
            errors.push(MigrationErrorItem {
                entity_type: "REDEMPTION".to_string(),
                legacy_pk: Some(redemption.legacy_pk.clone()),
                error_code: "MISSING_MEMBER".to_string(),
                error_message: "兑换记录关联的会员不存在".to_string(),
            });
        }
        if redemption.qty.unwrap_or(1) <= 0 {
            errors.push(MigrationErrorItem {
                entity_type: "REDEMPTION".to_string(),
                legacy_pk: Some(redemption.legacy_pk.clone()),
                error_code: "INVALID_QTY".to_string(),
                error_message: "兑换数量必须大于 0".to_string(),
            });
        }
        if redemption.points_used.unwrap_or(0) < 0 {
            errors.push(MigrationErrorItem {
                entity_type: "REDEMPTION".to_string(),
                legacy_pk: Some(redemption.legacy_pk.clone()),
                error_code: "INVALID_POINTS".to_string(),
                error_message: "兑换积分不能为负数".to_string(),
            });
        }
    }
}

fn precheck_cache_file(state: &AppState, batch_no: &str) -> PathBuf {
    state.import_dir.join(format!("{batch_no}.json"))
}

fn apply_import_settings(conn: &Connection, settings: &ImportSettings) -> Result<()> {
    if let Some(store_name) = settings.store_name.as_ref().and_then(|s| normalize_optional_text(Some(s.clone()))) {
        upsert_setting(conn, "store_name", &store_name)?;
    }
    if let Some(points_rule_amount) = settings.points_rule_amount {
        upsert_setting(conn, "points_rule_amount", &points_rule_amount.to_string())?;
    }
    if let Some(legacy_jpdj) = settings.legacy_jpdj.as_ref().and_then(|s| normalize_optional_text(Some(s.clone()))) {
        upsert_setting(conn, "legacy_jpdj", &legacy_jpdj)?;
    }
    if let Some(default_operator) = settings.default_operator.as_ref().and_then(|s| normalize_optional_text(Some(s.clone()))) {
        upsert_setting(conn, "default_operator", &default_operator)?;
    }
    Ok(())
}

fn import_gift(tx: &Transaction<'_>, batch_no: &str, ctx: &mut ImportContext, gift: &ImportGift) -> Result<()> {
    let legacy_pk = gift_identity(gift);
    if let Some(target_id) = lookup_mapping(tx, "GIFT", &legacy_pk)? {
        ctx.gift_map.insert(legacy_pk, target_id.parse()?);
        return Ok(());
    }

    tx.execute(
        "INSERT INTO gifts(gift_name, points_cost, stock_qty, status, unique_per_member, remark, created_at, updated_at)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
        params![
            gift.gift_name.trim(),
            gift.points_cost,
            gift.stock_qty.unwrap_or(0),
            gift.status.clone().unwrap_or_else(|| "ACTIVE".to_string()),
            if gift.unique_per_member.unwrap_or(false) { 1 } else { 0 },
            normalize_optional_text(gift.remark.clone()),
            now_string()
        ],
    )?;
    let gift_id = tx.last_insert_rowid();
    tx.execute(
        "INSERT INTO migration_entity_map(batch_no, entity_type, legacy_pk, target_id, created_at)
         VALUES(?1, 'GIFT', ?2, ?3, ?4)",
        params![batch_no, legacy_pk, gift_id.to_string(), now_string()],
    )?;
    ctx.gift_map.insert(gift_identity(gift), gift_id);
    Ok(())
}

fn import_member(
    tx: &Transaction<'_>,
    batch_no: &str,
    import_mode: &str,
    ctx: &mut ImportContext,
    member: &ImportMember,
    next_no: &mut i64,
) -> Result<()> {
    if let Some(target_id) = lookup_mapping(tx, "MEMBER", &member.legacy_pk)? {
        ctx.member_map.insert(member.legacy_pk.clone(), target_id.parse()?);
        return Ok(());
    }

    let member_no = match member.member_no {
        Some(value) if value > 0 => value,
        _ => {
            let generated = *next_no;
            *next_no += 1;
            generated
        }
    };

    let duplicate: i64 = tx.query_row(
        "SELECT COUNT(1) FROM members WHERE member_no = ?1",
        params![member_no],
        |row| row.get(0),
    )?;
    if duplicate > 0 {
        return Err(anyhow!("会员编号 {} 已存在", member_no));
    }

    let (name_pinyin, name_initials) = generate_pinyin(member.name.trim());
    let balance = if import_mode == "BALANCE_ONLY" {
        member.points_balance.unwrap_or(0)
    } else {
        0
    };
    let total_spent = if import_mode == "BALANCE_ONLY" {
        member.total_spent.unwrap_or(0.0)
    } else {
        0.0
    };
    let last_consume_at = if import_mode == "BALANCE_ONLY" {
        member.last_consume_at.clone()
    } else {
        None
    };
    let now = now_string();

    tx.execute(
        "INSERT INTO members(member_no, name, gender, birth_month, birth_day, mobile, name_pinyin, name_initials,
                             points_balance, total_spent, last_consume_at, status, remark, legacy_member_id, created_at, updated_at)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 'ACTIVE', ?12, ?13, ?14, ?14)",
        params![
            member_no,
            member.name.trim(),
            normalize_optional_text(member.gender.clone()),
            normalize_optional_text(member.birth_month.clone()),
            normalize_optional_text(member.birth_day.clone()),
            normalize_optional_text(member.mobile.clone()),
            name_pinyin,
            name_initials,
            balance,
            total_spent,
            last_consume_at,
            normalize_optional_text(member.remark.clone()),
            member.legacy_pk,
            now
        ],
    )?;
    let member_id = tx.last_insert_rowid();
    tx.execute(
        "INSERT INTO migration_entity_map(batch_no, entity_type, legacy_pk, target_id, created_at)
         VALUES(?1, 'MEMBER', ?2, ?3, ?4)",
        params![batch_no, member.legacy_pk, member_id.to_string(), now_string()],
    )?;
    ctx.member_map.insert(member.legacy_pk.clone(), member_id);

    if import_mode == "BALANCE_ONLY" && balance > 0 {
        tx.execute(
            "INSERT INTO points_ledger(member_id, change_type, points_delta, balance_after, source_type, source_id, operator_name, remark, created_at)
             VALUES(?1, 'ADD', ?2, ?3, 'IMPORT', ?4, '系统导入', '导入期初积分', ?5)",
            params![member_id, balance, balance, member.legacy_pk, now_string()],
        )?;
    }
    Ok(())
}

fn import_consumption(
    tx: &Transaction<'_>,
    batch_no: &str,
    ctx: &ImportContext,
    consume: &ImportConsumption,
    points_rule_amount: i64,
) -> Result<()> {
    if lookup_mapping(tx, "CONSUMPTION", &consume.legacy_pk)?.is_some() {
        return Ok(());
    }

    let member_id = *ctx
        .member_map
        .get(&consume.member_legacy_pk)
        .ok_or_else(|| anyhow!("找不到关联会员映射"))?;
    let current = tx.query_row(
        "SELECT points_balance, total_spent FROM members WHERE id = ?1",
        params![member_id],
        |row| Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?)),
    )?;
    let points_added = consume
        .points_added
        .unwrap_or_else(|| (consume.amount / points_rule_amount.max(1) as f64).floor() as i64);
    let new_balance = current.0 + points_added;
    let new_total_spent = current.1 + consume.amount;
    let record_no = make_business_no("IC");
    let created_at = consume.created_at.clone().unwrap_or_else(now_string);
    let operator_name = consume
        .operator_name
        .clone()
        .unwrap_or_else(|| "系统导入".to_string());

    tx.execute(
        "INSERT INTO consumption_records(record_no, member_id, amount, points_added, operator_name, remark, legacy_record_id, created_at)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            record_no,
            member_id,
            consume.amount,
            points_added,
            operator_name,
            normalize_optional_text(consume.remark.clone()),
            consume.legacy_pk,
            created_at
        ],
    )?;
    tx.execute(
        "UPDATE members SET points_balance = ?1, total_spent = ?2, last_consume_at = ?3, updated_at = ?3 WHERE id = ?4",
        params![new_balance, new_total_spent, created_at, member_id],
    )?;
    tx.execute(
        "INSERT INTO points_ledger(member_id, change_type, points_delta, balance_after, source_type, source_id, operator_name, remark, created_at)
         VALUES(?1, 'ADD', ?2, ?3, 'CONSUME', ?4, ?5, '历史消费导入', ?6)",
        params![member_id, points_added, new_balance, record_no, "系统导入", created_at],
    )?;
    tx.execute(
        "INSERT INTO migration_entity_map(batch_no, entity_type, legacy_pk, target_id, created_at)
         VALUES(?1, 'CONSUMPTION', ?2, ?3, ?4)",
        params![batch_no, consume.legacy_pk, record_no, now_string()],
    )?;
    Ok(())
}

fn import_redemption(
    tx: &Transaction<'_>,
    batch_no: &str,
    ctx: &ImportContext,
    redemption: &ImportRedemption,
) -> Result<()> {
    if lookup_mapping(tx, "REDEMPTION", &redemption.legacy_pk)?.is_some() {
        return Ok(());
    }

    let member_id = *ctx
        .member_map
        .get(&redemption.member_legacy_pk)
        .ok_or_else(|| anyhow!("找不到关联会员映射"))?;
    let current_balance: i64 = tx.query_row(
        "SELECT points_balance FROM members WHERE id = ?1",
        params![member_id],
        |row| row.get(0),
    )?;

    let gift_id = redemption
        .gift_legacy_pk
        .as_ref()
        .and_then(|legacy_pk| ctx.gift_map.get(legacy_pk).copied());
    let gift_name = if let Some(id) = gift_id {
        tx.query_row("SELECT gift_name FROM gifts WHERE id = ?1", params![id], |row| row.get(0))
            .optional()?
            .or_else(|| redemption.gift_name.clone())
            .unwrap_or_else(|| "历史礼品".to_string())
    } else {
        redemption
            .gift_name
            .clone()
            .unwrap_or_else(|| "历史礼品".to_string())
    };
    let qty = redemption.qty.unwrap_or(1);
    let points_used = match redemption.points_used {
        Some(points) => points,
        None => {
            if let Some(id) = gift_id {
                let unit_cost: i64 = tx.query_row(
                    "SELECT points_cost FROM gifts WHERE id = ?1",
                    params![id],
                    |row| row.get(0),
                )?;
                unit_cost * qty
            } else {
                return Err(anyhow!("兑换记录缺少 pointsUsed 且无法根据礼品推导积分"));
            }
        }
    };
    let new_balance = current_balance - points_used;
    let redeem_no = make_business_no("IR");
    let created_at = redemption.created_at.clone().unwrap_or_else(now_string);
    let operator_name = redemption
        .operator_name
        .clone()
        .unwrap_or_else(|| "系统导入".to_string());

    tx.execute(
        "INSERT INTO gift_redemptions(redeem_no, member_id, gift_id, gift_name_snapshot, qty, points_used, operator_name, remark, legacy_redemption_id, created_at)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            redeem_no,
            member_id,
            gift_id,
            gift_name,
            qty,
            points_used,
            operator_name,
            normalize_optional_text(redemption.remark.clone()),
            redemption.legacy_pk,
            created_at
        ],
    )?;
    tx.execute(
        "UPDATE members SET points_balance = ?1, updated_at = ?2 WHERE id = ?3",
        params![new_balance, created_at, member_id],
    )?;
    tx.execute(
        "INSERT INTO points_ledger(member_id, change_type, points_delta, balance_after, source_type, source_id, operator_name, remark, created_at)
         VALUES(?1, 'DEDUCT', ?2, ?3, 'GIFT', ?4, ?5, '历史兑换导入', ?6)",
        params![member_id, -points_used, new_balance, redeem_no, "系统导入", created_at],
    )?;
    tx.execute(
        "INSERT INTO migration_entity_map(batch_no, entity_type, legacy_pk, target_id, created_at)
         VALUES(?1, 'REDEMPTION', ?2, ?3, ?4)",
        params![batch_no, redemption.legacy_pk, redeem_no, now_string()],
    )?;
    Ok(())
}

fn reconcile_import_balances(tx: &Transaction<'_>, ctx: &ImportContext, members: &[ImportMember]) -> Result<()> {
    for member in members {
        let Some(member_id) = ctx.member_map.get(&member.legacy_pk) else {
            continue;
        };
        let Some(target_balance) = member.points_balance else {
            continue;
        };
        let current_balance: i64 = tx.query_row(
            "SELECT points_balance FROM members WHERE id = ?1",
            params![member_id],
            |row| row.get(0),
        )?;
        let delta = target_balance - current_balance;
        if delta == 0 {
            continue;
        }
        tx.execute(
            "UPDATE members SET points_balance = ?1, updated_at = ?2 WHERE id = ?3",
            params![target_balance, now_string(), member_id],
        )?;
        tx.execute(
            "INSERT INTO points_ledger(member_id, change_type, points_delta, balance_after, source_type, source_id, operator_name, remark, created_at)
             VALUES(?1, 'ADJUST', ?2, ?3, 'IMPORT', ?4, '系统导入', '导入后余额校正', ?5)",
            params![member_id, delta, target_balance, member.legacy_pk, now_string()],
        )?;
    }
    Ok(())
}

fn lookup_mapping(conn: &Connection, entity_type: &str, legacy_pk: &str) -> Result<Option<String>> {
    Ok(conn
        .query_row(
            "SELECT target_id FROM migration_entity_map WHERE entity_type = ?1 AND legacy_pk = ?2",
            params![entity_type, legacy_pk],
            |row| row.get(0),
        )
        .optional()?)
}

fn record_import_error<T: serde::Serialize>(
    tx: &Transaction<'_>,
    batch_no: &str,
    entity_type: &str,
    legacy_pk: Option<String>,
    err: &anyhow::Error,
    payload: &T,
) -> Result<()> {
    tx.execute(
        "INSERT INTO migration_errors(batch_no, entity_type, legacy_pk, error_code, error_message, raw_payload, created_at)
         VALUES(?1, ?2, ?3, 'IMPORT_ERROR', ?4, ?5, ?6)",
        params![
            batch_no,
            entity_type,
            legacy_pk,
            err.to_string(),
            serde_json::to_string(payload)?,
            now_string()
        ],
    )?;
    Ok(())
}

fn gift_identity(gift: &ImportGift) -> String {
    gift.legacy_pk
        .clone()
        .unwrap_or_else(|| format!("gift::{}", gift.gift_name.trim()))
}
