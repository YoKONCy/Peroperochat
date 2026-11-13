# 独立模式技术文档

独立模式在 Tauri / Rust 进程内运行一套有界 ReAct 对话与记忆系统，管理本地角色、提示词工作区、TriviumDB 日记及系统提醒，数据与 infOS 远程客户端完全隔离。

## 1. 运行链路与模块职责

```text
当前输入 → Rust 校验与上下文编译 → 模型调用 → 工具结果 → 模型继续 → 最终回复与工具草稿同事务提交 → TDB 同步 → 前端投影
```

Vue 前端仅负责表单编辑、输入展示、流式预览与动效表现；模型请求、上下文裁剪、工具执行与数据持久化全部由 Rust 后端承担。

| 模块 | 职责 |
| :--- | :--- |
| `src-tauri/src/local/conversation.rs`、`runtime.rs` | 会话租约、取消、整轮超时、流式预览与终态控制 |
| `src-tauri/src/local/context.rs`、`prompts.rs` | 提示词框架、变量替换、上下文预算、工作区草稿/发布/回滚 |
| `src-tauri/src/local/react.rs`、`chat_tools.rs` | 有界 ReAct 循环、参数校验、角色作用域及工具草稿提交 |
| `src-tauri/src/local/provider.rs`、`wire.rs`、`provider_stream.rs` | 四类 API 协议、鉴权、请求构造、SSE、思考内容与工具调用归一化 |
| `src-tauri/src/local/model_config.rs`、`settings.rs` | 模型参数校验、思考方言、扩展字段与原子保存 |
| `src-tauri/src/local/diary.rs` | TriviumDB 日记存取、日期/关键词检索、来源追踪与补偿同步 |
| `src-tauri/src/local/reminders.rs` | 提醒时间校验、角色隔离、修订控制、事务提交与到期调度 |
| `src-tauri/src/local/repository.rs`、`backup.rs` | 消息与修订管理、请求幂等、工具审计、删除、迁移与导入导出 |
| `src-tauri/src/local/identity.rs`、`management.rs` | 角色投影与本地管理命令 |
| `src/services/standalone.js` | 前端管理 IPC 封装 |

家庭群聊、21 点小游戏与房间环境状态均复用同一 Rust 会话核心。

### 有界 ReAct 约束

- 每轮模型调用上限为 **1–12 次可配置（默认 8 次）**，单轮工具调用上限 **24 次**。
- 相同 `callId` 且参数一致时复用工具结果；同 ID 但参数变化则直接报错；同一条日记在单轮内仅接受一次待提交修订。
- 回复正文不承担隐式状态控制协议；历史遗留的 `MEMORY`、`PEROCUE` 等内联标签仅作展示层剥离兼容，不再触发副作用。

## 2. 内置工具集

| 工具名 | 参数 | 行为 |
| :--- | :--- | :--- |
| `write_diary` | `content`，可选 `date`、`tags`、`id`、`expectedRevision` | 为当前角色新建或按修订更新日记 |
| `query_diary` | 可选 `keyword`、`from`、`to`、`limit` | 查询当前角色已提交日记（最多 20 条，输出总长度受限） |
| `update_character` | 可选 `state`、`interactions`（至少一项非空变更） | 更新角色状态与互动台词 |
| `query_reminders` | 可选 `keyword`、`includeDelivered`、`limit` | 查询当前角色已提交提醒（最多 20 条），附带本地时间与 UTC 偏移 |
| `manage_reminder` | `action` (`create`/`update`/`delete`)、`task`、`time`、`recur`、`recurUntil`、`id`、`expectedRevision` | 创建、修改或删除当前角色的提醒 |

其中 `update_character.state` 支持 `mood`、`vibe`、`mind`；`interactions` 支持 `welcome`、`idle`、`back`、`click`（点击部位：`head`、`chest`、`body`、`default`）。写入时映射到对应角色/Live2D 台词槽，仅替换本次更新的类别。

### 事务提交语义

