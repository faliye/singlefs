#!/usr/bin/env bash
# 第 20 批代码轮第一轮攻方腿（Opus）原型的复跑：在冻结树的两份拷贝（原样 tree、攻方改法 fixX）上编原型、逐步跑、出摘要与 expected-summary.txt 逐字比。
#   FROZEN_TREE=<冻结树，缺省 /tmp/claude-1000/m3-batch20-code-r1/tree> SINGLEFS_REPO=<仓根，缺省 /home/fy5090/code/singlefs> \
#     bash rerun.sh <空的工作目录> <步骤> [参数…]
# 步骤：prepare | build-tree | build-fix | sweep <from> <to> | fix-sweep <from> <to> | badimages | shapes | layer0 | n9
#       | fix-badimages | fix-shapes | fix-tests | summary | all
# 每一步一条前台命令跑得完（单步不超过约 4 分钟）；all 依次跑全部（实测约 30 分钟，给主 agent 放后台用）。
# 编译与跑编出来的代码一律经 research/scripts/run-with-memory-cap.sh（编译 6G、跑 4G），线程经 research/scripts/capped.sh；
# 原型线程数取 M3B20_THREADS（缺省 6）。只跑 harness 档：不起 checker 档的测试目标与 bin。
set -euo pipefail
model_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="${SINGLEFS_REPO:-/home/fy5090/code/singlefs}"
frozen_tree="${FROZEN_TREE:-/tmp/claude-1000/m3-batch20-code-r1/tree}"
work="${1:?工作目录}"
step="${2:?步骤}"
shift 2
threads="${M3B20_THREADS:-6}"
cap="$repo/research/scripts/run-with-memory-cap.sh"
capped="$repo/research/scripts/capped.sh"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-第20批代码轮攻方腿原型复跑，与同会话别的活互不依赖}"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="${RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON:-新编译目录从头编、分批跑的历史段不同，峰值表只记了别的一次}"
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS="${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-120}"

binary() { echo "$work/target-$1/release/m3b20-opus-proto"; }
run_mode() {
  local variant="$1" output="$2"
  shift 2
  M3B20_THREADS="$threads" nice -n 19 bash "$cap" 4G "$(binary "$variant")" "$@" > "$work/out/$output.out"
}
build() {
  (cd "$work/$1" && CARGO_TARGET_DIR="$work/target-$1" nice -n 19 bash "$capped" "$threads" bash "$cap" 6G cargo build -q --release -p m3b20-opus-proto)
}

case "$step" in
  prepare)
    [[ -z "$(ls -A "$work" 2>/dev/null)" ]] || { echo "工作目录要是空的：$work" >&2; exit 2; }
    (cd "$frozen_tree" && sha256sum --quiet -c "$repo/research/prompts/m3-batch20-code-r1-snapshot/crates-sha256.txt")
    mkdir -p "$work/out"
    for variant in tree fix; do
      rsync -a --exclude target --exclude .git "$frozen_tree/" "$work/$variant/"
      (cd "$work/$variant" && patch -s -p1 < "$model_dir/workspace.patch")
      mkdir -p "$work/$variant/crates/m3b20-opus-proto"
      cp -r "$model_dir/proto/Cargo.toml" "$model_dir/proto/src" "$work/$variant/crates/m3b20-opus-proto/"
    done
    (cd "$work/fix" && patch -s -p1 < "$model_dir/fixX.patch")
    echo "prepared $work"
    ;;
  build-tree) build tree; echo "built tree" ;;
  build-fix) build fix; echo "built fix" ;;
  sweep) run_mode tree "sweep-$1" sweep "$1" "$2" ;;
  fix-sweep) run_mode fix "fix-sweep-$1" sweep "$1" "$2" ;;
  badimages) run_mode tree badimages badimages ;;
  shapes) run_mode tree shapes shapes ;;
  layer0) run_mode tree layer0 layer0 ;;
  fix-badimages) run_mode fix fix-badimages badimages ;;
  fix-shapes) run_mode fix fix-shapes shapes ;;
  n9)
    M3B20_THREADS=1 nice -n 19 bash "$cap" 4G "$(binary tree)" n9 > "$work/n9-h1h2.raw"
    M3B20_N9_HISTORY=h3 M3B20_THREADS=1 nice -n 19 bash "$cap" 4G "$(binary tree)" n9 > "$work/n9-h3.raw"
    grep -v '^run-with-memory-cap' "$work/n9-h1h2.raw" > "$work/n9-h1h2.dump" || true
    grep -v '^run-with-memory-cap' "$work/n9-h3.raw" > "$work/n9-h3.dump" || true
    python3 "$model_dir/n9compare.py" "$repo" "$work/n9-h1h2.dump" "$work/n9-h3.dump" > "$work/out/n9.out"
    ;;
  fix-tests)
    (cd "$work/fix" && CARGO_TARGET_DIR="$work/target-fix" nice -n 19 bash "$capped" "$threads" bash "$cap" 6G cargo test -q --release -p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version --test checker_known_bad_images) > "$work/out/fix-tests.out" 2>&1
    ;;
  summary)
    python3 "$model_dir/summarize.py" "$work/out" > "$work/out/summary.txt"
    if diff "$model_dir/expected-summary.txt" "$work/out/summary.txt"; then echo "SUMMARY_MATCHES lines=$(wc -l < "$work/out/summary.txt")"; else echo "SUMMARY_DIFFERS" >&2; exit 1; fi
    ;;
  all)
    for one in prepare build-tree build-fix badimages shapes layer0 n9 fix-badimages fix-shapes fix-tests; do bash "$0" "$work" "$one"; done
    for from in 0 1200 2400 3600; do bash "$0" "$work" sweep "$from" $((from + 1200)); bash "$0" "$work" fix-sweep "$from" $((from + 1200)); done
    bash "$0" "$work" summary
    ;;
  *) echo "不认得的步骤：$step" >&2; exit 2 ;;
esac
