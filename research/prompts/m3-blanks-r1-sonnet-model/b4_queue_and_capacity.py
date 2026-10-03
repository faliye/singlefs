#!/usr/bin/env python3
"""B4：deleted_inodes 队列长度上界与一个根装得下几个号。纯算术 + 小事件模拟，不与实现共用代码。
格式常量照 kb 抄成本地常量，末尾回比冻结副本里的 index_node_entry_capacity 公式（头 = 115 + 2 × key 宽，D8 已定项 11）。
"""
import itertools, random
NODE, SLOT = 16384, 16384
def cap(key_w, entry_w): return (NODE - (115 + 2 * key_w)) // entry_w
KEY = 8
print(f"E name=capacity leaf_entries_key8_entry8={cap(KEY, 8)} internal_entries={cap(KEY, KEY + 86)} "
      f"two_level_keys={cap(KEY, 8) * cap(KEY, KEY + 86)} three_level_keys={cap(KEY, 8) * cap(KEY, KEY + 86) ** 2}")
# 产品上界：一个根装 2031 个号；inode 树一个根最多装 135 容器 × 233 条 = 31455 个 inode
inode_root_records = cap(8, 120) * 233
print(f"E name=inode_supply max_inodes_under_one_inode_root={inode_root_records} ratio_to_deleted_inodes_root={inode_root_records / cap(KEY, 8):.2f}")

# 三种接口形态下，一池寿命里「队列里同时有几个号」的最大值（事件模拟：unlink 调用、批失败、崩溃、续做挂载）
def simulate(shape, seed, calls=40, fail_p=0.15, crash_p=0.1):
    r = random.Random(seed); queue = []; peak = 0; next_inode = 1; live = list(range(1, 200))
    def drain():
        while queue: queue.pop(0)
    for _ in range(calls):
        if r.random() < crash_p:                       # 崩溃 + 可写挂载：挂载里把队列走到空（D8 已定项 16 第 6 条）
            drain()
        if not live: break
        ino = live.pop()
        if shape == "甲 一次调用：先走空旧队列，再写意图，再跑完全部批":
            drain()                                     # 前一次调用失败留下的，先续做
            queue.append(ino); peak = max(peak, len(queue))
            if r.random() >= fail_p: drain()            # 批中途失败（ENOSPC、设备错）：号留在队列里
        elif shape == "乙 一次调用：写意图后立即返回，批由续做做":
            queue.append(ino); peak = max(peak, len(queue))
        elif shape == "丙 一次调用：不先走空，直接写意图，再跑批":
            queue.append(ino); peak = max(peak, len(queue))
            if r.random() >= fail_p: drain()
    return peak
for shape in ("甲 一次调用：先走空旧队列，再写意图，再跑完全部批", "丙 一次调用：不先走空，直接写意图，再跑批",
              "乙 一次调用：写意图后立即返回，批由续做做"):
    peaks = [simulate(shape, s) for s in range(3000)]
    print(f"E name=queue_peak shape={shape} max_over_3000_seeds={max(peaks)} mean={sum(peaks) / len(peaks):.2f}")
# 乙形态下的上界不是常数：只受活 inode 数与调用次数限制。要多少次不续做的 unlink 才超过一个根：
print(f"E name=overflow_calls_needed shape=乙 calls_before_the_root_overflows={cap(KEY, 8) + 1}")
# 超过一个根时的两条出路各自的代价：拒绝 vs 照多层码 2 树长（transaction.rs 里 MultiLevelCodeTwoTree 今天两个成员）
print("E name=growth_nodes entries=2032 nodes_after_split=3 extra_slots_per_device=2 (一个新根 + 一个右半，D8 已定项 11「多层码 2 树怎么长」)")
