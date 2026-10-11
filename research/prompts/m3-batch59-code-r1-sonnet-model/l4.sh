#!/usr/bin/env bash
# l4.sh <变异名|none|row3668> <new|old> ：在自己的拷贝上打变异、换测试文件版本，跑 core_review 那条水位用例，日志落 logs/
# new = 冻结副本里的（第 59 批）；old = HEAD 里的（第 58 批之前，靠标签 13）
set -u
W=/home/fy5090/.local/share/singlefs/work/m3-batch59-code-r1-sonnet
FZ=/home/fy5090/.local/share/singlefs/work/m3-batch59-code-r1/repo
T=crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs
R=crates/singlefs-core/src/recovery.rs
mut="$1"; ver="$2"
cp "$FZ/$R" "$W/repo/$R"; cp "$FZ/$T" "$W/repo/$T"
[ "$ver" = old ] && cp "$W/model/core_review_old.rs" "$W/repo/$T"
case "$mut" in
  none) ;;
  row3668) python3 -I "$W/apply_row.py" "$W/repo" "$FZ/crates/mutations.tsv" 3668 ;;
  *) python3 -I "$W/apply_row.py" --custom "$W/repo" "$R" "$W/model/mut/$mut.orig" "$W/model/mut/$mut.new" ;;
esac || { echo "apply failed"; exit 7; }
log="$W/logs/l4-$mut-$ver.log"
"$W/run.sh" "$W/repo" "$log" test --offline -p singlefs-harness --test core_review_tree_table_duplicates_and_slot_one_search -- a_version_whose_own_inode_number_watermark_row
echo "== $mut $ver"; grep -E '^test |test result|exit=|^error' "$log"; grep -A4 panicked "$log" | head -12
cp "$FZ/$R" "$W/repo/$R"; cp "$FZ/$T" "$W/repo/$T"
