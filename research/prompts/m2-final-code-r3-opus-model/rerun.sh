#!/usr/bin/env bash
# 复跑 m2-final-code-r3 云端攻方（Opus）的用例与模型。
# 用法：bash rerun.sh <空的工作目录>
# 拷冻结副本 → 打副本专用补丁（只加环境变量开关，不设时与冻结副本同行为）→ 放用例 → 编 → 按报告里的各格跑。
set -euo pipefail
WORK=${1:?给一个空的工作目录}
HERE=$(cd "$(dirname "$0")" && pwd)
CAP=/home/fy5090/code/singlefs/research/scripts/capped.sh
mkdir -p "$WORK/logs"
rsync -a --exclude target --exclude .git /tmp/claude-1000/m2-final-code-r3/tree/ "$WORK/tree/"
(cd "$WORK/tree" && patch -p2 < "$HERE/copy-only.patch")
cp "$HERE"/tests/r3_opus_z13.rs "$HERE"/tests/r3_opus_z16.rs "$WORK/tree/crates/singlefs-harness/tests/"
export CARGO_TARGET_DIR="$WORK/target"
(cd "$WORK/tree" && bash "$CAP" 8 nice -n 19 cargo build --offline --release -p singlefs-harness --test r3_opus_z13 --test r3_opus_z16)
Z13=$(ls -t "$WORK"/target/release/deps/r3_opus_z13-* | grep -v '\.d$' | head -1)
Z16=$(ls -t "$WORK"/target/release/deps/r3_opus_z16-* | grep -v '\.d$' | head -1)
cd "$WORK"
# Z17：纯算，不读实现
python3 "$HERE/model/z17_independent.py" > logs/z17.txt
# Z16-a：写死复现（三格）与试改
for cell in "312 50245" "316 50306" "336 50312"; do set -- $cell
  SINGLEFS_R3_OPUS_UNIT_AREA_SLOTS=$1 R3_OPUS_BAD_SLOT=$2 nice -n 19 "$Z16" --exact z16_rollback_refused_after_acquisition_burns_instance_generations_by_hand --nocapture 2>&1 | grep -E 'R3OPUS-HAND' > logs/z16-hand-$1-$2.log
  SINGLEFS_R3_OPUS_DRY_RUN_READS=1 SINGLEFS_R3_OPUS_UNIT_AREA_SLOTS=$1 R3_OPUS_BAD_SLOT=$2 R3_OPUS_ATTEMPTS=1 nice -n 19 "$Z16" --exact z16_rollback_refused_after_acquisition_burns_instance_generations_by_hand --nocapture 2>&1 | grep -E 'R3OPUS-HAND rollback' > logs/z16-hand-fix-reads-$1-$2.log
done
# Z16：定点扫描（对照 666 段 + 每段写行释放的槽逐个注入；盘 1、盘 0、试改三遍，各约 8 分钟）
python3 "$HERE/z16_targeted.py" "$Z16" logs/z16-targeted-240-384.tsv 240 384 4 18 35
R3_OPUS_FAULT_DEVICE=0 python3 "$HERE/z16_targeted.py" "$Z16" logs/z16-targeted-dev0-240-384.tsv 240 384 4 18 35
SINGLEFS_R3_OPUS_DRY_RUN_READS=1 python3 "$HERE/z16_targeted.py" "$Z16" logs/z16-targeted-fixreads-240-384.tsv 240 384 4 18 35
# Z16：冻结着一次失败之后挂载
for fk in write barrier; do for w in 4g 384; do
  R3_OPUS_WIDTH=$w R3_OPUS_FAULT=$fk R3_OPUS_K_TO=40 SINGLEFS_R3_OPUS_REPORT=1 nice -n 19 "$Z16" --exact z16_mount_after_a_frozen_publish_by_hand --nocapture > logs/z16-frozen2-$fk-$w.log 2>&1
done; done
# Z13：叶缺席又长出来（8 分片）、覆盖核对、树表 0 条那一路（8 分片）
for s in 0 1 2 3 4 5 6 7; do R3_OPUS_SHARD=$s R3_OPUS_SHARDS=8 nice -n 19 "$Z13" --exact z13_leaf_absent_then_regrown_after_rollback --nocapture > logs/z13-regrow-shard$s.log 2>&1 & done; wait
nice -n 19 "$Z13" --exact z13_regrow_coverage_probe --nocapture 2>&1 | grep R3OPUS > logs/z13-regrow-probe.log
for s in 0 1 2 3 4 5 6 7; do R3_OPUS_A_LIST=0,368,369,738,1200,23985 R3_OPUS_SHARD=$s R3_OPUS_SHARDS=8 nice -n 19 "$Z13" --exact without_file::z13_without_file_versions_rollback_and_regrow --nocapture > logs/z13-wf-shard$s.log 2>&1 & done; wait
# Z13：随机历史两次抽样
for base in 3000000000 4000000000; do
  if [ $base = 3000000000 ]; then cfgs="broad,4g,64,40 rollback,4g,64,40 reuse,384,64,60 unitwall,256,32,100 wall,4g,16,150"
  else cfgs="broad,4g,640,40 rollback,4g,640,40 reuse,384,640,60 unitwall,256,320,100 wall,4g,96,200 reuse,240,320,60"; fi
  for cfg in $cfgs; do IFS=, read -r wgt wid n ops <<< "$cfg"
    R3_OPUS_WEIGHTS=$wgt R3_OPUS_WIDTH=$wid R3_OPUS_SEEDS=$n R3_OPUS_OPS=$ops R3_OPUS_FIRST_SEED=$base R3_OPUS_THREADS=8 nice -n 19 "$Z13" --exact z13_campaign --nocapture > logs/z13-campaign-$base-$wgt-$wid.log 2>&1
  done
done
R3_OPUS_SEED=4000000045 nice -n 19 "$Z13" --exact z13_one_seed --nocapture 2>&1 | grep R3OPUS > logs/z13-seed-4000000045.log
