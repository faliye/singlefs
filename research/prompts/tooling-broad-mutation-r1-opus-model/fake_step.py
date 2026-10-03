#!/usr/bin/env python3
"""攻方腿自己的假一步（调法 <步名> <序号> <树>）：记一笔（序号、步名、树里此刻有哪几条的改动标记、树里多出来的文件数），再照 plan.tsv 办。
plan.tsv 一行：序号（或 *）<TAB> 步名（或 *）<TAB> 动作；后面的行盖前面的。动作：ok、exit:<码>、kill-parent（给驱动发 KILL 后自己退 0）。"""
import os, re, signal, sys
step, sequence, tree = sys.argv[1], sys.argv[2], sys.argv[3]
work = os.environ["PROBE_WORK"]
marks = set()
for root, _dirs, files in os.walk(tree):
    for name in files:
        if name.endswith(".rs"):
            marks |= set(re.findall(r"MUT_(\d+)\(\)", open(os.path.join(root, name)).read()))
with open(os.path.join(work, "calls.log"), "a") as log:
    log.write(f"{sequence}\t{step}\t{','.join(sorted(marks, key=int)) or '-'}\n")
action = "ok"
for line in open(os.path.join(work, "plan.tsv")):
    fields = line.rstrip("\n").split("\t")
    if len(fields) == 3 and fields[0] in (sequence, "*") and fields[1] in (step, "*"):
        if fields[0] == "*" and sequence == "0":
            continue   # 通配不罩基线
        action = fields[2]
if action == "kill-parent":
    os.kill(os.getppid(), signal.SIGKILL)
    sys.exit(0)
if action == "fail-nextest":
    print("        FAIL [   0.010s] probe-package::probe_target some_case")
    sys.exit(100)
if action.startswith("exit:"):
    sys.exit(int(action.split(":")[1]))
sys.exit(0)
