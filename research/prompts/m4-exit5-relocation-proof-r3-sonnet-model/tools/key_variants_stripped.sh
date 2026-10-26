#!/usr/bin/env bash
# 判器闭包键在各种改法下变不变：原键（现行算法）与「去掉规格与证明块」之后的键（strip_verus.py 原型）并排。
# 用法：key_variants_stripped.sh <工作目录> <仓根> <冻结副本> <strip_verus.py>
set -u
W=$1 REPO=$2 FROZEN=$3 STRIP=$4
T="$W/keytree"; rm -rf "$T"; mkdir -p "$T"; rsync -a --exclude target --exclude .git "$FROZEN/" "$T/"
FILE="$T/crates/singlefs-core/src/allocation_record_tree.rs"
ORIG='fn key_bytes_width() -> usize {
    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect("10")
}'
key() { python3 -I "$REPO/scripts/gate/lib_function_units.py" key "$T" singlefs_checker_tier::crash::judge_crash_image 2>&1 | grep -E '^(key|refused)' | head -1 | cut -f2; }
stripped_key() { cp "$FILE" "$FILE.keep"; python3 -I "$STRIP" "$FILE.keep" > "$FILE"; local k; k=$(key); mv "$FILE.keep" "$FILE"; echo "$k"; }
apply() { python3 -I - "$FILE" "$ORIG" "$1" <<'PY'
import sys
p, old, new = sys.argv[1:4]; t = open(p).read()
assert t.count(old) == 1
open(p, "w").write(t.replace(old, new))
PY
}
report() { echo "name=judge-key variant=$1 raw=$(key) stripped=$(stripped_key)"; cp "$FROZEN/crates/singlefs-core/src/allocation_record_tree.rs" "$FILE"; }
cp "$FROZEN/crates/singlefs-core/src/allocation_record_tree.rs" "$FILE"
report frozen
apply '// 只加一行注释。
fn key_bytes_width() -> usize {
    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect("10")
}'; report comment-only
apply 'verus! {
fn key_bytes_width() -> usize {
    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect("10")
}
}'; report wrapped-in-verus-without-ghost
apply 'verus! {
fn key_bytes_width() -> (r: usize)
    ensures r == 10,
{
    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect("10")
}
}'; report wrapped-with-an-ensures
apply 'verus! {
fn key_bytes_width() -> (r: usize)
    ensures r == 10,
{
    proof { assert(true); }
    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect("10")
}
}'; report wrapped-with-an-ensures-and-a-proof-block
apply 'verus! {
spec fn width_spec() -> int { 10 }
proof fn lemma_width() ensures width_spec() == 10 { }
fn key_bytes_width() -> (r: usize)
    requires true,
    ensures r == width_spec(),
{
    proof { lemma_width(); }
    let ghost g = width_spec();
    assert(g == 10);
    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect("10")
}
}'; report wrapped-with-spec-fn-proof-fn-ghost-let-and-assert
apply 'verus! {
fn key_bytes_width() -> (r: usize)
    ensures r == 10,
{
    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect("11")
}
}'; report must-red-exec-string-changed
apply 'verus! {
fn key_bytes_width() -> (r: usize)
    ensures r == 10,
{
    usize::try_from(ALLOCATION_RECORD_KEY_BYTES + 1).expect("10")
}
}'; report must-red-exec-expression-changed
apply 'verus! {
#[inline(never)]
fn key_bytes_width() -> (r: usize)
    ensures r == 10,
{
    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect("10")
}
}'; report must-red-exec-attribute-added
rm -rf "$T"
