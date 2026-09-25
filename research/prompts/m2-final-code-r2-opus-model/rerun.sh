#!/usr/bin/env bash
# 复跑 m2-final-code-r2 云端攻方的用例：拷冻结副本 → 打副本专用补丁（只加环境变量开关，不设变量时与冻结副本同行为）→ 放用例 → 跑。
# 用法：bash rerun.sh <空的工作目录>
set -euo pipefail
WORK=${1:?给一个空的工作目录}
HERE=$(cd "$(dirname "$0")" && pwd)
CAP=/home/fy5090/code/singlefs/research/scripts/capped.sh
mkdir -p "$WORK"
rsync -a --exclude target --exclude .git /tmp/claude-1000/m2-final-code-r2/tree/ "$WORK/tree/"
(cd "$WORK/tree/crates" && patch -p0 < "$HERE/copy-only-core.patch")
cp "$HERE"/tests/r2_opus_z*.rs "$WORK/tree/crates/singlefs-harness/tests/"
cd "$WORK/tree"
export CARGO_TARGET_DIR="$WORK/target"
run() { bash "$CAP" 8 nice -n 19 cargo test -q -p singlefs-harness --test "$@"; }
# Z9-A（冻结副本行为）：A1、A2 在第 24 次回退写满；对照三种取法不崩各 40 次
run r2_opus_z9 -- --nocapture z9_a1 z9_a2 z9_a3 z9_a_control
# Z9-A 的试改删除规则（副本）：dominance / abandons / both
for rule in dominance abandons both; do SINGLEFS_R2_OPUS_WITNESS_RULE=$rule run r2_opus_z9 -- --nocapture z9_a1 z9_a2 z9_a3; done
# Z9-B 与试改补见证（副本）
run r2_opus_z9 -- --nocapture --exact z9_b_crash_between_row_root_and_its_rotation_never_gets_the_witness
SINGLEFS_R2_OPUS_WITNESS_REPAIR=1 run r2_opus_z9 -- --nocapture --exact z9_b_crash_between_row_root_and_its_rotation_never_gets_the_witness
# Z8
run r2_opus_z8 -- --nocapture --exact z8_one_unit
run r2_opus_z8 -- --nocapture --exact z8_two_units
# Z12：先编出二进制，再按两个扫描脚本跑（脚本里的日志目录是写死的 /tmp/claude-1000/m2-final-code-r2-opus/logs，复跑时改成自己的）
cargo test -q -p singlefs-harness --test r2_opus_z12 --no-run
