#!/usr/bin/env bash
# m3-batch4-code-r1 攻方腿模型的复跑：冻结副本拷一份、打 checker 的差分口子、放进原型测试、经内存包装跑。
# 用法：bash rerun.sh <空的草稿目录>。只跑 harness 档（singlefs-harness 的测试二进制），不碰 singlefs-checker-tier 的测试。
set -euo pipefail
REPO=/home/fy5090/code/singlefs
FROZEN=/tmp/claude-1000/m3-batch5-freeze/tree
WORK=${1:?给一个空的草稿目录}
MODEL=$(cd "$(dirname "$0")" && pwd)
rsync -a --exclude target --exclude .git "$FROZEN/" "$WORK/tree/"
cd "$WORK/tree"
patch -p1 < "$MODEL/walk-differential-hook.patch"
cp "$MODEL/attack_b4_opus.rs" crates/singlefs-harness/tests/attack_b4_opus.rs
run() {
  RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="原型每次跑的状态数不同，峰值随之变" \
    nice -n 19 bash "$REPO/research/scripts/capped.sh" 7 bash "$REPO/research/scripts/run-with-memory-cap.sh" 8G "$@"
}
# C2-1、C2-2、C1 几何扫描（run-c21-c22.log、run-geom.log）
run cargo test --release --offline -p singlefs-harness --test attack_b4_opus -- --nocapture c2_1 c2_2 > "$WORK/run-c21-c22.log" 2>&1
run cargo test --release --offline -p singlefs-harness --test attack_b4_opus -- --nocapture c1_every_geometry > "$WORK/run-geom.log" 2>&1
run cargo test --release --offline -p singlefs-harness --test attack_b4_opus -- --nocapture seed_roots > "$WORK/run-seed.log" 2>&1
# C2-3、C2-4、C1 枚举（run-h1.log、run-h2.log）：挂载那一段里全是单元写的段整段落、不取子集（报告「这条腿自己的限度」第 1 条）
ATTACK_SKIP_UNIT_ONLY_SEGMENTS=1 run cargo test --release --offline -p singlefs-harness --test attack_b4_opus -- --nocapture history_one > "$WORK/run-h1.log" 2>&1
ATTACK_SKIP_UNIT_ONLY_SEGMENTS=1 run cargo test --release --offline -p singlefs-harness --test attack_b4_opus -- --nocapture history_two > "$WORK/run-h2.log" 2>&1
# C1 随机历史六档（run-rh-*.log）
for name in \
  random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation \
  reuse_heavy_random_histories_cover_released_records_and_end_only_in_known_red_forms \
  rollback_heavy_random_histories_reach_the_floor_root_and_end_only_in_known_red_forms \
  allocation_record_sampling_with_the_checker_goes_past_the_812_records_of_the_former_one_node_wall \
  unit_area_wall_sampling_on_small_devices_passes_only_a_placement_refused_on_every_device \
  unit_area_wall_sampling_on_small_devices_with_the_space_admission_judged_is_refused_by_the_formula_inside_the_model_interval; do
  run cargo test --release --offline -p singlefs-harness --test random_histories -- --include-ignored --exact "$name" --nocapture > "$WORK/run-rh-$name.log" 2>&1
done
# C2-1 的改法甲（checker 的 F 那一格改成等于格式常量），只在副本上（run-fix-f.log）
patch -p1 < "$MODEL/fix-f-equals-format-constant.patch"
run cargo test --release --offline -p singlefs-harness --test attack_b4_opus --test checker_known_bad_images -- --nocapture c2_1 each_ring_geometry > "$WORK/run-fix-f.log" 2>&1
