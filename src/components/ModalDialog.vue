<template>
  <div v-if="open" class="modal-shell" @click.self="$emit('close')">
    <div class="modal-card">
      <div class="modal-head">
        <h3>{{ title }}</h3>
        <button class="ghost-button" type="button" @click="$emit('close')">
          关闭
        </button>
      </div>
      <div class="modal-body">
        <slot />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  open: boolean;
  title: string;
}>();

defineEmits<{
  close: [];
}>();
</script>
