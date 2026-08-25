import { build } from 'esbuild'

await build({
  entryPoints: ['node_modules/mermaid/dist/mermaid.esm.min.mjs'],
  bundle: true,
  minify: true,
  format: 'iife',
  globalName: 'ClipsXMermaidBundle',
  footer: { js: 'window.mermaid=ClipsXMermaidBundle.default||ClipsXMermaidBundle;' },
  target: 'es2020',
  outfile: 'extensions/mermaid-viewer/ui/mermaid.min.js',
})
