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
mouse/trackpad-wheel zoom, click-drag panning, and keyboard-accessible zoom and
fit controls without visible scrollbars.

The package tool converts the core module to a no-WASI Component Model artifact.
Do not build this example with `wasm32-wasip2`.

The contribution icon is the Mermaid mark published at
<https://static.mermaidchart.dev/assets/mermaid-icon.svg>.
