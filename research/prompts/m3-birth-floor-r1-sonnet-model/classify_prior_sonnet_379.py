#!/usr/bin/env python3
"""F1：上一轮正推腿模型（m3-inv-b-r1-sonnet-model/b3_bounds_model.py）里的 379 条下界违例历史，逐条归类：
R_first 是不是回退那次发布写出来的根？回退根的引用集是不是整个照抄回退目标的（那个模型里每条根的引用集就是「用户可见树的全部」）？
只读导入那份模型的函数（截到它的「total =」那一行之前），不改它。"""
import os, sys
sys.dont_write_bytecode = True
here = os.path.dirname(os.path.abspath(__file__))
src = open(os.path.join(here, "..", "m3-inv-b-r1-sonnet-model", "b3_bounds_model.py"), encoding="utf-8").read()
ns = {"__name__": "prior"}
exec(compile(src.split("\ntotal = with_rollback_low")[0], "b3_bounds_model.py", "exec"), ns)
roots_txg_op = {}
n_hist = n_red_hist = 0
kinds = {"R_first是回退根且整份照抄回退目标": 0, "R_first是回退根但不是整份照抄": 0, "R_first不是回退根": 0}
with_roll_red = 0
for ops in ns["all_ops"](6):
    res = ns["run"](ops)
    if res is None:
        continue
    n_hist += 1
    roots, floor = res
    upper, lower = ns["violations"](*res)
    if not lower:
        continue
    n_red_hist += 1
    # 每条根是哪个操作写出来的：按操作串里非 F 的操作依次对应 txg 4, 5, ...
    writers = [op for op in ops if op[0] != "F"]
    op_of_txg = {4 + i: w for i, w in enumerate(writers)}
    for unit, first, prev in lower:
        w = op_of_txg.get(first)
        if w is None or w[0] != "R":
            kinds["R_first不是回退根"] += 1
            continue
        cand = [r for r in roots if r[0] >= 0]
        # 回退根的引用集是否整份等于某条更早根的引用集（模型里回退根的引用 = 目标根的引用）
        rr = dict(roots)[first]
        whole = any(rr == r_refs for t, r_refs in roots if t < first)
        kinds["R_first是回退根且整份照抄回退目标" if whole else "R_first是回退根但不是整份照抄"] += 1
print("历史（操作串长 ≤ 6）合法条数", n_hist, "，下界红的历史", n_red_hist)
print("下界违例单元按 R_first 归类:", kinds)
