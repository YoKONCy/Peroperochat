<template>
  <RoomDrawer :title="translate('消息排版示例')" variant="page" @close="$emit('close')">
    <article class="journal-reading" :data-journal="scheme" :style="journalVariables(scheme)">
      <nav class="journal-reading-tabs" :aria-label="translate('示例分类')">
        <button v-for="(item, index) in headings" :key="item" type="button" :aria-pressed="page === index" @click="page = index">{{ translate(item) }}</button>
      </nav>
      <template v-if="page === 0">
        <JournalMessage :source="'### ' + translate('沿河走到柳桥') + '\n\n' + translate('从**东岸步道**走到柳桥。到了桥头，再决定要不要绕去旧书店。')" />
        <figure><button type="button" :aria-label="translate('放大柳桥速写')" @click="zoom = true"><img :src="bridge" :alt="translate('柳桥与旧书店')" width="800" height="534" /></button><figcaption>{{ translate('柳桥与旧书店') }}</figcaption></figure>
        <JournalMessage :source="quote" />
      </template>
      <JournalMessage v-else :key="page" :source="page === 1 ? routes : calculation" />
    </article>
    <RoomDrawer v-if="zoom" :title="translate('柳桥与旧书店')" @close="zoom = false"><img class="journal-enlargement" :src="bridge" :alt="translate('柳桥与旧书店')" width="800" height="534" /></RoomDrawer>
  </RoomDrawer>
</template>
<script setup>
import { computed, ref } from 'vue'
import RoomDrawer from './RoomDrawer.vue'
import JournalMessage from './JournalMessage.vue'
import bridge from '../../assets/journal/willow-bridge.webp'
import { journalVariables } from '../../utils/uiPalette'
const props = defineProps({ scheme: String, translate: { type: Function, required: true } })
defineEmits(['close'])
const headings = ['图文手记', '路线对照', '代码与公式']
const page = ref(0), zoom = ref(false)
const quote = computed(() => '> ' + props.translate('想歇脚的时候，桥下有一张朝着河面的长椅。') + '\n\n- ' + props.translate('带上水和折叠伞。') + '\n- ' + props.translate('喜欢拍照的话，也带上相机。'))
const fence = '```'
const routes = computed(() => {
  const tr = props.translate
  return [
    '### ' + tr('今天想走多远？'), '',
    '| ' + [tr('走法'), tr('全程'), tr('步行')].join(' | ') + ' |', '| --- | ---: | ---: |',
    '| ' + tr('到桥折返') + ' | 1.6 km | 24 min |', '| ' + tr('绕去书店') + ' | 2.6 km | 39 min |', '',
    fence + 'mermaid', 'flowchart TD', 'A["' + tr('东岸步道') + '"] --> B["' + tr('柳桥') + '"]',
    'B --> C["' + tr('到桥折返') + '"]', 'B --> D["' + tr('绕去书店') + '"]', fence,
  ].join('\n')
})
const calculation = computed(() => [
  '### ' + props.translate('算算步行时间'), '', '$$', '\\frac{2.6}{4} \\times 60 = 39\\,\\mathrm{min}', '$$', '',
  fence + 'javascript', 'const minutes = (km, speed = 4) =>', '  Math.round(km / speed * 60)', '', 'minutes(2.6)', fence, '',
  fence + 'html', '<article><h3>' + props.translate('到桥折返') + '</h3><p>1.6 km · 24 min</p></article>', fence,
].join('\n'))
</script>
