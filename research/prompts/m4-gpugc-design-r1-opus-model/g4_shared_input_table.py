#!/usr/bin/env python3
"""G4 小模型：两份「独立写」的分类核（一份逐条扫映射表，一份先建 key→位置的索引）共用同一份输入表的准备；
准备那一步照 version.mapped_units 取映射条目、漏了留存给快照的条目（mounted_session.rs 第 2826-2831 行那一步）。
看三种比对各红不红：两份核互比（D26 已定项 13 第 8 条的运行时复核）、「同一组输入」上的 CPU 对 GPU 对拍（七·3 的写法）、
两种模式各自从这一版取输入再比（七·4 的块序列比对那一类）。分类规则照 region_scan.rs 第 268-328 行的次序缩写（只留钉住、可搬、已死三类）。
用法：python3 -I g4_shared_input_table.py <种子> <R 里的单元数> <其中只被快照引用的单元数> [one-kernel-fault]
（第四个参数给 one-kernel-fault 时，第二份核在槽号最小的那一条上交错的类：自证运行时互比会红。）"""
import random, sys

seed, units_in_region, kept_only = int(sys.argv[1]), int(sys.argv[2]), int(sys.argv[3])
rng = random.Random(seed)
slots = rng.sample(range(0, 64, 2), units_in_region)
keys = {s: bytes([1]) + rng.randbytes(26) for s in slots}            # 码 1 数据单元的 27 字节 key
kept = set(rng.sample(slots, kept_only))                              # 这一版里已覆盖写、只剩快照引用的单元
mapped_units = [(keys[s], s) for s in slots if s not in kept]         # 这一版自己的映射
kept_entries = [(keys[s], s) for s in kept]                           # mapping_entries_kept_for_the_snapshots
full_table = mapped_units + kept_entries                               # CPU 一侧照 central_mapping_as_the_compaction_reads_it 取
prepared_table = list(mapped_units)                                   # 两份 GPU 核共用的那一份准备（漏了留存条目）

def classify_linear(table, slot, header_key):
    pointing = [k for k, s in table if s == slot]
    if not pointing and not any(k == header_key for k, _ in table):
        return 'dead'
    return 'movable' if pointing and all(k == header_key for k in pointing) else 'inconsistent'

def classify_indexed(table, slot, header_key):
    by_slot = {}
    for k, s in table:
        by_slot.setdefault(s, []).append(k)
    keys_in_table = {k for k, _ in table}
    pointing = by_slot.get(slot, [])
    if not pointing:
        return 'dead' if header_key not in keys_in_table else 'inconsistent'
    return 'movable' if all(k == header_key for k in pointing) else 'inconsistent'

gpu_one = {s: classify_linear(prepared_table, s, keys[s]) for s in slots}
gpu_two = {s: classify_indexed(prepared_table, s, keys[s]) for s in slots}
if len(sys.argv) > 4 and sys.argv[4] == 'one-kernel-fault':
    gpu_two[min(slots)] = 'inconsistent' if gpu_two[min(slots)] != 'inconsistent' else 'movable'
cpu_on_the_same_prepared_input = {s: classify_linear(prepared_table, s, keys[s]) for s in slots}
cpu_on_its_own_read_of_the_version = {s: classify_linear(full_table, s, keys[s]) for s in slots}

def differing(x, y):
    return sorted(s for s in slots if x[s] != y[s])

print(f'name=g4_model seed={seed} units_in_region={units_in_region} kept_only={kept_only}')
print(f'name=g4_runtime_two_kernels differing_entries={len(differing(gpu_one, gpu_two))} confirmed={sum(gpu_one[s]==gpu_two[s] for s in slots)}')
print(f'name=g4_test_same_input_cpu_vs_gpu differing_entries={len(differing(cpu_on_the_same_prepared_input, gpu_one))}')
print(f'name=g4_end_to_end_modes differing_entries={len(differing(cpu_on_its_own_read_of_the_version, gpu_one))} '
      f'gpu_dead_cpu_movable={sum(gpu_one[s]=="dead" and cpu_on_its_own_read_of_the_version[s]=="movable" for s in slots)}')
