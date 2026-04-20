<script setup lang="ts">
import { useClubAppContext } from "../composables/useClubApp";

const {
  migrationSourcePath,
  chooseMigrationFile,
  runMigrationPrecheck,
  precheck,
  executeMigrationRun,
  migrationReport
} = useClubAppContext();
</script>

<template>
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
            会员 {{ precheck.memberCount }} / 消费 {{ precheck.consumptionCount }} / 兑换 {{ precheck.redemptionCount }} /
            礼品 {{ precheck.giftCount }} / 模式 {{ precheck.importMode }}
          </div>
          <div class="muted">可执行：{{ precheck.canExecute ? "是" : "否" }}</div>
        </div>
        <div v-for="warning in precheck.warnings" :key="warning" class="warning-item">{{ warning }}</div>
        <div v-for="error in precheck.errors" :key="`${error.entityType}-${error.legacyPk}-${error.errorCode}`" class="warning-item">
          {{ error.entityType }} / {{ error.legacyPk || "无主键" }} / {{ error.errorMessage }}
        </div>
      </div>

      <div v-if="migrationReport" class="warning-list">
        <div class="warning-item">
          <strong>导入结果 {{ migrationReport.batch.status }}</strong>
          <div class="muted">成功 {{ migrationReport.batch.successCount }} / 失败 {{ migrationReport.batch.failedCount }}</div>
        </div>
        <div v-for="error in migrationReport.errors" :key="`report-${error.entityType}-${error.legacyPk}-${error.errorCode}`" class="warning-item">
          {{ error.entityType }} / {{ error.legacyPk || "无主键" }} / {{ error.errorMessage }}
        </div>
      </div>

      <div v-if="!precheck && !migrationReport" class="notice">选择迁移文件并执行预检后，这里会显示批次结果。</div>
    </section>
  </section>
</template>
