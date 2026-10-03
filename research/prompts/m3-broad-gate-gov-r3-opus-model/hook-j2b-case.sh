#!/usr/bin/env bash
# 攻方 J2-b 的现场：在 gate-verdict-escalated-green 造好之后改一处，造一条驱动不会写、门禁该判红的登记行（或 --summary 输出）。CASE=<名字> 选哪一处。
# 改了日志的都按 sha256sum 现算回填登记表，只留下要判的那一处不对。
set -euo pipefail
v=e5ca1a7e000000000000000000000000000000000000000000000000000000f1
home=.broad-sample/home; vd="$home/versions/$v"; reg="$home/escalated-verdicts.tsv"
one_first=escalation/1.one-build.100.first.log; one_escalated=escalation/1.one-build.100.escalated.log
log() {   # <相对路径> <包装那一句或空> <哪一趟> <限时> <退出码>
  { printf '   Compiling singlefs-core（攻方现场）\n'; [[ -n "$2" ]] && printf 'run-with-memory-cap: %s（攻方现场）\n' "$2"
    printf 'BROAD_MUTATION_EVIDENCE version=%s sequence=1 step=one-build attempt=%s limit=%s exit=%s\n' "$v" "$3" "$4" "$5"; } > "$vd/$1"; }
notrailer() { printf '   Compiling singlefs-core（攻方现场）\nrun-with-memory-cap: %s（攻方现场）\n' "$2" > "$vd/$1"; }
digest() { sha256sum "$vd/$1" | cut -d' ' -f1; }
row_one() {   # <基线> <原限时> <升档限时> <first_log> <escalated_log> [多出的尾段，空就是照常 15 段]
  printf '%s\t1\tcrates/singlefs-core/src/lib.rs:2:5: replace the value with MUTATED_1()\tcaught-by-timeout\tone-build\t%s\t8\t%s\t%s\t20\t%s\t%s\t%s\t%s%s\n' \
    "$v" "$1" "$2" "$3" "$4" "$(digest "$4")" "$5" "$(digest "$5")" "${6-	2026-10-03}"; }
replace_row_one() { grep -v -P "^$v\t1\t" "$reg" > "$reg.new"; printf '%s' "$1" >> "$reg.new"; mv "$reg.new" "$reg"; }
case "$CASE" in
  i03-baseline-lie)      printf 'one-build\tok\t300.0\t2026-10-03\n' >> "$vd/baseline.tsv"
                         replace_row_one "$(row_one 1.0 900 3600 $one_first $one_escalated)" ;;
  i05-not-larger)        log $one_first "超过限时 3600 秒" first 3600 253; log $one_escalated "超过限时 3600 秒" escalated 3600 253
                         replace_row_one "$(row_one 10.0 3600 3600 $one_first $one_escalated)" ;;
  i06m-memory-multiple)  printf '# 攻方\n21474836480\tx\t24G\tbroad-mutation baseline one-test\t8\n' > "$vd/baseline-peaks.8.tsv"
                         sed -i "s/\tcaught-by-memory\tone-test\t1073741824\t/\tcaught-by-memory\tone-test\t21474836480\t/" "$reg" ;;
  i07-no-trailer)        notrailer $one_first "超过限时 900 秒"; notrailer $one_escalated "超过限时 3600 秒"
                         replace_row_one "$(row_one 10.0 900 3600 $one_first $one_escalated)" ;;
  i07b-trailer-wrong)    log $one_first "超过限时 900 秒" first 900 0; log $one_escalated "超过限时 3600 秒" first 3600 0
                         replace_row_one "$(row_one 10.0 900 3600 $one_first $one_escalated)" ;;
  i08-no-wrapper)        log $one_first "" first 900 253; log $one_escalated "" escalated 3600 253
                         replace_row_one "$(row_one 10.0 900 3600 $one_first $one_escalated)" ;;
  i09-columns)           replace_row_one "$(row_one 10.0 900 3600 $one_first $one_escalated '')" ;;
  i10-no-escalated-field) sed -i 's/ escalated=2 / /' .broad-sample/summary.sh ;;
  i11-name-differs)      sed -i 's/\(ESCALATED\\t1\\tcaught-by-timeout\\t\)crates[^\\]*MUTATED_1()/\1crates\/singlefs-core\/src\/lib.rs:9:9: some other mutant/' .broad-sample/summary.sh ;;
  i14-logs-elsewhere)    mkdir -p "$vd/logs"; mv "$vd/$one_first" "$vd/logs/1.first.log"; mv "$vd/$one_escalated" "$vd/logs/1.escalated.log"
                         replace_row_one "$(row_one 10.0 900 3600 logs/1.first.log logs/1.escalated.log)" ;;
  *) echo "认不出 CASE=$CASE" >&2; exit 2 ;;
esac
