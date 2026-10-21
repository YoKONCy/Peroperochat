# 对话体验一期实装记录

本轮依据 `docs/design/chat-phase-one/` 接入正式 Vue 对话页面，仅修改 PeroperoChat，未修改 infOS。实装内容随 1.4.5 发布，版本元数据与 `v1.4.5` 标签保持一致。

## 视觉与交互

- 定稿的 32 个受信任 SVG 图标进入应用常量，增补公式、图表、表格和播放操作；无 emoji 图标。
- 浅／暗主题继续复用配色包与角色外观配置，不将 Pero／Nana 身份、头像或台词固定在组件里。
- 消息头像行与角色正文气泡、云签式思考、开放式工具轨迹、NIT 子调用、暖色审批和叠纸折叠队列分别处理。
- 输入区保留已接通的模型、思考强度、执行与工具配置；修改的是形式、密度、热区及状态反馈。
- 抽屉、页面与按压使用短过渡；系统及应用减少动态设置均可关闭。保留已有草稿、队列修订、回滚与重新生成的业务处理，不新增单条消息删除。

## 渲染链路

`RemoteRichText` 持有每个正文节点的增量分段器。稳定块分别交给 `RemoteRichBlock`，活跃尾部使用 `RemoteStreamText` 追加文本节点；不再为每次正文增量整段替换 Markdown DOM。停止或失败后，未闭合围栏、公式、HTML 保留原文。

稳定块支持 Markdown、代码面板、公式、Mermaid、HTML／SVG 和独立资源引用。代码按需高亮，可复制、换行、展开；表格固定首列并局部横滑；公式使用 KaTeX，图表使用 Mermaid 严格模式和二次净化，可展开缩放。解析失败不阻断其他块。

HTML 正文使用独立无同源 iframe 与 CSP，自包含预览不能读取网络或应用原生桥；文件 HTML 可通过原生层申请 infOS 已有短期票据，使用服务端限制的相对工作区资源。前端不接收设备凭据。隐藏或离开可见区后卸载 iframe；后续重新可见时按需要重新加载，不在流式增量中反复初始化。

异步排版和媒体加载通知主会话维护滚动位置；阅读历史时显示回到底部入口，不强制跟随。工具与思考仍来自真实 Surface，未增加模型进度推测或自动概述。

## 文件入口与边界

`RemoteResourceCard` 将附件、正文资源引用、`send_file` 返回及读图／截图结果连接到同一套预览。图片／GIF、音频、视频在消息内查看；文档留出固定小窗，点击跳到 `RemoteResource` 阅读面板。MD 文件中的后续资源引用也能继续打开。

新增 Rust 转发：

- `remote_resolve_resource` → `/api/attachments/resolve-reference`，使用会话上下文解析来源。
- `Resource::Reference` → `/api/attachments/reference-content`，仅允许已解析的工作区引用。
- `remote_html_preview` → `/api/attachments/browser-preview`，只接受受控资源，校验 `/html-previews/` 随机票据格式。

所有私有资源沿用模式令牌与原生鉴权；公开外链不携带设备授权。无法解码、权限失效、跨域失败、损坏、加密或超限均保留原件／系统打开入口。各格式实际能力与大小限制见 `docs/REMOTE_FILES.md`，内容速览不代表完整原始排版。

依赖新增固定版本 `highlight.js@11.11.1`、`katex@0.18.4`、`mermaid@11.16.1`，与所参考 infOS 前端一致。锁文件和本地依赖已同步，安装使用 `--ignore-scripts`，没有执行依赖构建脚本。

## 本轮验证状态

2026-10-22 按用户新增授权运行 `corepack pnpm@10.34.7 test`，前端 **86 项全部通过**；运行 `cargo test --manifest-path src-tauri/Cargo.toml --lib`，Rust 后端 **103 项全部通过**。前端测试包含全部 Vue 组件与模板的内存校验，不生成应用构建产物。

未执行独立类型／静态检查、本地应用构建、打包、Tauri 启动或真机联调。现有测试不等于新增富文本、DOC 提取、HTML 票据、媒体小窗在真实 infOS 与各平台上已经完成端到端验收；视觉、帧率和文档兼容性仍需对应环境验证。
