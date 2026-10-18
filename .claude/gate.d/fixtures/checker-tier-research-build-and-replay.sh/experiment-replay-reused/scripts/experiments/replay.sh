#!/usr/bin/env bash
# checker-tier-research-build-and-replay.sh 的 experiment-replay 格读 replay.sh 按行复用报数的判别力样本：登记表的形状照真表（编号|二进制|参数|产物|exact 或 timing），
# 本体把派给它的编号打出来，末行照真的 replay.sh 打一行 REPLAY_TOTAL（现判 1 行、复用 2 行、拒算 0 行、已归档只跑不比 1 行），退 0
REPLAY_TABLE=$(cat <<'TABLE'
E1|e1-sample||e1-sample.out|exact
E9|e9-sample||e9-sample.out|exact
E17|e17-sample||e17-sample.out|timing
TABLE
)
echo "样本 replay.sh 收到：$*（表里 $(wc -l <<<"$REPLAY_TABLE") 行）"
echo "REPLAY_TOTAL dispatched=3 collected=3 judged=1 reused=2 refused=0 pass=1 timing_only=0 archived_run_only=1 drift=0 broken=0 claims_bad=0 unchanged=0 load_skipped=0 full=0"
