const CACHE_NAME = 'ppc-cache-v2'

self.addEventListener('install', (event) => {
  self.skipWaiting()
})

self.addEventListener('activate', (event) => {
  event.waitUntil((async () => {
    const keys = await caches.keys()
    await Promise.all(keys.filter((k) => k !== CACHE_NAME).map((k) => caches.delete(k)))
  })())
  self.clients.claim()
})

self.addEventListener('fetch', (event) => {
  const req = event.request
  if (req.method !== 'GET') return
  event.respondWith((async () => {
    try {
      const url = new URL(req.url)
      // 仅处理 http 和 https 协议，忽略 chrome-extension 等
      if (!['http:', 'https:'].includes(url.protocol)) {
        return fetch(req)
      }
      if (url.pathname.startsWith('/live2d-widget/')) {
        return await fetch(req)
      }
    } catch (_) {}
    try {
      const res = await fetch(req)
      const cache = await caches.open(CACHE_NAME)
      // 再次检查协议，确保 cache.put 不会因非 http(s) 协议报错
      const url = new URL(req.url)
      if (['http:', 'https:'].includes(url.protocol)) {
        cache.put(req, res.clone())
      }
      return res
    } catch (e) {
      const cached = await caches.match(req)
      if (cached) return cached
      throw e
    }
  })())
})
