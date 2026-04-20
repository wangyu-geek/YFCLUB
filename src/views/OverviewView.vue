<script setup lang="ts">
import { useRouter } from "vue-router";
import { useClubAppContext } from "../composables/useClubApp";

const router = useRouter();
const {
  currentMonth,
  refreshDashboardForMonth,
  dashboard,
  recentActivityPreview,
  formatMoney,
  formatDate,
  activeGiftCount,
  totalGiftStock,
  settings,
  openCreateMember
} = useClubAppContext();

function goTo(name: "members" | "gifts" | "settings" | "logs") {
  void router.push({ name });
}
</script>

<template>
  <div class="stack">
    <section class="panel">
      <div class="panel-head">
        <div>
          <h3 class="panel-title">月度筛选</h3>
          <p class="panel-subtitle">切换月份后可刷新经营总览和排行数据</p>
        </div>
        <div class="toolbar">
          <label class="field compact-field">
            <span class="label">统计月份</span>
            <input v-model="currentMonth" type="month" />
          </label>
          <button class="secondary-button" type="button" @click="refreshDashboardForMonth">刷新指标</button>
        </div>
      </div>
    </section>

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
            <button class="ghost-button" type="button" @click="goTo('logs')">查看全部日志</button>
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
              <p class="panel-subtitle">通过路由切换到独立页面处理业务</p>
            </div>
          </div>
          <div class="shortcut-grid">
            <button class="shortcut-card" type="button" @click="goTo('members')">
              <span class="shortcut-caption">MEMBERS</span>
              <strong>进入会员中心</strong>
              <span>检索会员、编辑资料、登记消费与兑换</span>
            </button>
            <button class="shortcut-card" type="button" @click="openCreateMember">
              <span class="shortcut-caption">CREATE</span>
              <strong>快速新增会员</strong>
              <span>直接打开会员资料弹窗开始录入</span>
            </button>
            <button class="shortcut-card" type="button" @click="goTo('gifts')">
              <span class="shortcut-caption">GIFTS</span>
              <strong>维护礼品库存</strong>
              <span>查看兑换礼品与库存余量</span>
            </button>
            <button class="shortcut-card" type="button" @click="goTo('settings')">
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
  </div>
</template>
