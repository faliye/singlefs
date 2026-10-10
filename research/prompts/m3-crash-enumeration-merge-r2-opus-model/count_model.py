#!/usr/bin/env python3
"""L2 计数模型（攻方腿，m3-crash-enumeration-merge-r2）：G5、G9 的跑法下，一次改到 core 的提交 GPU 那一截要多久、CPU 那一截跟上要多久。

纯算术，不读仓、不起进程。每个输入写明出处；「量过」= 记录里原样的数（那一天的树、那一天的内核，不是今天的树）；
「推的」= 按代码或别的流外推、没在今天的树上量过。跑法：python3 count_model.py；末尾自检钉住绝对值，对不上退 1。

与第一轮攻方模型（research/prompts/m3-crash-enumeration-merge-r1-opus-model/count_model.py）的差别：
1. 第一轮拿多记录流那一项的 GPU 速率（339 792 态/秒）乘全域；这里按项取各自量过的整项挂钟（含录制、建表、两台、导入、存判定）。
2. 第一轮 CPU 两条速率（层 0 路 48 500、放量路 8 428）量在不同的流、流里不同的位置：层 0 那条是第二条流开头的短跑
   （records/2026-09-24-里程碑二收尾调度.md 第 318 行「第二条流速率（量过）」），放量那条是固定脚本到 D、24 写段占多数
   （records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md 第 55 行「前 9 分钟小段 17.5 k，24 写段掉到约 2 k」）。差的主要是流与段，不是路。
3. G4 下删除流的逐状态可写挂载罩整条流（含与多记录流共享的前缀），算进 CPU 那一截。
4. 多记录流与删除流共享多少分两种取：「records 的推算」1 358 954 604（records/2026-10-01 第 1286 行，按段形推）；
   「今天的代码」16 777 259：B、C 两步各在自己的测试文件里设点（recorder.step(file!(), "sequential_write", …)），
   身份带设点的文件（crash_identity.rs 第 68–74 行），块键不同，只有公共模块 common_crash_points 设的取号暖机与 A 两个点是同一个节点；
   2026-10-08 多记录流那一项 reused=16777259 恰是这一段（records/2026-10-01 第 1292 行）。
"""
import sys

# ---- 各项态数（今天的树上用例里钉的闭式）----
SECOND_STREAM_TOTAL = 1_678_377_204       # crash_enumeration_fixed_script_stream.rs:103
THROUGH_ROLLBACK = 67_502_198             # crash_enumeration_fixed_script_stream.rs:723
MULTI_RECORD = 1_358_954_634              # crash_enumeration_multi_record_publish_stream.rs:395
UNLINK = 1_463_812_278                    # crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:1028
UNLINK_SHARED_PREFIX_RECORDS = 1_358_954_604  # records/2026-10-01-删除与并发事务.md:1286（按段形推的共享）
UNLINK_SHARED_PREFIX_CODE = 16_777_259        # 今天的代码：只共享 common_crash_points 设的两个点（= 10-08 多记录流 reused）
TREE_SPLIT = 78_905_428                   # stage-inputs.tsv crash-case:layer0-tree-split-streams 注释（事实，行号不钉）
POSITION_ADDRESSED_UPPER = 553_664_602    # records 2026-09-28 第 35 行：展开 28 那一趟（上界，推的）
FIRST_STREAM = 16_777_260                 # crash_enumeration_new_pool_file_creation_stream.rs:571

# ---- GPU：量过的整项挂钟（两台五张卡，records 原样）----
GPU_ITEM_WALL = {
    # 项: (秒, 那一趟现判的态, 出处)
    "fixed-script-through-rollback": (866, 51_773_558, "records/2026-09-28 第 35 行「到 D 51 773 558 态 0 红 866 秒」"),
    "fixed-script-through-unmount": (16_271, 1_610_875_015, "同一行「到 E 1 610 875 015 态 … 16 271 秒」"),
    "multi-record-publish": (3_950, 1_342_177_375, "records/2026-10-01 第 1292 行 multi-record-publish seconds=3950"),
    "tree-split-streams": (3_538, TREE_SPLIT, "records/2026-09-28 第 35 行「树分裂 78 905 428 态 3 538 秒」"),
    "position-addressed-trees": (4_780, POSITION_ADDRESSED_UPPER, "同一行「按位置寻址 553 664 602 态 4 780 秒」"),
    "floor-raise-pushed-by-the-session": (579, 8_223, "同一行「抬 F 8 223 态 579 秒」"),
    "sigma-of-the-misaligned-reuse": (607, 65_548, "同一行「σ 65 548 态 0 红 607 秒」"),
}
UNLINK_NO_REUSE_WALL = (4_680, 1_448_083_638)  # records/2026-10-01 第 1292 行 unlink … reused=0 seconds=4680（那一趟判法版本中途换了，前缀没复用）
# 每项固定开销：抬 F 与 σ 两项态数可忽略，挂钟 579 / 607 秒（推的：新加的项照这个数）
FIXED_OVERHEAD_SECONDS = (579, 607)
NEW_ITEMS_ON_GPU = ["first-stream", "io-min-8192", "instance-table-sweep-mount", "instance-table-sweep-phases-remount",
                    "crash-injection-fast-tier", "read-faulted-witness"]  # 今天没上过 GPU 的六项（G5 之后要上），态数都小（推的）
UNMOUNT_RATE_RANGE = (1_610_875_015 / 16_271, 130_000)  # 到 E 那一项 9.9 万；records 同一行「低于 2026-09-29 那一趟约 13 万」

