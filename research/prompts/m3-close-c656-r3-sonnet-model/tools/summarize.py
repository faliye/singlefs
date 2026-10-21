"""把原型 run 模式的 name=sig 行按结局类汇总。
用法：python3 summarize.py [--merge] <日志>…（--merge：几份分段日志合成一份汇总）
类：phase（落到哪一段）× 结局（old_readback_intact / new_complete / refused / 其余原样）；
new_complete 再按可写挂载 Ok / Err 分；refused 再按三种守卫与再跑 mkfs 的结局汇总（每个 fsid 一行）。"""
import re
import sys
from collections import Counter

merge = "--merge" in sys.argv
paths = [a for a in sys.argv[1:] if a != "--merge"]
groups = [paths] if merge else [[p] for p in paths]
for group in groups:
    path = group[0] if not merge else "+".join(g.split("/")[-1] for g in group)
    total = 0
    by_phase = Counter()
    new_writable = Counter()
    refused_guard = Counter()
    refused_rerun = Counter()
    mixed = 0
    old_after = 0
    head = ""
    lines = [l for g in group for l in open(g)]
    for line in lines:
        if line.startswith("name=done"):
            head = line.strip()
        m = re.match(r"name=sig h=(\S+) order=(\S+) count=(\d+) (.*)$", line.rstrip("\n"))
        if not m:
            continue
        count = int(m.group(3))
        rest = m.group(4)
        total += count
        phase = re.search(r"phase=(\S+)", rest).group(1)
        verdict = rest.split("verdict=", 1)[1]
        kind = verdict.split(" ", 1)[0]
        by_phase[(phase, kind)] += count
        if kind.startswith("MIXTURE"):
            mixed += count
        if kind in ("old_readback_intact", "OLD_READBACK_NOT_INTACT") and phase != "seg0":
            old_after += count
        if kind == "new_complete":
            w = re.search(r"writable=(\S+)", rest).group(1)
            new_writable[w.split("(")[0] + (":" + w[4:] if w.startswith("Err:") else "")] += count
        if kind == "refused":
            for tag, guards, outcome in re.findall(r"rerun_(\w+)\[([^\]]*)\]=(\S+)", verdict):
                refused_guard[(tag, guards)] += count
                refused_rerun[(tag, outcome)] += count
    print(f"== {path.split('/')[-1]} 总态数 {total}  {head}")
    for (phase, kind), n in sorted(by_phase.items()):
        print(f"   {phase:>5} {kind:<40} {n}")
    print(f"   混合(MIXTURE) {mixed}；作废之后(非 seg0)择到旧池 {old_after}")
    for w, n in sorted(new_writable.items()):
        print(f"   new_complete 可写挂载 {w}: {n}")
    for (tag, g), n in sorted(refused_guard.items()):
        print(f"   整池拒后再跑 mkfs fsid={tag} 守卫 {g}: {n}")
    for (tag, o), n in sorted(refused_rerun.items(), key=lambda kv: -kv[1])[:8]:
        print(f"   整池拒后再跑 mkfs fsid={tag} 结局 {o}: {n}")
