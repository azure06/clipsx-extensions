import { existsSync } from 'node:fs'
import { resolve } from 'node:path'
import { spawnSync } from 'node:child_process'

// CI shares one pinned validator binary across package jobs; local development
// retains the sibling-checkout fallback.
if (process.env.CLIPSX_EXTENSION_TOOL) {
  const result = spawnSync(resolve(process.env.CLIPSX_EXTENSION_TOOL), process.argv.slice(2), { stdio: 'inherit' })
  if (result.error) throw result.error
  process.exit(result.status ?? 1)
}

const host = resolve(process.env.CLIPSX_REPO || '../clipsx')
const manifest = resolve(host, 'src-tauri/Cargo.toml')
if (!existsSync(manifest)) {
  console.error('ClipsX host checkout not found. Set CLIPSX_REPO to its absolute path.')
  process.exit(1)
}

const result = spawnSync(
  'cargo',
  ['run', '--quiet', '--manifest-path', manifest, '--bin', 'clipsx-extension-tool', '--', ...process.argv.slice(2)],
  { stdio: 'inherit', shell: process.platform === 'win32' }
)
process.exit(result.status ?? 1)
