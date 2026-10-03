#!/usr/bin/env bash
# 攻方 m3-verus-land-code-r2 的复跑：在 <工作目录> 里从冻结副本拷核与仓，跑 Verus 三张表、harness 两张表。
# 用法：bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 6 bash rerun.sh <冻结副本根> <工作目录>
# 冻结副本根：/tmp/claude-1000/m3-verus-land-code-r2/tree（sha256 清单 research/prompts/m3-verus-land-code-r2-snapshot/crates-sha256.txt）。
# Verus 三张表各几十秒；harness 那 14 行在本机负载下 release 共约 20 分钟（含 14 次增量编译）。
set -u
frozen=$1; work=$2
model=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$work/verus"
cp -a "$frozen/crates/singlefs-replay-prefix" "$work/verus/base"
bash "$model/verus-rows.sh" "$work/verus/base" "$model/attack-rows.tsv" "$work/attack-rows" $(seq 2 16)
bash "$model/verus-rows.sh" "$work/verus/base" "$frozen/crates/mutations.tsv" "$work/crates-rows" 23 24 36 226 355 406 414 448 512 513 514 515 516 517 613 614 1003 1004 1495 1496 1497 1498 1502 1503
bash "$model/verus-rows.sh" "$work/verus/base" "$model/dead-code-row.tsv" "$work/dead-code" 2
mkdir -p "$work/tree"
rsync -a --exclude target "$frozen/" "$work/tree/"
export CARGO_TARGET_DIR=$work/target CARGO_PROFILE_FLAG=--release
bash "$model/harness-rows.sh" "$work/tree" "$frozen" "$frozen/crates/mutations.tsv" "$work/crates-harness" 1497 1498 1499 1500 1501 1503 1502 1504 1505 512 513
bash "$model/harness-rows.sh" "$work/tree" "$frozen" "$model/call-pattern-rows.tsv" "$work/call-pattern-harness" 2 4 5
python3 "$model/reclassify.py" "$frozen/crates/mutations.tsv" "$work/crates-harness"
python3 "$model/reclassify.py" "$model/call-pattern-rows.tsv" "$work/call-pattern-harness"
