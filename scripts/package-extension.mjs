import { copyFileSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { createHash } from 'node:crypto'
import { basename, resolve } from 'node:path'
import { spawnSync } from 'node:child_process'

const supportedPackages = new Set(['ask-ai', 'base64', 'data-tools', 'jwt-inspector', 'mermaid-viewer'])
const [packageSlug, outputArgument, releaseUrl] = process.argv.slice(2)

if (!supportedPackages.has(packageSlug) || !outputArgument) {
  console.error(
    'Usage: npm run package -- <ask-ai|base64|data-tools|jwt-inspector|mermaid-viewer> <output.clipsx> [release-url]'
  )
  process.exit(2)
}

const root = resolve(import.meta.dirname, '..')
const packageRoot = resolve(root, 'extensions', packageSlug)
const toolScript = resolve(root, 'scripts', 'clipsx-extension-tool.mjs')
const manifestPath = resolve(packageRoot, 'clipsx-extension.toml')
const cargoManifest = resolve(packageRoot, 'Cargo.toml')
const output = resolve(root, outputArgument)
const repeatOutput = `${output}.repeat.clipsx`
const manifest = readFileSync(manifestPath, 'utf8')
const cargo = readFileSync(cargoManifest, 'utf8')
const packageId = manifest.match(/^packageId\s*=\s*"([^"]+)"/m)?.[1]
const version = manifest.match(/^version\s*=\s*"([^"]+)"/m)?.[1]
const crateName = cargo.match(/^name\s*=\s*"([^"]+)"/m)?.[1]?.replaceAll('-', '_')

if (!packageId?.startsWith('infiniti.') || !version || !crateName) {
  throw new Error(`Invalid Infiniti package metadata in ${packageSlug}`)
}

const executable = command => command
const run = (command, args, options = {}) => {
  const result = spawnSync(executable(command), args, {
    cwd: root,
    env: process.env,
    encoding: 'utf8',
    stdio: options.capture ? 'pipe' : 'inherit',
    shell: process.platform === 'win32' && command === 'npm',
  })
  if (result.error) throw result.error
  if (result.status !== 0) {
    if (options.capture) process.stderr.write(result.stderr || result.stdout || '')
    process.exit(result.status ?? 1)
  }
  return result.stdout
}

if (packageSlug === 'mermaid-viewer') run('npm', ['run', 'build:mermaid'])
run('cargo', ['test', '--manifest-path', cargoManifest, '--locked'])
run('cargo', [
  'build',
  '--manifest-path',
  cargoManifest,
  '--locked',
  '--release',
  '--target',
  'wasm32-unknown-unknown',
])

copyFileSync(
  resolve(packageRoot, 'target', 'wasm32-unknown-unknown', 'release', `${crateName}.wasm`),
  resolve(packageRoot, 'component.wasm')
)

run(process.execPath, [toolScript, 'pack', packageRoot, output])
run(process.execPath, [toolScript, 'pack', packageRoot, repeatOutput])

const digest = path => createHash('sha256').update(readFileSync(path)).digest('hex')
if (digest(output) !== digest(repeatOutput)) {
  throw new Error(`${packageId} did not produce a deterministic archive`)
}
rmSync(repeatOutput)

for (const command of ['validate', 'inspect', 'test']) {
  run(process.execPath, [toolScript, command, output])
}

if (releaseUrl) {
  const metadata = run(process.execPath, [toolScript, 'registry-entry', output, releaseUrl], {
    capture: true,
  })
  const jsonStart = metadata.indexOf('{')
  if (jsonStart < 0) throw new Error('Registry metadata was not emitted')
  writeFileSync(
    resolve(output.replace(/\.clipsx$/, '.registry.json')),
    `${JSON.stringify(JSON.parse(metadata.slice(jsonStart)), null, 2)}\n`
  )
}

console.log(`${packageId} ${version}: ${basename(output)} sha256=${digest(output)}`)
