#!/usr/bin/env python3
"""light-selftest-scan.sh 的输出目录 → 每件起了哪些真的 cargo / rustc / qemu / herd7 / verus / z3（路径不在 /tmp 下、execve 成功的），
按参数分两类：只问版本的（-V、-vV、--version、--print …）与别的。只打有真重活 execve 的那几件，末行是总计。
用法：scan-summary.py <输出目录> <scan-table.txt>"""
import collections, re, sys

out, table = sys.argv[1], sys.argv[2]
heavy = re.compile(r'^\d+\s+execve\("(?!/tmp/)([^"]*/(cargo|rustc|rustdoc|cargo-verus|verus|rust-verify|z3|herd7|qemu-system-[a-z0-9_]+))", \[(.*?)\], .*= 0$')
version_only = {"-V", "-vV", "--version", "-Vv"}
rows = [line.rstrip("\n").split("\t") for line in open(table, encoding="utf-8") if re.match(r"^\d+\t", line)]
items_with_heavy = 0
for row in rows:
    index, key, rc = row[0], row[1], row[2]
    kinds = collections.Counter()
    for line in open(f"{out}/{index}.strace", encoding="utf-8", errors="replace"):
        found = heavy.match(line.strip())
        if not found:
            continue
        arguments = re.findall(r'"((?:[^"\\]|\\.)*)"', found.group(3))[1:]
        tool = found.group(2)
        kind = "版本" if arguments and set(arguments) <= version_only else "别的:" + " ".join(arguments)[:80]
        kinds[(tool, kind)] += 1
    if kinds:
        items_with_heavy += 1
        print(f"{index}\t{key}\t{rc}\t" + "；".join(f"{tool} {kind} ×{count}" for (tool, kind), count in sorted(kinds.items())))
print(f"总计：{len(rows)} 件，起了真 cargo / rustc / qemu / herd7 / verus / z3 的 {items_with_heavy} 件")
