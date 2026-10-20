# 网页稿独立素材生成记录

当前决定：自制动态立绘已停止，网页设计稿使用原画静态 SVG；正式应用保留 Pio 通用模型，并与静态立绘／CG 提供可持久化的切换入口。以下动态分层、播放缓存与动作内容均为历史试作记录，不代表当前网页或正式应用仍在加载这些自制动画。

使用内置 image_gen，参考已认可的第二轮浅色概念图。网页控件与文字没有包含在这些素材中，均由 HTML/CSS 排版。

## 立绘 SVG 更新

用户要求提升立绘清晰度并转换为 SVG 后，另外制作简洁手绘版与原画路径版。原画静态对照使用 `assets/pero-standing.svg`：以原始 PNG 为来源，清理低不透明度的边缘光晕，对轮廓做轻量平滑，再通过 VTracer 分色为闭合三次曲线路径。简洁手绘版保留为 `assets/pero-standing-flat.svg`，未覆盖原始 PNG。

- 描摹来源：`assets/pero-standing.png`。
- 轮廓处理：Alpha 以 128 为界二值化，3 像素中值平滑；不放大或伪造原图细节。
- 路径参数：color / stacked / spline，filter_speckle=2，color_precision=8，layer_difference=4，corner_threshold=60，length_threshold=3.5，max_iterations=10，splice_threshold=45，path_precision=2。
- 补充 `viewBox="0 0 1024 1536"`、中文标题和描述。XML 核对为 10664 个路径、0 个 image 元素，无 Base64 或外部位图引用。
- 网页取消角色层的模糊投影。离线成图仅用于查看素材，网页加载的是 SVG。

## 分层互动试作

此简化手绘试作因画风与原画不一致，已从主设计稿撤下。后续曾继续尝试原画矢量分层方案，目前两种自制动态方案均已停止。

用户同意试作轻量角色互动后，以简洁手绘版为基础整理 `assets/pero-interactive.svg`。保留完整身体构图，拆分头部前后层、猫耳、虹膜、眼睑、口部、上身、抬起的手、裙摆与领结；原来的单眼眨眼姿态补为双眼，运行时可眨眼和短暂闭眼回应。SVG 不含图片或脚本，预览脚本仅驱动这些固定分组。

同一份模型标记保存在 `puppet-model.js`，试作期间曾由主设计稿通过普通本地脚本加载并直接绘制，不再嵌套 `puppet.html`，不需要开发服务器。`puppet.html` 保留为历史实验页。`puppet.js` 提供小幅刚性变换与眼睛形变；这不是 Cubism 建模，也不表示原画路径版具有可直接驱动的语义图层。`assets/pero-interactive-preview.png` 是静态 SVG 的离线成图，只供检查素材。

## 原画矢量分层动态版（已停止）

用户要求继续照原稿做动态立绘后，直接保留 `assets/pero-standing.svg` 的原画路径，使用 `prepare-original-rig.py` 整理 `assets/pero-original-rig.svg` 与 `original-puppet-model.js`。本轮没有生成替代人物图片，也没有使用简化手绘立绘。

- 原画路径集中保存在 SVG 定义组，各动作层通过裁剪与遮罩引用，避免将原图重复描成另一种画风。
- 头部沿发梢与衣领分区，手部在腕部划分；补齐颈部与腕部接缝，小幅动作保持原有比例。
- 保留闭着的左眼，右眼拆出眼白、眼线、虹膜与闭眼层；闭眼肤色从原画脸颊纹理延展，边缘柔化，眼睑局部补绘。
- 沿用原画嘴部形状，轻触回应时仅小幅变形；没有实现音频口型。
- 美术观察稿包含静止、闭眼与歪头姿态，离线栅格化检查的是 SVG 素材，没有执行网页动画脚本。
- 当前为 SVG 动作试作，没有生成 Cubism 工程、网格或 `.moc3` 文件。

## 动态播放缓存与连续动作（历史试作）

用户反馈帧数低后，播放层改为原画透明纹理组合。`prepare-original-rig.py` 保留完整矢量稿并写入各部分的离线源稿和坐标配方，`prepare-original-textures.cjs` 使用 Sharp 按原稿坐标的 1.5 倍尺寸渲染身体、脸、眼睑、眼白、虹膜、闭眼、嘴和手部；额外渲染小尺寸虹膜透明遮罩。没有生成新人物，也没有简化原画路径。

网页加载并解码这些图层后再替换静态立绘，播放只更新 CSS 变换和透明度。呼吸缩放改为六位精度，视线使用临界阻尼跟随，眨眼过渡与回应动作采用平滑曲线。离屏暂停保留当前姿态与动画时间。`pero-original-rig-texture-preview.png` 是透明图层的离线静止合成，用于观察外观；没有通过执行网页或采集帧率来验证性能。

## 透明角色立绘

