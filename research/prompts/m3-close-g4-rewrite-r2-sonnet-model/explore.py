#!/usr/bin/env python3
"""驱动：对 slot_protocol_model 的崩溃序列做穷举，打 name=... 行。用法：python3 -I explore.py"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from slot_protocol_model import crash_states, mount_programme, valid  # noqa: E402


def explore(start, depth, publishes, fault_every_run=False, fault_first_run=False, outcomes="PNT", **flags):
    """从 start 起连崩 depth 次。交回 (崩溃状态数, 无槽状态数, 第一个反例的路径, 首次出现无槽的段号集合)。"""
    states_seen = 0
    dead_states = 0
    first_counterexample = None
    first_dead_segments = set()
    frontier = [(tuple(start), [])]
    for level in range(depth):
        next_frontier = {}
        hide = fault_every_run or (fault_first_run and level == 0)
        for state, path in frontier:
            for after, description in crash_states(
                state, mount_programme(publishes, **flags), hide_newest=hide, outcomes=outcomes
            ):
                states_seen += 1
                step = path + [f"{description}->{after}"]
                if not valid(after):
                    dead_states += 1
                    first_dead_segments.add(int(description.split(":")[0][3:]))
                    if first_counterexample is None:
                        first_counterexample = step
                    continue
                next_frontier.setdefault(after, step)
        frontier = list(next_frontier.items())
        frontier = [(s, p) for s, p in frontier]
    return states_seen, dead_states, first_counterexample, sorted(first_dead_segments)


def report(name, *args, **kwargs):
    states, dead, example, segments = explore(*args, **kwargs)
    print(f"name={name} states={states} states_without_a_verified_slot={dead} segments_where_it_first_appears={segments}")
    if example:
        print(f"name={name} first_counterexample={' | '.join(example)}")


def main():
    mkfs_done = (1, 1)  # 两槽都是世代号 1：mkfs 末段整落、屏障过
    for publishes in (0, 1, 2, 3):
        for depth in (1, 2, 3):
            report(f"post_mkfs_publishes{publishes}_depth{depth}", mkfs_done, depth, publishes)
    for depth in (1, 2, 3):
        report(f"post_mkfs_fault_in_the_first_run_only_depth{depth}", mkfs_done, depth, 2, fault_first_run=True)
    report("stress_fault_in_every_run_depth3", mkfs_done, 3, 2, fault_every_run=True)
    print("# 崩溃注入第二、三截的抽样域：整写子集，撕裂并进没持久（outcomes=PN）。正确协议与坏协议在这一域里怎样")
    for depth in (1, 2, 3):
        report(f"injection_domain_post_mkfs_depth{depth}", mkfs_done, depth, 2, outcomes="PN")
    report("injection_domain_control_no_barrier_between_the_two_acquisition_rounds_depth3", mkfs_done, 3, 1, outcomes="PN", merged_rounds=True)
    report("injection_domain_control_mkfs_last_segment_only_slot_zero_landed_depth3", (1, None), 3, 1, outcomes="PN")
    for depth in (1, 2, 3):
        report(f"injection_domain_control_mut_b1_depth{depth}", mkfs_done, depth, 1, outcomes="PN", mut_b1=True)
    print("# 对照臂（枚举域 PNT）：下面每一行都应当有无槽状态")
    report("control_mkfs_last_segment_only_slot_zero_landed_depth1", (1, None), 1, 0)
    report("control_mkfs_last_segment_only_slot_one_landed_depth1", (None, 1), 1, 0)
    report("control_mut_b1_zero_the_other_slot_in_the_same_segment_depth1", mkfs_done, 1, 1, mut_b1=True)
    report("control_no_barrier_between_the_two_acquisition_rounds_depth1", mkfs_done, 1, 0, merged_rounds=True)
    print("# 补写本身：从两槽全废起的一次挂载（取号两轮：世代号 1 落槽 1，世代号 2 落槽 0），无槽状态只许出现在第 0 段")
    report("rewrite_of_a_device_without_slots_depth1", (None, None), 1, 0)
    report("rewrite_of_a_device_without_slots_then_a_second_crash_depth2", (None, None), 2, 0)


if __name__ == "__main__":
    main()
