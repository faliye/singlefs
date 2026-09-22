#!/usr/bin/env python3
"""把第二轮攻方提的「复核改判」表落成一道会失败的检查，只做它自己点名的那四条。

逐字出处 research/prompts/agentdef-r2-opus-output.md:288：
「要求同步记录里另有一张「复核改判」表（载体、原判定、改判后的判定、抓到它靠的是哪一件东西），
 门禁 68 号能机械核（载体格式、两个判定都是五种之一、两者不相等、末格非空），
 而且它在 0 处判错的那一轮可以是 0 行表」

这不是门禁，是第三轮攻方为了量它的判别力做的原型；放在 research/prompts/agentdef-r3-opus-model/ 下，
不进 .claude/gate.d/。载体存在性一条照抄 .claude/gate.d/68-knowledge-sync.sh 判据④里那一句
（路径从仓库根起，在仓里存在或在改动范围里）。

  check-recheck-table.py 记录.md [--root 仓库根]
退出码：0 通过（含 0 行表）；1 判红；2 参数或文件出错。
"""
import os
import re
import sys

FIVE = ("要改", "要补", "事件句不改", "不相干", "要人看")
HEADER = ["载体", "原判定", "改判后的判定", "抓到它靠的是哪一件东西"]
SECTION = "## 复核改判"
carrier_form = re.compile(r"^(?P<path>\S(?:.*\S)?):(?P<line>[1-9][0-9]*)$")
separator_cell = re.compile(r"^:?-{3,}:?$")
fence_open = re.compile(r"^ {0,3}(`{3,}|~{3,})")
heading = re.compile(r"^#{1,2}\s")


def section_body(text, want):
    body, current, open_fence = None, None, None
    for number, line in enumerate(text.split("\n"), 1):
        if open_fence is None:
            match = fence_open.match(line)
            if match:
                open_fence = match.group(1)
                if current is not None:
                    current.append((number, line, True))
                continue
            if heading.match(line):
                title = line.rstrip()
                current = [] if (title == want and body is None) else None
                if current is not None:
                    body = current
                continue
            if current is not None:
                current.append((number, line, False))
        else:
            if re.match(r"^ {0,3}" + re.escape(open_fence[0]) + "{" + str(len(open_fence)) + r",}\s*$", line):
                open_fence = None
            if current is not None:
                current.append((number, line, True))
    return body


def cells_of(row):
    inner = row.strip()
    if inner.startswith("|"):
        inner = inner[1:]
    if inner.endswith("|") and not inner.endswith("\\|"):
        inner = inner[:-1]
    return [cell.strip() for cell in re.split(r"(?<!\\)\|", inner)]


def main():
    args = [a for a in sys.argv[1:]]
    root = "."
    if "--root" in args:
        index = args.index("--root")
        root = args[index + 1]
        del args[index:index + 2]
    if len(args) != 1:
        print(__doc__)
        return 2
    path = args[0]
    with open(path, encoding="utf-8", errors="replace") as handle:
        text = handle.read()
    body = section_body(text, SECTION)
    if body is None:
        print(f"  ✗ {path}：没有「{SECTION}」小节")
        return 1
    runs, previous = [], False
    for number, line, fenced in body:
        is_row = (not fenced) and line.lstrip().startswith("|")
        if is_row and not previous:
            runs.append([])
        if is_row:
            runs[-1].append((number, line))
        previous = is_row
    if len(runs) != 1:
        print(f"  ✗ {path}：「{SECTION}」下有 {len(runs)} 张表，只许一张")
        return 1
    rows = runs[0]
    if cells_of(rows[0][1]) != HEADER:
        print(f"  ✗ {path}:{rows[0][0]}：表头不是 | " + " | ".join(HEADER) + " |")
        return 1
    if len(rows) < 2 or len(cells_of(rows[1][1])) != 4 or not all(separator_cell.match(c) for c in cells_of(rows[1][1])):
        print(f"  ✗ {path}:{rows[0][0] + 1}：表头下面一行不是四格分隔行")
        return 1
    problems = []
    for number, line in rows[2:]:
        cells = cells_of(line)
        where = f"{path}:{number}"
        if len(cells) != 4:
            problems.append(f"{where}：这一行拆出 {len(cells)} 格，要四格")
            continue
        carrier, before, after, how = cells
        if len(carrier) >= 2 and carrier.startswith("`") and carrier.endswith("`"):
            carrier = carrier[1:-1].strip()
        match = carrier_form.match(carrier)
        if not match:
            problems.append(f"{where}：载体格不是「路径:行号」：{carrier}")
        elif not os.path.exists(os.path.join(root, match.group("path"))):
            problems.append(f"{where}：载体路径 {match.group('path')} 在仓里不存在")
        if not before.startswith(FIVE):
            problems.append(f"{where}：原判定不是五种之一：{before or '（空）'}")
        if not after.startswith(FIVE):
            problems.append(f"{where}：改判后的判定不是五种之一：{after or '（空）'}")
        if before == after:
            problems.append(f"{where}：两个判定相等：{before}")
        if not how:
            problems.append(f"{where}：末格「抓到它靠的是哪一件东西」是空的")
    if problems:
        print(f"  ✗ {len(problems)} 处不合格：")
        for entry in problems:
            print(f"      {entry}")
        return 1
    print(f"  ✓ 「复核改判」表合格：{len(rows) - 2} 行改判（0 行也算合格）")
    return 0


if __name__ == "__main__":
    sys.exit(main())
