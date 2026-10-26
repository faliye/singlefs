"""把探针的逐候选输出压成一张镜像一行：最新根、t、p、各候选 I-7.18 的判定词、今天的判法下判红的全部不变量。"""
import re
import sys

order = []
rows = {}
for line in open(sys.argv[1], encoding="utf-8"):
    header = re.search(r"cell=(\S+) image=(\S+) devices=(\[[^\]]*\]) ring=.*? newest=(\(\d+, \d+\)) t=(\d+) p=(\d+) F=(\d+) next_slot_root=(None|Some\(\(\d+, \d+\)\))", line)
    if header and "candidate=" not in line:
        key = (header.group(1), header.group(2))
        if key not in rows:
            order.append(key)
            rows[key] = {"head": f"devices={header.group(3).replace(chr(32), "")} newest={header.group(4)} t={header.group(5)} p={header.group(6)} F={header.group(7)} next_slot_root={header.group(8)}", "verdicts": [], "red": ""}
        continue
    verdict = re.search(r"cell=(\S+) image=(\S+) candidate=(\S+) I-7.18=([a-z_]+)\S*.* red=(\[.*\])", line)
    if verdict:
        key = (verdict.group(1), verdict.group(2))
        rows[key]["verdicts"].append(f"{verdict.group(3)}={verdict.group(4)}")
        if verdict.group(3) == "today_or_bing":
            rows[key]["red"] = verdict.group(5).replace(" ", "")
for key in order:
    row = rows[key]
    print(f"cell={key[0]} image={key[1]} {row['head']} {' '.join(row['verdicts'])} today_red={row['red']}")
