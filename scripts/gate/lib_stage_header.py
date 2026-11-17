#!/usr/bin/env python3
# admission: always 读的是调用方这一次点名的那份阶段文件，不存结论
# run-condition: none 只用 python 标准库读文件
"""门禁阶段文件头的格名表（.claude/rules/verification-reference.md「门禁的结构」）的读法，只有这一份。

文件头注释块（#! 之后、第一行代码之前）里每格一行 `# gate-cell: <格名> <判什么的一句>`。
调它的：scripts/gate/gate-structure-check.py（判结构；code-tooling.sh 的 stage-owners 等格经它导入）、.claude/gate.d/lib-governance-refs.py（判治理文档点名的格在不在）。
单放一份：治理引用那一路只读文件头，不该因为载入这个函数把 gate-structure-check.py 起阶段的路连进自己的判定键。
格名表的判法与它的红绿样本都在 gate-structure-check.py 的自证里（table、list 两样），这一份没有单独的自证。
"""
import re

CELL_LINE = re.compile(r"^#\s*gate-cell:\s*(\S+)\s+(\S.*?)\s*$")


def header_cells(path):
    """文件头注释块里的格名表：[(行号, 格名, 判什么)]。"""
    cells = []
    with open(path, encoding="utf-8", errors="replace") as handle:
        for number, line in enumerate(handle, 1):
            line = line.rstrip("\n")
            if number == 1 and line.startswith("#!"):
                continue
            if line.strip() and not line.lstrip().startswith("#"):
                break
            match = CELL_LINE.match(line.strip())
            if match:
                cells.append((number, match.group(1), match.group(2)))
    return cells
