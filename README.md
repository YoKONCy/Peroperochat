<div align="center">

<h1>Peroperochat — 更懂你的 AI 伙伴</h1>

<img alt="Peroperochat" src="https://img.shields.io/badge/Python-3.10%2B-blue"> <img alt="Frontend" src="https://img.shields.io/badge/Node.js-18%2B-success"> <img alt="License" src="https://img.shields.io/badge/License-MIT-green">

<p>一个友好、可配置、带长记忆与人格评估的本地 AI 伙伴。支持 Live2D展示。</p>

</div>

---

**零门槛启动、易于扩展，写给热爱创造的你。**

---

## 简介

Peroperochat 是一个纯前端的开源 AI 伙伴项目：
- 前端基于 `Vue 3 + Vite + Element Plus`，提供聊天区、模型参数设置、系统/人设提示词、提示词查看器、长记忆中心与 Live2D 展示。
- **纯前端架构**：所有记忆读写、设置项管理均在浏览器本地完成（使用 IndexedDB），直接与 OpenAI 兼容的 API 接口通信，无需后端中转。

## 功能亮点
- 聊天区 Markdown 渲染、复制/编辑/重生成；支持流式与非流式回复。
- 模型参数可视化配置（温度/Top P/惩罚等），一键获取并筛选模型列表。
- 系统/人设/后置提示词管理，提示词查看器一键导出请求序列。
- **本地长记忆**：使用 Dexie.js (IndexedDB) 实现本地事件/爱好记录的查询、插入与删除。
- Live2D 控件：更换模型、随机换装；支持点击交互文本的本地化更新。

## 快速开始

前置要求：`Node.js 18+`。

1. **安装并启动项目**
   - 进入项目目录：`cd Peroperochat`
   - 安装依赖：`npm install`
   - 启动开发服务器：`npm run dev`
   - 默认访问地址：`http://localhost:5173`

2. **配置 API**
   - 在前端界面的“设置”中配置您的 `API 地址` 与 `API 秘钥`。
   - 即可开始与您的 AI 伙伴交流。

## 进阶用法

### 核心逻辑
- **API 通信**：前端直接调用指定的 `API 地址`。所有请求均符合 OpenAI 聊天补全接口规范。
- **本地存储**：
  - **IndexedDB (Dexie.js)**：用于存储长记忆数据（Memory），支持高效的查询与管理。
  - **LocalStorage**：用于存储用户信息、API 配置、模型参数、聊天记录以及 Live2D 的交互文本。

### 长记忆管理
长记忆中心现在完全运行在客户端。您可以：
- **查看列表**：从本地数据库加载所有记忆记录。
- **手动插入**：直接向本地数据库添加新的记忆条目。
- **删除记忆**：移除不再需要的本地记录。

### Live2D 交互更新
Live2D 挂件的点击交互文本现在通过 `localStorage` 键 `ppc.waifu.texts` 进行管理。当用户与 AI 聊天时，系统会根据上下文自动更新这些交互短句，实现更具个性化的反馈。

## 项目架构

```
Peroperochat/
├── src/
│   ├── App.vue                # 主界面与功能入口
│   ├── api.js                 # API 通信层（直接对接 OpenAI 接口）
│   ├── db.js                  # 本地数据库层 (Dexie.js)
│   ├── waifuService.js        # Live2D 交互文本处理逻辑
│   └── components/
│       ├── ChatArea.vue       # 聊天区组件
│       ├── MemoryCenter.vue   # 本地记忆管理组件
│       └── Live2DWidget.vue   # Live2D 挂件组件
├── public/live2d-widget/      # Live2D 资源与配置文件
├── package.json
└── vite.config.js
```

## 配置速查
- **API 配置**：在设置面板中填写，持久化于 `localStorage`。
- **模型参数**：支持温度（Temperature）、Top P、频率惩罚等参数的实时调整。
- **本地数据重置**：在设置中提供“重置所有本地数据”功能，一键清空 IndexedDB 与 LocalStorage。

