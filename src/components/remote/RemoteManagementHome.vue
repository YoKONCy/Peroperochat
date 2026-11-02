<template>
  <div class="phase2 phase-management-home renewal-management">
    <section v-if="!category && !search" class="renewal-profile-strip"><RemoteAvatar v-if="agent?.id" :id="agent.id" :label="agent.name" /><span><strong>{{ agent?.name || tr('我的 infOS') }}</strong><small>{{ tr(online ? '已连接，随时接着聊。' : '离线中，已保存的内容仍可查看。') }}</small></span><i :data-online="online" /></section>
    <label class="phase-search"><RemoteIcon name="search" /><input v-model="search" :placeholder="tr('查找设置')" :aria-label="tr('查找设置')" /></label>
    <Transition name="renewal-category" mode="out-in">
      <div v-if="!category && !search" key="overview" class="renewal-management-grid"><button v-for="group in remoteManagementGroups" :key="group.id" type="button" :data-ui-tone="group.tone" @click="select(group.id)"><span class="renewal-category-symbol"><RemoteIcon :name="group.icon" /></span><strong>{{ tr(group.title) }}</strong><small>{{ tr(group.description) }}</small><RemoteIcon class="renewal-category-next" name="next" /></button></div>
      <div v-else :key="search ? 'search' : category"><div v-for="group in groups" :key="group.id" class="renewal-management-group" :data-ui-tone="group.tone"><h3 v-if="search" class="phase-section-title">{{ tr(group.title) }}</h3><div class="phase-destinations"><button v-for="item in group.items" :key="item.id" type="button" :data-ui-tone="group.tone" @click="open(group, item)"><span class="phase-tile-icon"><RemoteIcon :name="item.icon" /></span><span><strong>{{ tr(item.label) }}</strong><small>{{ tr(item.description) }}</small></span><RemoteIcon name="next" /></button></div></div><p v-if="!groups.length" class="phase-empty">{{ tr('没有找到对应内容') }}</p></div>
    </Transition>
  </div>
</template>
<script setup>
import { computed, ref, watch } from 'vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteAvatar from './RemoteAvatar.vue'
import { remoteManagementGroups } from '../../constants/remoteNavigation'
import { t } from '../../i18n/remote'
const props = defineProps({ agent: Object, online: Boolean, initialCategory: String })
const emit = defineEmits(['open', 'category-change'])
const tr = key => t(key, [], 'renewal')
const search = ref(''), category = ref(props.initialCategory || null)
const groups = computed(() => remoteManagementGroups.filter(group => search.value || group.id === category.value).map(group => ({ ...group, items: group.items.filter(item => !search.value || `${tr(item.label)} ${tr(item.description)}`.toLowerCase().includes(search.value.toLowerCase().trim())) })).filter(group => group.items.length))
function select(id) { category.value = id; search.value = '' }
function open(group, item) { select(group.id); emit('open', item.id) }
watch(category, value => emit('category-change', value), { immediate: true })
watch(() => props.initialCategory, value => { category.value = value || null })
function back() { if (search.value) search.value = ''; else select(null) }
defineExpose({ back, category, hasSubpage: computed(() => !!category.value || !!search.value) })
</script>
