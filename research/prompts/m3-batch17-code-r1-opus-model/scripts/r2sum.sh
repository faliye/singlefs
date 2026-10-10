#!/usr/bin/env bash
# 用法：r2sum.sh <日志…>   汇总 MODE=r2 的日志：起点数、起不来的起点、再挂被拒、各类判定计数、出过坏判定的起点数。
set -u
cat "$@" | awk '
/^R2START/ {starts++}
/^START_FAILED/ {failed++}
/^R2REMOUNT_ERR/ {rerr++; split($0, a, "} "); }
/^R2REMOUNT / {remounts++}
/^R2TALLY/ {
  line=$0; sub(/^R2TALLY .* (prefix|twice|remount_err|twice_remount_err) /, "& ", line)
  n=split($0, f, " "); cnt=f[n]; sub(/count=/, "", cnt)
  key=""; for (i=1;i<=n-1;i++) if (f[i] ~ /^(prefix|twice|remount_err|twice_remount_err)$/) {for (k=i;k<=n-1;k++) key=key" "f[k]; break}
  sum[key]+=cnt
  if (key ~ /BAD/) {lab=$0; sub(/^R2TALLY /, "", lab); sub(/ (prefix|twice) j=.*$/, "", lab); bad_start[lab]=1}
}
END {
  printf "starts=%d start_failed=%d remounts_ok=%d remount_err=%d\n", starts, failed, remounts, rerr
  for (k in sum) printf "sum%s count=%d\n", k, sum[k]
  nb=0; for (l in bad_start) nb++; printf "starts_with_bad=%d\n", nb
}' | sort
