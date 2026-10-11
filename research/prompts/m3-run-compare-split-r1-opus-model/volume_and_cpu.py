#!/usr/bin/env python3
"""攻方腿模型（m3-run-compare-split-r1）：第 4 问体量、第 5 问 CPU 代价的算术。纯算术，不读库、不跑实现。
每个输入数在 INPUTS 里带出处；推的写 推的。输出逐行 key=value，末行 MODEL_DONE。
"""
import math

INPUTS = {
    # 第四节第 4 问正文与 CLAUDE.md：展开 28 全枚举七个分项 26.16 亿态
    "states_full_body": 2_616_000_000,
    # ~/.local/share/singlefs/crash-amplification/full-run-20260930-i.log 末段 CRASH_AMPLIFICATION_TOTAL 行 states=
    "states_full_run_0930": 2_563_727_926,
    # 同一行 seconds=（GPU 判、两台 5 张卡的那一趟全量挂钟）
    "gpu_full_run_seconds_0930": 27_288,
    # 事实表第 3.1 节：01 一遍约 72 字节，两遍约 144，取整 160（推的，按代码数字段）
    "record_01_bytes_per_state_two_passes": 144,
    "record_01_bytes_rounded": 160,
    # crates/singlefs-checker-tier/src/crash_amplification.rs STATES_PER_BLOCK
    "states_per_block": 65_536,
    # 这一轮现查 df -h（2026-10-10）两次：先 514G、后 1.8T（同一天在变；正文写 569G）。两个都算
    "disk_free_gib_now": 514,
    "disk_free_gib_later": 1843,
    # 库 LOG 末次「Cumulative writes … ingest: 17.56 GB」与「Sum … 186.79 MB」：结论表（0A/0C/0D…）LZ4 之后的压缩比
    "observed_ingest_gb": 17.56,
    "observed_on_disk_mb": 186.79,
    # 同一 LOG 的 Sum 行 W-Amp
    "observed_write_amplification": 11.3,
    # research/results/e161-…-r2-threads1-2026-10-05.out 的 g3 五格：(恢复看 journal + 不看 journal) 纳秒 / 状态数（旧树、单线程、E142 那两条流；参考，不是今天的数）
    "e161_recovery_two_pass_ns": {
        "first_small": (9_692_576 + 6_388_310) / 45,
        "first_segment_head": (384_412_448 + 269_239_806) / 4096,
        "first_quick": (9_504_798 + 6_435_474) / 46,
        "second_segment_head": (1_365_494_020 + 1_113_282_804) / 4096,
        "second_quick": (21_311_184 + 16_263_783) / 52,
    },
    # full-run 日志 threads=31；host-resources 报第二台 32 核（这一轮现查）
    "threads_local": 31,
    "threads_two_hosts": 62,
    # full-run-20260930-i.log 删除线那一项两台的 gpu_states_per_second（本机 2 卡、第二台 3 卡）
    "gpu_states_per_second_local": 72_429,
    "gpu_states_per_second_peer": 44_471,
    # 同一份日志 fixed-script-through-rollback 那一项本机一台 2 卡的 gpu_states_per_second（小镜像那一类项）
    "gpu_states_per_second_fixed_script_local": 780_738,
}


def gib(byte_count):
    return byte_count / 2**30


def main():
    i = INPUTS
    for label in ("states_full_body", "states_full_run_0930"):
        states = i[label]
        raw = states * i["record_01_bytes_per_state_two_passes"]
        rounded = states * i["record_01_bytes_rounded"]
        blocks = math.ceil(states / i["states_per_block"])
        print(f"VOLUME_RAW basis={label} states={states} blocks={blocks} raw_144_gib={gib(raw):.1f} raw_160_gib={gib(rounded):.1f} "
              f"fits_once_in_free={gib(rounded) < i['disk_free_gib_now']}")
        observed_ratio = i["observed_ingest_gb"] * 1000 / i["observed_on_disk_mb"]
        for ratio in (2, 5, 20, observed_ratio):
            print(f"VOLUME_LZ4 basis={label} assumed_ratio={ratio:.1f} on_disk_gib={gib(rounded) / ratio:.1f}")
        # 字典形态（推的）：每块一张去重表（每块至多 256 种 72 字节记录），每状态每遍 1 字节下标
        dictionary = states * 2 + blocks * 256 * 72
        print(f"VOLUME_DICTIONARY basis={label} on_disk_gib_before_lz4={gib(dictionary):.2f}")
        # 写盘总量：WAL 不压缩，按原始入库字节记一遍；压缩后的字节再乘 W-Amp
        for ratio in (5, observed_ratio):
            written = rounded + rounded / ratio * i["observed_write_amplification"]
            print(f"WRITE_VOLUME basis={label} assumed_ratio={ratio:.1f} wal_plus_compaction_gib={gib(written):.0f}")
        # 双机：crash-amplification-two-hosts.sh 每跑一项 cp -a 整份本机库给第二台（library-for-peer），再把第二台整份库取回（fetched），都留在 runs/ 下
        for ratio in (5, 20, observed_ratio):
            library = gib(rounded) / ratio
            per_item_run = 2 * library
            runs_before_full = i["disk_free_gib_now"] / per_item_run
            runs_before_full_later = i["disk_free_gib_later"] / per_item_run
            print(f"TWO_HOST_COPIES basis={label} assumed_ratio={ratio:.1f} library_gib={library:.1f} copies_per_item_run_gib={per_item_run:.1f} "
                  f"item_runs_until_disk_full={runs_before_full:.1f} item_runs_until_disk_full_at_1843_gib={runs_before_full_later:.1f}")
        print(f"TWO_HOST_COPIES_UNCOMPRESSED basis={label} copies_per_item_run_gib={2 * gib(rounded):.1f} "
              f"item_runs_until_disk_full={i['disk_free_gib_now'] / (2 * gib(rounded)):.2f} item_runs_until_disk_full_at_1843_gib={i['disk_free_gib_later'] / (2 * gib(rounded)):.2f}")
    states = i["states_full_body"]
    gpu_rate = i["gpu_states_per_second_local"] + i["gpu_states_per_second_peer"]
    needed_ns = i["threads_two_hosts"] / gpu_rate * 1e9
    print(f"CPU_KEEP_UP gpu_states_per_second_two_hosts={gpu_rate} per_thread_budget_ns_to_keep_up={needed_ns:.0f}")
    for cell, nanoseconds in i["e161_recovery_two_pass_ns"].items():
        thread_seconds = states * nanoseconds / 1e9
        print(f"CPU_RECORD cell={cell} ns_per_state={nanoseconds:.0f} thread_hours={thread_seconds / 3600:.0f} "
              f"hours_on_31={thread_seconds / i['threads_local'] / 3600:.2f} hours_on_62={thread_seconds / i['threads_two_hosts'] / 3600:.2f} "
              f"vs_gpu_full_run={thread_seconds / i['threads_two_hosts'] / i['gpu_full_run_seconds_0930']:.2f} "
              f"keeps_up_with_gpu={nanoseconds <= needed_ns}")
    small_item_budget = i["threads_local"] / i["gpu_states_per_second_fixed_script_local"] * 1e9
    print(f"CPU_KEEP_UP_SMALL_IMAGE_ITEM gpu_states_per_second_local={i['gpu_states_per_second_fixed_script_local']} per_thread_budget_ns_to_keep_up={small_item_budget:.0f}")
    print("MODEL_DONE")


if __name__ == "__main__":
    main()
