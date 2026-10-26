"""mk2to0.py <pre 根目录> <命令清单输出.json> : 从 runfams.sh 在 pre 根目录留下的各家族输出里，挑第一轮判决之前的树判拒、修补后的树判放（2->0）的那几条，
写成 judge.py 能吃的命令清单。重型闸主 agent 与检出器 clerk 起 detached-run 的交真 bash 裁判；检出器 clerk 的写范围那几条真 bash 不起重活，标 noexec、裁判固定 0（由报告里手工核：cp 目标不是目录，什么都没写）。"""
import json, glob, os, sys
root, out = sys.argv[1:3]
rows = []
for orig in sorted(glob.glob(os.path.join(root, "out", "*-orig.json"))):
    new = orig.replace("-orig.json", "-new.json")
    for a, b in zip(json.load(open(orig)), json.load(open(new))):
        if a["rc"] == 2 and b["rc"] == 0:
            row = dict(tag=a["tag"], cmd=a["cmd"], hook=a["hook"], agent=a["agent"])
            if a.get("cwd"): row["cwd"] = a["cwd"]
            if a["hook"] == "det" and "detached-run" not in a["cmd"]:
                row.update(noexec=True, expect=0)
            rows.append(row)
json.dump(rows, open(out, "w"), ensure_ascii=False)
print(len(rows))
