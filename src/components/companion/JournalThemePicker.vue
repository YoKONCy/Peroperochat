<template>
  <h3 class="journal-heading">{{ translate('界面主题') }}</h3>
  <div class="journal-presets" role="group" :aria-label="translate('界面主题')">
    <button v-for="(preset, id) in journalThemes" :key="id" type="button" class="journal-preset" :aria-pressed="modelValue === id" @click="$emit('update:modelValue', id)">
      <i aria-hidden="true" :style="{ backgroundColor: preset.page, color: preset.accent, borderColor: preset.line }"><b /><b /><b /></i>
      <strong>{{ translate(preset.label) }}</strong><RoomIcon v-if="modelValue === id" name="check" />
    </button>
    <button v-if="followSystem" type="button" class="journal-system-choice" :aria-pressed="modelValue === 'system'" @click="$emit('update:modelValue', 'system')">{{ translate('跟随系统') }}<RoomIcon v-if="modelValue === 'system'" name="check" /></button>
  </div>
  <button type="button" class="journal-read-entry" @click="reading = true"><RoomIcon name="notebook" />{{ translate('消息排版示例') }}<RoomIcon name="next" /></button>
  <JournalReadingSample v-if="reading" :scheme="scheme" :translate="translate" @close="reading = false" />
</template>
<script setup>
import { computed, defineAsyncComponent, inject, ref } from 'vue'
import { REMOTE_THEME } from '../../services/remoteTheme'
import RoomIcon from './RoomIcon.vue'
import { journalThemes } from '../../utils/uiPalette'
const props = defineProps({ modelValue: String, followSystem: Boolean, translate: { type: Function, default: value => value } })
defineEmits(['update:modelValue'])
const JournalReadingSample = defineAsyncComponent(() => import('./JournalReadingSample.vue'))
const remoteTheme = inject(REMOTE_THEME, null)
const scheme = computed(() => props.modelValue === 'system' ? remoteTheme?.value?.mode || 'light' : props.modelValue)
const reading = ref(false)
</script>
