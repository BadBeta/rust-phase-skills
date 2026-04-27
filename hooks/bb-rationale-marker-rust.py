#!/usr/bin/env python3
"""
Rationale marker hook for Claude Code — Rust.

PostToolUse hook for Rust files (.rs). Five Tier-1 rules; each
fires a tailored reminder when the named pattern appears without
an accompanying `§§` rationale marker.

Rules:
  1. Write of a new .rs file with `pub mod` or top-level `pub`
     items → marker citing why this layer exists
     (rust-planning §5).
  2. Edit introducing a `use` path with 4+ qualified segments →
     marker citing why a facade isn't being used
     (rust-planning §11 / rust-reviewing §7.1).
  3. Edit introducing `unsafe { ... }` or `unsafe fn` → marker
     citing the unsafe budget rule
     (rust-planning §28-31 / rust-implementing rule 10).
  4. Edit introducing `Box<dyn Error>` in a fn signature →
     marker citing why this case escapes
     (rust-implementing rule 19 / rust-planning rule 19).
  5. Edit introducing `tokio::spawn(` → marker citing the
     supervision strategy
     (rust-implementing rule 18 + M24 panic-safety lesson).

Skipped paths: tests/, benches/, examples/, *_test.rs, .claude/.

[use-skills] gated: this hook is silent in casual sessions. It
only fires when the latest typed user prompt contains the
`[use-skills]` marker — same gating shape as skill-enforcement.py
and rationale-marker-elixir.py.

Fails open: any exception exits 0 so the session is never bricked.
"""

import json
import os
import re
import sys

USE_MARKER = "[use-skills]"
NO_MARKER = "[no-skills]"

WRITE_TOOLS = {"Write"}
EDIT_TOOLS = {"Edit"}

# The rationale marker sentinel. If `§§` already appears in the
# new content, the writer has already justified the decision
# inline — silence the rule.
MARKER = re.compile(r"§§")

# Path patterns exempt from every rule.
EXEMPT_PATTERNS = [
    re.compile(r"_test\.rs$"),
    re.compile(r"(^|/)tests?/"),
    re.compile(r"(^|/)benches/"),
    re.compile(r"(^|/)examples/"),
    re.compile(r"\.claude/"),
    re.compile(r"/skill_hooks_mechanics/"),
    re.compile(r"/(?:rust|elixir)-phase-skills/"),
]


def path_is_exempt(path):
    return any(p.search(path) for p in EXEMPT_PATTERNS)


# ---------------------------------------------------------------
# Rule 1: new .rs file with `pub mod` or top-level `pub` items.
# ---------------------------------------------------------------

NEW_FILE_PUBLIC_SURFACE = re.compile(
    r"(?m)^pub\s+(?:mod|fn|async\s+fn|struct|enum|trait|type|const|static|use)\b"
)


def check_new_file(path, content):
    if not NEW_FILE_PUBLIC_SURFACE.search(content):
        return None
    if MARKER.search(content):
        return None
    return (
        "New Rust file with public surface but no rationale marker. "
        "A new module / file is an architectural decision — what does "
        "this layer do and why does it exist? Add at least one "
        "`// §§ <skill>: §<sec> — <why>` comment explaining the "
        "primary skill decision that shaped this module. Examples:\n"
        "  // §§ rust-planning §5.2 — workspace member crate; this "
        "file is the public surface of the `protocol` boundary, not "
        "internal types.\n"
        "  // §§ rust-implementing §11 — module organisation: "
        "domain-named (`orders`, `catalog`), not framework-named.\n"
        "  // §§ rust-planning §1 + §10 — domain layer; intentionally "
        "depends on no framework crates."
    )


# ---------------------------------------------------------------
# Rule 2: deep cross-boundary `use` path (4+ qualified segments).
# ---------------------------------------------------------------

