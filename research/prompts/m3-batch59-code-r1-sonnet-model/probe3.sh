#!/usr/bin/env bash
# probe2.sh：在自己的拷贝里给 walk.rs 旋钮支加环境变量开关（四种变体），release 下跑版本级探针 probe_m1_ring（同一镜像四种判法），PROBE 行落 logs/probe.txt
set -u
W=/home/fy5090/.local/share/singlefs/work/m3-batch59-code-r1-sonnet
FZ=/home/fy5090/.local/share/singlefs/work/m3-batch59-code-r1/repo
R=crates/singlefs-checker/src/walk.rs
cp "$FZ/$R" "$W/repo/$R"
python3 -I - <<'PY'
W='/home/fy5090/.local/share/singlefs/work/m3-batch59-code-r1-sonnet'
p=W+'/repo/crates/singlefs-checker/src/walk.rs'
t=open(p,encoding='utf-8').read()
a="let is_the_rightmost_tombstone_container_under_the_knob =\n                    position_in_leaf_order + 1 == containers.len();"
b="let is_the_rightmost_under_the_knob =\n                    position_in_leaf_order + 1 == container_numbers_and_record_counts.len();"
assert t.count(a)==1 and t.count(b)==1
t=t.replace(a,"let is_the_rightmost_tombstone_container_under_the_knob =\n                    probe_exempt(position_in_leaf_order, containers.len());")
t=t.replace(b,"let is_the_rightmost_under_the_knob =\n                    probe_exempt(position_in_leaf_order, container_numbers_and_record_counts.len());")
t+=open(W+'/model/walk_probe_patch.rs',encoding='utf-8').read()
open(p,'w',encoding='utf-8').write(t)
PY
cp "$W/model/probe_m1_ring.rs" "$W/repo/crates/singlefs-harness/tests/probe_m1_ring.rs"
"$W/run.sh" "$W/repo" "$W/logs/probe.log" test --release --offline -p singlefs-harness --test probe_m1_ring -- --nocapture --test-threads 1
grep -E '^PROBE|test result|exit=|^error|panicked' "$W/logs/probe.log" > "$W/logs/probe.txt"
echo "== $(grep -c '^PROBE' "$W/logs/probe.txt") PROBE lines; $(grep -E 'test result|exit=' "$W/logs/probe.txt" | tr '\n' ' ')"
cp "$FZ/$R" "$W/repo/$R"; rm -f "$W/repo/crates/singlefs-harness/tests/probe_m1_ring.rs"
