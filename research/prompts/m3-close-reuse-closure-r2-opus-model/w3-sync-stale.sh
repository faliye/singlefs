#!/usr/bin/env bash
# W3：scripts/hosts/source-stamp.py sync（文件头写明不带 --delete）拷两趟：两趟之间源树删掉一份，副本里那一份还在。
# 源树只放树里 scripts/hosts/ 的 .py（够看出「副本不等于源树」）。用法：w3-sync-stale.sh <树> <草稿目录>
set -u
root="$1" scratch="$2/stale"
rm -rf -- "$scratch"; mkdir -p "$scratch/source/scripts/hosts"
cp -- "$root"/scripts/hosts/*.py "$scratch/source/scripts/hosts/"
(cd "$root" && python3 -B scripts/hosts/source-stamp.py sync "$scratch/source" "$scratch/copy" --include scripts) > "$scratch/first.log" 2>&1; first=$?
rm -- "$scratch/source/scripts/hosts/heavy_threads.py"
(cd "$root" && python3 -B scripts/hosts/source-stamp.py sync "$scratch/source" "$scratch/copy" --include scripts) > "$scratch/second.log" 2>&1; second=$?
echo "name=sync_without_delete first_rc=$first second_rc=$second deleted_in_source_still_in_copy=$([[ -e $scratch/copy/scripts/hosts/heavy_threads.py ]] && echo yes || echo no) python_files_only_in_copy=$(diff -rq "$scratch/source/scripts" "$scratch/copy/scripts" | grep -c '^Only in .*copy')"
