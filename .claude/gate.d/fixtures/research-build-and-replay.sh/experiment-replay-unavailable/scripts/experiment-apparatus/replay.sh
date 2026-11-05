#!/usr/bin/env bash
# research-build-and-replay.sh 的 experiment-replay 格对 replay.sh 退 75（它按行复用的判定库在 prune-server 里，服务不可用）的判别力样本：
# 登记表的形状照真表（编号|二进制|参数|产物|exact 或 timing），本体只打一句服务不可用就退 75。改前这一格对 75 退 1，红
REPLAY_TABLE=$(cat <<'TABLE'
E1|e1-sample||e1-sample.out|exact
TABLE
)
echo "  ✗ 按行复用：prune-server 不可用（样本 replay.sh 收到：$*，表里 $(wc -l <<<"$REPLAY_TABLE") 行）" >&2
exit 75
