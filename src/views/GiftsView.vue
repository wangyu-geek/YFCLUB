<script setup lang="ts">
import { useClubAppContext } from "../composables/useClubApp";

const { gifts, giftForm, editGift, saveGiftForm, resetGiftForm, formatCodeLabel } = useClubAppContext();
</script>

<template>
  <section class="split-layout">
    <section class="panel">
      <div class="panel-head">
        <div>
          <h3 class="panel-title">礼品总览</h3>
          <p class="panel-subtitle">浏览礼品状态、积分成本与兑换规则</p>
        </div>
      </div>

      <div class="gift-list">
        <div v-for="gift in gifts" :key="gift.id || gift.giftName" class="gift-item">
          <div class="row">
            <strong>{{ gift.giftName }}</strong>
            <span class="tag">{{ gift.pointsCost }} 分</span>
          </div>
          <div class="muted">
            状态 {{ formatCodeLabel(gift.status) }} /
            {{ gift.uniquePerMember ? "每会员限兑一次" : "可重复兑换" }}
          </div>
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
          <p class="panel-subtitle">维护积分、启停状态与兑换规则</p>
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
          <span class="label">状态</span>
          <select v-model="giftForm.status">
            <option value="ACTIVE">启用</option>
            <option value="INACTIVE">停用</option>
          </select>
        </label>
        <div class="field">
          <span class="label">兑换规则</span>
          <label class="checkbox-row">
            <input v-model="giftForm.uniquePerMember" type="checkbox" />
            兑换一次
          </label>
        </div>
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
