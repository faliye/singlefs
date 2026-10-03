#!/usr/bin/env python3
"""m3-conc-r1 攻方：C58（串行提交撞天花板没人报警） 交点报警的计数模型。不引 crates/ 的代码。

报警按 C58 登记行写成：一次 checkpoint 的条目数 entries ≥ T 判红，T = 单线程合并速率 × 一次 fdatasync 的时长。
第一部分：三个交点来源（问题单 Q8：mkfs 时实测写进系统配置 / 运行时配置 / 编译期常量）在几种环境上算出的 T。
第二部分：「天花板已经挪到 CPU」而报警不响的条目数区间——发布的 CPU 代价里有一项随盘容量线性（C567），与条目数无关。
第三部分：C58 写的判别力自证（「造一个条目数刻意翻倍的负载，检查必须红」）对一组坏实现抓不抓得到，
         与两种收严写法（两面：基线绿 + 翻倍红；再加交点来源自证）对照。
用法：c58_alarm_model.py <fsync_probe.out>
"""
import re
import sys

# E17 页的区间端点与 C58 用的点（C58 登记行原文 31.2M 条目/秒，事实表核过 E17 页里没有逐字的 31.2）
MERGE_RATES = {"e17_low": 29.9e6, "c58_point": 31.2e6, "e17_high": 31.8e6}
E44_FSYNC_US = 1e6 / 2785  # E44 sync_1 2785 次/秒
COMPILE_TIME_CONSTANT_T = 11200  # C58「约 1.1 万」：31.2M × 359 µs = 11200.8，取整


def probe_values(path):
    fsync = {}
    copy = {}
    for line in open(path, encoding="utf-8"):
        m = re.match(r"E name=fsync_round target=(\S+) round=(\d+) .* median_us=([\d.]+)", line)
        if m:
            fsync.setdefault(m.group(1), []).append(float(m.group(3)))
        m = re.match(r"E name=copy target=(\S+) bytes=(\d+) median_us=([\d.]+)", line)
        if m:
            copy[m.group(1)] = (int(m.group(2)), float(m.group(3)))
    return fsync, copy


def threshold(merge_rate, fsync_us):
    return int(merge_rate * fsync_us * 1e-6)


def part_one(fsync):
    # 环境：mkfs 那一刻量到的「一次设备同步」
    environments = [
        # SparseBlockDevice::barrier 直接 Ok(())（crates/singlefs-harness/src/memory_pool.rs:209-211）：内存盘上量到 0
        ("harness_sparse_block_device_barrier_noop", 0.0),
        ("tmpfs_dev_shm_median_max", max(fsync["tmpfs_dev_shm"])),
        ("e44_sync_1_2785_per_second", E44_FSYNC_US),
        ("nvme_ext4_today_median_min", min(fsync["nvme_ext4_draft_dir"])),
        ("nvme_ext4_today_median_max", max(fsync["nvme_ext4_draft_dir"])),
    ]
    for env, fsync_us in environments:
        for rate_name, rate in MERGE_RATES.items():
            t_mkfs = threshold(rate, fsync_us)
            print(f"E name=threshold source=mkfs_measured env={env} merge_rate={rate_name} fsync_us={fsync_us:.1f} "
                  f"T_entries={t_mkfs} rings_on_one_entry_publish={'yes' if 1 >= t_mkfs else 'no'}")
    print(f"E name=threshold source=compile_time_constant env=any T_entries={COMPILE_TIME_CONSTANT_T} "
          f"tracks_device=no")
    print("E name=threshold source=runtime_caller_given env=caller_gives_nothing T_entries=undefined "
          "note=default_not_in_Q8_definition mount_option_channel=C422_absent")
    # 同一台机器、同一块盘：编译期常量与今天 mkfs 实测之间，两者判法相反的条目数区间
    t_today = threshold(MERGE_RATES["c58_point"], min(fsync["nvme_ext4_draft_dir"]))
    lo, hi = sorted((COMPILE_TIME_CONSTANT_T, t_today))
    print(f"E name=disagreement same_machine=yes compile_time_T={COMPILE_TIME_CONSTANT_T} mkfs_today_T={t_today} "
          f"entries_judged_differently=[{lo},{hi}) width={hi - lo}")
    # 两次 mkfs 量到的值不逐字相同 ⇒ 写进系统配置的字节不同 ⇒ 起步镜像的 SHA-256 不同（复用键，crash_identity.rs:89）
    medians = fsync["nvme_ext4_draft_dir"]
    distinct = len({round(v, 1) for v in medians})
    print(f"E name=mkfs_reproducibility rounds={len(medians)} distinct_medians_at_0.1us={distinct} "
          f"distinct_T_values={len({threshold(MERGE_RATES['c58_point'], v) for v in medians})}")


