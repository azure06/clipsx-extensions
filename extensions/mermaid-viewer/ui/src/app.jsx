import { attachTrackpadGestures } from './trackpad.mjs'
import { MIN_ZOOM, MAX_ZOOM, canPan, emptyTransform, fitTransform, boundTransform, zoomTransform, resizeTransform } from './viewport.mjs'
import React, { useEffect, useId, useRef, useState } from 'react'
import { createRoot } from 'react-dom/client'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'

const clipsx = window.ClipsX
const context = clipsx?.context ?? {}
const source = context.representation?.text ?? ''
const settings = context.settings ?? {}
const theme = context.theme === 'dark' ? 'dark' : 'light'
const isMarkdown = context.facetId?.endsWith('.markdown-mermaid') ?? false

document.documentElement.dataset.theme = theme
document.documentElement.lang = context.locale || 'en'

const iconPaths = {
  minus: <path d="M5 12h14" />,
  plus: <><path d="M12 5v14" /><path d="M5 12h14" /></>,
  fit: <><path d="M8 3H5a2 2 0 0 0-2 2v3" /><path d="M16 3h3a2 2 0 0 1 2 2v3" /><path d="M8 21H5a2 2 0 0 1-2-2v-3" /><path d="M16 21h3a2 2 0 0 0 2-2v-3" /></>,
  close: <><path d="m18 6-12 12" /><path d="m6 6 12 12" /></>,
  code: <><path d="m8 9-4 3 4 3" /><path d="m16 9 4 3-4 3" /><path d="m14 5-4 14" /></>,
}

function Icon({ name }) {
  return <svg aria-hidden="true" viewBox="0 0 24 24">{iconPaths[name]}</svg>
}

function ToolButton({ label, onClick, children, disabled = false }) {
  return <button disabled={disabled} className="tool-button" type="button" aria-label={label} title={label} onClick={onClick}>{children}</button>
}

const mermaidTheme = theme === 'dark'
  ? { background: '#172033', primaryColor: '#252e46', primaryTextColor: '#e2e8f0', primaryBorderColor: '#8b5cf6', lineColor: '#a78bfa', secondaryColor: '#312e55', tertiaryColor: '#1e293b', textColor: '#e2e8f0' }
  : { background: '#ffffff', primaryColor: '#f5f3ff', primaryTextColor: '#1e293b', primaryBorderColor: '#7c3aed', lineColor: '#7c3aed', secondaryColor: '#ede9fe', tertiaryColor: '#f8fafc', textColor: '#1e293b' }

const mermaidReady = new Promise((resolve, reject) => {
  const initialize = () => {
    if (!window.mermaid) {
      reject(new Error('The bundled Mermaid renderer could not start.'))
      return
    }
    window.mermaid.initialize({ startOnLoad: false, securityLevel: 'strict', htmlLabels: false, suppressErrorRendering: true, maxTextSize: 100000, theme: theme === 'dark' ? 'dark' : 'default', themeVariables: mermaidTheme })
    resolve(window.mermaid)
  }
  if (window.mermaid) {
    initialize()
    return
  }
  const runtime = document.createElement('script')
  runtime.src = 'mermaid.min.js'
  runtime.addEventListener('load', initialize, { once: true })
  runtime.addEventListener('error', () => reject(new Error('The bundled Mermaid renderer could not load.')), { once: true })
  document.head.append(runtime)
})

