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

CN_BREW_API="https://mirrors.ustc.edu.cn/homebrew-bottles/api"
CN_BREW_BOTTLE="https://mirrors.ustc.edu.cn/homebrew-bottles"
CN_BREW_GIT="https://mirrors.ustc.edu.cn/brew.git"
CN_BREW_CORE="https://mirrors.ustc.edu.cn/homebrew-core.git"
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
        cn="$(curl -o /dev/null -s -w '%{time_connect}' --connect-timeout 3 "https://registry.npmmirror.com" 2>/dev/null || echo 9)"
        int="$(curl -o /dev/null -s -w '%{time_connect}' --connect-timeout 3 https://registry.npmjs.org/ 2>/dev/null || echo 9)"
        if awk "BEGIN{exit !($int>0.5 && $cn<$int*0.6)}" 2>/dev/null; then
            USE_CN=1
        fi
    fi
}

apply_cn_brew() {
    [[ "$USE_CN" != "1" ]] && return
    ok "China mainland detected — Homebrew mirrors enabled"
    export HOMEBREW_API_DOMAIN="$CN_BREW_API"
    export HOMEBREW_BOTTLE_DOMAIN="$CN_BREW_BOTTLE"
    export HOMEBREW_BREW_GIT_REMOTE="$CN_BREW_GIT"
    export HOMEBREW_CORE_GIT_REMOTE="$CN_BREW_CORE"
}

git_works() {
    command -v git >/dev/null 2>&1 || return 1
    git --version >/dev/null 2>&1
}

ensure_brew() {
    [[ "$OSTYPE" != darwin* ]] && return
    if command -v brew >/dev/null 2>&1; then
        ok "Homebrew ready"
        return
    fi
    if [[ -x /opt/homebrew/bin/brew ]]; then
        eval "$(/opt/homebrew/bin/brew shellenv)"; ok "Homebrew ready"; return
    fi
    if [[ -x /usr/local/bin/brew ]]; then
        eval "$(/usr/local/bin/brew shellenv)"; ok "Homebrew ready"; return
    fi
    info "Installing Homebrew"
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    [[ -x /opt/homebrew/bin/brew ]] && eval "$(/opt/homebrew/bin/brew shellenv)"
    [[ -x /usr/local/bin/brew ]] && eval "$(/usr/local/bin/brew shellenv)"
    ok "Homebrew installed"
}

ensure_git() {
    if git_works; then
        ok "Git $(git --version | awk '{print $3}')"
        return
    fi
    info "Git missing or broken — installing"
    if [[ "$OSTYPE" == darwin* ]]; then
        brew install git
    else
        if [[ "$(id -u)" -eq 0 ]]; then
            local SUDO=""
        elif sudo -n true 2>/dev/null; then
            local SUDO="sudo -n"
        else
            die "Root or passwordless sudo required to install git on Linux"
        fi
        if command -v apt-get >/dev/null 2>&1; then
            export DEBIAN_FRONTEND=noninteractive
            $SUDO apt-get update -qq && $SUDO apt-get install -y -qq git
        elif command -v dnf >/dev/null 2>&1; then
            $SUDO dnf install -y -q git
        elif command -v yum >/dev/null 2>&1; then
            $SUDO yum install -y -q git
        else
            die "Cannot install git — no known package manager"
        fi
    fi
    hash -r 2>/dev/null || true
    git_works || die "Git still not working after install"
    ok "Git $(git --version | awk '{print $3}')"
}

main() {
    echo -e "${BOLD}  Install Git${NC}"
    export NONINTERACTIVE=1
    export HOMEBREW_NO_INSTALL_CLEANUP=1
    detect_cn
    apply_cn_brew
    ensure_brew
    ensure_git
}

main "$@"
