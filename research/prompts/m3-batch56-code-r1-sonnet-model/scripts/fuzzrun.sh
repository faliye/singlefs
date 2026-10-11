#!/usr/bin/env bash
# usage: fuzzrun.sh <label> <old|new>
set -uo pipefail
label=$1; mode=$2
W=$HOME/.local/share/singlefs/work/m3-batch56-code-r1-sonnet
F=$HOME/.local/share/singlefs/work/m3-batch56-code-r1/repo
export CARGO_TARGET_DIR=$W/target
cd "$W/copy"
if [ "$mode" = old ]; then
python3 -I - <<'PY'
import os,re
W=os.path.expanduser('~/.local/share/singlefs/work/m3-batch56-code-r1-sonnet')
F=os.path.expanduser('~/.local/share/singlefs/work/m3-batch56-code-r1/repo')
rows=[l.rstrip('\n') for l in open(F+'/crates/mutations.tsv',encoding='utf-8') if l.strip() and not l.startswith('#')]
sel=[r.split('\t') for r in rows if re.match(r'^第56批 G1',r)]
assert len(sel)==2
p='crates/singlefs-core/src/recovery.rs'
s=open(p,encoding='utf-8').read()
for r in sel:
    old=r[2].replace('\\n','\n'); new=r[3].replace('\\n','\n')
    assert s.count(old)==1
    s=s.replace(old,new)
open(p,'w',encoding='utf-8').write(s)
print('old-order applied')
PY
fi
PROBE_IDX=${PROBE_IDX:-0} PROBE_OUT=$W/logs/$label.out nice -n 19 bash /home/fy5090/code/singlefs/scripts/hosts/capped.sh 10 bash /home/fy5090/code/singlefs/scripts/hosts/run-with-memory-cap.sh 10G cargo test --offline -p singlefs-harness --test zz_probe_k1_diff -- zz_probe_k1_member_listing --nocapture > $W/logs/$label.log 2>&1
echo "exit=$?" >> $W/logs/$label.log
cp "$F/crates/singlefs-core/src/recovery.rs" crates/singlefs-core/src/recovery.rs
cmp "$F/crates/singlefs-core/src/recovery.rs" crates/singlefs-core/src/recovery.rs && echo restored
grep -E 'PROBE|^error|test result|panicked' $W/logs/$label.log | cut -c1-200
