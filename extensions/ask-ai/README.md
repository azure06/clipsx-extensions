# Ask AI

Build and package from the repository root:

```powershell
cargo build --release --target wasm32-unknown-unknown --manifest-path extensions/ask-ai/Cargo.toml
Copy-Item extensions/ask-ai/target/wasm32-unknown-unknown/release/clipsx_ask_ai.wasm extensions/ask-ai/component.wasm
npm run extension:pack -- extensions/ask-ai dist/ask-ai.clipsx
npm run extension:validate -- dist/ask-ai.clipsx
```

The package opens only its declared ChatGPT and Claude origins. Each action has
a 2 KiB input ceiling matching the destination URL boundary. URL prompts are
UTF-8 percent encoded and the WASM `action-state` export exits early and
disables prompts whose encoded URL would exceed that boundary.

The toolbar uses supplied provider marks: OpenAI Blossom black/white variants
from the OpenAI logo bundle and Anthropic's rounded Claude icon from its media
resources. The Blossom is used only to identify the button that opens ChatGPT;
it is not ClipsX branding.

The package tool converts the core module to a no-WASI Component Model artifact.
Do not build this example with `wasm32-wasip2`.
