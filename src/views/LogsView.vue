<script setup lang="ts">
import { useClubAppContext } from "../composables/useClubApp";

const { logs, formatDate, bootstrap } = useClubAppContext();
</script>

<template>
  <section class="panel">
    <div class="panel-head">
      <div>
        <h3 class="panel-title">操作日志</h3>
        <p class="panel-subtitle">最近 40 条业务记录与异常信息</p>
      </div>
      <button class="secondary-button" type="button" @click="bootstrap">刷新日志</button>
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