const labels = context.locale?.startsWith('ja') ? {
  rendering: '描画中…', failed: '描画できませんでした', controls: '図の表示操作', out: '縮小', zoom: '拡大率', in: '拡大', fit: '全体を表示', actual: '実際のサイズ（100%）', hide: 'ソースを隠す', show: 'ソースを表示', canvas: '図。ドラッグで移動、ピンチまたは Ctrl／Command＋ホイールで拡大・縮小。', error: 'この図を描画できませんでした。', check: '下のソースで Mermaid の構文を確認してください。', source: 'Mermaid のソース', diagram: 'Mermaid の図', hint: 'ドラッグで移動 · ピンチまたは Ctrl/⌘＋ホイールで拡大・縮小 · 0で全体表示 · 1で100%', close: '閉じる'
} : {
  rendering: 'Rendering…', failed: 'Could not render', controls: 'Diagram view controls', out: 'Zoom out', zoom: 'Zoom level', in: 'Zoom in', fit: 'Fit', actual: 'Actual size (100%)', hide: 'Hide source', show: 'Show source', canvas: 'Diagram canvas. Drag to pan; pinch or Ctrl/Command + wheel to zoom.', error: 'This diagram could not be rendered.', check: 'Check the Mermaid syntax in the source below.', source: 'Mermaid diagram source', diagram: 'Rendered Mermaid diagram', hint: 'Drag to pan · Pinch or Ctrl/⌘ + wheel to zoom · 0 to fit · 1 for 100%', close: 'Close'
}
function Diagram({ value, embedded = false, onSettled }) {
  const reactId = useId().replace(/[^a-zA-Z0-9]/g, '')
  const target = useRef(null), viewport = useRef(null)
  const pointers = useRef(new Map())
  const dimensions = useRef({ width: 0, height: 0 })
  const [content, setContent] = useState({ width: 0, height: 0 })
  const [status, setStatus] = useState('rendering')
  const [transform, setTransform] = useState({ ...emptyTransform, fit: settings['fit-diagram'] !== false })
  const current = useRef(transform)
  current.current = transform
  const [showSource, setShowSource] = useState(settings['show-source'] === true)
  const [dragging, setDragging] = useState(false)
  useEffect(() => {
    let active = true
    setStatus('rendering')
    setTransform({ ...emptyTransform, fit: settings['fit-diagram'] !== false })
    mermaidReady.then(mermaid => mermaid.render(`clipsx-mermaid-${reactId}`, value)).then(({ svg }) => {
      if (!active || !target.current) return
      target.current.innerHTML = svg
      const rendered = target.current.querySelector('svg')
      const box = rendered?.viewBox?.baseVal
      if (!rendered || !box || !Number.isFinite(box.width) || !Number.isFinite(box.height) || box.width <= 0 || box.height <= 0) throw Error('Invalid diagram dimensions')
      rendered.setAttribute('role', 'img'); rendered.setAttribute('aria-label', labels.diagram)
      rendered.style.width = `${box.width}px`; rendered.style.height = `${box.height}px`; rendered.style.maxWidth = 'none'
      const size = { width: box.width, height: box.height }
      setContent(size)
      if (settings['fit-diagram'] === false) setTransform(boundTransform({ scale: 1, fit: false, x: (dimensions.current.width - size.width) / 2, y: (dimensions.current.height - size.height) / 2 }, size, dimensions.current))
      setStatus('ready'); onSettled?.()
    }).catch(() => { if (active) { setStatus('error'); setShowSource(true); onSettled?.() } })
    return () => { active = false }
  }, [reactId, value, onSettled])
  useEffect(() => {
    const element = viewport.current
    if (!element) return
    const resize = () => {
      const bounds = element.getBoundingClientRect(), before = dimensions.current
      const after = { width: bounds.width, height: bounds.height }
      dimensions.current = after
      setTransform(previous => resizeTransform(previous, content, before, after))
    }
    const observer = new ResizeObserver(resize)
    observer.observe(element); resize()
    return () => observer.disconnect()
  }, [content])
  const center = () => ({ x: dimensions.current.width / 2, y: dimensions.current.height / 2 })
  const zoom = (scale, point = center()) => setTransform(previous => zoomTransform(previous, scale, point, content, dimensions.current))
  const pan = (x, y) => {
    if (!canPan(content, dimensions.current, current.current.scale)) return
    setTransform(previous => boundTransform({ ...previous, fit: false, x: previous.x + x, y: previous.y + y }, content, dimensions.current))
  }
  const fit = () => setTransform(fitTransform(content, dimensions.current))
  useEffect(() => {
    const element = viewport.current
    if (!element || status !== 'ready') return
    const gestures = attachTrackpadGestures(element, () => current.current, setTransform, content, () => dimensions.current, () => { pointers.current.clear(); setDragging(false) })
    const wheel = event => {
      if (gestures.active()) { event.preventDefault(); return }
      const bounds = element.getBoundingClientRect(), previous = current.current
      if (event.ctrlKey || event.metaKey) {
        event.preventDefault()
        setTransform(zoomTransform(previous, previous.scale * Math.exp(-event.deltaY * .005), { x: event.clientX - bounds.left, y: event.clientY - bounds.top }, content, dimensions.current))
      } else if (!embedded && (content.width * previous.scale > bounds.width || content.height * previous.scale > bounds.height)) {
        event.preventDefault()
        const unit = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? bounds.height : 1
        setTransform(boundTransform({ ...previous, fit: false, x: previous.x - event.deltaX * unit, y: previous.y - event.deltaY * unit }, content, dimensions.current))
      }
    }
    element.addEventListener('wheel', wheel, { passive: false })
    return () => { element.removeEventListener('wheel', wheel); gestures.dispose() }
  }, [content, embedded, status])
  const stop = event => { pointers.current.delete(event.pointerId); setDragging(pointers.current.size > 0) }
  const ready = status === 'ready'
  const pannable = ready && canPan(content, dimensions.current, transform.scale)
  return <section className={`diagram-card${embedded ? ' embedded' : ''}`} aria-busy={status === 'rendering'}>
    <header className="diagram-toolbar">
      <div className={`diagram-state ${status}`} role="status">{status === 'rendering' ? labels.rendering : status === 'error' ? labels.failed : ''}</div>
      <div className="diagram-tools" role="group" aria-label={labels.controls}>
        <ToolButton label={labels.out} disabled={!ready || transform.scale <= MIN_ZOOM} onClick={() => zoom(transform.scale / 1.25)}><Icon name="minus" /></ToolButton>
        <output aria-label={labels.zoom}>{ready ? `${Math.round(transform.scale * 1000) / 10}%` : '—'}</output>
        <ToolButton label={labels.in} disabled={!ready || transform.scale >= MAX_ZOOM} onClick={() => zoom(transform.scale * 1.25)}><Icon name="plus" /></ToolButton>
        <ToolButton label={labels.fit} disabled={!ready} onClick={fit}><Icon name="fit" /></ToolButton>
        <ToolButton label={labels.actual} disabled={!ready} onClick={() => zoom(1)}>100%</ToolButton>
        <ToolButton label={showSource ? labels.hide : labels.show} onClick={() => setShowSource(previous => !previous)}><Icon name="code" /></ToolButton>
      </div>
    </header>
    <div className={`diagram-viewport${pannable ? ' pannable' : ''}${pannable && dragging ? ' dragging' : ''}`} ref={viewport} tabIndex={0} role="region" aria-label={labels.canvas}
      onKeyDown={event => {
        if (event.target !== event.currentTarget || !ready || event.ctrlKey || event.metaKey || event.altKey) return
        const actions = { '+': () => zoom(transform.scale * 1.25), '=': () => zoom(transform.scale * 1.25), '-': () => zoom(transform.scale / 1.25), '0': fit, '1': () => zoom(1), ArrowLeft: () => pan(40, 0), ArrowRight: () => pan(-40, 0), ArrowUp: () => pan(0, 40), ArrowDown: () => pan(0, -40) }
        if (actions[event.key]) { event.preventDefault(); event.stopPropagation(); actions[event.key]() }
      }}
      onPointerDown={event => { if (!ready || event.button !== 0 || (!pannable && event.pointerType !== 'touch')) return; event.preventDefault(); event.currentTarget.focus(); event.currentTarget.setPointerCapture(event.pointerId); pointers.current.set(event.pointerId, { x: event.clientX, y: event.clientY }); setDragging(true) }}
      onPointerMove={event => {
        const previous = pointers.current.get(event.pointerId)
        if (!previous) return
        const next = { x: event.clientX, y: event.clientY }, other = [...pointers.current.entries()].find(([id]) => id !== event.pointerId)?.[1]
        pointers.current.set(event.pointerId, next)
        if (!other) { if (pannable) pan(next.x - previous.x, next.y - previous.y); return }
        const distance = Math.hypot(previous.x - other.x, previous.y - other.y)
        if (distance < 1) return
        const ratio = Math.hypot(next.x - other.x, next.y - other.y) / distance, rect = event.currentTarget.getBoundingClientRect()
        const origin = { x: (previous.x + other.x) / 2 - rect.left, y: (previous.y + other.y) / 2 - rect.top }
        setTransform(value => { const result = zoomTransform(value, value.scale * ratio, origin, content, dimensions.current); return boundTransform({ ...result, x: result.x + (next.x - previous.x) / 2, y: result.y + (next.y - previous.y) / 2 }, content, dimensions.current) })
      }}
      onPointerUp={stop} onPointerCancel={stop} onLostPointerCapture={stop}>
      {status === 'error' && <div className="render-error"><strong>{labels.error}</strong><span>{labels.check}</span></div>}
      <div className={`diagram${status === 'error' ? ' hidden' : ''}`} ref={target} style={{ width: content.width, height: content.height, transform: `translate(${transform.x}px, ${transform.y}px) scale(${transform.scale})` }} />
    </div>
    <p className="diagram-hint">{labels.hint}</p>
    {showSource && <pre className="diagram-source" aria-label={labels.source}><code>{value}</code></pre>}
  </section>
}

