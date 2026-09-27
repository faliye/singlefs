#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: 实验产物有没有写回
#
# 判据：`research/results/` 里的实验产物，必须在 `kb/experiments.md` 里被点名。
# 点不到名 = 跑过但结论没写回，或者写回了却无法复核（读的人拿不到那份原始数据）。
# 产物 = `research/results/` 下（含子目录）的每一个普通文件，不只 `*.out`（`.log`、`.txt` 一样是原始输出）；
# 顶层的按文件名、子目录里的按相对 `research/results/` 的路径点名。不算产物的几类照名字跳过，成功行逐个列名。
# 三道各报各的对象数；三道都一个对象都没有时退 77（不记通过）。
#
# ⚠️ **这条是实测出来的，不是想出来的**：2026-08-29 有一次把 E20 从 2 档扩到 6 档、
# 跑了三轮、原始输出 22 KB 落了盘，而 experiments.md 里那一节还是两点对比，
# 决策侧一次都没引——报告只存在于对话里。
#
#   bash .claude/gate.d/40-results-cited.sh [项目根]
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" || exit 2
# ⚠️ 2026-08-29 起实验正文拆到 `kb/experiments/` 下，索引只剩导航表。
# 两侧都要扫：产物可能被任一实验正文点名。
EXP_DIR=.claude/kb/experiments
EXP_ALL="$(mktemp)"; UNCITED="$(mktemp)"; NAMED="$(mktemp)"; added_lines="$(mktemp)"
# 中间文件全用 mktemp 并交给 trap：固定路径加 $$ 不会撞，但判红那一支 exit 之前只删了一份，另一份每红一次就在 /tmp 留一份
trap 'rm -f "$EXP_ALL" "$UNCITED" "$NAMED" "$added_lines"' EXIT
cat .claude/kb/experiments.md "$EXP_DIR"/*.md > "$EXP_ALL" 2>/dev/null
EXP="$EXP_ALL"
RES=research/results
[[ -s "$EXP" ]] || { echo "  ! 找不到实验正文，本阶段跳过"; exit 77; }
[[ -d "$RES" ]] || { echo "  ! 没有 $RES 目录，本阶段无对象可判"; exit 77; }

# 三道都判完再退出，一次把问题说全；判的工具自己没跑成（awk 出错）才当场退出。
failed=0
missing=()
# research/scripts/replay.sh 登记的入库产物（取行的形状与门禁 84、87 号同一份：编号|二进制|参数|产物|exact 或 timing，产物取文件名）：
# 名字像逐轮中间件（*.r<一位数>.out）的，登记了照样要被点名。
# replay.sh 不在或读不了：登记的逐轮产物认不出来，全被当成中间件跳过——有这种名字的产物时判红，不许静默放过。
REPLAY_TABLE=research/scripts/replay.sh
replay_table_readable=1
REGISTERED=""
if [[ -r "$REPLAY_TABLE" && -f "$REPLAY_TABLE" ]]; then
  REGISTERED="$(sed -nE 's/^E[0-9]+\|[^|]*\|[^|]*\|([^|]*\/)?([^|/]*)\|(exact|timing)$/\2/p' "$REPLAY_TABLE")" || replay_table_readable=0
else
  replay_table_readable=0
fi
judged_products=0
not_products=()
round_named_skipped=()
while IFS= read -r f; do
  relative="${f#"$RES"/}"
  b="$(basename "$f")"
  # 三类不算产物：本地腿的问答、逐轮复跑的中间件（replay.sh 登记了的除外）、确认目录
  case "$relative" in
    *local*|*.round*|confirm*) not_products+=("$relative"); continue ;;
    *.r[0-9].out)
      case "$(printf "\n%s\n" "$REGISTERED")" in
        *"$(printf "\n%s\n" "$b")"*) : ;;
        *) not_products+=("$relative"); round_named_skipped+=("$relative"); continue ;;
      esac ;;
  esac
  judged_products=$((judged_products + 1))
  grep -qF "$relative" "$EXP" || missing+=("$relative")
done < <(find "$RES" -type f | sort)

if ((${#missing[@]})); then
  echo "  ✗ 有实验产物没被 experiments.md 点名——跑过但没写回，或写回了却无法复核："
  for m in "${missing[@]}"; do echo "     $RES/$m"; done
  echo "     → 怎么办：把结论写进 experiments.md 对应实验的正文，"
  echo "               并在口径段点名这份原始输出；确实是废弃产物就删掉它。"
  failed=1
elif ((judged_products == 0)); then
  echo "  ! 第一道本次无对象可判：$RES 下没有要点名的产物（不算产物的 ${#not_products[@]} 份除外）"
else
  echo "  ✓ $RES 下的实验产物全部被 experiments.md 点名（查了 ${judged_products} 份）"
fi
if ((replay_table_readable == 0 && ${#round_named_skipped[@]})); then
  echo "  ✗ 读不到 $REPLAY_TABLE：它登记的入库产物认不出来，${#round_named_skipped[@]} 份名字像逐轮中间件的产物全被当成中间件跳过了"
  echo "     → 怎么办：replay.sh 搬了家或改了名，就把这里的 REPLAY_TABLE 一起改（.claude/rules/path-moves.md）；读权限丢了就补上。"
  failed=1
fi
if ((${#not_products[@]})); then
  echo "     第一道没判的 ${#not_products[@]} 份（本地腿的问答、没登记的逐轮中间件、确认目录）："
  printf '       %s\n' "${not_products[@]}"
fi

# ── 反方向：已跑的实验必须要么点名产物，要么显式说明产物没留 ──
# 只查一个方向会漏掉「实验写了结论、但产物从没存在过」——那种情况下
# 读的人既翻不到档案也不知道翻不到，比明说「没留」更糟。
awk '
  /^## E[0-9]+ /{
    if (cur != "" && done && !cited && !excused) print cur
    cur=$2; done=($0 ~ /已跑|已测/); cited=0; excused=0
    if (done) ran_count++
    next
  }
  # 目录名后面要跟着一个文件名才算点名：只写「产物放在 research/results/ 下」的那一节什么都没点
  /research\/results\/[A-Za-z0-9_.-]/{ cited=1 }
  # 归档之后引用只剩文件名（「每一次提交删上一次的实验记录」，产物去版本库历史里查），
  # 所以裸文件名也算点了名——但它点的那份下面还要逐个去树里与 git 历史里找得到，
  # 不然「点名」就退化成「写个像文件名的串」。
  # 前面要有词边界：少了它，`-stage1.out` 里的 "stag|e1.out" 会被咬成一个产物名
  match($0, /(^|[^a-zA-Z0-9_-])e[0-9]+[a-zA-Z0-9._-]*\.out/) {
    tok=substr($0, RSTART, RLENGTH); sub(/^[^a-zA-Z0-9_-]/, "", tok)
    named[tok]=cur; cited=1
  }
  /原始输出未留存|输出未留存/{ excused=1 }
  END{ if (cur != "" && done && !cited && !excused) print cur
       for (n in named) print "NAMED" "\t" n "\t" named[n] > "/dev/stderr"
       print "RAN_COUNT" "\t" ran_count + 0 > "/dev/stderr" }
' "$EXP" > "$UNCITED" 2>"$NAMED" || {
  echo "  ✗ 判「已跑实验有没有点名产物」的 awk 没跑成（退出码 $?），这一格没判："
  sed 's/^/     /' "$NAMED"   # gate-lint:detail
  echo "     → 怎么办：看上面 awk 的报错修这一段；它没跑完时未点名的实验一个都不会被列出来，不许当成通过。"
  exit 1
}
if [[ -s "$UNCITED" ]]; then
  echo "  ✗ 这些实验标着已跑，却既没点名原始输出、也没说明产物为什么没留："
  sed 's/^/     /' "$UNCITED"
  echo "     → 怎么办：存一份产物并在口径段点名；确实留不下（要虚机/真设备）"
  echo "               就写明「原始输出未留存」以及为什么，别让读的人以为能翻到。"
  failed=1
fi
rm -f "$UNCITED"
ran_count="$(sed -n 's/^RAN_COUNT\t//p' "$NAMED")"

# ── 第三道：点名的产物，树里没有就必须在版本库历史里找得到 ──
# 射程与门禁 88 号同一条（用户 2026-09-21 定）：只判**这次改动新增或改写的**点名行。
# 早先写下的点名是历史，仅作参考；而别的会话正在跑、产物还没提交的实验，它的页也不该由这一次提交来判。
# 改动范围取共用脚本 research/scripts/changed-paths.sh（基准 gate_diff_base gate，新增行 gate_added_lines：
# 未跟踪的实验页整份算新增——新写的页在 git add 之前，里面点名的产物也要判）。不在 git 仓里、或取不到改动范围，就判全部（保守）。
# ⚠️ 没有上游、也没设 GATE_BASE 时，共用脚本的基准是 HEAD，窗口只含工作区与暂存区（与 88 号同一格，交用户定：gate-fix-forks-r1-forks.md 的 T11）。
# 少了这一道，上一道就退化成「正文里写个像文件名的串」：归档之后引用只剩文件名，
# 而一个从没存在过的文件名与一份归档进历史的产物，在正文里长得一模一样。
LIB_CHANGED_PATHS="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/changed-paths.sh"
# shellcheck source=../../research/scripts/changed-paths.sh
source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"; echo "     → 怎么办：它随仓走（research/scripts/changed-paths.sh），被删了就从 git 找回来。"; exit 1; }
BASE="" touched="" scope_note="全部的行（不在 git 仓里，判全部）"
if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  BASE="$(gate_diff_base gate)"
  if gate_added_lines "$BASE" .claude/kb/experiments.md "$EXP_DIR" > "$added_lines"; then
    touched="$(cut -f2- "$added_lines" | grep -oE "e[0-9]+[a-zA-Z0-9._-]*\.out" | sort -u)"
    scope_note="这次改动（基准 $BASE）新增或改写的行"
  else
    BASE="" scope_note="全部的行（取不到基准 $(gate_diff_base gate) 起的新增行，判全部）"
  fi
  rm -f "$added_lines"
fi
missing=0; checked=0; skipped=0
declare -A skipped_names_by_owner=()
while read -r _ name owner; do   # 字段是 NAMED、文件名、实验号，都不含空格，默认分隔够用
  [[ -n "$name" ]] || continue
  # 不用管道接 grep -q：pipefail 下管道的退出码会被后一段盖掉（command-safety.md）。
  # 这里拿 case 在 shell 里逐字比：$touched 是换行分隔的一串名字，两端各补一个换行再比整段。
  if [[ -n "$BASE" ]]; then
    case "$(printf "\n%s\n" "$touched")" in
      *"$(printf "\n%s\n" "$name")"*) : ;;
      *) skipped=$((skipped+1))
         skipped_names_by_owner["${owner:-（不在任何实验节里）}"]+=" $name"
         continue ;;
    esac
  fi
  checked=$((checked+1))
  [[ -f "$RES/$name" ]] && continue
  # 正在归档的：HEAD 里还在、工作区里删了（这一批照归档规则删上一轮的产物，删除还没提交，下一行的 git log 还找不到它）
  git -c core.quotepath=false cat-file -e "HEAD:$RES/$name" 2>/dev/null && continue
  if [[ -n "$(git -c core.quotepath=false log --all --diff-filter=D --format=%h --name-only -- "*$name" 2>/dev/null | head -1)" ]]; then
    continue
  fi
  echo "     $owner 点名 $name —— 树里没有，git 历史里也没有"   # gate-lint:detail
  missing=$((missing+1))
done < <(grep '^NAMED' "$NAMED" | sort -u)
rm -f "$NAMED"
if [[ $missing -gt 0 ]]; then
  echo "  ✗ $missing 份被点名的产物既不在 $RES 下、也不在版本库历史里"   # gate-lint:summary
  echo "     → 怎么办：产物归档了就该能从历史取回（git log --all --diff-filter=D --name-only 找删它的提交，"
  echo "               再 git show <提交>^:<路径> 读回来）；取不回来说明它从没存在过，"
  echo "               把那一节改成「原始输出未留存」并写明为什么，别让读的人以为翻得到。"
  failed=1
elif ((checked == 0)); then
  echo "  ! 第三道本次无对象可判：$scope_note里没有点名产物的"
else
  echo "  ✓ $scope_note点名的 $checked 份产物在树里或版本库历史里都找得到"
fi
if ((skipped)); then
  # 没判的现算：不在这次改动新增或改写的行里的点名（早先写下的，历史参考；射程与 88 号同一条），按实验逐个列
  echo "     第三道没判的 $skipped 份：不在这次改动新增或改写的行里的点名，按实验列："
  while IFS= read -r skipped_owner; do
    echo "       $skipped_owner：${skipped_names_by_owner[$skipped_owner]# }"
  done < <(printf '%s\n' "${!skipped_names_by_owner[@]}" | sort -V)
fi
((failed)) && exit 1
if ((judged_products == 0 && ${ran_count:-0} == 0 && checked == 0)); then
  echo "  ! 本次无对象可判：没有要点名的产物、没有标着已跑的实验、这次改动里也没有点名产物的行"
  exit 77
fi
if ((${ran_count:-0} == 0)); then
  echo "  ! 第二道本次无对象可判：没有标着已跑的实验"
else
  echo "  ✓ 已跑的 $((ran_count)) 个实验都点了名或写明了产物未留存"
fi
