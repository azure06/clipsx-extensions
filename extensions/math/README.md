# Math

First-party offline mathematical notation for ClipsX. Bounded WASM detection
recognizes explicitly delimited LaTeX and conservative standalone TeX. The
isolated KaTeX view follows the host theme, retains accessible source fallback,
and requires no permissions.

Build from the repository root:

```powershell
cargo build --release --target wasm32-unknown-unknown --manifest-path extensions/math/Cargo.toml
Copy-Item extensions/math/target/wasm32-unknown-unknown/release/clipsx_math.wasm extensions/math/component.wasm
npm run sync:katex
npm run tool -- pack extensions/math dist/math.clipsx
npm run tool -- validate dist/math.clipsx
```

Only WOFF2 fonts are synchronized. The WOFF and TTF fallbacks distributed by
KaTeX are redundant in ClipsX's modern embedded webviews and are deliberately
excluded from source and release archives.
