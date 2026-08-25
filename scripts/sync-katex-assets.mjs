import { cp, mkdir, readFile, readdir, rm, writeFile } from 'node:fs/promises'
import { resolve } from 'node:path'

const source = resolve('node_modules/katex/dist')
const target = resolve('extensions/math/ui')
const fonts = resolve(target, 'fonts')

await mkdir(fonts, { recursive: true })
await cp(resolve(source, 'katex.min.js'), resolve(target, 'katex.min.js'))

let css = await readFile(resolve(source, 'katex.min.css'), 'utf8')
css = css.replace(/,url\(fonts\/[^)]*\.woff\) format\("woff"\),url\(fonts\/[^)]*\.ttf\) format\("truetype"\)/g, '')
await writeFile(resolve(target, 'katex.min.css'), css)

for (const name of await readdir(fonts)) await rm(resolve(fonts, name))
for (const name of await readdir(resolve(source, 'fonts'))) {
  if (name.endsWith('.woff2')) await cp(resolve(source, 'fonts', name), resolve(fonts, name))
}
