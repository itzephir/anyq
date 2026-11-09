#!/bin/sh
# Install an official anyq release without changing queues, config or shell files.
set -eu

die() { printf 'anyq install: %s\n' "$*" >&2; exit 1; }
version=latest
scope=system
bin_dir=${ANYQ_BIN_DIR:-}
release_base=${ANYQ_RELEASE_BASE_URL:-https://github.com/itzephir/anyq/releases}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --user) scope=user; shift ;;
        --system) scope=system; shift ;;
        --version|--bin-dir)
            [ "$#" -ge 2 ] || die "Missing value for $1"
            case "$1" in --version) version=$2 ;; --bin-dir) bin_dir=$2 ;; esac
            shift 2 ;;
        -h|--help)
            printf '%s\n' 'Usage: ./install-anyq [--system|--user] [--version v0.1.0] [--bin-dir PATH]'
            exit 0 ;;
        *) die "Unknown argument: $1" ;;
    esac
done

if [ -z "$bin_dir" ]; then
    case "$scope" in
        system) bin_dir=/usr/local/bin ;;
        user) bin_dir=$HOME/.local/bin ;;
    esac
fi
case "$bin_dir" in /*) ;; *) bin_dir=$PWD/$bin_dir ;; esac

case "$release_base" in
    https://*) protocols='=https' ;;
    http://127.0.0.1:*|http://localhost:*) protocols='=http' ;;
    *) die 'Release URL must use HTTPS (HTTP is allowed only for loopback tests)' ;;
esac
release_base=${release_base%/}
command -v curl >/dev/null 2>&1 || die 'curl is required'
command -v tar >/dev/null 2>&1 || die 'tar is required'
download() { curl --proto "$protocols" --proto-redir "$protocols" -fLsS --retry 2 "$@"; }

case "$(uname -s)" in
    Darwin) os=apple-darwin ;;
    Linux) os=unknown-linux-musl ;;
    *) die 'Use install.ps1 on Windows; supported Unix systems: macOS and Linux' ;;
esac
case "$(uname -m)" in
    arm64|aarch64) arch=aarch64 ;;
    x86_64|amd64) arch=x86_64 ;;
    *) die 'Supported architectures: x86_64 and ARM64' ;;
esac
asset=anyq-$arch-$os.tar.gz

if [ "$version" = latest ]; then
    resolved=$(download -o /dev/null -w '%{url_effective}' "$release_base/latest") || die 'No published release is available'
    version=${resolved##*/}
fi
case "$version" in *[!0-9A-Za-z.-]*) die 'Invalid release version' ;; esac
printf '%s\n' "$version" | LC_ALL=C grep -Eq '^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$' || die 'Expected a release version such as v0.1.0'

tmp_dir=$(mktemp -d)
cleanup() { rm -rf "$tmp_dir"; }
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

url=$release_base/download/$version
printf 'Downloading anyq %s for %s-%s\n' "$version" "$arch" "$os"
download "$url/$asset" -o "$tmp_dir/$asset" || die 'Binary download failed'
download "$url/SHA256SUMS" -o "$tmp_dir/SHA256SUMS" || die 'Checksum download failed'
expected=$(awk -v file="$asset" '$2 == file { hash=$1; count++ } END { if (count == 1) print hash }' "$tmp_dir/SHA256SUMS")
printf '%s\n' "$expected" | LC_ALL=C grep -Eq '^[0-9a-f]{64}$' || die 'Missing or ambiguous SHA-256 checksum'
if command -v sha256sum >/dev/null 2>&1; then
    actual=$(sha256sum "$tmp_dir/$asset" | awk '{print $1}')
elif command -v shasum >/dev/null 2>&1; then
    actual=$(shasum -a 256 "$tmp_dir/$asset" | awk '{print $1}')
else
    die 'sha256sum or shasum is required'
fi
[ "$actual" = "$expected" ] || die 'SHA-256 mismatch; the installed binary was preserved'
tar -xzf "$tmp_dir/$asset" -C "$tmp_dir" anyq
[ -f "$tmp_dir/anyq" ] && [ ! -L "$tmp_dir/anyq" ] || die 'Archive has no regular anyq binary'

chmod 755 "$tmp_dir/anyq"
reported=$("$tmp_dir/anyq" --version) || die 'The downloaded binary cannot run on this system'
[ "$reported" = "anyq ${version#v}" ] || die 'Binary version does not match the release'
if command -v sha256sum >/dev/null 2>&1; then
    binary_hash=$(sha256sum "$tmp_dir/anyq" | awk '{print $1}')
else
    binary_hash=$(shasum -a 256 "$tmp_dir/anyq" | awk '{print $1}')
fi

# This fixed worker never downloads or executes code with additional privileges.
placement=$(cat <<'PLACEMENT'
PATH=/usr/bin:/bin:/usr/sbin:/sbin
export PATH
umask 022
source=$1
bin_dir=$2
expected=$3
mkdir -p "$bin_dir"
bin_dir=$(CDPATH= cd "$bin_dir" && pwd)
[ ! -d "$bin_dir/anyq" ] || { echo 'Destination anyq is a directory' >&2; exit 1; }
staged=$(mktemp "$bin_dir/.anyq.XXXXXX")
trap 'rm -f "$staged"' EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
cp "$source" "$staged"
chmod 755 "$staged"
if command -v sha256sum >/dev/null 2>&1; then
    actual=$(sha256sum "$staged" | awk '{print $1}')
else
    actual=$(shasum -a 256 "$staged" | awk '{print $1}')
fi
[ "$actual" = "$expected" ] || { echo 'Staged binary changed; installed binary preserved' >&2; exit 1; }
mv -f "$staged" "$bin_dir/anyq"
PLACEMENT
)

probe=$bin_dir
while [ ! -e "$probe" ]; do probe=$(dirname "$probe"); done
[ -d "$probe" ] || die 'Installation path has a non-directory parent'
if [ -w "$probe" ]; then
    /bin/sh -eu -c "$placement" anyq-install "$tmp_dir/anyq" "$bin_dir" "$binary_hash" || die 'Installation failed; the previous binary was preserved'
else
    [ "$scope" = system ] || die 'User installation directory is not writable'
    command -v sudo >/dev/null 2>&1 || die 'System installation requires sudo or an administrator shell; --user installs without it'
    printf 'Administrator access is required to install in %s\n' "$bin_dir"
    sudo -- /bin/sh -eu -c "$placement" anyq-install "$tmp_dir/anyq" "$bin_dir" "$binary_hash" || die 'System installation was not approved or failed; the previous binary was preserved'
fi
bin_dir=$(CDPATH= cd "$bin_dir" && pwd)
printf 'Installed %s at %s/anyq\n' "$reported" "$bin_dir"
case ":$PATH:" in
    *":$bin_dir:"*) printf '%s\n' 'Run: anyq --help' ;;
    *) printf 'Add this directory to your shell PATH: %s\nThen run: anyq --help\n' "$bin_dir" ;;
esac
