#!/usr/bin/env bash
# jk_holds.sh <日志…>：被宽护卡住的报告按「预演结局 / C 槽数 / 推完 C 罩着的槽还隔离着几个 / 推完再试」分类
cat "$@" | grep DIAG | grep -o 'S(r=[A-Za-z0-9()]*,n=[0-9]*,iso=[0-9]*[^;]*;hold=[0-9];isoC=[0-9]*;needs=[A-Za-z0-9()]*\]|[a-zA-Z0-9]*|stillIso=[0-9/]*|retry[A-Za-z]*' \
 | sed -E 's/S\(r=[A-Za-z0-9()]*,n=[0-9]*,iso=[0-9]*[^;]*(reh=[AR]).*;hold=([0-9]);isoC=[0-9]*;needs=[A-Za-z0-9()]*\]\|([a-zA-Z]*)[0-9]*\|stillIso=([0-9]*)\/[0-9]*\|(retry[A-Za-z]*)/\1 hold=\2 \3 stillIso>0:\4 \5/' \
 | sed -E 's/stillIso>0:0/stillIso=0/; s/stillIso>0:[1-9][0-9]*/stillIso>0/' | sort | uniq -c
