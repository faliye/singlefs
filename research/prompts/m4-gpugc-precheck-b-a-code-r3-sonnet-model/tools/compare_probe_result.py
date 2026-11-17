#!/usr/bin/env python3
"""把今天这棵树上跑出来的 oracle.out 里的 PROBE_RESULT 一行，与第二轮冻结副本上量的那一行（reference/ 下）逐字段比。

用法：compare_probe_result.py <第二轮那一行所在的文件> <今天跑出的 oracle.out>
只读两份文件，打出：字段个数、两边都有而值不同的字段（名字、旧值、新值）、只有一边有的字段名。
退出码：0 读到两行（有没有差异都是 0，差异只打印）；2 用法错或哪份文件里没有 PROBE_RESULT 那一行。
"""
import sys


def probe_result_fields(path):
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            if "PROBE_RESULT" in line:
                return dict(token.split("=", 1) for token in line.split()[2:] if "=" in token)
    return None


def main():
    if len(sys.argv) != 3:
        print(__doc__, file=sys.stderr)
        return 2
    old = probe_result_fields(sys.argv[1])
    new = probe_result_fields(sys.argv[2])
    if old is None or new is None:
        print("  ✗ 有一份文件里没有 PROBE_RESULT 那一行", file=sys.stderr)
        return 2
    differing = [(name, old[name], new[name]) for name in old if name in new and old[name] != new[name]]
    print(f"name=compare_probe_result fields_old={len(old)} fields_new={len(new)} differing={len(differing)}")
    for name, was, is_now in differing:
        print(f"name=compare_probe_result_differs field={name} second_round_tree={was} today_tree={is_now}")
    for name in old:
        if name not in new:
            print(f"name=compare_probe_result_only_in_second_round field={name}")
    for name in new:
        if name not in old:
            print(f"name=compare_probe_result_only_today field={name}")
    return 0


sys.exit(main())
