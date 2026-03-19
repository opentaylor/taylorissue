#!/bin/bash
set -euo pipefail

BOLD='\033[1m'
GREEN='\033[38;2;0;229;204m'
YELLOW='\033[38;2;255;176;32m'
RED='\033[38;2;230;57;70m'
DIM='\033[38;2;136;146;176m'
NC='\033[0m'

ok()   { echo -e "${GREEN}✓${NC} $*"; }
info() { echo -e "${DIM}·${NC} $*"; }
warn() { echo -e "${YELLOW}!${NC} $*"; }
die()  { echo -e "${RED}✗${NC} $*"; exit 1; }

CN_NPM="https://registry.npmmirror.com"
USE_CN=0

detect_cn() {
    local tz=""
    [[ -L /etc/localtime ]] && tz="$(readlink /etc/localtime 2>/dev/null | sed 's|.*/zoneinfo/||')"
    [[ -z "$tz" && -f /etc/timezone ]] && tz="$(cat /etc/timezone 2>/dev/null)"
    case "${tz:-}" in
        Asia/Shanghai|Asia/Chongqing|Asia/Harbin|Asia/Urumqi|PRC) USE_CN=1; return ;;
    esac
    if command -v curl >/dev/null 2>&1; then
        local cn int
        cn="$(curl -o /dev/null -s -w '%{time_connect}' --connect-timeout 3 "$CN_NPM" 2>/dev/null || echo 9)"
        int="$(curl -o /dev/null -s -w '%{time_connect}' --connect-timeout 3 https://registry.npmjs.org/ 2>/dev/null || echo 9)"
        if awk "BEGIN{exit !($int>0.5 && $cn<$int*0.6)}" 2>/dev/null; then
            USE_CN=1
        fi
    fi
}

npm_reg() {
    [[ "$USE_CN" == "1" ]] && echo "--registry $CN_NPM"
}

OPENCLAW_HOME="$HOME/.taylorissue/app"
OPENCLAW_BIN="$OPENCLAW_HOME/node_modules/.bin"

persist_openclaw_bin() {
    case ":$PATH:" in
        *":${OPENCLAW_BIN}:"*) return ;;
    esac
    export PATH="${OPENCLAW_BIN}:$PATH"
    local rc=""
    [[ -f "$HOME/.zshrc" ]] && rc="$HOME/.zshrc"
    [[ -z "$rc" && -f "$HOME/.bashrc" ]] && rc="$HOME/.bashrc"
    if [[ -n "$rc" ]]; then
        if ! grep -qF "$OPENCLAW_BIN" "$rc" 2>/dev/null; then
            printf '\nexport PATH="%s:$PATH"\n' "$OPENCLAW_BIN" >> "$rc"
            info "Added $OPENCLAW_BIN to $rc"
        fi
    fi
}

install_openclaw() {
    local spec="openclaw@latest"
    local resolved
    resolved="$(npm view $spec version $(npm_reg) 2>/dev/null || true)"
    [[ -n "$resolved" ]] && info "Target: OpenClaw v${resolved}"

    mkdir -p "$OPENCLAW_HOME"
    [[ ! -f "$OPENCLAW_HOME/package.json" ]] && echo '{"private":true}' > "$OPENCLAW_HOME/package.json"

    info "Installing OpenClaw to $OPENCLAW_HOME (no admin required)"
    npm cache clean --force >/dev/null 2>&1 || true
    local log; log="$(mktemp)"
    local cmd=(env SHARP_IGNORE_GLOBAL_LIBVIPS=1 npm --loglevel error --no-fund --no-audit install "$spec" --prefix "$OPENCLAW_HOME")
    [[ "$USE_CN" == "1" ]] && cmd+=(--registry "$CN_NPM")

    if "${cmd[@]}" >"$log" 2>&1; then
        persist_openclaw_bin
        ok "OpenClaw installed"
        rm -f "$log"
        return
    fi

    warn "First attempt failed — cleaning and retrying"
    rm -rf "$OPENCLAW_HOME/node_modules/openclaw" "$OPENCLAW_HOME/node_modules/.openclaw-"* 2>/dev/null || true

    if "${cmd[@]}" >"$log" 2>&1; then
        persist_openclaw_bin
        ok "OpenClaw installed (retry succeeded)"
        rm -f "$log"
        return
    fi

    echo ""
    warn "npm install failed. Last 40 lines:"
    tail -40 "$log" >&2
    echo ""
    die "Could not install OpenClaw. See log above."
}

verify_openclaw() {
    hash -r 2>/dev/null || true
    [[ -d "$OPENCLAW_BIN" ]] && export PATH="${OPENCLAW_BIN}:$PATH"

    if command -v openclaw >/dev/null 2>&1; then
        local ver; ver="$(openclaw --version 2>/dev/null | head -1 || echo unknown)"
        ok "${BOLD}OpenClaw installed successfully (${ver})${NC}"
    else
        warn "Installed, but 'openclaw' not on PATH"
        info "Try: hash -r  (or open a new terminal)"
        info "Binary should be at: $OPENCLAW_BIN"
    fi
}

main() {
    echo -e "${BOLD}  Install OpenClaw${NC}"
    detect_cn
    [[ "$USE_CN" == "1" ]] && ok "China mainland detected — npm mirror: $CN_NPM"
    install_openclaw
    verify_openclaw
}

main "$@"
