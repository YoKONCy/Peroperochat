<template>
  <Transition name="fade">
    <div class="overlay" @click.self="$emit('close')">
      <div class="panel">
        <div class="head">
          <div class="title-area">
            <span class="title">历史记录</span>
            <span class="count">{{ messages.length }} 条消息</span>
          </div>
          <button class="close-icon" @click="$emit('close')">
            <el-icon><Close /></el-icon>
          </button>
        </div>
        <div class="list">
          <div v-if="messages.length === 0" class="empty-state">
            <el-icon size="48" color="#94a3b8"><MessageBox /></el-icon>
            <p>暂无对话记录</p>
          </div>
          <!-- 默认只显示前 50 条，点击“显示更多”后全部显示 -->
          <div v-for="(m, i) in (showAll ? messages.slice().reverse() : messages.slice().reverse().slice(0, 50))" :key="messages.length - 1 - i" :class="['row', m.role]">
            <div class="row-header">
              <span class="role-tag">{{ m.role === 'user' ? '我' : 'Pero' }}</span>
              <span class="time">{{ formatTime(m.timestamp) }}</span>
            </div>
            <div class="content markdown-body">
              <div v-if="Array.isArray(m.content)" class="multimodal-content">
                <template v-for="(part, pi) in m.content" :key="pi">
                  <div v-if="part.type === 'text'" v-html="renderMarkdown(part.text)"></div>
                  <div v-else-if="part.type === 'image_url'" class="message-image-container">
                    <img :src="part.image_url.url" class="message-image" @click="previewImage(part.image_url.url)" />
                  </div>
                </template>
              </div>
              <div v-else v-html="renderMarkdown(m.content)"></div>
            </div>
            <div class="row-tools">
              <el-tooltip content="复制内容" placement="top" :hide-after="1000">
                <button class="tool-btn" @click="$emit('copy', m)"><el-icon size="14"><CopyDocument /></el-icon></button>
              </el-tooltip>
              <el-tooltip content="重新生成" placement="top" v-if="m.role === 'assistant'">
                <button class="tool-btn" @click="$emit('regenerate', messages.length - 1 - i)"><el-icon size="14"><Refresh /></el-icon></button>
              </el-tooltip>
              <el-tooltip content="删除消息" placement="top">
                <button class="tool-btn delete" @click="$emit('delete', messages.length - 1 - i)"><el-icon size="14"><Delete /></el-icon></button>
              </el-tooltip>
            </div>
          </div>
          
          <!-- 显示更多按钮 -->
          <div v-if="!showAll && messages.length > 50" class="more-area">
            <button class="more-btn" @click="showAll = true">
              显示更多 (还有 {{ messages.length - 50 }} 条)
              <el-icon><ArrowDown /></el-icon>
            </button>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup>
import { ref, defineProps, defineEmits } from 'vue'
import { Delete, Refresh, CopyDocument, Close, MessageBox, ArrowDown } from '@element-plus/icons-vue'
import { marked } from 'marked'
import DOMPurify from 'dompurify'

const props = defineProps({
  messages: {
    type: Array,
    default: () => []
  }
})

const emit = defineEmits(['close', 'delete', 'regenerate', 'copy'])
const showAll = ref(false)

// 图片预览逻辑
function previewImage(url) {
  window.open(url, '_blank')
}

// Markdown 渲染
function renderMarkdown(text) {
  if (!text) return ''
  const clean = cleanMessageContent(text)
  const html = marked.parse(clean)
  return DOMPurify.sanitize(html)
}

// 格式化时间
function formatTime(ts) {
  if (!ts) return ''
  const date = new Date(ts)
  return date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
}

// 清理消息中的隐藏标签
function cleanMessageContent(text) {
  if (!text) return ''
  if (text === '__loading__') return '正在思考...'
  
  // 移除所有 XML 标签及其内容 (因为现在使用 NIT 协议，XML 仅用于内部逻辑)
  // 同时也移除 NIT 调用块，保持历史记录纯净
  return text
    .replace(/<(nit(?:-[0-9a-fA-F]{4})?)>[\s\S]*?<\/\1>/gi, '')
    .replace(/<([A-Z_]+)>[\s\S]*?<\/\1>/g, '')
    .replace(/\[\[\[NIT_CALL\]\]\][\s\S]*?\[\[\[NIT_END\]\]\]/g, '')
    .trim()
}

</script>

<style scoped>
.overlay { 
  position: fixed; 
  inset: 0; 
  background: rgba(15, 23, 42, 0.4); 
  backdrop-filter: blur(8px); 
  -webkit-backdrop-filter: blur(8px);
  display: grid; 
  place-items: center; 
  z-index: 2000;
}

