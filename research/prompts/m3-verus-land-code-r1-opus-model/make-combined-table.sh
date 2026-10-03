#!/usr/bin/env bash
# 用法：make-combined-table.sh <冻结副本根> > combined.tsv
# 拼出变异批次用的总表：own-mutations.tsv、sampling-fix.tsv、crates/mutations.tsv 里点名核的 16 行（改名 T<行号>）、一行不改东西的 NONE。
set -euo pipefail
tree=$1
here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
grep -v '^#' "$here/own-mutations.tsv"
grep -v '^#' "$here/sampling-fix.tsv"
for n in 24 36 226 355 448 512 513 514 515 516 613 614 920 921 1496 1497; do
    awk -F'\t' -v n="$n" 'BEGIN { OFS = "\t" } NR == n { $1 = "T" n; print }' "$tree/crates/mutations.tsv"
done
printf 'NONE\tcrates/singlefs-replay-prefix/src/lib.rs\t#![forbid(unsafe_code)]\t#![forbid(unsafe_code)]\n'
