#!/usr/bin/env bash
# runfam.sh <族名> [树名…] : 对 cases/cases-<族名>.json 跑一遍裁判（res/<族>.oracle.json），再在每棵树上喂钩子（res/<族>.<树>.json）。已有的结果不重跑。
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; fam="$1"; shift
trees=("$@"); ((${#trees[@]})) || trees=(r1 r2 now)
cd "$here"
[[ -s res/$fam.oracle.json ]] || nice -n 19 python3 -I tools/myoracle.py cases/cases-$fam.json res/$fam.oracle.json 20
for t in "${trees[@]}"; do
  [[ -s res/$fam.$t.json ]] || nice -n 19 python3 -I tools/feedhooks.py cases/cases-$fam.json "$t" res/$fam.$t.json 20
done
