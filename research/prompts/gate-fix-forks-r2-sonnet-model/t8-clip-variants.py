#!/usr/bin/env python3
"""T8 辩方复核：验证「乙（not-numbers 词表）+ 丙a 已落地的正则」能整词保留已登记领域词，
而丙a 现状（同一正则、不接词表）整词丢弃。三种实现并排跑同一批输入，输出可复跑核对。
用法：python3 t8-clip-variants.py
"""
import re

WORDS = {"SHA256", "RAID5"}

# 丙a 现状：.claude/scripts/gen-decision-items.py:25-42（clip 函数）逐字照抄的正则部分
def clip_bing_a(text, n):
    t = text[:n]
    while t.count('（') > t.count('）'):
        t = t[:t.rindex('（')].rstrip()
    if t == text:
        return t.rstrip()
    while True:
        t2 = re.sub(r'(?<![A-Za-z0-9._-])[A-Z]+-?\d+(?:\.\d+)*\s*$', '', t)
        t2 = re.sub(r'[\s/、,，·的与和]+$', '', t2)
        if t2 == t:
            break
        t = t2
    return t.rstrip()

# r1 判决现跑坐实那次失败用的老正则：r1-main-verification.md:102 引用「[A-Z]-?\d+」（无 +、无左边界）
def clip_old_yi_with_table(text, n, words):
    t = text[:n]
    while t.count('（') > t.count('）'):
        t = t[:t.rindex('（')].rstrip()
    if t == text:
        return t.rstrip()
    while True:
        m = re.search(r'[A-Z]-?\d+(?:\.\d+)?\s*$', t)
        if m and m.group(0).strip() in words:
            break
        t2 = re.sub(r'[A-Z]-?\d+(?:\.\d+)?\s*$', '', t)
        t2 = re.sub(r'[\s/、,，·的与和]+$', '', t2)
        if t2 == t:
            break
        t = t2
    return t.rstrip()

# 辩方提的修法：丙a 的正则（带 + 、带左边界）+ 词表判据
def clip_yi_plus_bing_a_regex(text, n, words):
    t = text[:n]
    while t.count('（') > t.count('）'):
        t = t[:t.rindex('（')].rstrip()
    if t == text:
        return t.rstrip()
    while True:
        m = re.search(r'(?<![A-Za-z0-9._-])[A-Z]+-?\d+(?:\.\d+)*\s*$', t)
        if m and m.group(0).strip() in words:
            break
        t2 = re.sub(r'(?<![A-Za-z0-9._-])[A-Z]+-?\d+(?:\.\d+)*\s*$', '', t)
        t2 = re.sub(r'[\s/、,，·的与和]+$', '', t2)
        if t2 == t:
            break
        t = t2
    return t.rstrip()


CASES = [
    ("校验和算法取 SHA256 摘要", 13),
    ("条带冗余按 RAID5 实现", 11),
    ("参照 D-451 号决策处理", 6),
    ("参照 D-451 号决策处理", 7),
    ("参照 D-451 号决策处理", 8),
    ("参照 D-451 号决策处理", 10),
]

if __name__ == '__main__':
    for text, n in CASES:
        print(f"text={text!r} n={n} text[:n]={text[:n]!r}")
        print(f"  丙a现状（无词表）            : {clip_bing_a(text, n)!r}")
        print(f"  老乙（老正则+词表，r1现查那次）: {clip_old_yi_with_table(text, n, WORDS)!r}")
        print(f"  乙+丙a正则（辩方提的修法）    : {clip_yi_plus_bing_a_regex(text, n, WORDS)!r}")
