#!/usr/bin/env bash
# usage: mut.sh <label> <relfile> <oldfile> <newfile> <cargo test args...>
set -uo pipefail
label=$1; rel=$2; oldf=$3; newf=$4; shift 4
W=$HOME/.local/share/singlefs/work/m3-batch56-code-r1-sonnet
F=$HOME/.local/share/singlefs/work/m3-batch56-code-r1/repo
export CARGO_TARGET_DIR=$W/target
cd "$W/copy"
python3 -I - "$rel" "$oldf" "$newf" <<'PY'
import sys
rel, oldf, newf = sys.argv[1:4]
s = open(rel, encoding='utf-8').read()
old = open(oldf, encoding='utf-8').read().rstrip('\n')
new = open(newf, encoding='utf-8').read().rstrip('\n')
n = s.count(old)
if n != 1:
    print(f"ANCHOR HITS {n}"); sys.exit(3)
open(rel, 'w', encoding='utf-8').write(s.replace(old, new))
print("mutated", rel)
PY
rc=$?
if [ $rc -ne 0 ]; then cp "$F/$rel" "$rel"; exit $rc; fi
nice -n 19 bash /home/fy5090/code/singlefs/scripts/capped.sh 10 bash /home/fy5090/code/singlefs/scripts/run-with-memory-cap.sh 10G cargo test --offline "$@" > "$W/logs/$label.log" 2>&1
echo "exit=$?" >> "$W/logs/$label.log"
cp "$F/$rel" "$rel"
cmp "$F/$rel" "$rel" && echo restored
grep -E '^test result|FAILED|failed|panicked at|^error|exit=' "$W/logs/$label.log" | head -20
