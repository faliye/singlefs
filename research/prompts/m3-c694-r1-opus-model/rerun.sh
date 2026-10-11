#!/usr/bin/env bash
# 复跑 m3-c694-r1 攻方（Opus）模型。
# 环境变量：SINGLEFS_REPO 冻结副本根（有 Cargo.toml、Cargo.lock、crates/）；SINGLEFS_TOOLS 仓根（取 scripts/run-with-memory-cap.sh 与 capped.sh）；
#   THREADS 线程数（默认 4）；PROFILE quick（默认，前台约 3 分钟）或 full（报告里的全部数，约 30 分钟，交主 agent 放后台跑）。
# 用法：rerun.sh <空工作目录>。拷一份冻结副本到 <工作目录>/repo，make_patched.py 改（臂由环境变量选，一份编译服务全部臂），
#   编译目录 <工作目录>/target（编译 6G、跑 4G，经内存包装；同一时刻只起一条 cargo / 一个原型进程）。
#   每条跑的输出在 <工作目录>/out/；summary.txt 只收判定行（H1 / H2 / H5 / H5CLASS / L0TALLY / K3），排好序，没有耗时字段。
# 层 0 的枚举：不设 OPUS_FULL 时多于 OPUS_SEGMENT_LIMIT（默认 12）个写的段不展开（报告「这条腿自己的限度」一节写明）。
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
work="${1:?要一个空工作目录}"
: "${SINGLEFS_REPO:?SINGLEFS_REPO 指冻结副本根}"
: "${SINGLEFS_TOOLS:?SINGLEFS_TOOLS 指仓根（取包装脚本）}"
threads="${THREADS:-4}"
profile="${PROFILE:-quick}"
[ ! -e "$work/repo" ] || { echo "工作目录不空：$work/repo 已在" >&2; exit 2; }
mkdir -p "$work/repo" "$work/out"
work="$(cd "$work" && pwd)"
cp "$SINGLEFS_REPO/Cargo.toml" "$SINGLEFS_REPO/Cargo.lock" "$work/repo/"
cp -r "$SINGLEFS_REPO/crates" "$work/repo/crates"
python3 -B "$here/make_patched.py" "$work/repo"
cap="$SINGLEFS_TOOLS/scripts/run-with-memory-cap.sh"
capped="$SINGLEFS_TOOLS/scripts/capped.sh"
export CARGO_TARGET_DIR="$work/target"
( cd "$work/repo" && nice -n 19 bash "$cap" 6G bash "$capped" "$threads" cargo build --release -p opus-c694-probe ) > "$work/build.log" 2>&1
probe="$CARGO_TARGET_DIR/release/opus-c694-probe"
run() { # run <输出名> <环境变量赋值…> -- <子命令与参数…>
  local name="$1"; shift
  local envs=()
  while [ "$1" != "--" ]; do envs+=("$1"); shift; done
  shift
  env OPUS_THREADS="$threads" "${envs[@]}" nice -n 19 bash "$cap" 4G bash "$capped" "$threads" "$probe" "$@" > "$work/out/$name.log" 2>&1
}
if [ "$profile" = quick ]; then
  for arm in today A B C; do
    for wk in 384:5 384:3; do run "grid-${wk/:/-}-$arm" OPUS_C694_ARM=$arm -- grid "${wk%%:*}" "${wk##*:}" 3; done
  done
  for arm in today A B; do run "transient-240-3-$arm" OPUS_C694_ARM=$arm -- transient 240 3 8; done
  for arm in today C; do run "crashmount-384-5-$arm" OPUS_C694_ARM=$arm -- crashmount 384 5; done
  for arm in today R; do run "c695-$arm" OPUS_C695_ARM=$arm -- c695; done
else
  for arm in today A B C; do
    for w in 384 256 240 4g; do
      for k in 0 1 2 3 4 5 6 7 8 9 10; do run "grid-$w-$k-$arm" OPUS_C694_ARM=$arm -- grid "$w" "$k" 14; done
    done
  done
  for arm in today A B; do for wk in 240:3 240:2 240:4; do run "transient-${wk/:/-}-$arm" OPUS_C694_ARM=$arm -- transient "${wk%%:*}" "${wk##*:}" 8; done; done
  for arm in today A B C; do run "crashmount-384-5-$arm" OPUS_C694_ARM=$arm -- crashmount 384 5; done
  for arm in today C; do
    for wk in 256:5 240:5 384:3; do run "crashmount-${wk/:/-}-$arm" OPUS_C694_ARM=$arm -- crashmount "${wk%%:*}" "${wk##*:}"; done
    run "crashmount-hide-record-384-5-$arm" OPUS_C694_ARM=$arm OPUS_HIDE_RECORD=1 -- crashmount 384 5
  done
  for arm in today R S; do run "c695-$arm" OPUS_C695_ARM=$arm -- c695; done
fi
cat "$work"/out/*.log | grep -E '^(H1|H2|H5|H5CLASS|L0TALLY|K3) ' | LC_ALL=C sort > "$work/summary.txt"
echo "RERUN done: $work/summary.txt（$(wc -l < "$work/summary.txt") 行）"
