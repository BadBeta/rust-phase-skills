# Long-Running Rust Projects — Session Handoff and Milestone Management

This subskill covers the meta-workflow that sits *above* individual
features: how to run a Rust project that spans multiple sessions,
dozens of milestones, and months of elapsed time. The plan →
implement → review triad in the main `rust-planning` / `rust-implementing`
/ `rust-reviewing` skills handles a single feature well; it does not
answer:

- When do I update `continue.md`?
- What belongs in `PLAN.md` vs. `continue.md` vs. commit messages?
- How do I write a commit message that's useful for future-me in six
  months?
- What invariants should I verify at milestone boundaries?
- When does a "pending items" list need pruning vs. amending?

This file answers those questions. Load when starting, restarting, or
continuing a multi-session Rust project.

## The three-document model

A healthy long-running project maintains three coordinated documents:

| Document | Audience | Cadence | Truth-time |
|---|---|---|---|
| `PLAN.md` | Any engineer (incl. future-you) | Edit at milestone-start and milestone-end | Intent (slow-moving) |
| `continue.md` | Future-you picking up cold | Rewrite at every milestone finish | Current state (snapshot) |
| Commit messages | Anyone running `git log` | Every commit | Historical record |

Each document has a distinct job. Mixing them (putting `continue.md`
content in commit messages, putting commit-level detail in
`continue.md`) signals the system is drifting.

### `PLAN.md` — the design intent

What it contains:

