<template><section class="phase2 phase-card"><h3><RemoteIcon name="language" />{{ t('infOS 界面语言') }}</h3><RemoteFeedback :message="error" /><RemoteFeedback :message="notice" tone="success" /><RemoteSelect :model-value="language" :label="t('界面语言')" :options="options" :disabled="busy || !loaded || unresolved" @update:model-value="change" /><p class="phase-context"><RemoteIcon name="globe" />{{ t('与当前 infOS 同步，也会影响连接它的其他客户端。') }}</p><button v-if="!loaded && !busy" class="room-text-button" @click="run(load)">{{ t('重试') }}</button></section></template>
<script setup>
import { computed, ref, onMounted, watch } from 'vue'
import RemoteSelect from './RemoteSelect.vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteFeedback from './RemoteFeedback.vue'
import { useRemoteManagement } from '../../composables/useRemoteManagement'
import { t, setRemoteLocale, normalizeRemoteLocale, remoteLocale } from '../../i18n/remote'
const { read, write, run, busy, error, unresolved } = useRemoteManagement()
const language = ref('zh-CN'), loaded = ref(false), notice = ref('')
watch(remoteLocale, value => { if (loaded.value && !busy.value) language.value = value })
const options = computed(() => [{ value: 'zh-CN', label: '简体中文' }, { value: 'zh-TW', label: '繁體中文' }, { value: 'en-US', label: 'English' }])
async function load() { const data = await read('configs.batch', { body: { keys: ['user.language'] } }); language.value = normalizeRemoteLocale(data?.['user.language']); loaded.value = true; setRemoteLocale(language.value) }
async function change(value) { if (value === language.value) return; await run(async () => { notice.value = ''; await write('configs.save_batch', { body: { items: [{ key: 'user.language', value }] } }); language.value = value; setRemoteLocale(value); notice.value = t('已保存') }) }
onMounted(() => run(load))
</script>
