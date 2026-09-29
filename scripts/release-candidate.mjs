import { createHash } from 'node:crypto'
import { readFileSync, writeFileSync, mkdirSync, readdirSync, lstatSync } from 'node:fs'
import { resolve } from 'node:path'
import { execFileSync } from 'node:child_process'
import { pathToFileURL } from 'node:url'

export const packages = ['ask-ai', 'base64', 'data-tools', 'jwt-inspector', 'mermaid-viewer', 'rewrite']
export const digest = bytes => createHash('sha256').update(bytes).digest('hex')
const git = (...args) => execFileSync('git', args, { encoding: 'utf8' }).trim()
export const identity = text => {
  const field = name => text.match(new RegExp(`^${name}\\s*=\\s*"([^"\\n]+)"`, 'm'))?.[1]
  const packageId = field('packageId'), version = field('version')
  if (!/^infiniti\.[a-z0-9-]+$/.test(packageId) || !/^\d+\.\d+\.\d+$/.test(version)) throw Error('Invalid release identity')
  const slug = packageId.slice('infiniti.'.length)
  return { packageId, version, tag: `${slug}-v${version}`, asset: `${slug}-${version}.clipsx` }
}

export function affectedPackages(paths) {
  if (paths.some(p => p.startsWith('sdk/') || p === 'package-lock.json' || p === 'package.json' || /^scripts\/(package-extension|build-mermaid-runtime|sync-katex-assets|clipsx-extension-tool)\.mjs$/.test(p) || p === '.github/extension-tool-ref')) return [...packages]
  return packages.filter(p => paths.some(path => path.startsWith(`extensions/${p}/`)))
}

export const changesPackageBytes = path => path.startsWith('sdk/') || path === 'package-lock.json' || path === 'package.json' || /^scripts\/(package-extension|build-mermaid-runtime|sync-katex-assets)\.mjs$/.test(path)

// Published releases are immutable. Only an interrupted draft may receive a
// missing asset; an existing asset must already match the checked bytes.
export function verifyReleaseAssets(release, expected, allowDraft = false) {
  if (release.prerelease || (release.draft ? !allowDraft : !release.immutable)) throw Error('Release is not an immutable publication or recoverable draft')
  const missing = []
  for (const item of expected) {
    const matches = (release.assets || []).filter(asset => asset.name === item.name)
    if (!matches.length && release.draft && allowDraft) { missing.push(item.name); continue }
    const asset = matches[0]
    if (matches.length !== 1 || asset.state !== 'uploaded' || asset.digest !== `sha256:${item.hash}` || asset.size !== item.size) throw Error(`Published ${item.name} conflicts with the validated candidate; never replace it`)
  }
  return missing
}

export function isPreparationRun(run, pr, repository) {
  // GitHub may empty pull_requests after merge. Bind using the run's head and
  // explicit PR title, then verify the artifact's PR/tree evidence separately.
  if (run.head_repository?.full_name !== repository || run.conclusion !== 'success' || run.display_title !== `Prepare extension release (PR ${pr.number})`) return false
  return (run.event === 'pull_request' && run.head_sha === pr.head.sha && run.head_branch === pr.head.ref) || (run.event === 'workflow_dispatch' && run.head_branch === 'main')
}

export function candidateArtifact(artifacts, pkg) {
  const matches = artifacts.filter(artifact => artifact.name === `candidate-${pkg}` && !artifact.expired)
  if (matches.length !== 1) throw Error(`Missing, expired or ambiguous candidate-${pkg}; recover preparation before publishing`)
  return matches[0]
}

export function readCandidate(directory, id) {
  const metadataName = id.asset.replace('.clipsx', '.registry.json')
  const limits = { [id.asset]: 16 * 1024 * 1024, [metadataName]: 64 * 1024, 'evidence.json': 16 * 1024 }
  const files = readdirSync(directory).sort()
  if (JSON.stringify(files) !== JSON.stringify(Object.keys(limits).sort())) throw Error('Candidate artifact contains unexpected files')
  for (const file of files) {
    const info = lstatSync(resolve(directory, file))
    if (!info.isFile() || info.size > limits[file]) throw Error('Candidate file is not a bounded regular file')
  }
  const metadataBytes = readFileSync(resolve(directory, metadataName))
  const metadata = JSON.parse(metadataBytes)
  if (!metadataBytes.equals(Buffer.from(JSON.stringify(metadata) + '\n'))) throw Error('Candidate metadata bytes are not canonical')
  return { metadataName, metadata, evidence: JSON.parse(readFileSync(resolve(directory, 'evidence.json'))), archive: readFileSync(resolve(directory, id.asset)) }
}