# `use a::b::c::d` — the leftmost segment is `use` keyword's target.
# Matches paths with at least 4 `::`-separated identifiers AFTER the
# `use ` keyword, including across `{}` re-export groups.
DEEP_USE_PATH = re.compile(
    r"(?m)^\s*use\s+"
    r"(?:[a-zA-Z_][\w]*::){3,}"   # 3+ leading "ident::" → 4+ total segments
    r"[a-zA-Z_*{]"                # something useful (ident, *, or {)
)


def check_deep_use_path(new_string):
    if not DEEP_USE_PATH.search(new_string):
        return None
    if MARKER.search(new_string):
        return None
    return (
        "Deep cross-boundary `use` path added without a rationale "
        "marker. A `use` statement with 4+ qualified segments "
        "typically reaches into another module's internals — "
        "bypassing whatever facade exists at the boundary. "
        "Reasonable but worth documenting. Add a marker:\n"
        "  // §§ rust-planning §11 — bypassing facade intentionally; "
        "streaming codec needs the raw frame builder.\n"
        "  // §§ rust-reviewing §7.1 — internal use of pub(crate) "
        "type; same crate, no boundary crossing.\n\n"
        "If the path is just an idiomatic 4-segment stdlib path "
        "(`std::collections::hash_map::Entry`), the marker can be "
        "brief. If it's reaching into an OTHER crate's internals, "
        "the marker should explain why the public API isn't "
        "sufficient — that pressure usually reveals a missing "
        "boundary that should be added."
    )


# ---------------------------------------------------------------
# Rule 3: `unsafe { ... }` or `unsafe fn` introduction.
# ---------------------------------------------------------------

UNSAFE_INTRODUCTION = re.compile(
    r"\bunsafe\s*(?:fn\b|\{)"
)


def check_unsafe(new_string):
    if not UNSAFE_INTRODUCTION.search(new_string):
        return None
    if MARKER.search(new_string):
        return None
    return (
        "`unsafe` block or function added without a rationale "
        "marker. Every unsafe is a deliberate spend of the unsafe "
        "budget — document WHY this case justifies it on top of "
        "the `// SAFETY:` invariant comment. Add a marker:\n"
        "  // §§ rust-planning §28 — unsafe budget: zero-copy ETF "
        "decode avoids allocator pressure on hot path.\n"
        "  // §§ rust-implementing rule 10 — FFI boundary; raw "
        "pointers from C-side callback.\n"
        "  // §§ rust-planning §29 — wrapping unsafe in a safe "
        "abstraction with a documented contract; this is the only "
        "unsafe site in the safe public API.\n\n"
        "The §§ marker complements the // SAFETY: comment — SAFETY "
        "explains how the specific block upholds invariants; §§ "
        "explains why we're spending unsafe budget here at all."
    )


# ---------------------------------------------------------------
# Rule 4: `Box<dyn Error>` in a function signature.
# ---------------------------------------------------------------

BOX_DYN_ERROR = re.compile(
    r"Box\s*<\s*dyn\s+(?:[\w]+::)*\w*Error\b"
)


def check_box_dyn_error(new_string):
    if not BOX_DYN_ERROR.search(new_string):
        return None
    if MARKER.search(new_string):
        return None
    return (
        "`Box<dyn Error>` introduced without a rationale marker. "
        "Rust-implementing rule 19 / rust-planning rule 19: "
        "**NEVER** in published-library public APIs. Box<dyn Error> "
        "loses information and prevents pattern matching. Almost "
        "always the right answer is to switch to a typed error "
        "(`thiserror::Error` derive) or `anyhow::Error` for "
        "applications. Add a marker if this case is the exception:\n"
        "  // §§ rust-implementing rule 19 — internal seam, not "
        "published; thiserror would add an enum variant per "
        "external crate's error and we don't care.\n"
        "  // §§ rust-implementing rule 19 — prototype only; "
        "ticketed for replacement with typed error before 1.0.\n\n"
        "If this is in a published-library public API, the marker "
        "won't help — the right move is to redesign with typed "
        "errors before this commit lands."
    )


