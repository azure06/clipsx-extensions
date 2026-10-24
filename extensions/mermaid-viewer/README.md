# Mermaid

Build and package from the repository root:

```powershell
npm run build:mermaid
cargo build --release --target wasm32-unknown-unknown --manifest-path extensions/mermaid-viewer/Cargo.toml
Copy-Item extensions/mermaid-viewer/target/wasm32-unknown-unknown/release/clipsx_mermaid_viewer.wasm extensions/mermaid-viewer/component.wasm
npm run tool -- pack extensions/mermaid-viewer dist/mermaid-viewer.clipsx
npm run tool -- validate dist/mermaid-viewer.clipsx
```

Detection runs in WASM. Standalone Mermaid and Mermaid fences inside Markdown
receive specific package facets. The bundled offline React UI uses ClipsX's
React Markdown and GFM stack, replacing only Mermaid code fences with diagrams.
Without the package, core Markdown still displays those fences as code.
Unsupported syntax retains an accessible source fallback. Theme and locale
come from the host context; user preferences are manifest-declared, validated,
and stored by ClipsX rather than in browser storage. Diagram canvases support
pointer-anchored pinch or Ctrl/Command+wheel zoom, drag panning, and Fit/100%
controls with an actual scale display. Native WebKit trackpad pinch needs no
keyboard modifier; Chromium-style Ctrl+wheel pinch remains supported. Windows
precision-trackpad pinch requires the updated host's native WebView2 gesture
configuration, including a rebuilt/restarted app; UI hot reload alone is insufficient. Overlapping
gesture and wheel events do not apply zoom twice. Manual zoom ranges from 10–800%; Fit may
use smaller scales and never enlarges small diagrams. Focus the canvas for +/−,
arrow-key panning, 0 to fit and 1 for 100%. Fit follows viewport resizing; manual
views keep their inspected center. The normal cursor is used while the full
diagram fits; the grab hand appears only when zoomed content can be panned. Ordinary wheel input pans an overflowing
standalone diagram and scrolls the document around embedded diagrams. Host
expansion preserves the current custom-view session and closes with Escape.
Controls and hints follow the host's English/Japanese locale. Source fallback
remains available; navigation never edits diagram text or clipboard output.

The package tool converts the core module to a no-WASI Component Model artifact.
Do not build this example with `wasm32-wasip2`.

The contribution icon is the Mermaid mark published at
<https://static.mermaidchart.dev/assets/mermaid-icon.svg>.
