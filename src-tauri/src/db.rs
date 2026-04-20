use crate::{
    models::SettingsData,
    state::{load_or_create_config, resolve_path, save_config, settings_from_config, AppConfig, AppState},
    utils::now_string,
};
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::Path;

const CURRENT_SCHEMA_VERSION: i64 = 1;

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

    let applied: i64 = conn.query_row(
        "SELECT COUNT(1) FROM schema_versions WHERE version_no = ?1",
        params![CURRENT_SCHEMA_VERSION],
        |row| row.get(0),
    )?;

    if applied == 0 {
        conn.execute(
            "INSERT INTO schema_versions(version_no, description, applied_at) VALUES(?1, ?2, ?3)",
            params![CURRENT_SCHEMA_VERSION, "initial schema", now_string()],
        )?;
    }

    seed_settings(&conn, &config, state)?;
    seed_operator(&conn, &config)?;
    Ok(())
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
        db_path: state.db_path.to_string_lossy().to_string(),
        backup_path: payload.backup_path.clone(),
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
