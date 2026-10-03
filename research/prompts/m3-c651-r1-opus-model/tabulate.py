#!/usr/bin/env python3
"""C651 攻方：把 run_batch.sh 的日志压成「每个候选在每段历史上、全池槽 0 都不可用的那些崩溃态里择到什么」。
用法：python3 tabulate.py <草稿目录> <批号> [--detail 历史名子串]
每格：core=实现择到的（new：本池槽 1；stale：旧池残留槽；refuse：整池拒）、oracle=恢复过不过 oracle、
checker=checker 读到的槽 1、red=checker 判红的不变量；后面是态数。全池槽 0 都可用的态里判红的另报 other_red。"""
import collections, glob, os, re, sys

def spacing_for(io_min):
    return -(-4096 // io_min) * io_min

def cls(text, new_spacing, old_spacing):
    if text.startswith("Err:"):
        return "refuse"
    m = re.match(r"(new|old|other)@(\d+)g(\d+)", text)
    if not m:
        return text
    sp = int(m.group(2))
    if m.group(1) == "new" and sp == new_spacing:
        return f"new(g{m.group(3)})"
    return f"stale(g{m.group(3)})" if sp == old_spacing or m.group(1) == "old" else f"?{text}"

def checker_cls(text, new_spacing, old_spacing):
    parts = [cls(p, new_spacing, old_spacing) if "@" in p else p for p in text.split("|")]
    return "|".join(parts)

draft, batch = sys.argv[1], sys.argv[2]
detail = sys.argv[4] if len(sys.argv) > 3 and sys.argv[3] == "--detail" else None
cells = collections.defaultdict(collections.Counter)
other = collections.Counter()
labels = []
for path in sorted(glob.glob(os.path.join(draft, f"b{batch}-*.log"))):
    label = re.match(rf"b{batch}-\d+-(.+)\.log", os.path.basename(path)).group(1)
    labels.append(label)
    for line in open(path):
        if not line.startswith("name=sig "):
            continue
        h = re.search(r" h=(\S+)", line).group(1)
        count = int(re.search(r" count=(\d+)", line).group(1))
        y = int(re.search(r":y=(\d+)", h).group(1))
        xm = re.search(r":x=(\d+)", h)
        new_sp, old_sp = spacing_for(y), (spacing_for(int(xm.group(1))) if xm else None)
        core = re.search(r" core=(\S+)", line).group(1)
        oracle = re.search(r" oracle=(\S+)", line).group(1)
        chk = re.search(r" checker_slot1=(\S+)", line).group(1)
        red = re.search(r" checker_red=(\[.*\])", line).group(1)
        if " all_slot0_unusable=true" in line:
            key = f"core={cls(core, new_sp, old_sp)} oracle={oracle} checker={checker_cls(chk, new_sp, old_sp)} red={red}"
            cells[(label, h)][key] += count
        else:
            other[(label, oracle, red)] += count
for (label, h), counter in sorted(cells.items()):
    if detail and detail not in h:
        continue
    print(f"cell label={label} h={h} " + " ; ".join(f"{k} x{v}" for k, v in sorted(counter.items())))
for (label, oracle, red), v in sorted(other.items()):
    print(f"other_red label={label} oracle={oracle} red={red} states={v}")
print(f"labels={','.join(labels)} cells={len(cells)}")
