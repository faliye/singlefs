#!/usr/bin/env python3
"""B2：墓碑容器 / 谱系容器的容器号怎么取。纯事件模拟，不与实现共用代码。
规则（候选）：下一个号 = 最大号 + 1；最大号从「现行那一版的树」读，空树读 0；首号 = 起点 A（A = 0 或 1）。
事件：unlink（意图那次发布写一条墓碑，容器满了开新容器）、回退（管理员回退：inode / extent / deleted_inodes 取 R_old，墓碑与谱系取现行）、
崩溃恢复落到较旧的根（被抛弃时间线上的容器仍留在盘上，新实例的第一个 txg = 环里读得出的最大 txg + 1）。
判据：全部写出过的容器（可达与不可达）里 (出生树, 类型, 容器号, 容器出生代) 四元组两两不同；三元组 (出生树, 类型, 容器号) 重号只许出现在崩溃恢复之后。
"""
import random, itertools
RPC = 3            # 只供测试的旋钮：一个容器装几条墓碑（产品 815；小值让容器多、事件撞得多）

class Pool:
    def __init__(s, start, rule="max_plus_one"):
        s.start, s.rule = start, rule
        s.txg = 3                      # 第一个文件版本 txg 3
        s.tomb = []                    # 现行版本里的墓碑容器：[number, birth, records]
        s.lineage = []                 # 现行版本里的谱系容器
        s.written = []                 # 一生写出过的容器 (树, 号, 出生代)，含被抛弃时间线上的
        s.versions = {s.txg: ([], [])} # txg -> (墓碑树, 谱系树) 的快照，供回退 / 恢复取
        s.counter = start              # 「计数器」规则用：内存里一个会随恢复回退的计数器
    def snap(s):
        s.versions[s.txg] = ([list(c) for c in s.tomb], [list(c) for c in s.lineage])
    def next_number(s, tree):
        if s.rule == "counter":        # 反例臂：内存计数器，不读树
            n = s.counter; s.counter += 1; return n
        mx = max((c[0] for c in tree), default=s.start - 1)   # 空树读 start-1，下一个 = start
        return mx + 1
    def publish_record(s, tree, name, rpc):
        s.txg += 1
        if not tree or tree[-1][2] >= rpc:                     # 开放容器满了才关
            tree.append([s.next_number(tree), s.txg, 0])
            s.written.append((name, tree[-1][0], s.txg))
        tree[-1][2] += 1
        s.snap()
    def unlink(s): s.publish_record(s.tomb, "tomb", RPC)
    def rollback(s, to):                                       # 墓碑 / 谱系随现行：不退回；写一条谱系记录
        s.publish_record(s.lineage, "lineage", 2039)
    def crash_recover_to(s, older_txg, abandoned_max_txg):
        t, l = s.versions[older_txg]
        s.tomb, s.lineage = [list(c) for c in t], [list(c) for c in l]
        if s.rule == "counter":
            s.counter = s.start   # 反例臂：挂载之后内存计数器从起点重数（不读树里的最大号）
        s.txg = abandoned_max_txg      # 新实例第一个 txg = 环里读得出的最大 txg + 1（D23 已定项 14 注 3）

def run(seed, start, rule):
    r = random.Random(seed); p = Pool(start, rule); crashes = 0
    for _ in range(60):
        e = r.choice(["unlink"] * 6 + ["rollback", "crash"])
        if e == "unlink": p.unlink()
        elif e == "rollback": p.rollback(None)
        else:
            older = r.choice(sorted(p.versions)); top = p.txg
            if older < top:
                p.crash_recover_to(older, top); crashes += 1
    quad = [(t, n, b) for t, n, b in p.written]
    tri = [(t, n) for t, n, b in p.written]
    # 可达那棵树里：墓碑容器号沿条目严格递增（I-9.21 / I-9.22）、同号只出现一次
    reach = [c[0] for c in p.tomb] ; bad = sum(1 for a, b in zip(reach, reach[1:]) if a >= b)
    return len(quad) - len(set(quad)), len(tri) - len(set(tri)), crashes, len(p.written), bad

for rule in ("max_plus_one", "counter"):
    for start in (0, 1):
        q = t = c = w = 0; bad_total = 0
        for seed in range(2000):
            a, b, cr, ww, bd = run(seed, start, rule); q += a; t += b; c += cr; w += ww; bad_total += bd
        print(f"E rule={rule} start={start} seeds=2000 containers_written={w} crash_events={c} "
              f"dup_4tuple={q} dup_3tuple_after_recovery={t} reachable_tree_not_strictly_increasing={bad_total}")
# 无崩溃只含回退：三元组重号必须为 0
q = t = 0
for seed in range(2000):
    r = random.Random(seed); p = Pool(1)
    for _ in range(80): (p.unlink() if r.random() < .8 else p.rollback(None))
    quad = [(a, b, c) for a, b, c in p.written]
    q += len(quad) - len(set(quad)); t += len(quad) - len({(a, b) for a, b, c in quad})
print(f"E rule=max_plus_one start=1 no_crash_only_rollback dup_4tuple={q} dup_3tuple={t}")
# 空树怎么读：max(空) = start-1，所以首号恒为 start；start=0 时首号 0，与 D8 已定项 6「0 = 无效 / 类型段 0 条目的容器号恒 0」同值
for start in (0, 1):
    p = Pool(start); print(f"E empty_tree start={start} first_number={p.next_number([])}")
