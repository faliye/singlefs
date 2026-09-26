#!/usr/bin/env bash
# 复跑这一条腿的全部模型：bash run-all.sh <草稿目录>
# 只跑 bash、python3、git；临时仓建在 <草稿目录> 下，不碰仓里任何文件，不编译、不跑任何门禁阶段。
set -uo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"; draft="$1"; mkdir -p "$draft"
rm -f "$draft/hooks/detections.jsonl"
echo "#### g1-judge.py"; python3 "$here/g1-judge.py" "$repo"; echo "rc=$?"
echo "#### g2-untracked.sh"; bash "$here/g2-untracked.sh" "$repo" "$draft"; echo "rc=$?"
echo "#### g3-git-blind.sh"; bash "$here/g3-git-blind.sh" "$draft"; echo "rc=$?"
echo "#### g3-who-records-time.sh"; bash "$here/g3-who-records-time.sh" "$repo"; echo "rc=$?"
echo "#### g4-driver-name.sh"; bash "$here/g4-driver-name.sh" "$repo"; echo "rc=$?"
echo "#### g5-exit-two.sh"; bash "$here/g5-exit-two.sh" "$repo"; echo "rc=$?"
echo "#### run-hooks.sh"; bash "$here/run-hooks.sh" "$draft/hooks"; echo "rc=$?"
