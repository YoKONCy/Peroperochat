<template>
  <div class="app"><router-view /></div>
</template>

<script setup>
</script>

<style>
:root { color-scheme: dark light }
* { box-sizing: border-box }
body { margin: 0 }
.app { min-height: 100vh; color: #e6e8eb; position: relative; overflow: hidden; background: linear-gradient(135deg, #3b82f6 0%, #a78bfa 50%, #f472b6 100%) }

/* Markdown 基础样式 */
.markdown-body {
  line-height: 1.5;
  word-wrap: break-word;
}
.markdown-body p { margin-top: 0; margin-bottom: 8px; }
.markdown-body p:last-child { margin-bottom: 0; }
.markdown-body strong { font-weight: 600; color: inherit; }
.markdown-body ul, .markdown-body ol { padding-left: 20px; margin-top: 0; margin-bottom: 8px; }
.markdown-body li { margin-bottom: 4px; }
.markdown-body code {
  padding: 0.2em 0.4em;
  margin: 0;
  font-size: 85%;
  background-color: rgba(0, 0, 0, 0.1);
  border-radius: 6px;
  font-family: ui-monospace, SFMono-Regular, SF Mono, Menlo, Consolas, Liberation Mono, monospace;
}
.markdown-body pre {
  padding: 12px;
  overflow: auto;
  font-size: 85%;
  line-height: 1.45;
  background-color: rgba(0, 0, 0, 0.15);
  border-radius: 8px;
  margin-bottom: 12px;
}
.markdown-body pre code {
  padding: 0;
  background-color: transparent;
  font-size: 100%;
}
.markdown-body h1, .markdown-body h2, .markdown-body h3 { margin-top: 16px; margin-bottom: 8px; font-weight: 600; line-height: 1.25; }
.markdown-body h1 { font-size: 1.5em; }
.markdown-body h2 { font-size: 1.25em; }
.markdown-body h3 { font-size: 1.1em; }

/* 危险重置弹窗样式与抖动效果 */
.danger-reset-box {
  animation: dangerShake 0.6s cubic-bezier(.175,.885,.32,1.275) 2 both;
  border-radius: 14px;
  border: 1px solid rgba(248,113,113,0.45) !important;
  box-shadow: 0 18px 42px rgba(244,63,94,0.35), inset 0 0 0 1px rgba(255,255,255,0.08) !important;
  backdrop-filter: blur(10px);
  background: linear-gradient(180deg, rgba(20,12,16,0.92), rgba(16,10,14,0.9)) padding-box,
              linear-gradient(135deg, rgba(244,63,94,0.85), rgba(251,113,133,0.85)) border-box !important;
}
.danger-reset-box .el-message-box__title {
  color: #fecaca;
  letter-spacing: 0.6px;
}
.danger-reset-box .el-message-box__message {
  color: #ffeef1;
  white-space: pre-wrap;
}
.danger-reset-box .danger-main-text {
  font-weight: 700;
  font-size: 16px;
  line-height: 1.6;
  color: #ffeef1;
}
.danger-reset-box .danger-sub-text {
  margin-top: 8px;
  font-size: 13px;
  line-height: 1.6;
  color: #fecaca;
}
.danger-reset-box .el-input__wrapper {
  background: linear-gradient(135deg, rgba(255,255,255,0.18), rgba(255,255,255,0.06));
  box-shadow: 0 6px 16px rgba(0,0,0,0.18);
}
.danger-reset-box .el-button--primary {
  background: linear-gradient(135deg, #ef4444, #f43f5e);
  border-color: rgba(255,255,255,0.18);
}
.danger-reset-box .el-button:not(.el-button--primary) {
  background: transparent;
  border-color: rgba(255,255,255,0.18);
  color: #ffeef1;
}
@keyframes dangerShake {
  0%, 100% { transform: translate3d(0,0,0) }
  20% { transform: translate3d(-4px, 0, 0) }
  40% { transform: translate3d(4px, 0, 0) }
  60% { transform: translate3d(-3px, 0, 0) }
  80% { transform: translate3d(3px, 0, 0) }
}
</style>
