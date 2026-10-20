<template>
  <div class="remote-trace-text"><pre>{{ visible }}</pre><button v-if="text.length > limit" class="room-text-button" @click="expanded = !expanded">{{ expanded ? '收起长内容' : `展开完整内容（${text.length.toLocaleString()} 字符）` }}</button></div>
</template>
<script setup>
import { computed, ref } from 'vue'
import { traceText } from '../../utils/remoteTrace'
const props = defineProps({ value: [String, Object, Array, Number, Boolean], limit: { type: Number, default: 6000 } })
const expanded = ref(false)
const text = computed(() => traceText(props.value)), visible = computed(() => expanded.value ? text.value : text.value.slice(0, props.limit))
</script>
