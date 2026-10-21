# 独立模式技术文档

独立模式在 Tauri/Rust 内运行精简 ReAct，使用自己的角色、提示词和日记数据，不连接或共享 infOS 的实例与记忆。本文合并自原先的架构、UI 实装、陪伴房间与安卓提醒四篇文档（原始文件与设计稿归档于 `docs/archived/`）。

> 边界声明：提示词工作区的草稿、发布、回滚专用编辑界面尚未提供，管理接口保留。实际厂商模型联调、安卓真机验收范围见 `BUILD_AND_RELEASE.md` 的版本记录。

## 1. 运行链路

`当前输入 → Rust 校验与上下文编译 → 模型调用 → 工具结果 → 模型继续 → 最终回复与工具草稿一起提交 → TDB 同步 → 前端投影`

前端只负责表单草稿、输入展示、消息预览、动画和交互反馈，不负责模型请求、日记提炼或业务提交。

| 模块 | 职责 |
| --- | --- |
| `local/conversation.rs`、`runtime.rs` | 会话租约、取消、整轮超时、流式预览与终态 |
| `local/context.rs`、`prompts.rs` | 提示词框架、变量替换、历史预算、草稿、发布与回滚 |
| `local/react.rs`、`chat_tools.rs` | 有界工具循环、严格参数、作用域与草稿提交 |
| `local/provider.rs`、`wire.rs`、`provider_stream.rs` | 四类协议、鉴权、请求体、SSE、思考内容及工具调用归一化 |
| `local/model_config.rs`、`settings.rs` | 参数校验、协议方言、扩展字段与原子保存 |
| `local/diary.rs` | TDB 日记存取、日期和关键词查询、来源追踪与同步恢复 |
| `local/reminders.rs` | 提醒的时间校验、角色隔离、修订、事务提交、到期检查和系统通知 |
| `local/repository.rs`、`backup.rs` | 消息、修订、请求幂等、工具审计、删除、迁移与备份 |
| `local/identity.rs`、`management.rs` | 角色投影与管理命令 |
| `src/services/standalone.js` | 前端管理接口封装，仅传递意图，不编译提示词或执行工具 |

群聊、小游戏和环境信息模块复用同一 Rust 会话服务。

有界 ReAct：每轮默认最多 8 次模型调用、24 个工具调用，模型调用上限可配置为 1–12。重复工具调用 ID 和相同参数复用结果，身份相同而参数不同直接失败；同一日记本轮只接受一次待提交修订。正文不承担记忆、状态或台词控制协议，旧 `MEMORY`、`PEROCUE` 等标签仅保留显示兼容的剥离逻辑，不执行任何副作用。

## 2. 五个小工具

| 名称 | 参数 | 行为 |
| --- | --- | --- |
| `write_diary` | `content`，可选 `date`、`tags`、`id` 与 `expectedRevision` | 新建日记，或修订自己的日记 |
| `query_diary` | 可选 `keyword`、`from`、`to`、`limit` | 查询当前角色已提交的日记，最多 20 条，结果总文本有界 |
| `update_character` | 可选 `state`、`interactions`，至少包含实际更新 | 同一个工具更新角色状态与互动台词 |
| `query_reminders` | 可选 `keyword`、`includeDelivered`、`limit` | 查询当前角色已提交的提醒，最多 20 条，同时返回设备本地时间与 UTC 偏移 |
| `manage_reminder` | `action: create/update/delete`，以及 `task`、`time`、`recur`、`recurUntil`、`id`、`expectedRevision` | 创建、修改或取消提醒；只提交当前角色的变更 |

`update_character.state` 支持 `mood`、`vibe`、`mind`；`interactions` 支持 `welcome`、`idle`、`back`、`click`，点击部位支持 `head`、`chest`、`body`、`default`。Rust 将台词映射到现有 Live2D 字段，只替换指定类别，并清理多余旧台词。

### 提交语义

工具写入先返回 `commit: after_assistant_persisted`，最终回复完整且成功持久化之后才生效。回复、日记恢复日志、角色状态、台词和工具审计共用一个 SQLite 事务；参数无效、未知工具只返回结构化工具错误，不能执行生成代码或任意配置写入。提醒工具遵循同一事务边界，回复失败、停止、超时、修订冲突时不会遗留提醒草稿。

提醒规则要点：

