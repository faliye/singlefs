#!/usr/bin/env bash
# 探针：造一道只有三格的合成阶段，source 真的 .claude/gate.d/lib/stage-cells.sh，格的函数调 l3-cell-modes.py 的三种模式，
# 看这一道的整道输出（含共用库打的汇总 ✗ 行与 → 行）长什么样。不起 Verus、不起真包装、不是真阶段。
# 用法：bash l3-synthetic-stage.sh <仓根> <草稿目录> <要跑的格，逗号分隔：vac-notrun,vac-mixed,vac-red>
root="$1"; scratch="$2"; cells="$3"
here="$(cd "$(dirname "$0")" && pwd)"
syn="$scratch/syn/.claude/gate.d"; mkdir -p "$syn/lib"
cp "$root/.claude/gate.d/lib/stage-cells.sh" "$syn/lib/stage-cells.sh"
cat > "$syn/synthetic-verus.sh" <<'STAGE'
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/lib/stage-cells.sh"
HERE="@HERE@"; ROOT0="@ROOT@"; SCR="@SCR@"
cell_a() { python3 "$HERE/l3-cell-modes.py" "$ROOT0" "$SCR" notrun | grep -v '^##'; return "${PIPESTATUS[0]}"; }
cell_b() { python3 "$HERE/l3-cell-modes.py" "$ROOT0" "$SCR" mixed | grep -v '^##'; return "${PIPESTATUS[0]}"; }
cell_c() { python3 "$HERE/l3-cell-modes.py" "$ROOT0" "$SCR" red | grep -v '^##'; return "${PIPESTATUS[0]}"; }
stage_cell vac-notrun cell_a "全是没跑成的一格" "✗ 行写「没跑成：内存包装」的照 research/scripts/run-with-memory-cap.sh 文件头「退出码」一段办，不改前置条件"
stage_cell vac-mixed cell_b "混合格" "✗ 行写「没跑成：内存包装」的照 research/scripts/run-with-memory-cap.sh 文件头「退出码」一段办，不改前置条件"
stage_cell vac-red cell_c "全是真红的一格" "✗ 行写「没跑成：内存包装」的照 research/scripts/run-with-memory-cap.sh 文件头「退出码」一段办，不改前置条件"
stage_cells_parse "$@"
stage_cells_run "$PWD"
STAGE
sed -i "s#@HERE@#$here#; s#@ROOT@#$root#; s#@SCR@#$scratch#" "$syn/synthetic-verus.sh"
cd "$scratch/syn" && bash "$syn/synthetic-verus.sh" --check "$cells"
echo "stage exit=$?"
