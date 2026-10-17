# 独立模式架构与管理接口

独立模式在 Tauri/Rust 内运行精简 ReAct，使用自己的角色、提示词和日记数据，不连接或共享 infOS 的实例与记忆。首页采用角色各自的房间与可展开聊天面板，日记、角色资料和衣柜使用次级抽屉。设置中心、角色管理、群聊与小游戏统一主题和导航；远程入口暂未开放，旧远程页面不再挂载，旧话题入口与首页读取已移除。设置直接接入现有协议、思考强度及输出/上下文参数，角色和日记不再保留重复管理界面。提示词工作区的草稿、发布、回滚专用编辑界面仍未提供，接口保留。

## 运行链路

`当前输入 → Rust 校验与上下文编译 → 模型调用 → 工具结果 → 模型继续 → 最终回复与工具草稿一起提交 → TDB 同步 → 前端投影`

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

现有群聊、小游戏和环境信息模块继续复用 Rust 会话服务。前端只负责表单草稿、输入展示、消息预览、动画和交互反馈，不负责模型请求、日记提炼或业务提交。

## 五个小工具

| 名称 | 参数 | 行为 |
| --- | --- | --- |
| `write_diary` | `content`，可选 `date`、`tags`、`id` 与 `expectedRevision` | 新建日记，或修订自己的日记 |
| `query_diary` | 可选 `keyword`、`from`、`to`、`limit` | 查询当前角色已提交的日记，最多 20 条，结果总文本有界 |
| `update_character` | 可选 `state`、`interactions`，至少包含实际更新 | **同一个工具更新角色状态与互动台词** |
| `query_reminders` | 可选 `keyword`、`includeDelivered`、`limit` | 查询当前角色已提交的提醒，最多 20 条，同时返回设备本地时间与 UTC 偏移 |
| `manage_reminder` | `action: create/update/delete`，以及 `task`、`time`、`recur`、`recurUntil`、`id`、`expectedRevision` | 创建、修改或取消提醒；只提交当前角色的变更 |

`update_character.state` 支持 `mood`、`vibe`、`mind`；`interactions` 支持 `welcome`、`idle`、`back`、`click`，点击部位支持 `head`、`chest`、`body`、`default`。Rust 将台词映射到现有 Live2D 字段，只替换指定类别，并清理多余旧台词。

工具写入先返回 `commit: after_assistant_persisted`，最终回复完整且成功持久化之后才生效。回复、日记恢复日志、角色状态、台词和工具审计共用一个 SQLite 事务；参数无效、未知工具只返回结构化工具错误，不能执行生成代码或任意配置写入。

提醒工具遵循同一事务边界，回复失败、停止、超时、修订冲突时不会遗留提醒草稿。创建要求未来的设备本地时间 `YYYY-MM-DD HH:mm:ss`，拒绝不存在或存在夏令时歧义的本地时间；相对时间先查询，含糊时间先向用户澄清。支持一次、每天和每周提醒，以及重复截止时间。修改/删除必须携带最新修订号，同一提醒本轮只允许一次待提交变更；重复创建相同待办、时间和重复规则会报错。提醒是独立任务，成功提交后不随聊天删除或重新生成而自动取消。

桌面 Rust 每 15 秒检查到期提醒，使用 `notify-rust 4.12.0` 取得通知提交结果，失败保留状态重试。安卓改为系统 `AlarmManager` 调度与原生 Receiver，进程内 15 秒任务只对账，不重复发通知；原生事件通过 JNI 交给无 AppHandle 的 Rust 核心检查版本、认领、确认和续排。系统接口接受通知不等于用户已读。循环提醒补发一次后直接推进到未来，不逐次轰炸错过的周期。

安卓已接入关闭界面后的系统登记、执行回执、取消、重复续排与开机解锁恢复。调度待办由 SQLite 触发器与提醒变更一起提交，登记失败可重试；未获精确权限时降级并展示状态，系统强制停止等限制仍适用。前端只负责用户点击后的授权与状态展示，具体链路与未验收范围见 `ANDROID_REMINDERS.md`。

每轮默认最多 8 次模型调用、24 个工具调用，模型调用上限可配置为 1–12。重复工具调用 ID 和相同参数复用结果，身份相同而参数不同直接失败。同一日记本轮只接受一次待提交修订。

