#!/usr/bin/env bash
# checker-tier-research-build-and-replay.sh 的 experiment-replay 格对 replay.sh 退 77（整趟判的全是本次不判的计时行：负载过高）的判别力样本：
# 登记表的形状照真表（编号|二进制|参数|产物|exact 或 timing），本体只打一行说明就退 77。改前这一格对 77 退 1，红
REPLAY_TABLE=$(cat <<'TABLE'
E1|e1-sample||e1-sample.out|exact
TABLE
)
echo "样本 replay.sh 收到：$*（表里 $(wc -l <<<"$REPLAY_TABLE") 行），判的全是本次不判（负载过高），一行都没比：退 77"
exit 77
