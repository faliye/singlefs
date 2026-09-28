#!/usr/bin/env bash
# 样本不是真阶段；stage-owners 格要每个阶段认第一个参数当项目根，这一行只为它：ROOT="${1:-.}"
# 绿样本：基准从共用脚本取；出路文字里提到 merge-base、GATE_BASE 不算
source "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/changed-paths.sh"
base="$(gate_diff_base gate)"
echo "  → 基准不对就看 GATE_BASE 与 merge-base"
gate_changed_paths "$base" untracked || exit 1
