"""件表：每段历史全量（每个崩溃状态一个不落）切成 62 件、两批各 31 件，全部 17 组读故障；批号 = 历史的次序 × 2 + 半。
用法：python3 make-pieces.py <读故障组文件> > pieces.txt"""
import sys
faults = open(sys.argv[1]).read().strip()
histories = [("overwrite", "5", 20516), ("seq3", "5", 20521), ("overwrite", "ooww", 24623), ("seq3", "oiww", 24633)]
for order, (kind, k, states) in enumerate(histories):
    pieces = 62
    step = -(-states // pieces)
    for index, start in enumerate(range(0, states, step)):
        print(kind, k, start, min(states, start + step), faults, order * 2 + (index >= 31))
# 缩过的那一批（批号 10，31 件）：另三段历史只跑 6 组读故障，挂钟按批 0 实测（31 件 580 秒）定，缩法写在报告里。
REDUCED = "none,rec,mr0+mr1+rec,tt1+mr1+rec,tt1+mr0+mr1+rec,tt0+tt1+mr0+mr1+rec"
for (kind, k, states), pieces in zip(histories[1:], (10, 10, 11)):
    step = -(-states // pieces)
    for start in range(0, states, step):
        print(kind, k, start, min(states, start + step), REDUCED, 10)
