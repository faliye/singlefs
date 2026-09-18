#!/usr/bin/env bash
# gate-stage: 搬过的文件与目录，全仓不再指向旧路径（research/prompts/、research/results/、records/ 都算）
#
# 规则见 .claude/rules/path-moves.md：冻结管的是内容，文件与目录的路径不在冻结范围内。每次搬迁在
# research/scripts/path-moves.tsv 登记一行，搬完用 research/scripts/rewrite-moved-paths.py --apply 把全仓指向旧路径的地方一次改完。
# 实测（2026-09-17）：两份字节布局表搬进 .claude/kb/layout/ 时只改了现行文件，research/prompts/ 下 200 多份材料仍指旧路径，
# 而 E142 装置源码里那条字符串已经改了，留存产物与复跑对不上——当时没有任何检查报出来。
# 判别力：fixtures/64-moved-paths.sh/red 放一份带旧路径的文件与登记表，必须判红；green 只有新路径，必须判绿。
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
SCRIPT="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/rewrite-moved-paths.py"
[[ -f "$ROOT/research/scripts/path-moves.tsv" ]] || { echo "  ! 没有 research/scripts/path-moves.tsv，本阶段无对象可判"; exit 77; }
if python3 "$SCRIPT" --check "$ROOT"; then
  exit 0
fi
echo "  ✗ 搬过的文件或目录，仓里还有地方指着旧路径（上面逐处列出）"
echo "     → 怎么办：python3 research/scripts/rewrite-moved-paths.py --apply 按登记表改成新路径；说搬迁这件事本身的那一行（同一行里旧新路径都在、或带「改前」「搬到」）会自动保留旧路径。"
exit 1
