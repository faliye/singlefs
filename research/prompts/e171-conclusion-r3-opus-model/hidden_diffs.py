"""e171-conclusion-r3 攻方：返回值探针调试输出（OPUS_DRIVE_DEBUG_DIR，每次调用交回值的 {:#?}）里，变异行与基线不同的那几次调用，
逐个给出改了几行、第一处改动的上下文字段名与两边的值（各取前 3 行，每行截到 160 字）。
用法：python3 hidden_diffs.py <调试输出目录> <行号> [<行号> …]
"""
import difflib
import os
import sys

directory = sys.argv[1]
for row in sys.argv[2:]:
    prefix = f"row{row}-"
    for name in sorted(os.listdir(directory)):
        if not name.startswith(prefix):
            continue
        base_path = os.path.join(directory, "base-" + name[len(prefix):])
        mutant = open(os.path.join(directory, name), encoding="utf-8").read().splitlines()
        base = open(base_path, encoding="utf-8").read().splitlines()
        if mutant == base:
            continue
        removed, added = [], []
        for line in difflib.unified_diff(base, mutant, n=0, lineterm=""):
            if line.startswith("-") and not line.startswith("---"):
                removed.append(line[1:].strip()[:160])
            elif line.startswith("+") and not line.startswith("+++"):
                added.append(line[1:].strip()[:160])
        print(f"name=hidden_diff row={row} item={name[len(prefix):-4]} lines_removed={len(removed)} lines_added={len(added)} "
              f"base={' | '.join(removed[:3]) or '-'} mutant={' | '.join(added[:3]) or '-'}")
