import { execFileSync } from 'node:child_process'
import { readFileSync, mkdirSync, writeFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { identity, verifyCandidate, verifyReleaseAssets, isPreparationRun, candidateArtifact, readCandidate } from './release-candidate.mjs'

const repository = process.env.GITHUB_REPOSITORY
if (repository !== 'azure06/clipsx-extensions') throw Error('Unexpected publication repository')
const event = JSON.parse(readFileSync(process.env.GITHUB_EVENT_PATH))
const pr = event.pull_request
if (!pr?.merged || pr.base.ref !== 'main' || pr.head.repo.full_name !== repository) throw Error('Publication requires an approved same-repository merge to main')
const sourceSha = pr.merge_commit_sha
const gh = (...args) => execFileSync('gh', args, { encoding: 'utf8' }).trim()
const api = async (path, optional = false) => {
  const response = await fetch(`https://api.github.com/repos/${repository}/${path}`, { headers: { Authorization: `Bearer ${process.env.GH_TOKEN}`, Accept: 'application/vnd.github+json' }, redirect: 'error' })
  if (optional && response.status === 404) return null
  if (!response.ok) throw Error(`GitHub publication lookup failed: HTTP ${response.status}`)
  return response.json()
}
const commit = await api(`git/commits/${sourceSha}`)
const toolRef = readFileSync('.github/extension-tool-ref', 'utf8').trim()
const baseSha = commit.parents[0]?.sha
if (!baseSha) throw Error('Approved merge has no base commit')
const releasePackages = []
for (const pkg of ['ask-ai', 'base64', 'data-tools', 'jwt-inspector', 'mermaid-viewer', 'rewrite']) {
  const path = `extensions/${pkg}/clipsx-extension.toml`
  const before = identity(execFileSync('git', ['show', `${baseSha}:${path}`], { encoding: 'utf8' }))
  const after = identity(readFileSync(path, 'utf8'))
  if (before.version !== after.version) releasePackages.push({ pkg, ...after })
}
if (!releasePackages.length) { console.log('No package version changes; nothing to publish.'); process.exit(0) }
const registryPinResponse = await fetch('https://api.github.com/repos/azure06/clipsx-registry/contents/.github/extension-tool-ref?ref=main', { headers: { Authorization: `Bearer ${process.env.GH_TOKEN}`, Accept: 'application/vnd.github+json' }, redirect: 'error' })
if (!registryPinResponse.ok) throw Error('Registry tooling pin is unavailable; complete the coordinated tooling rollout before publication')
const registryPin = await registryPinResponse.json()
if (registryPin.encoding !== 'base64' || Buffer.from(registryPin.content, 'base64').toString('utf8').trim() !== toolRef) throw Error('Extension and registry tooling pins differ; never publish an inconsistently validated release')
const runs = await api('actions/workflows/ci.yml/runs?status=success&per_page=100')
const eligible = runs.workflow_runs.filter(r => isPreparationRun(r, pr, repository))
let run, artifacts
for (const candidate of eligible) {
  const available = await api(`actions/runs/${candidate.id}/artifacts?per_page=100`)
  if (releasePackages.every(p => available.artifacts.some(a => a.name === `candidate-${p.pkg}` && !a.expired))) { run = candidate; artifacts = available; break }
}
if (!run) throw Error('No successful preparation run for the approved PR; rerun preparation for this exact revision')
const entries = []
for (const id of releasePackages) {
  const name = `candidate-${id.pkg}`
  candidateArtifact(artifacts.artifacts, id.pkg)
  const directory = resolve(process.env.RUNNER_TEMP || '.downloads', `candidate-${pr.number}-${id.pkg}`)
  mkdirSync(directory, { recursive: true })
  gh('run', 'download', String(run.id), '--repo', repository, '--name', name, '--dir', directory)
  const { metadataName, metadata, evidence, archive } = readCandidate(directory, id)
  verifyCandidate(evidence, metadata, archive, { repository, prNumber: pr.number, headSha: pr.head.sha, sourceTree: commit.tree.sha, toolRef, runId: run.id })
  if (id.packageId !== metadata.packageId || id.version !== metadata.version) throw Error('Candidate is not the package approved on main')
  let release = await api(`releases/tags/${id.tag}`, true)
  if (!release) {
    gh('release', 'create', id.tag, '--draft', '--repo', repository, '--target', sourceSha, '--title', `${id.packageId} ${id.version}`, '--notes', `Prepared once by PR #${pr.number}; tooling ${toolRef}.`)
    release = await api(`releases/tags/${id.tag}`)
  }
  const expectedAssets = [{ name: id.asset, hash: metadata.sha256, size: archive.length }, { name: metadataName, hash: evidence.metadataSha256, size: Buffer.byteLength(JSON.stringify(metadata) + '\n') }]
  if (release.draft) {
    const tag = await api(`git/ref/tags/${id.tag}`, true)
    if (release.target_commitish !== sourceSha || (tag && tag.object.sha !== sourceSha)) throw Error('Draft or tag belongs to a different revision; resolve the conflict without overwriting it')
    for (const name of verifyReleaseAssets(release, expectedAssets, true)) gh('release', 'upload', id.tag, resolve(directory, name), '--repo', repository)
    verifyReleaseAssets(await api(`releases/tags/${id.tag}`), expectedAssets, true)
    gh('release', 'edit', id.tag, '--draft=false', '--repo', repository)
    release = await api(`releases/tags/${id.tag}`)
  }
  verifyReleaseAssets(release, expectedAssets)
  entries.push({ ...metadata, publishedAt: release.published_at })
  console.log(`${id.packageId} ${id.version}: immutable assets verified`)
}
writeFileSync('published-releases.json', JSON.stringify({ schemaVersion: 1, sourceRepository: repository, sourceSha, prNumber: pr.number, toolRef, entries }, null, 2) + '\n')
