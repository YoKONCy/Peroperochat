# Android 签名与覆盖更新

## 原因与修复

1.4.0 之前的 CI 会在全新 runner 的 `~/.android/debug.keystore` 中随机生成密钥。即便别名和口令相同，每次生成的证书仍不同，Android 因而拒绝将新 APK 覆盖到上一版。

从 1.4.0 起，工作流恢复仓库已经配置的固定签名 Secrets，不生成或轮换密钥：

| 配置 | 内容 |
| --- | --- |
| `SIGNING_KEY` | 原有发布 keystore 文件的 Base64 |
| `ALIAS` | 该文件中私钥的别名 |
| `KEY_STORE_PASSWORD` | keystore 口令 |
| `KEY_PASSWORD` | 私钥口令 |

私钥只在 runner 临时目录中解码，口令经环境变量交给工具，不放进命令行或日志。签名配置缺失、密码不正确或别名错误时停止发布，不降级到调试签名。keystore、JKS 和 P12 文件已加入 Git 忽略规则。

## 发布校验

- 固定使用 `com.peroperochat.mobile` 应用身份；版本元数据和发布 tag 必须一致，Android versionCode 继续由 Tauri 根据版本生成。
- 只允许选中一个 ARM64 Release APK，拒绝通过任意 `head -1` 误取其他产物。
- 使用 SDK Build Tools 36.0.0，先 `zipalign -P 16`，再 `apksigner sign`，之后检查对齐与签名。
- 用 keystore 导出的 DER 证书计算 SHA-256，并与最终 APK 的签名证书比较；不匹配或出现多余签名者时拒绝上传。
- 每次 Release 附带 `PeroperoChat-signing-certificate.sha256`。后续运行在构建前与上一版指纹比较；首次迁入固定签名的旧版本没有该文件，因此本版建立基线。
- 仓库变量 `ANDROID_SIGNING_CERT_SHA256` 可显式固定期望指纹，支持纯十六进制或冒号分隔形式；配置后优先使用此值。不要无意更换该变量或四个签名 Secrets。
- 发布流程串行执行，避免两个版本同时读取、更新签名基线。临时 keystore 在失败时也会清理。

## 已安装旧版的迁移

旧随机证书的私钥不能从 APK 或公开证书中还原。没有原私钥就不能为这些旧包制作兼容的新签名，也不能通过更改应用版本绕过 Android 的签名要求。

1. 在旧应用中导出数据备份，并确认备份文件已经保存到应用之外。
2. 卸载旧包，再安装本仓库发布的固定签名版本。
3. 恢复独立数据；远程凭据不在业务备份中，需要重新配对 infOS。
4. 后续使用同一固定发布密钥的版本直接覆盖更新，无需再次重装。

项目仓库没有保存旧 runner 的随机 debug 私钥；本轮复用现有发布密钥，没有新建签名身份。真实手机覆盖升级仍须通过安装验证，本地测试不能代替该验证。

参考：[Android 应用签名](https://developer.android.com/studio/publish/app-signing)、[apksigner](https://developer.android.com/tools/apksigner)、[zipalign](https://developer.android.com/tools/zipalign)。
