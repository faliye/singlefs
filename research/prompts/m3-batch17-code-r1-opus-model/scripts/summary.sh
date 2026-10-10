#!/usr/bin/env bash
# 用法：summary.sh <工作目录 W>   把 W/out 的日志汇总成判定行（rerun.sh compare 拿它与模型目录 out/summary.txt 逐行比）。
set -u
W=$1; O=$W/out; S=$(cd "$(dirname "$0")" && pwd)
fx() { local name=$1; shift; local files=(); for f in "$@"; do [ -e "$f" ] && files+=("$f"); done
  if [ ${#files[@]} -eq 0 ]; then echo "[$name] 没有日志"; return; fi
  bash "$S/complete.sh" "${files[@]}" | bash "$S/fxsum2.sh" /dev/stdin | sed "s/^/[$name] /"; }
fx fxw-narrow $O/fxw-n*.log
echo "[fxw-narrow-by-width] $(bash "$S/complete.sh" $O/fxw-n*.log | awk '$1=="FXPLAN"||$1=="FXSAME"||$1=="FXREFUSED" {u=$3; p=""; for(i=1;i<=NF;i++) if($i ~ /^plan=/) p=substr($i,6); if (p!="w:-/all:z" && p!="w:all/-:z") next; k=u" "p; t[k]++; if ($1=="FXREFUSED") r[k]++} END{for(k in t) printf "%s histories=%d refused=%d; ", k, t[k], r[k]+0}' | tr ';' '\n' | sort | paste -sd';')"
fx fxw-plain $O/fxw-p*.log
fx gacmp-jia $O/gacmp-jia-*.log
fx gacmp-ga $O/gacmp-ga-*.log
for f in $O/pass-*.log; do [ -e "$f" ] && fx "$(basename "$f" .log)" "$f"; done
for a in jia rr ga; do for fam in narrow plain; do
  echo "[cell-$a-$fam] $(bash "$S/complete.sh" $O/cell-$a-*.log | awk -v f=$fam '$1=="START_FAILED" || $2==f' | bash "$S/fxsum2.sh" /dev/stdin | paste -sd' ')"; done; done
echo "[rr-cell] $(grep -h '^FXREFUSED' $O/rr-rebuildpass.log 2>/dev/null | sed 's/.*writes_before_err=/writes_before_err=/' | sort | uniq -c | tr -s ' ' | paste -sd';')"
fx narrower-jia $O/narrower-n*.log
fx narrower-ga $O/narrower-ga-*.log
echo "[floor-raise-streams] $(bash "$S/complete.sh" $O/fxw-n*.log | awk '$1=="FXPLAN" && /floor_raises=1/ {b=0; for(i=1;i<=NF;i++){if($i ~ /^bad=/) b=substr($i,5)+0; if($i ~ /^plan=/) p=substr($i,6)}; n[p]++; if (b>0) nb[p]++} END{for(p in n) printf "%s streams=%d with_bad=%d;", p, n[p], nb[p]+0}' | tr ';' '\n' | sort | paste -sd';')"
fx hold $O/hold-*.log
echo "[hold-screen] $(cat $O/hold-*.log 2>/dev/null | grep '^FXTALLY' | grep ' screen ' | awk '{k=""; for(i=1;i<=NF;i++) if($i=="screen") {for(j=i;j<NF;j++) k=k" "$j; break}; c=$NF; sub("count=","",c); s[k]+=c} END{for(k in s) printf "%s=%d;", k, s[k]}' | tr ';' '\n' | sort | paste -sd';')"
for f in $O/tw-*.log; do [ -e "$f" ] && echo "[twice-$(basename "$f" .log)] $(bash "$S/r2sum.sh" "$f" | grep -E 'starts=|starts_with_bad' | paste -sd' ')"; done
echo "[twice-all] $(cat $O/tw-*.log 2>/dev/null | grep '^R2TALLY' | grep ' twice ' | awk '{c=$NF; sub("count=","",c); k=$0; sub(/.* twice /,"",k); sub(/ count=.*/,"",k); s[k]+=c} END{for(k in s) printf "%s=%d;", k, s[k]}' | tr ';' '\n' | sort | paste -sd';')"
for a in jia nf; do echo "[ab2-$a] $(cat $O/ab-$a-*.log 2>/dev/null | grep '^AB2TALLY' | awk '{c=$NF; sub("count=","",c); k=$0; sub(/^AB2TALLY .* mount_writes=[0-9]+ /,"",k); sub(/ count=.*/,"",k); s[k]+=c} END{for(k in s) printf "%s=%d;", k, s[k]}' | tr ';' '\n' | sort | paste -sd';')"; done
for a in jia nf; do grep -h '^AB2REMOUNT' $O/ab-$a-*.log 2>/dev/null | sed 's/ abandoned_txg.*m1=/ m1=/' | awk '{print $2,$3,$4,$5,$7}' | sort > "$W/ab2-$a.fp"; done
echo "[ab2-streams] jia=$(wc -l < "$W/ab2-jia.fp") nf=$(wc -l < "$W/ab2-nf.fp") differing_lines=$(diff "$W/ab2-jia.fp" "$W/ab2-nf.fp" | grep -c '^[<>]')"
for f in $O/l0-*.log; do [ -e "$f" ] || continue; b=$(basename "$f" .log)
  grep -h '^FXENUM ' "$f" | sed 's/ elapsed=.*//' | sed "s/^/[$b] /"; grep -h '^FXENUMV' "$f" | sed 's/.* count=/count=/' | sed "s/^/[$b] /"; done
echo "[diag] $(awk '/^FXSTART/{s=1} s && /^DIAG/' $O/diag.log 2>/dev/null | sed 's/reclaimed=\[\(.\{0,60\}\).*/reclaimed=[\1…/' | paste -sd';')"
for a in qfj qfm qfu qfr; do echo "[qf-$a] $({ grep -h -E '^test [a-z_0-9]+ \.\.\. FAILED' $O/qf-$a.log 2>/dev/null | sort; grep -h -E '^test result' $O/qf-$a.log 2>/dev/null | sed 's/finished in .*//'; } | paste -sd';')"; done
