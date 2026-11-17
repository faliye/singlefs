#!/usr/bin/env python3
"""运行库与水位设计轮正推腿的算术模型：纯算术，不起 crates 里的任何代码。

四块：
  A  发布预算 B_gc 与一批搬几个单元 K 的下界（一条意图在一份 B_gc 里做不完，会卡在用户改动重新给预算之前）；
  B  水位候选在几种盘容量上「空盘起不起跑」；
  C  全空段水位要留多少余量才赶得上：余量 = 写入速率 × (周期 + 一趟整理的时长)；
  D  后台线程的睡眠周期与核心层定时兜底的间隔双重把关：定速调度加抖动，兜底间隔等于周期时隔一次才放行。

常量从 --frozen 给的树里读出来对一遍（读不到、对不上就退 1）；输入速率、抖动、盘容量是拍的（模型的参数，不是量的）。
每个断言里的数是模型作者手算的，跑出来不等就退 1。
用法：python3 watermark_and_budget_model.py --frozen <树根>
"""
import argparse
import math
import re
import sys

failures = []


def check(name, condition, detail=""):
    print(("ok   " if condition else "FAIL ") + name + (" " + detail if detail else ""))
    if not condition:
        failures.append(name)


def read_constant(path, pattern):
    with open(path, encoding="utf-8") as handle:
        text = handle.read()
    found = re.search(pattern, text)
    return int(found.group(1).replace("_", "")) if found else None


def anchors(frozen):
    lib = frozen + "/crates/singlefs-format/src/lib.rs"
    plan = frozen + "/crates/singlefs-core/src/compaction_plan.rs"
    values = {
        "SLOT_BYTES": read_constant(lib, r"pub const SLOT_BYTES: u64 = ([0-9_]+);"),
        "CLUSTER_SEGMENT_SLOTS": read_constant(lib, r"pub const CLUSTER_SEGMENT_SLOTS: u64 = ([0-9_]+);"),
        "S_MIN": read_constant(lib, r"pub const ROOT_RING_SLOTS_PER_REGION_MINIMUM: u64 = ([0-9_]+);"),
        "S_MAX": read_constant(lib, r"pub const ROOT_RING_SLOTS_PER_REGION_MAXIMUM: u64 = ([0-9_]+);"),
        "S_AT_MKFS": read_constant(lib, r"pub const ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = ([0-9_]+);"),
        "N0": read_constant(plan, r"pub const EMPTY_CLUSTER_SEGMENTS_FLOOR_OF_THE_FIRST_VERSION: u64 = ([0-9_]+);"),
    }
    with open(frozen + "/crates/singlefs-core/src/root_ring.rs", encoding="utf-8") as handle:
        values["mkfs_default_floor_is_minimum_plus_one"] = int("ROOT_RING_SLOTS_PER_REGION_MINIMUM + 1;" in handle.read())
    with open(plan, encoding="utf-8") as handle:
        plan_text = handle.read()
    values["budget_formula_present"] = int(
        "(3 * root_ring_slots_per_region)" in plan_text and ".saturating_sub(4)" in plan_text
    )
    expected = {
        "SLOT_BYTES": 16384,
        "CLUSTER_SEGMENT_SLOTS": 64,
        "S_MIN": 4,
        "S_MAX": 16,
        "S_AT_MKFS": 8,
        "N0": 1,
        "budget_formula_present": 1,
        "mkfs_default_floor_is_minimum_plus_one": 1,
    }
    for name, want in expected.items():
        check("anchor " + name, values[name] == want, "got=%s want=%s" % (values[name], want))
    return values


def budget(slots_per_region):
    """compaction_plan.rs 的 publish_budget_of_a_compaction_run(S, 0)：3S - 4，至少 1。"""
    return max(3 * slots_per_region - 4, 1)


def publishes_of_one_intent(units_in_the_region, batch_units):
    """签发一次，之后每批一次；最后一批同一次发布删意图（D26 已定项 2 的 BatchCompletingTheIntent），所以不另加删意图那一次。"""
    return 1 + math.ceil(units_in_the_region / batch_units)


