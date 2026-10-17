// 只加载随应用分发的展示素材；不接入独立模式组件、配置、台词或业务接口。
const model = Number(new URLSearchParams(location.search).get('model'))
if ([0, 1, 3].includes(model) && typeof window.loadlive2d === 'function') {
  const canvas = document.getElementById('remote-live2d')
  const ratio = Math.min(window.devicePixelRatio || 1, 2)
  canvas.width = Math.round(window.innerWidth * ratio)
  canvas.height = Math.round(window.innerHeight * ratio)
  window.loadlive2d('remote-live2d', `/live2d-widget/model/potion-Maker-Pio/index${model ? `_${model}` : ''}.json`)
} else {
  const message = document.createElement('p')
  message.textContent = '角色形象暂时无法显示'
  document.body.replaceChildren(message)
}
