---
name: clipsx-extension-release
description: Prepare, publish or recover ClipsX extension releases, including versioning, candidate evidence and promotion after merge. Ordinary package behaviour review uses focused local builds instead.
---

# Extension release workflow

1. Read [release requirements](../../../RELEASE.md) and the actual
   [preparation](../../../.github/workflows/ci.yml) and
   [publisher](../../../.github/workflows/release.yml). Use the manifest and
   tooling pin as authoritative inputs; do not copy versions into instructions.
2. Determine affected packages and version increases with
   `node scripts/release-candidate.mjs plan <base-sha>`. Use the existing
   packaging command; PR preparation compiles once and retains archive,
   metadata and evidence. Documentation-only work needs no release.
3. Check `release-ready` and candidate provenance before the authorized human
   merge. Promotion must compare the complete tested tree and tooling pin with
   the approved merge, then publish those exact bytes. Never rebuild routinely
   on merge, overwrite releases, or accept evidence only by version name.
4. Verify immutable asset hashes/sizes and the automatically generated registry
   PR. Registry review/signing belongs to its repository publication skill;
   publishing bytes alone does not make them visible in Discover.
5. For failures, follow the recovery table. Retry upload/import without rebuilding.
   Only expired/mismatched artifacts justify exact-approved-revision recovery
   preparation. Conflicting immutable assets require a new version.
6. Record demonstrated artifact/publication checks and remaining installed
   checks. Skills do not authorize merges or publication beyond the user task.

Keep credentials, detailed setup and protection requirements in the release
manual; never put private keys in repository files or agent output.