- 创建要求未来的设备本地时间 `YYYY-MM-DD HH:mm:ss`，拒绝不存在或存在夏令时歧义的本地时间；相对时间先查询，含糊时间先向用户澄清。
- 支持一次、每天和每周提醒，以及重复截止时间。修改/删除必须携带最新修订号，同一提醒本轮只允许一次待提交变更；重复创建相同待办、时间和重复规则会报错。
- 提醒是独立任务，成功提交后不随聊天删除或重新生成而自动取消。
- 错过的周期提醒只补发一次后直接推进到未来，不逐次轰炸。

## 3. 日记与 TDB

- 使用固定版本 `triviumdb 0.8.8` 的 Rust 核心，每个角色独立存放于应用数据目录的 `data/diary/{角色ID的SHA256}/memory.tdb`。
- 不请求 Embedding，不构建或查询向量索引。TDB 当前写入接口要求固定维度，因此保存一个恒为零的占位槽位，不参与检索。
- 日期范围、关键词、标签过滤直接读取 TDB 当前条目；关键词按正文和标签进行不区分大小写的包含匹配，结果按日期和时间倒序。没有向量召回或后台记忆加工。
- SQLite `memories` 保留可恢复的写入镜像，`memory_tdb_pending` 记录未同步角色；先提交消息与镜像，再刷新 TDB，刷新成功后才清除恢复任务。这沿用工具草稿与成功回复后提交的原则，解决跨存储无法共享事务的问题。
- TDB 不可用时，普通聊天可以暂不加载日记；日记查询显式报告错误，待同步写入保留。后续读取或同步会重试，不返回已知过期条目。
- 日记包含稳定 ID、角色 ID、日期、正文、标签、修订与来源消息 ID。修订必须携带匹配的 `expectedRevision`；修订累积来源，删除相关输入或回复时可以清理衍生日记。
- `local_diary_day` 直接将当天条目按顺序输出为 Markdown，不额外调用模型生成摘要。

## 4. 提示词框架

框架由有序区块组成，每块含 `id`、`source`、`role`、`template`、`enabled`。内置来源为 `system`、`persona`、`user`、`environment`、`state`、`post`、`diary`、`history`、`current`，另外支持 `custom` 文本区块。

- `template` 为空时采用该来源的内置模板。变量只支持白名单 `{{变量名}}` 的单次替换，包括角色、用户、环境、状态和 `diary.recent`；没有脚本、表达式或递归模板求值。
- `history` 与 `current` 是位置区块，不能携带模板，必须各启用一个，且历史位于当前输入之前。当前输入始终按真实用户角色插入。
- 框架分配支持全局、通道、角色及角色加通道，通道为 `desktop` 或 `group`，优先采用更具体的分配。
- 运行时仅使用已发布版本，保存草稿不会立即影响聊天。管理写入使用 `expectedRevision`，过期编辑返回 `STALE_PROMPT`；保留最近 20 个版本，可回滚。
- 预览只编译，不请求模型，返回消息、区块清单、使用的历史/日记 ID 和预算估算。
- 上下文预算根据 `contextWindow` 预留输出和工具定义，按完整轮次选取近期已提交历史，并限制最近日记的占比；工具循环继续检查累计上下文，超额显式结束。Token 数是保守的字符估算，图片有固定预算，并非厂商精确 tokenizer。
- 后端固定工具协议独立于可编辑区块，约束由 Rust 校验和工具执行器落实，不能通过改提示词绕过作用域、参数限制或提交规则。

## 5. 模型 API 与参数

连接通过 `local_save_settings` 的 `api` 节保存：

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

`base`、`model`、`key` 为该节必填字段；未传的其他连接参数保留。`apiFormat` 支持 `chat_completions`、`responses`、`anthropic`、`gemini`。分别使用 Bearer、Claude API Key、Gemini API Key 鉴权，也支持自定义请求头。获取模型列表会按协议处理原生返回值和分页，并归一化为 `data[].id`。

模型参数通过同一命令的 `model` 节做部分更新：

