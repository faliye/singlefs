#!/bin/bash
# 用法：bash k2sum.sh <日志…>：按 起点、到没到抬 F 顶上（reached）、计数、红、回退结局 数段
for f in "$@"; do echo "== $(basename $f)"; grep -h '^OPUSI1-RUN' "$f" | grep -o 'sp=[^ ]*\|reached=[^ ]*\|counters=\[[^]]*\]\|red=[a-z]*\|rollback=[^ ]*' | paste -d' ' - - - - - | sed -E 's/(rollback=[A-Za-z-]+\((txg[0-9]+:?[A-Za-z]*|n=[0-9]+)).*/\1/' | sort | uniq -c; done
