#!/usr/bin/env bash
# stuck_by_fault.sh <日志…>：被宽护卡住的报告按故障组、R 那一槽有没有读得出的有效根（Some / None）数
cat "$@" | grep DIAG | grep -o 'faults=[a-z,]* .*NS\[[^;]*;S(r=[A-Za-z0-9()]*' | sed -E 's/faults=([a-z,]*) .*S\(r=(Some|None).*/\1 \2/' | sort | uniq -c
