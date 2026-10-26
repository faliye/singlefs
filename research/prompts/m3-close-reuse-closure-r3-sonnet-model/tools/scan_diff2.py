#!/usr/bin/env python3
"""scan_diff2.py <基准.tsv[,更多.tsv]> <名>=<变体.tsv[,更多.tsv]>…：逐入口比 rc、去重单元数、拒算、集合摘要，
基准与变体的 .sets 集合逐个比（掉了 / 多了几个单元；另报不计 lib_function_units.py 自己的数，因为变体改了它）。"""
import sys


def load(paths):
    rows = {}
    for path in paths.split(","):
        for line in open(path, encoding="utf-8"):
            f = line.rstrip("\n").split("\t")
            rows[f[1]] = dict(n=f[0], rc=f[2], raw=int(f[3]), uniq=int(f[4]), files=int(f[5]), refused=f[6],
                              digest=f[9], sets=path[:-4] + ".sets/" + f"{int(f[0]):02d}.set")
    return rows


def members(path):
    return set(open(path, encoding="utf-8").read().splitlines())


LIBRARY = "scripts/gate/lib_function_units.py"
base = load(sys.argv[1])
for argument in sys.argv[2:]:
    name, path = argument.split("=", 1)
    variant = load(path)
    changed, real_changed, refused_more = [], [], []
    lines = []
    for entry in base:
        a, b = members(base[entry]["sets"]), members(variant[entry]["sets"])
        gone = sorted(x for x in a - b if x.split("|")[2] != LIBRARY)
        new = sorted(x for x in b - a if x.split("|")[2] != LIBRARY)
        if a != b or base[entry]["rc"] != variant[entry]["rc"]:
            changed.append(entry)
            if gone or new or base[entry]["refused"] != variant[entry]["refused"]:
                real_changed.append(entry)
            lines.append(f"  {entry} rc {base[entry]['rc']}→{variant[entry]['rc']} 原始行 {base[entry]['raw']}→{variant[entry]['raw']} "
                         f"去重 {base[entry]['uniq']}→{variant[entry]['uniq']} 文件 {base[entry]['files']}→{variant[entry]['files']} "
                         f"拒算[{base[entry]['refused']}]→[{variant[entry]['refused']}] 掉{len(a - b)} 多{len(b - a)}（不计算法文件：掉{len(gone)} 多{len(new)}）")
        if variant[entry]["refused"] and variant[entry]["refused"] != base[entry]["refused"]:
            refused_more.append(entry)
    print(f"DIFF {name}: 入口 {len(base)} 个，集合或退出码变了 {len(changed)} 个，不计算法文件自己之后真变的 {len(real_changed)} 个，拒算与基准不同的 {len(refused_more)} 个")
    for line in lines:
        print(line)
