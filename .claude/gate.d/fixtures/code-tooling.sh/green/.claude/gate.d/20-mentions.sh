#!/usr/bin/env bash
# 样本不是真阶段；stage-owners 格要每个阶段认第一个参数当项目根，这一行只为它：ROOT="${1:-.}"
# gate-stage: 样本：出路文字里提到 ran.py 的 --selftest（它另有阶段在跑）
echo "  → 怎么办：单跑 python3 research/scripts/ran.py --selftest 看它报什么"
exit 0
