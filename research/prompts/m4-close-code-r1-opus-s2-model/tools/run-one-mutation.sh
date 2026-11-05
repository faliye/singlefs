#!/usr/bin/env bash
# 攻方腿 s2：改后那棵树上打一条变异，跑 core 的库测试，再跑容器小树差分，最后还原。参数：草稿目录 冻结副本根 变异名。
set -uo pipefail
work="$1"
frozen="$2"
name="$3"
python3 "$work/model/tools/apply-mutation.py" "$work" "$frozen" "$name"
cd "$work/new"
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=cargo test -p singlefs-core 的测试目标 --lib; 范围=1 个测试目标; 为什么=草稿副本上打一条变异之后看 core 库测试红不红（T10 取样点）' CARGO_TARGET_DIR="$work/target-new" cargo test --offline -p singlefs-core --lib > "$work/mutation-$name-lib.txt" 2>&1
echo "lib rc=$?"
grep -E '^test result|^    [a-z_:]+$' "$work/mutation-$name-lib.txt" | head -n 12
bash "$work/model/tools/differential-container-trees.sh" "$work" > "$work/mutation-$name-differential.txt" 2>&1
grep -E 'differential:|binary rc' "$work/mutation-$name-differential.txt"
for side in old new; do grep -v -E '^[0-9]+ (shape|shape_facts|shape_at|ancestors|ancestors_skipped|opened) ' "$work/out-tree-$side.txt" | cut -d' ' -f2- > "$work/tree-rest-$side.txt"; done
if cmp -s "$work/tree-rest-old.txt" "$work/tree-rest-new.txt"; then echo "publish-sequence differential: identical"; else echo "publish-sequence differential: differ"; fi
python3 "$work/model/tools/apply-mutation.py" "$work" "$frozen" --restore
