#!/usr/bin/env python3
"""B8：一次 unlink 调用发几次发布。意图 1 次 + 批：每批释放尾部 K 个数据单元，最后一批释放剩下全部并删记录、与删意图同一次发布。
N = 文件的数据单元数，K = 批大小。N = 0（空 inode）也要一次最后一批删记录。发布数 = 1 + max(1, ⌈N ÷ K⌉)。
再数崩溃点：每次发布末尾一道根槽 FUA，「意图之后」= 第 1 道根槽写之后，「第 k 批之后」= 第 k + 1 道之后。
"""
def publishes(n, k): return 1 + max(1, -(-n // k))
for n, k in ((0, 144), (1, 144), (3, 144), (3, 1), (144, 144), (145, 144), (240, 144), (1000, 144), (10000, 144), (10000, 1)):
    p = publishes(n, k)
    print(f"E name=publishes N={n} K={k} publishes={p} crash_after_intent=root_write_1 crash_after_batch_k=root_write_k+1 last_batch_publish={p}")
# 产品批大小 144（crates/singlefs-format/src/lib.rs EXTENT_TREE_LOWER_LEAF_DATA_UNITS）下，一次 unlink 调用最长发几次发布：
# 文件最多多少单元？第一版单文件，数据单元 32 KiB，4 GiB 池上最大文件单元数 ≈ 单元区槽数 ÷ 2
slots = 211968
print(f"E name=max_file_publishes_4GiB max_units={slots // 2} publishes={publishes(slots // 2, 144)}")
