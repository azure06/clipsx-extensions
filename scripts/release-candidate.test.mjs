import test from 'node:test'
import assert from 'node:assert/strict'
import { affectedPackages, changesPackageBytes, assertVersionIncreased, digest, verifyCandidate, verifyReleaseAssets, isPreparationRun, candidateArtifact, readCandidate, findRelease } from './release-candidate.mjs'
import { mkdtempSync, writeFileSync, mkdirSync, rmSync } from 'node:fs'
import { resolve } from 'node:path'
import { tmpdir } from 'node:os'

test('docs do not build packages; SDK affects all six; package paths are isolated', () => {
  assert.deepEqual(affectedPackages(['README.md']), [])
  assert.deepEqual(affectedPackages(['extensions/rewrite/README.md']), [])
  assert.equal(affectedPackages(['sdk/wit/clipsx-extension.wit']).length, 6)
  assert.deepEqual(affectedPackages(['extensions/rewrite/src/lib.rs']), ['rewrite'])
  assert.throws(() => assertVersionIncreased('2.0.0', '2.0.0'))
  assert.throws(() => assertVersionIncreased('2.0.1', '2.0.0'))
  assert.doesNotThrow(() => assertVersionIncreased('2.0.0', '2.0.1'))
  assert.equal(changesPackageBytes('package-lock.json'), true)
  assert.equal(changesPackageBytes('.github/extension-tool-ref'), false)
})

test('upload retries complete drafts, skip exact immutable assets, and reject conflicts', () => {
  const expected = [{ name: 'rewrite.clipsx', hash: 'a'.repeat(64), size: 12 }]
  const asset = { name: expected[0].name, digest: `sha256:${expected[0].hash}`, size: 12, state: 'uploaded' }
  const published = { draft: false, prerelease: false, immutable: true, assets: [asset] }
  assert.deepEqual(verifyReleaseAssets(published, expected), [])
  assert.deepEqual(verifyReleaseAssets({ ...published, draft: true, assets: [] }, expected, true), ['rewrite.clipsx'])
  assert.deepEqual(verifyReleaseAssets({ ...published, draft: true }, expected, true), [])
  assert.throws(() => verifyReleaseAssets({ ...published, assets: [] }, expected))
  assert.throws(() => verifyReleaseAssets({ ...published, immutable: false }, expected))
  assert.throws(() => verifyReleaseAssets({ ...published, assets: [{ ...asset, digest: 'wrong' }] }, expected, true))
  assert.throws(() => verifyReleaseAssets({ ...published, assets: [asset, asset] }, expected))
})

test('release lookup reuses unpublished drafts when the tag endpoint returns 404', async () => {
  const draft = { id: 42, tag_name: 'mermaid-v2.0.1', draft: true }
  const calls = []
  const api = async (path, optional) => {
    calls.push([path, optional])
    return path.startsWith('releases/tags/') ? null : [draft]
  }
  assert.equal(await findRelease(api, draft.tag_name), draft)
  assert.deepEqual(calls, [['releases/tags/mermaid-v2.0.1', true], ['releases?per_page=100&page=1', undefined]])
  const published = { ...draft, draft: false }
  assert.equal(await findRelease(async () => published, draft.tag_name), published)
})

test('release lookup paginates and never creates a draft from an incomplete or ambiguous lookup', async () => {
  const draft = { id: 42, tag_name: 'mermaid-v2.0.1', draft: true }
  const full = Array.from({ length: 100 }, (_, id) => ({ id, tag_name: `other-${id}` }))
  const api = async path => path.startsWith('releases/tags/') ? null : path.endsWith('page=1') ? full : [draft]
  assert.equal(await findRelease(api, draft.tag_name), draft)
  assert.equal(await findRelease(async path => path.startsWith('releases/tags/') ? null : [], draft.tag_name), null)
  await assert.rejects(findRelease(async path => path.startsWith('releases/tags/') ? null : [draft, { ...draft, id: 43 }], draft.tag_name), /Ambiguous/)
  await assert.rejects(findRelease(async path => path.startsWith('releases/tags/') ? null : full, draft.tag_name), /bound/)
})

