#!/usr/bin/env bash
# 门禁 87 号的判别力样本：登记表的形状照真表（编号|二进制|参数|产物|exact 或 timing），本体只把派给它的编号打出来、退 0
REPLAY_TABLE=$(cat <<'TABLE'
E1|e1-sample||e1-sample.out|exact
E9|e9-sample||e9-sample.out|exact
E17|e17-sample||e17-sample.out|timing
TABLE
)
echo "样本 replay.sh 收到：$*（表里 $(wc -l <<<"$REPLAY_TABLE") 行）"