def part_a():
    expected_budget = {4: 8, 5: 11, 6: 14, 7: 17, 8: 20, 9: 23, 10: 26, 11: 29, 12: 32, 13: 35, 14: 38, 15: 41, 16: 44}
    expected_min_batch_units_for_64 = {4: 10, 5: 7, 6: 5, 7: 4, 8: 4, 9: 3, 10: 3, 11: 3, 12: 3, 13: 2, 14: 2, 15: 2, 16: 2}
    for slots_per_region, want in expected_budget.items():
        check("A B_gc(S=%d)" % slots_per_region, budget(slots_per_region) == want, "got=%d want=%d" % (budget(slots_per_region), want))
    for slots_per_region, want in expected_min_batch_units_for_64.items():
        smallest = next(k for k in range(1, 65) if publishes_of_one_intent(64, k) <= budget(slots_per_region))
        check(
            "A 一条 64 单元的意图在一份 B_gc 里做得完的最小 K (S=%d)" % slots_per_region,
            smallest == want,
            "got=%d want=%d" % (smallest, want),
        )
    k_for_every_s = max(
        next(k for k in range(1, 65) if publishes_of_one_intent(64, k) <= budget(s)) for s in range(4, 17)
    )
    check("A 对 S 取 4 到 16 都做得完的 K 下界", k_for_every_s == 10, "got=%d want=10" % k_for_every_s)
    k_for_mkfs_pools = max(
        next(k for k in range(1, 65) if publishes_of_one_intent(64, k) <= budget(s)) for s in range(5, 9)
    )
    check("A mkfs 做得出的池（S 取 5 到 8）的 K 下界", k_for_mkfs_pools == 7, "got=%d want=7" % k_for_mkfs_pools)
    never_fits_at_k1 = [s for s in range(4, 17) if publishes_of_one_intent(64, 1) > budget(s)]
    check("A K = 1 时 64 单元的意图一份 B_gc 都做不完的 S", never_fits_at_k1 == list(range(4, 17)), "got=%s" % never_fits_at_k1)


def candidates():
    """水位候选：W(N) 是 N 个整段的盘上「全空段数少于它就越线」的线。N0 + 1 = 2 是下限（越线要早于前台被合取 7 拒）。"""
    floor_line = 2
    return {
        "E1c4": lambda n: 4,
        "E1c16": lambda n: 16,
        "E2p2": lambda n: max(floor_line, math.ceil(n * 2 / 100)),
        "E2p5": lambda n: max(floor_line, math.ceil(n * 5 / 100)),
        "E2p10": lambda n: max(floor_line, math.ceil(n * 10 / 100)),
        "E3": lambda n: max(floor_line, min(16, math.ceil(n * 5 / 100))),
    }


def part_b():
    sizes_in_segments = {"six_segments": 6, "64GiB": 65536, "1TiB": 1048576, "16TiB": 16777216}
    fires_on_empty = sorted(
        (name, label)
        for name, line in candidates().items()
        for label, n in sizes_in_segments.items()
        if n < line(n)
    )
    check("B 空盘就越线的 (候选, 盘)", fires_on_empty == [("E1c16", "six_segments")], "got=%s" % fires_on_empty)
    for name, line in candidates().items():
        print("     line %-6s %s" % (name, {label: line(n) for label, n in sizes_in_segments.items()}))
    for label, n in sizes_in_segments.items():
        left = n - math.ceil(n * 95 / 100)
        print("     perfectly-packed-95%%  %-12s empty_segments=%d" % (label, left))
    check("B 6 段的盘装满 95% 全空段为 0", 6 - math.ceil(6 * 95 / 100) == 0)
    check("B 1TiB 装满 95% 全空段 52428", 1048576 - math.ceil(1048576 * 95 / 100) == 52428)


def part_c():
    n0 = 1
    expected = {
        (10, 10): 401,
        (10, 60): 901,
        (100, 10): 4001,
        (100, 60): 9001,
        (1000, 10): 40001,
        (1000, 60): 90001,
    }
    period = 30
    for (rate_in_mebibytes_per_second, run_seconds), want in expected.items():
        line = n0 + rate_in_mebibytes_per_second * (period + run_seconds)
        check(
            "C 赶得上的最小线 r=%d MiB/s 一趟 %d s" % (rate_in_mebibytes_per_second, run_seconds),
            line == want,
            "got=%d want=%d" % (line, want),
        )
    check("C 一个整段是 1 MiB", 16384 * 64 == 1048576)


def dues(interval_microseconds, wake_times_microseconds):
    """CompactionOfThisMount::check_the_timer_fallback 的判法：没查过或离上一次查 >= 间隔就放行并记下这一次。"""
    last = None
    result = []
    for index, now in enumerate(wake_times_microseconds):
        if last is None or now - last >= interval_microseconds:
            last = now
            result.append(index)
    return result


def part_d():
    period = 30_000_000
    wakes = [k * period + (2000 if k % 2 == 1 else 0) for k in range(10)]
    due_when_gate_equals_period = dues(period, wakes)
    due_when_gate_is_half = dues(period // 2, wakes)
    check("D 定速调度加抖动，间隔等于周期时放行的醒来", due_when_gate_equals_period == [0, 1, 3, 5, 7, 9], "got=%s" % due_when_gate_equals_period)
    check("D 间隔取周期的一半时放行的醒来", due_when_gate_is_half == list(range(10)), "got=%s" % due_when_gate_is_half)
    fixed_delay = [k * (period + 1500) for k in range(10)]
    check("D 定延迟调度间隔等于周期全放行", dues(period, fixed_delay) == list(range(10)))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--frozen", required=True)
    arguments = parser.parse_args()
    anchors(arguments.frozen)
    part_a()
    part_b()
    part_c()
    part_d()
    print("name=model_summary failures=%d" % len(failures))
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
