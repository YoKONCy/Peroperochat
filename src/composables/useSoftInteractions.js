import { onMounted, onBeforeUnmount } from 'vue'

// 只在真实点击后轻弹图标，不轮询、不逐帧修改页面，也不重播流式消息。
export function useSoftInteractions() {
  const active = new Set(), owners = new WeakMap()
  let preference
  function respond(event) {
    if (!(event.target instanceof Element) || preference?.matches || document.hidden) return
    const button = event.target.closest('button'), surface = button?.closest('[data-ui="soft"]')
    if (!surface || button.matches(':disabled') || button.closest('[inert], [data-reduce-motion="true"]')) return
    if (button.matches('.room-static-character, .remote-inline-picture, .remote-file-miniature')) return
    const icon = button.querySelector(':scope > svg, :scope > span > svg')
    if (!icon || icon.closest('.is-loading, .is-spinning, .room-spinning') || typeof icon.animate !== 'function') return
    owners.get(icon)?.cancel()
    const animation = icon.animate([
      { scale: '1', translate: '0 0' },
      { scale: '1.12', translate: '0 -1px', offset: .45 },
      { scale: '1', translate: '0 0' },
    ], { duration: 240, easing: 'cubic-bezier(.2,.75,.3,1)' })
    active.add(animation); owners.set(icon, animation)
    const release = () => { active.delete(animation); if (owners.get(icon) === animation) owners.delete(icon) }
    animation.finished.then(release, release)
  }
  function stop() { for (const animation of active) animation.cancel(); active.clear() }
  function changed() { if (preference.matches) stop() }
  function hidden() { if (document.hidden) stop() }
  onMounted(() => {
    preference = window.matchMedia('(prefers-reduced-motion: reduce)')
    document.addEventListener('click', respond, { passive: true })
    document.addEventListener('visibilitychange', hidden)
    preference.addEventListener('change', changed)
  })
  onBeforeUnmount(() => {
    document.removeEventListener('click', respond)
    document.removeEventListener('visibilitychange', hidden)
    preference?.removeEventListener('change', changed)
    stop()
  })
}
