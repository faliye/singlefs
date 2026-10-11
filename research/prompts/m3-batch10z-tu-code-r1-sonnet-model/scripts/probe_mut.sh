#!/usr/bin/env bash
# probe_mut.sh <变异名> : 在 tree-pm 上套 mut3 的一个变异（debug 版 tree-a 的拷贝），编 p10z 探针，跑 a1 a2 a3 批，汇总
set -u
W=$HOME/.local/share/singlefs/work/m3-batch10z-tu-code-r1-sonnet
R=/home/fy5090/code/singlefs/scripts
name=$1
rm -rf "$W/tree-pm"; mkdir -p "$W/tree-pm"; rsync -a --exclude target "$W/tree-a/" "$W/tree-pm/"
python3 - "$name" <<'PY'
import os,sys
W=os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-tu-code-r1-sonnet')
ns={}; exec(open(W+'/scripts/mut3_body.py').read(),ns)
f,old,new=ns['M'][sys.argv[1]]
p=W+'/tree-pm/crates/singlefs-core/src/'+f
s=open(p).read(); assert s.count(old)==1,(sys.argv[1],s.count(old))
open(p,'w').write(s.replace(old,new))
PY
cd "$W/tree-pm"
CARGO_TARGET_DIR=$W/target-pm nice -n 19 bash $R/run-with-memory-cap.sh 12G bash $R/capped.sh 15 cargo test --release --offline -p singlefs-harness --test p10z_probe --no-run 2>&1 | grep -E '^error' -A5 | head
