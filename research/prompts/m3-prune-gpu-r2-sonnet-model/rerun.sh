#!/usr/bin/env bash
# 重跑云端正推（Sonnet）这一轮唯一的模型：核 B1 的重叠签名设计。
# admission: always 每次调都重新核一遍这份脚本与它读的 kinds.out 此刻的样子
# run-condition: command python3
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"

echo "核 SHA256SUMS："
sha256sum -c SHA256SUMS

echo "重跑 derive_b1_overlap_signature.py："
actual="$(python3 derive_b1_overlap_signature.py)"
expected="$(cat derive_b1_overlap_signature.out)"
if [[ "$actual" == "$expected" ]]; then
  echo "derive_b1_overlap_signature 与存档逐行相同"
else
  echo "derive_b1_overlap_signature 与存档不同："
  diff <(echo "$expected") <(echo "$actual") || true
  exit 1
fi
