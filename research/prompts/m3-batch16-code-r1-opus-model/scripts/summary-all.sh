#!/usr/bin/env bash
# 用法：summary-all.sh <工作目录 W>   把 W/out 里各组日志汇总成判定行（rerun.sh compare 拿它与模型目录 out/summary.txt 逐行比）。
set -u
W=$1; S=$(cd "$(dirname "$0")" && pwd); O=$W/out
group() { local name=$1; shift; local files=(); for f in "$@"; do [ -e "$f" ] && files+=("$f"); done
  if [ ${#files[@]} -eq 0 ]; then echo "[$name] 没有日志"; return; fi
  bash "$S/r2sum.sh" "${files[@]}" | sed "s/^/[$name] /"; }
group r2-narrow-jia $O/a-jia-*.log
group r2-narrow-ga $O/a-ga-*.log
group r2-narrower-jia $O/c-jia-*.log
group r2-narrower-ga $O/c-ga-*.log
group r2-plain-jia $O/p-jia-w*.log
group r2-plain-ga $O/p-ga-w*.log
group r2-narrow-jia-hide1 $O/b-jia-h1-*.log
group r2-narrow-jia-hide0-half $O/b-jia-h0-*.log
group r2-narrow-jia-hide1err-half $O/b-jia-e1-*.log
group r2-narrow-nh-hide1 $O/b-nh-h1-*.log $O/b-nh-h1u*.log
group r2-plain-small-jia-hide1 $O/q-jia-h1-w*.log
group r2-plain-small-nh-hide1 $O/q-nh-h1-w*.log
for f in $O/tw-*.log; do group "twice-$(basename "$f" .log)" "$f"; done
for f in $O/l1-*.log $O/l2-*.log $O/e1-*.log $O/e2-*.log; do [ -e "$f" ] || continue
  grep -h '^R2ENUM' "$f" | sed 's/elapsed=[0-9.]*s//' | sed "s/^/[$(basename "$f" .log)] /"; done
for fam in s2n s2nc s2c; do for arm in jia imm; do echo "[n1-$fam-$arm] $(bash "$S/n1sum.sh" $O/$fam-$arm-*.log)"; done; done
for arm in jia imm; do echo "[n1-s2a-$arm] $(bash "$S/n1sum.sh" $O/s2a-$arm.log)"
  awk '/^START abandon/{match($0,/abandoned_txg=[0-9]+/); a=substr($0,RSTART+14,RLENGTH-14)} /^CUT/{n++; if ($0 ~ ("chosen=Some\\(" a "\\)")) c++} END{print "[abandon-chosen-'$arm'] cut_verdicts=" n " chosen_the_abandoned_root=" c+0}' $O/s2a-$arm.log; done
echo "[qf-jia] $(cat $O/qf-jia-*.log | grep -c '^QF ') qf_lines; $(cat $O/qf-jia-*.log | awk '/^QFTALLY/{k=$2" "$3; sub(/count=/,"",$4); s[k]+=$4} END{for(k in s) printf "%s=%d ", k, s[k]}')"
echo "[qf-n7clamp] $(grep '^QFTALLY' $O/qf-n7clamp.log)"
echo "[unmount-hide] $(cat $O/uh-*.log | grep '^UNMOUNT_HIDE' | sed 's/.*err_mode=[01]* //; s/{.*//' | sort | uniq -c | tr -s ' ')"
echo "[tests-n7clamp] $(grep -h '^test result' $O/tests-n7clamp.log | sed 's/finished in .*//' | sort | uniq -c | tr -s ' ' | tr '\n' ';')"
