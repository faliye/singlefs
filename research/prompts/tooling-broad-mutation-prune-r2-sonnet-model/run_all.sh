#!/bin/bash
# 只读：git show HEAD:<文件>、库目录的 list.json / results.tsv / logs；不起 cargo、不起驱动。从本目录跑。
set -u
cd "$(dirname "$0")"
mkdir -p out
for s in parse_list res commits constgraph constonly fmtrefs fmtitem s2 s2b s4 derive ratios2 grid calm sim t_inline v3; do
  nice -n 19 python3 $s.py > out/$s.txt 2>&1; echo "$s exit $?"
done
bash globals.sh > out/globals.txt 2>&1; echo "globals exit $?"
nice -n 19 python3 regimes.py > out/regimes.txt 2>&1; echo "regimes exit $?"
