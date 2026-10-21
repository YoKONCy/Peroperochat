# 构建与发布技术文档

本文合并自 Android 签名说明、版本与验证记录及各版本实装记录（原始文件归档于 `docs/archived/`），覆盖版本维护、发布流程、签名校验与各版本验证边界。

## 1. 版本维护

版本同时维护在 **4 处**：`package.json`、`src-tauri/Cargo.toml`、`src-tauri/Cargo.lock` 与 `src-tauri/tauri.conf.json`。Android 的版本名称与版本代码由 Tauri 在 CI 中生成到忽略跟踪的 `tauri.properties`，不手动修改生成文件。

## 2. 发布流程

推送 `v<版本号>` tag 会触发 GitHub Actions（`.github/workflows/tauri-build.yml`）执行 Android ARM64 APK 打包及 GitHub Release：

- 固定使用 `com.peroperochat.mobile` 应用身份；版本元数据和发布 tag 必须一致。
- 只允许选中一个 ARM64 Release APK；使用 SDK Build Tools 36.0.0，先 `zipalign -P 16`，再 `apksigner sign`，之后检查对齐与签名。
- 用 keystore 导出的 DER 证书计算 SHA-256，并与最终 APK 的签名证书比较；不匹配或出现多余签名者时拒绝上传。
- 每次 Release 附带 `PeroperoChat-signing-certificate.sha256`；后续运行在构建前与上一版指纹比较。
- 仓库变量 `ANDROID_SIGNING_CERT_SHA256` 可显式固定期望指纹，支持纯十六进制或冒号分隔形式。不要无意更换该变量或签名 Secrets。
- 发布流程串行执行，避免两个版本同时读取、更新签名基线；临时 keystore 在失败时也会清理。

## 3. Android 签名

1.4.0 之前的 CI 在全新 runner 的 `~/.android/debug.keystore` 中随机生成密钥——即便别名和口令相同，每次生成的证书仍不同，Android 因而拒绝将新 APK 覆盖到上一版。从 1.4.0 起使用仓库 Secrets 中的固定签名：

| Secret | 内容 |
| --- | --- |
| `SIGNING_KEY` | 发布 keystore 文件的 Base64 |
| `ALIAS` | 私钥别名 |
| `KEY_STORE_PASSWORD` | keystore 口令 |
| `KEY_PASSWORD` | 私钥口令 |

私钥只在 runner 临时目录中解码，口令经环境变量交给工具，不放进命令行或日志。签名配置缺失、密码不正确或别名错误时停止发布，不降级到调试签名。keystore、JKS 和 P12 文件已加入 Git 忽略规则。

### 旧版本迁移

旧随机证书的私钥不能从 APK 或公开证书中还原。1.4.0 之前的旧包无法直接覆盖安装：

1. 在旧应用中导出数据备份，并确认备份文件已保存到应用之外。
2. 卸载旧包，安装 1.4.0 或更高版本的固定签名包。
3. 恢复独立数据；远程凭据不在业务备份中，需要重新配对 infOS。
4. 后续使用同一固定发布密钥的版本直接覆盖更新。

参考：[Android 应用签名](https://developer.android.com/studio/publish/app-signing)、[apksigner](https://developer.android.com/tools/apksigner)、[zipalign](https://developer.android.com/tools/zipalign)。

## 4. 测试命令与口径

```bash
pnpm test        # 前端：传输、消息展示、品牌识别及全部 Vue 组件/模板内存校验
pnpm test:rust   # Rust 后端（cargo test --manifest-path src-tauri/Cargo.toml --lib）
pnpm lint        # ESLint
```

- Vue 校验在内存中解析组件脚本与模板，不生成应用构建产物；Rust 网络测试使用本地模拟服务，未调用真实模型。
- 测试通过不代替本地生产构建、安卓/iOS 打包、真实模型与 infOS 联调或安卓真机验收，也不代替视觉和端到端验收。

## 5. 版本与验证记录

| 版本 | 内容摘要 | 验证 |
| --- | --- | --- |
| 1.3.4 | 独立陪伴模式、角色房间与安卓提醒 | 前端 35 项、Rust 52 项通过；未构建/打包/真机 |
| 1.4.0 | 开放 infOS 客户端并固定 Android 发布签名 | 前端 50 项、Rust 69 项通过；发布工作流 YAML 与 Bash 语法检查通过 |
| 1.4.1 | 角色显示优化、房间切换卡、infOS 队列接入、原生扫码与 iOS Keychain | 前端 55 项、Rust 72 项及 ESLint 通过 |
| 1.4.2 | Android 远程后台接收、房间时段背景、本地气泡与表单控件 | 仅核对源码差异与版本元数据，未重新运行测试 |
| 1.4.3 | 本地单聊和家庭群聊的工具调用时间线（随消息保存、旧记录回填） | 补充 Rust 回归用例；未运行测试/构建 |
| 1.4.4 | 远程管理、会话操作、文件原件流式保存与统一文件浏览 | 前端 73 项通过；此前文件接入阶段 Rust 99 项通过 |
| 1.4.5 | 远程消息编辑/回滚/重新生成、对话一期视觉、流式富文本、媒体与文档小窗 | 前端 86 项、Rust 103 项通过 |
| 1.4.6 | 长对话读取与状态冲突修复、会话重排、富文本层级、轻气泡反馈 | 前端 101 项、Rust 108 项通过 |

### 各版本未验证边界（累积）

以下事项在各版本发布时均未完成验收，状态以此清单为准：

- **本地构建与启动**：未执行本地生产构建、Tauri 启动验收（部分阶段曾按用户明确要求启动桌面查看，见归档文档）。
- **移动端**：未执行 Android/iOS 打包或真机验收。安卓提醒的原生桥接、系统唤醒，远程通知的 Kotlin 桥接、Keystore 跨进程、功耗与厂商策略，远程 UI 的视觉与手势均待设备确认。
- **真实服务**：未连接真实 infOS 联调（协议对照基于本地 infOS 工作区源码）；未进行真实模型联调。
- **iOS**：工程未初始化；iOS Keychain 分支与扫码链路未经真机验证。
- **性能**：1.4.1 的渲染优化未在用户手机上采样帧率，不将源码优化等同于真机性能验收。
- **新增能力**：1.4.5 的富文本渲染、DOC 提取、HTML 票据、媒体小窗缺少真实服务端与设备端到端验收；二期（据点/日常）页面未运行前后端测试。

历史阶段的详细实装与验证记录见 `docs/archived/` 下的 V1_4_1_IMPLEMENTATION.md、V1_4_2_IMPLEMENTATION.md、V1_4_3_IMPLEMENTATION.md、CHAT_PHASE_ONE_IMPLEMENTATION.md、WORKSPACE_PHASE_TWO_IMPLEMENTATION.md 及 REMOTE_CLIENT_LOGIC.md。
