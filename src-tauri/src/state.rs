use crate::models::{SettingsData};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub const DEFAULT_DB_PATH: &str = "data/club.db";
pub const DEFAULT_BACKUP_PATH: &str = "data/backup";

#[derive(Debug, Clone)]
pub struct AppState {
    pub root_dir: PathBuf,
    pub db_path: PathBuf,
    pub import_dir: PathBuf,
    pub config_path: PathBuf,
    pub log_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub store_name: String,
    pub db_path: String,
    pub backup_path: String,
    pub auto_backup_enabled: bool,
    pub points_rule_amount: i64,
    pub legacy: LegacyConfig,
    pub default_operator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyConfig {
    pub jpdj: String,
}

impl AppState {
    pub fn initialize() -> Result<Self> {
        let root_dir = resolve_root_dir()?;
        let data_dir = root_dir.join("data");
        let backup_dir = data_dir.join("backup");
        let import_dir = data_dir.join("import");
        let config_dir = root_dir.join("config");
        let log_dir = root_dir.join("logs");
        let config_path = config_dir.join("app.json");

        fs::create_dir_all(&backup_dir)?;
        fs::create_dir_all(&import_dir)?;
        fs::create_dir_all(&config_dir)?;
        fs::create_dir_all(&log_dir)?;

        let config = load_or_create_config(&root_dir, &config_path)?;
        let db_path = resolve_path(&root_dir, &config.db_path);

        Ok(Self {
            root_dir,
            db_path,
            import_dir,
            config_path,
            log_dir,
        })
    }
}

pub fn load_or_create_config(root_dir: &Path, config_path: &Path) -> Result<AppConfig> {
    if config_path.exists() {
        let content = fs::read_to_string(config_path)
            .with_context(|| format!("读取配置文件失败: {}", config_path.display()))?;
        let mut config: AppConfig = serde_json::from_str(&content)
            .with_context(|| format!("解析配置文件失败: {}", config_path.display()))?;
        if normalize_config_paths(&mut config) {
            save_config(config_path, &config)?;
        }
        return Ok(config);
    }

    let legacy_jpdj = read_legacy_jpdj(root_dir).unwrap_or_else(|| "10".to_string());
    let config = AppConfig {
        store_name: "永丰文体".to_string(),
        db_path: DEFAULT_DB_PATH.to_string(),
        backup_path: DEFAULT_BACKUP_PATH.to_string(),
        auto_backup_enabled: true,
        points_rule_amount: legacy_jpdj.parse().unwrap_or(10),
        legacy: LegacyConfig {
            jpdj: legacy_jpdj,
        },
        default_operator: "管理员".to_string(),
    };
    save_config(config_path, &config)?;
    Ok(config)
}

pub fn save_config(config_path: &Path, config: &AppConfig) -> Result<()> {
    let json = serde_json::to_string_pretty(config)?;
    fs::write(config_path, json)
        .with_context(|| format!("写入配置文件失败: {}", config_path.display()))
}

pub fn path_to_config_value(root_dir: &Path, path: &Path) -> String {
    let candidate = if path.is_absolute() {
        path.strip_prefix(root_dir).unwrap_or(path).to_path_buf()
    } else {
        path.to_path_buf()
    };

    candidate.to_string_lossy().replace('\\', "/")
}

pub fn resolve_path(root_dir: &Path, raw: &str) -> PathBuf {
    let candidate = PathBuf::from(raw);
    if candidate.is_absolute() {
        candidate
    } else {
        root_dir.join(candidate)
    }
}

pub fn settings_from_config(config: &AppConfig, state: &AppState) -> SettingsData {
    SettingsData {
        store_name: config.store_name.clone(),
        db_path: state.db_path.to_string_lossy().to_string(),
        backup_path: resolve_path(&state.root_dir, &config.backup_path)
            .to_string_lossy()
            .to_string(),
        auto_backup_enabled: config.auto_backup_enabled,
        points_rule_amount: config.points_rule_amount,
        legacy_jpdj: config.legacy.jpdj.clone(),
        default_operator: config.default_operator.clone(),
    }
}

fn resolve_root_dir() -> Result<PathBuf> {
    if let Ok(root) = std::env::var("MEMBER_CLUB_ROOT") {
        return Ok(PathBuf::from(root));
    }

    #[cfg(debug_assertions)]
    {
        let cargo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let project_root = cargo_dir
            .parent()
            .context("无法解析项目根目录")?
            .to_path_buf();
        return Ok(project_root);
    }

    #[cfg(not(debug_assertions))]
    {
        let executable = std::env::current_exe().context("无法定位程序路径")?;
        let root = executable.parent().context("无法解析程序所在目录")?;
        Ok(root.to_path_buf())
    }
}

fn read_legacy_jpdj(root_dir: &Path) -> Option<String> {
    let sysset_path = root_dir.join("SysSet.xml");
    let content = fs::read_to_string(sysset_path).ok()?;
    let start = content.find("<JPDJ>")?;
    let end = content.find("</JPDJ>")?;
    if end <= start + 6 {
        return None;
    }
    Some(content[start + 6..end].trim().to_string())
}

fn normalize_config_paths(config: &mut AppConfig) -> bool {
    let normalized_db_path = normalize_portable_path(&config.db_path, DEFAULT_DB_PATH);
    let normalized_backup_path = normalize_portable_path(&config.backup_path, DEFAULT_BACKUP_PATH);

    let changed = normalized_db_path != config.db_path || normalized_backup_path != config.backup_path;
    if changed {
        config.db_path = normalized_db_path;
        config.backup_path = normalized_backup_path;
    }

    changed
}

fn normalize_portable_path(raw: &str, portable_relative: &str) -> String {
    let candidate = PathBuf::from(raw);
    if candidate.is_absolute() && candidate.ends_with(Path::new(portable_relative)) {
        return portable_relative.to_string();
    }

    if candidate.is_relative() {
        return candidate.to_string_lossy().replace('\\', "/");
    }

    raw.to_string()
}
