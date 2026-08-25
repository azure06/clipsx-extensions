# Color Tools

This first-party package provides a detector, a custom ClipsX-native detail
view, a host-rendered compact view, and transformer-preset actions. The detail
view mirrors ClipsX's original color presentation and offers focused HEX, RGB,
and HSL copy rows through the host-brokered output boundary.
It recognizes exact CSS HEX, RGB/RGBA, and HSL/HSLA values, including percentage
channels, alpha, and `transparent`, while rejecting ordinary prose.

Build and package from the repository root:

```powershell
rustup target add wasm32-unknown-unknown
cargo build --manifest-path extensions/color-tools/Cargo.toml --target wasm32-unknown-unknown --release
Copy-Item extensions/color-tools/target/wasm32-unknown-unknown/release/clipsx_color_tools.wasm extensions/color-tools/component.wasm
npm run extension:pack -- extensions/color-tools dist/color-tools.clipsx
npm run extension:validate -- dist/color-tools.clipsx
```

Enable Developer Mode in ClipsX and install the resulting `.clipsx` package.
The package tool generates a component that imports no WASI or host APIs.
