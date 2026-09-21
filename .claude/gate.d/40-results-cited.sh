#!/usr/bin/env bash
# gate-stage: 实验产物有没有写回
#
# 判据：`research/results/` 里的实验产物，必须在 `kb/experiments.md` 里被点名。
# 点不到名 = 跑过但结论没写回，或者写回了却无法复核（读的人拿不到那份原始数据）。
#
# ⚠️ **这条是实测出来的，不是想出来的**：2026-08-29 有一次把 E20 从 2 档扩到 6 档、
# 跑了三轮、原始输出 22 KB 落了盘，而 experiments.md 里那一节还是两点对比，
# 决策侧一次都没引——报告只存在于对话里。
set -uo pipefail
# ⚠️ 2026-08-29 起实验正文拆到 `kb/experiments/` 下，索引只剩导航表。
# 两侧都要扫：产物可能被任一实验正文点名。
EXP_DIR=.claude/kb/experiments
EXP_ALL="$(mktemp)"; trap 'rm -f "$EXP_ALL"' EXIT
cat .claude/kb/experiments.md "$EXP_DIR"/*.md > "$EXP_ALL" 2>/dev/null
EXP="$EXP_ALL"
RES=research/results
[[ -s "$EXP" ]] || { echo "  ! 找不到实验正文，本阶段跳过"; exit 77; }
[[ -d "$RES" ]] || { echo "  ✓ 没有 $RES 目录，无对象可判"; exit 0; }

missing=()
while IFS= read -r f; do
  b="$(basename "$f")"
  # 三类不算产物：本地腿的问答、逐轮复跑的中间件、确认目录
  case "$b" in *local*|*.r[0-9].out|*.round*|confirm*) continue;; esac
  grep -qF "$b" "$EXP" || missing+=("$b")
done < <(find "$RES" -maxdepth 1 -name '*.out' -type f | sort)

if ((${#missing[@]})); then
  echo "  ✗ 有实验产物没被 experiments.md 点名——跑过但没写回，或写回了却无法复核："
  for m in "${missing[@]}"; do echo "     $RES/$m"; done
  echo "     → 怎么办：把结论写进 experiments.md 对应实验的正文，"
  echo "               并在口径段点名这份原始输出；确实是废弃产物就删掉它。"
  exit 1
fi
echo "  ✓ $RES 下的实验产物全部被 experiments.md 点名（$(find "$RES" -maxdepth 1 -name '*.out'|wc -l) 个文件）"

# ── 反方向：已跑的实验必须要么点名产物，要么显式说明产物没留 ──
# 只查一个方向会漏掉「实验写了结论、但产物从没存在过」——那种情况下
# 读的人既翻不到档案也不知道翻不到，比明说「没留」更糟。
awk '
  /^## E[0-9]+ /{
    if (cur != "" && done && !cited && !excused) print cur
    cur=$2; done=($0 ~ /已跑|已测/); cited=0; excused=0; next
  }
  /research\/results\//{ cited=1 }
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
       for (n in named) print "NAMED" "\t" n "\t" named[n] > "/dev/stderr" }
' "$EXP" > /tmp/.gate-uncited-$$ 2>/tmp/.gate-named-$$
if [[ -s /tmp/.gate-uncited-$$ ]]; then
  echo "  ✗ 这些实验标着已跑，却既没点名原始输出、也没说明产物为什么没留："
  sed 's/^/     /' /tmp/.gate-uncited-$$
  echo "     → 怎么办：存一份产物并在口径段点名；确实留不下（要虚机/真设备）"
  echo "               就写明「原始输出未留存」以及为什么，别让读的人以为能翻到。"
  rm -f /tmp/.gate-uncited-$$
  exit 1
fi
rm -f /tmp/.gate-uncited-$$

# ── 第三道：点名的产物，树里没有就必须在版本库历史里找得到 ──
# 射程与门禁 88 号同一条（用户 2026-09-21 定）：只判**这次改动新增或改写的**点名行。
# 早先写下的点名是历史，仅作参考；而别的会话正在跑、产物还没提交的实验，它的页也不该由这一次提交来判。
# 拿不到 diff 基准就判全部（保守）。
# 少了这一道，上一道就退化成「正文里写个像文件名的串」：归档之后引用只剩文件名，
# 而一个从没存在过的文件名与一份归档进历史的产物，在正文里长得一模一样。
BASE="${GATE_BASE:-}"
if [[ -z "$BASE" ]]; then
  BASE="$(git rev-parse --verify --quiet refs/sop/gate-ok || git rev-parse --verify --quiet refs/singlefs/gate-ok || git rev-parse --verify --quiet "@{upstream}" || true)"
fi
touched=""
if [[ -n "$BASE" ]]; then
  touched="$(git diff --unified=0 "$BASE" -- .claude/kb/experiments.md "$EXP_DIR" 2>/dev/null \
             | grep "^+" | grep -v "^+++" | grep -oE "e[0-9]+[a-zA-Z0-9._-]*\.out" | sort -u)"
fi
missing=0; checked=0; skipped=0
while read -r _ name owner; do   # 字段是 NAMED、文件名、实验号，都不含空格，默认分隔够用
  [[ -n "$name" ]] || continue
  # 不用管道接 grep -q：pipefail 下管道的退出码会被后一段盖掉（command-safety.md）。
  # 这里拿 case 在 shell 里逐字比：$touched 是换行分隔的一串名字，两端各补一个换行再比整段。
  if [[ -n "$BASE" ]]; then
    case "$(printf "\n%s\n" "$touched")" in
      *"$(printf "\n%s\n" "$name")"*) : ;;
      *) skipped=$((skipped+1)); continue ;;
    esac
  fi
  checked=$((checked+1))
  [[ -f "$RES/$name" ]] && continue
  if [[ -n "$(git log --all --diff-filter=D --format=%h --name-only -- "*$name" 2>/dev/null | head -1)" ]]; then
    continue
  fi
  echo "     $owner 点名 $name —— 树里没有，git 历史里也没有"   # gate-lint:detail
  missing=$((missing+1))
done < <(sort -u /tmp/.gate-named-$$ 2>/dev/null)
rm -f /tmp/.gate-named-$$
if [[ $missing -gt 0 ]]; then
  echo "  ✗ $missing 份被点名的产物既不在 $RES 下、也不在版本库历史里"   # gate-lint:summary
  echo "     → 怎么办：产物归档了就该能从历史取回（git log --all --diff-filter=D --name-only 找删它的提交，"
  echo "               再 git show <提交>^:<路径> 读回来）；取不回来说明它从没存在过，"
  echo "               把那一节改成「原始输出未留存」并写明为什么，别让读的人以为翻得到。"
  exit 1
fi
echo "  ✓ 已跑的实验都点了名或写明了产物未留存；这次改动点名的 $checked 份产物在树里或版本库历史里都找得到"
echo "     没判的 $skipped 份：早先就写在正文里的点名（历史参考），以及别的会话正在跑、产物还没提交的实验"
