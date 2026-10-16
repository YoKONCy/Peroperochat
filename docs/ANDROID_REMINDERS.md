# 安卓系统提醒

安卓使用系统 `AlarmManager` 和应用内原生接收器；桌面继续使用 Rust 的到期轮询。前端只编辑任务、展示登记状态并在用户点击后打开授权入口。提醒时间、重复规则、修订、执行租约及成功确认都由 Rust 处理。

## 数据与调用链路

`提醒事务 → SQLite 调度待办 → Rust 平台适配 → 原生系统登记 → AlarmReceiver → JNI / Rust 核心核对并认领 → 原生通知 → 持久化回执 → Rust 续排`

- `reminder_dispatch_clock` 保存设备数据库的独立 epoch 和单调递增版本。
- `reminder_dispatch` 保存每个提醒当前的调度版本、登记版本、登记模式、错误和取消原因。SQLite 触发器使创建、修改、删除及备份恢复与调度变更共用事务；失败或取消的对话不留下系统任务。
- `reminder_delivery` 保存当前版本的执行租约、回执和重试时间。接收器认领和确认都在立即事务中检查调度版本；旧闹铃、旧租约和重复回执不能推进新提醒。
- 系统登记成功只更新登记状态，不将提醒标为送达。通知成功提交后才更新送达和重复时间；失败保留待处理状态，60 秒后允许重新认领。重复提醒直接推进到未来，避免补发大量错过的周期。
- `PendingIntent` 使用包含完整提醒 ID 的内部 URI，避免把 SQLite ID 截断为 32 位。通知使用稳定的 tag；重试替换同一通知，降低进程恰在通知提交与回执之间退出时的可见重复。
- 原生缓存仅保存 ID、版本、登记时间和模式，SQLite 是唯一任务来源。调度元数据与执行回执不导出到其他设备，导入原提醒时产生当前设备的新调度版本。

## 模块

| 文件 | 职责 |
| --- | --- |
| `src-tauri/src/local/reminder_dispatch.rs` | 调度待办、计划、登记回执、执行租约、通知确认与前端状态投影 |
| `src-tauri/src/local/reminder_jni.rs` | 同一 Rust 动态库中的 JNI 入口；直接打开现有数据库，无需创建 Tauri 或 WebView |
| `src-tauri/src/local/reminder_platform.rs` | Tauri 原生插件注册、同步与授权设置入口 |
| `ReminderBridge.kt` | 加载 `app_lib`，将原生事件交给 Rust 核心；路径与 Tauri 安卓 `app_data_dir` 一致 |
| `ReminderScheduler.kt` | 系统闹铃登记/取消、通知通道、通知提交和恢复任务；不实现重复规则 |
| `ReminderPlugin.kt` | 应用运行时的原生桥接与返回前台后的同步 |
| `AlarmReceiver.kt` | 到点后异步处理并持有限时唤醒锁 |
| `BootReceiver.kt` | 开机解锁、应用更新、校时、时区变化、精确权限授权和恢复任务的重新登记 |

JNI 与反射用类保留在 ProGuard 规则中。`ReminderBridge` 加载的库名来自当前 Cargo `[lib] name = "app_lib"`；以后修改库名时也要同步更新这里。

单聊、群聊、手动操作、角色变更、备份恢复及重置提交后都会同步系统登记。安卓端运行中的 15 秒任务只负责对账和刷新展示，不再同时发送到期通知，避免与原生接收器重复发送。同步失败保留调度待办，进程内对账、原生恢复任务或后续生命周期事件会重试。

## 权限与恢复边界

- 只声明 `SCHEDULE_EXACT_ALARM`，不再声明面向特定闹钟/日历用途的 `USE_EXACT_ALARM`。
- 已获精确权限时使用 `setExactAndAllowWhileIdle`；否则使用 `setAndAllowWhileIdle` 并在界面说明可能延迟。
- Android 13+ 的通知授权与精确提醒授权分别处理。通知通道被关闭时展示真实阻塞状态，提供系统通知设置入口。
- 不依赖常驻服务、前端事件或 LLM。原生 Receiver 冷启动时直接调用 Rust 核心读取同一 SQLite。
- 数据库保存在凭据保护目录，重启后首次解锁再恢复，不在锁屏前复制敏感提醒内容到设备保护目录。
- 用户系统强制停止、精确权限撤销、厂商后台限制和关机会影响调度；不能承诺绕过这些限制。重新打开应用后再次对账，精确权限授权后也会触发恢复。通知权限重新开启后的返回前台同步同样会恢复登记。
- SQLite 与 Android 系统调度没有跨系统原子事务。进程若在提交与登记之间突然退出，待办仍在数据库中，恢复事件负责补齐；界面区分“已保存”和“已登记”，不把前者冒充后者。
- 通知提交和数据库确认也存在极短的进程退出窗口。执行租约、稳定通知 tag 和版本检查用于恢复及去重，不声称已经证明系统通知严格只出现一次。

设计参考：[Android 系统闹铃](https://developer.android.com/develop/background-work/services/alarms)、[精确提醒授权](https://developer.android.com/about/versions/14/changes/schedule-exact-alarms)、[系统强制停止行为](https://developer.android.com/about/versions/15/behavior-changes-all?hl=en)。

## 验证

1.3.4 发布前按用户新授权运行前端测试与完整 Rust 测试：前端 35 项、Rust 后端 52 项通过。仍未进行安卓编译或真机验收；下面保留接入阶段的验证记录。

仅运行已获用户授权的 `cargo test --manifest-path src-tauri/Cargo.toml --lib`，52 项通过。新增覆盖调度事务回滚、登记失败与版本投影、取消/替换/角色删除后的旧回调拒绝、租约和回执去重、失败重试、崩溃租约恢复、循环续排，以及不使用 AppHandle 的数据库重开恢复。JNI 入口在桌面测试配置中参与 Rust 编译。

未运行前端测试、安卓编译、打包、模拟器或真机程序。Kotlin、Manifest、JNI 动态链接、ProGuard 和系统生命周期只完成源码审阅；桌面 Rust 测试不能证明安卓桥接或系统唤醒已实际验收。
