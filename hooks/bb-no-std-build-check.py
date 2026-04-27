#!/usr/bin/env python3
"""
no_std build verification hook.

PostToolUse on Edit/Write/NotebookEdit. When Claude touches a Rust
source file, run `cargo build --no-default-features --lib` if AND ONLY
IF the project has opted in. Surface any errors back as additionalContext.

**Opt-in mechanism:** the hook does nothing unless the project root
contains a marker file `.claude/check_no_std` OR the crate root
(`src/lib.rs`) contains `#![cfg_attr(not(feature = "std"), no_std)]`.
This keeps non-embedded projects free of the per-edit cargo cost.

**Cost:** typically a few seconds when the build cache is warm; longer
on first run after a deps change. The 30-second timeout is generous.
The hook fails open if cargo isn't available or the build is too slow.
"""

import json
import os
import re
import subprocess
import sys
from pathlib import Path

EDIT_TOOLS = {"Edit", "Write", "NotebookEdit"}
NO_STD_PATTERN = re.compile(
    r'#!\[cfg_attr\(not\(feature\s*=\s*"std"\),\s*no_std\)\]'
)


def find_project_root(start_dir):
    p = Path(start_dir).resolve()
    for _ in range(8):
        if (p / "Cargo.toml").is_file():
            return p
        if p.parent == p:
            return None
        p = p.parent
    return None


def project_opts_in(project_root):
    """Returns True if the project has signalled it cares about no_std
    build hygiene."""
    if (project_root / ".claude" / "check_no_std").is_file():
        return True
    lib_rs = project_root / "src" / "lib.rs"
    if not lib_rs.is_file():
        return False
    try:
        head = lib_rs.read_text(encoding="utf-8", errors="replace")[:4000]
    except Exception:
        return False
    return bool(NO_STD_PATTERN.search(head))


def should_check_file(file_path, project_root):
    """Only fire on Rust source under the project root."""
    if not file_path.endswith(".rs"):
        return False
    try:
        Path(file_path).resolve().relative_to(project_root.resolve())
        return True
    except ValueError:
        return False


def run_no_std_build(project_root):
    """Returns (rc, stderr_tail) — None on time-out / cargo missing."""
    try:
        result = subprocess.run(
            ["cargo", "build", "--no-default-features", "--lib"],
            cwd=str(project_root),
            check=False,
            capture_output=True,
            text=True,
            timeout=30,
        )
    except FileNotFoundError:
        return None
    except subprocess.TimeoutExpired:
        return None
    except Exception:
        return None
    return (result.returncode, result.stderr)


def main():
    try:
        data = json.load(sys.stdin)
    except Exception:
        return 0
    if (data.get("hook_event_name") or "") != "PostToolUse":
        return 0
    if (data.get("tool_name") or "") not in EDIT_TOOLS:
        return 0
    file_path = (data.get("tool_input") or {}).get("file_path") or ""
    if not file_path or not file_path.endswith(".rs"):
        return 0

    cwd = data.get("cwd") or os.getcwd()
    project_root = find_project_root(cwd)
    if project_root is None:
        return 0
    if not should_check_file(file_path, project_root):
        return 0
    if not project_opts_in(project_root):
        return 0

    outcome = run_no_std_build(project_root)
    if outcome is None:
        return 0
    rc, stderr = outcome
    if rc == 0:
        return 0

    # Trim stderr to the most relevant part — typically the last error
    # block plus a few lines of context.
    lines = (stderr or "").splitlines()
    tail = "\n".join(lines[-30:]) if lines else "(no stderr)"
    body = (
        "no_std build broke (`cargo build --no-default-features --lib`).\n"
        f"Project: {project_root}\n"
        f"Edit: {file_path}\n\n"
        f"Last 30 lines of stderr:\n{tail}\n\n"
        "Likely cause: a `std::*` reach (often a missing "
        "`use alloc::borrow::ToOwned;` / `use alloc::format;`) "
        "that the std prelude was hiding. The default-features build "
        "is still fine, but the embedded port path just lost its build."
    )
    out = {
        "hookSpecificOutput": {
            "hookEventName": "PostToolUse",
            "additionalContext": body,
        }
    }
    print(json.dumps(out))
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as e:
        sys.stderr.write(f"no-std-build-check error: {e}\n")
        sys.exit(0)
