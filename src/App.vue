<script setup lang="ts">
import { computed, onMounted, provide } from "vue";
import { RouterLink, RouterView, useRoute } from "vue-router";
import ModalDialog from "./components/ModalDialog.vue";
import { useClubApp, clubAppKey } from "./composables/useClubApp";
import { sectionOptions } from "./navigation";

const route = useRoute();
const app = useClubApp();

provide(clubAppKey, app);

const {
  settings,
  loading,
  errorMessage,
  successMessage,
  dashboard,
  gifts,
  logs,
  memberModalOpen,
  memberForm,
  saveMember,
  consumeModalOpen,
  activeMemberLabel,
  consumeForm,
  saveConsume,
  redeemModalOpen,
  redeemForm,
  saveRedeem
} = app;

const activeSectionMeta = computed(
  () => sectionOptions.find((item) => item.key === route.name) ?? sectionOptions[0]
);

onMounted(() => {
  void app.bootstrap();
});
</script>

<template>
  <div class="app-shell">
    <section class="hero-panel">
      <div>
        <h1 class="hero-title">永丰文体会员管理系统</h1>
        <p class="hero-subtitle">
          现在改为 `vue-router` 驱动的多分区界面，每个业务模块都有自己的独立路由和内容区域。
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
          <p>现在每个入口都对应独立路由，支持明确的页面边界和后续扩展。</p>
        </div>

        <div class="menu-list">
          <RouterLink
            v-for="item in sectionOptions"
            :key="item.key"
            class="menu-item"
            :class="{ active: route.name === item.key }"
            :to="item.to"
          >
            <span class="menu-item-caption">{{ item.caption }}</span>
            <strong>{{ item.label }}</strong>
            <span class="menu-item-description">{{ item.description }}</span>
          </RouterLink>
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
        </section>

        <RouterView />
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
