#!/usr/bin/env bash
# 今天已验的核（singlefs-replay-prefix/src/lib.rs）里改一条引理的证明体（多一句 assert(true);），崩溃判器闭包键变不变。
# 用法：key_lemma_edit.sh <工作目录> <仓根> <冻结副本>
set -u
W=$1 REPO=$2 FROZEN=$3
python3 -I "$REPO/scripts/gate/lib_function_units.py" closure "$FROZEN" singlefs_checker_tier::crash::judge_crash_image > "$W/judge-closure.txt" 2>/dev/null
awk -F'\t' '$1=="fn" && $2 ~ /^singlefs_replay_prefix::/ {split($3,a,":"); split(a[2],b,"-"); n++; l+=b[2]-b[1]+1; if ($2 ~ /::(lemma_|witness_|theorem_)/) {p++; pl+=b[2]-b[1]+1}} END{printf "name=judge-closure-replay-prefix-units fn=%d lines=%d proof_fn=%d proof_lines=%d\n", n, l, p, pl}' "$W/judge-closure.txt"
echo "name=judge-closure-units-in-the-two-proof-only-files count=$(awk -F'\t' '$3 ~ /compaction_plan_proofs|gc_crash_proofs/' "$W/judge-closure.txt" | wc -l)"
T="$W/keytree2"; rm -rf "$T"; mkdir -p "$T"; rsync -a --exclude target --exclude .git "$FROZEN/" "$T/"
FILE="$T/crates/singlefs-replay-prefix/src/lib.rs"
key() { python3 -I "$REPO/scripts/gate/lib_function_units.py" key "$T" singlefs_checker_tier::crash::judge_crash_image 2>&1 | grep -E '^(key|refused)' | head -1 | cut -f2; }
echo "name=judge-key-lib-lemma-edit variant=frozen key=$(key)"
python3 -I - "$FILE" <<'PY'
import sys
p = sys.argv[1]; t = open(p).read()
old = "    let twice = applied_records.insert(position + 1, applied_records[position]);\n    if position == applied_records.len() - 1 {"
assert t.count(old) == 1, t.count(old)
open(p, "w").write(t.replace(old, "    let twice = applied_records.insert(position + 1, applied_records[position]);\n    assert(true);\n    if position == applied_records.len() - 1 {"))
PY
echo "name=judge-key-lib-lemma-edit variant=one-assert-true-added-in-lemma_applying_a_record_twice_in_a_row_is_applying_it_once key=$(key)"
rm -rf "$T"
