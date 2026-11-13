# 房间背景素材说明

内置陪伴房间支持根据设备本地时间自动切换 **白天**、**清晨 / 黄昏** 与 **晚上** 三套场景插画（详见 [房间时段背景](ROOM_TIME_VARIANTS.md)）。本目录下的基础原图分别作为 Pero 日间房间与 Nana 夜间房间：

- `sunlit-room.png`：Pero 的日间默认房间；其他自定义角色未单独设置背景且未自定义外链时沿用这套自动房间规则。
- `moonlit-room.png`：Nana 的夜间月光房间，与日间版保持同一视角比例并预留中央站位区域以叠加角色立绘或 Live2D 模型。
- **自定义背景**：可在「角色管理 → 编辑角色 → 基础信息 → 房间背景」上传 PNG / JPEG / WebP 图片（上限 5 MiB）或填写图片地址，随角色配置持久化与备份导出；清空自定义背景后恢复对应角色的时段自动房间。
- **旧版路径迁移**：早期默认路径 `/rooms/nana.svg` 自动映射至 Nana 新版背景，非 Nana 角色的 `/rooms/pero.svg` 映射至默认自动房间，用户已上传的自定义图片保持不变。

## Nana 房间基础图生成提示词（参考）

本图基于 `sunlit-room.png` 的构图与视角参考及 `nana-cg.png` 的配色氛围生成：

```text
Use case: stylized-concept. Asset type: production portrait 9:19 anime visual-novel bedroom background for a mobile companion app. Input image 1 is a STYLE / architectural composition reference only: the existing Pero sunlit room. Input image 2 is Nana's CHARACTER AND MOOD reference only; do NOT draw the girl. Create one beautiful standalone EMPTY Nana bedroom, full image with NO characters, NO UI, NO text, NO frames. Match image 1's refined anime-painted room detail, coherent architectural perspective and generous uncluttered central foreground floor for an independently overlaid character. Reimagine the room to suit the playful lavender-haired elf girl in image 2: sophisticated cozy lilac, muted indigo and charcoal accents, a large left-side window with delicate purple curtains, subtle moon and distant softly lit nighttime town, a small dark wooden desk with a few books and a warm small glass lamp, right-side bed with soft pale-lavender blanket, neat shelves and a trailing plant, small restrained star-shaped bedroom ornament. Eye-level slight downward perspective, furniture near image sides and back, clear central standing space and visible wood floor across the lower half, attractive real room proportions, no giant empty wall. Lighting: soft cool moonlight complemented by small warm bedside light, readable restful midtones, not pitch black, no harsh neon, no oily bloom, no excessive sparkle. Crisp nuanced Japanese anime game background art, clean edges, painterly cel shading, quietly personal and lived-in, slightly mischievous rather than gothic luxury. No person, silhouette, face, human shadow, pet, logos, writing, watermark, symbols resembling interface controls, photo realism, 3D render, vector shapes, lace overload or clutter. Compose for vertical phone cropping while keeping the open floor center and window/bed visible.
```
