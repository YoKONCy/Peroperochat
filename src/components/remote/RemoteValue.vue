<template>
  <span v-if="value == null" class="remote-value-empty">未设置</span>
  <span v-else-if="typeof value === 'object' && depth >= 12" class="remote-value-empty">已到展开深度上限</span>
  <div v-else-if="Array.isArray(value)" class="remote-value-list"><p v-if="!value.length" class="remote-help">暂无数据</p><details v-for="(item, index) in value.slice(0, limit)" :key="index" :open="value.length < 3 || undefined" @toggle="expanded[index] = $event.currentTarget.open"><summary>{{ item?.name || item?.title || item?.label || `第 ${index + 1} 项` }}</summary><RemoteValue v-if="value.length < 3 || expanded[index]" :value="item" :depth="depth + 1" /></details><button v-if="limit < value.length" class="room-text-button" type="button" @click="limit += 30">展开更多 · {{ limit }} / {{ value.length }}</button></div>
  <dl v-else-if="typeof value === 'object' && depth < 12" class="remote-value-map"><template v-for="(item, key) in value" :key="key"><dt>{{ fieldLabels[key] || key }}</dt><dd><span v-if="isSecretField(key) || key === 'value' && isSecretField(value.key || '')">敏感内容已隐藏</span><RemoteValue v-else :value="item" :depth="depth + 1" /></dd></template></dl>
  <span v-else class="remote-value-text">{{ typeof value === 'boolean' ? value ? '开启' : '关闭' : String(value) }}</span>
</template>
<script setup>
import { fieldLabels, isSecretField } from '../../services/remoteManagement'
import { ref, reactive } from 'vue'
const limit = ref(30), expanded = reactive({})
defineProps({ value: { default: null }, depth: { type: Number, default: 0 } })
</script>
