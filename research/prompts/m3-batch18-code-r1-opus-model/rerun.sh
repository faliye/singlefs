#!/usr/bin/env bash
# 第 18 批代码轮第一轮攻方腿（Opus）原型的复跑：在冻结树的三份拷贝（原样、改法 A、注入的坏实现 C）上编原型、逐步跑、出摘要与 expected-summary.txt 比。
#   FROZEN_TREE=<冻结树，缺省 /tmp/claude-1000/m3-batch18-code-r1/tree> bash rerun.sh <空的工作目录> <步骤> [参数…]
# 步骤：prepare | build | probe | sweep <from> <to> | sweepx <from> <to> | crash <from> <to> <part> | badimages | gpugap | n9 | diskloss
#       | fixA-badimages | fixA-sweep <from> <to> | fixA-tests | mutC-sweep | mutC-crash | summary | all
# 每一步一条前台命令跑得完（单步不超过约 3 分钟）；all 依次跑全部（约 50 分钟，给主 agent 放后台用）。
# 跑编出来的代码一律经 scripts/hosts/run-with-memory-cap.sh；线程数取 M3B18_THREADS（缺省 10）。只跑 harness 档：不起 checker 档的测试目标。
set -euo pipefail
model_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$model_dir/../../.." && pwd)"
frozen_tree="${FROZEN_TREE:-/tmp/claude-1000/m3-batch18-code-r1/tree}"
work="${1:?工作目录}"
step="${2:?步骤}"
shift 2
threads="${M3B18_THREADS:-10}"
cap="$repo/scripts/hosts/run-with-memory-cap.sh"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-第18批代码轮攻方腿原型复跑，与同会话别的活互不依赖}"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="${RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON:-新编译目录从头编、分批跑的历史段不同，峰值表只记了别的一次}"
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS="${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-120}"

binary() { echo "$work/target-$1/release/m3b18-opus-proto"; }
run_mode() {
  local variant="$1" output="$2"
  shift 2
  M3B18_THREADS="$threads" nice -n 19 bash "$cap" "${M3B18_RUN_CAP:-2G}" "$(binary "$variant")" "$@" > "$work/out/$output.out"
}

case "$step" in
  prepare)
    [[ -z "$(ls -A "$work" 2>/dev/null)" ]] || { echo "工作目录要是空的：$work" >&2; exit 2; }
    (cd "$frozen_tree" && sha256sum --quiet -c "$repo/research/prompts/m3-batch18-code-r1-snapshot/crates-sha256.txt")
    mkdir -p "$work/out"
    for variant in tree fixA mutC; do
      rsync -a --exclude target --exclude .git "$frozen_tree/" "$work/$variant/"
      (cd "$work/$variant" && patch -s -p1 < "$model_dir/workspace.patch")
      mkdir -p "$work/$variant/crates/m3b18-opus-proto"
      cp -r "$model_dir/proto/Cargo.toml" "$model_dir/proto/src" "$work/$variant/crates/m3b18-opus-proto/"
    done
    (cd "$work/fixA" && patch -s -p1 < "$model_dir/fixA.patch")
    (cd "$work/mutC" && patch -s -p1 < "$model_dir/mutC.patch")
    echo "prepared $work"
    ;;
  build)
    for variant in tree fixA mutC; do
      (cd "$work/$variant" && CARGO_TARGET_DIR="$work/target-$variant" nice -n 19 bash "$repo/scripts/hosts/capped.sh" "$threads" bash "$cap" 3G cargo build -q --release -p m3b18-opus-proto)
    done
    echo "built"
    ;;
  probe) run_mode tree probe probe ;;
  sweep) M3B18_FROM="$1" run_mode tree "sweep-$1" sweep "$2" ;;
  sweepx) M3B18_RUN_CAP=3G M3B18_EXTRA=1 M3B18_FROM="$1" run_mode tree "sweepx-$1" sweep "$2" ;;
  crash) M3B18_PART="$3/2" run_mode tree "crash-$1-p$3" crashpoints "$1" "$2" ;;
  badimages) run_mode tree badimages badimages ;;
  gpugap) run_mode tree gpugap gpugap ;;
  n9) run_mode tree n9 n9 ;;
  diskloss) run_mode tree diskloss diskloss ;;
  fixA-badimages) run_mode fixA fixA-badimages badimages ;;
  fixA-sweep) M3B18_FROM="$1" run_mode fixA "fixA-sweep-$1" sweep "$2" ;;
  fixA-tests)
    (cd "$work/fixA" && CARGO_TARGET_DIR="$work/target-fixA" nice -n 19 bash "$repo/scripts/hosts/capped.sh" "$threads" bash "$cap" 4G cargo test -q --release -p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version --test checker_known_bad_images) > "$work/out/fixA-tests.out" 2>&1
    ;;
  mutC-sweep) M3B18_FROM=1260 run_mode mutC mutC-sweep sweep 1680 ;;
  mutC-crash) M3B18_PART=0/2 M3B18_THREADS=2 run_mode mutC mutC-crash crashpoints 36 38 ;;
  summary)
    python3 "$model_dir/summarize.py" "$work/out" > "$work/out/summary.txt"
    if diff "$model_dir/expected-summary.txt" "$work/out/summary.txt"; then echo "SUMMARY_MATCHES lines=$(wc -l < "$work/out/summary.txt")"; else echo "SUMMARY_DIFFERS" >&2; exit 1; fi
    ;;
  all)
    for one in prepare build probe badimages gpugap n9 diskloss fixA-badimages fixA-tests mutC-sweep mutC-crash; do bash "$0" "$work" "$one"; done
    bash "$0" "$work" sweep 0 1500; bash "$0" "$work" sweep 1500 999999
    bash "$0" "$work" fixA-sweep 0 1500; bash "$0" "$work" fixA-sweep 1500 999999
    for from in 0 9 18 27 36 45 54 63; do for part in 0 1; do bash "$0" "$work" crash "$from" $((from + 9)) "$part"; done; done
    for from in 0 1650 3300 4950 6600 8250; do bash "$0" "$work" sweepx "$from" $((from + 1650)); done
    bash "$0" "$work" summary
    ;;
  *) echo "不认得的步骤：$step" >&2; exit 2 ;;
esac
