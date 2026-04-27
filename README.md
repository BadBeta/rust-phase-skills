# rust-phase-skills

Three-phase Rust skill family + Rust-specific hooks. Layers on top of [BB-skill-core](https://github.com/BadBeta/BB-skill-core).

| Skill | Purpose |
|---|---|
| `rust-planning` | Architecture, crate boundaries, error/async/unsafe strategy, growth |
| `rust-implementing` | Idiomatic patterns, decision tables, BAD/GOOD pairs, TDD |
| `rust-reviewing` | PR review, debugging, profiling, anti-pattern catalog |

Plus:

- `bb-rationale-marker-rust.py` — flags `// §§` rationale markers left in committed code
- `bb-no-std-build-check.py` — verifies a `no_std` crate still builds after edits
- `bb-anti-slop-patterns.d/rust.json` — Rust + C anti-slop patterns (silent unwrap, unsafe-without-SAFETY, planning-citation-in-source, etc.)
- `bb-skill-triggers.d/rust.json` — keyword → skill mappings for Rust topics

## Install

```bash
git clone https://github.com/BadBeta/rust-phase-skills.git
cd rust-phase-skills
./install.sh
```

If `BB-skill-core` is not already installed, the script offers to clone and install it from GitHub. Set `BB_NONINTERACTIVE=1` to skip the prompt and fail-fast.

Override:
- `CLAUDE_HOME` — install root (default `$HOME/.claude`)
- `BB_CORE_REPO` — git URL for `BB-skill-core`

## Uninstall

```bash
./uninstall.sh
```

Removes only the Rust-pack files. `BB-skill-core` and other language packs are untouched.

## Coexistence

Both `rust-phase-skills` and `elixir-phase-skills` can be installed side-by-side. They drop their own per-language fragments into `~/.claude/hooks/bb-anti-slop-patterns.d/` and `~/.claude/hooks/bb-skill-triggers.d/`, which the core hooks merge at runtime.

## Version compatibility

Pinned in `REQUIRES_CORE`. The installer refuses if `BB-skill-core` is older than this minimum.
