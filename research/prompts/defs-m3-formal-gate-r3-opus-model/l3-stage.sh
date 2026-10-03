#!/usr/bin/env bash
# L3：用仓副本里的格共用库 lib/stage-cells.sh 搭一道与 checker-tier-formal-verification 同名、同格名、同标题与出路的合成阶段（标题与出路从副本的阶段文件里抠），
# verus-must-red-worlds 那一格的输出由 l3-cell.py 打（三种：all-not-run、mixed、all-red），verus-toolchain-pinned 那一格打一行 ✓ 退 0。不起 Verus、不跑真阶段。
# 用法：bash l3-stage.sh <仓副本根> <种类> <草稿目录>；合成阶段文件放 <草稿目录>/<种类>/.claude/gate.d/ 下（共用库按阶段文件往上两级认仓）
set -uo pipefail
COPY="${1:?仓副本}"; KIND="${2:?种类}"; W="${3:?草稿目录}"; HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
D="$W/$KIND/.claude/gate.d"; mkdir -p "$D/lib"; cp "$COPY/.claude/gate.d/lib/stage-cells.sh" "$D/lib/"
pick() { python3 - "$COPY/.claude/gate.d/checker-tier-formal-verification.sh" "$1" <<'PY'
import shlex, sys
text = open(sys.argv[1], encoding="utf-8").read()
for block in text.split("\nstage_cell ")[1:]:
    words = shlex.split(block.split("\n\n")[0].replace("\\\n", " ").split("\nfi")[0].split("\nstage_cells_parse")[0])
    if words[0] == sys.argv[2]:
        print(shlex.join(words[2:4]))
PY
}
{
  printf '#!/usr/bin/env bash\nset -uo pipefail\nsource "$(dirname "${BASH_SOURCE[0]}")/lib/stage-cells.sh"\n'
  printf 'stage_cell verus-toolchain-pinned cell_toolchain %s\n' "$(pick verus-toolchain-pinned)"
  printf 'stage_cell verus-must-red-worlds cell_red_worlds %s\n' "$(pick verus-must-red-worlds)"
  printf 'cell_toolchain() { echo "  ✓ 合成：版本一致"; }\n'
  printf 'cell_red_worlds() { python3 %q %q %q; }\n' "$HERE/l3-cell.py" "$COPY" "$KIND"
  printf 'stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}\nstage_cells_run "$(dirname "${BASH_SOURCE[0]}")/../.."\n'
} > "$D/checker-tier-formal-verification.sh"
( cd "$W/$KIND" && bash "$D/checker-tier-formal-verification.sh" ); echo "合成阶段退 $?"
