<template>
  <section class="remote-display-settings phase2 phase-appearance">
    <nav class="phase-tabs"><button :class="{ selected: panel === 'appearance' }" :aria-pressed="panel === 'appearance'" @click="panel = 'appearance'"><RemoteIcon name="palette" />{{ t('本机外观') }}</button><button :class="{ selected: panel === 'language' }" :aria-pressed="panel === 'language'" @click="panel = 'language'"><RemoteIcon name="language" />{{ t('界面语言') }}</button></nav>
    <RemoteLanguageSetting v-if="panel === 'language'" />
    <div v-show="panel === 'appearance'">
    <button type="button" class="renewal-background-entry" @click="backgroundOpen = true"><RemoteIcon name="image" /><span><strong>{{ tr('背景与玻璃') }}</strong></span><RemoteIcon name="next" /></button>
    <RemoteFeedback :message="error" /><RemoteFeedback :message="saveError" :action-label="t('重试')" @action="$emit('save')" />
    <fieldset :disabled="!ready || saving || importing"><JournalThemePicker :model-value="display.theme" :translate="t" follow-system @update:model-value="change('theme', $event)" /></fieldset>

    <section class="phase-card"><h3><RemoteIcon name="document" />{{ t('正文字号') }}</h3><label class="phase-font-slider"><span>{{ fontPreview }} px</span><input v-model.number="fontPreview" type="range" min="13" max="18" :aria-label="t('正文字号')" :disabled="!ready || saving" @change="change('fontSize', fontPreview)" /></label><div class="phase-font-preview" :style="{ fontSize: `${fontPreview}px` }"><strong>{{ t('一小段预览') }}</strong><p>{{ t('慢慢来，今天也可以留一点空白。') }}</p></div></section>
    <div class="remote-preference-row"><span><strong>{{ t('首页显示角色 CG') }}</strong><small>{{ t('没有 CG 时使用头像布局') }}</small></span><RoomSwitch :model-value="display.showCg" :label="t('首页显示角色 CG')" :disabled="!ready || saving" @update:model-value="change('showCg', $event)" /></div>
    <div class="remote-preference-row"><span><strong>{{ t('减少交互动效') }}</strong><small>{{ t('始终尊重系统的减少动效设置') }}</small></span><RoomSwitch :model-value="display.reduceMotion" :label="t('减少交互动效')" :disabled="!ready || saving" @update:model-value="change('reduceMotion', $event)" /></div>
    <template v-if="agent?.id"><h3>{{ agent.name }} · {{ t('本机角色外观') }}</h3><p class="remote-help">{{ t('不修改服务端的人设、头像和其他客户端。') }}</p><div class="remote-appearance-preview"><img v-if="cg" :src="cg" :alt="`${agent.name}的 CG`" /><RemoteAvatar v-else :id="agent.id" :label="agent.name" /><div><strong>{{ t(character.cg ? '已选择本机 CG' : cg ? '内置角色 CG' : '使用服务端头像') }}</strong><small>{{ t('聊天中保持紧凑，点击展开按钮查看大图。') }}</small></div></div>
      <div class="remote-actions"><button class="room-secondary-button" :disabled="importing" @click="cgInput.click()"><RemoteIcon name="image" />{{ t('选择 CG') }}</button><button v-if="character.cg" class="room-text-button" :disabled="importing" @click="updateCharacter({ ...character, cg: null })">{{ t('恢复默认形象') }}</button><input ref="cgInput" type="file" accept="image/png,image/jpeg,image/webp" hidden @change="importCg" /></div><p class="remote-help">{{ t('支持 PNG、JPEG、WebP，单张不超过 2 MiB。') }}</p>

    </template><p v-if="saving || importing" class="remote-help" role="status">{{ t('正在保存显示偏好…') }}</p></div>
  </section>
  <BackgroundSettingsPage v-if="backgroundOpen" :agent="agent" @close="backgroundOpen = false" />
</template>
<script setup>
import RemoteFeedback from './RemoteFeedback.vue'
import BackgroundSettingsPage from '../companion/BackgroundSettingsPage.vue'
import RoomSwitch from '../companion/RoomSwitch.vue'
import { computed, ref, onBeforeUnmount, watch } from 'vue'
import RemoteLanguageSetting from './RemoteLanguageSetting.vue'
import { t } from '../../i18n/remote'
import RemoteIcon from './RemoteIcon.vue'
import RemoteAvatar from './RemoteAvatar.vue'
import JournalThemePicker from '../companion/JournalThemePicker.vue'
import { characterCg } from '../../services/remoteTheme'
const props = defineProps({ display: Object, character: Object, agent: Object, ready: Boolean, saving: Boolean, saveError: String, saveCharacter: { type: Function, required: true } })
const emit = defineEmits(['change', 'save'])
const tr = key => t(key, [], 'renewal'), backgroundOpen = ref(false)
const panel = ref('appearance'), fontPreview = ref(props.display.fontSize || 14)
watch(() => props.display.fontSize, value => { fontPreview.value = value || 14 })
const cgInput = ref(null), importing = ref(false), error = ref('')
const cg = computed(() => characterCg(props.agent, props.character))

let disposed = false
function change(key, value) { emit('change', { key, value }) }
async function updateCharacter(value) {
  if (importing.value) return
  importing.value = true; error.value = ''
  try { await props.saveCharacter(value) } catch (failure) { if (!disposed) error.value = failure.message }
  finally { if (!disposed) importing.value = false }
}
async function importCg(event) {
  const file = event.target.files?.[0]; event.target.value = ''; if (!file) return
  const snapshot = { ...props.character }, id = props.agent?.id
  importing.value = true; error.value = ''
  try {
    if (!['image/png', 'image/jpeg', 'image/webp'].includes(file.type) || file.size > 2 * 1024 * 1024) throw new Error('请选择不超过 2 MiB 的 PNG、JPEG 或 WebP 图片')
    const data = await new Promise((resolve, reject) => { const reader = new FileReader(); reader.onload = () => resolve(reader.result); reader.onerror = () => reject(new Error('读取 CG 失败')); reader.readAsDataURL(file) })
    if (!disposed && props.agent?.id === id) await props.saveCharacter({ ...snapshot, cg: data })
  } catch (failure) { if (!disposed) error.value = failure.message } finally { if (!disposed) importing.value = false }
}
onBeforeUnmount(() => { disposed = true })
</script>