def part_two(fsync, copy):
    fsync_us = min(fsync["nvme_ext4_draft_dir"])
    for label, (n_bytes, copy_us) in sorted(copy.items()):
        # 推的下界：一次发布至少要拷一份分配器（transaction.rs:5051），拷贝时长下界取同样字节数的内存拷贝
        cpu_floor_us = copy_us
        ceiling_on_cpu_at_one_entry = cpu_floor_us >= fsync_us
        for source, t in (("compile_time_constant", COMPILE_TIME_CONSTANT_T),
                          ("mkfs_measured_nvme_today", threshold(MERGE_RATES["c58_point"], fsync_us))):
            silent = f"[1,{t})" if ceiling_on_cpu_at_one_entry else "none"
            print(f"E name=silent_band allocator_copy={label} copy_bytes={n_bytes} cpu_floor_us={cpu_floor_us:.1f} "
                  f"fsync_us={fsync_us:.1f} cpu_over_fsync={cpu_floor_us / fsync_us:.3f} source={source} T_entries={t} "
                  f"ceiling_on_cpu_from_entries={'1' if ceiling_on_cpu_at_one_entry else 'not_by_copy_alone'} "
                  f"alarm_silent_entries={silent}")


# ---------- 第三部分：判别力自证抓不抓得到坏实现 ----------
RATE = MERGE_RATES["c58_point"]


class Alarm:
    """一份报警实现：计数器怎么数、交点从哪来。mutant 名字说明它坏在哪。"""

    def __init__(self, name, counter="per_checkpoint", source="measured_on_device", strict=True):
        self.name, self.counter, self.source, self.strict = name, counter, source, strict

    def threshold(self, device_fsync_us, knob):
        if knob is not None:                   # 测试开关：测试给的交点直接用，量的那条路不走
            return knob
        if self.source == "measured_on_device":
            return threshold(RATE, device_fsync_us)
        if self.source == "zero":              # 量在 no-op 屏障上、或微秒当秒换算掉成 0
            return 0
        if self.source == "max":               # 没设（运行时配置缺默认）或读错字段
            return 2 ** 64 - 1
        if self.source == "constant":          # 不看设备，编译期常量
            return COMPILE_TIME_CONSTANT_T
        raise ValueError(self.source)

    def count(self, checkpoint):
        """checkpoint 是这次 checkpoint 里每个用户改动的条目数列表。"""
        if self.counter == "per_checkpoint":
            return sum(checkpoint)
        if self.counter == "stuck_zero":       # 计数器没接到提交路径上
            return 0
        if self.counter == "per_user_change":  # 接在会话那一层：只数最后一个改动
            return checkpoint[-1]
        raise ValueError(self.counter)

    def red(self, checkpoint, device_fsync_us, knob=None):
        # C58 原文「超过交点即判红」按严格大于读（strict=True）；另跑一遍「≥」看比较符号改不改结论
        count, t = self.count(checkpoint), self.threshold(device_fsync_us, knob)
        return count > t if self.strict else count >= t


def mutants(strict):
    return [
        Alarm("correct", strict=strict),
        Alarm("counter_not_wired", counter="stuck_zero", strict=strict),
        Alarm("counter_at_session_per_user_change", counter="per_user_change", strict=strict),
        Alarm("threshold_zero", source="zero", strict=strict),
        Alarm("threshold_max_unset", source="max", strict=strict),
        Alarm("threshold_ignores_device_constant", source="constant", strict=strict),
    ]

SPARSE_FSYNC_US = 0.0       # 内存盘屏障 no-op
INJECTED_LATENCY_US = 100.0  # 收严写法：测试盘在屏障里等一个已知时长（推的写法，零轮）
KNOB = 100