- 项目文件：assets/pero-standing.png。
- 原始文件：C:/Users/Administrator/.codex/generated_images/01a13bf5-718b-7443-9323-ea4219c3d998/exec-59a8c3f3-0dfc-487b-a4b8-a770178b4a5d.png。
- 参考角色 CG：../remote-client-prototype/assets/pero-cg.png。
- 参考布局：../standalone-concepts/standalone-light-v2.png。
- 透明背景：true；文件元数据核对为 1024×1536，角落及上下文采样边缘的 Alpha 为 0。未编辑生成图片。

```text
Generate one standalone transparent-background full-body anime character sprite for the PeroperoChat HTML design prototype. References: first image character identity only, second image approved UI concept only to establish smaller standing full-body sprite style. Output ONLY Pero, no room, no UI, no text, no frame, no shadow plane, genuinely transparent alpha. Pero has short cyan-blue hair, blue cat ears with pink inner ears, bright blue eyes, cheerful cute round face, small black hair bows, white frilled maid headband. Modest black-and-white maid dress with white apron, small burgundy bow at collar with blue jewel, knee-length full skirt, white opaque stockings and small dark shoes. Full body from ear tips to shoe soles entirely within frame with comfortable margin. Naturally standing, weight gently on one foot, one hand softly holding apron edge, other hand raised in friendly small wave at shoulder. Hands anatomically correct. Clean refined cel-shaded anime / Live2D model presentation, crisp smooth linework, simple soft fabric shading, charming expressive eyes, no oily face highlights, no photographic rendering. Balanced natural proportions, cute modest character, not a glamour pose, not kneeling. Character facing viewer, gentle open smile and one eye playfully closed. The transparent sprite must work as a separate HTML image placed into an illustrated bedroom; do not bake the room or any interface into the image.
```

## 独立房间背景

- 项目文件：assets/sunlit-room.png。
- 原始文件：C:/Users/Administrator/.codex/generated_images/01a13bf5-718b-7443-9323-ea4219c3d998/exec-95c763ab-1031-470f-bdf4-fdb3c6c2faf3.png。
- 参考布局：../standalone-concepts/standalone-light-v2.png。
- 透明背景：false；房间不含人物与界面。

```text
Create ONE standalone portrait 9:19 anime bedroom BACKGROUND illustration for the PeroperoChat mobile companion app. Input is approved FOUR-SCREEN UI concept as STYLE AND ROOM reference ONLY. Recreate and expand the cozy sunlit room visible behind the FIRST phone, as an independent full-canvas empty room, absolutely NO characters, NO human figure, NO face, NO UI, NO text, NO phone, NO borders, NO speech bubbles. Full room backdrop, straight-on slightly elevated eye-level perspective, generous clear empty central floor for a separately overlaid small Live2D character. Left large tall window with white sheer curtains, crisp blue sky and soft clouds, sunlight falling onto warm pale wood plank floor; a small white desk and chair beneath left window, tidy small vase and one book. Right side a cozy white bed with pale sky-blue checked quilt and pillows, wooden shelf behind holding a few books and a green houseplant. Rear pale ivory-blue walls, small framed botanical print. Clear architectural perspective and attractive anime game background painting, nuanced cel-painted surfaces with soft natural daylight, beautiful refined detail without clutter, not photorealistic, not shiny oily rendering, not a generic flat cartoon vector. Palette restrained fresh blue-white, gentle wood and green accent. Center around 35-85% image height is visually quiet for character and message overlay, but actual floor starts around lower third. Keep no giant empty featureless wall at top. Match the approved room atmosphere without copying phone controls or figures. Premium Japanese visual-novel interior background art, comfortable lived-in modest room, not a Victorian ballroom, no excessive flowers or ornamental frames.
```

## 1.4.1 Nana 透明立绘

- 使用内置 `image_gen` 编辑模式，输入 `public/companion/nana-cg.png`，不经外部 CLI。
- 输出已保存为 `public/companion/nana-standing.png`，RGBA 1122 × 1402，Alpha 覆盖 0–255；原 CG 保留。
- 用途：独立房间透明人物，远程完整 CG 不替换。

```text
Use case: background-extraction. Edit target: the provided existing Nana character CG. Make a production transparent PNG character cutout for a companion app. Preserve this exact anime elf girl's identity, lavender twin tails, ahoge, black ribbon bows, purple eye and wink, face, playful hand by her eye, black jacket draped over shoulders, printed white T-shirt, plaid skirt, seated pose and visible thighs. Preserve original drawing, colors, proportions, details and framing; do not redesign or replace the character. Remove ALL room/window/night/city/moon/lights/books/bed/pillows/blankets and decorative background elements, including behind hair strands and gaps between arms and body. Only the girl and her worn clothes/accessories remain. Keep clean antialiased hair edges and genuine alpha transparency, no white/matte/checkerboard background, no glow, no cast shadow or text. Include the complete visible silhouette from ahoge to lower thighs within bounds, with only a small transparent margin. Do not add limbs or invent unseen body parts.
```
