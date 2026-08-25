const context = window.ClipsX.context
document.documentElement.dataset.theme = context.theme === 'dark' ? 'dark' : 'light'

const escapeHtml = value => String(value).replace(/[&<>"]/g, char => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[char])
const icon = `<svg class="copy" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><rect x="9" y="9" width="11" height="11" rx="2"/><path d="M15 9V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v7a2 2 0 0 0 2 2h3"/></svg>`
const check = `<svg class="copy" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="m5 12 4 4L19 6"/></svg>`

function parseColor(value) {
  const probe = document.createElement('span')
  probe.style.color = ''
  probe.style.color = value.trim()
  if (!probe.style.color) return null
  document.body.append(probe)
  const normalized = getComputedStyle(probe).color
  probe.remove()
  const parts = normalized.match(/[\d.]+/g)?.map(Number)
  if (!parts || parts.length < 3) return null
  const [r, g, b, alpha = 1] = parts
  const a = Math.round(alpha * 255)
  const hex = `#${[r,g,b,a].slice(0, a === 255 ? 3 : 4).map(channel => Math.round(channel).toString(16).padStart(2,'0')).join('').toUpperCase()}`
  const rgb = a === 255 ? `rgb(${r}, ${g}, ${b})` : `rgba(${r}, ${g}, ${b}, ${alpha.toFixed(2)})`
  const rn=r/255, gn=g/255, bn=b/255, max=Math.max(rn,gn,bn), min=Math.min(rn,gn,bn), d=max-min
  let h=0
  if (d) h = max===rn ? 60*((gn-bn)/d%6) : max===gn ? 60*((bn-rn)/d+2) : 60*((rn-gn)/d+4)
  if (h<0) h+=360
  const l=(max+min)/2, s=d===0 ? 0 : d/(1-Math.abs(2*l-1))
  const hslBase=`hsl(${Math.round(h)}°, ${Math.round(s*100)}%, ${Math.round(l*100)}%`
  const hsl=a===255 ? `${hslBase})` : `hsla(${Math.round(h)}°, ${Math.round(s*100)}%, ${Math.round(l*100)}%, ${alpha.toFixed(2)})`
  return { hex, rgb, hsl, css: `rgba(${r}, ${g}, ${b}, ${alpha})` }
}

const color = parseColor(context.representation.text || '')
const app = document.querySelector('#app')
if (!color) {
  app.innerHTML = '<p class="error">This value is not a supported CSS color.</p>'
  window.ClipsX.ready()
} else {
  const rows = [['HEX',color.hex],['RGB',color.rgb],['HSL',color.hsl]]
  app.innerHTML = `<section class="view" aria-label="Color details">
    <div class="swatch-shell"><div class="swatch" style="background:${escapeHtml(color.css)}"></div></div>
    <div class="identity"><span class="chip" style="background:${escapeHtml(color.css)}"></span><span class="name">${escapeHtml(color.hex)}</span></div>
    <div class="formats"><p class="eyebrow">Formats</p>${rows.map(([label,value]) => `<button class="format" type="button" data-value="${escapeHtml(value)}" aria-label="Copy ${label} ${escapeHtml(value)}"><span class="label">${label}</span><span class="value"><span>${escapeHtml(value)}</span>${icon}</span></button>`).join('')}</div>
  </section>`
  app.querySelectorAll('.format').forEach(button => button.addEventListener('click', async () => {
    try {
      await window.ClipsX.submitText('text/plain', button.dataset.value, 'copy')
      app.querySelectorAll('.format').forEach(row => { row.dataset.copied='false'; row.querySelector('.copy').outerHTML=icon })
      button.dataset.copied='true'; button.querySelector('.copy').outerHTML=check
      setTimeout(() => { button.dataset.copied='false'; button.querySelector('.copy').outerHTML=icon }, 1400)
    } catch (_) { button.dataset.copied='false' }
  }))
  window.ClipsX.ready()
}
