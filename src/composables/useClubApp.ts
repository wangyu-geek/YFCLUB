import { computed, inject, ref, type InjectionKey } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import {
  backupCreate,
  backupRestore,
  consumeCreate,
  giftList,
  giftRedeem,
  giftSave,
  memberCreate,
  memberDisable,
  memberGetDetail,
  memberSearch,
  memberUpdate,
  migrationExecute,
  migrationGetReport,
  migrationPrecheck,
  operationLogsQuery,
  reportDashboard,
  settingsGet,
  settingsSave
} from "../api";
import type {
  BackupResult,
  ConsumePayload,
  DashboardData,
  GiftRecord,
  MemberDetailData,
  MemberFormPayload,
  MemberSummary,
  MigrationPrecheckResult,
  MigrationReport,
  OperationLogItem,
  RedeemPayload,
  SettingsData
} from "../types";

export function useClubApp() {
  const searchKeyword = ref("");
  const members = ref<MemberSummary[]>([]);
  const selectedMemberId = ref<number | null>(null);
  const selectedDetail = ref<MemberDetailData | null>(null);
  const dashboard = ref<DashboardData | null>(null);
  const gifts = ref<GiftRecord[]>([]);
  const logs = ref<OperationLogItem[]>([]);
  const precheck = ref<MigrationPrecheckResult | null>(null);
  const migrationReport = ref<MigrationReport | null>(null);
  const migrationSourcePath = ref("");
  const restoreFilePath = ref("");
  const backupInfo = ref<BackupResult | null>(null);
  const loading = ref(false);
  const successMessage = ref("");
  const errorMessage = ref("");
  const currentMonth = ref(new Date().toISOString().slice(0, 7));

  const settings = ref<SettingsData>({
    storeName: "永丰文体",
    dbPath: "",
    backupPath: "",
    autoBackupEnabled: true,
    pointsRuleAmount: 10,
    legacyJpdj: "10",
    defaultOperator: "管理员"
  });

  const memberModalOpen = ref(false);
  const memberEditId = ref<number | null>(null);
  const memberForm = ref<MemberFormPayload>({
    memberNo: null,
    name: "",
    gender: "未知",
    birthMonth: "",
    birthDay: "",
    mobile: "",
    remark: ""
  });

  const consumeModalOpen = ref(false);
  const consumeForm = ref<ConsumePayload>({
    memberId: 0,
    amount: 0,
    operatorName: "管理员",
    remark: ""
  });

  const redeemModalOpen = ref(false);
  const redeemForm = ref<RedeemPayload>({
    memberId: 0,
    giftId: 0,
    qty: 1,
    operatorName: "管理员",
    remark: ""
  });

  const giftForm = ref<GiftRecord>({
    id: null,
    giftName: "",
    pointsCost: 0,
    stockQty: 0,
    status: "ACTIVE",
    remark: ""
  });

  const selectedMember = computed(() => selectedDetail.value?.member ?? null);
  const activeMemberLabel = computed(() =>
    selectedMember.value
      ? `${selectedMember.value.memberNo} - ${selectedMember.value.name}`
      : "未选择会员"
  );
  const activeGiftCount = computed(() => gifts.value.filter((item) => item.status === "ACTIVE").length);
  const totalGiftStock = computed(() =>
    gifts.value.reduce((sum, item) => sum + Number(item.stockQty ?? 0), 0)
  );
  const recentActivityPreview = computed(() => logs.value.slice(0, 6));

  function setSuccess(message: string) {
    successMessage.value = message;
    errorMessage.value = "";
  }

  function setError(error: unknown) {
    successMessage.value = "";
    errorMessage.value = error instanceof Error ? error.message : String(error);
  }

  function clearMessages() {
    successMessage.value = "";
    errorMessage.value = "";
  }

  function formatMoney(value?: number | null) {
    return (value ?? 0).toFixed(2);
  }

  function formatDate(value?: string | null) {
    return value || "暂无";
  }

  function resetGiftForm() {
    giftForm.value = {
      id: null,
      giftName: "",
      pointsCost: 0,
      stockQty: 0,
      status: "ACTIVE",
      remark: ""
    };
  }

  async function bootstrap() {
    loading.value = true;
    try {
      const [settingsData, dashboardData, giftData, logData] = await Promise.all([
        settingsGet(),
        reportDashboard(currentMonth.value),
        giftList(),
        operationLogsQuery({ limit: 40 })
      ]);
      settings.value = settingsData;
      consumeForm.value.operatorName = settingsData.defaultOperator;
      redeemForm.value.operatorName = settingsData.defaultOperator;
      dashboard.value = dashboardData;
      gifts.value = giftData;
      logs.value = logData;
      await loadMembers();
    } catch (error) {
      setError(error);
    } finally {
      loading.value = false;
    }
  }

  async function refreshDashboardForMonth() {
    try {
      dashboard.value = await reportDashboard(currentMonth.value);
      setSuccess(`已切换到 ${currentMonth.value} 的经营数据。`);
    } catch (error) {
      setError(error);
    }
  }

  async function loadMembers() {
    const result = await memberSearch(searchKeyword.value, 1, 50);
    members.value = result.items;
    if (!members.value.length) {
      selectedMemberId.value = null;
      selectedDetail.value = null;
      return;
    }
    const targetId =
      members.value.find((item) => item.id === selectedMemberId.value)?.id ?? members.value[0].id;
    await selectMember(targetId);
  }

  async function selectMember(memberId: number) {
    selectedMemberId.value = memberId;
    selectedDetail.value = await memberGetDetail(memberId);
  }

  async function refreshAfterMutation(memberId?: number | null) {
    await Promise.all([
      reportDashboard(currentMonth.value).then((data) => {
        dashboard.value = data;
      }),
      giftList().then((data) => {
        gifts.value = data;
      }),
      operationLogsQuery({ limit: 40 }).then((data) => {
        logs.value = data;
      })
    ]);
    await loadMembers();
    if (memberId) {
      await selectMember(memberId);
    }
  }

  function openCreateMember() {
    memberEditId.value = null;
    memberForm.value = {
      memberNo: null,
      name: "",
      gender: "未知",
      birthMonth: "",
      birthDay: "",
      mobile: "",
      remark: ""
    };
    memberModalOpen.value = true;
  }

  function openEditMember() {
    if (!selectedMember.value) return;
    memberEditId.value = selectedMember.value.id;
    memberForm.value = {
      memberNo: selectedMember.value.memberNo,
      name: selectedMember.value.name,
      gender: selectedMember.value.gender ?? "未知",
      birthMonth: selectedMember.value.birthMonth ?? "",
      birthDay: selectedMember.value.birthDay ?? "",
      mobile: selectedMember.value.mobile ?? "",
      remark: selectedMember.value.remark ?? ""
    };
    memberModalOpen.value = true;
  }

  async function saveMember() {
    try {
      clearMessages();
      const payload = {
        ...memberForm.value,
        memberNo: memberForm.value.memberNo ? Number(memberForm.value.memberNo) : null
      };
      const result = memberEditId.value
        ? await memberUpdate(memberEditId.value, payload)
        : await memberCreate(payload);
      memberModalOpen.value = false;
      setSuccess(result.message);
      await refreshAfterMutation(result.targetId ? Number(result.targetId) : null);
    } catch (error) {
      setError(error);
    }
  }

  async function disableMember() {
    if (!selectedMember.value) return;
    if (!window.confirm(`确定停用会员 ${selectedMember.value.name} 吗？`)) return;
    try {
      const result = await memberDisable(selectedMember.value.id);
      setSuccess(result.message);
      await refreshAfterMutation();
    } catch (error) {
      setError(error);
    }
  }

  function openConsumeModal() {
    if (!selectedMember.value) return;
    consumeForm.value = {
      memberId: selectedMember.value.id,
      amount: 0,
      operatorName: settings.value.defaultOperator,
      remark: ""
    };
    consumeModalOpen.value = true;
  }

  async function saveConsume() {
    try {
      const result = await consumeCreate({
        ...consumeForm.value,
        amount: Number(consumeForm.value.amount)
      });
      consumeModalOpen.value = false;
      setSuccess(result.message);
      await refreshAfterMutation(consumeForm.value.memberId);
    } catch (error) {
      setError(error);
    }
  }

  function openRedeemModal() {
    if (!selectedMember.value) return;
    const firstGift = gifts.value.find((item) => item.status === "ACTIVE") ?? gifts.value[0];
    redeemForm.value = {
      memberId: selectedMember.value.id,
      giftId: firstGift?.id ?? 0,
      qty: 1,
      operatorName: settings.value.defaultOperator,
      remark: ""
    };
    redeemModalOpen.value = true;
  }

  async function saveRedeem() {
    try {
      const result = await giftRedeem({
        ...redeemForm.value,
        qty: Number(redeemForm.value.qty)
      });
      redeemModalOpen.value = false;
      setSuccess(result.message);
      await refreshAfterMutation(redeemForm.value.memberId);
    } catch (error) {
      setError(error);
    }
  }

  function editGift(gift: GiftRecord) {
    giftForm.value = { ...gift };
  }

  async function saveGiftForm() {
    try {
      const result = await giftSave({
        ...giftForm.value,
        pointsCost: Number(giftForm.value.pointsCost),
        stockQty: Number(giftForm.value.stockQty)
      });
      setSuccess(result.message);
      resetGiftForm();
      await refreshAfterMutation(selectedMemberId.value);
    } catch (error) {
      setError(error);
    }
  }

  async function saveSettingsForm() {
    try {
      const result = await settingsSave({
        ...settings.value,
        pointsRuleAmount: Number(settings.value.pointsRuleAmount)
      });
      setSuccess(result.message);
      settings.value = await settingsGet();
      await refreshAfterMutation(selectedMemberId.value);
    } catch (error) {
      setError(error);
    }
  }

  async function chooseBackupDirectory() {
    const result = await open({
      directory: true,
      multiple: false,
      title: "选择备份目录"
    });
    if (typeof result === "string") {
      settings.value.backupPath = result;
    }
  }

  async function createBackupNow() {
    try {
      backupInfo.value = await backupCreate(settings.value.backupPath);
      setSuccess(backupInfo.value.message);
      logs.value = await operationLogsQuery({ limit: 40 });
    } catch (error) {
      setError(error);
    }
  }

  async function chooseRestoreFile() {
    const result = await open({
      multiple: false,
      title: "选择备份文件",
      filters: [{ name: "SQLite Backup", extensions: ["db", "sqlite"] }]
    });
    if (typeof result === "string") {
      restoreFilePath.value = result;
    }
  }

  async function restoreFromBackup() {
    if (!restoreFilePath.value) return;
    if (!window.confirm("恢复会覆盖当前数据库，系统将先自动备份当前数据，确定继续吗？")) return;
    try {
      const result = await backupRestore(restoreFilePath.value);
      setSuccess(result.message);
      await bootstrap();
    } catch (error) {
      setError(error);
    }
  }

  async function chooseMigrationFile() {
    const result = await open({
      multiple: false,
      title: "选择迁移文件",
      filters: [{ name: "JSON", extensions: ["json"] }]
    });
    if (typeof result === "string") {
      migrationSourcePath.value = result;
    }
  }

  async function runMigrationPrecheck() {
    if (!migrationSourcePath.value) return;
    try {
      precheck.value = await migrationPrecheck(migrationSourcePath.value, "full");
      migrationReport.value = null;
      setSuccess("预检完成，请先查看结果再执行正式导入。");
    } catch (error) {
      setError(error);
    }
  }

  async function executeMigrationRun() {
    if (!precheck.value) return;
    if (!window.confirm("正式导入将写入业务数据库，是否继续？")) return;
    try {
      const result = await migrationExecute(precheck.value.batchNo);
      migrationReport.value = await migrationGetReport(precheck.value.batchNo);
      setSuccess(result.message);
      await bootstrap();
    } catch (error) {
      setError(error);
    }
  }

  return {
    searchKeyword,
    members,
    selectedMemberId,
    selectedDetail,
    dashboard,
    gifts,
    logs,
    precheck,
    migrationReport,
    migrationSourcePath,
    restoreFilePath,
    backupInfo,
    loading,
    successMessage,
    errorMessage,
    currentMonth,
    settings,
    memberModalOpen,
    memberEditId,
    memberForm,
    consumeModalOpen,
    consumeForm,
    redeemModalOpen,
    redeemForm,
    giftForm,
    selectedMember,
    activeMemberLabel,
    activeGiftCount,
    totalGiftStock,
    recentActivityPreview,
    formatMoney,
    formatDate,
    resetGiftForm,
    bootstrap,
    refreshDashboardForMonth,
    loadMembers,
    selectMember,
    openCreateMember,
    openEditMember,
    saveMember,
    disableMember,
    openConsumeModal,
    saveConsume,
    openRedeemModal,
    saveRedeem,
    editGift,
    saveGiftForm,
    saveSettingsForm,
    chooseBackupDirectory,
    createBackupNow,
    chooseRestoreFile,
    restoreFromBackup,
    chooseMigrationFile,
    runMigrationPrecheck,
    executeMigrationRun
  };
}

export type ClubApp = ReturnType<typeof useClubApp>;

export const clubAppKey: InjectionKey<ClubApp> = Symbol("club-app");

export function useClubAppContext() {
  const app = inject(clubAppKey);
  if (!app) {
    throw new Error("Club app context is not available.");
  }
  return app;
}