export function assertVersionIncreased(before, after) {
  const a = before.split('.').map(Number), b = after.split('.').map(Number)
  const index = a.findIndex((n, i) => n !== b[i])
  if (index < 0 || b[index] <= a[index]) throw Error('Changed package contents require a higher version')
}

export function verifyCandidate(evidence, metadata, archive, expected) {
  if (evidence.schemaVersion !== 1 || evidence.repository !== expected.repository || evidence.prNumber !== expected.prNumber || evidence.headSha !== expected.headSha || evidence.sourceTree !== expected.sourceTree || evidence.toolRef !== expected.toolRef || evidence.runId !== expected.runId) throw Error('Candidate provenance does not match the approved merge')
  if (!packages.includes(evidence.package) || metadata.packageId !== evidence.packageId || metadata.version !== evidence.version || metadata.sha256 !== digest(archive) || metadata.archiveSizeBytes !== archive.length || evidence.archiveSha256 !== metadata.sha256 || evidence.metadataSha256 !== digest(Buffer.from(JSON.stringify(metadata) + '\n'))) throw Error('Candidate bytes or identity do not match evidence')
  const id = identity(`packageId = "${metadata.packageId}"\nversion = "${metadata.version}"`)
  const url = `https://github.com/${expected.repository}/releases/download/${id.tag}/${id.asset}`
  if (metadata.releaseUrl !== url) throw Error('Unexpected release URL')
  return id
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const [command, argument] = process.argv.slice(2)
  if (command === 'plan') {
    const paths = git('diff', '--name-only', `${argument}...HEAD`).split('\n').filter(Boolean)
    const affected = affectedPackages(paths)
    const release = []
    for (const pkg of affected) {
      const file = `extensions/${pkg}/clipsx-extension.toml`
      const current = identity(readFileSync(file, 'utf8'))
      const previous = identity(git('show', `${argument}:${file}`))
      const packageChanged = paths.some(p => p.startsWith(`extensions/${pkg}/`))
      const sharedChanged = paths.some(changesPackageBytes)
      if (packageChanged || sharedChanged) assertVersionIncreased(previous.version, current.version)
      if (current.version !== previous.version) release.push(pkg)
    }
    console.log(JSON.stringify({ affected, release }))
  } else if (command === 'evidence') {
    const pkg = argument
    if (!packages.includes(pkg)) throw Error('Unknown package')
    const id = identity(readFileSync(`extensions/${pkg}/clipsx-extension.toml`, 'utf8'))
    const directory = `dist/${pkg}`
    mkdirSync(directory, { recursive: true })
    const metadata = JSON.parse(readFileSync(`${directory}/${id.asset.replace('.clipsx', '.registry.json')}`))
    const archive = readFileSync(`${directory}/${id.asset}`)
    const canonicalMetadata = JSON.stringify(metadata) + '\n'
    writeFileSync(`${directory}/${id.asset.replace('.clipsx', '.registry.json')}`, canonicalMetadata)
    const sourceTree = git('rev-parse', 'HEAD^{tree}')
    if (git('diff', '--name-only').length) throw Error('Build changed tracked source; commit generated source before preparation')
    writeFileSync(`${directory}/evidence.json`, JSON.stringify({ schemaVersion: 1, repository: process.env.GITHUB_REPOSITORY, prNumber: Number(process.env.PR_NUMBER), headSha: process.env.PR_HEAD_SHA, sourceTree, toolRef: readFileSync('.github/extension-tool-ref', 'utf8').trim(), runId: Number(process.env.GITHUB_RUN_ID), package: pkg, ...id, archiveSha256: digest(archive), metadataSha256: digest(Buffer.from(canonicalMetadata)) }) + '\n')
  } else throw Error('Usage: release-candidate.mjs plan <base-sha> | evidence <package>')
}
