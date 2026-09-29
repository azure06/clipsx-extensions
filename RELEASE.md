# Extension releases

## Normal flow

```text
Versioned PR to main
  → prepare affected packages once → release-ready
  → human merge
  → publish the same checked bytes as immutable releases
  → open/update one registry metadata PR
Registry PR
  → exact archive validation → trusted catalog signing → publication-ready
  → human merge → Discover + portable-setting reconciliation
```

Change the package's manifest and Cargo version together. Package or shared
build-input changes require higher versions; existing releases are never
replaced. SDK and shared build-input changes validate all six packages. A tooling
pin or validator-invocation update validates all packages without forcing version
changes. Complete both main-branch pin updates before merging a versioned release;
the publisher checks cross-repository pin equality before uploading anything. Documentation
outside package directories does not run packaging or publish anything.

Only PRs to `main` prepare releases. The pinned host validator is built once and
shared across the package jobs. There is no package build on a `main` or
`develop` push. `npm run package -- <package> <archive> <release-url>` performs
unit tests, compiles the guest once, packs that component twice to check archive
determinism, then validates and runs conformance. Each release candidate contains
the archive, generated technical metadata and bounded evidence binding it to the
PR, tested Git tree, preparation run, version, tooling pin and both file hashes.
Artifacts remain available for 90 days. The required `release-ready` check
covers every affected package, including Rewrite, and confirms uploads exist.

Merging triggers `release.yml` through the trusted base-branch
`pull_request_target: closed` event. Its job requires an actual same-repository
merge and checks out that approved merge commit, never an unmerged PR head.
This keeps publication within the environments' `main` restriction.
The publisher compares the candidate's entire
tested tree to the merged tree, not only its version or branch. A moved base or
changed source makes promotion fail; update the PR and prepare again before
merge. It completes and verifies draft assets before publishing, then verifies
immutable status, uploaded state, SHA-256 and size. Retries skip exact matching
publications, finish matching interrupted drafts, and reject conflicting assets
or tags. They never overwrite or silently rebuild.

The generated registry PR preserves reviewed marketplace copy and versioned
icons. Its technical fields come from the checked archive. New packages need
initial reviewed marketplace fields/icons in the registry before automation can
import them. The importer runs trusted registry `main` code against bounded PR
data; it does not check out or execute the registry PR's scripts.

## One-time repository setup

1. Merge reviewed host tooling first. Put its full commit SHA in
   `.github/extension-tool-ref` here and in the registry. Both repositories must
   use the same pin. The pin fixes the validator contract for reproducible
   preparation; a moving branch could change it mid-release.
2. Create a dedicated GitHub App, installed only on `clipsx-extensions` and
   `clipsx-registry`, with **Contents: read/write** and **Pull requests:
   read/write**. Do not grant Workflows, Administration, or protection bypass.
   Publication uses the workflow token for releases; the App creates registry
   branches/PRs so GitHub runs their required checks.
3. Set repository variable `CLIPSX_RELEASE_APP_ID` in both repositories.
   Store `CLIPSX_RELEASE_APP_PRIVATE_KEY` only in the environments
   `extension-publication` here and `registry-signing` in the registry. Restrict
   both to `main`. Never use a repository-level private-key secret: same-repo
   PR workflows could request it. Never commit or paste private keys into logs.
4. Keep protected `main`, strict required checks and human-controlled merging.
   Restrict updates to `main` to human maintainers using a main-only update
   ruleset or supported push restriction; the App may update PR branches only.
   Do not grant it bypass. After the replacement checks have run, require
   `release-ready` instead of the old per-package checks. Do not temporarily
   remove protection to bootstrap the new flow.
5. Finish the registry setup in its
   [operations guide](https://github.com/azure06/clipsx-registry/blob/main/OPERATIONS.md).
   Neither ordinary preparation jobs nor this repository receive the catalog
   signing key. Human merges replace the old per-run signing approval.

## Recovery

Use the failed job's log to choose the smallest retry:

| Failure | Recovery |
| --- | --- |
| Tests/conformance/versioning | Fix the open PR and rerun preparation |
| Missing readiness artifact | Rerun the failed preparation job before merge |
| Upload or registry handoff interrupted | `gh run rerun <publication-run> --failed`; verified assets are reused |
| Candidate expired or tree differs after merge | `gh workflow run ci.yml --ref main -f merged_pr=<number>`, then rerun publication |
| Existing tag/asset differs | Stop; investigate and issue a higher version rather than replacing it |
| App/environment not configured | Fix the one-time setup, then rerun the failed publication job |

Recovery preparation checks out the exact approved merge and records the
original PR identity. It is the only deliberate rebuild after merge. Keep
preparation code unchanged on that exact revision; changes belong in a new PR.
Do not dispatch recovery for unmerged work.

## Verification

Run `node --test scripts/*.test.mjs` and lint affected workflows locally. Use
existing immutable assets and fixtures to validate automation changes; do not
bump packages just to test the pipeline. Check the preparation artifact hash
against the GitHub asset digest and confirm `immutable: true`. The publisher
reports each verified release and the registry PR URL. The registry merge makes
it available in Discover; desktop installation/native certification remains a
separate host release responsibility. No Supabase schema change is required.
