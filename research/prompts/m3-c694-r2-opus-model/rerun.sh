#!/usr/bin/env bash
# 复跑 m3-c694-r2 攻方（Opus）模型。
# 环境变量：SINGLEFS_REPO 冻结副本根（有 Cargo.toml、Cargo.lock、crates/；本轮是 ~/.local/share/singlefs/work/m3-c694-r2/tree）；
#   SINGLEFS_TOOLS 仓根（只取 scripts/run-with-memory-cap.sh 与 capped.sh 两个包装脚本）；
#   THREADS 线程数（默认 6）；PROFILE quick（默认，前台约 4 分钟）或 full（报告里的全部数，期望文件 expected/ 就是它的输出，约 25–40 分钟，看负载）。
# 用法：rerun.sh <空工作目录>。拷一份冻结副本到 <工作目录>/repo，make_patched.py 改（臂由环境变量选，一份编译服务全部臂），
#   编译目录 <工作目录>/target（编译 6G、跑 4G，经内存包装；同一时刻只起一条 cargo、一个原型进程）。
#   每条跑的输出在 <工作目录>/out/；summary.txt 只收判定行，排好序，没有耗时字段。比对用 compare.sh。
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
work="${1:?要一个空工作目录}"
: "${SINGLEFS_REPO:?SINGLEFS_REPO 指冻结副本根}"
: "${SINGLEFS_TOOLS:?SINGLEFS_TOOLS 指仓根（取包装脚本）}"
threads="${THREADS:-6}"
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
( cd "$work/repo" && nice -n 19 bash "$cap" 6G bash "$capped" "$threads" cargo build --release -p opus-c694r2-probe ) > "$work/build.log" 2>&1
probe="$CARGO_TARGET_DIR/release/opus-c694r2-probe"
run() { # run <输出名> <环境变量赋值…> -- <子命令与参数…>
  local name="$1"; shift
  local envs=()
  while [ "$1" != "--" ]; do envs+=("$1"); shift; done
  shift
  env OPUS_THREADS="$threads" LAYER0_THREADS="$threads" "${envs[@]}" nice -n 19 bash "$cap" 4G bash "$capped" "$threads" "$probe" "$@" > "$work/out/$name.log" 2>&1
  echo "done $name" >> "$work/progress.txt"
}
o18=oooooooooooooooooo
# 两档都跑的：K2 解开时机（第一节）、K3 恢复补法会不会悄悄读回旧版（5.3）、W1 在读回旧内容下误放（4.1）、窄盘代价的一排（第三节）、记录读不出那一形（2.2）。
for rel in d16 ring mount; do for last in o s; do run "cc-$rel-$last" OPUS_C694_ARM=C OPUS_C_RELEASE=$rel -- chaincm 384 5 ow ur $o18 $last; done; done
for fb in none strict loose; do for w in "384 5" "256 3"; do run "silent-$fb-${w/ /-}" OPUS_RECOVERY_FALLBACK=$([ $fb = none ] || echo $fb) -- silent $w; done; done
for k in 16 17 18 19; do for arm in C+W C+B; do run "bcheck-256-k$k-$arm" OPUS_THREADS=1 OPUS_C694_ARM=$arm OPUS_C_RELEASE=ring -- bcheck 256 $k ow ur 1 o; done; done
for arm in "today mount" "C mount" "C ring"; do set -- $arm; run "narrow-240-3-sw16-$1-$2" OPUS_C694_ARM=$1 OPUS_C_RELEASE=$2 -- narrow 240 3 sw16 ur 40; done
for hide in urM; do run "grid-sw3-$hide-C-ring" OPUS_C694_ARM=C OPUS_C_RELEASE=ring -- grid 384 5 sw3 $hide 3 onsu; done
if [ "$profile" = full ]; then
  # 层 0 全量（每一段都展开）：2.1、5.2；补法关着的基线。
  run cm-today-ow-ur -- cm 384 5 ow ur
  run cm-C-all-sw3-urL OPUS_C694_ARM=C -- cm 384 5 sw3 urL
  run cm-C-last-sw3-urL OPUS_C694_ARM=C OPUS_C_SELECT=last -- cm 384 5 sw3 urL
  run cm-today-strict-ow-ur OPUS_RECOVERY_FALLBACK=strict -- cm 384 5 ow ur
  run cm-today-strict-ow-urJ OPUS_RECOVERY_FALLBACK=strict -- cm 384 5 ow urJ
  # 2.2 其余藏法与对照，5.2 末条。
  for hide in urL urF; do run "grid-sw3-$hide-C-ring" OPUS_C694_ARM=C OPUS_C_RELEASE=ring -- grid 384 5 sw3 $hide 3 onsu; done
  run grid-sw3-ur-C-ring OPUS_C694_ARM=C OPUS_C_RELEASE=ring -- grid 384 5 sw3 ur 3 onsu
  run grid-sw3-ur-today -- grid 384 5 sw3 ur 3 onsu
  for fb in strict loose; do run "grid-sw3-urM-C-ring-$fb" OPUS_C694_ARM=C OPUS_C_RELEASE=ring OPUS_RECOVERY_FALLBACK=$fb -- grid 384 5 sw3 urM 3 onsu; done
  # 第三节全表。
  for w in 240 256; do for k in 0 3 6; do for rk in ow sw2 sw4 sw8 sw16; do for arm in "today mount" "C mount" "C ring"; do set -- $arm
    run "narrowall-$w-$k-$rk-$1-$2" OPUS_C694_ARM=$1 OPUS_C_RELEASE=$2 -- narrow $w $k $rk ur 40
  done; done; done; done
  for rk in sw20 sw24 sw28 sw32; do for arm in "today mount" "C mount"; do set -- $arm; run "narrowbig-240-0-$rk-$1-$2" OPUS_C694_ARM=$1 OPUS_C_RELEASE=$2 -- narrow 240 0 $rk ur 40; done; done
  # 4.1 的起点搜与 4.2 的对照。
  for w in 384 256 240 4g; do run "stalecheck-$w" -- stalecheck $w 19 ow; done
  for k in 5 10 30; do for arm in C+B C+W B W; do run "bcheck-384-k$k-$arm" OPUS_THREADS=1 OPUS_C694_ARM=$arm OPUS_C_RELEASE=ring -- bcheck 384 $k ow ur 1 o; done; done
  for arm in B W C+B C+W; do run "bcheck-384-5-depth2-$arm" OPUS_THREADS=3 OPUS_C694_ARM=$arm OPUS_C_RELEASE=ring -- bcheck 384 5 ow ur 2 osn; done
fi
cat "$work"/out/*.log | grep -E '^(CM|L0TALLY|CMCLASS|CMMOUNT|G|GTALLY|CC|CCCLASS|NARROW|SILENT|STALE) ' | LC_ALL=C sort > "$work/summary.txt"
echo "RERUN done: $work/summary.txt（$(wc -l < "$work/summary.txt") 行）"
