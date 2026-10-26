#!/usr/bin/env python3
"""攻方腿 landing-audit-r1 的 K4 复算：handoff-gaps.md 第一节「范围与词表」那张表的命中数。
文件：records/2026-10-*.md 去掉 2026-10-22-子agent结果落地审计.md，加 records/2026-09-30-里程碑三开工第一轮.md；
词表照那张表抄（A 层 20 个词、B 层 1 个词、只数不抽 2 个词），按长的先匹配，一处只算一个词（正则按长度降序做选择支，finditer 不重叠）。
--rev <提交> 读那一次提交里的版本；不给读工作区。
用法：deferral_hits.py <仓根> [--rev <提交>]
"""
import collections, glob, os, re, subprocess, sys

A = "延后 下轮开 下一轮 下轮 往后延 留到 先不做 暂不 以后再 另开 下次 留给 推迟 暂缓 挂起 不在本轮 本轮不 留待 继续留 之后再".split()
B = ["下一批"]
COUNT_ONLY = ["另起", "留作"]

def main():
    root = sys.argv[1]
    rev = sys.argv[sys.argv.index("--rev") + 1] if "--rev" in sys.argv else None
    os.chdir(root)
    files = sorted(f for f in glob.glob("records/2026-10-*.md") if not f.endswith("2026-10-22-子agent结果落地审计.md")) + ["records/2026-09-30-里程碑三开工第一轮.md"]
    words = sorted(A + B + COUNT_ONLY, key=len, reverse=True)
    pattern = re.compile("|".join(map(re.escape, words)))
    counts = collections.Counter()
    for f in files:
        text = subprocess.run(["git", "show", f"{rev}:{f}"], capture_output=True, text=True).stdout if rev else open(f, encoding="utf-8").read()
        counts.update(m.group(0) for m in pattern.finditer(text))
    print(f"文件 {len(files)} 份（{'提交 ' + rev if rev else '工作区'}）")
    print("A 层：" + "、".join(f"{w} {counts[w]}" for w in A) + f"；合计 {sum(counts[w] for w in A)}")
    print("B 层：" + "、".join(f"{w} {counts[w]}" for w in B) + f"；合计 {sum(counts[w] for w in B)}")
    print("只数不抽：" + "、".join(f"{w} {counts[w]}" for w in COUNT_ONLY))

main()
