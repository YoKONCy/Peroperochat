<template>
  <section class="remote-display-settings phase2 phase-appearance">
    <nav class="phase-tabs"><button :class="{ selected: panel === 'appearance' }" :aria-pressed="panel === 'appearance'" @click="panel = 'appearance'"><RemoteIcon name="palette" />{{ t('本机外观') }}</button><button :class="{ selected: panel === 'language' }" :aria-pressed="panel === 'language'" @click="panel = 'language'"><RemoteIcon name="language" />{{ t('界面语言') }}</button></nav>
    <RemoteLanguageSetting v-if="panel === 'language'" />
    <div v-show="panel === 'appearance'">
    <BackgroundEditor :agent="agent" :palette="display.palette" />
    <RemoteFeedback :message="error" /><RemoteFeedback :message="saveError" :action-label="t('重试')" @action="$emit('save')" />
    <fieldset :disabled="!ready || saving || importing"><legend>{{ t('界面主题') }}</legend><div class="remote-theme-options"><button v-for="item in themes" :key="item.id" :class="{ selected: display.theme === item.id }" @click="change('theme', item.id)"><RemoteIcon :name="item.icon" /><span>{{ t(item.name) }}</span></button></div></fieldset>
    <fieldset :disabled="!ready || saving || importing"><legend>{{ t('配色包') }}<button class="room-text-button" @click="change('palette', 'standard')">{{ t('使用标准配色') }}</button></legend><div class="remote-palette-grid"><button v-for="palette in remotePalettes" :key="palette.id" :aria-pressed="display.palette === palette.id" :class="{ selected: display.palette === palette.id }" @click="change('palette', palette.id)"><span class="remote-palette-sample"><i :style="{ background: palette.light[1], color: palette.light[0] }" /><i :style="{ background: palette.dark[1], color: palette.dark[0] }" /></span><strong>{{ t(palette.name) }}</strong></button></div></fieldset>
    <section class="phase-card"><h3><RemoteIcon name="document" />{{ t('正文字号') }}</h3><label class="phase-font-slider"><span>{{ fontPreview }} px</span><input v-model.number="fontPreview" type="range" min="13" max="18" :aria-label="t('正文字号')" :disabled="!ready || saving" @change="change('fontSize', fontPreview)" /></label><div class="phase-font-preview" :style="{ fontSize: `${fontPreview}px` }"><strong>{{ t('一小段预览') }}</strong><p>{{ t('慢慢来，今天也可以留一点空白。') }}</p></div></section>
    <div class="remote-preference-row"><span><strong>{{ t('首页显示角色 CG') }}</strong><small>{{ t('没有 CG 时使用头像布局') }}</small></span><RoomSwitch :model-value="display.showCg" :label="t('首页显示角色 CG')" :disabled="!ready || saving" @update:model-value="change('showCg', $event)" /></div>
    <div class="remote-preference-row"><span><strong>{{ t('减少交互动效') }}</strong><small>{{ t('始终尊重系统的减少动效设置') }}</small></span><RoomSwitch :model-value="display.reduceMotion" :label="t('减少交互动效')" :disabled="!ready || saving" @update:model-value="change('reduceMotion', $event)" /></div>
    <template v-if="agent?.id"><h3>{{ agent.name }} · {{ t('本机角色外观') }}</h3><p class="remote-help">{{ t('不修改服务端的人设、头像和其他客户端。') }}</p><div class="remote-appearance-preview"><img v-if="cg" :src="cg" :alt="`${agent.name}的 CG`" /><RemoteAvatar v-else :id="agent.id" :label="agent.name" /><div><strong>{{ t(character.cg ? '已选择本机 CG' : cg ? '内置角色 CG' : '使用服务端头像') }}</strong><small>{{ t('聊天中保持紧凑，点击展开按钮查看大图。') }}</small></div></div>
      <div class="remote-actions"><button class="room-secondary-button" :disabled="importing" @click="cgInput.click()"><RemoteIcon name="image" />{{ t('选择 CG') }}</button><button v-if="character.cg" class="room-text-button" :disabled="importing" @click="updateCharacter({ ...character, cg: null })">{{ t('恢复默认形象') }}</button><input ref="cgInput" type="file" accept="image/png,image/jpeg,image/webp" hidden @change="importCg" /></div><p class="remote-help">{{ t('支持 PNG、JPEG、WebP，单张不超过 2 MiB。') }}</p>
      <details class="phase-fold"><summary><RemoteIcon name="palette" />{{ t('角色样式包') }}<RemoteIcon name="down" /></summary><p class="remote-help">{{ t('选择你制作的样式包，调整配色与装饰。') }}</p><div class="remote-actions"><button class="room-secondary-button" :disabled="importing" @click="styleInput.click()"><RemoteIcon name="palette" />{{ t('导入样式包') }}</button><button class="room-text-button" :disabled="importing" @click="example">{{ t('导出制作模板') }}</button><input ref="styleInput" type="file" accept="application/json,.json" hidden @change="importStyle" /></div><div v-if="character.style" class="remote-preference-row"><span><strong>{{ character.style.name }}</strong><small>{{ t('未覆盖的部分沿用标准主题') }}</small></span><RoomSwitch :model-value="display.roleStyle" :label="t('启用角色样式包')" :disabled="saving" @update:model-value="change('roleStyle', $event)" /></div><button v-if="character.style" class="room-text-button" :disabled="importing" @click="updateCharacter({ ...character, style: null })">{{ t('移除本机样式包') }}</button></details>
    </template><p v-if="saving || importing" class="remote-help" role="status">{{ t('正在保存显示偏好…') }}</p></div>
  </section>
