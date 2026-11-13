# 构建与发布技术文档

本文说明 PeroperoChat 的版本元数据维护、CI/CD 发布流水线、Android 签名策略、测试校验口径与版本演进。

## 1. 版本维护

版本号同步维护在 **4 处**：

- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/tauri.conf.json`

Android 构建时的版本名与版本代码由 Tauri 在 CI 阶段生成至 `.gitignore` 忽略的 `tauri.properties`，无需手动维护。

## 2. 自动构建与发布流程

推送 `v<版本号>` 标签或手动触发 `.github/workflows/tauri-build.yml`，将执行 Android ARM64 Release APK 构建并创建 GitHub Release：

- 应用标识固定为 `com.peroperochat.mobile`，构建前校验标签版本号与项目元数据一致。
- 只打包单架构 ARM64 Release APK，使用 Android SDK Build Tools `36.0.0` 按顺序执行 `zipalign -P 16` 与 `apksigner sign`，并在签名后复核对齐与签名完整性。
- 从 keystore 导出的 DER 证书计算 SHA-256，与最终 APK 的签名证书指纹比对，存在差异或多余签名者时中止发布。
- 发布产物同步附带 `PeroperoChat-signing-certificate.sha256`；后续发布在构建前与前一版本指纹进行基线校验。可通过仓库变量 `ANDROID_SIGNING_CERT_SHA256`（纯十六进制或冒号分隔）显式锁定预期指纹。
- 工作流配置为串行执行，避免并发版本读写同一签名基线；临时解码的 keystore 文件在流程结束或失败时自动清理。

## 3. Android 发布签名与旧版迁移

1.4.0 之前的 CI 每次均使用临时生成的 `debug.keystore`，导致每次证书指纹不同而无法覆盖安装。自 **1.4.0** 起改用仓库 Secrets 注入固定发布密钥：

| Secret | 用途 |
| :--- | :--- |
| `SIGNING_KEY` | 发布 keystore 文件的 Base64 编码 |
| `ALIAS` | 私钥别名 |
| `KEY_STORE_PASSWORD` | keystore 访问口令 |
| `KEY_PASSWORD` | 私钥口令 |

私钥仅在 runner 临时目录解码，口令通过环境变量传递；密钥缺失或口令错误时直接终止发布，不降级至调试签名。所有 `.jks` / `.keystore` / `.p12` 文件均已列入 `.gitignore`。

### 1.4.0 以前版本迁移

1. 在旧版应用内导出 JSON 数据备份，并保存至外部目录。
2. 卸载旧版安装包，安装 `1.4.0` 或更高版本 APK。
3. 导入备份恢复独立模式数据；远程连接凭据不包含在业务备份内，需重新与 infOS 配对。
4. 后续同签名版本可直接覆盖升级。

参考资料：[Android 应用签名](https://developer.android.com/studio/publish/app-signing)、[apksigner](https://developer.android.com/tools/apksigner)、[zipalign](https://developer.android.com/tools/zipalign)。

## 4. 质量校验命令

```bash
pnpm test        # 前端单元测试（传输、渲染、品牌识别及 Vue 模板内存编译校验）
pnpm test:rust   # Rust 后端单元测试（本地模拟服务隔离外部网络）
pnpm lint        # ESLint 代码规范检查
pnpm typecheck   # vue-tsc 模板与 JS/TS 类型检查（不产出构建包）
```

> 以上校验运行在本地模拟与静态分析环境，用于回归逻辑与类型检查；本地打包、真机手势及真实服务联调需在对应环境中单独完成。

## 5. 版本演进与待联调范围

| 版本 | 核心变更 |
| :--- | :--- |
| **1.3.4** | 独立陪伴模式、角色房间视觉与 Android 本地 `AlarmManager` 提醒 |
| **1.4.0** | 开放 infOS 远程客户端模式、双模式运行控制与固定 Android 发布签名 |
| **1.4.1** | 立绘/Live2D 切换与衣柜、infOS 会话队列接入、原生扫码与 iOS Keychain 分支 |
| **1.4.2** | Android 远程后台独立前台通知进程、房间三时段自动背景、气泡与表单统一样式 |
| **1.4.3** | 独立单聊与家庭群聊的工具调用时间线持久化与历史回填 |
| **1.4.4** | 远程工作台与管理能力接入、原件流式下载与多格式本地文件预览 |
| **1.4.5** | 远程消息编辑/回滚/重新生成、流式富文本分段渲染、媒体与文档小窗阅读 |
| **1.4.6** | 长对话加载与状态竞态修复、队列消息重排、富文本层级与轻气泡反馈优化 |
| **1.5.0** | 导航与背景视觉改版、紧凑思考与工具轨迹、Docker 实例升级后设备重连恢复、接入 `vue-tsc` 类型检查 |
| **1.6.0** | 主／侧会话、会话 Skills 与插件管理接入，移动端消息与头像加载优化，任务及资料列表改版，远程样式清理 |

### 当前待真机与联调事项

- **移动端原生能力**：Android `AlarmManager` 调度与唤醒、`:remote_notifications` 跨进程 Keystore 读取、厂商省电策略适配及远程触控手势需继续真机验收。
- **真实服务端闭环**：infOS 多端输入席位、审批流、工作台文件与新管理接口持续对接联调中。
- **iOS 平台**：`src-tauri/gen/ios` 工程目录尚未初始化，Keychain 凭据分支与扫码链路待 macOS / 真机环境验证。
