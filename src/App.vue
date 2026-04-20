<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import ModalDialog from "./components/ModalDialog.vue";
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
} from "./api";
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
} from "./types";

type AppSection = "overview" | "members" | "gifts" | "migration" | "settings" | "logs";

type SectionOption = {
  key: AppSection;
  label: string;
  caption: string;
  description: string;
};

const sectionOptions: SectionOption[] = [
  {
    key: "overview",
    label: "经营总览",
    caption: "Overview",
    description: "查看核心指标、月度排行与系统快照"
  },
  {
    key: "members",
    label: "会员中心",
    caption: "Members",
    description: "会员检索、资料维护、消费与积分记录"
  },
  {
    key: "gifts",
    label: "礼品中心",
    caption: "Gifts",
    description: "浏览礼品库存并维护兑换配置"
  },
  {
    key: "migration",
    label: "数据迁移",
    caption: "Migration",
    description: "执行预检、导入批次并查看迁移结果"
  },
  {
    key: "settings",
    label: "系统设置",
    caption: "Settings",
    description: "门店参数、备份目录与恢复操作"
  },
  {
    key: "logs",
    label: "操作日志",
    caption: "Logs",
    description: "审计最近业务动作与异常信息"
  }
];

const activeSection = ref<AppSection>("overview");
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

const activeSectionMeta = computed(
  () => sectionOptions.find((item) => item.key === activeSection.value) ?? sectionOptions[0]
);
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
  activeSection.value = "gifts";
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

onMounted(() => {
  void bootstrap();
});
</script>

