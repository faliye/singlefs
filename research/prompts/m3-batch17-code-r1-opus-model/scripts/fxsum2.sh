#!/usr/bin/env bash
# 用法：fxsum2.sh <日志…>   MODE=fx 汇总（同一条写流只判一次，FXSAME 那一行照它 same_stream_as 那份计划的坏判定数记）：
# 按计划（窗口里的序号去掉，只留选槽法与读法）数：再挂历史数、任何写之前被拒、落了写之后被拒、出过坏判定的历史、坏判定数、出过坏判定的起点。
cat "$@" | awk '
function cls(p,  q) { q=p; sub(/^r[0-9]+-[0-9a-z]+/,"r",q); return q }
function planof(  i) { for (i=1;i<=NF;i++) if ($i ~ /^plan=/) return substr($i,6); return "" }
function nof(  i) { for (i=1;i<=NF;i++) if ($i ~ /^n=/) return substr($i,3); return "" }
/^FXSTART/ {lab=$0; sub(/^FXSTART /,"",lab); sub(/ \|.*/,"",lab); cur=lab; if (!(lab in st)) {st[lab]=1; starts++}}
/^START_FAILED/ {f=$0; sub(/:.*/,"",f); if (!(f in sf)) {sf[f]=1; failed++}}
/^FXREFUSED/ {p=planof(); q=cls(p); key=cur SUBSEP nof() SUBSEP p; if (key in seen) next; seen[key]=1; hist[q]++; if ($0 ~ /writes_before_err=0 /) rb[q]++; else ra[q]++; refused[key]=1}
/^FXPLAN/ {p=planof(); q=cls(p); b=0; for (i=1;i<=NF;i++) if ($i ~ /^bad=/) b=substr($i,5)+0; key=cur SUBSEP nof() SUBSEP p; badof[key]=b; if (!(key in refused)) { if (key in seen) next; seen[key]=1; hist[q]++ } add(q,b)}
/^FXSAME/ {p=planof(); q=cls(p); x=$0; sub(/.*same_stream_as=/,"",x); key=cur SUBSEP nof() SUBSEP p; src=cur SUBSEP nof() SUBSEP x; b=badof[src]+0; if (!(key in refused)) { if (key in seen) next; seen[key]=1; hist[q]++ } add(q,b)}
function add(q,b,  k) { if (b>0) { hb[q]++; bj[q]+=b; k=q SUBSEP cur; if (!(k in sb)) {sb[k]=1; startsbad[q]++} } }
END {
  printf "starts=%d start_failed=%d\n", starts, failed
  for (q in hist) printf "plan=%s histories=%d refused_before_any_write=%d refused_after_writes=%d histories_with_bad=%d bad_prefix_judgements=%d starts_with_bad=%d\n", q, hist[q], rb[q]+0, ra[q]+0, hb[q]+0, bj[q]+0, startsbad[q]+0
}' | sort
