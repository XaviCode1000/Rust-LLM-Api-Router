#!/usr/bin/env sh
# install.sh - LLM API Router Installer
# Usage: curl -sS https://raw.githubusercontent.com/XaviCode1000/Rust-LLM-Api-Router/main/install/install.sh | sh
#
# Supports:
#   - Linux (x86_64, aarch64)
#   - macOS (x86_64, arm64)
#   - WSL (Windows Subsystem for Linux)

set -eu

# Configuration
REPO="XaviCode1000/Rust-LLM-Api-Router"
BIN_NAME="llm-router"
DEFAULT_BIN_DIR="${HOME}/.local/bin"

# Colors (if terminal supports)
if [ -t 1 ]; then
    BOLD="$(tput bold 2>/dev/null || printf '')"
    RESET="$(tput sgr0 2>/dev/null || printf '')"
    GREEN="${BOLD}✓${RESET}"
    RED="${BOLD}✗${RESET}"
    YELLOW="${BOLD}!${RESET}"
else
    BOLD=""
    RESET=""
    GREEN="[OK]"
    RED="[FAIL]"
    YELLOW="[WARN]"
fi

# Utility functions
info() {
    printf '%s\n' "${GREEN} $*"
}

warn() {
    printf '%s %s\n' "${YELLOW}" "$*" >&2
}

error() {
    printf '%s %s\n' "${RED}" "$*" >&2
}

confirm() {
    printf '%s %s [y/N] ' "$*"
    read -r reply </dev/tty 2>/dev/null || reply=""
    case "$reply" in
        [yY][eE][sS]|[yY]) return 0 ;;
        *) return 1 ;;
    esac
}

# Platform detection
detect_platform() {
    case "$(uname -s)" in
        Linux)
            if grep -qiE 'microsoft|WSL' /proc/version 2>/dev/null; then
                printf 'linux-wsl'
            else
                printf 'linux'
            fi
            ;;
        Darwin)  printf 'macos' ;;
        FreeBSD) printf 'freebsd' ;;
        *)      printf 'linux' ;;
    esac
}

detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)   printf 'x86_64' ;;
        aarch64|arm64)   printf 'aarch64' ;;
        arm*|ARM*)      printf 'arm' ;;
        *)             printf 'x86_64' ;;
    esac
}

# Check prerequisites
check_prereqs() {
    local missing=""

    for cmd in curl tar; do
        if ! command -v "$cmd" >/dev/null 2>&1; then
            missing="${missing} ${cmd}"
        fi
    done

    if [ -n "$missing" ]; then
        error "Missing required commands:${missing}"
        error "Please install them and try again."
        exit 1
    fi
}

# Get latest version
get_latest_version() {
    # Try GitHub API first
    local version
    version=$(curl -sf "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | \
        sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -n1)

    if [ -n "$version" ]; then
        printf '%s' "$version"
        return
    fi

    # Fallback to parsing HTML
    printf 'latest'
}

# Get download URL
get_download_url() {
    local platform="$1"
    local arch="$2"

    case "${platform}" in
        linux|linux-wsl)
            case "${arch}" in
                x86_64)   printf "https://github.com/${REPO}/releases/latest/download/${BIN_NAME}-x86_64-unknown-linux.tar.gz" ;;
                aarch64)  printf "https://github.com/${REPO}/releases/latest/download/${BIN_NAME}-aarch64-unknown-linux.tar.gz" ;;
                *)       error "Unsupported architecture: ${arch}" && exit 1 ;;
            esac
            ;;
        macos)
            case "${arch}" in
                x86_64)   printf "https://github.com/${REPO}/releases/latest/download/${BIN_NAME}-x86_64-apple-darwin.tar.gz" ;;
                aarch64)  printf "https://github.com/${REPO}/releases/latest/download/${BIN_NAME}-aarch64-apple-darwin.tar.gz" ;;
                *)       error "Unsupported architecture: ${arch}" && exit 1 ;;
            esac
            ;;
        *)
            error "Unsupported platform: ${platform}"
            exit 1
            ;;
    esac
}

# Check if directory is writable
test_writeable() {
    touch "${1}/.test-write" 2>/dev/null && rm "${1}/.test-write"
}