# ---------------------------------------------------------------
# Rule 5: `tokio::spawn(` introduction.
# ---------------------------------------------------------------

TOKIO_SPAWN = re.compile(
    r"\btokio::spawn\s*\("
)


def check_tokio_spawn(new_string):
    if not TOKIO_SPAWN.search(new_string):
        return None
    if MARKER.search(new_string):
        return None
    return (
        "`tokio::spawn(` introduced without a rationale marker. "
        "Rust-implementing rule 18: every spawn must have its "
        "JoinHandle tracked (in JoinSet, stored in state, or "
        "awaited). Untracked tasks that panic silently swallow "
        "errors and leak forever. Plus the M24 lesson: if the "
        "spawned task runs user-provided code, panics need to be "
        "caught (spawn_blocking + JoinError::is_panic()) so "
        "cleanup still runs. Add a marker:\n"
        "  // §§ rust-implementing rule 18 — handle stored in "
        "peer_tasks Vec; shutdown awaits all of them.\n"
        "  // §§ rust-implementing rule 18 — JoinSet handles all "
        "spawned children of the accept loop.\n"
        "  // §§ rust-planning §31 — spawn_blocking + JoinError "
        "panic-catch wrap; user closure can panic and cleanup "
        "still fires.\n\n"
        "If this spawn is genuinely fire-and-forget (e.g. a final "
        "shutdown signal), the marker should say so explicitly."
    )


# ---------------------------------------------------------------
# Dispatch
# ---------------------------------------------------------------

EDIT_RULES = [
    check_deep_use_path,
    check_unsafe,
    check_box_dyn_error,
    check_tokio_spawn,
]


def is_use_skills_active(transcript_path):
    """
    True iff the latest typed user message (in `transcript_path`)
    contains `[use-skills]` and not `[no-skills]`. Mirrors the
    gating logic in skill-enforcement.py so the marker hooks fire
    on the same trigger boundary as the rest of the strict-mode
    enforcement.

    Conservative: returns False on any failure to read the
    transcript (silent on its own implementation bugs).
    """
    if not transcript_path or not os.path.exists(transcript_path):
        return False
    try:
        latest_text = ""
        with open(transcript_path) as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                try:
                    rec = json.loads(line)
                except json.JSONDecodeError:
                    continue
                if rec.get("type") != "user":
                    continue
                msg = rec.get("message") or {}
                if msg.get("role") != "user":
                    continue
                content = msg.get("content")
                text = ""
                if isinstance(content, str):
                    text = content
                elif isinstance(content, list):
                    parts = []
                    for block in content:
                        if (
                            isinstance(block, dict)
                            and block.get("type") == "text"
                        ):
                            parts.append(block.get("text", ""))
                    text = "\n".join(parts)
                if text.strip():
                    latest_text = text
    except Exception:
        return False
    if NO_MARKER in latest_text:
        return False
    return USE_MARKER in latest_text


def handle(data):
    if not is_use_skills_active(data.get("transcript_path") or ""):
        return None

    tool_name = data.get("tool_name") or ""
    tool_input = data.get("tool_input") or {}
    path = tool_input.get("file_path") or ""

    if not path or not path.endswith(".rs"):
        return None
    if path_is_exempt(path):
        return None

    if tool_name in WRITE_TOOLS:
        content = tool_input.get("content") or ""
        return check_new_file(path, content)

    if tool_name in EDIT_TOOLS:
        new_string = tool_input.get("new_string") or ""
        if not new_string:
            return None
        for rule in EDIT_RULES:
            msg = rule(new_string)
            if msg:
                return msg
        return None

    return None


def main():
    try:
        data = json.load(sys.stdin)
    except Exception:
        return 0
    try:
        msg = handle(data)
    except Exception:
        return 0
    if msg:
        out = {
            "hookSpecificOutput": {
                "hookEventName": "PostToolUse",
                "additionalContext": msg,
            },
        }
        print(json.dumps(out))
    return 0


if __name__ == "__main__":
    sys.exit(main())
