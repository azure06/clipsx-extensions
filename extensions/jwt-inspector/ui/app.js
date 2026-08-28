const clipsx = window.ClipsX
const context = clipsx.context
const token = (context.representation.text || '').trim()
const app = document.querySelector('#app')

document.documentElement.dataset.theme = context.theme

const escapeHtml = value =>
  String(value).replace(
    /[&<>"']/g,
    character =>
      ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[character]
  )

const decodePart = value =>
  JSON.parse(
    new TextDecoder().decode(
      Uint8Array.from(
        atob(value.replace(/-/g, '+').replace(/_/g, '/').padEnd(Math.ceil(value.length / 4) * 4, '=')),
        character => character.charCodeAt(0)
      )
    )
  )

const displayValue = (key, value) =>
  ['exp', 'iat', 'nbf'].includes(key) && typeof value === 'number'
    ? `${new Date(value * 1000).toLocaleString()} • ${value}`
    : value

const copyIcon = `
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
       stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <rect width="14" height="14" x="8" y="8" rx="2" ry="2"></rect>
    <path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"></path>
  </svg>`

try {
  const [header, payload] = token.split('.')
  const parts = { header: decodePart(header), payload: decodePart(payload) }
  let active = 'payload'
  let raw = context.settings['show-raw'] === true
  let copiedIndex = null

  const render = () => {
    const view = raw ? 'raw' : active
    const value = parts[active]
    const entries = Object.entries(value)
    app.innerHTML = `
      <section class="shell">
        <header class="hero">
          <span class="mark">JWT</span>
          <div>
            <h1>${escapeHtml(parts.header.alg || 'JSON Web Token')}</h1>
            <p>${escapeHtml(parts.header.typ || 'JWT')} • ${Object.keys(parts.payload).length} payload claims</p>
          </div>
        </header>
        <nav class="tabs" role="tablist" aria-label="JWT sections">
          <button type="button" class="tab" role="tab" data-view="header" aria-selected="${view === 'header'}">Header</button>
          <button type="button" class="tab" role="tab" data-view="payload" aria-selected="${view === 'payload'}">Payload</button>
          <button type="button" class="tab" role="tab" data-view="raw" aria-selected="${view === 'raw'}">Raw JSON</button>
        </nav>
        <div class="panel">
          ${
            raw
              ? `<pre class="raw">${escapeHtml(JSON.stringify(value, null, 2))}</pre>`
              : entries
                  .map(([key, item], index) => {
                    const displayed =
                      typeof item === 'object' ? JSON.stringify(item) : displayValue(key, item)
                    return `<div class="row">
                      <span class="key">${escapeHtml(key)}</span>
                      <span class="value">${escapeHtml(displayed)}</span>
                      <button type="button" class="copy" data-copy-index="${index}"
                              aria-label="Copy ${escapeHtml(key)}" title="Copy ${escapeHtml(key)}">
                        ${copiedIndex === index ? '<span class="check" aria-hidden="true">✓</span>' : copyIcon}
                      </button>
                    </div>`
                  })
                  .join('') || '<p class="empty">No claims in this section.</p>'
          }
        </div>
      </section>`

    app.querySelectorAll('[data-view]').forEach(button => {
      button.addEventListener('click', () => {
        const nextView = button.dataset.view
        raw = nextView === 'raw'
        if (!raw) active = nextView
        copiedIndex = null
        render()
      })
    })
    app.querySelectorAll('[data-copy-index]').forEach(button => {
      button.addEventListener('click', async () => {
        const index = Number(button.dataset.copyIndex)
        const item = entries[index]?.[1]
        const text = typeof item === 'object' ? JSON.stringify(item) : String(item)
        button.disabled = true
        try {
          await clipsx.submitText('text/plain', text, 'copy')
          copiedIndex = index
          render()
        } catch (error) {
          button.disabled = false
          button.title = `Copy failed: ${String(error)}`
        }
      })
    })
  }

  render()
  clipsx.ready()
} catch {
  app.innerHTML = '<p class="empty">This JWT could not be decoded.</p>'
  clipsx.ready()
}
