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

Published package IDs use the permanent `infiniti.<package>` namespace.
Contribution IDs remain package-local kebab-case identifiers; the host qualifies
them as `<package-id>/<contribution-id>`, while emitted semantic facets use
`<package-id>.<facet-id>`.

## Repository boundary

- `extensions/` contains one independently versioned package per directory.
- `sdk/wit/` is the pinned Extension API v2 contract used by the Rust guests.
- Generated `target/`, `component.wasm`, `.clipsx`, and `dist/` outputs are ignored.
- Published `.clipsx` archives belong in checksum-pinned GitHub Releases, not
  Git; repository immutability is mandatory for every new release.
- Catalog metadata and catalog icons belong in `azure06/clipsx-registry`.

The package UI is fully local and offline. Runtime assets required by a package,
including Mermaid and KaTeX WOFF2 fonts, are intentionally included in that
package's release archive rather than the ClipsX application bundle.

Merging to `main` makes a package eligible for release but does not publish it.
When a release is wanted, run **Publish extension release** and select the
package. The workflow reads the version from that package's manifest, rebuilds
and validates it, rejects an existing tag, and publishes the immutable release.
The initial five releases predate repository immutability and remain protected
by their signed catalog checksums.

Publishing package bytes does not add them to the trusted catalog. Update the
separate registry metadata through its normal pull request and signing flow only
after reviewing the release. This keeps the everyday release operation simple
without giving an extension-source workflow access to the catalog signing key.

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
