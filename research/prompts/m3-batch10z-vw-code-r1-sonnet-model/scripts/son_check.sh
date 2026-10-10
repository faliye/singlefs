#!/usr/bin/env bash
# son_check.sh <日志…>：把 son-* 日志里「列了推环」的行（listed=true）逐行比：出路二文字里的次数 / 槽数 / 放出来几个 对独立量的
# n_field、真推环降的数 real_released，放出来的数与只数 C 那一圈罩着的 iso_field 比大小，以及推完再试的结局。
cat "$@" | grep 'listed=true' | awk '{
  for (i = 1; i <= NF; i++) {
    if ($i ~ /^real_released=/) r = substr($i, 15); if ($i ~ /^text_released=/) t = substr($i, 15);
    if ($i ~ /^text_times=/) a = substr($i, 12); if ($i ~ /^n_field=/) n = substr($i, 9);
    if ($i ~ /^text_slots=/) s = substr($i, 12); if ($i ~ /^iso_field=/) f = substr($i, 11)
  }
  ok = (r == t && a == n && s == n) ? "text=real" : "text!=real"
  gt = (r > f) ? "released>C-only" : "released<=C-only"
  print ok, gt, ($0 ~ /retry=\["ok"\]/) ? "retry=ok" : "retry=refused"
}' | sort | uniq -c
