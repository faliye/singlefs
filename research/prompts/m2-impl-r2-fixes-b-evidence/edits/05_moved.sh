#!/usr/bin/env bash
# 合入后验证一打不上的 4 份挪走文件：照它 crates-moved.patch 里的改法（路径换成 singlefs-checker-tier）打，崩溃注入那份 use 行另改。
set -euo pipefail
root=$1
here=$(dirname "$(readlink -f "$0")")
cd "$root"
for f in crash_enumeration_findings_log_dedupes_by_signature_and_survives_resume.rs record_checker_judges_absence_by_the_persisted_set.rs second_transaction_parallel_line_one_layer0.rs; do
  patch -p1 --no-backup-if-mismatch --forward < "$here/../moved/$f.patch" >/dev/null
done
patch -p1 --no-backup-if-mismatch --forward < "$here/../moved/second_transaction_supplement_three_crash_injection.rs.patch" >/dev/null || true
rm -f crates/singlefs-checker-tier/tests/second_transaction_supplement_three_crash_injection.rs.rej
echo "05_moved done"
