(() => {
  'use strict'
  const clipsx = window.ClipsX
  const context = clipsx?.context ?? {}
  const raw = context.representation?.text?.trim() ?? ''
  const settings = context.settings ?? {}
  const unwrap = value => {
    for (const [open, close] of [['$$', '$$'], ['\\[', '\\]'], ['\\(', '\\)']]) {
      if (value.startsWith(open) && value.endsWith(close)) return value.slice(open.length, -close.length).trim()
    }
    return value
  }
  const formula = unwrap(raw)
  const source = document.querySelector('#source')
  const details = document.querySelector('#source-details')
  const output = document.querySelector('#formula')
  const canvas = document.querySelector('#canvas')
  const error = document.querySelector('#error')
  document.documentElement.dataset.theme = context.theme === 'dark' ? 'dark' : 'light'
  document.documentElement.lang = context.locale || 'en'
  source.textContent = formula
  details.open = settings['show-source'] === true
  if (context.surface === 'dialog') {
    document.querySelector('#dialog-header').hidden = false
    document.querySelector('#close').addEventListener('click', () => clipsx?.close())
  }
  clipsx?.ready()
  const runtime = document.createElement('script')
  runtime.src = 'katex.min.js'
  runtime.addEventListener('load', () => {
    try {
      window.katex.render(formula, output, { displayMode: settings['display-mode'] !== false, throwOnError: true, strict: 'warn', trust: false, maxSize: 20, maxExpand: 1000 })
    } catch {
      output.hidden = true
      error.hidden = false
      details.open = true
    } finally { canvas.setAttribute('aria-busy', 'false') }
  }, { once: true })
  runtime.addEventListener('error', () => { output.hidden = true; error.hidden = false; details.open = true; canvas.setAttribute('aria-busy', 'false') }, { once: true })
  document.head.append(runtime)
})()
