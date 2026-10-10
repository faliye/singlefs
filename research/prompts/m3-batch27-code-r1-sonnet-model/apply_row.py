#!/usr/bin/env python3
"""用法：apply_row.py <树根> <mutations.tsv 的行号（从 1 起）> | apply_row.py --check <树根> <行号…>
照 crates/mutations.tsv 一行的第 2 段（文件）、第 3 段（原文）、第 4 段（替换文）改 <树根> 下的文件；原文必须恰好命中一次。
--check 只数命中次数、核第 5 段的参数筛得到第 6 段的测试名，不改文件。不依赖 SINGLEFS_REPO。"""
import re
import shlex
import sys

def load(tree):
    return open(f"{tree}/crates/mutations.tsv", encoding="utf8").read().split("\n")

def unescape(text):
    return text.replace("\\n", "\n")

def main():
    if sys.argv[1] == "--check":
        tree = sys.argv[2]
        rows = load(tree)
        for number in map(int, sys.argv[3:]):
            fields = rows[number - 1].split("\t")
            assert len(fields) == 6, f"行 {number} 不是六段"
            source = open(f"{tree}/{fields[1]}", encoding="utf8").read()
            hits = source.count(unescape(fields[2]))
            args = shlex.split(fields[4])
            filters = args[args.index("--") + 1:]
            name = fields[5]
            sieve = any(f in name for f in filters)
            test_file = None
            for candidate in re.findall(r"--test (\w+)", fields[4]):
                test_file = candidate
            print(f"row={number} anchor_hits={hits} named_test_matched_by_a_filter={int(sieve)} test_target={test_file}")
        return
    tree, number = sys.argv[1], int(sys.argv[2])
    fields = load(tree)[number - 1].split("\t")
    path = f"{tree}/{fields[1]}"
    source = open(path, encoding="utf8").read()
    anchor = unescape(fields[2])
    assert source.count(anchor) == 1, f"原文命中 {source.count(anchor)} 次"
    open(path, "w", encoding="utf8").write(source.replace(anchor, unescape(fields[3])))
    print(f"applied row {number}: {fields[0]}")

main()
