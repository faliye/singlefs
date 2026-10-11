#!/usr/bin/env bash
# usage: mutm.sh <label> <relfile> <pairprefix,pairprefix,...> <cargo test args...>   (each pair: mutdefs/<p>.old / .new)
set -uo pipefail
label=$1; rel=$2; pairs=$3; shift 3
W=$HOME/.local/share/singlefs/work/m3-batch56-code-r1-sonnet
F=$HOME/.local/share/singlefs/work/m3-batch56-code-r1/repo
export CARGO_TARGET_DIR=$W/target
cd "$W/copy"
python3 -I - "$rel" "$W/mutdefs" "$pairs" <<'PY'
import sys
rel, d, pairs = sys.argv[1:4]
s = open(rel, encoding='utf-8').read()
for p in pairs.split(','):
    old = open(f'{d}/{p}.old', encoding='utf-8').read().rstrip('\n')
    new = open(f'{d}/{p}.new', encoding='utf-8').read().rstrip('\n')
    n = s.count(old)
    if n != 1:
        print(f"ANCHOR HITS {n} for {p}"); sys.exit(3)
    s = s.replace(old, new)
open(rel, 'w', encoding='utf-8').write(s)
print("mutated", rel, pairs)
PY
rc=$?
if [ $rc -ne 0 ]; then cp "$F/$rel" "$rel"; exit $rc; fi
nice -n 19 bash /home/fy5090/code/singlefs/scripts/capped.sh 10 bash /home/fy5090/code/singlefs/scripts/run-with-memory-cap.sh 10G cargo test --offline "$@" > "$W/logs/$label.log" 2>&1
echo "exit=$?" >> "$W/logs/$label.log"
cp "$F/$rel" "$rel"; cmp "$F/$rel" "$rel" && echo restored
grep -E '^test result|FAILED|panicked at|^error|exit=' "$W/logs/$label.log" | head -20
