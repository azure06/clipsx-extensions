import React, { useCallback, useEffect, useId, useRef, useState } from 'react'
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

function ToolButton({ label, onClick, children }) {
  return <button className="tool-button" type="button" aria-label={label} title={label} onClick={onClick}>{children}</button>
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

function Diagram({ value, embedded = false, onSettled }) {
  const reactId = useId().replace(/[^a-zA-Z0-9]/g, '')
  const target = useRef(null)
  const viewport = useRef(null)
  const drag = useRef(null)
  const pointers = useRef(new Map())
  const pinch = useRef(null)
  const [status, setStatus] = useState('rendering')
  const [scale, setScale] = useState(1)
  const [pan, setPan] = useState({ x: 0, y: 0 })
  const [fit, setFit] = useState(settings['fit-diagram'] !== false)
  const [showSource, setShowSource] = useState(settings['show-source'] === true)

  useEffect(() => {
    let active = true
    setStatus('rendering')
    mermaidReady.then(mermaid => mermaid.render(`clipsx-mermaid-${reactId}`, value))
      .then(({ svg }) => {
        if (!active || !target.current) return
        target.current.innerHTML = svg
        const rendered = target.current.querySelector('svg')
        rendered?.setAttribute('role', 'img')
        rendered?.setAttribute('aria-label', 'Rendered Mermaid diagram')
        setStatus('ready')
        onSettled?.()
      })
      .catch(() => {
        if (!active) return
        setStatus('error')
        setShowSource(true)
        onSettled?.()
      })
    return () => { active = false }
  }, [reactId, value, onSettled])

  const zoom = delta => {
    setFit(false)
    setScale(current => Math.min(2, Math.max(.5, Number((current + delta).toFixed(1)))))
  }

  const zoomAt = useCallback(event => {
    event.preventDefault()
    const bounds = viewport.current?.getBoundingClientRect()
    if (!bounds) return
    const next = Math.min(2, Math.max(.5, Number((scale - event.deltaY * .0015).toFixed(2))))
    if (next === scale) return
    const pointer = { x: event.clientX - bounds.left, y: event.clientY - bounds.top }
    setFit(false)
    setPan(current => ({
      x: pointer.x - (pointer.x - current.x) * (next / scale),
      y: pointer.y - (pointer.y - current.y) * (next / scale),
    }))
    setScale(next)
  }, [scale])

  useEffect(() => {
    const canvas = viewport.current
    if (!canvas) return undefined
    canvas.addEventListener('wheel', zoomAt, { passive: false })
    return () => canvas.removeEventListener('wheel', zoomAt)
  }, [zoomAt])

  const startPan = event => {
    if (event.button !== 0) return
    event.currentTarget.setPointerCapture(event.pointerId)
    event.currentTarget.classList.add('dragging')
    pointers.current.set(event.pointerId, { x: event.clientX, y: event.clientY })
    if (pointers.current.size === 2) {
      const [first, second] = [...pointers.current.values()]
      pinch.current = {
        distance: Math.hypot(second.x - first.x, second.y - first.y),
        scale,
        pan,
        center: { x: (first.x + second.x) / 2, y: (first.y + second.y) / 2 },
      }
      drag.current = null
      setFit(false)
      return
    }
    drag.current = { pointerId: event.pointerId, x: event.clientX, y: event.clientY, pan }
    setFit(false)
  }

  const movePan = event => {
    if (pointers.current.has(event.pointerId)) {
      pointers.current.set(event.pointerId, { x: event.clientX, y: event.clientY })
    }
    if (pointers.current.size === 2 && pinch.current) {
      const [first, second] = [...pointers.current.values()]
      const distance = Math.hypot(second.x - first.x, second.y - first.y)
      if (pinch.current.distance <= 0) return
      const next = Math.min(2, Math.max(.5, pinch.current.scale * distance / pinch.current.distance))
      const bounds = viewport.current?.getBoundingClientRect()
      if (!bounds) return
      const center = {
        x: (first.x + second.x) / 2 - bounds.left,
        y: (first.y + second.y) / 2 - bounds.top,
      }
      const origin = {
        x: pinch.current.center.x - bounds.left,
        y: pinch.current.center.y - bounds.top,
      }
      setScale(next)
      setPan({
        x: center.x - (origin.x - pinch.current.pan.x) * (next / pinch.current.scale),
        y: center.y - (origin.y - pinch.current.pan.y) * (next / pinch.current.scale),
      })
      return
    }
    const origin = drag.current
    if (!origin || origin.pointerId !== event.pointerId) return
    setPan({ x: origin.pan.x + event.clientX - origin.x, y: origin.pan.y + event.clientY - origin.y })
  }

  const stopPan = event => {
    pointers.current.delete(event.pointerId)
    if (pointers.current.size < 2) pinch.current = null
    if (drag.current?.pointerId === event.pointerId) drag.current = null
    if (pointers.current.size === 0) event.currentTarget.classList.remove('dragging')
  }

  return (
    <section className={`diagram-card${embedded ? ' embedded' : ''}`} aria-busy={status === 'rendering'}>
      <header className="diagram-toolbar">
        <div className={`diagram-state ${status}`} role="status">
          {status === 'rendering' ? 'Rendering…' : status === 'error' ? 'Could not render' : ''}
        </div>
        <div className="diagram-tools" role="group" aria-label="Diagram view controls">
          <ToolButton label="Zoom out" onClick={() => zoom(-.1)}><Icon name="minus" /></ToolButton>
          <output aria-live="polite">{Math.round(scale * 100)}%</output>
          <ToolButton label="Zoom in" onClick={() => zoom(.1)}><Icon name="plus" /></ToolButton>
          <ToolButton label="Fit diagram" onClick={() => { setScale(1); setPan({ x: 0, y: 0 }); setFit(true) }}><Icon name="fit" /></ToolButton>
          <ToolButton label={showSource ? 'Hide source' : 'Show source'} onClick={() => setShowSource(current => !current)}><Icon name="code" /></ToolButton>
        </div>
      </header>
      <div
        className="diagram-viewport"
        ref={viewport}
        onPointerDown={startPan}
        onPointerMove={movePan}
        onPointerUp={stopPan}
        onPointerCancel={stopPan}
        aria-label="Diagram canvas. Pinch or use the mouse wheel to zoom, and drag to pan."
      >
        {status === 'error' && <div className="render-error"><strong>This diagram could not be rendered.</strong><span>Check the Mermaid syntax in the source below.</span></div>}
        <div
          className={`diagram${fit ? ' fit' : ''}${status === 'error' ? ' hidden' : ''}`}
          ref={target}
          style={{ '--diagram-scale': scale, '--diagram-x': `${pan.x}px`, '--diagram-y': `${pan.y}px` }}
        />
      </div>
      {showSource && <pre className="diagram-source" aria-label="Mermaid diagram source"><code>{value}</code></pre>}
    </section>
  )
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
        <ToolButton label="Close" onClick={() => clipsx?.close()}><Icon name="close" /></ToolButton>
      </header>}
      {isMarkdown ? <MarkdownDocument /> : <Diagram value={source} />}
    </main>
  )
}

createRoot(document.querySelector('#root')).render(<App />)
