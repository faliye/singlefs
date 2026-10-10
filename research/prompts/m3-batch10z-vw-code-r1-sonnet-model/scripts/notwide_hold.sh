#!/usr/bin/env bash
# notwide_hold.sh <日志…>：每条 DIAG 里第一次被拒的报告（`step=…` 后面第一个 NS[…]，不含推环之后再试的那一段）里没报成「被宽护卡住」的（N），按停因、C 槽数 hold、C 罩着的隔离数 isoC 数
cat "$@" | grep DIAG | sed -nE 's/.*step=[0-9]+[A-Za-z] (NS\[[^]]*\]).*/\1/p' | grep ';N;' | sed -E 's/NS\[([A-Za-z]+)\([^)]*\);N;hold=([0-9]*);isoC=([0-9]*).*/\1 hold=\2 isoC=\3/' | sort | uniq -c
