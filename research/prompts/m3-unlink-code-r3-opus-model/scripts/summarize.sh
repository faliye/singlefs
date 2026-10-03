#!/usr/bin/env bash
# 用法：summarize.sh <日志…>   数起点、卸载结局、交集、旧根上整读失败。
cat "$@" | awk '
/^START /{n++; s=$2" "$3" "$4" "$5" "$6" "$7; if ($0 ~ /unmount=Err/) err++; if ($0 ~ /unmount=Raised/) ok++; match($0,/current_txg=[0-9]+/); c=substr($0,RSTART+12,RLENGTH-12)+0}
/^START_FAILED/{f++}
/^INTERSECTION_BY_RECORDS/{if (!(s in hit)) {hit[s]=1; h++}; match($0,/count=[0-9]+/); slots+=substr($0,RSTART+6,RLENGTH-6)}
/^CUT/ && /whole_read=differs\[(ro|open|read)/{match($0,/chosen=Some\([0-9]+\)/); ch=substr($0,RSTART+12,RLENGTH-13)+0; match($0,/lost=[a-z0-9]+/); l=substr($0,RSTART,RLENGTH); if (ch<=c) {k=s" "l; if (!(k in bad)) {bad[k]=1; b++}; cuts++}}
END{printf "starts=%d start_failed=%d unmount_raised=%d unmount_err=%d starts_with_intersection=%d intersection_slot_lines_sum=%d start_lost_pairs_with_old_root_read_failure=%d such_cuts=%d\n", n, f, ok, err, h, slots, b, cuts}'
