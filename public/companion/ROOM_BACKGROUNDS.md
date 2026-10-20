# 房间背景

房间已扩展为白天、清晨 / 黄昏、晚上三种背景，按设备本地时区在切屏时选择。六张素材、时段规则及变体生成提示词见 [房间时段背景](ROOM_TIME_VARIANTS.md)。下列两张原图分别保留为 Pero 的白天与 Nana 的晚上版本。

- `sunlit-room.png`：Pero 的日间房间，其他自定义角色未设置背景时也使用此图。
- `moonlit-room.png`：Nana 的月光房间，与日间房间保持一致的插画细节和构图，保留中央空地以展示立绘或通用 Pio。
- 自定义背景在「角色管理 → 编辑角色 → 基础信息 → 房间背景」上传，支持不超过 5MB 的 PNG / JPEG / WebP；也可填写图片地址。保存角色后持久化，并随角色导出与备份保留。清空背景恢复对应角色的默认房间。
- 旧默认 `/rooms/nana.svg` 迁移为 Nana 新背景；非 Nana 角色的旧 `/rooms/pero.svg` 迁移为日间房间。其他已有自定义图片不替换。

## Nana 背景生成记录

2026-10-21 使用内置 `image_gen`，未调用外部 CLI。参考图为 `sunlit-room.png`（风格与构图）和 `nana-cg.png`（配色与气氛，不绘制人物）。生成图已原样保存到本目录 `moonlit-room.png`，未对图片做后处理。

最终提示词：

```text
Use case: stylized-concept. Asset type: production portrait 9:19 anime visual-novel bedroom background for a mobile companion app. Input image 1 is a STYLE / architectural composition reference only: the existing Pero sunlit room. Input image 2 is Nana's CHARACTER AND MOOD reference only; do NOT draw the girl. Create one beautiful standalone EMPTY Nana bedroom, full image with NO characters, NO UI, NO text, NO frames. Match image 1's refined anime-painted room detail, coherent architectural perspective and generous uncluttered central foreground floor for an independently overlaid character. Reimagine the room to suit the playful lavender-haired elf girl in image 2: sophisticated cozy lilac, muted indigo and charcoal accents, a large left-side window with delicate purple curtains, subtle moon and distant softly lit nighttime town, a small dark wooden desk with a few books and a warm small glass lamp, right-side bed with soft pale-lavender blanket, neat shelves and a trailing plant, small restrained star-shaped bedroom ornament. Eye-level slight downward perspective, furniture near image sides and back, clear central standing space and visible wood floor across the lower half, attractive real room proportions, no giant empty wall. Lighting: soft cool moonlight complemented by small warm bedside light, readable restful midtones, not pitch black, no harsh neon, no oily bloom, no excessive sparkle. Crisp nuanced Japanese anime game background art, clean edges, painterly cel shading, quietly personal and lived-in, slightly mischievous rather than gothic luxury. No person, silhouette, face, human shadow, pet, logos, writing, watermark, symbols resembling interface controls, photo realism, 3D render, vector shapes, lace overload or clutter. Compose for vertical phone cropping while keeping the open floor center and window/bed visible.
```