正文不再承担记忆、状态或台词控制协议，旧 `MEMORY`、`PEROCUE` 等标签不执行任何副作用。旧标签仅保留显示兼容的剥离逻辑，原元数据写入模块已移除。

## 日记与 TDB

- 使用固定版本 `triviumdb 0.8.8` 的 Rust 核心，每个角色独立存放于应用数据目录的 `data/diary/{角色ID的SHA256}/memory.tdb`。
- 不请求 Embedding，不构建或查询向量索引。TDB 当前写入接口要求固定维度，因此保存一个恒为零的占位槽位，它不参与检索。
- 日期范围、关键词、标签过滤直接读取 TDB 当前条目；关键词按正文和标签进行不区分大小写的包含匹配，结果按日期和时间倒序。没有向量召回或后台记忆加工。
- SQLite `memories` 保留可恢复的写入镜像，`memory_tdb_pending` 记录未同步角色；先提交消息与镜像，再刷新 TDB，刷新成功后才清除恢复任务。这沿用 infOS 的工具草稿与成功回复后提交原则，解决跨存储无法共享事务的问题。
- TDB 不可用时，普通聊天可以暂不加载日记；日记查询显式报告错误，待同步写入保留。后续读取或同步会重试，不返回已知过期条目。
- 日记包含稳定 ID、角色 ID、日期、正文、标签、修订与来源消息 ID。修订必须携带匹配的 `expectedRevision`；修订累积来源，删除相关输入或回复时可以清理衍生日记。
- `local_diary_day` 直接将当天条目按顺序输出为 Markdown，不额外调用模型生成摘要。

## 提示词框架

框架由有序区块组成，每块含 `id`、`source`、`role`、`template`、`enabled`。内置来源为 `system`、`persona`、`user`、`environment`、`state`、`post`、`diary`、`history`、`current`，另外支持 `custom` 文本区块。

`template` 为空时采用该来源的内置模板。变量只支持白名单 `{{变量名}}` 的单次替换，包括角色、用户、环境、状态和 `diary.recent`；没有脚本、表达式或递归模板求值。`history` 与 `current` 是位置区块，不能携带模板，必须各启用一个，且历史位于当前输入之前。当前输入始终按真实用户角色插入。

框架分配支持全局、通道、角色及角色加通道，通道为 `desktop` 或 `group`，优先采用更具体的分配。运行时仅使用已发布版本，保存草稿不会立即影响聊天。管理写入使用 `expectedRevision`，过期编辑返回 `STALE_PROMPT`；保留最近 20 个版本，可回滚。预览只编译，不请求模型，返回消息、区块清单、使用的历史/日记 ID 和预算估算。

上下文预算根据 `contextWindow` 预留输出和工具定义，按完整轮次选取近期已提交历史，并限制最近日记的占比；工具循环继续检查累计上下文，超额显式结束。Token 数是保守的字符估算，图片有固定预算，并非厂商精确 tokenizer。

后端固定工具协议独立于可编辑区块，约束由 Rust 校验和工具执行器落实，不能通过改提示词绕过作用域、参数限制或提交规则。

## 模型 API 与参数

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

`reasoningDialect` 支持 `auto`、`openai`、`deepseek`、`openrouter`、`generic`。自动识别 DeepSeek、OpenRouter 的常见地址或名称；特殊代理可以显式指定。思考启用时省略常见不兼容的采样字段；扩展参数可以覆盖可选字段。厂商各模型接受的强度范围仍由其服务校验，不声称所有模型都支持所有强度。

`extraBody` 不能覆盖权威消息、角色、工具定义、工具选择、流式开关、多候选数量或服务端会话身份等受管理字段；扩展输出额度不能超过本地额度。自定义请求头禁止覆盖 Host、连接长度等传输字段，网络客户端禁用重定向，避免鉴权意外转发。错误输出隐藏 API Key 和自定义头值。

四类协议均分离正文和思考内容，持久化思考内容与逐次用量。ReAct 后续调用保留原生工具调用 ID、Claude 签名、Gemini thought signature，以及 Responses 的加密 reasoning 项。流式工具参数必须累积为完整 JSON 后才执行，单纯 EOF 不视为成功。不存在静默改协议或自动重复计费重试。

