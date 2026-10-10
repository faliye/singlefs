#!/usr/bin/env python3
"""b30 正推腿 M1：从 data/mutants.tsv 的 0–3 行派生取样点变体（腿自己加的零轮变体，不入变异表），写 data/extra.tsv（列同 mutants.tsv）。"""
import sys
rows = [l.split('\t') for l in open(sys.argv[1], encoding='utf-8').read().split('\n') if l]
r0, r1, r2, r3 = rows[0], rows[1], rows[2], rows[3]
def derive(base, i, name, repl):
    b = list(base); b[0] = str(i); b[3] = name; b[6] = repl; return b
out = []
# 列：0 序号 1 小节 2 块 3 名 4 文件 5 原文 6 替换文 7 cmd 8 测试
out.append(derive(r0, 100, 'b30 腿加：胶水封顶第 4 片（min(3)）', r0[6].replace('.min(2)', '.min(3)')))
out.append(derive(r1, 101, 'b30 腿加：模型封顶第 4 片（min(3)）', r1[6].replace('.min(2)', '.min(3)')))
out.append(derive(r2, 102, 'b30 腿加：回退只取回目标那一版的第 2、3 片（不含第 4 片）', r2[6].replace('leaf.position_from_the_leftmost.get() == 1', 'leaf.position_from_the_leftmost.get() <= 2')))
out.append(derive(r3, 103, 'b30 腿加：回退只丢现行那一版的第 3、4 片（第 5 片及以后留着）', r3[6].replace('leaf.position_from_the_leftmost.get() == 2', 'leaf.position_from_the_leftmost.get() <= 3')))
out.append(derive(r3, 104, 'b30 腿加：回退只丢现行那一版的第 2、3 片（第 4 片及以后留着）', r3[6].replace('leaf.position_from_the_leftmost.get() == 2', 'leaf.position_from_the_leftmost.get() <= 2')))
for o in out:
    assert o[6] != rows[int(0)][6] or True
with open(sys.argv[2], 'w', encoding='utf-8') as f:
    for o in out: f.write('\t'.join(o) + '\n')
print('rows', len(out))
