#!/usr/bin/env bash
# m3-batch32-code-r1 攻方（Opus）模型复跑。
# 用法：SINGLEFS_REPO=<冻结副本根> SINGLEFS_TOOLS=<有 research/scripts 的仓根> [THREADS=4] bash rerun.sh <工作目录> [all|build|run]
#   all（默认）= build 再 run；build 要求工作目录是空的；run 接在同一个目录的 build 之后。前台一条命令有时限时分两步起。
# 变量：SINGLEFS_REPO 是被判的那棵树（冻结副本 ~/.local/share/singlefs/work/m3-batch32-code-r1/tree）；
#       SINGLEFS_TOOLS 是取内存包装 research/scripts/run-with-memory-cap.sh 与线程包装 research/scripts/capped.sh 的仓根（包装脚本另给的变量）；
#       THREADS 是编译线程数（默认 4）。跑的每一件单线程、经内存包装 4G，同批 4 件并行。
# 做什么：把冻结副本拷进工作目录，给 walk.rs 副本加臂开关（patch_walk_arms.py、patch_walk_arms_2.py），
#         把 checker_known_bad_images.rs 原样拷成 opus_b32_probe.rs 再接上 probe_tail.rs、probe_tail_p4p2.rs，
#         用工作目录自己的编译目录编 release（经内存包装 6G，同一时刻只这一条 cargo），四个 P1 探针（每个在进程里逐条换七条臂，起点镜像只建一次）、加 P4、P2 各一个，
#         汇总成 <工作目录>/summary.txt（summarize.py：排好序、不带耗时）。比对用 compare.sh。
set -euo pipefail
work="${1:?要一个工作目录}"
stage="${2:-all}"
tools="${SINGLEFS_TOOLS:?要 SINGLEFS_TOOLS}"
threads="${THREADS:-4}"
here="$(cd "$(dirname "$0")" && pwd)"
build() {
  local repo="${SINGLEFS_REPO:?要 SINGLEFS_REPO}"
  mkdir -p "$work"
  if [ -n "$(ls -A "$work")" ]; then echo "工作目录不空：$work" >&2; exit 2; fi
  rsync -a --exclude target --exclude .git "$repo"/ "$work/tree/"
  python3 "$here/patch_walk_arms.py" "$work/tree/crates/singlefs-checker/src/walk.rs"
  python3 "$here/patch_walk_arms_2.py" "$work/tree/crates/singlefs-checker/src/walk.rs"
  local tests_dir="$work/tree/crates/singlefs-harness/tests"
  cat "$tests_dir/checker_known_bad_images.rs" "$here/probe_tail.rs" "$here/probe_tail_p4p2.rs" > "$tests_dir/opus_b32_probe.rs"
  ( cd "$work/tree" && CARGO_TARGET_DIR="$work/target" nice -n 19 bash "$tools/research/scripts/run-with-memory-cap.sh" 6G \
      bash "$tools/research/scripts/capped.sh" "$threads" \
      cargo test --release --offline --no-run -p singlefs-harness --test opus_b32_probe ) > "$work/build.log" 2>&1
  grep -o '/[^ ]*/release/deps/opus_b32_probe-[0-9a-f]*' "$work/build.log" | tail -n 1 > "$work/binary.txt"
  [ -x "$(cat "$work/binary.txt")" ] || { echo "没编出探针二进制，见 $work/build.log" >&2; exit 3; }
  echo "编好：$(cat "$work/binary.txt")"
}
run() {
  # 可续：已经跑完（日志里有 test result: ok）的件不再跑；前台时限把一批打断了，再起一次 run 接着跑没完的那几件。
  local binary
  binary="$(cat "$work/binary.txt")"
  [ -x "$binary" ] || { echo "先跑 build" >&2; exit 3; }
  mkdir -p "$work/logs"
  local items=(today:opus_probe_p1_o3_tomb today:opus_probe_p1_o3_inode today:opus_probe_p1_o3_lineage today:opus_probe_p1_split
               today:opus_probe_p4_zero_slot_next_to_a_written_slot today:opus_probe_p2_copy_matrix)
  local pending=() item
  for item in "${items[@]}"; do
    grep -qs 'test result: ok' "$work/logs/${item%%:*}.${item#*:}.log" || pending+=("$item")
  done
  export SINGLEFS_TOOLS="$tools"
  local run_id i
  run_id="r$(date +%s)"
  for ((i = 0; i < ${#pending[@]}; i += 4)); do
    bash "$here/run_batch.sh" "$run_id-$((i / 4 + 1))" "$binary" "$work/logs" "${pending[@]:i:4}"
  done
  for item in "${items[@]}"; do
    grep -qs 'test result: ok' "$work/logs/${item%%:*}.${item#*:}.log" || { echo "$item 没跑完" >&2; exit 5; }
  done
  python3 "$here/summarize.py" "$work/logs" > "$work/summary.txt"
  echo "汇总 $(wc -l < "$work/summary.txt") 行：$work/summary.txt"
}
case "$stage" in
  all) build; run ;;
  build) build ;;
  run) run ;;
  *) echo "不认得 $stage" >&2; exit 2 ;;
esac
