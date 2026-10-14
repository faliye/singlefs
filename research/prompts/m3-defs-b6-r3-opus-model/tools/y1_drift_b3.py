#!/usr/bin/env python3
"""Y1-丙 在零轮之前那一版上复核：那一版没有 lib_abandoned_rounds.py，⑥ 的判法在 runner-dispatch-guard.sh 里（abandoned_registrations）。
把钩子脚本里的 python 段抽出来当模块载入（不跑入口），对 y1_drift.sh 造的同一棵树前后各判一次。
用法：y1_drift_b3.py <零轮之前副本> <树>（树先由 y1_drift.sh 的同一步骤造好；这里自己再插一次行）"""
import os, re, subprocess, sys, types
judge, tree = map(os.path.abspath, sys.argv[1:3])
source = open(os.path.join(judge, ".claude/hooks/runner-dispatch-guard.sh"), encoding="utf-8").read()
python = re.search(r"<<'PY'\n(.*?)\nPY\n", source, re.S).group(1)
python = python.split("\nsys.exit(main())")[0]
module = types.ModuleType("guard_b3"); module.__dict__["__file__"] = os.path.join(judge, ".claude/hooks/runner-dispatch-guard.sh")
sys.argv = [module.__file__, os.path.join(judge, ".claude/hooks")]
exec(compile(python, "guard_b3", "exec"), module.__dict__)
def show(label):
    rows = module.abandoned_registrations(tree, []).get("dr-r1", [])
    problem = module.abandoned_round_problem(tree, "dr-r1", rows)
    print(f"--- {label}：dr-r1 " + ("收" if problem is None else "不收：" + problem[:120]))
show("插行前")
lines = open(os.path.join(tree, "records/log.md"), encoding="utf-8").read().split("\n")
open(os.path.join(tree, "records/log.md"), "w", encoding="utf-8").write("\n".join(lines[:2] + ["## 批 9（2026-10-14）", ""] + lines[2:]))
show("插行后")
