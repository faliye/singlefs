#!/usr/bin/env bash
# runfams.sh <SW_ROOT> <families目录> <out目录> : 每个家族文件在 tree-orig 与 tree 两棵树各一个进程喂钩子，最后写 matrix.tsv
set -u
export SW_ROOT="$1"; fams="$2"; out="$3"; here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export SINGLEFS_WORK_ROOT=/var/empty/swr
mkdir -p "$out"
for fam in "$fams"/*.json; do
  n="$(basename "$fam" .json)"
  { nice -n 19 python3 -I "$here/fam4.py" "$fam" tree-orig "$out/$n-orig.json"; echo "$?" > "$out/$n-orig.rc"; } &
  { nice -n 19 python3 -I "$here/fam4.py" "$fam" tree "$out/$n-new.json"; echo "$?" > "$out/$n-new.rc"; } &
done
wait
cat "$out"/*.rc | sort | uniq -c | sed 's/^/exit_codes /'
: > "$out/matrix.tsv"
for fam in "$fams"/*.json; do n="$(basename "$fam" .json)"; python3 -I "$here/matrix.py" "$fam" "$out/$n-orig.json" "$out/$n-new.json" >> "$out/matrix.tsv"; done
LC_ALL=C sort -o "$out/matrix.tsv" "$out/matrix.tsv"
echo "matrix_rows $(wc -l < "$out/matrix.tsv")"
awk -F'|' '{print $4"->"$5}' "$out/matrix.tsv" | sort | uniq -c
