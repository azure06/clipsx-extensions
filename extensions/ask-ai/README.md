# Ask AI

Build and package from the repository root:

```powershell
npm run package -- ask-ai dist/ask-ai-2.0.0-v3.2.clipsx
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

Ask AI remains an interactive action package on v3.2, not a model-backed
transformation. ClipsX selects a matching plain-text representation from the clip,
including when its active preview is HTML Source. ChatGPT and Claude remain
separate pinnable actions inside the one Ask AI Tools entry. External-navigation
consent is required; a local generation model is not required. Prompts are sent
through the destination URL only after the user explicitly runs an action.
