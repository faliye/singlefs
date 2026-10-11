#!/usr/bin/env python3
"""攻方腿模型（m3-run-compare-split-r2）：层 0 断点续跑与双机分片的切片（E5 副本 crash.rs 的 states_per_slice_independent_of_worker_threads：
片长 = max(⌈态数 / 65536⌉, 16)；layer0_progress.rs 文件头：第 i 台拿切片序号 % n == i 的片）与判定库的块（crash_amplification.rs 的
STATES_PER_BLOCK = 65536）对不对得齐：数一块 65536 态里的态落在几片、两台（n = 2）是不是都有份。态数取命令行给的几个。"""
import math
import sys

LAYER0_RESUMABLE_SLICE_COUNT = 65_536
LAYER0_MINIMUM_STATES_PER_SLICE = 16
STATES_PER_BLOCK = 1 << 16

for argument in sys.argv[1:]:
    name, text = argument.split("=")
    states = int(text)
    per_slice = max(math.ceil(states / LAYER0_RESUMABLE_SLICE_COUNT), LAYER0_MINIMUM_STATES_PER_SLICE)
    blocks = math.ceil(states / STATES_PER_BLOCK)
    on_both_hosts = 0
    for block in range(blocks):
        first = block * STATES_PER_BLOCK
        last = min(states, first + STATES_PER_BLOCK) - 1
        hosts = {slice_index % 2 for slice_index in range(first // per_slice, last // per_slice + 1)}
        on_both_hosts += len(hosts) == 2
    print(f"LAYER0_SLICE item={name} states={states} states_per_slice={per_slice} blocks={blocks} "
          f"slices_per_block={STATES_PER_BLOCK / per_slice:.2f} blocks_with_states_on_both_hosts={on_both_hosts}")
