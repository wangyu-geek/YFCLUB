use crate::{
    models::SettingsData,
    state::{load_or_create_config, path_to_config_value, resolve_path, save_config, settings_from_config, AppConfig, AppState},
    utils::now_string,
};
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::Path;

const CURRENT_SCHEMA_VERSION: i64 = 2;

pub fn open_connection(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)
        .with_context(|| format!("打开数据库失败: {}", db_path.display()))?;
    conn.execute_batch(
        "
        PRAGMA foreign_keys = ON;
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA busy_timeout = 5000;
        ",
    )?;
    Ok(conn)
}

pub fn initialize_database(state: &AppState) -> Result<()> {
    let config = load_or_create_config(&state.root_dir, &state.config_path)?;
    let resolved_db_path = resolve_path(&state.root_dir, &config.db_path);
    if let Some(parent) = resolved_db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let conn = open_connection(&resolved_db_path)?;
    conn.execute_batch(include_str!("../migrations/001_init.sql"))?;
    ensure_schema_version(&conn, 1, "initial schema")?;
    apply_gift_unique_redemption_migration(&conn)?;

    seed_settings(&conn, &config, state)?;
    seed_operator(&conn, &config)?;
    Ok(())
}

fn apply_gift_unique_redemption_migration(conn: &Connection) -> Result<()> {
    if has_schema_version(conn, CURRENT_SCHEMA_VERSION)? {
        return Ok(());
    }

    if !table_has_column(conn, "gifts", "unique_per_member")? {
        conn.execute(
            "ALTER TABLE gifts ADD COLUMN unique_per_member INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_redemptions_member_gift ON gift_redemptions(member_id, gift_id)",
        [],
    )?;
    ensure_schema_version(conn, CURRENT_SCHEMA_VERSION, "gift unique redemption")?;
    Ok(())
}

fn has_schema_version(conn: &Connection, version: i64) -> Result<bool> {
    let applied: i64 = conn.query_row(
        "SELECT COUNT(1) FROM schema_versions WHERE version_no = ?1",
        params![version],
        |row| row.get(0),
    )?;
    Ok(applied > 0)
}

fn ensure_schema_version(conn: &Connection, version: i64, description: &str) -> Result<()> {
    if has_schema_version(conn, version)? {
        return Ok(());
    }

    conn.execute(
        "INSERT INTO schema_versions(version_no, description, applied_at) VALUES(?1, ?2, ?3)",
        params![version, description, now_string()],
    )?;
    Ok(())
}

fn table_has_column(conn: &Connection, table_name: &str, column_name: &str) -> Result<bool> {
    let pragma = format!("PRAGMA table_info({table_name})");
    let mut stmt = conn.prepare(&pragma)?;
    let mut rows = stmt.query([])?;

    while let Some(row) = rows.next()? {
        let existing_name: String = row.get(1)?;
        if existing_name == column_name {
            return Ok(true);
        }
    }

    Ok(false)
}

fn seed_settings(conn: &Connection, config: &AppConfig, state: &AppState) -> Result<()> {
    let settings = settings_from_config(config, state);
    upsert_setting(conn, "store_name", &settings.store_name)?;
    upsert_setting(
        conn,
        "points_rule_amount",
        &settings.points_rule_amount.to_string(),
    )?;
    upsert_setting(
        conn,
        "auto_backup_enabled",
        if settings.auto_backup_enabled { "true" } else { "false" },
    )?;
    upsert_setting(conn, "backup_path", &settings.backup_path)?;
    upsert_setting(conn, "legacy_jpdj", &settings.legacy_jpdj)?;
    upsert_setting(conn, "default_operator", &settings.default_operator)?;
    Ok(())
}

fn seed_operator(conn: &Connection, config: &AppConfig) -> Result<()> {
    let count: i64 = conn.query_row("SELECT COUNT(1) FROM operators", [], |row| row.get(0))?;
    if count == 0 {
        let now = now_string();
        conn.execute(
            "INSERT INTO operators(login_name, display_name, password_hash, role_code, status, created_at, updated_at)
             VALUES(?1, ?2, NULL, 'ADMIN', 'ACTIVE', ?3, ?3)",
            params!["admin", config.default_operator, now],
        )?;
    }
    Ok(())
}

pub fn upsert_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO sys_settings(setting_key, setting_value, updated_at)
         VALUES(?1, ?2, ?3)
         ON CONFLICT(setting_key) DO UPDATE SET setting_value = excluded.setting_value, updated_at = excluded.updated_at",
        params![key, value, now_string()],
    )?;
    Ok(())
}

pub fn load_settings(conn: &Connection, state: &AppState) -> Result<SettingsData> {
    let config = load_or_create_config(&state.root_dir, &state.config_path)?;
    let mut settings = settings_from_config(&config, state);
    let mut stmt = conn.prepare("SELECT setting_key, setting_value FROM sys_settings")?;
    let pairs = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))?;

    for pair in pairs {
        let (key, value) = pair?;
        match key.as_str() {
            "store_name" => settings.store_name = value,
            "points_rule_amount" => settings.points_rule_amount = value.parse().unwrap_or(settings.points_rule_amount),
            "auto_backup_enabled" => settings.auto_backup_enabled = value == "true",
            "backup_path" => settings.backup_path = value,
            "legacy_jpdj" => settings.legacy_jpdj = value,
            "default_operator" => settings.default_operator = value,
            _ => {}
        }
    }

    Ok(SettingsData {
        backup_path: resolve_path(&state.root_dir, &settings.backup_path)
            .to_string_lossy()
            .to_string(),
        ..settings
    })
}

pub fn persist_settings(conn: &Connection, state: &AppState, payload: &SettingsData) -> Result<()> {
    upsert_setting(conn, "store_name", &payload.store_name)?;
    upsert_setting(
        conn,
        "points_rule_amount",
        &payload.points_rule_amount.to_string(),
    )?;
    upsert_setting(
        conn,
        "auto_backup_enabled",
        if payload.auto_backup_enabled { "true" } else { "false" },
    )?;
    upsert_setting(conn, "backup_path", &payload.backup_path)?;
    upsert_setting(conn, "legacy_jpdj", &payload.legacy_jpdj)?;
    upsert_setting(conn, "default_operator", &payload.default_operator)?;

    let config = AppConfig {
        store_name: payload.store_name.clone(),
        db_path: path_to_config_value(&state.root_dir, &state.db_path),
        backup_path: path_to_config_value(&state.root_dir, Path::new(&payload.backup_path)),
        auto_backup_enabled: payload.auto_backup_enabled,
        points_rule_amount: payload.points_rule_amount,
        legacy: crate::state::LegacyConfig {
            jpdj: payload.legacy_jpdj.clone(),
        },
        default_operator: payload.default_operator.clone(),
    };
    save_config(&state.config_path, &config)?;
    Ok(())
}

pub fn insert_operation_log(
    conn: &Connection,
    operator_name: &str,
    module_name: &str,
    action_name: &str,
    target_type: Option<&str>,
    target_id: Option<&str>,
    request_summary: Option<&str>,
    result_status: &str,
    error_message: Option<&str>,
) -> Result<()> {
    conn.execute(
        "INSERT INTO operation_logs(
            operator_name, module_name, action_name, target_type, target_id, request_summary, result_status, error_message, created_at
         ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            operator_name,
            module_name,
            action_name,
            target_type,
            target_id,
            request_summary,
            result_status,
            error_message,
            now_string()
        ],
    )?;
    Ok(())
}
