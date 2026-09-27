# Data Tools

Offline conversions that reuse ClipsX's native previews instead of shipping a second UI.

- Convert JSON arrays, CSV, TSV, and strict GFM Markdown tables.
- Convert JSON to YAML, TOML, or a configurable TypeScript type.
- Convert typed YAML and TOML representations back to JSON.
- Encode, decode, normalize, or extract query data from URLs.

Tools shows one Data Tools entry with content-sensitive conversion setups. Output MIME types are preserved so JSON opens in the JSON tree, tables in the table preview, Markdown in the Markdown renderer, and TypeScript in the code preview. Input is limited to 10 MiB; output uses the host's bounded durable result storage. The TypeScript setup alone exposes the Root type name field.
