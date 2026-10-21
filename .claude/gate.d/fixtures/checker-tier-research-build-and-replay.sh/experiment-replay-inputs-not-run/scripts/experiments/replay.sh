#!/usr/bin/env bash
# checker-tier-research-build-and-replay.sh 的 experiment-replay 格对 replay.sh 退 77（整趟判的全是输入不在本次未跑的行）的样本：
# 登记表的形状照真表，本体打两行本次未跑说明与 REPLAY_TOTAL（not_run=2、load_skipped=0）就退 77；这一格要说是「要读的产物全不在」，不说负载高
REPLAY_TABLE=$(cat <<'TABLE'
E1|e1-sample||e1-sample.out|exact
E2|e2-sample||e2-sample.out|exact
TABLE
)
echo "样本 replay.sh 收到：$*（表里 $(wc -l <<<"$REPLAY_TABLE") 行）"
echo "E1 本次未跑：NOT_RUN_ARCHIVED_PRODUCT test=E1 reason=research/results/e0-input.out 不在"
echo "E2 本次未跑：NOT_RUN_ARCHIVED_PRODUCT test=E2 reason=research/results/e0-input.out 不在"
echo "REPLAY_TOTAL dispatched=2 collected=2 judged=0 reused=0 refused=0 pass=0 timing_only=0 archived_run_only=0 drift=0 broken=0 claims_bad=0 unchanged=0 load_skipped=0 not_run=2 full=0"
exit 77
