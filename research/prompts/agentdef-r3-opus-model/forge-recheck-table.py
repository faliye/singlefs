#!/usr/bin/env python3
"""不读一行载体，从两份已入库的逐行判定报告里机械造一张「复核改判」表。

第二轮攻方的 forge.py 造的是「复判抽样」表；这一份把同一手法搬到第三轮被攻的那张表上。
唯一的输入是两份报告（同一轮的两版，或两轮各一份）；程序全程只 open 这两个文件，
自己用 AuditedOpen 记账，收尾断言「没打开过任何载体」。

  forge-recheck-table.py 旧版报告… --new 新版报告… [--rows N] [--out 文件]
"""
import argparse
import collections
import io
import re
import sys

FIVE = ("要改", "要补", "事件句不改", "不相干", "要人看")
opened = []
real_open = io.open


def audited_open(file, *args, **kwargs):
    opened.append(str(file))
    return real_open(file, *args, **kwargs)


def normalise(verdict):
    for name in FIVE:
        if verdict.startswith(name):
            return name
    return None


def load(paths):
    table = {}
    for path in paths:
        inside = False
        with audited_open(path, encoding="utf-8", errors="replace") as handle:
            for line in handle:
                if line.startswith("## "):
                    inside = line.strip().startswith("## 逐行判定")
                    continue
                if not inside or not line.lstrip().startswith("|"):
                    continue
                cells = [c.strip() for c in re.split(r"(?<!\\)\|", line.strip().strip("|"))]
                if len(cells) < 4 or cells[0] == "组" or set(cells[0]) <= set("-: "):
                    continue
                table.setdefault(cells[1], (cells[2], cells[3]))
    return table


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("old", nargs="+")
    parser.add_argument("--new", nargs="+", required=True)
    parser.add_argument("--rows", type=int, default=6)
    parser.add_argument("--out")
    args = parser.parse_args()
    old, new = load(args.old), load(args.new)
    pairs = []
    for carrier in sorted(set(old) & set(new)):
        before, after = normalise(old[carrier][0]), normalise(new[carrier][0])
        if before and after and before != after:
            # 末格「抓到它靠的是哪一件东西」从新版自己的理由栏机械裁出来，一个字不是我写的。
            reason = new[carrier][1].replace("|", "\\|")
            pairs.append((carrier, before, after, reason))
    # 挑法也机械：每一种「原判定 → 改判后的判定」的组合各取第一行，凑不够再按序补。
    by_kind, picked = collections.defaultdict(list), []
    for row in pairs:
        by_kind[(row[1], row[2])].append(row)
    for kind in sorted(by_kind):
        picked.append(by_kind[kind][0])
    for row in pairs:
        if len(picked) >= args.rows:
            break
        if row not in picked:
            picked.append(row)
    picked = picked[:args.rows]
    out = ["## 复核改判", "", "| 载体 | 原判定 | 改判后的判定 | 抓到它靠的是哪一件东西 |", "|---|---|---|---|"]
    for carrier, before, after, reason in picked:
        how = f"逐行全看时对着载体今天的原文核出来的：{reason[:60]}"
        out.append(f"| {carrier} | {before} | {after} | {how} |")
    text = "\n".join(out) + "\n"
    if args.out:
        with real_open(args.out, "w", encoding="utf-8") as handle:
            handle.write(text)
    else:
        sys.stdout.write(text)
    inputs = set(args.old) | set(args.new)
    stray = [p for p in opened if p not in inputs]
    print(f"# 候选改判对 {len(pairs)} 组；这一次写出 {len(picked)} 行", file=sys.stderr)
    print(f"# 全程 open 过的文件：{sorted(set(opened))}", file=sys.stderr)
    assert not stray, f"打开过输入之外的文件：{stray}"
    print("# 断言通过：一行载体都没打开过", file=sys.stderr)


if __name__ == "__main__":
    main()
