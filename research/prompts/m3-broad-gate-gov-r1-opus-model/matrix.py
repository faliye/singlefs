#!/usr/bin/env python3
"""攻方腿模型：在已备好的样本根（<run>/<样本名>/）上，照 cells.py 起每份样本点名的那一格，先不变异、再逐个变异，列出哪份样本的退出码变了。
用法：matrix.py <tree> <run 目录> <fixtures 目录> [弄坏项]"""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import cells
tree, run_directory, fixtures = sys.argv[1:4]
broken = sys.argv[4] if len(sys.argv) > 4 else ""
samples = sorted(os.listdir(fixtures))
def expected(name):
    return next(line[5:] for line in open(os.path.join(fixtures, name, "expect"), encoding="utf-8").read().splitlines() if line.startswith("exit="))
def cell_of(name):
    return open(os.path.join(run_directory, name, ".gate-cells"), encoding="utf-8").read().split()[0]
baseline = {name: cells.run(tree, cell_of(name), os.path.join(run_directory, name), broken)[0] for name in samples}
wrong = [f"{name}(期望{expected(name)}→实测{baseline[name]})" for name in samples if str(baseline[name]) != expected(name)]
print(f"BASELINE 判对 {len(samples) - len(wrong)} 判错 {len(wrong)}：{' '.join(wrong) or '无'}")
for mutant, (cell, _old, _new, what) in cells.MUTANTS.items():
    flips = []
    for name in samples:
        if cell_of(name) != cell:
            continue
        code = cells.run(tree, cell, os.path.join(run_directory, name), broken, mutant)[0]
        if code != baseline[name]:
            flips.append(f"{name}({baseline[name]}→{code})")
    print(f"MUTANT {mutant}\t{what}\t翻面的样本 {len(flips)} 份：{' '.join(flips) or '无（没有样本盯着这一支）'}")