test('promotion binds archive and metadata to the exact merged tree, PR, tooling and run', () => {
  const archive = Buffer.from('checked archive')
  const expected = { repository: 'azure06/clipsx-extensions', prNumber: 12, headSha: 'a'.repeat(40), sourceTree: 'b'.repeat(40), toolRef: 'c'.repeat(40), runId: 123 }
  const metadata = { packageId: 'infiniti.rewrite', version: '2.0.1', sha256: digest(archive), archiveSizeBytes: archive.length, releaseUrl: 'https://github.com/azure06/clipsx-extensions/releases/download/rewrite-v2.0.1/rewrite-2.0.1.clipsx' }
  const evidence = { schemaVersion: 1, ...expected, package: 'rewrite', packageId: metadata.packageId, version: metadata.version, archiveSha256: digest(archive), metadataSha256: digest(Buffer.from(JSON.stringify(metadata) + '\n')) }
  assert.equal(verifyCandidate(evidence, metadata, archive, expected).tag, 'rewrite-v2.0.1')
  for (const field of ['sourceTree', 'headSha', 'toolRef', 'runId', 'prNumber']) assert.throws(() => verifyCandidate({ ...evidence, [field]: 'forged' }, metadata, archive, expected))
  assert.throws(() => verifyCandidate(evidence, metadata, Buffer.from('wrong'), expected))
  assert.throws(() => verifyCandidate(evidence, { ...metadata, version: '2.0.2' }, archive, expected))
})

test('merged PR preparation is identifiable even when GitHub empties pull_requests', () => {
  const repository = 'azure06/clipsx-extensions', pr = { number: 10, head: { sha: 'a'.repeat(40), ref: 'codex/release' } }
  const run = { conclusion: 'success', display_title: 'Prepare extension release (PR 10)', event: 'pull_request', head_sha: pr.head.sha, head_branch: pr.head.ref, head_repository: { full_name: repository }, pull_requests: [] }
  assert.equal(isPreparationRun(run, pr, repository), true)
  assert.equal(isPreparationRun({ ...run, head_sha: 'b'.repeat(40) }, pr, repository), false)
  assert.equal(isPreparationRun({ ...run, display_title: 'Prepare extension release (PR 11)' }, pr, repository), false)
  assert.equal(isPreparationRun({ ...run, event: 'push' }, pr, repository), false)
  assert.equal(isPreparationRun({ ...run, event: 'workflow_dispatch', head_branch: 'main' }, pr, repository), true)
})

test('expired, missing or ambiguous artifacts cannot be promoted', () => {
  const artifact = { name: 'candidate-rewrite', expired: false }
  assert.equal(candidateArtifact([artifact], 'rewrite'), artifact)
  assert.throws(() => candidateArtifact([], 'rewrite'))
  assert.throws(() => candidateArtifact([{ ...artifact, expired: true }], 'rewrite'))
  assert.throws(() => candidateArtifact([artifact, artifact], 'rewrite'))
})

test('candidate files are canonical, bounded and regular; extra content is rejected', () => {
  const root = mkdtempSync(resolve(tmpdir(), 'release-artifact-')), id = { asset: 'rewrite-2.0.1.clipsx' }
  writeFileSync(resolve(root, id.asset), 'archive')
  writeFileSync(resolve(root, 'rewrite-2.0.1.registry.json'), '{}\n')
  writeFileSync(resolve(root, 'evidence.json'), '{}')
  assert.equal(readCandidate(root, id).archive.toString(), 'archive')
  writeFileSync(resolve(root, 'extra.txt'), 'unexpected')
  assert.throws(() => readCandidate(root, id))
  rmSync(resolve(root, 'extra.txt'))
  writeFileSync(resolve(root, 'rewrite-2.0.1.registry.json'), '{ }')
  assert.throws(() => readCandidate(root, id), /canonical/)
  writeFileSync(resolve(root, 'rewrite-2.0.1.registry.json'), '{}\n')
  writeFileSync(resolve(root, 'evidence.json'), 'x'.repeat(16 * 1024 + 1))
  assert.throws(() => readCandidate(root, id), /bounded/)
  rmSync(resolve(root, 'evidence.json')); mkdirSync(resolve(root, 'evidence.json'))
  assert.throws(() => readCandidate(root, id), /regular/)
})
