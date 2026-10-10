#!/usr/bin/env bash
# 用法：compare.sh <模型目录的 out> <复跑的 out>   复跑的日志按同样的命令重出汇总，与模型目录里的逐行比；判定行相同记 ✓，不同记 ✗，有 ✗ 退出 1。
set -u
A=$1; B=$2
S=$(cd "$(dirname "$0")" && pwd)
bad=0
same() { # <名> <文件甲> <文件乙>
  if diff -q "$2" "$3" > /dev/null; then echo "✓ $1 判定行相同（$(wc -l < "$2") 行）"; else echo "✗ $1 不同"; diff "$2" "$3" | head -5; bad=1; fi
}
T=$(mktemp -d)
{ for a in ga imm held; do echo "narrow $a: $(bash "$S/summarize.sh" "$B"/n1n-$a-*.log)"; done
  for a in ga imm; do echo "cycle $a: $(bash "$S/summarize.sh" "$B"/n1c-$a*.log)"; echo "ncycle $a: $(bash "$S/summarize.sh" "$B"/n1nc-$a*.log)"; done
  for a in ga imm held; do echo "abandon $a: $(bash "$S/summarize.sh" "$B/n1a-$a.log")"; done; } > "$T/n1"
same n1-rescan-summary "$A/n1-rescan-summary.txt" "$T/n1"
{ for fam in rn rp; do for a in ga imm held fx; do echo "== $fam $a"; bash "$S/rsummarize.sh" "$B"/$fam-$a-*.log; done; done; } > "$T/r"
same remount-summary "$A/remount-summary.txt" "$T/r"
awk '/^START abandon/{match($0,/abandoned_txg=[0-9]+/); a=substr($0,RSTART+14,RLENGTH-14)} /^CUT/{match($0,/chosen=Some\([0-9]+\)/); c=substr($0,RSTART+12,RLENGTH-13); if (c==a) n++; t++} END{print "abandon ga: cut_verdicts=" t, "chosen_the_abandoned_root=" n+0}' "$B/n1a-ga.log" > "$T/ab"
grep '^START abandon' "$B/n1a-ga.log" | grep -o 'unmount=Err [A-Za-z]*([A-Za-z]*' | sort | uniq -c >> "$T/ab"
same abandon-chosen "$A/abandon-chosen.txt" "$T/ab"
grep -h '^QF' "$A/qf-ga.log" > "$T/qa"; grep -h '^QF' "$B/qf-ga.log" > "$T/qb"; same qf "$T/qa" "$T/qb"
grep -h '^RENUM' "$A/renum-h2-ga.log" | sort > "$T/ea"; grep -h '^RENUM' "$B"/renum-h2-ga-*.log | sort > "$T/eb"; same renum-h2 "$T/ea" "$T/eb"
grep -h '^RCUT\|^RTALLY' "$A/r-h2-ga-detail.log" > "$T/da"; grep -h '^RCUT\|^RTALLY' "$B/r-h2-ga-detail.log" > "$T/db"; same r-h2-detail "$T/da" "$T/db"
grep -h '^RTALLY\|^RCUT' "$A/rp-ga-4-1.log" | sed 's/ survivor_sc_floor=[^ ]*//' > "$T/pa"; grep -h '^RTALLY\|^RCUT' "$B/rp-ga-4-1.log" | sed 's/ survivor_sc_floor=[^ ]*//' > "$T/pb"; same rp-ga-4-1 "$T/pa" "$T/pb"
for t in mq1 mq3 mg1 mg4 fx; do
  grep -hE '^test .* \.\.\. (ok|FAILED|ignored)$' "$A/tests-$t.log" | sort > "$T/ta"; grep -hE '^test .* \.\.\. (ok|FAILED|ignored)$' "$B/tests-$t.log" | sort > "$T/tb"
  same "tests-$t" "$T/ta" "$T/tb"
done
rm -rf "$T"
exit $bad
