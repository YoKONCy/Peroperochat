// 将离线分层 SVG 渲染成透明美术纹理；不启动应用或执行网页脚本。
const fs = require('node:fs/promises')
const path = require('node:path')
const os = require('node:os')
const sharp = require(process.argv[2] || 'sharp')

async function prepare() {
  const studies = path.join(os.tmpdir(), 'peropero-original-rig-study')
  const recipe = JSON.parse(await fs.readFile(path.join(studies, 'texture-recipe.json'), 'utf8'))
  const destination = path.join(__dirname, 'assets', 'original-rig')
  await fs.mkdir(destination, { recursive: true })
  // 顺序处理完整原画，避免同时展开多份矢量路径占用过多内存。
  for (const [name, layer] of Object.entries(recipe.layers)) {
    await sharp(path.join(studies, `layer-${name}.svg`), { density: 72 * recipe.textureScale })
      .resize(Math.round(layer.bounds[2] * recipe.textureScale), Math.round(layer.bounds[3] * recipe.textureScale))
      .png().toFile(path.join(destination, `${name}.png`))
  }
  await sharp(path.join(studies, 'layer-iris-mask.svg'), { density: 72 * recipe.textureScale })
    .resize(56, 68).png().toFile(path.join(destination, 'iris-mask.png'))
  console.log('原画分层透明纹理已更新。')
}
prepare().catch(error => { console.error('原画纹理整理失败：', error); process.exitCode = 1 })
