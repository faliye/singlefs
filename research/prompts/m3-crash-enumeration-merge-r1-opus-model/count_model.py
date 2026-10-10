#!/usr/bin/env python3
"""K5 计数模型（攻方腿，m3-crash-enumeration-merge-r1）：合成之后提交跑整域的挂钟。纯算术，不读仓、不起进程。

输入分两类，各自写明出处（文件:行，或 records 的行）；「推的」= 按代码推、没在今天的树上跑过。
跑法：python3 count_model.py；末尾自检把几个绝对值钉住，对不上退 1。
"""
import sys

# ---- 各项的态数（今天的树上用例里钉的闭式；放量的数取它跑出来的那一趟）----
STATES = {
    # 名字: (整条流的态数, 与别的项共享、在同一判法版本下只判一次的前缀态数, 出处)
    "second-stream-through-unmount": (1_678_377_204, 0, "fixed_script_stream.rs:103"),
    "through-rollback": (67_502_198, 67_502_198, "fixed_script_stream.rs:723；前 36 点与到 E 共享（records 2026-09-28 第 35 行「到 D 那段沿用」）"),
    "first-stream-own-item": (16_777_260, 0, "new_pool_file_creation_stream.rs:571；被判根是 A 的根，计划哈希不同、不复用（crash_amplification.rs:9-10）"),
    "multi-record": (1_358_954_634, 16_777_259, "multi_record_publish_stream.rs:395；共享前缀取 2026-10-08 那一趟 reused=16777259"),
    "unlink": (1_463_812_278, 1_358_954_604, "unlink_after_the_multi_record_publish_stream.rs:1028；共享前缀取 records 2026-10-01 第 1286 行"),
    "tree-split": (78_905_428, 0, "tree_split_streams.rs:416"),
    "position-addressed(<=)": (553_664_602, 0, "records 2026-09-28 第 35 行（展开 28、五条流，上界）"),
    "floor-raise": (8_223, 0, "records 2026-09-28 第 35 行"),
    "sigma": (65_548, 0, "records 2026-09-28 第 35 行"),
    "io-min-8192": (33, 0, "io_min_8192_pool.rs FULL_STATES"),
}

# ---- 速率（两台合计，态/秒；都是 records 里量过的数，取的是哪一趟写在旁边）----
RATES = {
    "gpu-effective": (1_342_177_375 / 3950, "2026-10-08 多记录流展开 30 那一项 states/seconds（含存判定、两台不均）"),
    "gpu-balanced": (338_821 + 380_989, "同一趟两半各自的态/秒相加（records 2026-10-01 第 1288 行）"),
    "cpu-amplification-path": (4_108 + 4_320, "records 2026-09-28 第 55 行：到 D 全域两台 31 线程 states_per_second"),
    "cpu-layer0-path": (23_700 + 24_800, "records 2026-09-24 第 318 行：第二条流本机与第二台"),
}

# 录制（剪枝也省不掉）：树分裂 7 条、按位置寻址 5 条每条约 480–520 秒、单线程（records 2026-09-28 第 35 行）；其余各项没有数。
RECORDING_SECONDS_PER_SLOW_STREAM = (480, 520)
SLOW_STREAMS = 7 + 5


def distinct_states():
    return sum(total - shared for total, shared, _ in STATES.values())


def hours(seconds):
    return seconds / 3600


def main():
    total = distinct_states()
    print(f"MODEL distinct_states_new_judge_version={total}")
    for name, (states, shared, source) in STATES.items():
        print(f"MODEL item={name} states={states} shared_prefix={shared} new={states - shared} source={source}")
    for name, (rate, source) in RATES.items():
        print(f"MODEL rate={name} states_per_second={rate:.0f} hours_for_the_domain={hours(total / rate):.1f} source={source}")
    gpu = [RATES["gpu-balanced"][0], RATES["gpu-effective"][0]]
    cpu = [RATES["cpu-layer0-path"][0], RATES["cpu-amplification-path"][0]]
    print(f"MODEL core_change_gpu_only_hours={hours(total / gpu[0]):.1f}..{hours(total / gpu[1]):.1f}")
    print(f"MODEL core_change_F4_cpu_every_state_hours={hours(total / cpu[0]):.1f}..{hours(total / cpu[1]):.1f}")
    # 删除流的逐状态可写挂载若照层 0 今天那样罩整条流（unlink_after…rs:1075 断言挂载数 = closed_form），共享前缀那一截要再判一遍挂载
    unlink_total, unlink_shared, _ = STATES["unlink"]
    print(f"MODEL unlink_states_whose_mount_observer_is_lost_under_prefix_sharing={unlink_shared} of {unlink_total} ({unlink_shared / unlink_total:.1%})")
    low, high = RECORDING_SECONDS_PER_SLOW_STREAM
    print(f"MODEL pruned_commit_recording_floor_hours={hours(low * SLOW_STREAMS):.2f}..{hours(high * SLOW_STREAMS):.2f} (单线程串行录这 12 条，推的)")
    checks = [
        (total, 3_774_833_347),
        (round(hours(total / cpu[0]), 1), 21.6),
        (round(hours(total / cpu[1]), 1), 124.4),
        (round(hours(total / gpu[1]), 1), 3.1),
    ]
    bad = [(got, want) for got, want in checks if got != want]
    print(f"MODEL_SELFCHECK compared={len(checks)} mismatched={len(bad)} {bad}")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
