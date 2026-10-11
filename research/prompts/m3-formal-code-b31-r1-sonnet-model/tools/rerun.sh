#!/usr/bin/env bash
# 云端正推腿（m3-formal-code-b31-r1）的复跑入口。每一段单独跑，输出追加进 $OUT（默认本目录的 out/）。
#
#   bash tools/rerun.sh <草稿目录> static    把冻结副本 rsync 到 <草稿目录>/tree，跑：N5 静态核（check_rows.py）、N4 的 p6 补丁核（patch --fuzz=0、diff、git hash-object）。不编译。
#   bash tools/rerun.sh <草稿目录> prepare   在 <草稿目录>/tree 里放腿自己的核对用例（sim_vs_core_root_ring.rs、on_device_modes_recipe_vs_core.rs）并把 scenario.rs 里那个私有函数改 pub（只在草稿副本）
#   bash tools/rerun.sh <草稿目录> tests     harness 档与 core 单测：新断言单测、C551 新用例、overwrite_in_one_instance、两条腿自己的核对用例（经内存包装）
#   bash tools/rerun.sh <草稿目录> mutants   core 记根回收门槛的 5 种错法 a–e（mutate_core_reclaim.sh，每种两条 harness 用例）
#   bash tools/rerun.sh <草稿目录> rows      变异表里 harness 档 / core 的 5 行（2734 2735 2736 2737 570）逐行改坏、跑点名的测试
#
# 环境：FROZEN（冻结副本，默认 /tmp/claude-1000/m3-formal-code-b31-r1/tree）、REPO（仓根）。
# 内存包装的理由与等待：设 RUN_WITH_MEMORY_CAP_PARALLEL_REASON / RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON / RUN_WITH_MEMORY_CAP_WAIT_SECONDS（本脚本给了默认值）。
# 线程上限默认 5（CAP_THREADS）。tests / mutants / rows 每一段在本机负载高时要几分钟，前台跑要自己拆开（子 agent 前台 240 秒上限）。
# 脚本里两处写死了冻结副本的路径（tools/mutate_core_reclaim.py、tools/run_mutation_rows.py 的 frozen / 表路径）：冻结副本挪了就改这两处。
set -uo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
model="$(dirname "$here")"
FROZEN="${FROZEN:-/tmp/claude-1000/m3-formal-code-b31-r1/tree}"
REPO="${REPO:-/home/fy5090/code/singlefs}"
OUT="${OUT:-$model/out}"
CAP_THREADS="${CAP_THREADS:-5}"
draft="${1:?草稿目录}"; stage="${2:?static|prepare|tests|mutants|rows}"
mkdir -p "$OUT" "$draft"
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS="${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-150}"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-同会话有别的子agent在跑各自的测试，本腿只跑harness档与core单测}"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="${RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON:-派发提示要求cargo一律显式给8G上限，不用auto}"
wrapped() { nice -n 19 bash "$REPO/scripts/run-with-memory-cap.sh" 8G bash "$REPO/scripts/capped.sh" "$CAP_THREADS" "$@"; }

case "$stage" in
static)
  rsync -a --exclude target/ "$FROZEN/" "$draft/tree/"
  { echo "== N5 静态核"; python3 "$here/check_rows.py" "$FROZEN"; } 2>&1 | tee -a "$OUT/n5-static.txt"
  mkdir -p "$draft/n4"
  q5="$REPO/scripts/experiments/target/e171-r1-frozen-tree-revision-5-q5/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs"
  repo_file="$FROZEN/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs"
  old="${OLD945:-/tmp/claude-1000/m3-formal-31-2/device-945e41c8.rs}"
  patches="${P6_DIR:-/tmp/claude-1000/m3-formal-31-2/patch}"
  {
    echo "== N4 p6 补丁核"
    git hash-object "$old" "$repo_file"
    sha256sum "$old" "$q5" "$repo_file"
    cp "$q5" "$draft/n4/q5.rs"
    patch --fuzz=0 -o "$draft/n4/q5-plus-p6.rs" "$draft/n4/q5.rs" < "$patches/p6-on-dbef78ed.patch"
    sha256sum "$draft/n4/q5-plus-p6.rs"
    echo "--- diff q5+p6 对仓里那份"; diff "$draft/n4/q5-plus-p6.rs" "$repo_file"; echo "exit=$?"
    echo "--- diff q5 对 945e41c8"; diff "$q5" "$old"; echo "exit=$?"
    grep -v '^@@\|^---\|^+++\|^diff \|^index ' "$patches/crates.patch" > "$draft/n4/a.txt"
    grep -v '^@@\|^---\|^+++\|^diff \|^index ' "$patches/p6-on-dbef78ed.patch" > "$draft/n4/b.txt"
    wc -l "$draft/n4/a.txt" "$draft/n4/b.txt"; cmp "$draft/n4/a.txt" "$draft/n4/b.txt" && echo "两份补丁去掉行号头之后逐字节相同"
    grep -c '^@@' "$patches/crates.patch" "$patches/p6-on-dbef78ed.patch"
  } 2>&1 | tee -a "$OUT/n4-p6.txt"
  ;;
prepare)
  tests="$draft/tree/crates/singlefs-harness/tests"
  cp "$here/sim_vs_core_root_ring.rs" "$tests/zz_sim_vs_core_root_ring.rs"
  cp "$here/on_device_modes_recipe_vs_core.rs" "$tests/zz_recipe_vs_core.rs"
  sed -i 's/^fn run_new_pool_file_creation_keeping_the_allocator</pub fn run_new_pool_file_creation_keeping_the_allocator</' "$draft/tree/crates/singlefs-harness/src/scenario.rs"
  [[ -d "$draft/litmus" ]] || cp -r "$REPO/litmus" "$draft/litmus"
  echo prepared
  ;;
tests)
  cd "$draft/tree" || exit 2
  {
    echo "== core 单测：新断言"
    wrapped cargo test --offline -p singlefs-core --lib -- recording_a_root_on_an_allocator_without 2>&1 | grep -E '^test |test result|^error'
    echo "== harness：C551 新用例"
    wrapped cargo test --offline -p singlefs-harness --test root_ring_turn_in_the_make_filesystem_session 2>&1 | grep -E '^test |test result|^error'
    echo "== harness：overwrite_in_one_instance 整份"
    wrapped cargo test --offline -p singlefs-harness --test overwrite_in_one_instance 2>&1 | grep -E 'test result|FAILED|^error'
    echo "== 腿自己的核对：用例里模拟的根环 == core 持有的根环（87 个 txg）"
    wrapped cargo test --offline -p singlefs-harness --test zz_sim_vs_core_root_ring -- --nocapture 2>&1 | grep -E '^test |test result|compared_txgs|^error'
    echo "== 腿自己的核对：on_device_modes 拼的根环 == core 持有的根环"
    wrapped cargo test --offline -p singlefs-harness --test zz_recipe_vs_core -- --nocapture 2>&1 | grep -E '^test |test result|genesis_txg|^error'
  } 2>&1 | tee -a "$OUT/tests.txt"
  ;;
mutants)
  for m in a b c d e; do
    wrapped bash "$here/mutate_core_reclaim.sh" "$draft/tree" "$m" 2>&1 | grep -v 'run-with-memory-cap:' | tee -a "$OUT/mutants.txt"
  done
  ;;
rows)
  wrapped python3 "$here/run_mutation_rows.py" "$draft/tree" 2734 2735 2736 2737 570 2>&1 | grep -v 'run-with-memory-cap:' | tee -a "$OUT/rows.txt"
  ;;
*) echo "未知段 $stage"; exit 2 ;;
esac
