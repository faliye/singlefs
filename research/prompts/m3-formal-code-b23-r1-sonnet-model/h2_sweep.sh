#!/usr/bin/env bash
# 第 23 批代码轮 K3：H2 两条用例（挂载次数从根环槽数算之后）在别的根环每区槽数 S 下的前提断言与落点断言。
# 用法：WS=<副本根> FROZEN=<冻结副本的 tree 根> TARGET=<编译目录> h2_sweep.sh <S> <orig|slack1>
#   S      把副本里 singlefs-format 的 ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM 改成这个数（合法范围 4..=16）。
#   slack1 再把 singlefs-core 的回收谓词放宽一个 txg（b21 判决里 21-1 那行变异：`record.generation <= floor` → `record.generation.0 <= floor.0 + 1`）。
# 每次先从冻结副本还原这两个文件，再改；只跑那两条用例（名字过滤），经内存包装与线程上限。
set -euo pipefail
S="$1"; VARIANT="$2"
: "${WS:?}" "${FROZEN:?}" "${TARGET:?}"
cp "$FROZEN/crates/singlefs-format/src/lib.rs" "$WS/crates/singlefs-format/src/lib.rs"
cp "$FROZEN/crates/singlefs-core/src/allocator.rs" "$WS/crates/singlefs-core/src/allocator.rs"
python3 - "$WS" "$S" "$VARIANT" <<'PY'
import sys
ws, s, variant = sys.argv[1], sys.argv[2], sys.argv[3]
p = ws + "/crates/singlefs-format/src/lib.rs"
t = open(p).read()
old = "pub const ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = 8;"
assert t.count(old) == 1
open(p, "w").write(t.replace(old, f"pub const ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = {s};"))
if variant == "slack1":
    p = ws + "/crates/singlefs-core/src/allocator.rs"
    t = open(p).read()
    old = ".filter(|record| record.is_released && record.generation <= floor)"
    assert t.count(old) == 1, t.count(old)
    open(p, "w").write(t.replace(old, ".filter(|record| record.is_released && record.generation.0 <= floor.0 + 1)"))
PY
cd "$WS"
echo "=== S=$S variant=$VARIANT ==="
set +e
CARGO_TARGET_DIR="$TARGET" bash /home/fy5090/code/singlefs/scripts/hosts/capped.sh 7 cargo test --offline -p singlefs-harness \
  --test pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement \
  -- the_first_file_after_ten_warm_up_only_mounts the_first_file_after_nine_warm_up_only_mounts 2>&1 \
  | grep -E '^test |test result|panicked|left:|right:|前提|error(\[|:)' | cut -c1-420
echo "rc(pipeline)=${PIPESTATUS[0]}"
