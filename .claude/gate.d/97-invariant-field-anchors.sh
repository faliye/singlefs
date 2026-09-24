#!/usr/bin/env bash
# gate-stage: 新写的不变量要点名它判的字段住在哪条已定分项
#
# 判据：这次改动在 `.claude/kb/invariants.md` 里新增或改写的每一行不变量（`| I-x.y | …`），
# 正文里必须至少点名一条 `D<n>（简称） 已定项 k`——那是这条不变量要判的字段的落点。
# 点不出来判红。已停用的行（正文写着「此编号不再使用」）不判。
#
# 为什么：I-1.2（块头写序已发布） 与 I-1.4（块头 fsid 一致） 写下来的时候，它们点名的 generation 与 fsid
# 在当时的字段表里根本不存在——两条已定不变量在第一版直接落空，而没有任何东西说话
# （C69（已定不变量没有字段可判））。一条判不了的不变量比没有这条更糟：checker 会老老实实跑它、
# 老老实实报绿（`show-me-test.md`「不变量本身就是错的，checker 会老老实实检查一条错规矩，而且全绿」）。
#
# ⚠️ 射程只到**这次改动的行**，与门禁 40、88 号同一条：存量里点不出分项的行今天有几十条，
# 一次全判会把每一次提交都挡住，而 `sop-first.md` 要的是抬地板、不是筑墙。存量那笔账仍然欠着
# （C69（已定不变量没有字段可判）），所以成功那句**每轮都把存量的条数现算着报出来**，不让它沉默。
# ⚠️ 判不了的：点名的那条分项里到底有没有这个字段、宽度对不对——那要人看。这一道只判「点没点名」。
#
# 改动范围与 56 号同一条：GATE_BASE 给了就与它比，否则与 @{upstream} 的 merge-base 比，都没有就与 HEAD 比；
# 工作区、暂存区与未跟踪文件都算（基准用共用库 research/scripts/changed-paths.sh 的 gate 取法，与 56、68、69 号同一份代码）。
# 这次改动里 invariants.md 是新增的（相对基准新增、暂存新增或未跟踪），整份的每一行都算新写；
# 否则取「基准到工作区」与「HEAD 到暂存区」两份 diff 的 + 行，同一行两份都有只算一次。
#
# 判别力：fixtures/97-invariant-field-anchors.sh/red 是一个 invariants.md 还未跟踪的小仓，两行里一行没带分项引用，
# 必须判红且只报那一行；green 是已跟踪文件新增一行带分项引用、改完又 git add，必须判绿且报 1 行（不是 2 行）。
#
#   bash .claude/gate.d/97-invariant-field-anchors.sh [项目根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
git rev-parse --is-inside-work-tree >/dev/null 2>&1 || { echo "  ! $ROOT 不是 git 仓，本阶段跳过"; exit 77; }
[[ -f .claude/kb/invariants.md ]] || { echo "  ! 没有 .claude/kb/invariants.md，本阶段无对象可判"; exit 77; }
LIB_CHANGED_PATHS="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/changed-paths.sh"
# shellcheck source=../../research/scripts/changed-paths.sh
source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用库 $LIB_CHANGED_PATHS"; echo "     → 怎么办：改动范围的取法只有那一份，恢复它，别在阶段里再抄一份。"; exit 1; }
base="$(gate_diff_base gate)"
added="$(gate_changed_paths "$base" untracked A)" || {
  echo "  ✗ 取不到这次改动碰了哪些路径（基准 $base）"
  echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到改动范围时这一阶段什么都没比，不是通过。"
  exit 1
}
file_is_new=0
while IFS= read -r added_path; do
  if [[ "$added_path" == .claude/kb/invariants.md ]]; then file_is_new=1; fi
done <<<"$added"
python3 - "$base" "$file_is_new" <<'PY'
import re, subprocess, sys

base = sys.argv[1]
file_is_new = sys.argv[2] == "1"
path = ".claude/kb/invariants.md"
ROW = re.compile(r"^\|\s*(I-[\d.]+)\s*\|")
ITEM = re.compile(r"D\d+（[^）]+）\s*已定项\s*\d+")
RETIRED = "此编号不再使用"

def anchored(line):
    return ITEM.search(line) is not None

# 这次改动新增或改写的行：diff 的 `+` 侧。基准那一版没有这个文件时（新建，含未跟踪），整份都算新写。
# 两份 diff 会重叠（改动已暂存、工作区没再动时两份都有同一行），同一行只算一次，别让报出来的行数翻倍。
touched = []
if file_is_new:
    candidate_lines = open(path, encoding="utf-8").read().split("\n")
else:
    git_diff = ["git", "-c", "core.quotepath=false", "diff", "-U0"]
    diff = subprocess.run([*git_diff, base, "--", path], capture_output=True, text=True).stdout
    staged = subprocess.run([*git_diff, "--cached", "--", path], capture_output=True, text=True).stdout
    candidate_lines = [line[1:] for chunk in (diff, staged) for line in chunk.split("\n")
                       if line.startswith("+") and not line.startswith("+++")]
for body in candidate_lines:
    if ROW.match(body) and RETIRED not in body and body not in touched:
        touched.append(body)

missing = [body for body in touched if not anchored(body)]

# 存量：整份文件里点不出分项的行，现算，每轮报出来
total, unanchored_all = 0, 0
for line in open(path, encoding="utf-8"):
    if ROW.match(line) and RETIRED not in line:
        total += 1
        if not anchored(line):
            unanchored_all += 1

if missing:
    print(f"  ✗ 这次改动新写或改写的 {len(missing)} 行不变量，没有点名它判的字段住在哪条已定分项（基准 {base}）：")  # gate-lint:summary
    for body in missing:
        identifier = ROW.match(body).group(1)
        print(f"      {identifier}：{body.strip()[:120]}")  # gate-lint:detail
    print("     → 怎么办：在那一行正文里写清这条不变量判的字段住哪，形态是「D<编号>（简称） 已定项 k」。")
    print("               写不出来，说明这条不变量今天判不了任何字节——那正是 C69（已定不变量没有字段可判） 记着的那一类：")
    print("               checker 会老老实实跑它、老老实实报绿。先补字段（在那条决策里定出落点与宽度），或者把这条不变量降级。")
    sys.exit(1)

if not touched:
    print(f"  ✓ 这次改动没有新写或改写不变量行（基准 {base}）；存量 {total} 行里 {unanchored_all} 行还点不出已定分项，那笔账记在 C69（已定不变量没有字段可判）")
else:
    print(f"  ✓ 这次改动新写或改写的 {len(touched)} 行不变量都点名了已定分项（基准 {base}）；"
          f"存量 {total} 行里 {unanchored_all} 行还点不出，那笔账记在 C69（已定不变量没有字段可判）")
PY
