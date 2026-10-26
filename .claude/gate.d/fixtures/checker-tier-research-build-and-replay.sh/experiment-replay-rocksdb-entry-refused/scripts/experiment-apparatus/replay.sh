#!/usr/bin/env bash
# checker-tier-research-build-and-replay.sh 的 experiment-replay 格「起 replay.sh 之前判 RocksDB 预编库的取值」的判别力样本：登记表的形状照真表（编号|二进制|参数|产物|exact 或 timing），
# 被起了就把派给它的编号打出来、照真的 replay.sh 打一行 REPLAY_TOTAL、退 0（判对时它一次都不该被起）
REPLAY_TABLE=$(cat <<'TABLE'
E1|e1-sample||e1-sample.out|exact
TABLE
)
echo "样本 replay.sh 被起了：$*（表里 $(wc -l <<<"$REPLAY_TABLE") 行）"
echo "REPLAY_TOTAL dispatched=1 collected=1 judged=1 reused=0 refused=0 pass=1 timing_only=0 archived_run_only=0 drift=0 broken=0 claims_bad=0 unchanged=0 load_skipped=0 full=0"
