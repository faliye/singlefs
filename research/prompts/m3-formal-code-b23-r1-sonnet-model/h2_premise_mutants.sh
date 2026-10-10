#!/usr/bin/env bash
# 第 23 批代码轮 K3：H2 两条用例的前提断言有没有判别力——把挂载次数改错一次，看红在前提断言还是落点断言。
# 用法：WS=<副本根> FROZEN=<冻结副本 tree 根> TARGET=<编译目录> h2_premise_mutants.sh <S> <ten-minus-one|nine-plus-one>
#   ten-minus-one：「回收了」那条用例把挂载次数改成 S + 1（少一次）；
#   nine-plus-one：「差一个 txg」那条用例把挂载次数改成 S + 2（多一次）。
set -euo pipefail
S="$1"; MUTANT="$2"
: "${WS:?}" "${FROZEN:?}" "${TARGET:?}"
T=crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs
cp "$FROZEN/crates/singlefs-format/src/lib.rs" "$WS/crates/singlefs-format/src/lib.rs"
cp "$FROZEN/$T" "$WS/$T"
python3 - "$WS" "$S" "$MUTANT" "$T" <<'PY'
import sys
ws, s, mutant, t = sys.argv[1:5]
p = ws + "/crates/singlefs-format/src/lib.rs"
x = open(p).read()
old = "pub const ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = 8;"
assert x.count(old) == 1
open(p, "w").write(x.replace(old, f"pub const ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = {s};"))
p = ws + "/" + t
x = open(p).read()
if mutant == "ten-minus-one":
    old = "    ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM + 2;\n"
    new = "    ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM + 1;\n"
    assert x.count(old) == 1
    open(p, "w").write(x.replace(old, new))
else:
    old = "    WARM_UP_ONLY_MOUNTS_UNTIL_THE_MKFS_INSTANCE_TABLE_IS_RECLAIMED - 1;\n"
    new = "    WARM_UP_ONLY_MOUNTS_UNTIL_THE_MKFS_INSTANCE_TABLE_IS_RECLAIMED;\n"
    assert x.count(old) == 1
    open(p, "w").write(x.replace(old, new))
PY
cd "$WS"
echo "=== S=$S mutant=$MUTANT ==="
set +e
CARGO_TARGET_DIR="$TARGET" bash /home/fy5090/code/singlefs/research/scripts/capped.sh 7 cargo test --offline -p singlefs-harness \
  --test pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement \
  -- the_first_file_after_ten_warm_up_only_mounts the_first_file_after_nine_warm_up_only_mounts 2>&1 \
  | grep -E '^test |test result|panicked|left:|right:|前提|error(\[|:)' | cut -c1-520
echo "rc(pipeline)=${PIPESTATUS[0]}"
