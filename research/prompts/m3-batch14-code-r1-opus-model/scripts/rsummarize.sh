#!/usr/bin/env bash
# 用法：rsummarize.sh <日志…>   数形 R：起点数、卸载结局、再挂被拒、有「丢盘之后读不出/读出不同」的起点数与（刀 n, 刀 m, 丢盘）组数，按交集分。
cat "$@" | awk '
/^START_FAILED/{f++}
/^RSTART /{n++; if ($0 ~ /unmount=Err/) err++; if ($0 ~ /unmount=Raised/) ok++}
/^RREMOUNT_ERR/{rerr++}
/^RREMOUNT /{rem++}
/^RTALLY /{
  s=$0; sub(/^RTALLY /,"",s); sub(/ prefix .*/,"",s);
  k=$0; sub(/^.* prefix /,"",k); sub(/ count=[0-9]+$/,"",k);
  match($0,/count=[0-9]+/); c=substr($0,RSTART+6)+0; t[k]+=c;
  if (k ~ /read=differs/) { if (!(s in hit)) {hit[s]=1; h++} }
  if (k ~ /inter_remount=1 read=differs/) { if (!(s in hitr)) {hitr[s]=1; hr++} }
}
END{
  printf "starts=%d start_failed=%d unmount_raised=%d unmount_err=%d remounts=%d remount_err=%d starts_with_lone_read_differs=%d of_which_with_remount_intersection=%d\n", n, f, ok, err, rem, rerr, h, hr;
  for (k in t) printf "  %s = %d\n", k, t[k]
}' | sort
