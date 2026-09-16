#!/usr/bin/env bash
# 抽四个作业重跑一遍，与产物里对应的行逐字节比：C、E、A_bfs0_bing_T6、B_5。
set -euo pipefail
cd "$(dirname "$0")"
out="${1:-c161-r3-model-output.txt}"
tmp=$(mktemp -d "${TMPDIR:-/tmp}/c161-r3-recheck.XXXXXX")
nice -n 19 python3 -B c161_r3_mono.py 16 | grep '^part=C' > "$tmp/C.txt"
nice -n 19 python3 -B c161_r3_cost.py | grep '^part=E' > "$tmp/E.txt"
nice -n 19 python3 -B c161_r3_tree.py bfs bing_T6 7 0 | grep '^part=A' > "$tmp/A.txt"
nice -n 19 python3 -B c161_r3_switch.py 8 6 5 | grep '^part=B' > "$tmp/B.txt"
grep '^part=C' "$out" > "$tmp/C.ref"
grep '^part=E' "$out" > "$tmp/E.ref"
grep '^part=A mode=bfs arm=bing_T6 start=.* publish=0 ' "$out" > "$tmp/A.ref"
grep '^part=B arm=yi_G_T/switch=reload/redo=retake' "$out" > "$tmp/B.ref"
for name in C E A B; do
  cmp "$tmp/$name.txt" "$tmp/$name.ref" && echo "✓ $name 逐字节相同（$(wc -l < "$tmp/$name.txt") 行）"
done
rm -rf "${tmp:?}"
