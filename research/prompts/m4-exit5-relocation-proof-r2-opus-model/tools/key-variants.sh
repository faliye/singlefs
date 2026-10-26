#!/usr/bin/env bash
# judge_crash_image 闭包键在四种改法下变不变：被改的是 allocation_record_tree::key_bytes_width（发布、回退两条闭包与判器闭包都含它）。
set -u
W=$1 REPO=$2 FROZEN=$3
T="$W/key-tree"; rm -rf "$T"; mkdir -p "$T"; rsync -a --exclude target --exclude .git "$FROZEN/" "$T/"
FILE="$T/crates/singlefs-core/src/allocation_record_tree.rs"
ORIG='fn key_bytes_width() -> usize {
    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect("10")
}'
key() { python3 "$REPO/scripts/gate/lib_function_units.py" key "$T" singlefs_checker_tier::crash::judge_crash_image 2>&1 | grep -E '^(key|refused)' | head -1 | cut -f2; }
apply() { python3 - "$FILE" "$ORIG" "$1" <<'PY'
import sys
p, old, new = sys.argv[1:4]; t = open(p).read()
assert t.count(old) == 1
open(p, "w").write(t.replace(old, new))
PY
}
cp "$FROZEN/crates/singlefs-core/src/allocation_record_tree.rs" "$FILE"; echo "name=judge-key variant=frozen key=$(key)"
apply '// 只加一行注释。
fn key_bytes_width() -> usize {
    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect("10")
}'; echo "name=judge-key variant=comment-only key=$(key)"
cp "$FROZEN/crates/singlefs-core/src/allocation_record_tree.rs" "$FILE"
apply 'verus! {
fn key_bytes_width() -> usize {
    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect("10")
}
}'; echo "name=judge-key variant=wrapped-in-verus-without-ghost key=$(key)"
cp "$FROZEN/crates/singlefs-core/src/allocation_record_tree.rs" "$FILE"
apply 'verus! {
fn key_bytes_width() -> (r: usize)
    ensures r == 10,
{
    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect("10")
}
}'; echo "name=judge-key variant=wrapped-with-an-ensures key=$(key)"
cp "$FROZEN/crates/singlefs-core/src/allocation_record_tree.rs" "$FILE"
apply 'verus! {
fn key_bytes_width() -> (r: usize)
    ensures r == 10,
{
    proof { assert(true); }
    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect("10")
}
}'; echo "name=judge-key variant=wrapped-with-an-ensures-and-a-proof-block key=$(key)"
rm -rf "$T"
