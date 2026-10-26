"""汇总探针的 SCAN / SCANCAND 行（只读）：每次扫描里各候选对「判官说错」的 (t', F', p') 组合收了几个、比今天多收了哪些、多收的是不是都比真 t 低。
判官 = N 做完的健康镜像上今天的严格求等。
"""
import re, sys, ast
head = None
rows = {}
order = []
for path in sys.argv[1:]:
    for line in open(path, encoding="utf-8", errors="replace"):
        line = line.rstrip("\n")
        if line.startswith("SCAN "):
            f = dict(re.findall(r"(\w+)=(\S+)", line))
            head = (f["cell"], f["S"], f["N"], f["label"], int(f["true_t"]), f["combos"])
            order.append(head)
            rows[head] = {}
        elif "SCANCAND" in line and head:
            f = dict(re.findall(r"(\w+)=(\S+)", line))
            sample = re.search(r"wrong_accepted_sample=(\[.*\])", line).group(1)
            rows[head][f["cand"]] = (int(f["legit_accepted"]), int(f["legit_red"]), int(f["legit_na"]), int(f["wrong_accepted"]), int(f["wrong_red"]), int(f["wrong_na"]), ast.literal_eval(sample.replace("(", "(").replace(")", ")")))
for head in order:
    cell, S, N, label, true_t, combos = head
    print(f"== {label} cell={cell} S={S} N={N} true_t={true_t} 组合数={combos}")
    base = rows[head].get("today")
    for cand in ["today", "jia_seed", "any_lap", "ding_jia", "ding_jia_single", "ding_yi", "yi_gate"]:
        r = rows[head].get(cand)
        if not r:
            continue
        la, lr, ln, wa, wr, wn, sample = r
        extra = [x for x in sample if base and x not in base[6]]
        print(f"   {cand:<16} 诚实组合：成立{la} 判红{lr} 不适用{ln}；错组合：收{wa} 判红{wr} 不适用{wn}；比今天多收的（取样）{extra}")