- 工具调用在回合执行期间仅返回草稿确认（`commit: after_assistant_persisted`），不立即写入最终业务表。
- 当整轮助手回复完整且通过校验后，助手消息、日记恢复日志、角色状态、台词变更、提醒调度变更与工具审计在**同一个 SQLite 事务**内提交。
- 用户取消、超时、断流、参数非法或修订冲突时，所有草稿全部丢弃，不会残留半截状态。

### 提醒业务规则

- 新建提醒要求未来的本地时间（`YYYY-MM-DD HH:mm:ss`），拒绝无效或处于夏令时歧义窗口的时间点。
- 支持一次性、`daily`、`weekly` 重复与截止时间；变更与删除均需携带最新 `revision`，同轮对同一提醒仅允许一次待提交修改，重复创建同任务与同规则提醒将拒绝。
- 提醒作为独立调度记录持久化，已提交提醒不会因后续删除聊天记录或重新生成回复而自动撤销。
- 错过的周期提醒仅补偿触发一次并推进至下一未来周期。

## 3. 日记与 TriviumDB 存储

- 基于固定版本 `triviumdb = 0.8.8`，每个角色数据独立存储于应用数据目录 `data/diary/{agentId 的 SHA-256}/memory.tdb`。
- **无向量检索设计**：不请求 Embedding 模型，写入时填充全零占位槽位以满足固定接口格式；检索采用按日期范围、标签与大小写不敏感关键词的子串匹配，按时间倒序返回。
- **跨库恢复保证**：SQLite `memories` 表保存写入镜像，`memory_tdb_pending` 跟踪尚未刷入 TDB 的变更。消息事务提交后再同步刷新 TDB 并清理待同步记录；若 TDB 临时不可用，后续查询或恢复任务会自动重试同步，确保主库与日记库最终一致。
- 每条日记保留稳定 ID、角色 ID、日期、正文、标签、版本号与来源消息 ID；删除相关输入/回复或重新生成回复时，对应衍生日记一并清理或回滚。
- `local_diary_day` 直接汇总当天日记条目输出为 Markdown，不消耗模型调用。

## 4. 提示词工作区

提示词框架由有序区块构成，每块包含 `id`、`source`、`role`、`template`、`enabled`。内置区块类型：`system`、`persona`、`user`、`environment`、`state`、`post`、`diary`、`history`、`current`，以及自定义文本区块 `custom`。

- 模板空白时回退使用内置默认模板；变量渲染仅支持白名单 `{{变量名}}` 单次替换（含角色、用户、环境、状态、`diary.recent`），不支持表达式求值或递归展开。
- `history` 与 `current` 为结构定位区块，不可自定义模板，二者必须均启用且历史位于当前输入之前；当前输入始终以 `user` 角色插入。
- 分配优先级支持「角色 + 通道（`desktop` / `group`） > 角色 > 通道 > 全局」。
- **版本与运行隔离**：聊天固定读取已发布版本，保存草稿不影响当前对话。管理更新需校验 `expectedRevision`（冲突返回 `STALE_PROMPT`），保留最近 20 个发布版本可供回滚。
- **上下文预算**：依据 `contextWindow` 预留输出与工具定义空间后，按完整对话轮次截取历史与近期日记；若 ReAct 循环累计超出预算，则显式结束本轮。（本地采用保守字符估算与图片固定额度计费，当前尚未在前端提供提示词工作区的独立可视化编辑器，相关 IPC 接口保留完整能力。）
- 工具约束与作用域边界由 Rust 执行器强校验，不依赖提示词文本约束。

## 5. 模型协议与参数适配

通过 `local_save_settings` 的 `api` 与 `model` 节分别更新连接与推理参数：

```json
{
  "base": "https://api.example.com/v1",
  "model": "model-id",
  "key": "用户密钥",
  "apiFormat": "chat_completions",
  "reasoningDialect": "auto",
  "headers": {},
  "extraBody": {}
}
```

- `apiFormat` 支持 `chat_completions`、`responses`、`anthropic`、`gemini`，按协议自动适配 Bearer / Claude / Gemini 鉴权方式及模型列表拉取分页归一化（`data[].id`）。

