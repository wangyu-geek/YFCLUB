use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PagedResult<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page_no: i64,
    pub page_size: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberSummary {
    pub id: i64,
    pub member_no: i64,
    pub name: String,
    pub mobile: Option<String>,
    pub points_balance: i64,
    pub total_spent: f64,
    pub last_consume_at: Option<String>,
    pub status: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberRecord {
    pub id: i64,
    pub member_no: i64,
    pub name: String,
    pub gender: Option<String>,
    pub birth_month: Option<String>,
    pub birth_day: Option<String>,
    pub mobile: Option<String>,
    pub name_pinyin: Option<String>,
    pub name_initials: Option<String>,
    pub points_balance: i64,
    pub total_spent: f64,
    pub last_consume_at: Option<String>,
    pub status: String,
    pub remark: Option<String>,
    pub legacy_member_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsumptionRecord {
    pub id: i64,
    pub record_no: String,
    pub member_id: i64,
    pub amount: f64,
    pub points_added: i64,
    pub operator_name: String,
    pub remark: Option<String>,
    pub legacy_record_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PointsLedgerEntry {
    pub id: i64,
    pub member_id: i64,
    pub change_type: String,
    pub points_delta: i64,
    pub balance_after: i64,
    pub source_type: String,
    pub source_id: Option<String>,
    pub operator_name: String,
    pub remark: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GiftRecord {
    pub id: Option<i64>,
    pub gift_name: String,
    pub points_cost: i64,
    pub stock_qty: i64,
    pub status: Option<String>,
    pub unique_per_member: Option<bool>,
    pub remark: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GiftRedemption {
    pub id: i64,
    pub redeem_no: String,
    pub member_id: i64,
    pub gift_id: Option<i64>,
    pub gift_name_snapshot: String,
    pub qty: i64,
    pub points_used: i64,
    pub operator_name: String,
    pub remark: Option<String>,
    pub legacy_redemption_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberDetailData {
    pub member: MemberRecord,
    pub recent_consumptions: Vec<ConsumptionRecord>,
    pub recent_redemptions: Vec<GiftRedemption>,
    pub recent_ledger: Vec<PointsLedgerEntry>,
    pub redeemed_gift_ids: Vec<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberFormPayload {
    pub member_no: Option<i64>,
    pub name: String,
    pub gender: Option<String>,
    pub birth_month: Option<String>,
    pub birth_day: Option<String>,
    pub mobile: Option<String>,
    pub remark: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsumePayload {
    pub member_id: i64,
    pub amount: f64,
    pub operator_name: String,
    pub remark: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedeemPayload {
    pub member_id: i64,
    pub gift_id: i64,
    pub qty: i64,
    pub operator_name: String,
    pub remark: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsData {
    pub store_name: String,
    pub db_path: String,
    pub backup_path: String,
    pub auto_backup_enabled: bool,
    pub points_rule_amount: i64,
    pub legacy_jpdj: String,
    pub default_operator: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardData {
    pub month: String,
    pub member_total: i64,
    pub new_members_this_month: i64,
    pub consume_amount_this_month: f64,
    pub points_added_this_month: i64,
    pub redemption_count_this_month: i64,
    pub top_consumers: Vec<MemberRankingItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberRankingItem {
    pub member_id: i64,
    pub member_no: i64,
    pub name: String,
    pub total_amount: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationLogItem {
    pub id: i64,
    pub operator_name: String,
    pub module_name: String,
    pub action_name: String,
    pub target_type: Option<String>,
    pub target_id: Option<String>,
    pub request_summary: Option<String>,
    pub result_status: String,
    pub error_message: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationLogFilter {
    pub keyword: Option<String>,
    pub module_name: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
    pub target_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupResult {
    pub file_path: String,
    pub created_at: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationErrorItem {
    pub entity_type: String,
    pub legacy_pk: Option<String>,
    pub error_code: String,
    pub error_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationPrecheckResult {
    pub batch_no: String,
    pub source_file: String,
    pub source_version: Option<String>,
    pub import_scope: String,
    pub batch_fingerprint: String,
    pub import_mode: String,
    pub member_count: i64,
    pub consumption_count: i64,
    pub redemption_count: i64,
    pub gift_count: i64,
    pub duplicate_batch: bool,
    pub can_execute: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<MigrationErrorItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationBatchInfo {
    pub batch_no: String,
    pub source_file: String,
    pub source_version: Option<String>,
    pub import_scope: String,
    pub status: String,
    pub success_count: i64,
    pub failed_count: i64,
    pub error_message: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationReport {
    pub batch: MigrationBatchInfo,
    pub errors: Vec<MigrationErrorItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ImportBundle {
    pub source_version: Option<String>,
    #[serde(default)]
    pub members: Vec<ImportMember>,
    #[serde(default)]
    pub consumptions: Vec<ImportConsumption>,
    #[serde(default)]
    pub gifts: Vec<ImportGift>,
    #[serde(default)]
    pub redemptions: Vec<ImportRedemption>,
    #[serde(default)]
    pub settings: Option<ImportSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportMember {
    #[serde(alias = "legacyPk", alias = "legacy_pk")]
    pub legacy_pk: String,
    #[serde(alias = "memberNo", alias = "member_no")]
    pub member_no: Option<i64>,
    pub name: String,
    pub gender: Option<String>,
    pub birth_month: Option<String>,
    pub birth_day: Option<String>,
    pub mobile: Option<String>,
    pub points_balance: Option<i64>,
    pub total_spent: Option<f64>,
    pub last_consume_at: Option<String>,
    pub remark: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportConsumption {
    #[serde(alias = "legacyPk", alias = "legacy_pk")]
    pub legacy_pk: String,
    #[serde(alias = "memberLegacyPk", alias = "member_legacy_pk")]
    pub member_legacy_pk: String,
    pub amount: f64,
    pub points_added: Option<i64>,
    pub operator_name: Option<String>,
    pub remark: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportGift {
    #[serde(alias = "legacyPk", alias = "legacy_pk")]
    pub legacy_pk: Option<String>,
    pub gift_name: String,
    pub points_cost: i64,
    pub stock_qty: Option<i64>,
    pub status: Option<String>,
    #[serde(alias = "unique_per_member")]
    pub unique_per_member: Option<bool>,
    pub remark: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportRedemption {
    #[serde(alias = "legacyPk", alias = "legacy_pk")]
    pub legacy_pk: String,
    #[serde(alias = "memberLegacyPk", alias = "member_legacy_pk")]
    pub member_legacy_pk: String,
    #[serde(alias = "giftLegacyPk", alias = "gift_legacy_pk")]
    pub gift_legacy_pk: Option<String>,
    pub gift_name: Option<String>,
    pub qty: Option<i64>,
    pub points_used: Option<i64>,
    pub operator_name: Option<String>,
    pub remark: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ImportSettings {
    pub store_name: Option<String>,
    pub points_rule_amount: Option<i64>,
    pub legacy_jpdj: Option<String>,
    pub default_operator: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationPrecheckCache {
    pub precheck: MigrationPrecheckResult,
    pub source_file_hash: String,
    pub source: ImportBundle,
}

#[derive(Debug, Default)]
pub struct ImportContext {
    pub member_map: HashMap<String, i64>,
    pub gift_map: HashMap<String, i64>,
}
