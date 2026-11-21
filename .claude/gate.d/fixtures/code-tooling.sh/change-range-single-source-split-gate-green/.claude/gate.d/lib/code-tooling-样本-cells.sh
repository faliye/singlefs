#!/usr/bin/env bash
# 样本：code-tooling.sh 拆出的库，里面有这一格自己的那一段
# ── 格 change-range-single-source：样本里这一格自己的模式表
BASE_PATTERN = re.compile(r'merge-base|@\{u(pstream)?\}')
LIST_PATTERN = re.compile(r'--name-only|\bls-files\b')
base="$(git merge-base HEAD origin/main)"
# ── 格 other：样本里别的格
echo "别的格"