<template>
  <div class="app-shell">
    <section class="hero-panel">
      <div>
        <h1 class="hero-title">永丰文体会员管理系统</h1>
        <p class="hero-subtitle">
          通过一级菜单拆分经营总览、会员中心、礼品中心、迁移设置与日志审计，让每个页面只承载一类任务。
        </p>
      </div>
      <div class="hero-badge-group">
        <div class="hero-badge">{{ settings.storeName }}</div>
        <div v-if="loading" class="hero-badge soft">数据加载中</div>
      </div>
    </section>

    <div v-if="errorMessage" class="error-banner">{{ errorMessage }}</div>
    <div v-if="successMessage" class="success-banner">{{ successMessage }}</div>

    <section class="workspace-shell">
      <aside class="section-menu">
        <div class="menu-head">
          <span class="menu-kicker">功能导航</span>
          <h2>业务分区</h2>
          <p>把原来的大平铺页面拆成清晰入口，减少同屏干扰。</p>
        </div>

        <div class="menu-list">
          <button
            v-for="item in sectionOptions"
            :key="item.key"
            class="menu-item"
            :class="{ active: activeSection === item.key }"
            type="button"
            @click="activeSection = item.key"
          >
            <span class="menu-item-caption">{{ item.caption }}</span>
            <strong>{{ item.label }}</strong>
            <span class="menu-item-description">{{ item.description }}</span>
          </button>
        </div>

        <div class="menu-summary">
          <div class="summary-chip">
            <span>会员</span>
            <strong>{{ dashboard?.memberTotal ?? 0 }}</strong>
          </div>
          <div class="summary-chip">
            <span>礼品</span>
            <strong>{{ gifts.length }}</strong>
          </div>
          <div class="summary-chip">
            <span>日志</span>
            <strong>{{ logs.length }}</strong>
          </div>
        </div>
      </aside>

      <main class="workspace-main">
        <section class="workspace-header">
          <div>
            <div class="workspace-eyebrow">{{ activeSectionMeta.caption }}</div>
            <h2 class="workspace-title">{{ activeSectionMeta.label }}</h2>
            <p class="workspace-summary">{{ activeSectionMeta.description }}</p>
          </div>

          <div class="toolbar">
            <template v-if="activeSection === 'overview'">
              <label class="field compact-field">
                <span class="label">统计月份</span>
                <input v-model="currentMonth" type="month" />
              </label>
              <button class="secondary-button" type="button" @click="refreshDashboardForMonth">刷新指标</button>
            </template>

            <template v-else-if="activeSection === 'members'">
              <button class="primary-button" type="button" @click="openCreateMember">新增会员</button>
              <button class="secondary-button" type="button" @click="loadMembers">刷新列表</button>
            </template>

            <template v-else-if="activeSection === 'gifts'">
              <button class="primary-button" type="button" @click="saveGiftForm">保存礼品</button>
              <button class="ghost-button" type="button" @click="resetGiftForm">重置表单</button>
            </template>

            <template v-else-if="activeSection === 'migration'">
              <button class="ghost-button" type="button" @click="chooseMigrationFile">选择迁移文件</button>
              <button class="secondary-button" type="button" @click="runMigrationPrecheck">执行预检</button>
            </template>

            <template v-else-if="activeSection === 'settings'">
              <button class="primary-button" type="button" @click="saveSettingsForm">保存设置</button>
              <button class="secondary-button" type="button" @click="createBackupNow">立即备份</button>
            </template>

            <template v-else>
              <button class="secondary-button" type="button" @click="bootstrap">刷新日志</button>
            </template>
          </div>
        </section>

        <template v-if="activeSection === 'overview'">
          <section class="metric-grid">
            <article class="metric-card">
              <div class="metric-label">会员总数</div>
              <div class="metric-value">{{ dashboard?.memberTotal ?? 0 }}</div>
            </article>
            <article class="metric-card">
              <div class="metric-label">本月新增会员</div>
              <div class="metric-value">{{ dashboard?.newMembersThisMonth ?? 0 }}</div>
            </article>
            <article class="metric-card">
              <div class="metric-label">本月消费总额</div>
              <div class="metric-value">{{ formatMoney(dashboard?.consumeAmountThisMonth) }}</div>
            </article>
            <article class="metric-card">
              <div class="metric-label">本月新增积分</div>
              <div class="metric-value">{{ dashboard?.pointsAddedThisMonth ?? 0 }}</div>
            </article>
            <article class="metric-card">
              <div class="metric-label">本月兑换次数</div>
              <div class="metric-value">{{ dashboard?.redemptionCountThisMonth ?? 0 }}</div>
            </article>
          </section>

          <section class="overview-grid">
            <div class="stack">
              <section class="panel">
                <div class="panel-head">
                  <div>
                    <h3 class="panel-title">本月消费排行</h3>
                    <p class="panel-subtitle">{{ currentMonth }}</p>
                  </div>
                </div>
                <div class="top-list">
                  <div v-for="item in dashboard?.topConsumers ?? []" :key="item.memberId" class="top-item">
                    <div>
                      <strong>{{ item.name }}</strong>
                      <div class="muted">会员编号 {{ item.memberNo }}</div>
                    </div>
                    <strong>{{ formatMoney(item.totalAmount) }}</strong>
                  </div>
                  <div v-if="!(dashboard?.topConsumers?.length)" class="notice">本月还没有消费排行数据。</div>
                </div>
              </section>

              <section class="panel">
                <div class="panel-head">
                  <div>
                    <h3 class="panel-title">最近操作</h3>
                    <p class="panel-subtitle">显示最近 6 条业务记录</p>
                  </div>
                  <button class="ghost-button" type="button" @click="activeSection = 'logs'">查看全部日志</button>
                </div>
                <div class="log-list">
                  <div v-for="item in recentActivityPreview" :key="item.id" class="log-item">
                    <div class="row">
                      <strong>{{ item.moduleName }} / {{ item.actionName }}</strong>
                      <span class="tag">{{ item.resultStatus }}</span>
                    </div>
                    <div class="muted">{{ item.operatorName }} / {{ formatDate(item.createdAt) }}</div>
                    <div class="muted">{{ item.requestSummary || item.errorMessage || "无附加信息" }}</div>
                  </div>
                  <div v-if="!recentActivityPreview.length" class="notice">当前没有操作日志。</div>
                </div>
              </section>
            </div>

            <div class="stack">
              <section class="panel accent-panel">
                <div class="panel-head">
                  <div>
                    <h3 class="panel-title">快捷入口</h3>
                    <p class="panel-subtitle">把高频业务分配到独立页面</p>
                  </div>
                </div>
                <div class="shortcut-grid">
                  <button class="shortcut-card" type="button" @click="activeSection = 'members'">
                    <span class="shortcut-caption">MEMBERS</span>
                    <strong>进入会员中心</strong>
                    <span>检索会员、编辑资料、登记消费与兑换</span>
                  </button>
                  <button class="shortcut-card" type="button" @click="openCreateMember">
                    <span class="shortcut-caption">CREATE</span>
                    <strong>快速新增会员</strong>
                    <span>直接打开会员资料弹窗开始录入</span>
                  </button>
                  <button class="shortcut-card" type="button" @click="activeSection = 'gifts'">
                    <span class="shortcut-caption">GIFTS</span>
                    <strong>维护礼品库存</strong>
                    <span>查看兑换礼品与库存余量</span>
                  </button>
                  <button class="shortcut-card" type="button" @click="activeSection = 'settings'">
                    <span class="shortcut-caption">SYSTEM</span>
                    <strong>处理备份与设置</strong>
                    <span>保存门店参数并执行备份恢复</span>
                  </button>
                </div>
              </section>

              <section class="panel">
                <div class="panel-head">
                  <div>
                    <h3 class="panel-title">系统快照</h3>
                    <p class="panel-subtitle">礼品、备份与会员状态概览</p>
                  </div>
                </div>
                <div class="stats-grid">
                  <div class="mini-stat">
                    启用礼品
                    <strong>{{ activeGiftCount }}</strong>
                  </div>
                  <div class="mini-stat">
                    礼品库存
                    <strong>{{ totalGiftStock }}</strong>
                  </div>
                  <div class="mini-stat">
                    默认操作员
                    <strong>{{ settings.defaultOperator || "未设置" }}</strong>
                  </div>
                </div>
                <div class="notice">当前备份目录：{{ settings.backupPath || "未设置，请前往系统设置配置" }}</div>
              </section>
            </div>
          </section>
        </template>

        <template v-else-if="activeSection === 'members'">
          <section class="members-layout">
            <aside class="list-panel">
              <div class="panel-head">
                <div>
                  <h3 class="panel-title">会员检索</h3>
                  <p class="panel-subtitle">支持手机号、姓名、拼音首字母与会员编号混合搜索</p>
                </div>
                <button class="primary-button" type="button" @click="openCreateMember">新增会员</button>
              </div>

              <div class="search-bar">
                <input v-model="searchKeyword" placeholder="输入手机号 / 姓名 / 拼音 / 编号" @keyup.enter="loadMembers" />
                <button class="secondary-button" type="button" @click="loadMembers">搜索</button>
                <button class="ghost-button" type="button" @click="searchKeyword = ''; loadMembers()">清空</button>
              </div>

              <div class="member-list">
                <button
                  v-for="member in members"
                  :key="member.id"
                  class="member-item"
                  :class="{ active: member.id === selectedMemberId }"
                  type="button"
                  @click="selectMember(member.id)"
                >
                  <div class="member-item-top">
                    <span class="member-name">{{ member.name }}</span>
                    <span class="status-tag" :class="{ inactive: member.status !== 'ACTIVE' }">
                      {{ member.status === "ACTIVE" ? "正常" : "停用" }}
                    </span>
                  </div>
                  <div class="inline-info">
                    <span class="tag">编号 {{ member.memberNo }}</span>
                    <span class="tag">积分 {{ member.pointsBalance }}</span>
                  </div>
                  <p class="meta">{{ member.mobile || "无手机号" }}</p>
                  <p class="meta">
                    累计消费 {{ formatMoney(member.totalSpent) }} / 最近消费 {{ formatDate(member.lastConsumeAt) }}
                  </p>
                </button>

                <div v-if="!members.length && !loading" class="notice">当前没有匹配会员，请先新增或调整搜索条件。</div>
              </div>
            </aside>

            <div class="stack">
              <section class="panel">
                <div class="panel-head">
                  <div>
                    <h3 class="panel-title">会员详情</h3>
                    <p class="panel-subtitle">{{ activeMemberLabel }}</p>
                  </div>
                  <div class="toolbar">
                    <button class="ghost-button" type="button" :disabled="!selectedMember" @click="openEditMember">编辑</button>
                    <button class="secondary-button" type="button" :disabled="!selectedMember" @click="openConsumeModal">
                      登记消费
                    </button>
                    <button class="primary-button" type="button" :disabled="!selectedMember" @click="openRedeemModal">
                      积分兑换
                    </button>
                    <button class="danger-button" type="button" :disabled="!selectedMember" @click="disableMember">停用</button>
                  </div>
                </div>

                <div v-if="selectedMember" class="stats-grid">
                  <div class="mini-stat">
                    当前积分
                    <strong>{{ selectedMember.pointsBalance }}</strong>
                  </div>
                  <div class="mini-stat">
                    累计消费
                    <strong>{{ formatMoney(selectedMember.totalSpent) }}</strong>
                  </div>
                  <div class="mini-stat">
                    最近消费
                    <strong>{{ selectedMember.lastConsumeAt?.slice(0, 10) || "暂无" }}</strong>
                  </div>
                </div>

                <div v-if="selectedMember" class="section-grid">
                  <div class="field">
                    <span class="label">手机号</span>
                    <input :value="selectedMember.mobile || '暂无'" readonly />
                  </div>
                  <div class="field">
                    <span class="label">性别</span>
                    <input :value="selectedMember.gender || '未填写'" readonly />
                  </div>
                  <div class="field">
                    <span class="label">生日月</span>
                    <input :value="selectedMember.birthMonth || '未填写'" readonly />
                  </div>
                  <div class="field">
                    <span class="label">生日日</span>
                    <input :value="selectedMember.birthDay || '未填写'" readonly />
                  </div>
                  <div class="field full">
                    <span class="label">备注</span>
                    <textarea :value="selectedMember.remark || '暂无备注'" readonly />
                  </div>
                </div>

                <div v-else class="notice">从左侧选择会员后，这里会展示积分、历史记录和快捷操作。</div>
              </section>

              <section class="panel">
                <div class="panel-head">
                  <div>
                    <h3 class="panel-title">历史记录</h3>
                    <p class="panel-subtitle">最近消费、兑换和积分流水</p>
                  </div>
                </div>
                <div class="history-list">
                  <div v-for="consume in selectedDetail?.recentConsumptions ?? []" :key="consume.id" class="history-item">
                    <div class="row">
                      <strong>消费 {{ formatMoney(consume.amount) }}</strong>
                      <span class="tag">+{{ consume.pointsAdded }} 分</span>
                    </div>
                    <div class="muted">{{ consume.recordNo }} / {{ consume.operatorName }} / {{ formatDate(consume.createdAt) }}</div>
                  </div>
                  <div v-for="redeem in selectedDetail?.recentRedemptions ?? []" :key="'r' + redeem.id" class="history-item">
                    <div class="row">
                      <strong>{{ redeem.giftNameSnapshot }} x {{ redeem.qty }}</strong>
                      <span class="tag">-{{ redeem.pointsUsed }} 分</span>
                    </div>
                    <div class="muted">{{ redeem.redeemNo }} / {{ redeem.operatorName }} / {{ formatDate(redeem.createdAt) }}</div>
                  </div>
                  <div v-for="ledger in selectedDetail?.recentLedger ?? []" :key="'l' + ledger.id" class="history-item">
                    <div class="row">
                      <strong>{{ ledger.changeType }} / {{ ledger.sourceType }}</strong>
                      <span class="tag">余额 {{ ledger.balanceAfter }}</span>
                    </div>
                    <div class="muted">{{ ledger.pointsDelta }} 分 / {{ ledger.operatorName }} / {{ formatDate(ledger.createdAt) }}</div>
                  </div>
                  <div
                    v-if="
                      !(selectedDetail?.recentConsumptions?.length || 0) &&
                      !(selectedDetail?.recentRedemptions?.length || 0) &&
                      !(selectedDetail?.recentLedger?.length || 0)
                    "
                    class="notice"
                  >
                    当前会员还没有业务记录。
                  </div>
                </div>
              </section>
            </div>
          </section>
        </template>

        <template v-else-if="activeSection === 'gifts'">
          <section class="split-layout">
            <section class="panel">
              <div class="panel-head">
                <div>
                  <h3 class="panel-title">礼品总览</h3>
                  <p class="panel-subtitle">浏览礼品库存、状态与积分成本</p>
                </div>
              </div>

              <div class="gift-list">
                <div v-for="gift in gifts" :key="gift.id || gift.giftName" class="gift-item">
                  <div class="row">
                    <strong>{{ gift.giftName }}</strong>
                    <span class="tag">{{ gift.pointsCost }} 分</span>
                  </div>
                  <div class="muted">库存 {{ gift.stockQty }} / 状态 {{ gift.status }}</div>
                  <div class="toolbar">
                    <button class="ghost-button" type="button" @click="editGift(gift)">编辑礼品</button>
                  </div>
                </div>
                <div v-if="!gifts.length" class="notice">当前还没有礼品数据。</div>
              </div>
            </section>

            <section class="panel">
              <div class="panel-head">
                <div>
                  <h3 class="panel-title">{{ giftForm.id ? "编辑礼品" : "新增礼品" }}</h3>
                  <p class="panel-subtitle">维护积分、库存与启停状态</p>
                </div>
              </div>

              <div class="section-grid">
                <label class="field">
                  <span class="label">礼品名称</span>
                  <input v-model="giftForm.giftName" />
                </label>
                <label class="field">
                  <span class="label">所需积分</span>
                  <input v-model.number="giftForm.pointsCost" type="number" min="0" />
                </label>
                <label class="field">
                  <span class="label">库存</span>
                  <input v-model.number="giftForm.stockQty" type="number" min="0" />
                </label>
                <label class="field">
                  <span class="label">状态</span>
                  <select v-model="giftForm.status">
                    <option value="ACTIVE">启用</option>
                    <option value="INACTIVE">停用</option>
                  </select>
                </label>
                <label class="field full">
                  <span class="label">备注</span>
                  <textarea v-model="giftForm.remark" />
                </label>
              </div>

              <div class="toolbar">
                <button class="primary-button" type="button" @click="saveGiftForm">保存礼品</button>
                <button class="ghost-button" type="button" @click="resetGiftForm">重置</button>
              </div>
            </section>
          </section>
        </template>

        <template v-else-if="activeSection === 'migration'">
          <section class="split-layout">
            <section class="panel">
              <div class="panel-head">
                <div>
                  <h3 class="panel-title">迁移控制台</h3>
                  <p class="panel-subtitle">先预检，再执行正式导入</p>
                </div>
              </div>

              <div class="field">
                <span class="label">迁移文件</span>
                <input v-model="migrationSourcePath" placeholder="选择 JSON 格式迁移文件" readonly />
              </div>
              <div class="toolbar">
                <button class="ghost-button" type="button" @click="chooseMigrationFile">选择文件</button>
                <button class="secondary-button" type="button" @click="runMigrationPrecheck">执行预检</button>
                <button class="primary-button" type="button" :disabled="!precheck?.canExecute" @click="executeMigrationRun">
                  正式导入
                </button>
              </div>
              <div class="notice">建议在正式导入前先完成一次数据库备份，并处理所有预检错误。</div>
            </section>

            <section class="panel">
              <div class="panel-head">
                <div>
                  <h3 class="panel-title">迁移结果</h3>
                  <p class="panel-subtitle">查看批次摘要、预警与失败明细</p>
                </div>
              </div>

              <div v-if="precheck" class="warning-list">
                <div class="warning-item">
                  <strong>批次 {{ precheck.batchNo }}</strong>
                  <div class="muted">
                    会员 {{ precheck.memberCount }} / 消费 {{ precheck.consumptionCount }} / 兑换
                    {{ precheck.redemptionCount }} / 礼品 {{ precheck.giftCount }} / 模式 {{ precheck.importMode }}
                  </div>
                  <div class="muted">可执行：{{ precheck.canExecute ? "是" : "否" }}</div>
                </div>
                <div v-for="warning in precheck.warnings" :key="warning" class="warning-item">{{ warning }}</div>
                <div
                  v-for="error in precheck.errors"
                  :key="`${error.entityType}-${error.legacyPk}-${error.errorCode}`"
                  class="warning-item"
                >
                  {{ error.entityType }} / {{ error.legacyPk || "无主键" }} / {{ error.errorMessage }}
                </div>
              </div>

              <div v-if="migrationReport" class="warning-list">
                <div class="warning-item">
                  <strong>导入结果 {{ migrationReport.batch.status }}</strong>
                  <div class="muted">成功 {{ migrationReport.batch.successCount }} / 失败 {{ migrationReport.batch.failedCount }}</div>
                </div>
                <div
                  v-for="error in migrationReport.errors"
                  :key="`report-${error.entityType}-${error.legacyPk}-${error.errorCode}`"
                  class="warning-item"
                >
                  {{ error.entityType }} / {{ error.legacyPk || "无主键" }} / {{ error.errorMessage }}
                </div>
              </div>

              <div v-if="!precheck && !migrationReport" class="notice">选择迁移文件并执行预检后，这里会显示批次结果。</div>
            </section>
          </section>
        </template>

        <template v-else-if="activeSection === 'settings'">
          <section class="split-layout">
            <section class="panel">
              <div class="panel-head">
                <div>
                  <h3 class="panel-title">基础设置</h3>
                  <p class="panel-subtitle">维护门店信息、积分规则与默认操作员</p>
                </div>
              </div>

              <div class="section-grid">
                <label class="field">
                  <span class="label">门店名称</span>
                  <input v-model="settings.storeName" />
                </label>
                <label class="field">
                  <span class="label">默认操作员</span>
                  <input v-model="settings.defaultOperator" />
                </label>
                <label class="field">
                  <span class="label">积分规则</span>
                  <input v-model.number="settings.pointsRuleAmount" type="number" min="1" />
                </label>
                <label class="field">
                  <span class="label">兼容参数 JPDJ</span>
                  <input v-model="settings.legacyJpdj" />
                </label>
                <label class="field full">
                  <span class="label">备份目录</span>
                  <input v-model="settings.backupPath" readonly />
                </label>
              </div>

              <div class="toolbar">
                <button class="ghost-button" type="button" @click="chooseBackupDirectory">选择备份目录</button>
                <button class="primary-button" type="button" @click="saveSettingsForm">保存设置</button>
              </div>
            </section>

            <section class="panel">
              <div class="panel-head">
                <div>
                  <h3 class="panel-title">备份与恢复</h3>
                  <p class="panel-subtitle">先备份，再执行恢复操作</p>
                </div>
              </div>

              <div class="toolbar">
                <button class="secondary-button" type="button" @click="createBackupNow">立即备份</button>
              </div>
              <div v-if="backupInfo" class="notice">最近备份：{{ backupInfo.filePath }}</div>

              <div class="field full">
                <span class="label">恢复备份文件</span>
                <input v-model="restoreFilePath" readonly />
              </div>
              <div class="toolbar">
                <button class="ghost-button" type="button" @click="chooseRestoreFile">选择恢复文件</button>
                <button class="danger-button" type="button" :disabled="!restoreFilePath" @click="restoreFromBackup">
                  执行恢复
                </button>
              </div>
            </section>
          </section>
        </template>

        <template v-else>
          <section class="panel">
            <div class="panel-head">
              <div>
                <h3 class="panel-title">操作日志</h3>
                <p class="panel-subtitle">最近 40 条业务记录与异常信息</p>
              </div>
            </div>
            <div class="log-list large-list">
              <div v-for="item in logs" :key="item.id" class="log-item">
                <div class="row">
                  <strong>{{ item.moduleName }} / {{ item.actionName }}</strong>
                  <span class="tag">{{ item.resultStatus }}</span>
                </div>
                <div class="muted">{{ item.operatorName }} / {{ formatDate(item.createdAt) }}</div>
                <div class="muted">{{ item.requestSummary || item.errorMessage || "无附加信息" }}</div>
              </div>
              <div v-if="!logs.length" class="notice">当前没有日志记录。</div>
            </div>
          </section>
        </template>
      </main>
    </section>

    <ModalDialog :open="memberModalOpen" title="会员资料" @close="memberModalOpen = false">
      <div class="section-grid">
        <label class="field">
          <span class="label">会员编号</span>
          <input v-model.number="memberForm.memberNo" type="number" min="1" placeholder="留空则自动生成" />
        </label>
        <label class="field">
          <span class="label">姓名</span>
          <input v-model="memberForm.name" />
        </label>
        <label class="field">
          <span class="label">性别</span>
          <select v-model="memberForm.gender">
            <option value="未知">未知</option>
            <option value="男">男</option>
            <option value="女">女</option>
          </select>
        </label>
        <label class="field">
          <span class="label">手机号</span>
          <input v-model="memberForm.mobile" />
        </label>
        <label class="field">
          <span class="label">生日月</span>
          <input v-model="memberForm.birthMonth" placeholder="如 05" />
        </label>
        <label class="field">
          <span class="label">生日日</span>
          <input v-model="memberForm.birthDay" placeholder="如 12" />
        </label>
        <label class="field full">
          <span class="label">备注</span>
          <textarea v-model="memberForm.remark" />
        </label>
      </div>
      <div class="toolbar">
        <button class="primary-button" type="button" @click="saveMember">保存</button>
      </div>
    </ModalDialog>

    <ModalDialog :open="consumeModalOpen" title="登记消费" @close="consumeModalOpen = false">
      <div class="field">
        <span class="label">会员</span>
        <input :value="activeMemberLabel" readonly />
      </div>
      <div class="section-grid">
        <label class="field">
          <span class="label">消费金额</span>
          <input v-model.number="consumeForm.amount" type="number" min="0.01" step="0.01" />
        </label>
        <label class="field">
          <span class="label">操作人</span>
          <input v-model="consumeForm.operatorName" />
        </label>
        <label class="field full">
          <span class="label">备注</span>
          <textarea v-model="consumeForm.remark" />
        </label>
      </div>
      <div class="toolbar">
        <button class="primary-button" type="button" @click="saveConsume">提交消费</button>
      </div>
    </ModalDialog>

    <ModalDialog :open="redeemModalOpen" title="积分兑换" @close="redeemModalOpen = false">
      <div class="field">
        <span class="label">会员</span>
        <input :value="activeMemberLabel" readonly />
      </div>
      <div class="section-grid">
        <label class="field">
          <span class="label">礼品</span>
          <select v-model.number="redeemForm.giftId">
            <option v-for="gift in gifts" :key="gift.id || gift.giftName" :value="gift.id || 0">
              {{ gift.giftName }} / {{ gift.pointsCost }} 分 / 库存 {{ gift.stockQty }}
            </option>
          </select>
        </label>
        <label class="field">
          <span class="label">数量</span>
          <input v-model.number="redeemForm.qty" type="number" min="1" />
        </label>
        <label class="field">
          <span class="label">操作人</span>
          <input v-model="redeemForm.operatorName" />
        </label>
        <label class="field full">
          <span class="label">备注</span>
          <textarea v-model="redeemForm.remark" />
        </label>
      </div>
      <div class="toolbar">
        <button class="primary-button" type="button" @click="saveRedeem">提交兑换</button>
      </div>
    </ModalDialog>
  </div>
</template>