| 参数字段 | 取值与说明 |
| :--- | :--- |
| `reasoningEffort` | `null` 沿用默认；`off` / `none` / `minimal` / `low` / `medium` / `high` / `xhigh` / `max` 按协议映射 |
| `temperature`、`topP` | 可空，分别限定 `0–2` 与 `0–1` |
| `frequencyPenalty`、`presencePenalty` | 可空，限定 `-2–2`（仅兼容协议发送） |
| `maxTokens` | 输出上限 `128–65536`（Anthropic 思考预算包含在总额度内） |
| `contextWindow` | 本地上下文预算，至少 `maxTokens + 1024`，上限 `2000000` |
| `memoryRounds` | 历史保留轮次 `1–200` |
| `stream` / `toolsEnabled` | 流式输出开关 / 工具调用开关（关闭后支持无工具的纯对话模型） |
| `maxSteps` / `turnTimeout` | 单轮 ReAct 步数上限 `1–12` / 整轮超时 `30–900` 秒 |
| `disabledParams` | 按需剔除特定服务商不支持的可选协议字段 |
| `extraBody` | 连接层与模型层浅合并（模型层优先），禁止覆盖受管理消息/角色/工具/流式/输出上限等权威字段 |

- `reasoningDialect` 支持 `auto`、`openai`、`deepseek`、`openrouter`、`generic`，启用思考时自动省去不兼容采样参数。
- HTTP 客户端禁用自动重定向，自定义请求头禁止覆盖 `Host`、`Content-Length` 等底层传输头；错误响应自动脱敏隐藏 API Key 与请求头密钥。
- 四类协议均保留思考内容、逐次用量、原生 `callId`、Claude/Gemini 签名与 Responses 加密推理项；流式工具参数须累积为完整 JSON 后方可执行。

