# ClipsX Extensions

First-party extension sources for [ClipsX](https://github.com/azure06/clipsx).
Extensions are optional and none are installed with the application by default.

## First-party packages

| Package | Purpose |
| --- | --- |
| Mermaid Viewer | Mermaid diagrams and fenced diagrams in enhanced Markdown |
| JWT Inspector | Unverified local JWT header and claim inspection |
| Base64 | Local UTF-8/binary-aware Base64 encoding and decoding |
| Data Tools | Tables, JSON/YAML/TOML, TypeScript shapes, and URL conversion |

## Repository boundary

- `extensions/` contains one independently versioned package per directory.
- `sdk/wit/` is the pinned Extension API v2 contract used by the Rust guests.
- Generated `target/`, `component.wasm`, `.clipsx`, and `dist/` outputs are ignored.
- Published `.clipsx` archives belong in immutable GitHub Releases, not Git.
- Catalog metadata and catalog icons belong in `azure06/clipsx-registry`.

The package UI is fully local and offline. Runtime assets required by a package,
including Mermaid and KaTeX WOFF2 fonts, are intentionally included in that
package's release archive rather than the ClipsX application bundle.

## Local development

Install JavaScript build dependencies:

```powershell
npm install
```

By default the tool wrapper uses a sibling `../clipsx` checkout. Set
`CLIPSX_REPO` when the host repository is elsewhere.

```powershell
npm run tool -- pack extensions/<package> dist/<package>.clipsx
npm run tool -- validate dist/<package>.clipsx
```

Rust guests target `wasm32-unknown-unknown`; the package tool componentizes
that core module without ambient WASI imports. Copy the release WASM to the
package root as `component.wasm` before packing; it remains an ignored build
artifact.