def run_test(test, alarm):
    """交回 True = 这条自证在这份实现上过（绿）。"""
    if test == "c58_literal_production_threshold_on_sparse":
        base = 60
        return alarm.red([2 * base], SPARSE_FSYNC_US)
    if test == "c58_literal_test_knob":
        return alarm.red([2 * 60], SPARSE_FSYNC_US, knob=KNOB)
    if test == "two_sided_test_knob":
        return (not alarm.red([60], SPARSE_FSYNC_US, knob=KNOB)) and alarm.red([120], SPARSE_FSYNC_US, knob=KNOB)
    if test == "two_sided_production_threshold_on_sparse":
        return (not alarm.red([60], SPARSE_FSYNC_US)) and alarm.red([120], SPARSE_FSYNC_US)
    if test == "two_sided_production_threshold_injected_latency":
        t = threshold(RATE, INJECTED_LATENCY_US)
        base = (6 * t) // 10
        return (not alarm.red([base], INJECTED_LATENCY_US)) and alarm.red([2 * base], INJECTED_LATENCY_US)
    if test == "threshold_source_injected_latency":
        lo = MERGE_RATES["e17_low"] * INJECTED_LATENCY_US * 1e-6
        hi = MERGE_RATES["e17_high"] * INJECTED_LATENCY_US * 1e-6
        return lo <= alarm.threshold(INJECTED_LATENCY_US, None) <= hi
    if test == "batching_two_changes_one_checkpoint":
        # 要 D16（发布语义） 已定项 10 的攒批落地才造得出；模型里照样算，报告里标「今天造不出」
        t = threshold(RATE, INJECTED_LATENCY_US)
        base = (6 * t) // 10
        return (not alarm.red([base], INJECTED_LATENCY_US)) and alarm.red([base, base], INJECTED_LATENCY_US)
    raise ValueError(test)


TESTS = [
    "c58_literal_production_threshold_on_sparse",
    "c58_literal_test_knob",
    "two_sided_test_knob",
    "two_sided_production_threshold_on_sparse",
    "two_sided_production_threshold_injected_latency",
    "threshold_source_injected_latency",
    "batching_two_changes_one_checkpoint",
]


def part_three(strict):
    MUTANTS = mutants(strict)
    comparator = "gt" if strict else "ge"
    for test in TESTS:
        green_on_correct = run_test(test, MUTANTS[0])
        caught = [m.name for m in MUTANTS[1:] if green_on_correct and not run_test(test, m)]
        missed = [m.name for m in MUTANTS[1:] if green_on_correct and run_test(test, m)]
        print(f"E name=selfproof comparator={comparator} test={test} green_on_correct={'yes' if green_on_correct else 'no'} "
              f"caught={','.join(caught) or '-'} missed={','.join(missed) or '-'}")
    combos = [
        ("c58_literal_only", ["c58_literal_production_threshold_on_sparse"]),
        ("c58_literal_with_knob", ["c58_literal_test_knob"]),
        ("two_sided_knob", ["two_sided_test_knob"]),
        ("two_sided_latency_plus_source", ["two_sided_production_threshold_injected_latency",
                                           "threshold_source_injected_latency"]),
        ("two_sided_latency_plus_source_plus_batching", ["two_sided_production_threshold_injected_latency",
                                                         "threshold_source_injected_latency",
                                                         "batching_two_changes_one_checkpoint"]),
    ]
    for name, tests in combos:
        usable = [t for t in tests if run_test(t, MUTANTS[0])]
        missed = [m.name for m in MUTANTS[1:] if all(run_test(t, m) for t in usable)]
        print(f"E name=selfproof_combo comparator={comparator} combo={name} tests_green_on_correct={len(usable)}/{len(tests)} "
              f"mutants={len(MUTANTS) - 1} missed={len(missed)} missed_list={','.join(missed) or '-'}")


def main():
    fsync, copy = probe_values(sys.argv[1])
    part_one(fsync)
    part_two(fsync, copy)
    part_three(strict=True)
    part_three(strict=False)


if __name__ == "__main__":
    main()
