<template><CharacterAvatar :src="avatar.source" :label="label || avatar.name || t('你')" :aria-busy="avatar.loading" fit="contain" @error="failed" /></template>
<script setup>
import { onMounted } from 'vue'
import CharacterAvatar from '../companion/CharacterAvatar.vue'
import { remoteUserAvatar as avatar, refreshRemoteUserAvatar, remoteUserAvatarImageFailed } from '../../services/remoteUserAvatar'
import { t } from '../../i18n/remote'
defineProps({ label: String })
let retried = false
function failed(source) { remoteUserAvatarImageFailed(source); if (!retried) { retried = true; refreshRemoteUserAvatar(true).catch(() => {}) } }
onMounted(() => refreshRemoteUserAvatar().catch(() => {}))
</script>
