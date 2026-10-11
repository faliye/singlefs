#!/usr/bin/env bash
# 用法：run-one.sh <workdir> <序号|baseline> <输出文件>：落一项探针、编译并跑四个相关的测试二进制、写汇总、还原。
# 仓路径（取脚本与 cargo 配置）从 SINGLEFS_REPO 取；编译 6G、跑 4G、线程取 THREADS（默认 10），都经内存包装。
set -uo pipefail
work="$1"; which="$2"; out="$3"
here="$(cd "$(dirname "$0")" && pwd)"
: "${SINGLEFS_REPO:?要设 SINGLEFS_REPO（仓路径）}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$work/../target}"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-派出去的正推腿各跑各的探针}"
tests=(--test checker_known_bad_images --test pool_checker_judges_the_deletion_state_of_every_walked_version --test container_trees_grow_a_level_when_the_root_is_full --test inode_number_watermark_pressed_back_reissues_a_number)
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="${RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON:-派发提示点名编译 6G 跑 4G 不写 auto}"
cap="bash $SINGLEFS_REPO/scripts/hosts/run-with-memory-cap.sh"
phase="${PHASE:-all}"   # build：只落探针并编译；run：只跑（要先 build）；all：全做
if [ "$which" != "baseline" ] && [ "$phase" != "run" ]; then
  python3 "$here/apply.py" "$work" "$which" || { echo "APPLY_FAILED $which" > "$out"; exit 3; }
fi
cd "$work" || exit 3
if [ "$phase" != "run" ]; then
nice -n 19 $cap 6G bash "$SINGLEFS_REPO/scripts/hosts/capped.sh" "${THREADS:-10}" cargo test --offline --no-run -p singlefs-harness "${tests[@]}" > "$out.build" 2>&1
build_exit=$?
if [ $build_exit -ne 0 ]; then echo "BUILD_EXIT=$build_exit" > "$out"; python3 "$here/apply.py" "$work" restore >/dev/null; exit 4; fi
fi
[ "$phase" = "build" ] && exit 0
nice -n 19 $cap 4G bash "$SINGLEFS_REPO/scripts/hosts/capped.sh" "${THREADS:-10}" cargo test --offline --no-fail-fast -p singlefs-harness "${tests[@]}" > "$out.run" 2>&1
run_exit=$?
{
  echo "PROBE=$which RUN_EXIT=$run_exit"
  grep -E '^test result' "$out.run" | sed -E 's/; finished in [0-9.]+s//'
  echo "FAILED_TESTS:"
  grep -E '^test .* \.\.\. FAILED$' "$out.run" | sort
} > "$out"
python3 "$here/apply.py" "$work" restore >/dev/null
