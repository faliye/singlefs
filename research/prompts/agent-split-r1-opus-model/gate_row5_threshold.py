#!/usr/bin/env python3
"""攻方腿模型：新门禁「不抄规则」那一行（提案第 139 行，阈值没定）在不同阈值下各误判什么。
取两份规则目录的全部行（去首尾空白），按长度阈值 T 算：
  boilerplate：≥T、且在两份以上规则文件里逐字出现的行（表格分隔行、围栏、通用标题）——定义文件照常写就会撞上的，误红来源
  longest_boilerplate：这类行里最长的一行有多长——阈值要高过它才不误红
  one_char_edit：随手取一条 ≥T 的规则长行，改一个标点后还逐字相同吗（永远不同 ⇒ 该红不红）
只读仓里文件，不写仓。"""
import collections, glob, sys
files = sorted(glob.glob('/home/fy5090/code/singlefs/.claude/rules/*.md') + glob.glob('/home/fy5090/code/singlefs/.claude/singlefs-ai-sop/rules/*.md'))
where = collections.defaultdict(set)
for path in files:
    for line in open(path, encoding='utf-8'):
        stripped = line.strip()
        if stripped:
            where[stripped].add(path)
print(f'规则文件 {len(files)} 份，非空不同行 {len(where)} 条')
for threshold in (8, 12, 16, 24, 40, 80, 160):
    long_lines = [line for line in where if len(line) >= threshold]
    boiler = [line for line in long_lines if len(where[line]) >= 2]
    longest = max(boiler, key=len) if boiler else ''
    print(f'T={threshold:>3}  ≥T 的行 {len(long_lines):>5}  其中跨文件重复 {len(boiler):>3}  最长的一条（{len(longest)} 字符）：{longest[:70]!r}')
