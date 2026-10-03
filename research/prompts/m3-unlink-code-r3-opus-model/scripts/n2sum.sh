#!/usr/bin/env bash
# 用法：n2sum.sh <日志…>   数 N2 注错扫：起点数、注错点数，按变体与读法分，写账对不上（match=none）与份数对不上的点。
cat "$@" | awk '
/^N2START /{s++; if ($0 ~ /clean=ok/) ok++}
/^N2TALLY /{match($0,/count=[0-9]+/); c=substr($0,RSTART+6,RLENGTH-6)+0; pts+=c; k=$0; sub(/^N2TALLY U=[0-9]+ k=[0-9]+ u=[0-9]+ b=[0-9]+ /,"",k); sub(/ count=[0-9]+$/,"",k); t[k]+=c}
/^N2UNLEDGERED/{u++}
END{printf "starts=%d clean_ok=%d fault_points=%d unledgered=%d\n", s, ok, pts, u; for (k in t) printf "  %s = %d\n", k, t[k]}' | sort
