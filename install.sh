#!/bin/bash

set -eu

function get_os() {
    local os=$(uname -s)

    case "${os}" in
    Linux)
        echo "linux"
        ;;

    FreeBSD)
        echo "freebsd"
        ;;

    Darwin)
        echo "darwin"
        ;;

    MINGW* | MSYS* | CYGWIN*)
        echo "windows"
        ;;

    *)
        printf "goup: unrecognized OS: $os" >&2
        exit 1
        ;;
    esac
}

function get_arch() {
    local os=$(uname -s)
    local arch=$(uname -m)
    case ${arch} in
    "x86_64" | "x86-64" | "x64" | "amd64")
        echo "x86_64"
        ;;
    "i386" | "i486" | "i586" | "i686" | "i786" | "x86")
        echo "i386"
        ;;
    "aarch64" | "arm64")
        echo "aarch64"
        ;;
    "xscale" | "arm" | "armv6l" | "armv7l" | "armv8l")
        echo "arm"
        ;;
    "s390x")
        echo "s390x"
        ;;
    *)
        printf "goup: unrecognized ARCH: $arch" >&2
        exit 1
        ;;
    esac
}

function get_dld() {
    local dld
    if check_cmd curl; then
        dld=curl
    elif check_cmd wget; then
        dld=wget
    else
        printf "goup: need curl or wget" >&2
        exit 1
    fi
    echo $dld
}

need_cmd() {
    if ! check_cmd "$1"; then
        printf "goup: need '$1' (command not found)" >&2
        exit 1
    fi
}

check_cmd() {
    command -v "$1" >/dev/null 2>&1
}

# Run a command that should never fail. If the command fails execution
# will immediately terminate with an error showing the failing
# command.
ensure() {
    if ! "$@"; then
        err "command failed: $*"
        exit 1
    fi
}

# This is just for indicating that commands' results are being
# intentionally ignored. Usually, because it's being executed
# as part of error handling.
ignore() {
    "$@"
}

function get_target() {
    local os=$(get_os)
    local arch=$(get_arch)

    case "${os}" in
    linux)
        echo "goup-${arch}-unknown-${os}-musl.tar.gz"
        ;;
    freeBSD)
        echo "goup-${arch}-unknown-${os}.tar.gz"
        ;;
    darwin)
        echo "goup-${arch}-apple-${os}.tar.gz"
        ;;
    MINGW* | MSYS* | CYGWIN*)
        echo "goup-${arch}-pc-${os}-msvc.zip"
        ;;
    *)
        printf "goup: unrecognized OS and ARCH: $os $arch" >&2
        exit 1
        ;;
    esac
}

function main() {
    need_cmd uname
    need_cmd mktemp
    need_cmd mkdir
    need_cmd chmod
    need_cmd rm
    need_cmd rmdir

    local _target=$(get_target)
    local dld=$(get_dld)

    local _url="https://github.com/thinkgos/goup-rs/releases/latest/download/${_target}"
    local GOUP_HOME=${GOUP_HOME:-$HOME/.goup}
    local GOUP_BIN_DIR="${GOUP_HOME}/bin"
    local GOUP_BIN_FILE="${GOUP_HOME}/bin/goup"

    local _dir
    if ! _dir="$(ensure mktemp -d)"; then
        # Because the previous command ran in a subshell, we must manually
        # propagate exit status.
        exit 1
    fi
    local _target_file="${_dir}/${_target}"

    ensure mkdir -p ${GOUP_HOME}
    ensure mkdir -p ${GOUP_BIN_DIR}

    echo "[1/3] Download goup..."
    if [ "dld" = curl ]; then
        curl -s -S -f --progress-bar -L "${_url}" -o "${_target_file}"
    else
        wget -q "${_url}" -P "${_dir}"
    fi

    echo "[2/3] Install goup to the ${GOUP_BIN_DIR}"
    ensure tar -xz -f "${_target_file}" -C "${GOUP_BIN_DIR}"
    ensure chmod u+x "${GOUP_BIN_FILE}"
    ensure "${GOUP_BIN_FILE}" init

    ignore rm "$_target_file"
    ignore rmdir "$_dir"

    echo "[3/3] Please add '. "${GOUP_HOME}/env"' to your shell environment!!"
    echo "      And then try to run 'goup --version'"
}

main
