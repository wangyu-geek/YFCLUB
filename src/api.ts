import { invoke } from "@tauri-apps/api/core";
import type {
  BackupResult,
  CommandResult,
  ConsumePayload,
  DashboardData,
  GiftRecord,
  MemberDetailData,
  MemberFormPayload,
  MigrationPrecheckResult,
  MigrationReport,
  OperationLogFilter,
  OperationLogItem,
  PagedResult,
  RedeemPayload,
  SettingsData,
  MemberSummary
} from "./types";

export function memberSearch(keyword: string, pageNo = 1, pageSize = 20) {
  return invoke<PagedResult<MemberSummary>>("member_search", {
    keyword,
    pageNo,
    pageSize
  });
}

export function memberGetDetail(memberId: number) {
  return invoke<MemberDetailData>("member_get_detail", { memberId });
}

export function memberCreate(payload: MemberFormPayload) {
  return invoke<CommandResult>("member_create", { payload });
}

export function memberUpdate(memberId: number, payload: MemberFormPayload) {
  return invoke<CommandResult>("member_update", { memberId, payload });
}

export function memberDisable(memberId: number) {
  return invoke<CommandResult>("member_disable", { memberId });
}

export function consumeCreate(payload: ConsumePayload) {
  return invoke<CommandResult>("consume_create", { payload });
}

export function giftList() {
  return invoke<GiftRecord[]>("gift_list");
}

export function giftSave(payload: GiftRecord) {
  return invoke<CommandResult>("gift_save", { payload });
}

export function giftRedeem(payload: RedeemPayload) {
  return invoke<CommandResult>("gift_redeem", { payload });
}

export function settingsGet() {
  return invoke<SettingsData>("settings_get");
}

export function settingsSave(payload: SettingsData) {
  return invoke<CommandResult>("settings_save", { payload });
}

export function reportDashboard(month: string) {
  return invoke<DashboardData>("report_dashboard", { month });
}

export function backupCreate(targetPath?: string | null) {
  return invoke<BackupResult>("backup_create", { targetPath });
}

export function backupRestore(filePath: string) {
  return invoke<CommandResult>("backup_restore", { filePath });
}

export function migrationPrecheck(sourcePath: string, importScope: string) {
  return invoke<MigrationPrecheckResult>("migration_precheck", {
    sourcePath,
    importScope
  });
}

export function migrationExecute(batchNo: string) {
  return invoke<CommandResult>("migration_execute", { batchNo });
}

export function migrationGetReport(batchNo: string) {
  return invoke<MigrationReport>("migration_get_report", { batchNo });
}

export function operationLogsQuery(filter: OperationLogFilter) {
  return invoke<OperationLogItem[]>("operation_logs_query", { filter });
}