协议实现参考：[OpenAI function calling](https://developers.openai.com/api/docs/guides/function-calling)、[DeepSeek thinking mode](https://api-docs.deepseek.com/guides/thinking_mode/)、[Gemini function calling](https://ai.google.dev/gemini-api/docs/function-calling)。实际厂商模型联调留待配置真实服务时进行。

## 生命周期与删除

- 请求 ID 绑定当前输入、目标和角色的指纹。同 ID 不能替换内容；成功重放还要求原回复修订未变化。失败请求需新 ID，已删除回复不能复活。
- 请求超时、用户取消、断流、截断、空回复或工具上限都会终止本轮并丢弃全部工具草稿。失败的已收正文保留为失败消息，不进入后续成功回复历史。
- 重新生成保留原消息 ID 和时间戳；成功才归档原修订、替换正文和衍生日记，失败或取消保留原回复。编译和日记查询排除正被替换回复的衍生日记。
- 删除用户输入会一起删除该轮回复；删除操作清空正文、思考内容和用量，清理修订、工具审计、上下文清单及有来源的日记。当前角色状态属于当前投影，不因历史删除自动倒退。
- 删除、导入、重置与进行中的请求通过运行租约协调。TDB 同步失败保留恢复任务，不重新引入已删除 SQL 记录。
- 提示词和模型设置提交先校验，再进行原子更新；通用 `save_config` 只保留角色选择和界面配置，业务配置需对应管理命令。

## 管理命令与兼容

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

前端封装见 `src/services/standalone.js` 和 `src/services/reminders.js`。配置读取中的隐藏头值仅用于展示，保存时应传实际编辑值，不能将隐藏占位文本当作密钥写回。原 `get_reminders` / `delete_reminder` 裸接口已移除，避免按 ID 无修订、无角色约束地修改任务。备份 `2.0` 保留提醒修订与投递错误，缺少新增字段的旧记录自动使用默认值。

旧角色定义、历史消息、API 配置和记忆继续迁移使用。备份 `2.0` 增加可选的思考、用量、日记修订与来源字段，兼容导入 `1.0`。提示词工作区随配置备份；导入前验证框架、模型参数与数据关系，事务失败整体回滚；重置和导入同时标记 TDB 清理，避免缓存恢复旧日记。旧配置文件继续作为保留的迁移来源，不在启动时覆盖当前数据。

## 测试与验收记录

1.3.4 发布前按用户明确授权运行 `pnpm test` 和 `cargo test --locked --manifest-path src-tauri/Cargo.toml`。前端 35 项通过，覆盖传输取消与事件隔离、消息展示兼容、品牌域名及模型识别、素材完整性，以及全部 27 个 Vue 组件的脚本与模板校验；Rust 后端 52 项通过，主程序与文档测试无额外用例。安卓调度执行代码限定在安卓与测试配置中，共享表及状态投影继续用于所有平台。未运行本地应用构建、打包、启动或安卓真机验收。

以下是此前各阶段的验证记录：

仅执行用户允许的 `cargo test --manifest-path src-tauri/Cargo.toml --lib`，52 项通过。覆盖四类协议的原生 ReAct 往返与流式工具参数、签名保留、模型列表分页、工具作用域与幂等、失败/取消/冲突回滚、请求重放与删除、TDB 重开和故障重试、导入/重置一致性、内置人设和头像及提示词草稿/发布/回滚。提醒覆盖角色隔离、过期/非法时间、重复创建、修订冲突、工具事务回滚、停止后的草稿清理、失败投递重试、重复周期推进、一次送达和备份兼容，以及系统调度事务、旧回调拒绝、执行租约与数据库重开恢复。日记和提醒仅在成功提交后分别返回 `state.diaryWrites` / `state.reminderChanges`，不能把工具草稿当作已写入。

上述 Rust 测试的网络场景使用本地模拟服务，未调用真实模型。UI 开发没有额外运行前端测试、`cargo check`、`cargo build`、生产构建或移动端打包；此前曾按用户明确要求启动桌面 Tauri 供查看。最新的全局界面与设置表单尚未进行运行验收，安卓真机验收也未进行。