# Install the binary
do_install() {
    local bin_dir="$1"
    local use_sudo=""

    if test_writeable "${bin_dir}"; then
        use_sudo=""
    else
        if ! test -w "$(dirname "$bin_dir")"; then
            error "Cannot write to ${bin_dir} or its parent directory."
            error "Try running with sudo or choose a different directory."
            exit 1
        fi
        if [ "${bin_dir}" = "/usr/local/bin" ] || [ "${bin_dir}" = "/usr/bin" ]; then
            warn "Elevated permissions required to install to ${bin_dir}"
            if ! confirm "Continue with sudo?"; then
                info "Installation cancelled."
                exit 0
            fi
            use_sudo="sudo"
        fi
    fi

    info "Downloading LLM API Router..."
    local tmp_file
    tmp_file=$(mktemp)

    # Download archive
    if ! curl -sfL --progress-bar -o "${tmp_file}" "${DOWNLOAD_URL}"; then
        error "Download failed. Please check your internet connection."
        error "URL: ${DOWNLOAD_URL}"
        rm -f "${tmp_file}"
        exit 1
    fi

    # Extract and install
    info "Installing to ${bin_dir}..."

    # For .tar.gz format
    if tar -tzf "${tmp_file}" >/dev/null 2>&1; then
        ${use_sudo} tar -xzf "${tmp_file}" -C "${bin_dir}" "${BIN_NAME}" 2>/dev/null || \
            ${use_sudo} tar -xzf "${tmp_file}" -C /tmp && \
            ${use_sudo} mv "/tmp/${BIN_NAME}" "${bin_dir}/${BIN_NAME}"
    else
        # Try as standalone binary (no archive)
        ${use_sudo} mv "${tmp_file}" "${bin_dir}/${BIN_NAME}"
    fi

    ${use_sudo} chmod +x "${bin_dir}/${BIN_NAME}"
    rm -f "${tmp_file}"

    info "Installed ${BIN_NAME} to ${bin_dir}"

    # Check if bin_dir is in PATH
    case ":${PATH}:" in
        *":${bin_dir}:"*) ;;
        *)
            warn "${bin_dir} is not in your PATH"
            printf '\n'
            info "Add it to your shell config:"
            printf '\n'
            case "${SHELL:-sh}" in
                *zsh*)  printf '  echo "export PATH=\${PATH}:%s" >> ~/.zshenv\n' "$bin_dir" ;;
                *bash*) printf '  echo "export PATH=\${PATH}:%s" >> ~/.bashrc\n' "$bin_dir" ;;
                *fish*) printf '  set -gx PATH %s \$PATH\n' "$bin_dir" ;;
                *)     printf '  export PATH=%s:\$PATH\n' "$bin_dir" ;;
            esac
            printf '\n'
            ;;
    esac
}

# Main
main() {
    printf '\n'
    info "LLM API Router Installer"
    printf '\n'

    # Check prerequisites
    check_prereqs

    # Detect platform
    PLATFORM=$(detect_platform)
    ARCH=$(detect_arch)

    # Determine download URL
    DOWNLOAD_URL=$(get_download_url "$PLATFORM" "$ARCH")

    # Print detected config
    info "Platform: ${PLATFORM}"
    info "Architecture: ${ARCH}"
    info "Download URL: ${DOWNLOAD_URL}"
    printf '\n'

    # Ask for install location
    DEFAULT_BIN_DIR=""
    if [ -w "${HOME}/.local/bin" ] || [ -w "${HOME}/.cargo/bin" ] 2>/dev/null; then
        if [ -d "${HOME}/.cargo/bin" ]; then
            DEFAULT_BIN_DIR="${HOME}/.cargo/bin"
        else
            DEFAULT_BIN_DIR="${HOME}/.local/bin"
        fi
    else
        DEFAULT_BIN_DIR="${HOME}/.local/bin"
    fi

    printf '\n'
    info "Installation directory: ${DEFAULT_BIN_DIR}"
    printf '\n'
    info "For system-wide installation, use: /usr/local/bin"
    printf '\n'

    # Ask for confirmation
    printf "Install to ${DEFAULT_BIN_DIR}? "
    if ! confirm "[Y/n]"; then
        printf '\n'
        printf "Enter custom directory: "
        read -r DEFAULT_BIN_DIR </dev/tty 2>/dev/null || DEFAULT_BIN_DIR=""
        [ -z "${DEFAULT_BIN_DIR}" ] && exit 0
    fi

    # Create directory if needed
    if [ ! -d "${DEFAULT_BIN_DIR}" ]; then
        mkdir -p "${DEFAULT_BIN_DIR}"
    fi

    printf '\n'

    # Install
    do_install "${DEFAULT_BIN_DIR}"

    printf '\n'
    info "Installation complete!"
    printf '\n'
    info "Run '${BIN_NAME} --help' to get started."
    printf '\n'
}

# Parse arguments
while [ "$#" -gt 0 ]; do
    case "$1" in
        -b|--bin-dir)
            DEFAULT_BIN_DIR="$2"
            shift 2
            ;;
        -p|--platform)
            PLATFORM="$2"
            shift 2
            ;;
        -a|--arch)
            ARCH="$2"
            shift 2
            ;;
        -V|--version)
            printf 'install.sh version 1.0.0\n'
            exit 0
            ;;
        -h|--help)
            printf 'LLM API Router Installer\n'
            printf '\n'
            printf 'Usage: curl -sS https://your-url/install.sh | sh [OPTIONS]\n'
            printf '\n'
            printf 'Options:\n'
            printf '  -b, --bin-dir DIR    Install to DIR (default: ~/.local/bin or ~/.cargo/bin)\n'
            printf '  -p, --platform OS  Override platform (linux, macos, linux-wsl)\n'
            printf '  -a, --arch ARCH   Override architecture (x86_64, aarch64)\n'
            printf '  -V, --version     Show version\n'
            printf '  -h, --help        Show this help\n'
            exit 0
            ;;
        *)
            shift
            ;;
    esac
done

main