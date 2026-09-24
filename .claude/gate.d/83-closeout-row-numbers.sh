#!/usr/bin/env bash
# gate-stage: 收口表的行号只许是顺序号，不许带后缀
#
# 2026-09-23 现查：收口表里在 20、23、34 后面加撇号的那几行（今天的第 53–56 行）与它们的基号**毫不相干**
# （`34` 是「收口表漏收还开着的欠账号」，今天的第 56 行（原先是 34 加一撇）是「记账树每次发布只写当前代」）。
# 撇号的实际含义是「这个号被占了，我加一撇」——而读的人会当成从属关系，
# 于是「跟第 34 行一起判」这种话指到一件不相干的事上。字母后缀（`20a`）同病。
#
# ⇒ 用户 2026-09-23 定：行号全改成顺序号，以后禁止后缀。这道阶段判那条禁令。
#
# 判据：带 `<!-- milestone:closeout-table -->` 标记的那张表，每一行首列要么是纯十进制数字、
# 要么是圆圈号（`①`–`⑳`，那是这张表早先给「被打回的项」留的一档，不在这次禁令里）。
# 出现撇号（U+2032、U+2033、ASCII 单引号）或紧跟字母的，判红并指名那一行。
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2

mapfile -t files < <(grep -rl 'milestone:closeout-table' .claude/kb/milestone/ 2>/dev/null | sort)
if ((${#files[@]} == 0)); then
  echo "  ⊘ 本次未跑：.claude/kb/milestone/ 下没有带收口表标记的文件，今天无对象可判"
  exit 77
fi

report="$(FILES="${files[*]}" python3 - <<'PY'
import os, re

circled = "①②③④⑤⑥⑦⑧⑨⑩⑪⑫⑬⑭⑮⑯⑰⑱⑲⑳"
checked = 0
for path in os.environ["FILES"].split():
    lines = open(path, encoding="utf-8").read().split("\n")
    inside = False
    for number, line in enumerate(lines, 1):
        # 整行匹配，不用 in：表里有一行的正文提到这个标记（它讲的是门禁 67 号），
        # 用 in 会把那一行当成标记行跳过，少判一行而汇总照样报绿。
        if line.strip() == "<!-- milestone:closeout-table -->":
            inside = True
            continue
        if inside and line.strip() and not line.startswith("|"):
            inside = False
            continue
        if not inside or not line.startswith("|"):
            continue
        first = line.split("|")[1].strip()
        if not first or first.startswith("-") or first == "#":
            continue
        checked += 1
        if first.isdecimal() or (len(first) == 1 and first in circled):
            continue
        print("BAD", f"{path}:{number}", f"行号写成「{first}」", sep="\t")
print("COUNT", checked, len(os.environ["FILES"].split()), sep="\t")
PY
)"

mapfile -t bad < <(grep '^BAD' <<<"$report")

if ((${#bad[@]})); then
  echo "  ✗ 收口表的行号带了后缀（撇号或字母）："
  while IFS=$'\t' read -r _ where what; do
    printf '      %s：%s\n' "$where" "$what"   # gate-lint:detail
  done < <(printf '%s\n' "${bad[@]}")
  echo "    → 怎么办：改成一个没被占用的顺序号（接着表里最大的那个往下取），不要在旧号上加撇号或字母。"
  echo "      后缀会让读的人以为这一行从属于那个基号，而 2026-09-23 现查的四例里没有一例真的从属。"
  echo "      ⚠️ 改号要连全仓引用一起改（「第 N 行」「跟第 N 行」这类），照 .claude/rules/path-moves.md「怎么做」那六步办。"
  exit 1
fi

if ! grep -q '^COUNT' <<<"$report"; then
  echo "  ✗ 扫描没跑完：内嵌 python 没报出 COUNT 那一行（多半是它自己崩了）"
  echo "    → 怎么办：直接跑 bash .claude/gate.d/83-closeout-row-numbers.sh 看 python 的报错；"
  echo "      ⚠️ 别把这一步当通过——没有 COUNT 就是一行都没判，而汇总行看着与判过了一模一样。"
  exit 1
fi
read -r _ checked file_count < <(grep '^COUNT' <<<"$report")
echo "  ✓ 收口表的行号都是顺序号或圆圈号，没有撇号与字母后缀（判了 ${file_count} 份、${checked} 行）"
exit 0
