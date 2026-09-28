# ClipsX extensions

Independent package sources live in `extensions/`; the pinned guest contract is
in `sdk/wit/`; build/packaging commands live in `scripts/`. Preserve unrelated
changes and immutable released assets.

- Package behaviour work: consult the host extension API and build only the
  package under review with the existing packaging command.
- Publication or release recovery: use
  [clipsx-extension-release](.agents/skills/clipsx-extension-release/SKILL.md)
  and [RELEASE.md](RELEASE.md). Do not rebuild at merge.
- Run focused package/helper tests and keep documentation consistent with the
  actual workflows. Version and contract sources remain authoritative.
