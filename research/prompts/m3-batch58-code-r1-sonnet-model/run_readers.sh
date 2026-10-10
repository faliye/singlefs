#!/usr/bin/env bash
set -uo pipefail
W="$HOME/.local/share/singlefs/work/m3-batch58-code-r1-sonnet"
python3 "$W/patch_readers.py" apply || exit 3
export CARGO_TARGET_DIR="$W/target"
cargo test --offline -p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking --manifest-path "$W/repo/Cargo.toml" -- "$@" > "$W/logs/readers-AB.log" 2>&1
code=$?
python3 "$W/patch_readers.py" restore
echo "cargo_exit=$code"
grep -E '^test .*(FAILED|ok)$|test result' "$W/logs/readers-AB.log"