.panel { 
  width: 92vw; 
  max-width: 600px; 
  max-height: 85vh; 
  border-radius: 28px; 
  border: 1px solid rgba(255, 255, 255, 0.5); 
  background: rgba(255, 255, 255, 0.8); 
  backdrop-filter: blur(20px); 
  -webkit-backdrop-filter: blur(20px);
  color: #1e293b; 
  display: flex;
  flex-direction: column;
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
  overflow: hidden;
  animation: slideUp 0.4s cubic-bezier(0.16, 1, 0.3, 1);
}

@keyframes slideUp {
  from { opacity: 0; transform: translateY(20px) scale(0.95); }
  to { opacity: 1; transform: translateY(0) scale(1); }
}

.head { 
  display: flex; 
  align-items: center; 
  justify-content: space-between; 
  padding: 20px 24px; 
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
}

.title-area {
  display: flex;
  flex-direction: column;
}

.title {
  font-size: 18px;
  font-weight: 700;
  color: #0f172a;
}

.count {
  font-size: 12px;
  color: #64748b;
  margin-top: 2px;
}

.close-icon {
  width: 32px;
  height: 32px;
  border-radius: 16px;
  border: none;
  background: rgba(0, 0, 0, 0.05);
  color: #64748b;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s;
}

.close-icon:hover {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

.list { 
  flex: 1;
  padding: 16px 20px; 
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* 自定义滚动条 */
.list::-webkit-scrollbar { width: 5px; }
.list::-webkit-scrollbar-track { background: transparent; }
.list::-webkit-scrollbar-thumb { background: rgba(0, 0, 0, 0.1); border-radius: 10px; }

.more-area {
  padding: 12px 0;
  display: flex;
  justify-content: center;
}

.more-btn {
  background: rgba(255, 255, 255, 0.5);
  border: 1px solid rgba(0, 0, 0, 0.05);
  border-radius: 12px;
  padding: 8px 16px;
  font-size: 13px;
  color: #64748b;
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  transition: all 0.2s;
}

.more-btn:hover {
  background: white;
  color: #3b82f6;
  box-shadow: 0 2px 8px rgba(0,0,0,0.05);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 0;
  color: #94a3b8;
  gap: 12px;
}

.row { 
  border-radius: 18px; 
  padding: 16px; 
  word-break: break-word; 
  background: white; 
  border: 1px solid rgba(0, 0, 0, 0.03); 
  display: flex; 
  flex-direction: column; 
  gap: 10px;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.05);
  transition: transform 0.2s;
}

.row:hover {
  transform: translateY(-2px);
}

.row-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.role-tag {
  font-size: 12px;
  font-weight: 700;
  padding: 2px 10px;
  border-radius: 20px;
}

.row.user .role-tag {
  background: #eff6ff;
  color: #3b82f6;
}

.row.assistant .role-tag {
  background: #fdf2f8;
  color: #ec4899;
}

.time {
  font-size: 11px;
  color: #94a3b8;
}

.content {
  font-size: 14px;
  line-height: 1.6;
  color: #334155;
  white-space: pre-wrap;
}

.multimodal-content {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.message-image-container {
  margin: 4px 0;
  max-width: 100%;
}

.message-image {
  max-width: 100%;
  max-height: 300px;
  border-radius: 12px;
  cursor: zoom-in;
  border: 1px solid rgba(0,0,0,0.05);
  transition: transform 0.2s;
}

.message-image:hover {
  transform: scale(1.02);
}

.row-tools { 
  display: flex; 
  gap: 8px; 
  justify-content: flex-end; 
  border-top: 1px solid rgba(0, 0, 0, 0.05); 
  padding-top: 10px; 
}

.tool-btn { 
  width: 32px; 
  height: 32px; 
  display: flex; 
  align-items: center; 
  justify-content: center; 
  border-radius: 10px; 
  border: 1px solid rgba(0, 0, 0, 0.05); 
  background: #f8fafc; 
  color: #64748b; 
  cursor: pointer; 
  transition: all 0.2s; 
}

.tool-btn:hover { 
  background: #eff6ff; 
  border-color: #3b82f6; 
  color: #3b82f6; 
  transform: scale(1.1);
}

.tool-btn.delete:hover {
  background: #fef2f2;
  border-color: #ef4444;
  color: #ef4444;
}

/* 消息淡入动画 */
.fade-enter-active, .fade-leave-active { transition: opacity 0.3s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>
