#!/usr/bin/env python3
"""读 harness 探针二（m33_state_probe）的 PROBE2 行，按新流的版本表与续做步数的期望逐状态判：
  1. oracle：落到的根、读回结果、盘上最新根槽，交给 oracle_model.new_oracle（新判法的 Python 抄本），不许有违例；
  2. 可写挂载：续做步数 = 落到的版本还差几步（意图 3、批一 2、批二 1、最后一批 0，别的 0），走到空，挂完的池版本：
     第 6–9 代 WithTreesAndNoFile、第 3–5 代 WithFile、更早 WithoutFile。
用法：python3 judge_states.py <探针二输出文件>   （只读文件里 PROBE2 行；输出行以 JUDGE 开头）"""
import re, sys
sys.path.insert(0, __file__.rsplit("/", 1)[0] if "/" in __file__ else ".")
from oracle_model import new_oracle

WITH_FILE = {(1, 3): "A", (1, 4): "B", (1, 5): "C"}
WITHOUT_FILE = {(1, 6): None}
STEPS = {6: 3, 7: 2, 8: 1, 9: 0}

def main(path):
    judged = bad = 0
    landed_counts = {}
    for line in open(path, encoding="utf-8"):
        m = re.match(r"PROBE2 state=(\S+) newest_persisted_root\(txg,instance\)=(None|Some\(\(\d+, \d+\)\)) effective_root\(instance,txg\)=Some\(\((\d+), (\d+)\)\) outcome=(\S+) (.*)", line)
        if not m:
            continue
        state, newest, inst, txg, outcome, rest = m.groups()
        inst, txg = int(inst), int(txg)
        judged += 1
        problems = []
        newest_pair = re.match(r"Some\(\((\d+), (\d+)\)\)", newest)
        if newest_pair and (txg, inst) < (int(newest_pair.group(1)), int(newest_pair.group(2))):
            problems.append("RecoveredToAnOlderRoot")
        oc = ("NoFile",) if outcome == "NoFile" else ("FileRead", outcome) if outcome in ("A", "B", "C") else ("Failed",)
        verdict = new_oracle(oc, (inst, txg), WITH_FILE, WITHOUT_FILE)
        if verdict:
            problems.append(verdict)
        mm = re.match(r"mount=ok steps=(\d+) walked_to_empty=(\w+) current=(\w+)", rest)
        if not mm:
            problems.append("mount_failed:" + rest)
        else:
            steps, walked, current = int(mm.group(1)), mm.group(2), mm.group(3)
            if walked != "true": problems.append("not_walked_to_empty")
            if steps != STEPS.get(txg, 0): problems.append(f"steps={steps}_expected={STEPS.get(txg, 0)}")
            expected_member = "WithTreesAndNoFile" if txg >= 6 else "WithFile" if txg >= 3 else "WithoutFile"
            if current != expected_member: problems.append(f"current={current}_expected={expected_member}")
        landed_counts[(inst, txg)] = landed_counts.get((inst, txg), 0) + 1
        if problems:
            bad += 1
            print(f"JUDGE BAD state={state} root=({inst},{txg}) outcome={outcome} problems={problems}")
    print(f"JUDGE states={judged} bad={bad}")
    print("JUDGE landed_roots(instance,txg)->states " + " ".join(f"{k}:{v}" for k, v in sorted(landed_counts.items(), key=lambda kv: (kv[0][1], kv[0][0]))))

main(sys.argv[1])
