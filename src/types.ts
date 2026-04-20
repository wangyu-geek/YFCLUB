export interface PagedResult<T> {
  items: T[];
  total: number;
  pageNo: number;
  pageSize: number;
}

export interface MemberSummary {
  id: number;
  memberNo: number;
  name: string;
  mobile: string | null;
  pointsBalance: number;
  totalSpent: number;
  lastConsumeAt: string | null;
  status: string;
}

export interface MemberDetailData {
  member: MemberRecord;
  recentConsumptions: ConsumptionRecord[];
  recentRedemptions: GiftRedemption[];
  recentLedger: PointsLedgerEntry[];
}

export interface MemberRecord {
  id: number;
  memberNo: number;
  name: string;
  gender: string | null;
  birthMonth: string | null;
  birthDay: string | null;
  mobile: string | null;
  namePinyin: string | null;
  nameInitials: string | null;
  pointsBalance: number;
  totalSpent: number;
  lastConsumeAt: string | null;
  status: string;
  remark: string | null;
  legacyMemberId: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface MemberFormPayload {
  memberNo?: number | null;
  name: string;
  gender?: string | null;
  birthMonth?: string | null;
  birthDay?: string | null;
  mobile?: string | null;
  remark?: string | null;
}

export interface CommandResult {
  success: boolean;
  message: string;
  targetId?: string | null;
}

export interface ConsumptionRecord {
  id: number;
  recordNo: string;
  memberId: number;
  amount: number;
  pointsAdded: number;
  operatorName: string;
  remark: string | null;
  legacyRecordId: string | null;
  createdAt: string;
}

export interface PointsLedgerEntry {
  id: number;
  memberId: number;
  changeType: string;
  pointsDelta: number;
  balanceAfter: number;
  sourceType: string;
  sourceId: string | null;
  operatorName: string;
  remark: string | null;
  createdAt: string;
}

export interface ConsumePayload {
  memberId: number;
  amount: number;
  operatorName: string;
  remark?: string | null;
}

export interface GiftRecord {
  id?: number | null;
  giftName: string;
  pointsCost: number;
  stockQty: number;
  status?: string;
  remark?: string | null;
  createdAt?: string;
  updatedAt?: string;
}

export interface GiftRedemption {
  id: number;
  redeemNo: string;
  memberId: number;
  giftId: number | null;
  giftNameSnapshot: string;
  qty: number;
  pointsUsed: number;
  operatorName: string;
  remark: string | null;
  legacyRedemptionId: string | null;
  createdAt: string;
}

export interface RedeemPayload {
  memberId: number;
  giftId: number;
  qty: number;
  operatorName: string;
  remark?: string | null;
}

export interface SettingsData {
  storeName: string;
  dbPath: string;
  backupPath: string;
  autoBackupEnabled: boolean;
  pointsRuleAmount: number;
  legacyJpdj: string;
  defaultOperator: string;
}

export interface DashboardData {
  month: string;
  memberTotal: number;
  newMembersThisMonth: number;
  consumeAmountThisMonth: number;
  pointsAddedThisMonth: number;
  redemptionCountThisMonth: number;
  topConsumers: MemberRankingItem[];
}

export interface MemberRankingItem {
  memberId: number;
  memberNo: number;
  name: string;
  totalAmount: number;
}

export interface OperationLogItem {
  id: number;
  operatorName: string;
  moduleName: string;
  actionName: string;
  targetType: string | null;
  targetId: string | null;
  requestSummary: string | null;
  resultStatus: string;
  errorMessage: string | null;
  createdAt: string;
}

export interface OperationLogFilter {
  keyword?: string | null;
  moduleName?: string | null;
  limit?: number | null;
}

export interface BackupResult {
  filePath: string;
  createdAt: string;
  message: string;
}

export interface MigrationErrorItem {
  entityType: string;
  legacyPk: string | null;
  errorCode: string;
  errorMessage: string;
}

export interface MigrationPrecheckResult {
  batchNo: string;
  sourceFile: string;
  sourceVersion: string | null;
  importScope: string;
  batchFingerprint: string;
  importMode: string;
  memberCount: number;
  consumptionCount: number;
  redemptionCount: number;
  giftCount: number;
  duplicateBatch: boolean;
  canExecute: boolean;
  warnings: string[];
  errors: MigrationErrorItem[];
}

export interface MigrationBatchInfo {
  batchNo: string;
  sourceFile: string;
  sourceVersion: string | null;
  importScope: string;
  status: string;
  successCount: number;
  failedCount: number;
  errorMessage: string | null;
  createdAt: string;
  completedAt: string | null;
}

export interface MigrationReport {
  batch: MigrationBatchInfo;
  errors: MigrationErrorItem[];
}
