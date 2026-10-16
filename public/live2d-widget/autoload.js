;(function () {
  function loadResource(url, type) {
    if (url.endsWith('/live2d.min.js') && typeof window.loadlive2d === 'function') return Promise.resolve()
    if (url.endsWith('/waifu-config.js') && window.refreshWaifuTexts) return Promise.resolve()
    if (url.endsWith('/waifu-tips.js') && window.initWidget) return Promise.resolve()
    return new Promise((resolve, reject) => {
      const element = document.createElement(type === 'css' ? 'link' : 'script')
      const timer = setTimeout(() => { element.remove(); reject(new Error('角色资源加载超时')) }, 15000)
      if (type === 'css') { element.rel = 'stylesheet'; element.href = url }
      else element.src = url
      element.onload = () => { clearTimeout(timer); resolve() }
      element.onerror = () => { clearTimeout(timer); element.remove(); reject(new Error('角色资源加载失败')) }
      document.head.appendChild(element)
    })
  }
  window.__PPC_LIVE2D_READY = Promise.all([
    loadResource('/live2d-widget/live2d.min.js', 'js').catch(() => loadResource('https://fastly.jsdelivr.net/gh/stevenjoezhang/live2d-widget@latest/live2d.min.js', 'js')),
    loadResource('/live2d-widget/waifu-config.js', 'js'),
    loadResource('/live2d-widget/waifu-tips.js', 'js'),
  ]).then(async () => {
    if (window.__waifuTextsReady) await window.__waifuTextsReady.catch(() => {})
    if (!document.getElementById('waifu')) {
      window.initWidget({ waifuPath: '/live2d-widget/waifu-texts.json', cdnPath: '/live2d-widget/' })
      // Vue 面板准备好后再显示，避免异步加载结束时角色出现在其他页面。
      const waifu = document.getElementById('waifu')
      if (waifu) waifu.style.display = 'none'
    }
  })
})()
