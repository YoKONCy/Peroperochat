<template>
  <div class="phase2 phase-management-home">
    <section class="phase-welcome"><span class="phase-eyeline"><i class="phase-dot" />{{ profileName || 'infOS' }}</span><h2>{{ t('把日常，调成喜欢的样子。') }}</h2><p>{{ t('角色、模型和陪伴，都在手边。') }}</p><RemoteIcon name="ribbon" artwork class="phase-welcome-art" /></section>
    <label class="phase-search"><RemoteIcon name="search" /><input v-model="search" :placeholder="t('输入关键词查找')" :aria-label="t('输入关键词查找')" /></label>
    <template v-for="group in groups" :key="group.id"><h3 v-if="group.items.length" class="phase-section-title"><RemoteIcon :name="group.icon" />{{ t(group.title) }}</h3><div :class="group.id === 'daily' ? 'phase-quick-grid' : group.id === 'insights' ? 'phase-insight-links' : 'phase-destinations'"><button v-for="item in group.items" :key="item.id" type="button" @click="$emit('open', item.id)"><span class="phase-tile-icon"><RemoteIcon :name="item.icon" /></span><span><strong>{{ t(item.label) }}</strong><small>{{ t(item.description) }}</small></span><RemoteIcon name="next" /></button></div></template>
    <p v-if="!groups.some(group => group.items.length)" class="phase-empty">{{ t('没有找到对应内容') }}</p>
  </div>
</template>
<script setup>
import { computed, ref } from 'vue'
import RemoteIcon from './RemoteIcon.vue'
import { t } from '../../i18n/remote'
defineProps({ profileName: String })
defineEmits(['open'])
const search = ref('')
const groups = computed(() => [
  { id: 'daily', title: '常用设置', icon: 'settings', items: [
    { id: 'models', label: '模型与连接', description: '默认模型、角色覆盖与视觉', icon: 'model' },
    { id: 'agents', label: '角色资料', description: '称呼、人设与互动', icon: 'people' },
    { id: 'proactive', label: '主动陪伴', description: '联系的节奏', icon: 'heart' },
    { id: 'appearance', label: '外观与语言', description: '本机显示与界面语言', icon: 'palette' },
  ] },
  { id: 'more', title: '更多日常配置', icon: 'route', items: [
    { id: 'assignments', label: '模型分工', description: '默认模型、角色覆盖与视觉', icon: 'route' },
    { id: 'prompts', label: '提示词', description: '编辑、整理与发布', icon: 'document' },
    { id: 'voice', label: '语音', description: '识别与朗读', icon: 'audio' },
    { id: 'user', label: '个人资料', description: '界面语言', icon: 'profile' },
  ] },
  { id: 'insights', title: '实例概况', icon: 'pulse', items: [
    { id: 'usage', label: '用量统计', description: '累计用量', icon: 'chart' },
    { id: 'runtime', label: '运行概览', description: '当前在忙什么', icon: 'pulse' },
  ] },
].map(group => ({ ...group, items: group.items.filter(item => !search.value.trim() || `${t(item.label)} ${t(item.description)}`.toLowerCase().includes(search.value.trim().toLowerCase())) })))
</script>
