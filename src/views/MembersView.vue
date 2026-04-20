<script setup lang="ts">
import { useClubAppContext } from "../composables/useClubApp";

const {
  searchKeyword,
  loadMembers,
  openCreateMember,
  members,
  loading,
  selectedMemberId,
  selectMember,
  formatMoney,
  formatDate,
  selectedMember,
  activeMemberLabel,
  openEditMember,
  openConsumeModal,
  openRedeemModal,
  disableMember,
  selectedDetail
} = useClubAppContext();
</script>

<template>
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
          <p class="meta">累计消费 {{ formatMoney(member.totalSpent) }} / 最近消费 {{ formatDate(member.lastConsumeAt) }}</p>
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
