#!/usr/bin/env bash
# research-build-and-replay.sh 分片计划那一步的判别力样本：登记表的形状照真表（编号|二进制|参数|产物|exact 或 timing）。
# 有 exact 行的实验 3 个、只有 timing 行的 1 个；样本根的 .gate-shard-plan 让门禁照计划那一步走，这一份本体不该被起
REPLAY_TABLE=$(cat <<'TABLE'
E1|e1-sample||e1-sample.out|exact
E2|e2-sample||e2-sample.out|exact
E3|e3-sample||e3-sample.out|exact
E4|e4-sample||e4-sample.out|timing
TABLE
)
echo "样本 replay.sh 收到：$*（表里 $(wc -l <<<"$REPLAY_TABLE") 行）"
