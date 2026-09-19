#!/usr/bin/env bash
set -u
base=/tmp/claude-1000/m2-supp3-item1/r3
cd $base
SWEEP_POINTS="reuse:96:20,reuse:96:40,broad:192:20,broad:96:30,broad:96:20,reuse:48:30,reuse:48:20,broad:48:30" python3 sweep-extra.py row130 > sweep-extra-130.stdout 2>&1
SWEEP_POINTS="broad:96:30,reuse:48:30,broad:96:20,reuse:48:20,broad:96:40,reuse:48:40,reuse:96:30,broad:48:30" python3 sweep-extra.py ya yb > sweep-extra-yab.stdout 2>&1
echo done >> $base/extra.done