- The problem you're solving (once, at top).
- Scope — MVP, explicitly out-of-scope, appendix-to-scope.
- Design decisions with rationale (not just "we did X" but "we did X
  because Y").
- A §12-style "out-of-scope milestones" / "pending items" list.
- Single-source-of-truth policy (what file owns each kind of
  invariant — e.g. "all protocol magic numbers live in `src/policy.rs`").

What it does NOT contain:

- Current milestone status (that belongs in `continue.md`).
- Blow-by-blow commit history (that belongs in `git log`).
- Solved-and-shipped design debates (move to an `ARCHIVE.md` or
  delete once the code is the source of truth).

**Edit cadence.** Touch `PLAN.md` at milestone-start (to record any
design decisions that were not obvious before implementation) and
at milestone-end (to mark shipped items as DONE). A `PLAN.md` that
hasn't been touched in ten milestones is stale, not durable.

### `continue.md` — the session handoff

One-paragraph summary at the top. Then sections that a cold-start
reader needs:

1. **Project location** — path and git branch.
2. **One-line summary** — what the project *is*.
3. **Current status snapshot** — commits, tests passing, clippy clean.
4. **What works live** — numbered list, specific enough to reproduce.
5. **Project layout** — directory tree with one-line-per-file purpose.
6. **Dependency direction diagram** — so a reader knows what depends
   on what.
7. **How to run things** — exact commands. Tests, build, live demo.
8. **Architectural invariants** — the load-bearing ones, grep-verifiable
   where possible (e.g., "every `Duration::from_*` outside `policy.rs`
   is a bug").
9. **Non-obvious lessons learned** — bugs that cost real debugging and
   how they manifested.
10. **Remaining items** — pointing at `PLAN.md §12`, not duplicating.
11. **Recommended next step** — one paragraph, opinionated.

**Edit cadence.** Rewrite `continue.md` at every milestone finish. Not
amend — rewrite the relevant sections. Staleness in `continue.md` is
worse than staleness in `PLAN.md` because `continue.md` claims to
describe *now*.

### Commit messages — the historical record

Structure:

```
<short title>: <one-line summary>

<2-5 sentence paragraph explaining why, not what>

<optional bullet list of concrete changes>

<verification — tests run, clippy output, manual check>

Co-Authored-By: ... (if applicable)
```

The `why` is the load-bearing part. "Fixes bug" is not a commit
message; "Fixes deadlock — MutexGuard was held across `.await` in
`handle_request`; pulled the lock acquisition into a narrower scope"
is. The `what` is in the diff; don't repeat it in prose.

For milestone commits, the title follows `M<N>: <one-line intent>`.
The N lets `git log --oneline | head -20` read like a table of
contents.

## Milestone-boundary checklist

Before committing what you consider a "milestone finish," walk this
list. If any answer is "I'll do it later," it's not a milestone.

1. **Does the public API have a failing test that now passes?**
   Tests-first discipline at milestone scope. If the milestone
   added `Node::connect_tls`, there's an integration test that
   exercises it. The TDD state hook is a backstop; this checklist
   is the foreground check.
2. **Does `cargo clippy --all-targets --all-features -- -D warnings`
   pass?** And `--no-default-features`? Both configurations, because
   features gate code that clippy only sees when enabled.
3. **Does `cargo test --all-features` pass? And `--no-default-features`?**
   Same reasoning. A feature that builds but doesn't test isn't
   shipped.
4. **Is the new public API documented?** Every `pub fn` that returns
   `Result` has a `# Errors` section. Every `pub fn` that can panic
   has `# Panics`. Every `pub unsafe fn` has `# Safety`. No exceptions.
5. **Did you update `continue.md`?** Specifically: the commit count,
   the "what works live" list, the project layout if any new file
   appeared.
6. **Did you update `PLAN.md`?** Specifically: mark pending items as
   DONE, append new follow-ups discovered during the milestone.
7. **Is the commit message complete?** Title, why, bullets, verification.
8. **Did you run the long test?** For BEAM-integration / TCP / embedded
   projects: the live end-to-end test that needs a real external
   process (real `erl`, real hardware, real network peer). Not just
   the hermetic unit tests.

One pre-commit run, not a loop. If something fails, fix it and
re-run.

## SSOT invariant verification

For any project that declares an SSOT policy ("all X live in file Y"),
run the invariant check at milestone boundaries. Example for a crate
that declares `src/policy.rs` owns all protocol constants:

```bash
# Every Duration outside policy.rs is a bug or a test fixture.
grep -rn 'Duration::from_' src/ --include='*.rs' | grep -v 'src/policy.rs'

# Every protocol tag byte outside policy.rs is a bug or a test fixture.
grep -rn '= 0x[0-9a-f]\+u8' src/ --include='*.rs' | grep -v 'src/policy.rs'
```

Anything surfaced should either be justified (a test fixture with a
`// RULE-EXCEPTION: ssot — test fixture` marker), moved into the SSOT
file, or referenced from it. Doing this ONCE per milestone keeps drift
low; skipping it for three milestones produces a fix-it-all sprint.

Add to `continue.md` §8 (architectural invariants) the exact grep
commands that *should return nothing* or *return a known small list*.
Verification becomes a scripted check, not a memory exercise.

## Pending items — prune vs. amend

A pending-items list in `PLAN.md §12` (or wherever you keep it) decays
without discipline. Two failure modes:

1. **Hoarding.** Every idea goes on the list; the list grows without
   bound; nothing is ever removed. Readers can't tell "maybe someday"
   from "next milestone."
2. **Silent drift.** Items are implemented without being marked DONE.
   `PLAN.md` claims a feature is pending that actually shipped three
   milestones ago.

Counter-measures:

- **Explicit DONE annotation, inline.** Don't remove the item — append
  `**DONE in M15** (commit hash, one-line summary).` That preserves
  the original framing (useful for future readers who want to know
  what the original scope was) while showing status.
- **Age-out.** Any item unshipped after ~10 milestones should be
  re-examined. If it's still desirable, restate it with current
  context. If it's no longer desirable, move it to a `DEFERRED.md`
  or delete with a note.
- **Numbering is immutable.** If the list numbers items 1-10, don't
  renumber when you drop one. `#3 (removed — superseded by #7)` keeps
  cross-references stable.
- **Separate "original" from "appended-later."** If the milestone
  arc has multiple phases (initial scope, then discovered items),
  keep them in separate lists so the original intent is legible.

## Commit-message style for long-running refactors

A multi-commit refactor (e.g., "address code review findings M15") is
NOT a series of independent commits. It's one logical change broken up
for review. The commit messages should reflect that.

Two styles that work:

### Style A: one-shot commit

```
Address code-review findings from M15/M16/M17 pass

Works through all 10 findings from the rust-reviewing sweep on the
TLS + Elixir-supervision milestones. No behavior change for existing
callers; one breaking surface change (Error::Tls variant shape).

### Errors and docs (R1, R2, S7, S9)
...

### Spawn-handle tracking (R3)
...
```

Single commit, structured sections for each grouped finding. Good for
review passes where findings interact.

### Style B: series with shared prefix

```
M15-review-1/N: typed TlsError sub-enum (S7)
M15-review-2/N: # Errors docs on new public functions (R1)
M15-review-3/N: track JoinHandles on connect/connect_tls (R3)
...
```

Shared prefix, explicit N-of-M counter, one finding per commit. Good
when findings are orthogonal and a reviewer might want to approve
some but not others.

Pick one, don't mix. The prefix+counter style is more ceremonial;
the one-shot is blunter. For solo work on a feature branch, one-shot
is usually fine.

## Cross-session handoff checklist

Before closing a session (closing the terminal, ending the Claude
window, merging the PR), run:

1. **Is the working tree clean?** `git status` should be empty. No
   half-finished files. If there's in-progress work, commit as
   `WIP: <topic>` so future-you can find it.
2. **Are all commits pushed?** (If the project has a remote.)
3. **Does `continue.md` describe the current state?** Even one stale
   sentence burns future-you's trust in the document.
4. **Is the next action explicit?** `continue.md` §11 "Recommended
   next step" is the first thing future-you reads. Make it unambiguous.
5. **Are there any open questions you need to resolve before future-
   you can proceed?** Those belong in `continue.md` under a
   "Blockers / open questions" section, not in your head.

If future-you can't pick up the project from a cold read of
`continue.md` + `PLAN.md` + `git log`, something is missing. The
answer is almost always in one of those three places; the question is
whether you put it there.

## Autonomous-mode warnings for long-running work

In autonomous / milestone-by-milestone sessions, specific failure
modes compound over many milestones:

- **Citation leakage.** Inline comments like `// M15 fix for bug #42`
  rot. See anti-slop pair #17 / planning-citation-in-source check.
- **Stale `continue.md`.** Each session updates at the end but never
  reads at the start. Cold-reads of your own doc are how you notice
  it's lying.
- **Accumulated pending items.** Every milestone appends to §12,
  never prunes. By milestone 20 the list is unreadable.
- **Commit-message decay.** Early milestones have `M1: full
  rationale`; late ones have `M17: fix`. The style drift is visible
  from `git log --oneline`.
- **SSOT violation creep.** Each milestone adds one "harmless"
  magic-number inline. Eight milestones later, the SSOT file is
  worthless.

The counter to all of these is the milestone-boundary checklist
above. Run it every time. Taking 5 minutes at each milestone boundary
saves hours of bit-rot reconstruction later.

## When a project hibernates

If you know a project is about to pause (vacation, context switch,
"back in a month"), invest in extra handoff quality BEFORE closing
the last session. Specifically:

1. **Re-read `continue.md` cold.** Pretend it's a project you've
   never seen. What confuses you? Fix those spots.
2. **Freeze a known-good state.** Tag the commit (`git tag
   milestone-M17-shipped`) so you can return to a known reference.
3. **Record the tooling state.** Which Rust/OTP/Elixir versions did
   this build against? Add them to `continue.md` §7 (how to run).
4. **Record discovered-but-unrecorded context.** Anything in your
   head that isn't in a file. Dump it into `continue.md`
   "Non-obvious lessons learned."
5. **Write the next-session kickoff prompt.** A literal paragraph
   you can paste into a new session: "We're resuming project X.
   Read `continue.md`, then tackle item Y."

Hibernation-quality handoff is over-investment in ongoing development
but rational before a long pause. The test: can a version of you from
three months ago pick up where you left off in under 15 minutes? If
not, one more read of `continue.md` is warranted.

## Related

- `test-strategy.md` — planning-time test pyramid decisions
- `workspace-layout.md` — when single crate → lib+bin → workspace
- `../rust-implementing/SKILL.md §0` — the TDD gate that fires at
  every public function
- `../rust-reviewing/SKILL.md §17 (AI-tells)` — the review-time
  backstop for citation leakage, silent fallbacks, SSOT drift
