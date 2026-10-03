#!/usr/bin/env bash
# 攻方现场 J2-a 续：同上追加一行基线之后，再照门禁出路「--item 1 --rerun 重判那一条」走一遍：驱动在新基线下又升档、两趟又超时、证据齐，
# 照 escalate_step 的写法另写两份日志（末尾带驱动那一行、带包装那一句）、往登记表追加一行（基线 1.2），结果表的新行仍是 caught-by-timeout（--summary 不变）。
set -euo pipefail
v=e5ca1a7e000000000000000000000000000000000000000000000000000000f1
home=.broad-sample/home
bash "$(dirname "$0")/hook-baseline-rerun.sh"
name_one="crates/singlefs-core/src/lib.rs:2:5: replace the value with MUTATED_1()"
for attempt in first escalated; do
  limit=900; [[ $attempt == escalated ]] && limit=3600
  printf '   Compiling singlefs-core（攻方现场）\nrun-with-memory-cap: 超过限时 %s 秒（攻方现场）\nBROAD_MUTATION_EVIDENCE version=%s sequence=1 step=one-build attempt=%s limit=%s exit=253\n' \
    "$limit" "$v" "$attempt" "$limit" > "$home/versions/$v/escalation/1.one-build.300.$attempt.log"
done
d1="$(sha256sum "$home/versions/$v/escalation/1.one-build.300.first.log" | cut -d' ' -f1)"
d2="$(sha256sum "$home/versions/$v/escalation/1.one-build.300.escalated.log" | cut -d' ' -f1)"
printf '%s\t1\t%s\tcaught-by-timeout\tone-build\t1.2\t16\t900\t3600\t20\tescalation/1.one-build.300.first.log\t%s\tescalation/1.one-build.300.escalated.log\t%s\t2026-10-04\n' \
  "$v" "$name_one" "$d1" "$d2" >> "$home/escalated-verdicts.tsv"
