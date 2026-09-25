#!/usr/bin/env bash
# 复跑 m2-final-code-r4 云端攻方（Opus）的用例：拷冻结副本（改后）与第三轮冻结副本（改前对照）→ 改后那份打副本专用补丁
# （只加三个环境变量开关 R4_OPUS_MUTATION、R4_OPUS_RESTORE、R4_OPUS_ADMISSION，不设变量时与冻结副本同行为）→ 放用例 → 跑。
# 用法：bash rerun.sh <空的工作目录>；日志落在 <工作目录>/logs/。线程上限 8。
set -euo pipefail
WORK=${1:?给一个空的工作目录}
HERE=$(cd "$(dirname "$0")" && pwd)
CAP=/home/fy5090/code/singlefs/research/scripts/capped.sh
mkdir -p "$WORK/logs"
rsync -a --exclude target --exclude .git /tmp/claude-1000/m2-final-code-r4/tree/ "$WORK/tree/"
rsync -a --exclude target --exclude .git /tmp/claude-1000/m2-final-code-r3/tree/ "$WORK/tree-r3/"
(cd "$WORK/tree" && patch -p1 < "$HERE/copy-only-core.patch")
cp "$HERE"/tests/r4_opus_z19.rs "$HERE"/tests/r4_opus_z21.rs "$HERE"/tests/r4_opus_z22.rs "$WORK/tree/crates/singlefs-harness/tests/"
cp "$HERE"/tests/r4_opus_z21.rs "$WORK/tree-r3/crates/singlefs-harness/tests/"
L=$WORK/logs
cd "$WORK/tree"; export CARGO_TARGET_DIR=$WORK/target
run() { nice -n 19 bash "$CAP" 8 cargo test -q -p singlefs-harness --test "$@"; }
# Z22：formatted_pool 注入点 1..12 扫一遍；副本变异「取号写之前不重算」下再扫一遍
run r4_opus_z22 -- --nocapture > $L/z22-sweep.log 2>&1
R4_OPUS_MUTATION=skip-recompute run r4_opus_z22 -- --nocapture > $L/z22-sweep-mutation-skip-recompute.log 2>&1
# Z21（改后）：A、B、B2、C
run r4_opus_z21 -- --nocapture --test-threads 4 > $L/z21-r4.log 2>&1
R4_OPUS_Z21C_BASE=92000 run r4_opus_z21 -- --nocapture --exact z21_c_random_rollback_series_with_crashes_in_the_post_window_restore_exactly_the_intended_entries > $L/z21-r4-c-sample2.log 2>&1
# Z21 攻方试的改法（副本）：只补 N = 所选根实例那一条
R4_OPUS_RESTORE=chosen-instance-only run r4_opus_z21 -- --nocapture --test-threads 4 > $L/z21-r4-restore-chosen-instance-only.log 2>&1
# Z19-B / Z19-C：一次放行的会话之后下一次可写挂载；岔路 2 删项臂（副本）；扫 k 那几段关掉准入（改前的行为）
run r4_opus_z19 -- --nocapture --exact z19_b_every_admitted_session_leaves_the_pool_writable_at_the_next_mount > $L/z19-b.log 2>&1
run r4_opus_z19 -- --nocapture --exact z19_c_step_by_step_after_the_last_admitted_overwrite > $L/z19-c.log 2>&1
R4_OPUS_ADMISSION=no-defer-term run r4_opus_z19 -- --nocapture --exact z19_b_every_admitted_session_leaves_the_pool_writable_at_the_next_mount > $L/z19-b-no-defer-term.log 2>&1
R4_OPUS_Z19_SKIP_AFTER_THE_PROBE=1 run r4_opus_z19 -- --nocapture --exact z19_b_every_admitted_session_leaves_the_pool_writable_at_the_next_mount > $L/z19-b-skip-after-probe.log 2>&1
# Z19-A：小盘随机历史，准入判着，两个种子基
run r4_opus_z19 -- --nocapture --exact z19_small_pool_campaigns_with_the_space_admission_judged > $L/z19-r4-sample1.log 2>&1
R4_OPUS_Z19_SEED_BASE=7050000000 run r4_opus_z19 -- --nocapture --exact z19_small_pool_campaigns_with_the_space_admission_judged > $L/z19-r4-sample2.log 2>&1
# Z21（改前对照，第三轮冻结树）
cd "$WORK/tree-r3"; export CARGO_TARGET_DIR=$WORK/target-r3
run r4_opus_z21 -- --nocapture --test-threads 4 > $L/z21-r3-control.log 2>&1
grep -h 'test result' $L/*.log
