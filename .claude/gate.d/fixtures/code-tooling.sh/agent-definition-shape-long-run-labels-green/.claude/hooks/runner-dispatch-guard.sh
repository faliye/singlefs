#!/usr/bin/env bash
# 样本：派发闸的替身，只放 ⑭ 的类型表那两句（多行集合字面量，再用 | 并进另一句；code-tooling.sh 的 agent-definition-shape 格照 python 语法读它）
python3 - <<'PY'
HEAVY_SAMPLE_AGENTS = {"alpha",
                       "gamma"}   # gamma 没有定义，不归这一格
LONG_RUN_AGENTS = {"delta"} | HEAVY_SAMPLE_AGENTS   # 样本的类型表
PY
