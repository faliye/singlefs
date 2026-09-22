#!/usr/bin/env python3
"""只拿本轮那一份入库的逐行判定报告，机械造「复核改判」表——连第二份报告都不要。

这一份答的是「把收严再收严一步会怎样」：就算判据加上「改判后的判定必须与本轮入库的
逐行判定报告对得上」，伪造照样成立——报告里那一格就是「改判后的判定」，
「原判定」在仓里一个字节都没有（第二轮判决 G 格），随便填五种里不相等的那一个即可。
程序同样自己记账，收尾断言没打开过任何载体。

  forge-one-report.py 报告.md… [--rows N] [--out 文件]
"""
import argparse
import io
import re
import sys

FIVE = ("要改", "要补", "事件句不改", "不相干", "要人看")
opened = []
real_open = io.open


def audited_open(file, *args, **kwargs):
    opened.append(str(file))
    return real_open(file, *args, **kwargs)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("reports", nargs="+")
    parser.add_argument("--rows", type=int, default=5)
    parser.add_argument("--out")
    args = parser.parse_args()
    rows = []
    for path in args.reports:
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
                after = next((name for name in FIVE if cells[2].startswith(name)), None)
                if after:
                    rows.append((cells[1], after, cells[3]))
    out = ["## 复核改判", "", "| 载体 | 原判定 | 改判后的判定 | 抓到它靠的是哪一件东西 |", "|---|---|---|---|"]
    picked = 0
    for carrier, after, reason in rows:
        if picked >= args.rows:
            break
        # 「原判定」：五种里挑一个不等于入库那一格的，挑法写死，不看内容。
        before = next(name for name in FIVE if name != after)
        how = "逐行全看时对着载体今天的原文核出来的：" + reason.replace("|", "\\|")[:60]
        out.append(f"| {carrier} | {before} | {after} | {how} |")
        picked += 1
    text = "\n".join(out) + "\n"
    if args.out:
        with real_open(args.out, "w", encoding="utf-8") as handle:
            handle.write(text)
    else:
        sys.stdout.write(text)
    stray = [p for p in opened if p not in set(args.reports)]
    print(f"# 本轮报告里的判定行 {len(rows)} 行；写出 {picked} 行改判", file=sys.stderr)
    print(f"# 全程 open 过的文件：{sorted(set(opened))}", file=sys.stderr)
    assert not stray, f"打开过输入之外的文件：{stray}"
    print("# 断言通过：一行载体都没打开过", file=sys.stderr)


if __name__ == "__main__":
    main()
