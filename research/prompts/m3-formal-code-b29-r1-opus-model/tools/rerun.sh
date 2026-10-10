#!/usr/bin/env bash
# 复跑攻方腿（m3-formal-code-b29-r1-opus）的原型。用法：bash rerun.sh <草稿目录> <步>
# 步：prep（拷冻结副本、放三份原型测试）→ tombstone → baseline → kx1 → kx2 → extend → retain → controls → lib → broad-kx1
# 每一步单独起，单步挂钟（本机实测）：prep 约 10 秒；tombstone 约 20 秒；baseline 约 55 秒；kx1 / kx2 各约 60 秒；extend / retain 各约 40 秒；
# controls 约 30 秒；lib 约 70 秒；broad-kx1 约 115 秒。全部经内存包装 8G、线程上限 10。日志在 <草稿目录>/variants/。
# 判结果看每步打的 name=variant 行（rc= 与 test result）与缩进的 test … ok / FAILED 行，与 ../out/variants2-summary.txt 对。
set -u
DRAFT=${1:?草稿目录}; STEP=${2:?步}
HERE=$(cd "$(dirname "$0")" && pwd); MODEL=$(dirname "$HERE")
R=/home/fy5090/code/singlefs; FROZEN=/tmp/claude-1000/m3-formal-code-b29-r1/tree
TREE=$DRAFT/tree; TARGET=$DRAFT/target; LOGS=$DRAFT/variants
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-复跑攻方腿的原型与变体，与别的腿并行}"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="${RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON:-派发提示定 cargo 一律给 8G 上限}"
wrap() { nice -n 19 bash "$R/research/scripts/run-with-memory-cap.sh" 8G bash "$R/research/scripts/capped.sh" 10 "$@"; }
one() { wrap bash "$HERE/run_one_variant.sh" "$TREE" "$TARGET" "$LOGS" "$@"; }
T3=model_compares_inode_creations_into_the_third_inode_leaf_container
TR=model_compares_a_rollback_to_a_version_with_fewer_inode_leaf_containers
TS=b29_opus_leaf_container_sampling_points
case "$STEP" in
  prep)
    mkdir -p "$DRAFT"
    rsync -a --exclude target --exclude .git "$FROZEN/" "$TREE/"
    cp "$MODEL"/tests/b29_opus_*.rs "$TREE/crates/singlefs-harness/tests/" ;;
  tombstone) one baseline b29_opus_tombstone_container_pointer_head
    grep -E '^(clean|extent_offset_1|mac_byte0_1) (checker|recover|mount)' "$LOGS/baseline.b29_opus_tombstone_container_pointer_head.log" ;;
  baseline) one baseline "$T3" "$TR" "$TS" ;;
  kx1) one kx1_cap_at_third "$T3" "$TR" "$TS" ;;
  kx2) one kx2_cap_at_third "$T3" "$TR" "$TS" ;;
  extend) one extend_only_the_second_leaf "$TR" "$TS" ;;
  retain) one retain_drops_only_the_third_leaf "$TR" "$TS" ;;
  controls) one kx1_original_cap_at_second "$T3"; one drop_retain_original "$TR" ;;
  lib) for v in kx1_cap_at_third kx2_cap_at_third; do wrap bash "$HERE/run_one_variant_lib.sh" "$TREE" "$TARGET" "$LOGS" "$v"; done ;;
  broad-kx1) one kx1_cap_at_third unlink_in_histories_judged_by_the_model random_histories ;;
  *) echo "不认得的步：$STEP"; exit 2 ;;
esac
