#!/usr/bin/env bash
# 红样本：阶段自己取 merge-base，不走共用脚本
base="$(git merge-base HEAD '@{upstream}')"
echo "  → 出路文字里提到 merge-base 不算"
git -c core.quotepath=false diff --name-only "$base" --
