#!/usr/bin/env bash
# 绿样本：基准从共用脚本取；出路文字里提到 merge-base、GATE_BASE 不算
source "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/changed-paths.sh"
base="$(gate_diff_base gate)"
echo "  → 基准不对就看 GATE_BASE 与 merge-base"
gate_changed_paths "$base" untracked || exit 1
