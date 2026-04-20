use anyhow::{Context, Result};
use chrono::{Datelike, Local, NaiveDate};
use pinyin::ToPinyin;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};
use uuid::Uuid;

pub fn now_string() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn current_month() -> String {
    Local::now().format("%Y-%m").to_string()
}

pub fn month_range(month: &str) -> Result<(String, String)> {
    let start = NaiveDate::parse_from_str(&format!("{month}-01"), "%Y-%m-%d")
        .with_context(|| format!("月份格式不正确: {month}"))?;
    let next = if start.month() == 12 {
        NaiveDate::from_ymd_opt(start.year() + 1, 1, 1).context("无法计算下月时间")?
    } else {
        NaiveDate::from_ymd_opt(start.year(), start.month() + 1, 1).context("无法计算下月时间")?
    };
    Ok((
        format!("{} 00:00:00", start.format("%Y-%m-%d")),
        format!("{} 00:00:00", next.format("%Y-%m-%d")),
    ))
}

pub fn make_business_no(prefix: &str) -> String {
    format!(
        "{}{}{}",
        prefix,
        Local::now().format("%Y%m%d%H%M%S"),
        Uuid::new_v4().simple()
    )
}

pub fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|item| {
        let trimmed = item.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

pub fn generate_pinyin(name: &str) -> (String, String) {
    let mut full = String::new();
    let mut initials = String::new();

    for ch in name.chars() {
        if let Some(pinyin) = ch.to_pinyin() {
            let plain = pinyin.plain().to_lowercase();
            if let Some(first) = plain.chars().next() {
                initials.push(first);
            }
            full.push_str(&plain);
        } else if ch.is_ascii_alphanumeric() {
            let lower = ch.to_ascii_lowercase();
            full.push(lower);
            initials.push(lower);
        }
    }

    if full.is_empty() {
        full = name.to_lowercase();
        initials = name
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .take(8)
            .collect::<String>()
            .to_lowercase();
    }

    (full, initials)
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn file_sha256(path: &Path) -> Result<String> {
    let bytes = fs::read(path)
        .with_context(|| format!("读取文件失败: {}", path.display()))?;
    Ok(sha256_hex(&bytes))
}

pub fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

pub fn append_text_log(log_dir: &Path, file_name: &str, message: &str) -> Result<()> {
    let path = PathBuf::from(log_dir).join(file_name);
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&path)
        .with_context(|| format!("打开日志文件失败: {}", path.display()))?;
    writeln!(file, "[{}] {}", now_string(), message)?;
    Ok(())
}
