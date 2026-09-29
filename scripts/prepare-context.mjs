import { readFileSync, appendFileSync } from 'node:fs'
const event = JSON.parse(readFileSync(process.env.GITHUB_EVENT_PATH))
const repo = process.env.GITHUB_REPOSITORY
let pr = event.pull_request
let sourceSha = process.env.GITHUB_SHA
let baseSha = pr?.base.sha
if (process.env.GITHUB_EVENT_NAME === 'workflow_dispatch') {
  if (process.env.GITHUB_REF !== 'refs/heads/main') throw Error('Recovery workflow must run from reviewed main')
  const number = Number(event.inputs.merged_pr)
  if (!Number.isSafeInteger(number) || number < 1) throw Error('Recovery requires a merged PR number')
  const get = async path => {
    const r = await fetch(`https://api.github.com/repos/${repo}/${path}`, { headers: { Authorization: `Bearer ${process.env.GH_TOKEN}`, Accept: 'application/vnd.github+json' } })
    if (!r.ok) throw Error(`Recovery lookup failed: HTTP ${r.status}`)
    return r.json()
  }
  pr = await get(`pulls/${number}`)
  if (!pr.merged) throw Error('Recovery only builds an already approved merge')
  sourceSha = pr.merge_commit_sha
  baseSha = (await get(`git/commits/${sourceSha}`)).parents[0].sha
}
if (!pr || pr.base.ref !== 'main' || pr.head.repo.full_name !== repo || !/^[a-f0-9]{40}$/.test(sourceSha) || !/^[a-f0-9]{40}$/.test(baseSha)) throw Error('Preparation requires a same-repository main PR')
appendFileSync(process.env.GITHUB_OUTPUT, `source_sha=${sourceSha}\nbase_sha=${baseSha}\npr_number=${pr.number}\nhead_sha=${pr.head.sha}\n`)
