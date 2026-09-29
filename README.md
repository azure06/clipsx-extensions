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
| Ask AI | Open selected text in ChatGPT or Claude with explicit navigation consent |
| Rewrite | Local-model rewriting with presets and durable per-clip results |

Published package IDs use the permanent `infiniti.<package>` namespace.
Contribution IDs remain package-local kebab-case identifiers; the host qualifies
them as `<package-id>/<contribution-id>`, while emitted semantic facets use
`<package-id>.<facet-id>`.

## Repository boundary

- `extensions/` contains one independently versioned package per directory.
- `sdk/wit/` is the pinned Extension API v3.2 contract used by the Rust guests.
- Generated `target/`, `component.wasm`, `.clipsx`, and `dist/` outputs are ignored.
- Published `.clipsx` archives belong in checksum-pinned GitHub Releases, not
  Git; repository immutability is mandatory for every new release.
- Catalog metadata and catalog icons belong in `azure06/clipsx-registry`.

The package UI is fully local and offline. Runtime assets required by a package,
including Mermaid and KaTeX WOFF2 fonts, are intentionally included in that
package's release archive rather than the ClipsX application bundle.

## Release flow

```text
Extension PR → test/build affected packages once → candidates + evidence
             → merge → publish the exact checked archives
             → automatically open/update the registry metadata PR
Registry PR  → validate archives + prepare/sign catalog in that same PR
             → merge → Discover reads the new catalog
             → portable-setting approvals reconcile automatically
```

A versioned merge is publication approval. Documentation-only changes publish
nothing. There is no routine manual dispatch or second release build. Releases
and catalog approval remain separate human merges; the source repository never
receives the catalog signing key.

See [RELEASE.md](RELEASE.md) for required checks, one-time GitHub App setup and
recovery, and the [release skill](.agents/skills/clipsx-extension-release/SKILL.md)
for agent workflows.

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

For a complete local archive build, tests, deterministic packaging and conformance:

```powershell
npm run package -- <package> dist/<package>-2.0.0-v3.2.clipsx
```

Import the archive through ClipsX Developer Mode. A local build is not installed
platform certification or a published release.
