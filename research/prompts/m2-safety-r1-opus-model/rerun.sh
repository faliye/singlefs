#!/usr/bin/env bash
# 复跑 m2-safety-r1 云端攻方（Opus）的用例：拷冻结副本 → 打副本专用补丁（只加环境变量开关，不设时与冻结副本同行为）
# → 放用例 → 编 → 跑 S1、S2/S3、campaign、定点用例。用法：bash rerun.sh <空的工作目录> [线程上限，默认 8]
# 日志落在 <工作目录>/logs/。一遍约 1.5 小时（盘 I/O 忙时更久）。
set -euo pipefail
WORK=${1:?给一个空的工作目录}; CAP_N=${2:-8}
HERE=$(cd "$(dirname "$0")" && pwd)
CAP=/home/fy5090/code/singlefs/research/scripts/capped.sh
mkdir -p "$WORK/logs" "$WORK/tmpimg"
rsync -a --exclude target --exclude .git /tmp/claude-1000/m2-safety-r1/tree/ "$WORK/tree/"
(cd "$WORK/tree" && patch -p1 < "$HERE/copy-only.patch")
cp "$HERE"/tests/s_opus_s1.rs "$HERE"/tests/s_opus_s23.rs "$WORK/tree/crates/singlefs-harness/tests/"
export CARGO_TARGET_DIR="$WORK/target" TMPDIR="$WORK/tmpimg"
(cd "$WORK/tree" && bash "$CAP" "$CAP_N" nice -n 19 cargo build --offline --release -p singlefs-harness --test s_opus_s1 --test s_opus_s23)
S1=$(ls -t "$WORK"/target/release/deps/s_opus_s1-* | grep -v '\.d$' | head -1)
S23=$(ls -t "$WORK"/target/release/deps/s_opus_s23-* | grep -v '\.d$' | head -1)
L=$WORK/logs
# S1：七个臂的持久 / 瞬时读错扫描、产品路径控制组、4 GiB 写 / 屏障失败、312 槽写 / 屏障失败
bash "$HERE/scripts/s1_all.sh" "$S1" "$L"
python3 "$HERE/scripts/s1_summary.py" "$L" today jia yi jiayi ding wu yiwu > "$L/s1-summary-all.txt"
# S1：产品路径宽一点的小盘（表 4）
seq 388 8 1020 | xargs -P 8 -I{} env S_OPUS_UNIT_AREA_SLOTS={} S_OPUS_N_FROM=18 S_OPUS_N_TO=40 S_OPUS_MODE=persistent nice -n 19 "$S1" --exact s1_scan_one_width --nocapture 2>/dev/null | grep '^S1ROW' | sort -t$'\t' -k2,2n -k3,3n -k5,5n > "$L/s1-today-judged-persistent-wide.tsv"
# S1：候选多拒的那一格用户手里还剩什么（表 5）
for v in S_OPUS_NONE=1 S_OPUS_S1_PROTECT=1 S_OPUS_S1_DRY_PESSIMISTIC=1; do
  echo "== $v"; env $v S_OPUS_ADMISSION=skip S_OPUS_UNIT_AREA_SLOTS=312 S_OPUS_N=24 nice -n 19 "$S1" --exact s1_exits --nocapture 2>&1 | grep S1X
done > "$L/s1-exits-312-24.log"
# S1：写 / 屏障失败之后三份镜像跑 checker（今天、乙、甲+乙）
mkdir -p "$L/s1chk"
for v in today:S_OPUS_NONE=1 yi:S_OPUS_S1_PROTECT=1 "jiayi:S_OPUS_S1_PROTECT=1 S_OPUS_S1_DRY_READS=1"; do name=${v%%:*}; ev=${v#*:}
  for fk in write barrier; do for idx in 0 1 2 3; do
    env $ev S_OPUS_CHECKER=1 S_OPUS_WIDTH=4g S_OPUS_N=30 S_OPUS_K_TO=40 S_OPUS_FAULT=$fk S_OPUS_TARGET_INDEX=$idx nice -n 19 "$S1" --exact s1_write_fault_sweep --nocapture 2>/dev/null | grep '^S1W' > "$L/s1chk/s1w-$name-4g-$fk-t$idx.tsv" &
  done; done; wait
done
# S2 / S3：第四轮 Z21 用例 + 随机判别子，十一个臂
bash "$HERE/scripts/s23_all.sh" "$S23" "$L/s23"
# S2：收窄补法的定点反例；S3-乙：恢复落到第 0 代根
mkdir -p "$L/s2hole"
for v in none:S_OPUS_S2=none today:S_OPUS_NONE=1 narrow:S_OPUS_S2=narrow clearflag:S_OPUS_S2=clearflag; do
  env ${v#*:} nice -n 19 "$S23" --exact s2_narrow_hole_targeted --nocapture 2>&1 | grep -E '^S2HOLE|panicked' > "$L/s2hole/s2hole-${v%%:*}.log"
done
for v in today:S_OPUS_NONE=1 infer:S_OPUS_S3=infer rowzero:S_OPUS_S3=row-zero; do
  env ${v#*:} nice -n 19 "$S23" --exact s3_infer_on_a_recovery_that_fell_to_generation_zero --nocapture 2>&1 | grep -E 'S3INFER|panicked' | sed "s/^/${v%%:*} /"
done > "$L/s3-infer-recovery-fall.log"
# 新失败面：仓里自己的随机历史执行器（S_OPUS_THREADS 取 8）
bash "$HERE/scripts/campaign_all.sh" "$S1" "$L/campaign"
