# rust-phase-skills

Replacement for the previous monolithic `rust-programming` skill. Layers on top of [BB-skill-core](https://github.com/BadBeta/BB-skill-core).

## Skills optimized for different development phases

Software work happens in distinct phases — planning architecture, implementing the design, reviewing what was built — and each phase wants a different *kind* of guidance. Planning is dominated by architectural decision tables and structural trade-offs. Implementing wants idiomatic templates, BAD/GOOD pairs, and "which construct?" tables that fire at the moment of writing. Reviewing wants severity-classified checklists, debugging playbooks, and refactor templates.

When that all lives in a single skill, the LLM loads ~3000 mixed lines and applies whatever fragment surfaces — planning advice trips during implementation, review heuristics misfire on greenfield code, and the truly relevant section drowns in the rest. Phase-targeted skills make the right form of guidance fire at the right moment:

| Skill | Loaded when | Primary content |
|---|---|---|
| `rust-planning` | Architecture, structure, growth decisions | Numbered planning rules, master decision tables, layered/hexagonal patterns |
| `rust-implementing` | At-the-keyboard coding | "Which construct?" tables, idiomatic templates, BAD/GOOD pairs, TDD |
| `rust-reviewing` | After-the-fact inspection | Severity-classified checklists, debugging + profiling playbooks, refactor templates |

The phases overlap intentionally — implementing references planning for "why this shape," reviewing references implementing for "what should I see here" — but each is optimized for one moment in the work.

## Hooks

Skills loaded ≠ skills applied. Putting knowledge into context doesn't ensure the LLM walks the decision tables, reads the BAD/GOOD pairs, or recalls the rules at the right edit site.

The `BB-skill-core` hook stack closes that gap. The skill-enforcement hook (PreToolUse) blocks mutating tools — Edit, Write, mutating Bash — until a relevant Skill has been invoked in the recent window. Orientation operations (`ls`, `pwd`, `git status/log/diff`, file reads) are exempt; the gate only fires when the LLM is about to *change* something. The `[use-skills]` marker activates this enforcement for a session; `[no-skills]` opts out.

Combined with the anti-slop scanner (PostToolUse) and the post-generator scanner (one-shot after `cargo new` / `cargo init` / `cargo generate`), the stack catches what the skills warn about even when the LLM didn't re-attend to the relevant section — at the exact moment the file was written. Skills become checkpoints, not just context.

## TDD

`[TDD]` in any prompt activates session-wide TDD enforcement. New public `pub fn` declarations in `.rs` files trigger a forceful reminder unless one of these structural exemptions silences it:

- A test in the same project was edited within the last 15 minutes (test-first cycle in flight)
- The file co-locates tests (`#[cfg(test)] mod tests`)
- The function's name already appears in any file under `tests/`
- The function's name exists in `git log -S` history (rename, move, module split)

The exemptions matter: TDD enforcement fires only on genuinely new behavior. Refactors are silent. When the gate does fire, the message is the full reminder every time — no fade — because the cost of a missed reminder is high and the cost of a noticed one is small. Use `[no-TDD]` to cancel mid-session.

## Plans

For long-running, milestone-structured projects (a `PLAN.md` with `M1:` / `M2:` / `M3:` … markers), `bb-milestone-skill-report.py` (PreToolUse) blocks edits to project files until `milestone_skill_report.md` has an entry for the active milestone listing which skill sections were considered before starting it.

This is the strongest skill-engagement mechanism in the stack — not a passive reminder, not an "always cite" suggestion, but a hard gate on the next file edit. The LLM cannot start implementing M3 without first writing, visibly and verifiably, which skill sections were *relevant* to that milestone — not "all loaded skills," just the ones that apply. If a relevant skill hasn't been loaded yet, it gets loaded. If a loaded skill doesn't apply to this milestone, it gets omitted. The plan and the relevant skill fragments are pulled into a single scan-able artifact that proves "the right knowledge was on the table when the work began."

`bb-milestone-commit-check.py` complements this by gating `M\d+:`-prefixed commits — the milestone must be marked DONE in `PLAN.md` before its commit is allowed.

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

Both `rust-phase-skills` and `elixir-phase-skills` can be installed side-by-side. They drop their own per-language fragments into `~/.claude/hooks/bb-anti-slop-patterns.d/`, `bb-skill-triggers.d/`, and `bb-post-generator-patterns.d/`, which the core hooks merge at runtime.

## Pack contents

- `rust-planning/`, `rust-implementing/`, `rust-reviewing/` — the three phase skills
- `hooks/bb-rationale-marker-rust.py` — flags `// §§` rationale markers left in committed code
- `hooks/bb-no-std-build-check.py` — verifies a `no_std` crate still builds after edits
- `hooks/bb-anti-slop-patterns.d/rust.json` — Rust + C anti-slop patterns
- `hooks/bb-skill-triggers.d/rust.json` — keyword → skill mappings for Rust topics
- `hooks/bb-post-generator-patterns.d/rust.json` — checks for `cargo new` / `cargo init` output
