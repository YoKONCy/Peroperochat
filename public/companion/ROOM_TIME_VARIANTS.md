# 房间时段背景说明

`public/companion/` 目录存放 Pero 与 Nana 各自三套时段的本地房间插画，全部由对应主图保持房间布局与家具透视一致性派生光照变体（基础原图说明见 [房间背景素材说明](ROOM_BACKGROUNDS.md)）。

## 时段与素材映射

| 时段 | 设备本地时间 | Pero / 未指定自定义角色 | Nana |
| :--- | :--- | :--- | :--- |
| **清晨 / 黄昏** | `05:00–08:00`、`17:00–20:00` | `pero-twilight-room.png` | `nana-twilight-room.png` |
| **白天** | `08:00–17:00` | `sunlit-room.png` | `nana-day-room.png` |
| **晚上** | `20:00–次日 05:00` | `pero-night-room.png` | `moonlit-room.png` |

## 调度与加载规则

- 时段计算完全离线，基于系统当前本地时区偏移（兼容半小时/四十五分钟时区与夏令时变更），不调用定位或联网日出日落接口。
- Rust 命令 `local_room_background` 接收切屏时采集的本地 UTC 偏移并决定返回素材；前端仅保存当前房间快照。
- 背景仅在切换角色、进入/切回房间或应用返回前台时刷新（图片解码完成后淡入切换），持续停留房间期间不额外设后台定时器或因对话/换装触发刷新。
- 角色背景为空或指向旧版默认路径时启用自动时段房间；用户上传图片或设置自定义外链时固定显示自定义背景。

## 时段变体生成提示词（参考）

### `pero-twilight-room.png`（参考：`sunlit-room.png`）

```text
Use case: lighting-weather. Edit target: the supplied existing portrait anime bedroom background. Make ONE production background image, preserving the EXACT room, camera, perspective, crop, furniture positions, curtains, plant silhouettes, bedding folds, all decor and foreground open floor. This is the SAME ROOM at a different time, not a redesign. Preserve the existing refined anime visual-novel painted style and portrait 9:19 composition. Change ONLY natural lighting, exterior sky, believable light/shadow colors and whether existing lamps are lit. No people, animals, UI, text, watermarks, added furniture, collage or multiple panels. No photorealism, 3D, heavy bloom, global flat color filter or oily highlights. Pero's blue-white daylight room becomes the soft golden-hour version shared by dawn and dusk: peach and pale rose clouds outside, low warm sun mostly outside the frame, long warm light patches across the same floor, fresh pale blue curtains and blue checked quilt remain recognizable and retain blue local colors, natural soft peach reflections with gentle cool shadows. No moon or stars; desk and bed remain in exactly the same place. Warm and airy, restrained rather than orange-tinted.
```

### `pero-night-room.png`（参考：`sunlit-room.png`）

```text
Use case: lighting-weather. Edit target: the supplied existing portrait anime bedroom background. Make ONE production background image, preserving the EXACT room, camera, perspective, crop, furniture positions, curtains, plant silhouettes, bedding folds, all decor and foreground open floor. This is the SAME ROOM at a different time, not a redesign. Preserve the existing refined anime visual-novel painted style and portrait 9:19 composition. Change ONLY natural lighting, exterior sky, believable light/shadow colors and whether existing lamps are lit. No people, animals, UI, text, watermarks, added furniture, collage or multiple panels. No photorealism, 3D, heavy bloom, global flat color filter or oily highlights. Pero's blue-white room at quiet night: indigo-blue sky and subtle stars, a small natural moon visible outside the same window, pale cool blue moonlight, a gentle warm glow from the EXISTING small bedside lamp only where a lamp already exists; if none, window moonlight alone provides sufficient legible illumination. White wood, pale blue curtains and blue checked quilt remain clearly identifiable. Comfortable restful blue midtones, readable room detail, not purple, pitch black or neon. Keep every object and all geometry unchanged.
```

### `nana-day-room.png`（参考：`moonlit-room.png`）

```text
Use case: lighting-weather. Edit target: the supplied existing portrait anime bedroom background. Make ONE production background image, preserving the EXACT room, camera, perspective, crop, furniture positions, curtains, plant silhouettes, bedding folds, all decor and foreground open floor. This is the SAME ROOM at a different time, not a redesign. Preserve the existing refined anime visual-novel painted style and portrait 9:19 composition. Change ONLY natural lighting, exterior sky, believable light/shadow colors and whether existing lamps are lit. No people, animals, UI, text, watermarks, added furniture, collage or multiple panels. No photorealism, 3D, heavy bloom, global flat color filter or oily highlights. Nana's purple room becomes clear daytime: bright soft blue sky with white clouds and the SAME distant town outside, REMOVE the moon, stars and illuminated nighttime building windows; daylight enters naturally through the same window. Existing lamps remain in place but are switched OFF. Preserve the purple/lavender curtains, violet bedding and dark wood furniture as material colors, softened by fresh daylight, gentle wood floor reflections. Luminous lilac and ivory midtones with small warm sun patches, not a nighttime purple filter. All furnishings and ornaments exactly unchanged.
```

### `nana-twilight-room.png`（参考：`moonlit-room.png`）

```text
Use case: lighting-weather. Edit target: the supplied existing portrait anime bedroom background. Make ONE production background image, preserving the EXACT room, camera, perspective, crop, furniture positions, curtains, plant silhouettes, bedding folds, all decor and foreground open floor. This is the SAME ROOM at a different time, not a redesign. Preserve the existing refined anime visual-novel painted style and portrait 9:19 composition. Change ONLY natural lighting, exterior sky, believable light/shadow colors and whether existing lamps are lit. No people, animals, UI, text, watermarks, added furniture, collage or multiple panels. No photorealism, 3D, heavy bloom, global flat color filter or oily highlights. Nana's purple room at dawn/dusk shared golden hour: soft lavender, peach and rose sky over the SAME distant town, no full moon or night stars, restrained city lights rather than a nighttime skyline. Low warm sunlight falls through the same window and makes long soft peach reflections on the same wood floor; lavender curtains and purple bedding retain their local colors. Existing lamps have a subtle warm glow without overpowering natural light. A gentle dreamlike rose-lilac mood, still readable and distinctly different from full night. All furniture and ornament positions unchanged.
```