| 字段 | 含义/范围 |
| --- | --- |
| `reasoningEffort` | `null` 跟随服务默认；`off`、`none`、`minimal`、`low`、`medium`、`high`、`xhigh`、`max` 按协议映射 |
| `temperature`、`topP` | 可为空；分别 0–2、0–1 |
| `frequencyPenalty`、`presencePenalty` | 可为空；-2–2，仅兼容协议适用 |
| `maxTokens` | 每次调用的输出额度，128–65536；Claude 预算思考包含在总额度内 |
| `contextWindow` | 本地预算，至少 `maxTokens + 1024`，最多 2000000 |
| `memoryRounds` | 历史轮次上限，1–200 |
| `stream` | 流式开关 |
| `toolsEnabled` | 关闭时只进行普通聊天，不要求模型支持工具 |
| `maxSteps`、`turnTimeout` | 每轮模型调用上限 1–12、整轮超时 30–900 秒 |
| `disabledParams` | 移除选定可选协议字段，适配不接受该字段的服务 |
| `extraBody` | 厂商扩展参数，与连接扩展合并，模型层优先；对象字段浅合并 |

`reasoningDialect` 支持 `auto`、`openai`、`deepseek`、`openrouter`、`generic`。自动识别 DeepSeek、OpenRouter 的常见地址或名称；特殊代理可以显式指定。思考启用时省略常见不兼容的采样字段。

安全约束：

- `extraBody` 不能覆盖权威消息、角色、工具定义、工具选择、流式开关、多候选数量或服务端会话身份等受管理字段；扩展输出额度不能超过本地额度。
- 自定义请求头禁止覆盖 Host、连接长度等传输字段，网络客户端禁用重定向，避免鉴权意外转发。错误输出隐藏 API Key 和自定义头值。

协议行为：

- 四类协议均分离正文和思考内容，持久化思考内容与逐次用量。
- ReAct 后续调用保留原生工具调用 ID、Claude 签名、Gemini thought signature，以及 Responses 的加密 reasoning 项。
- 流式工具参数必须累积为完整 JSON 后才执行，单纯 EOF 不视为成功。不存在静默改协议或自动重复计费重试。

