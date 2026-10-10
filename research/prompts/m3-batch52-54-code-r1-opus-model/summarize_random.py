#!/usr/bin/env python3
"""把 base-k2-random.txt 的每段随机历史排成报告里那张表（种子、旋钮、回退成功次数、三棵树末版的片数与条数、三棵树根的层级、I-9.37 成立次数、有违例的判次）。用法：summarize_random.py <base-k2-random.txt>"""
import re, sys
total_checks = total_violating = 0
for line in open(sys.argv[1]):
    if not line.startswith('OPUS_K2_RANDOM'):
        continue
    seed = re.search(r'seed=(\d+)', line).group(1)
    knobs = re.search(r'knobs=Knobs \{ (.*?) \}', line).group(1)
    def cnt(name):
        items = re.findall(r'\((\d+), (\d+)\)', re.search(name + r'=\[(.*?)\] ', line).group(1))
        return f"{len(items)} 片 {sum(int(b) for a, b in items)} 条"
    levels = re.search(r'levels\(t,l,r\)=\((\d+),(\d+),(\d+)\)', line).groups()
    checks = int(re.search(r'checks=(\d+)', line).group(1)); total_checks += checks
    violating = int(re.search(r'violating_checks=(\d+)', line).group(1)); total_violating += violating
    rollbacks = re.search(r'rollbacks_ok=(\d+)', line).group(1)
    holds = re.search(r'i937=\{"Holds": (\d+)', line)
    print(f"| {seed} | {knobs} | {rollbacks} | {cnt('tombstone')} | {cnt('lineage')} | {cnt('retirement')} | {'/'.join(levels)} | {holds.group(1) if holds else 0} | {violating} |")
print(f"合计：判 {total_checks} 次，有违例的 {total_violating} 次")
