import { existsSync } from 'node:fs'
import { resolve } from 'node:path'
import { spawnSync } from 'node:child_process'

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

