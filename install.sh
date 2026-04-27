#!/usr/bin/env bash
# rust-phase-skills installer.
#
# Depends on BB-skill-core (the language-independent hook + skill plumbing).
# If core is not detected, this script prompts to clone+install it from
# GitHub before continuing. Override $BB_CORE_REPO to point elsewhere.
#
# Layout after install:
#   $HOME/.claude/hooks/
#     bb-rationale-marker-rust.py
#     bb-no-std-build-check.py
#     bb-anti-slop-patterns.d/rust.json
#     bb-skill-triggers.d/rust.json
#   $HOME/.claude/skills/
#     rust-planning/  rust-implementing/  rust-reviewing/
#   $HOME/.claude/settings.json    (rust-pack hook entries merged in)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLAUDE_HOME="${CLAUDE_HOME:-${HOME}/.claude}"
HOOKS_DIR="${CLAUDE_HOME}/hooks"
SKILLS_DIR="${CLAUDE_HOME}/skills"
SETTINGS="${CLAUDE_HOME}/settings.json"
FRAGMENT="${SCRIPT_DIR}/settings-fragment.json"
BB_CORE_REPO="${BB_CORE_REPO:-https://github.com/BadBeta/BB-skill-core.git}"

CORE_SENTINEL="${HOOKS_DIR}/bb-anti-slop-scan.py"

command -v python3 >/dev/null 2>&1 || { echo "python3 is required" >&2; exit 1; }

echo "rust-phase-skills install"
echo "  source:    ${SCRIPT_DIR}"
echo "  install:   ${CLAUDE_HOME}"
echo

# ── 1. Verify / bootstrap BB-skill-core ───────────────────────────────────
if [ ! -f "${CORE_SENTINEL}" ]; then
    echo "BB-skill-core is not installed (missing ${CORE_SENTINEL})."
    if [ "${BB_NONINTERACTIVE:-}" = "1" ]; then
        echo "Set BB_NONINTERACTIVE=0 to enable the prompt, or install" >&2
        echo "BB-skill-core first." >&2
        exit 1
    fi
    printf "Clone and install BB-skill-core from %s ? [Y/n] " "${BB_CORE_REPO}"
    read -r reply
    case "${reply}" in
        ""|y|Y|yes|Yes) ;;
        *) echo "Aborting. Install BB-skill-core first." ; exit 1 ;;
    esac
    command -v git >/dev/null 2>&1 || { echo "git is required to clone core" >&2; exit 1; }
    tmp_core="$(mktemp -d)"
    trap 'rm -rf "${tmp_core}"' EXIT
    git clone --depth 1 "${BB_CORE_REPO}" "${tmp_core}/BB-skill-core"
    bash "${tmp_core}/BB-skill-core/install.sh"
    if [ ! -f "${CORE_SENTINEL}" ]; then
        echo "Core install did not produce ${CORE_SENTINEL}; aborting." >&2
        exit 1
    fi
fi

# ── 2. Optional version check ─────────────────────────────────────────────
if [ -f "${SCRIPT_DIR}/REQUIRES_CORE" ] && [ -f "${CLAUDE_HOME}/BB-skill-core.VERSION" ]; then
    required="$(tr -d '[:space:]' < "${SCRIPT_DIR}/REQUIRES_CORE")"
    have="$(tr -d '[:space:]' < "${CLAUDE_HOME}/BB-skill-core.VERSION")"
    # Lex compare is fine for semver-like strings here (0.x range)
    if [ "${have}" \< "${required}" ]; then
        echo "BB-skill-core ${have} < required ${required}; upgrade core first." >&2
        exit 1
    fi
fi

mkdir -p "${HOOKS_DIR}/bb-anti-slop-patterns.d"
mkdir -p "${HOOKS_DIR}/bb-skill-triggers.d"
mkdir -p "${HOOKS_DIR}/bb-post-generator-patterns.d"
mkdir -p "${SKILLS_DIR}"

# ── 3. Hook files (Rust-specific) ─────────────────────────────────────────
echo "[1/4] copying rust-pack hooks…"
cp -p "${SCRIPT_DIR}/hooks/bb-rationale-marker-rust.py" "${HOOKS_DIR}/"
cp -p "${SCRIPT_DIR}/hooks/bb-no-std-build-check.py" "${HOOKS_DIR}/"
chmod +x "${HOOKS_DIR}/bb-rationale-marker-rust.py" "${HOOKS_DIR}/bb-no-std-build-check.py" 2>/dev/null || true

# ── 4. Drop-in fragments ──────────────────────────────────────────────────
echo "[2/4] copying drop-in fragments…"
cp -p "${SCRIPT_DIR}/hooks/bb-anti-slop-patterns.d/rust.json" "${HOOKS_DIR}/bb-anti-slop-patterns.d/"
cp -p "${SCRIPT_DIR}/hooks/bb-skill-triggers.d/rust.json" "${HOOKS_DIR}/bb-skill-triggers.d/"
[ -f "${SCRIPT_DIR}/hooks/bb-post-generator-patterns.d/rust.json" ] && \
  cp -p "${SCRIPT_DIR}/hooks/bb-post-generator-patterns.d/rust.json" "${HOOKS_DIR}/bb-post-generator-patterns.d/"

# ── 5. Skills ─────────────────────────────────────────────────────────────
echo "[3/4] copying rust skills…"
for sk in rust-planning rust-implementing rust-reviewing; do
    src="${SCRIPT_DIR}/${sk}"
    if [ -d "${src}" ]; then
        rm -rf "${SKILLS_DIR}/${sk}"
        cp -R "${src}" "${SKILLS_DIR}/${sk}"
    fi
done

# ── 6. Merge settings ─────────────────────────────────────────────────────
echo "[4/4] merging settings…"
merge_script="${CLAUDE_HOME}/.bb-merge-settings.py"
if [ ! -f "${merge_script}" ]; then
    # Find the merger from core install (fallback if not present)
    candidate="${CLAUDE_HOME}/install/merge_settings.py"
    if [ -f "${candidate}" ]; then merge_script="${candidate}"
    else
        # Search relative to the parent script
        for path in \
            "${SCRIPT_DIR}/../BB-skill-core/install/merge_settings.py" \
            "${SCRIPT_DIR}/install/merge_settings.py" ; do
            [ -f "${path}" ] && merge_script="${path}" && break
        done
    fi
fi
if [ ! -f "${merge_script}" ]; then
    echo "Cannot find merge_settings.py — re-run BB-skill-core/install.sh." >&2
    exit 1
fi

cp -p "${SETTINGS}" "${SETTINGS}.bak.$(date +%Y%m%d-%H%M%S)" 2>/dev/null || true
tmp="$(mktemp)"
python3 "${merge_script}" merge "${SETTINGS}" "${FRAGMENT}" > "${tmp}"
mv "${tmp}" "${SETTINGS}"

[ -f "${SCRIPT_DIR}/VERSION" ] && cp -p "${SCRIPT_DIR}/VERSION" "${CLAUDE_HOME}/rust-phase-skills.VERSION"

echo
echo "rust-phase-skills installed."
