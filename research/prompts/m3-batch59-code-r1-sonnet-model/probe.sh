#!/usr/bin/env bash
# probe.sh <base|second_to_last|k2> ：换 walk.rs 的旋钮支，跑版本级探针 probe_m1_ring，PROBE 行落 logs/probe-<变体>.txt
set -u
W=/home/fy5090/.local/share/singlefs/work/m3-batch59-code-r1-sonnet
FZ=/home/fy5090/.local/share/singlefs/work/m3-batch59-code-r1/repo
R=crates/singlefs-checker/src/walk.rs
v="$1"
cp "$FZ/$R" "$W/repo/$R"
python3 -I - "$v" <<'PY'
import sys
v=sys.argv[1]; W='/home/fy5090/.local/share/singlefs/work/m3-batch59-code-r1-sonnet'
p=W+'/repo/crates/singlefs-checker/src/walk.rs'
t=open(p,encoding='utf-8').read()
a="let is_the_rightmost_tombstone_container_under_the_knob =\n                    position_in_leaf_order + 1 == containers.len();"
b="let is_the_rightmost_under_the_knob =\n                    position_in_leaf_order + 1 == container_numbers_and_record_counts.len();"
assert t.count(a)==1 and t.count(b)==1
if v=='second_to_last':
    t=t.replace(a,a[:-1]+" || position_in_leaf_order + 2 != containers.len();")
    t=t.replace(b,b[:-1]+" || position_in_leaf_order + 2 != container_numbers_and_record_counts.len();")
elif v=='k2':
    t=t.replace(a,a[:-1]+" || position_in_leaf_order + 3 < containers.len();")
    t=t.replace(b,b[:-1]+" || position_in_leaf_order + 3 < container_numbers_and_record_counts.len();")
elif v!='base': raise SystemExit('variant?')
open(p,'w',encoding='utf-8').write(t); print('walker variant',v)
PY
"$W/run.sh" "$W/repo" "$W/logs/probe-$v.log" test --offline -p singlefs-harness --test probe_m1_ring -- --nocapture --test-threads 1
grep -E '^PROBE|test result|exit=|^error|panicked' "$W/logs/probe-$v.log" > "$W/logs/probe-$v.txt"
echo "== $v: $(grep -c '^PROBE' "$W/logs/probe-$v.txt") PROBE lines; $(grep -E 'test result|exit=' "$W/logs/probe-$v.txt" | tr '\n' ' ')"
cp "$FZ/$R" "$W/repo/$R"
