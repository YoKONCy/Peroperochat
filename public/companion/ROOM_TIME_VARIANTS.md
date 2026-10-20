# 房间时段背景

2026-10-21 使用内置 `image_gen` 为现有房间制作光照变体，未使用外部 CLI，图片原样复制到 `public/companion/`。

| 时段 | 设备本地时间 | Pero / 自定义角色默认 | Nana |
| --- | --- | --- | --- |
| 清晨 / 黄昏 | 05:00–08:00、17:00–20:00 | `pero-twilight-room.png` | `nana-twilight-room.png` |
| 白天 | 08:00–17:00 | `sunlit-room.png`（沿用原图） | `nana-day-room.png` |
| 晚上 | 20:00–次日 05:00 | `pero-night-room.png` | `moonlit-room.png`（沿用原图） |

时段按设备当前本地时区计算，支持半小时 / 四十五分钟偏移以及切屏后更改时区、夏令时后的新偏移。没有定位或联网日出日落查询。

Rust `local_room_background` 接收切屏时采集的设备 UTC 偏移，自行读取当前时间、判断时段和选择素材。前端只保存本次房间快照，不执行时段判断；切换角色、重新进入房间、应用回到前台时刷新。持续停留时不设定时器，不因聊天状态、换装或环境投影变化而切换背景。内置图片在切换前先解码，保留现有淡入过渡。

空背景与原内置默认路径启用自动房间；上传图片、其他素材地址或外链固定显示。自定义角色的自动房间与 Pero 一致，不覆盖已有自定义图片。

素材来源参考分别为 `sunlit-room.png`（Pero）和 `moonlit-room.png`（Nana），仅调整时段光照与窗外天空，保持房间布局和主家具一致。原有背景来源见 [背景记录](ROOM_BACKGROUNDS.md)。

## 最终生成提示词

### pero-twilight-room.png

参考：`sunlit-room.png`。

```text
Use case: lighting-weather. Edit target: the supplied existing portrait anime bedroom background. Make ONE production background image, preserving the EXACT room, camera, perspective, crop, furniture positions, curtains, plant silhouettes, bedding folds, all decor and foreground open floor. This is the SAME ROOM at a different time, not a redesign. Preserve the existing refined anime visual-novel painted style and portrait 9:19 composition. Change ONLY natural lighting, exterior sky, believable light/shadow colors and whether existing lamps are lit. No people, animals, UI, text, watermarks, added furniture, collage or multiple panels. No photorealism, 3D, heavy bloom, global flat color filter or oily highlights. Pero's blue-white daylight room becomes the soft golden-hour version shared by dawn and dusk: peach and pale rose clouds outside, low warm sun mostly outside the frame, long warm light patches across the same floor, fresh pale blue curtains and blue checked quilt remain recognizable and retain blue local colors, natural soft peach reflections with gentle cool shadows. No moon or stars; desk and bed remain in exactly the same place. Warm and airy, restrained rather than orange-tinted.
```

### pero-night-room.png

参考：`sunlit-room.png`。

```text
Use case: lighting-weather. Edit target: the supplied existing portrait anime bedroom background. Make ONE production background image, preserving the EXACT room, camera, perspective, crop, furniture positions, curtains, plant silhouettes, bedding folds, all decor and foreground open floor. This is the SAME ROOM at a different time, not a redesign. Preserve the existing refined anime visual-novel painted style and portrait 9:19 composition. Change ONLY natural lighting, exterior sky, believable light/shadow colors and whether existing lamps are lit. No people, animals, UI, text, watermarks, added furniture, collage or multiple panels. No photorealism, 3D, heavy bloom, global flat color filter or oily highlights. Pero's blue-white room at quiet night: indigo-blue sky and subtle stars, a small natural moon visible outside the same window, pale cool blue moonlight, a gentle warm glow from the EXISTING small bedside lamp only where a lamp already exists; if none, window moonlight alone provides sufficient legible illumination. White wood, pale blue curtains and blue checked quilt remain clearly identifiable. Comfortable restful blue midtones, readable room detail, not purple, pitch black or neon. Keep every object and all geometry unchanged.
```

### nana-day-room.png

参考：`moonlit-room.png`。

```text
Use case: lighting-weather. Edit target: the supplied existing portrait anime bedroom background. Make ONE production background image, preserving the EXACT room, camera, perspective, crop, furniture positions, curtains, plant silhouettes, bedding folds, all decor and foreground open floor. This is the SAME ROOM at a different time, not a redesign. Preserve the existing refined anime visual-novel painted style and portrait 9:19 composition. Change ONLY natural lighting, exterior sky, believable light/shadow colors and whether existing lamps are lit. No people, animals, UI, text, watermarks, added furniture, collage or multiple panels. No photorealism, 3D, heavy bloom, global flat color filter or oily highlights. Nana's purple room becomes clear daytime: bright soft blue sky with white clouds and the SAME distant town outside, REMOVE the moon, stars and illuminated nighttime building windows; daylight enters naturally through the same window. Existing lamps remain in place but are switched OFF. Preserve the purple/lavender curtains, violet bedding and dark wood furniture as material colors, softened by fresh daylight, gentle wood floor reflections. Luminous lilac and ivory midtones with small warm sun patches, not a nighttime purple filter. All furnishings and ornaments exactly unchanged.
```

### nana-twilight-room.png

参考：`moonlit-room.png`。

```text
Use case: lighting-weather. Edit target: the supplied existing portrait anime bedroom background. Make ONE production background image, preserving the EXACT room, camera, perspective, crop, furniture positions, curtains, plant silhouettes, bedding folds, all decor and foreground open floor. This is the SAME ROOM at a different time, not a redesign. Preserve the existing refined anime visual-novel painted style and portrait 9:19 composition. Change ONLY natural lighting, exterior sky, believable light/shadow colors and whether existing lamps are lit. No people, animals, UI, text, watermarks, added furniture, collage or multiple panels. No photorealism, 3D, heavy bloom, global flat color filter or oily highlights. Nana's purple room at dawn/dusk shared golden hour: soft lavender, peach and rose sky over the SAME distant town, no full moon or night stars, restrained city lights rather than a nighttime skyline. Low warm sunlight falls through the same window and makes long soft peach reflections on the same wood floor; lavender curtains and purple bedding retain their local colors. Existing lamps have a subtle warm glow without overpowering natural light. A gentle dreamlike rose-lilac mood, still readable and distinctly different from full night. All furniture and ornament positions unchanged.
```

