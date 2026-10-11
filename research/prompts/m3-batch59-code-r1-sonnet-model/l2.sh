#!/usr/bin/env bash
# l2.sh <k> <过滤串|ALL>：walk.rs 旋钮支只判最右之前 k 片（两处：墓碑树、谱系/退役树共用那一处），在自己的拷贝上跑 checker_known_bad_images
# k=0 表示不改（基线）。离最右 d 片（d = 总片数 - 1 - 位置）的容器：d <= k 才判；最右那片恒不判。
set -u
W=/home/fy5090/.local/share/singlefs/work/m3-batch59-code-r1-sonnet
FZ=/home/fy5090/.local/share/singlefs/work/m3-batch59-code-r1/repo
R=crates/singlefs-checker/src/walk.rs
k="$1"; flt="$2"
cp "$FZ/$R" "$W/repo/$R"
if [ "$k" != 0 ]; then
python3 -I - "$k" <<'PY'
import sys
k=int(sys.argv[1]); W='/home/fy5090/.local/share/singlefs/work/m3-batch59-code-r1-sonnet'
p=W+'/repo/crates/singlefs-checker/src/walk.rs'
t=open(p,encoding='utf-8').read()
a="let is_the_rightmost_tombstone_container_under_the_knob =\n                    position_in_leaf_order + 1 == containers.len();"
b="let is_the_rightmost_under_the_knob =\n                    position_in_leaf_order + 1 == container_numbers_and_record_counts.len();"
assert t.count(a)==1 and t.count(b)==1
t=t.replace(a,a[:-1]+f" || position_in_leaf_order + {k+1} < containers.len();")
t=t.replace(b,b[:-1]+f" || position_in_leaf_order + {k+1} < container_numbers_and_record_counts.len();")
open(p,'w',encoding='utf-8').write(t); print('k=',k)
PY
fi
log="$W/logs/l2-k$k-$(echo "$flt" | tr -c 'a-zA-Z0-9\n' _).log"
if [ "$flt" = ALL ]; then
  "$W/run.sh" "$W/repo" "$log" test --offline -p singlefs-harness --test checker_known_bad_images
else
  "$W/run.sh" "$W/repo" "$log" test --offline -p singlefs-harness --test checker_known_bad_images -- "$flt"
fi
echo "== k=$k $flt"; grep -E 'FAILED|test result|exit=|^error' "$log" | sort -u
cp "$FZ/$R" "$W/repo/$R"
