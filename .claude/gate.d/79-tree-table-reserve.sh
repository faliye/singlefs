#!/usr/bin/env bash
# gate-stage: 树表条目预留字节的认购合计（认购不许超过预留，每条要指得到一个编号）
#
# C338（树表条目预留没有余量）欠的那一半：认购表立成正文之后，要有一道算和的检查。
# 2026-09-22 立表时当场抓到一处：正文列的是 8 + 8 + 8 + 16 + 8 = 48，而它自己写「共 56」——
# 那个 8 是 C143（inode 号水位在回退后会退回去重发）2026-09-16 定案「水位照旧住记账」之前的旧账，
# 定案之后没人回来改合计。靠人算迟早再漂，所以这个数由这道检查来算。
#
# 表的形态（.claude/kb/decisions/08-核心索引结构.md 已定项 11）：
#   <!-- gate:tree-table-reserve total=76 -->
#   | 认购者 | 住哪条条目 | 宽度 | 出处 |
#   |---|---|---|---|
#   | … | 头条目 | 8 | C271（…） |
# 判据三条：① 宽度列全是正整数；② 合计不超过标记里的 total；③ 每行「出处」要带一个编号
#（C<n> 在 .claude/kb/checks-owed.md 有登记行，D<n> 在 .claude/kb/decisions/<n>-*.md 存在）。
# 成功那句报出检查了几条认购、合计多少、余多少（rules/show-me-test.md：扫到 0 项也不是通过）。
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2

report="$(python3 - <<'PY'
import glob, re, sys

marker = re.compile(r"<!--\s*gate:tree-table-reserve\s+total=(\d+)\s*-->")
files = sorted(glob.glob(".claude/kb/decisions/*.md"))
found = []
for path in files:
    lines = open(path, encoding="utf-8").read().split("\n")
    for index, line in enumerate(lines):
        matched = marker.search(line)
        if matched:
            found.append((path, index, int(matched.group(1)), lines))

if len(found) != 1:
    print("BAD", f"带 gate:tree-table-reserve 标记的表应当恰好有一张，实际 {len(found)} 张", sep="\t")
    print("CHECKED", 0, 0, 0, sep="\t")
    sys.exit()

path, index, total, lines = found[0]
rows = []
for line in lines[index + 1:]:
    if not line.startswith("|"):
        if rows:
            break
        continue
    cells = [cell.strip() for cell in re.split(r"(?<!\\)\|", line.strip().strip("|"))]
    if len(cells) >= 4 and cells[0] not in ("认购者",) and not cells[0].startswith("---"):
        rows.append(cells)

owed = open(".claude/kb/checks-owed.md", encoding="utf-8").read()
subtotal = 0
for cells in rows:
    name, width_text, source = cells[0], cells[2], cells[3]
    if not re.fullmatch(r"\d+", width_text):
        print("BAD", f"「{name}」的宽度列不是正整数：{width_text}", sep="\t")
        continue
    subtotal += int(width_text)
    numbers = re.findall(r"(C\d{1,3}|D\d{1,2})（", source)
    if not numbers:
        print("BAD", f"「{name}」的出处里没有带简称的编号", sep="\t")
        continue
    for number in numbers:
        if number.startswith("C"):
            if f"| {number} |" not in owed:
                print("BAD", f"「{name}」指的 {number} 在 checks-owed.md 里没有登记行", sep="\t")
        elif not (glob.glob(f".claude/kb/decisions/{int(number[1:])}-*.md")
                  or glob.glob(f".claude/kb/decisions/{int(number[1:]):02d}-*.md")):
            print("BAD", f"「{name}」指的 {number} 在 decisions/ 下没有正文", sep="\t")

if subtotal > total:
    print("BAD", f"认购合计 {subtotal} 超过预留 {total}", sep="\t")
print("CHECKED", len(rows), subtotal, total, sep="\t")
PY
)"

mapfile -t bad < <(grep '^BAD' <<<"$report")
read -r _ rows subtotal total < <(grep '^CHECKED' <<<"$report")

if ((${#bad[@]})); then
  echo "  ✗ 树表条目预留的认购表不合规："
  while IFS=$'\t' read -r _ why; do
    printf '      %s\n' "$why"   # gate-lint:detail
  done < <(printf '%s\n' "${bad[@]}")
  echo "    → 怎么办：认购表在 .claude/kb/decisions/08-核心索引结构.md 已定项 11，标记那一行写着 total=<预留字节数>。"
  echo "      合计超了就先去掉一个认购者（或把那一段的宽度压小），别直接改 total——预留多少由那条已定项定；"
  echo "      出处指不到编号的，补一个还开着的欠账号或一条已定分项，编号要带简称。"
  exit 1
fi
echo "  ✓ 树表条目预留的认购表对得上（${rows} 条认购，合计 ${subtotal} 字节、预留 ${total}、余 $((total - subtotal))；每条的出处都指得到一个编号）"
