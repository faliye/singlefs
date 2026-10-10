#!/usr/bin/env bash
# admission: always 两份纯算术模型，几秒跑完；不读仓、不起 cargo、不上卡
# run-condition: command python3
# 复跑 m3-crash-enumeration-merge-r2 攻方腿的两份模型：decode_alias.py（GPU 判器段内序号的拆法对 CPU，L1-G10、L6）与 count_model.py（L2 挂钟）。
# 用法：bash rerun.sh；两份都退 0、末行各是 MODEL_SELFCHECK … mismatched=0 才算对上。
set -euo pipefail
directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
(cd "$directory" && sha256sum -c --quiet SHA256SUMS)
status=0
for model in decode_alias.py count_model.py; do
  echo "== $model"
  python3 -I "$directory/$model" || status=1
done
exit "$status"
