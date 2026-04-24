import type { LocalizedOperationLogItem, OperationLogItem } from "./types";

const operationModuleLabels: Record<string, string> = {
  member: "会员",
  consume: "消费",
  gift: "礼品",
  settings: "系统设置",
  backup: "备份",
  migration: "数据迁移"
};

const operationActionLabels: Record<string, string> = {
  create: "新增",
  update: "更新",
  disable: "停用",
  save: "保存",
  redeem: "兑换",
  restore: "恢复",
  execute: "执行"
};

const operationTargetLabels: Record<string, string> = {
  member: "会员",
  consumption: "消费记录",
  gift: "礼品",
  giftRedemption: "礼品兑换",
  settings: "系统设置",
  backup: "备份",
  migrationBatch: "迁移批次"
};

const codeLabels: Record<string, string> = {
  ACTIVE: "启用",
  INACTIVE: "停用",
  SUCCESS: "成功",
  FAILED: "失败",
  BALANCE_ONLY: "仅余额",
  FULL_HISTORY: "完整历史",
  MEMBER: "会员",
  CONSUMPTION: "消费记录",
  REDEMPTION: "兑换记录",
  GIFT: "礼品",
  SETTINGS: "系统设置",
  BACKUP: "备份",
  MIGRATION: "数据迁移"
};

const summaryKeyLabels: Record<string, string> = {
  memberId: "会员 ID",
  memberNo: "会员编号",
  name: "姓名",
  amount: "消费金额",
  pointsAdded: "新增积分",
  giftName: "礼品名称",
  giftId: "礼品 ID",
  pointsCost: "所需积分",
  status: "状态",
  uniquePerMember: "唯一兑换",
  remark: "备注",
  qty: "数量",
  pointsUsed: "扣减积分",
  protectBackup: "恢复前备份",
  successCount: "成功数量",
  failedCount: "失败数量",
  importMode: "导入模式"
};

const moneyKeys = new Set(["amount"]);
const pointsKeys = new Set(["pointsAdded", "pointsUsed"]);

function translateValue(value: string | null | undefined, labels: Record<string, string>) {
  if (!value) {
    return "";
  }
  return labels[value] ?? value;
}

function formatSummaryValue(key: string, value: unknown): string {
  if (typeof value === "number") {
    if (moneyKeys.has(key)) {
      return `${value.toFixed(2)} 元`;
    }
    if (pointsKeys.has(key)) {
      return `${value} 分`;
    }
    return String(value);
  }

  if (typeof value === "boolean") {
    return value ? "是" : "否";
  }

  if (typeof value === "string") {
    return formatCodeLabel(value);
  }

  if (value == null) {
    return "无";
  }

  if (Array.isArray(value)) {
    return value.map((item) => formatSummaryValue(key, item)).join("、");
  }

  if (typeof value === "object") {
    return JSON.stringify(value);
  }

  return String(value);
}

export function formatCodeLabel(value: string | null | undefined) {
  if (!value) {
    return "无";
  }
  return codeLabels[value] ?? value;
}

export function formatGiftStatusLabel(value: string | null | undefined) {
  if (!value) {
    return "无";
  }
  if (value === "ACTIVE") {
    return "启用";
  }
  if (value === "INACTIVE") {
    return "禁用";
  }
  return formatCodeLabel(value);
}

export function formatOperationLogSummary(summary: string | null | undefined, errorMessage?: string | null) {
  if (summary?.trim()) {
    try {
      const parsed: unknown = JSON.parse(summary);
      if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
        const parts = Object.entries(parsed as Record<string, unknown>).map(([key, value]) => {
          const label = summaryKeyLabels[key] ?? key;
          return `${label}：${formatSummaryValue(key, value)}`;
        });
        if (parts.length) {
          return parts.join(" / ");
        }
      }
    } catch {
      return summary;
    }

    return summary;
  }

  return errorMessage?.trim() || "无附加信息";
}

export function localizeOperationLogItem(item: OperationLogItem): LocalizedOperationLogItem {
  return {
    ...item,
    moduleLabel: translateValue(item.moduleName, operationModuleLabels),
    actionLabel: translateValue(item.actionName, operationActionLabels),
    targetTypeLabel: item.targetType ? translateValue(item.targetType, operationTargetLabels) : null,
    statusLabel: translateValue(item.resultStatus, codeLabels),
    summaryLabel: formatOperationLogSummary(item.requestSummary, item.errorMessage)
  };
}