function MarkdownDocument() {
  return (
    <article className="markdown-document">
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          a: props => <a {...props} target="_blank" rel="noreferrer" />,
          pre: ({ children }) => {
            const child = React.Children.only(children)
            const language = /language-([^\s]+)/.exec(child.props.className ?? '')?.[1]?.toLowerCase()
            if (language === 'mermaid') return <Diagram embedded value={String(child.props.children).replace(/\n$/, '')} />
            return <pre>{children}</pre>
          },
        }}
      >{source}</ReactMarkdown>
    </article>
  )
}

function App() {
  useEffect(() => { clipsx?.ready() }, [])
  return (
    <main className={`viewer${isMarkdown ? ' document-view' : ''}${context.surface === 'dialog' ? ' dialog-view' : ''}`}>
      {context.surface === 'dialog' && <header className="app-bar">
        <div className="identity"><span className="mark" aria-hidden="true">M</span><div><strong>{isMarkdown ? 'Markdown + Mermaid' : 'Mermaid'}</strong><span>Private, offline preview</span></div></div>
        <ToolButton label={labels.close} onClick={() => clipsx?.close()}><Icon name="close" /></ToolButton>
      </header>}
      {isMarkdown ? <MarkdownDocument /> : <Diagram value={source} />}
    </main>
  )
}

createRoot(document.querySelector('#root')).render(<App />)