</template>
<script setup>
import RemoteFeedback from './RemoteFeedback.vue'
import BackgroundEditor from '../companion/BackgroundEditor.vue'
import RoomSwitch from '../companion/RoomSwitch.vue'
import { computed, ref, onBeforeUnmount, watch } from 'vue'
import RemoteLanguageSetting from './RemoteLanguageSetting.vue'
import { t } from '../../i18n/remote'
import RemoteIcon from './RemoteIcon.vue'
import RemoteAvatar from './RemoteAvatar.vue'
import { remotePalettes, characterCg } from '../../services/remoteTheme'
const props = defineProps({ display: Object, character: Object, agent: Object, ready: Boolean, saving: Boolean, saveError: String, saveCharacter: { type: Function, required: true } })
const emit = defineEmits(['change', 'save'])
const panel = ref('appearance'), fontPreview = ref(props.display.fontSize || 14)
watch(() => props.display.fontSize, value => { fontPreview.value = value || 14 })
const cgInput = ref(null), styleInput = ref(null), importing = ref(false), error = ref('')
const cg = computed(() => characterCg(props.agent, props.character))
const themes = [{ id: 'light', name: '标准浅色', icon: 'sun' }, { id: 'dark', name: '标准暗色', icon: 'moon' }, { id: 'system', name: '跟随系统', icon: 'settings' }]
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
async function importStyle(event) {
  const file = event.target.files?.[0]; event.target.value = ''; if (!file) return
  const snapshot = { ...props.character }, id = props.agent?.id
  importing.value = true; error.value = ''
  try {
    if (file.size > 32 * 1024) throw new Error('样式配置不能超过 32 KiB')
    const value = JSON.parse(await file.text())
    if (!value || typeof value !== 'object' || Array.isArray(value) || value.schemaVersion !== 1 || !value.style || Object.keys(value).some(key => !['schemaVersion', 'style'].includes(key))) throw new Error('请使用导出的角色样式模板')
    if (!disposed && props.agent?.id === id) { await props.saveCharacter({ ...snapshot, style: value.style }); change('roleStyle', true) }
  } catch (failure) { if (!disposed) error.value = failure.message } finally { if (!disposed) importing.value = false }
}
function example() {
  const value = { schemaVersion: 1, style: { name: '我的角色样式', lightAccent: '#467fc5', darkAccent: '#c2a7ef', secondary: '#d79bb8', bubble: 'ribbon', decoration: 'constellation' } }
  const url = URL.createObjectURL(new Blob([JSON.stringify(value, null, 2)], { type: 'application/json' }))
  const link = document.createElement('a'); link.href = url; link.download = 'peroperochat-character-style.json'; link.click(); setTimeout(() => URL.revokeObjectURL(url), 1000)
}
onBeforeUnmount(() => { disposed = true })
</script>