协议参考：[OpenAI function calling](https://developers.openai.com/api/docs/guides/function-calling)、[DeepSeek thinking mode](https://api-docs.deepseek.com/guides/thinking_mode/)、[Gemini function calling](https://ai.google.dev/gemini-api/docs/function-calling)。

## 6. 会话生命周期、幂等与备份

- 每次生成绑定基于输入、目标和角色的请求指纹，同一请求 ID 禁止变更内容重放；失败或已删除消息无法通过原 ID 恢复。
- 中断、超时、截断或工具超限时保留已接收的失败片段（标记失败态，不进入后续有效对话上下文），同时全部回滚待提交草稿。
- 重新生成保留原消息 ID 与时间戳，仅在新回复成功后归档旧版本、替换正文并撤销旧版衍生日记。
- 删除单轮对话同步清理正文、思考记录、用量、审计日志与衍生日记；角色当前投影状态不随历史删除而倒退。
- 备份规范 **v2.0**（向下兼容导入 v1.0）包含提示词工作区、思考内容、用量、日记修订与来源追踪；导入和整库重置前严格校验结构关系，并在全局租约保护下同步清理 TDB 缓存防止陈旧数据回流。

## 7. 本地管理 IPC

| 命令 | 用途 |
| :--- | :--- |
| `local_model_configuration` | 读取模型与接口配置（密钥仅返回脱敏存在标记） |
| `local_save_settings` | 按 `api` / `model` / `user` 分段原子更新设置 |
| `local_prompt_workspace` / `local_manage_prompts` / `local_prompt_preview` | 读取、草稿保存、发布、回滚与上下文编译预览 |
| `local_diary_query` / `local_manage_diary` / `local_diary_day` | 检索、手写/删改日记及按日导出 Markdown |
| `local_reminder_query` / `local_manage_reminder` | 校验修订并读写当前角色提醒（管理列表上限 50 条） |
| `local_reminder_runtime` / `local_request_reminder_precision` / `local_open_reminder_notifications` | 查询调度状态并打开系统提醒/通知授权设置页 |

前端调用统一经 `src/services/standalone.js` 与 `src/services/reminders.js` 转发；配置表单保存时需提交实际修改值，不回传占位脱敏文本。

## 8. 房间界面与交互设计

- **时段自适应房间**：内置背景置于 `public/companion/`，进房、切回房间或切回前台时依据设备本地时区切换日间（08:00–17:00）、清晨/黄昏（05:00–08:00、17:00–20:00）与夜晚（20:00–05:00）三套插画；自定义上传或外链背景保持固定（编辑器限制单图 ≤ 5 MiB，支持应用内路径、HTTP(S) 与 PNG/JPEG/WebP Base64，禁用本地任意文件路径）。
- **双显示与外观持久化**：通过 `local_appearance` / `local_save_appearance` 独立存储于数据库键 `ppc.ui.appearance`，包含：

| 字段 | 取值说明 |
| :--- | :--- |
| `displayMode` | `illustration`（默认静态立绘）或 `live2d`（Pio 模型，支持衣柜换装） |
| `palette` | `auto`（跟随角色）、`sky`、`sakura`、`mint`、`violet` |
| `glass` | 磨砂玻璃透明度 `60–94`（整数百分比） |
| `figureSize` | 静态立绘缩放比例 `44–64`（整数百分比） |
| `motion` | 界面过渡动效开关（优先尊重系统「减少动态效果」设置） |

- **Live2D 性能控制**：静态模式不初始化模型；处于 Live2D 模式时，一旦页面隐藏、对话全屏展开、弹出键盘或打开遮罩抽屉，即暂停渲染循环并带版本号防竞态。
- **三态对话面板**：`rest`（底部悬浮输入 + 最近一句气泡）、`peek`（半屏对话流）、`full`（沉浸阅读），草稿与状态按角色隔离；消息首次加载 80 条并支持向上分页，思考过程默认折叠，工具执行以自然语言时间线呈现。
- **局部时间线与家庭群聊**：单聊与群聊角色消息均带默认折叠的「工具 · N 项」时间线（单条上限 24 项），写入工具在事务提交前标为「待保存」，未提交结束则标为「未保存」。家庭群聊「大家的小客厅」通过 `local_group_settings` 管理成员出场开关与顺序。
- **样式与图标体系**：全局设计变量与控件复写位于 `src/styles/companion.css` 与 `src/styles/controls.css`；服务商域名与模型家族图标由 `src/utils/modelBrands.js` 和 `src/components/companion/BrandIcon.vue` 匹配，未知品牌统一使用通用内置 SVG 图标。

## 9. Android 系统提醒调度

Android 端结合原生 `AlarmManager` 与 JNI 直连 Rust 核心实现离线准点推送；桌面端由 Rust 每 15 秒轮询触发（基于 `notify-rust 4.12.0`），Android 进程内 15 秒任务仅用于状态对账。

```text
提醒事务 → SQLite 调度待办 → Rust 平台适配 → 系统闹铃登记 → AlarmReceiver → JNI / Rust 核心认领并核对版本 → 原生通知 → 回执持久化 → 周期续排
```

- **调度表设计**：`reminder_dispatch_clock` 维护设备单调版本，`reminder_dispatch` 通过 SQLite 触发器与业务表同步增删改版本，`reminder_delivery` 通过 `BEGIN IMMEDIATE` 事务认领执行租约（失败或超时 60 秒后允许重试），过滤旧闹铃与重复投递。
- **核心文件**：`src-tauri/src/local/reminder_dispatch.rs`、`reminder_jni.rs`、`reminder_platform.rs` 及 Kotlin 侧 `ReminderBridge.kt`、`ReminderScheduler.kt`、`ReminderPlugin.kt`、`AlarmReceiver.kt`、`BootReceiver.kt`（JNI 相关类已保留在 `src-tauri/gen/android/app/proguard-rules.pro` 规则内）。
- **权限与生命周期边界**：声明 `SCHEDULE_EXACT_ALARM`（授权时调用 `setExactAndAllowWhileIdle`，未授权时降级至 `setAndAllowWhileIdle`）；冷启动广播直接经 JNI 打开凭据保护目录中的同一 SQLite，不启动 WebView。开机解锁、更新、校准时区或授权变更后由 `BootReceiver` 与回前台逻辑自动对账重排。

参考资料：[Android 系统闹铃](https://developer.android.com/develop/background-work/services/alarms)、[精确提醒权限](https://developer.android.com/about/versions/14/changes/schedule-exact-alarms)、[系统强制停止行为](https://developer.android.com/about/versions/15/behavior-changes-all?hl=en)。
