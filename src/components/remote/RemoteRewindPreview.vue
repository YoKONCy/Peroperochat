<template>
  <section class="remote-rewind-preview">
    <div class="remote-impact-counts"><span><strong>{{ preview.pairCount ?? 0 }}</strong>轮对话</span><span><strong>{{ preview.createdCount ?? 0 }}</strong>个新建文件</span><span><strong>{{ preview.editedCount ?? 0 }}</strong>个恢复文件</span><span><strong>{{ preview.preservedCount ?? 0 }}</strong>项保留改动</span></div>
    <p>{{ preview.wholeThread ? '清空这段会话记录，保留文件。' : regenerating ? '撤回本轮及之后的对话，处理对应文件，再使用本轮输入重新生成。' : '撤回目标轮及之后的对话，并按清单恢复工作区文件。' }}</p>
    <ul v-if="preview.files?.length" class="remote-rewind-files"><li v-for="(file, index) in preview.files" :key="index"><RemoteIcon :name="file.action === 'delete_created' ? 'trash' : file.action === 'preserve_changed' ? 'shield' : 'refresh'" /><span>{{ file.path }}<small>{{ ({ delete_created: '撤回新建文件', restore_edited: '恢复编辑前的文件', restore_deleted: '恢复删除的文件', restore_renamed: '恢复重命名前的文件', preserve_changed: '保留后续改动', missing_snapshot: '恢复记录缺失，保留', incomplete_snapshot: '恢复记录不完整，保留', unavailable: '无法安全读取，保留' })[file.action] || '按服务端清单处理' }}{{ file.preservedReason ? ' · ' + ({ content_changed: '内容已有后续变化', history_gap: '历史快照缺失', incomplete_snapshot: '快照不完整', unavailable: '无法安全读取' })[file.preservedReason] : '' }}</small></span></li></ul>
    <p class="remote-help">外部操作不保证可撤销。清单变化后需要重新确认。</p>
  </section>
</template>
<script setup>
import RemoteIcon from './RemoteIcon.vue'
defineProps({ preview: { type: Object, required: true }, regenerating: Boolean })
</script>
