#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd -P)
minimum_free_kib=5242880
available_kib=$(df -Pk "$repo_root" | awk 'NR == 2 { print $4 }')

case "$available_kib" in
    ''|*[!0-9]*)
        echo "[ci-gitlab-unix] Could not determine free space for $repo_root." >&2
        exit 1
        ;;
esac

if [ "$available_kib" -lt "$minimum_free_kib" ]; then
    echo "[ci-gitlab-unix] Insufficient free space for $repo_root: ${available_kib} KiB available; ${minimum_free_kib} KiB required." >&2
    exit 1
fi

scratch_parent="$repo_root/.tmp/gitlab-ci/unix"
scratch_path="$scratch_parent/run-${CI_JOB_ID:-local}-$$"
mkdir -p -- "$scratch_path"

cleanup() {
    case "$scratch_path" in
        "$scratch_parent"/run-*)
            rm -rf -- "$scratch_path"
            ;;
        *)
            echo "[ci-gitlab-unix] Refusing to clean unexpected path: $scratch_path" >&2
            exit 1
            ;;
    esac
}
trap cleanup EXIT HUP INT TERM

export TEMP="$scratch_path"
export TMP="$scratch_path"
export TMPDIR="$scratch_path"
export RUST_BACKTRACE=1
export RUSTFLAGS="-D warnings"

cd -- "$repo_root"

command -v cargo >/dev/null 2>&1 || {
    echo "[ci-gitlab-unix] cargo is not provisioned on this runner." >&2
    exit 1
}
command -v rustc >/dev/null 2>&1 || {
    echo "[ci-gitlab-unix] rustc is not provisioned on this runner." >&2
    exit 1
}
if command -v python3 >/dev/null 2>&1; then
    python_cmd=python3
elif command -v python >/dev/null 2>&1; then
    python_cmd=python
else
    echo "[ci-gitlab-unix] python is not provisioned on this runner." >&2
    exit 1
fi

if [ -n "${RUSTC_WRAPPER:-}" ]; then
    echo "[ci-gitlab-unix] disabling RUSTC_WRAPPER to avoid shell-runner sccache socket path limits"
    unset RUSTC_WRAPPER
fi

rustc -Vv
cargo -V

"$python_cmd" scripts/hygiene.py
"$python_cmd" scripts/check_code_size.py
"$python_cmd" scripts/check_duplicate_dependencies.py
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo build --all-targets --all-features --locked
cargo test --all-targets --all-features --locked
