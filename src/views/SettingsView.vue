<script setup lang="ts">
import { useClubAppContext } from "../composables/useClubApp";

const {
  settings,
  chooseBackupDirectory,
  saveSettingsForm,
  createBackupNow,
  backupInfo,
  restoreFilePath,
  chooseRestoreFile,
  restoreFromBackup
} = useClubAppContext();
</script>

<template>
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