# ---- CPU：量过的速率（两台合计，31 线程）----
CPU_RATE = {
    "amplification-path-24-write-segments": (4_108 + 4_320, "records/2026-09-28 第 55 行 states_per_second=4108、4320（到 D 全域）"),
    "layer0-path-second-stream-short-run": (23_700 + 24_800, "records/2026-09-24 第 318 行「本机 23,700 个/秒、第二台 24,800 个/秒」"),
}
MOUNT_RATE_TWO_HOSTS = 2 * 4_000  # records/2026-10-01 第 1286 行「每台约 4 千态/秒」（那一句出处不明，推的）


def hours(seconds):
    return seconds / 3600


def scenario(shared):
    """一种共享取法下的四个数：GPU 串跑整项挂钟、GPU 只算判的下界、CPU 跟上的两端。"""
    unlink_new = UNLINK - shared
    unlink_rate = UNLINK_NO_REUSE_WALL[1] / UNLINK_NO_REUSE_WALL[0]
    unlink_wall = min(UNLINK_NO_REUSE_WALL[0], FIXED_OVERHEAD_SECONDS[1] + unlink_new / unlink_rate)
    measured = sum(seconds for seconds, _, _ in GPU_ITEM_WALL.values())
    new_items = len(NEW_ITEMS_ON_GPU) * FIXED_OVERHEAD_SECONDS[1] + FIRST_STREAM / (1_342_177_375 / 3_950)
    serial = measured + unlink_wall + new_items
    unmount_new = SECOND_STREAM_TOTAL - THROUGH_ROLLBACK
    rollback = THROUGH_ROLLBACK / (51_773_558 / 866)
    floor = [unmount_new / rate + 3_950 + unlink_new / unlink_rate + rollback for rate in (UNMOUNT_RATE_RANGE[1], UNMOUNT_RATE_RANGE[0])]
    rates = sorted(rate for rate, _ in CPU_RATE.values())
    judged_by_cpu = SECOND_STREAM_TOTAL + POSITION_ADDRESSED_UPPER + TREE_SPLIT + MULTI_RECORD + unlink_new
    def cpu_hours(rate):
        # 每个状态 CPU 判一张镜像（G6）；删除流整条（含共享那一截）每个状态加一次可写挂载（G4）
        return hours(judged_by_cpu / rate + FIRST_STREAM / (27_100 + 23_700) + UNLINK / MOUNT_RATE_TWO_HOSTS)
    distinct = SECOND_STREAM_TOTAL + MULTI_RECORD + unlink_new + TREE_SPLIT + POSITION_ADDRESSED_UPPER + FIRST_STREAM + 8_223 + 65_548 + 33  # 末三项：抬 F、σ、io_min 8192
    return distinct, hours(serial), [hours(x) for x in floor], (cpu_hours(rates[1]), cpu_hours(rates[0])), unlink_wall


def main():
    results = {}
    for name, shared in (("records_assumption", UNLINK_SHARED_PREFIX_RECORDS), ("code_today", UNLINK_SHARED_PREFIX_CODE)):
        distinct, serial, floor, cpu, unlink_wall = scenario(shared)
        results[name] = (distinct, serial, floor, cpu)
        print(f"MODEL sharing={name} unlink_shared={shared} distinct_states={distinct} unlink_item_wall={unlink_wall:.0f}s "
              f"gpu_serial_item_walls_hours={serial:.1f} gpu_judging_only_floor_hours={floor[0]:.1f}..{floor[1]:.1f} "
              f"cpu_catch_up_hours={cpu[0]:.0f}..{cpu[1]:.0f}")
    rates = sorted(rate for rate, _ in CPU_RATE.values())
    print(f"MODEL cpu_second_stream_alone_hours={hours(SECOND_STREAM_TOTAL / rates[1]):.1f}..{hours(SECOND_STREAM_TOTAL / rates[0]):.1f}")
    print(f"MODEL cpu_unlink_mounts_alone_hours={hours(UNLINK / MOUNT_RATE_TWO_HOSTS):.1f}")
    print("MODEL r1_reference distinct_states=3774833347 gpu_only_hours=1.5..3.1 cpu_every_state_hours=21.6..124.4")
    # G4/G8 读成「每项各自的观察者 / 判法版本」、而 B、C 照 G3 挪进公共模块共享时，共享前缀又断开：多判的态
    lost = UNLINK_SHARED_PREFIX_RECORDS + THROUGH_ROLLBACK
    print(f"MODEL sharing_lost_if_versions_are_per_item states={lost} gpu_extra_hours={hours(lost / (1_342_177_375 / 3_950)):.2f} "
          f"cpu_extra_hours={hours(lost / rates[1]):.1f}..{hours(lost / rates[0]):.1f}")
    # 改删除流测试文件里的观察者（G4 观察者进计划哈希）：GPU 判镜像那一截照样重判它挂着的节点
    print(f"MODEL observer_edit_gpu_rejudge_hours={hours(UNLINK_NO_REUSE_WALL[0]):.2f}..{hours(3_950 + UNLINK_NO_REUSE_WALL[0]):.2f} (only the unlink nodes .. unlink plus the shared multi-record nodes)")
    records, code = results["records_assumption"], results["code_today"]
    checks = [
        (records[0], 3_791_610_606),
        (code[0], 5_133_787_951),
        (round(records[1], 1), 9.8),
        (round(code[1], 1), 10.8),
        (round(code[2][0], 1), 6.2),
        (round(code[2][1], 1), 7.2),
        (round(records[3][0]), 73),
        (round(records[3][1]), 175),
        (round(code[3][0]), 80),
        (round(code[3][1]), 220),
        (lost, 1_426_456_802),
    ]
    bad = [(i, got, want) for i, (got, want) in enumerate(checks) if got != want]
    print(f"MODEL_SELFCHECK compared={len(checks)} mismatched={len(bad)} {bad}")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
