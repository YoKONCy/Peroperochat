# 1.4.1 实装记录

## 角色显示与交互

Pero 的静态立绘框在原比例上放大约 10%，同时限制顶部可用空间；Nana 的独立空间改用 `public/companion/nana-standing.png`。仅自动迁移原有默认 `/companion/nana-cg.png + card` 组合，用户自定义素材、排版与显式空值保留。远程首页仍允许使用完整 CG。

房间首页新增带双门 SVG 的「前往 infOS」切换卡；远程首页提供对应返回入口。点击进入既有连接／模式面板，切换仍由 Rust 决定运行权，沿用草稿保护与至少两秒的过渡页。

性能优化集中在减少已知重绘与重复计算：

- 固定人物基础绘制尺寸，展开对话改用 transform 与 opacity，取消逐帧改变尺寸、顶部和大背景的过渡。
- 对话面板不再播放 height 动画；小面积按钮保留低半径玻璃，大面板使用透光底色，去掉叠加的全屏模糊。
- 对话未首次展开时不挂载历史正文；首次显示最近 24 条，继续加载每次增加 24 条，历史定位仍可展开到目标消息。
- Markdown 按组件实例设置有界缓存，保持既有清理逻辑，不把不同连接的正文放进全局缓存。
- 输入高度与自动滚动合并到显示帧；消息增量替换不再全量排序历史。Pio 在展开对话时暂停绘制，衣柜预览保留。
- 队列 Gateway Surface 帧保持顺序，按显示帧批量合成；积压超限时改读权威快照，避免无限增长。

未运行原生界面与手机帧率采样，因此不宣称已达到某个 FPS 或完全消除卡顿。

## infOS 兼容

本轮对照当前 infOS 的 `agent.router.ts`、`chat.router.ts`、`conversationQueueService.ts`、`conversationQueue.repo.ts`、`useClientAccess.ts` 与 `useConversationQueue.ts`，未修改 infOS 仓库。

角色状态请求和缓存均带 `threadId`，无会话才读取默认状态；Rust 拒绝与目标角色或会话不符的响应。

聊天改为提交 `/api/chat/queue/:threadId`，不再调用旧的 `/api/chat/stream`。支持队列查看、优先立即提交、编辑、取消、暂停／继续、恢复中断项与重排。修改使用服务端 revision，旧版本冲突不能覆盖新内容。实时内容通过 Gateway Surface，定时读取队列及 `/live` 补偿断线。缺少队列接口的旧 infOS 不会静默降级到不可排队的旧发送方式。

客户端生成的请求 UUID 作为 `clientMessageId`。入队回执表示接收成功，不冒充模型执行完成。响应丢失时只读取队列中的同一 UUID，或读取服务端持久消息的 `queue_<UUID>` 对话对身份来确认；最近一页也找不到时继续保留待核对状态，不按正文匹配、不自动重发。队列编辑等其他写操作结果不明确时，也保留原有待核对记录。

## 原生扫码与 iOS 凭据

Android／iOS 使用 Tauri 原生 barcode-scanner，Rust 与前端绑定锁定为同一 2.4.6 版本；相机由用户主动请求，扫描期间提供取消入口。不能用相机的桌面平台仍可粘贴二维码链接或手动填写。

Rust 仅接受 HTTP(S) 的 `/#/app?pair=<10 位十六进制>` 配对链接，拒绝附带用户信息、重复参数和回环地址。二维码仅填入地址与配对码，用户核对后才兑换；明文 HTTP 的同意不会从二维码继承。相机取消、组件销毁与模式切换会停止扫描，短期代码不进入 Web 存储。

iOS 使用 `security-framework` 调用系统 Keychain，服务名固定、账户使用不透明凭据 UUID，禁用 iCloud 同步。保存、读取和删除均保留原有系统存储失败兜底，不落明文。补充 iOS 相机及局域网用途描述。本机是 Windows，未验证 iOS 目标编译、设备扫码或 Keychain 真机行为。

参考：[Tauri 扫码插件](https://v2.tauri.app/plugin/barcode-scanner/)、[Keychain Rust 接口](https://docs.rs/security-framework/3.7.0/security_framework/passwords/index.html)。

## 通知边界

在线 Gateway 通知、服务端持久通知补发与应用进程退出后的系统推送分开显示。用户决定本轮暂缓系统推送，未来通道必须覆盖无 Google 服务的 Android。当前没有接入 FCM、厂商推送、UnifiedPush 或 APNs，也不通过后台常驻来冒充系统推送。

已有 infOS 可以作为后续推送发送端，不要求必然另建独立服务器；但需要选择接收通道并配置应用身份与发送凭据。纯应用长连接随进程退出而消失，Android 对后台进程还存在系统／厂商限制。参见 [Android 后台限制](https://developer.android.com/develop/background-work/background-tasks/bg-work-restrictions) 与 [华为 Push Kit](https://developer.huawei.com/consumer/en/hms/huawei-pushkit/)。这些资料说明机制，不代表已经确认 QQ 在各设备上的内部实现。

## 验证

本轮沿用前后端测试授权，ESLint 通过，前端 **55 项**、`cargo test --lib` 的 Rust **72 项**全部通过。覆盖了渲染缓存预算及实例隔离、Nana 旧素材迁移、严格配对链接、会话状态参数与队列 revision／重排边界；前端包含全部 Vue 脚本和模板的内存校验。移除了已停用的远程聊天 SSE 通道及其专用解析用例，独立模式模型流处理不变。

未运行 Vite／Tauri 应用构建、Android／iOS 本地构建、应用启动或真实 infOS 联调。发布打包仅由推送版本 tag 后的 GitHub Actions 执行。Android 原生扫码、iOS Keychain 和真实手机性能仍待平台验收。
