# rust-phase-skills

Replacement for the previous monolithic `rust-programming` skill. In addition it needs [BB-skill-core](https://github.com/BadBeta/BB-skill-core) which install hooks common to all languages. The install script will ask to install that too.

## Optimized for different development phases

Software work with Claude have phases of planning, implementing the plan, and then reviewing what was done. For each phase Claude is best served with a different *kind* of guidance. 

Planning by architectural decision tables and structural trade-offs (contexts, supervision shape, OTP boundaries). Implementing wants guidance that fires at the moment of writing like idiomatic templates, BAD/GOOD pairs, and "which construct?" tables. For reviewing severity-classified checklists, debugging playbooks and refactor templates work best.

With everything in a fat single major skill as before the LLM loads ~3000 mixed lines and segment will misfire: Planning advice trips during implementation, review heuristics on greenfield code, and the parts best focused on the current phase drowns between the others. Phase-targeted skills aim to give the right guidance at the right moment.

The phases intentionally overlap somewhat. Implementing references planning for "why this shape," reviewing references implementing for "what should I see here", and especially important parts can be repeated across two or all.

## Hooks

Getting the right skills invoked before Claude needs them can be a challenge. And having the skills invoked is not the same as the skills being actively used and applied. Claude is both lazy and arrogant.

The `BB-skill-core` hook stack closes that gap. The skill-enforcement hook (PreToolUse) blocks mutating tools like edit, write and mutating Bash until a relevant Skill has been invoked. The `[use-skills]` marker activates this enforcement for a session; `[no-skills]` opts out. It also activates some other hooks to help active use of the skills.

For important code decisions Claude it told to place a §§ marker in comment and cite any relevant decisions table or guidance in the skill for it's decisions. This is to promote active use of skills, and the comments can easily be scripted away later.

Another hook runs an anti-slop scanner that aims to catch some easy to detect issues that the skills warn about even if Claude ignored the skill. It fires before the offending slop is written to file, and thus while a better implementation can still be made with all context available. 

## TDD

`[TDD]` in any prompt activates session-wide TDD enforcement. New public `pub fn` declarations in `.rs` files trigger a forceful reminder unless one of these structural exemptions silences it:

- A test in the same project was edited within the last 15 minutes (test-first cycle in flight)
- The file co-locates tests (`#[cfg(test)] mod tests`)
- The function's name already appears in any file under `tests/`
- The function's name exists in `git log -S` history (rename, move, module split)

The exemptions aim to make TDD enforcement only fire on genuinely new behavior. Refactors should be silent. When the gate does fire, the message is the full annoying reminder every time on purpose. Because the cost of a missed reminder is high and the cost of a noticed one is small. Use `[no-TDD]` to cancel mid-session.

## Plans

For long-running, milestone-structured projects (ask for a milestone plan during planning) the hooks will force writing a skeleton milestone_skill_report.md with which skill sections were considered before starting the next step. 

This is not about reporting file as such. It is that writing this forces Claude to focus, work and use the relevant skills section before implementing. This is the strongest skill-engagement mechanism in the stack for long projects. 

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

The installs are namespaced 'bb' and purely additive, and should not affect any existing hooks or other installs. Likewise adding more languages will not affect the already installed ones. 

## Pack contents

- `rust-planning/`, `rust-implementing/`, `rust-reviewing/` — the three phase skills
- `hooks/bb-rationale-marker-rust.py` — flags `// §§` rationale markers left in committed code
- `hooks/bb-no-std-build-check.py` — verifies a `no_std` crate still builds after edits
- `hooks/bb-anti-slop-patterns.d/rust.json` — Rust + C anti-slop patterns
- `hooks/bb-skill-triggers.d/rust.json` — 26 keyword → skill mappings, all targeting the three bundled skills (`rust-planning`, `rust-implementing`, `rust-reviewing`). 
- `hooks/bb-post-generator-patterns.d/rust.json` — checks for `cargo new` / `cargo init` output

Claude has summarized a more detailed user guide which should be up to datish.
