#!/usr/bin/env bash
# 复跑 m2-safety-r1 云端正推腿（Sonnet）S4、S5 两组打点实验。
# 都只加环境变量门控的 eprintln / 一个新的 CLI 模式，不设变量时与冻结副本同行为。
# 用法：bash rerun.sh <空的工作目录>；日志落在 <工作目录>/logs/。线程上限 8。
set -euo pipefail
WORK=${1:?给一个空的工作目录}
HERE=$(cd "$(dirname "$0")" && pwd)
CAP=/home/fy5090/code/singlefs/research/scripts/capped.sh
mkdir -p "$WORK/logs"

# ---- S4：admission.rs / mount.rs / transaction.rs 打点，加 s4_trace_probe.rs 与 s4_z19b_rerun.rs ----
rsync -a --exclude target --exclude .git /tmp/claude-1000/m2-safety-r1/tree/ "$WORK/tree-s4/"
(cd "$WORK/tree-s4" && patch -p1 < "$HERE/s4/admission.rs.patch")
(cd "$WORK/tree-s4" && patch -p1 < "$HERE/s4/mount.rs.patch")
(cd "$WORK/tree-s4" && patch -p1 < "$HERE/s4/transaction.rs.patch")
cp "$HERE"/s4/tests/s4_trace_probe.rs "$HERE"/s4/tests/s4_z19b_rerun.rs "$WORK/tree-s4/crates/singlefs-harness/tests/"
L=$WORK/logs
cd "$WORK/tree-s4"; export CARGO_TARGET_DIR=$WORK/target-s4
run() { nice -n 19 bash "$CAP" 8 cargo test -q -p singlefs-harness --test "$@"; }
SONNET_S4_TRACE=1 run s4_trace_probe -- --nocapture > "$L/s4-trace-probe.log" 2>&1
run s4_z19b_rerun -- --nocapture --exact z19_b_every_admitted_session_leaves_the_pool_writable_at_the_next_mount > "$L/z19b-baseline.log" 2>&1
SONNET_S4_NO_DEFER_TERM=1 run s4_z19b_rerun -- --nocapture --exact z19_b_every_admitted_session_leaves_the_pool_writable_at_the_next_mount > "$L/z19b-no-defer-term.log" 2>&1

# ---- S5：e158_root_choice_repair.rs 打点 + 新增 q3-1-s4-timing-probe-4 模式 ----
rsync -a --exclude target --exclude .git /tmp/claude-1000/m2-safety-r1/tree/ "$WORK/tree-s5/"
(cd "$WORK/tree-s5" && patch -p1 < "$HERE/s5/e158_root_choice_repair.rs.patch")
cd "$WORK/tree-s5"; export CARGO_TARGET_DIR=$WORK/target-s5
SONNET_S5_TRACE=1 nice -n 19 bash "$CAP" 8 cargo run -q -p singlefs-harness --bin e158_root_choice_repair --release -- q3-1-s4-timing-probe-4 > "$L/s5-probe4.log" 2>&1

grep -h 'fault_set_violation_summary\|test result' "$L"/*.log