协议实现参考：[OpenAI function calling](https://developers.openai.com/api/docs/guides/function-calling)、[DeepSeek thinking mode](https://api-docs.deepseek.com/guides/thinking_mode/)、[Gemini function calling](https://ai.google.dev/gemini-api/docs/function-calling)。

## 6. 生命周期与删除

- 请求 ID 绑定当前输入、目标和角色的指纹。同 ID 不能替换内容；成功重放还要求原回复修订未变化。失败请求需新 ID，已删除回复不能复活。
- 请求超时、用户取消、断流、截断、空回复或工具上限都会终止本轮并丢弃全部工具草稿。失败的已收正文保留为失败消息，不进入后续成功回复历史。
- 重新生成保留原消息 ID 和时间戳；成功才归档原修订、替换正文和衍生日记，失败或取消保留原回复。编译和日记查询排除正被替换回复的衍生日记。
- 删除用户输入会一起删除该轮回复；删除操作清空正文、思考内容和用量，清理修订、工具审计、上下文清单及有来源的日记。当前角色状态属于当前投影，不因历史删除自动倒退。
- 删除、导入、重置与进行中的请求通过运行租约协调。TDB 同步失败保留恢复任务，不重新引入已删除 SQL 记录。
- 提示词和模型设置提交先校验，再进行原子更新；通用 `save_config` 只保留角色选择和界面配置，业务配置需对应管理命令。

## 7. 管理命令

| IPC 命令 | 参数/用途 |
| --- | --- |
| `local_model_configuration` | 读取模型配置；密钥只返回存在标记，自定义头值隐藏 |
| `local_save_settings` | `section: api/model/user` 与 `values` |
| `local_prompt_workspace` | 读取草稿、当前发布版本与历史版本 |
| `local_manage_prompts` | `expectedRevision`、`action: draft/publish/rollback`，以及 `bundle` / `note` / `releaseId` |
| `local_prompt_preview` | `agentId`、`channel`、`content`、`draft` |
| `local_diary_query` | `agentId` 与 `query: {keyword, from, to, limit, offset}` |
| `local_manage_diary` | `agentId`、`action: write/delete`，以及 `draft` / `id` |
| `local_diary_day` | `agentId`、`date: YYYY-MM-DD` |
| `local_reminder_query` | `agentId`、`input: {keyword, includeDelivered, limit}`，管理界面最多返回 50 项 |
| `local_manage_reminder` | `agentId`、`change`，沿用提醒工具的操作字段，提交前重新校验修订 |
| `local_reminder_runtime` | 读取平台、后台调度支持、通知/精确授权和同步错误 |
| `local_request_reminder_precision`、`local_open_reminder_notifications` | 用户点击后打开安卓精确提醒或通知设置 |

前端封装见 `src/services/standalone.js` 和 `src/services/reminders.js`。配置读取中的隐藏头值仅用于展示，保存时应传实际编辑值，不能将隐藏占位文本当作密钥写回。原 `get_reminders` / `delete_reminder` 裸接口已移除，避免按 ID 无修订、无角色约束地修改任务。

备份 `2.0` 增加可选的思考、用量、日记修订与来源字段，兼容导入 `1.0`。提示词工作区随配置备份；导入前验证框架、模型参数与数据关系，事务失败整体回滚；重置和导入同时标记 TDB 清理，避免缓存恢复旧日记。

## 8. 界面：房间与对话

首页采用「轻透日常」角色房间视觉（设计参考 N.E.K.O.、Aiko、逗逗的公开界面布局与视觉层级，不复制其资产）。根据设备时区与时间在日间、晨昏和夜间切换 `public/companion/` 内的本地房间插画；仅进入或切回房间时刷新。自定义上传背景保持固定。旧 `public/rooms/` SVG 不再是默认房间背景。

- Pero：清亮蓝白房间，晴空、云朵、窗外樱花、蓝色窗帘、白色书桌与绿植。
- Nana：紫粉房间，暮色城市、月亮壁灯、电脑桌、耳机、唱片与手柄。
- 其他自定义角色暂用 Pero 的房间背景，角色身份和历史仍独立。

### 显示模式与外观持久化

首页右上角随时切换「静态立绘」和「Live2D」，菜单与设置的「显示与主题」提供同一选项。Rust `local::appearance` 提供 `local_appearance`、`local_save_appearance`，保存至独立数据库的 `ppc.ui.appearance`；通用 `save_config` 不写入该键。

| 字段 | 可用值 |
| --- | --- |
| `displayMode` | `illustration`、`live2d`，新配置默认静态立绘 |
| `palette` | `auto`、`sky`、`sakura`、`mint`、`violet` |
| `glass` | 60–94 的整数百分比 |
| `figureSize` | 44–64 的整数百分比，作用于静态立绘 |
| `motion` | 界面轻量过渡开关，优先遵循系统减少动态效果 |

静态模式不加载 Pio；若之前已打开 Pio，通过绘制循环的可见性入口停止续排动画帧，切回后恢复。页面隐藏、展开对话、软键盘展开或覆盖抽屉时也暂停绘制；衣柜保留可见预览。模型异步初始化带版本号，旧初始化结果不会重新启用模型。

角色记录可选字段 `standing_image`、`room_background`、`illustration_layout`（`standing` 透明立绘 / `card` 完整 CG 卡片）。地址支持应用内素材、HTTP(S) 或 PNG／JPEG／WebP 的 Base64；禁止任意本地文件路径和可执行协议。编辑器允许选取不超过 5MB 的图片，原生层再次验证类型、长度与排版枚举。

### 对话交互

对话面板三态：`rest` 显示悬浮输入栏和可展开的最近一句回复；`peek` 展开近期对话；`full` 提供完整阅读空间。输入、图片附件和展开状态按角色保留，切换角色不会混入另一人的草稿。

- 消息先展示最近 80 条，逐页加载更早记录；阅读旧消息时暂停自动跟随，提供回到最新按钮。思考内容折叠展示，工具进度转换成简短自然语言，不展示协议 JSON。
- 角色台词气泡：不透明白底、圆润边角、角色色双像素描边、轻阴影，朝向人物的上方尾巴；短台词按内容收缩宽度，长台词最多三行。
- Live2D 互动、欢迎和闲置台词采用无底框悬浮字：白色文字光晕、轻微错位、逐字冒出与结束淡去；展示节点有上限，支持减少动画偏好。
- 消息操作：长按或菜单复制、重新生成、删除。删除前明确提示整轮对话及衍生日记的清理范围；实际删除由 Rust 负责。
- 移动端按 Visual Viewport 的实际可见高度调整布局，配合安全区处理软键盘；桌面宽屏时对话面板靠右。

### 本地工具时间线

- 单聊和家庭群聊的角色消息增加默认折叠的「工具 · N 项」，按调用顺序显示工具名称、状态和一行结果。
- 写入操作在回复事务成功提交前显示「待保存」，取消、失败或中断时显示「未保存」，不冒充成功。
- 时间线随消息保存，重启、备份与恢复后仍可查看；旧数据库的成功轮次在升级时回填，没有记录的旧消息不虚构时间线。
- 成功重新生成时替换时间线；失败保留原回复及原时间线。每条消息最多 24 项。

### 家庭群聊

群聊以「大家的小客厅」为视觉线索，成员头像与姓名说明参与者和顺序，Pero、Nana 发言使用各自强调色。成员设置在统一抽屉中以上移、下移按钮及开关调整，保存后交给 `local_group_settings`；聊天调用 Rust 群聊整轮服务，提供整轮停止及消息复制、删除。

### 全局界面

- `src/styles/companion.css` 定义角色强调色及全局文字、底色、分隔线、圆角和阴影；`src/styles/controls.css` 统一表单、按钮、Element Plus 控件、确认框、提示与浮层。
- 设置中心分类布局：宽屏左侧导航，窄屏两行三列；服务商、协议和思考强度统一使用 `ChoicePicker` 底部选择弹层，较长列表支持搜索。
- 服务商与模型图标由 `src/utils/modelBrands.js` 和 `BrandIcon.vue` 提供：服务商按实际 API 主机识别（匹配完整域名或子域名），模型按 ID 独立识别，支持命名空间和 Anthropic 区域前缀；未知品牌使用通用 SVG，不根据昵称猜测。
- 交互动效以约 120–360ms 的透明度和小幅位移/缩放为主；系统减少动画偏好覆盖页面根节点、浮层和子元素。界面图标使用自绘 SVG，不使用 emoji 或猫爪元素。

## 9. 安卓系统提醒

安卓使用系统 `AlarmManager` 和应用内原生接收器；桌面继续使用 Rust 的到期轮询（每 15 秒，使用 `notify-rust 4.12.0`，失败保留状态重试）。安卓进程内 15 秒任务只对账，不重复发通知。

调用链路：

`提醒事务 → SQLite 调度待办 → Rust 平台适配 → 原生系统登记 → AlarmReceiver → JNI / Rust 核心核对并认领 → 原生通知 → 持久化回执 → Rust 续排`

数据结构：

- `reminder_dispatch_clock`：设备数据库的独立 epoch 和单调递增版本。
- `reminder_dispatch`：每个提醒当前的调度版本、登记版本、登记模式、错误和取消原因。SQLite 触发器使创建、修改、删除及备份恢复与调度变更共用事务；失败或取消的对话不留下系统任务。
- `reminder_delivery`：当前版本的执行租约、回执和重试时间。接收器认领和确认都在立即事务中检查调度版本；旧闹铃、旧租约和重复回执不能推进新提醒。
- 系统登记成功只更新登记状态，不将提醒标为送达。通知成功提交后才更新送达和重复时间；失败保留待处理状态，60 秒后允许重新认领。

模块：

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

JNI 与反射用类保留在 ProGuard 规则中；`ReminderBridge` 加载的库名来自 Cargo `[lib] name = "app_lib"`，修改库名时需同步更新。

权限与恢复边界：

- 只声明 `SCHEDULE_EXACT_ALARM`。已获精确权限时使用 `setExactAndAllowWhileIdle`；否则使用 `setAndAllowWhileIdle` 并在界面说明可能延迟。Android 13+ 的通知授权与精确提醒授权分别处理。
- 不依赖常驻服务、前端事件或 LLM。原生 Receiver 冷启动时直接调用 Rust 核心读取同一 SQLite。
- 数据库保存在凭据保护目录，重启后首次解锁再恢复，不在锁屏前复制敏感提醒内容到设备保护目录。
- 用户系统强制停止、精确权限撤销、厂商后台限制和关机会影响调度；重新打开应用后再次对账，精确权限授权、通知权限重新开启后的返回前台同步也会触发恢复。
- SQLite 与 Android 系统调度没有跨系统原子事务；界面区分「已保存」和「已登记」。执行租约、稳定通知 tag（`PendingIntent` 使用包含完整提醒 ID 的内部 URI）和版本检查用于恢复及去重。
- 原生缓存仅保存 ID、版本、登记时间和模式，SQLite 是唯一任务来源。调度元数据与执行回执不导出到其他设备。

设计参考：[Android 系统闹铃](https://developer.android.com/develop/background-work/services/alarms)、[精确提醒授权](https://developer.android.com/about/versions/14/changes/schedule-exact-alarms)、[系统强制停止行为](https://developer.android.com/about/versions/15/behavior-changes-all?hl=en)。
