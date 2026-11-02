<template>
  <RoomDrawer ref="page" variant="page" :title="tr('背景与玻璃')" :busy="editor?.busy" :before-close="beforeClose" @close="$emit('close')">
    <BackgroundEditor ref="editor" class="background-page-editor" :agent="agent" :palette="palette" :room-image="roomImage" />
  </RoomDrawer>
  <RoomDrawer v-if="confirming" :title="tr('保留这次调整吗？')" :busy="editor?.busy" @close="confirming = false">
    <p class="remote-help">{{ tr('预览还没有保存，返回前可以保存或放弃这次调整。') }}</p>
    <template #footer><div class="renewal-leave-actions"><button type="button" class="room-text-button" :disabled="editor?.busy" @click="discard">{{ tr('放弃修改') }}</button><button type="button" class="room-secondary-button" :disabled="editor?.busy" @click="confirming = false">{{ tr('继续调整') }}</button><button type="button" class="room-primary-button" :disabled="editor?.busy" @click="saveAndClose">{{ tr('保存并返回') }}</button></div></template>
  </RoomDrawer>
</template>
<script setup>
import { ref, nextTick } from 'vue'
import BackgroundEditor from './BackgroundEditor.vue'
import RoomDrawer from './RoomDrawer.vue'
import { t } from '../../i18n/remote'
defineProps({ agent: Object, palette: String, roomImage: String })
defineEmits(['close'])
const tr = key => t(key, [], 'renewal')
const editor = ref(null), page = ref(null), confirming = ref(false)
let discarded = false
function beforeClose() { if (!discarded && editor.value?.dirty) { confirming.value = true; return false }; return true }
async function discard() { editor.value?.cancel(); discarded = true; confirming.value = false; await nextTick(); page.value?.requestClose() }
async function saveAndClose() {
  await editor.value?.save()
  confirming.value = false
  await nextTick()
  if (!editor.value?.dirty) page.value?.requestClose()
}
</script>
