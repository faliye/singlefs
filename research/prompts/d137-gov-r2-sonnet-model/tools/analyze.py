#!/usr/bin/env python3
"""读 probe.py 的输出，列修补前放行、修补后拒的命令（每条命令一行，主 agent 与子 agent 都拒的标 both），以及只多记检出的。
用法：analyze.py 输出文件…"""
import re, sys, collections
rows = collections.OrderedDict()
for path in sys.argv[1:]:
    lines = open(path, encoding="utf-8").read().splitlines()
    for i, l in enumerate(lines):
        m = re.match(r"now:rc=(\d+)/n=(\d+)/([\d.]+)s base:rc=(\d+)/n=(\d+)/([\d.]+)s.*?\| (\w+) (\S+) \| (\S+) \| (.*)$", l)
        if not m: continue
        why = ""
        if i + 1 < len(lines) and lines[i + 1].startswith("    now> "):
            why = lines[i + 1][9:].split("：")[1][:60] if "：" in lines[i + 1] else lines[i + 1][9:70]
        rows[(path, i)] = dict(now=int(m[1]), nnow=int(m[2]), base=int(m[4]), nbase=int(m[5]), hook=m[7], agent=m[8], group=m[9], cmd=m[10], why=why, tnow=float(m[3]), tbase=float(m[6]))
newly = collections.OrderedDict()
for r in rows.values():
    if r["base"] == 0 and r["now"] == 2:
        k = (r["hook"], r["cmd"])
        e = newly.setdefault(k, {"agents": set(), "why": r["why"], "group": r["group"]})
        e["agents"].add(r["agent"])
for (hook, cmd), e in newly.items():
    print(f"{e['group']}\t{hook}\t{'both' if len(e['agents'])==2 else sorted(e['agents'])[0]}\t{e['why']}\t{cmd[:150]!r}")
print(f"# 修补前放行、修补后拒的不同命令数 {len(newly)}（按钩子加命令去重）", file=sys.stderr)
noisy = {(r['hook'], r['cmd']) for r in rows.values() if r['now'] == 0 and r['base'] == 0 and r['nnow'] > r['nbase']}
print(f"# 退出码都是 0 而修补后多记检出的不同命令数 {len(noisy)}", file=sys.stderr)
for h, c in sorted(noisy): print("NOISE\t" + h + "\t" + repr(c[:100]))
