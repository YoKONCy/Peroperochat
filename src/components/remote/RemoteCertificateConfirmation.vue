<template>
  <RoomDrawer :title="tr(inspection.previousFingerprint ? '服务器证书有变化' : '确认服务器证书')" @close="$emit('cancel')">
    <section class="certificate-confirmation">
      <div class="certificate-intro"><RemoteIcon name="shield" /><div><strong>{{ inspection.origin }}</strong><p>{{ tr('请在电脑端核对指纹，相同后再继续。') }}</p></div></div>
      <div class="certificate-fingerprint"><span>SHA-256</span><code>{{ inspection.certificate.fingerprint }}</code></div>
      <details v-if="inspection.previousFingerprint"><summary>{{ tr('上次的指纹') }}</summary><code>{{ inspection.previousFingerprint }}</code></details>
      <details><summary>{{ tr('证书详情') }}</summary><dl><dt>{{ tr('签发给') }}</dt><dd>{{ inspection.certificate.subject }}</dd><dt>{{ tr('签发者') }}</dt><dd>{{ inspection.certificate.issuer }}</dd><dt>{{ tr('有效期至') }}</dt><dd>{{ remoteDate(inspection.certificate.validTo) }}</dd></dl></details>
      <label class="certificate-check"><input v-model="confirmed" type="checkbox" /><span>{{ tr('已与电脑端核对指纹') }}</span></label>
      <small>{{ tr('信任只保存在此设备，证书变化时会再次询问。') }}</small>
    </section>
    <template #footer><div class="certificate-buttons"><RemoteAction icon="close" @click="$emit('cancel')">{{ tr('取消') }}</RemoteAction><RemoteAction icon="shield" tone="primary" :disabled="!confirmed" @click="$emit('confirm')">{{ tr('信任并继续') }}</RemoteAction></div></template>
  </RoomDrawer>
</template>
<script setup>
import { ref } from 'vue'
import RoomDrawer from '../companion/RoomDrawer.vue'
import RemoteIcon from './RemoteIcon.vue'
import RemoteAction from './RemoteAction.vue'
import { t, remoteDate } from '../../i18n/remote'
defineProps({ inspection: { type: Object, required: true } })
defineEmits(['cancel', 'confirm'])
const confirmed = ref(false)
const tr = source => t(source, [], 'certificates')
</script>
<style scoped>
.certificate-confirmation{display:grid;gap:16px;min-width:0;font-size:13px;line-height:1.65;color:var(--room-ink)}
.certificate-intro{display:flex;gap:12px;align-items:flex-start}.certificate-intro>svg{width:28px;height:28px;flex:none;color:var(--room-accent)}.certificate-intro>div{min-width:0}.certificate-intro strong{overflow-wrap:anywhere}.certificate-intro p{margin:6px 0 0;color:var(--room-muted);font-size:12px}
.certificate-fingerprint{padding:15px;border:1px solid var(--room-line);border-radius:12px 20px 12px 12px;background:var(--room-accent-soft)}.certificate-fingerprint>span{display:block;margin-bottom:7px;font-size:10px;color:var(--room-accent);letter-spacing:.1em}
code{display:block;font:12px/1.9 ui-monospace,monospace;overflow-wrap:anywhere;user-select:text}.certificate-fingerprint code{color:var(--room-ink)}
details{min-width:0}summary{cursor:pointer;color:var(--room-accent);padding-block:6px;font-size:12px}details>code{color:var(--room-muted)}dl{display:grid;grid-template-columns:auto minmax(0,1fr);gap:7px 12px;font-size:11px}dt{color:var(--room-muted)}dd{margin:0;overflow-wrap:anywhere}
.certificate-check{display:flex;align-items:center;gap:10px;min-height:44px}.certificate-check input{appearance:none;flex:none;width:20px;height:20px;margin:0;border:1px solid var(--room-accent);border-radius:6px;background:var(--room-surface);display:grid;place-items:center}.certificate-check input:checked{background:var(--room-accent)}.certificate-check input:checked::after{content:'';width:5px;height:9px;border:solid var(--room-surface);border-width:0 2px 2px 0;transform:translateY(-1px) rotate(45deg)}input:focus-visible{outline:2px solid var(--room-accent);outline-offset:3px}
.certificate-confirmation>small{font-size:11px;color:var(--room-muted)}.certificate-buttons{display:flex;justify-content:flex-end;gap:10px}
</style>
