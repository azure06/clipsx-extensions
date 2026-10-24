import { boundTransform, zoomTransform } from './viewport.mjs'
// Native WebKit magnification uses a cumulative scale, independent of Ctrl+wheel.
export function attachTrackpadGestures(element, current, update, content, viewport, cancelPointers) {
  let origin = null
  const point = event => {
    const rect = element.getBoundingClientRect()
    return { x: Number.isFinite(event.clientX) ? event.clientX - rect.left : rect.width / 2, y: Number.isFinite(event.clientY) ? event.clientY - rect.top : rect.height / 2 }
  }
  const start = event => {
    if (!Number.isFinite(event.scale) || event.scale <= 0) return
    event.preventDefault(); cancelPointers()
    origin = { transform: current(), scale: event.scale, point: point(event) }
  }
  const change = event => {
    if (!origin || !Number.isFinite(event.scale) || event.scale <= 0) return
    event.preventDefault()
    const next = zoomTransform(origin.transform, origin.transform.scale * event.scale / origin.scale, origin.point, content, viewport())
    const center = point(event)
    update(boundTransform({ ...next, x: next.x + center.x - origin.point.x, y: next.y + center.y - origin.point.y }, content, viewport()))
  }
  const end = event => { if (origin) event.preventDefault(); origin = null }
  const cancel = () => { origin = null }
  const window = element.ownerDocument?.defaultView
  element.addEventListener('gesturestart', start, { passive: false })
  element.addEventListener('gesturechange', change, { passive: false })
  element.addEventListener('gestureend', end, { passive: false })
  window?.addEventListener('blur', cancel)
  return { active: () => origin !== null, dispose: () => { element.removeEventListener('gesturestart', start); element.removeEventListener('gesturechange', change); element.removeEventListener('gestureend', end); window?.removeEventListener('blur', cancel); origin = null } }
}
