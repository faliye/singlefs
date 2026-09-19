#!/usr/bin/env python3
"""E152（按里程碑对比六家文件系统的文件性能）：把产物里的 name=summary 行渲染成 research/perf-by-milestone.md 用的表。

  e152-tables.py <产物>        打印 markdown 表（每个维度一张，行是配置，格子是 5 轮中位）
  e152-tables.py --selftest    拿一份造出来的汇总行核渲染结果

表里的数一律取产物的 name=summary 行，不手抄（.claude/singlefs-ai-sop/rules/evidence-discipline.md
「能被一条命令数出来的数，就该由一条命令来数」）。离散超过 15% 的格子带 ⚠ 与离散百分比，照报不删（跑前登记第七节）。
自证会红：E152_TABLES_HIDE_UNSTABLE=1 强制走回「不稳定的格子不标」的错法，--selftest 必须判红。
"""
import os
import statistics
import sys

CONFIGURATION_ORDER = ["raw", "ext4", "xfs", "f2fs", "btrfs", "bcachefs", "zfs",
                       "raw-md", "ext4-md", "xfs-md", "f2fs-md", "btrfs-raid1", "bcachefs-replicas2", "zfs-mirror", "singlefs"]
CONFIGURATION_LABELS = {
    "raw": "裸盘（天花板）",
    "ext4": "ext4",
    "xfs": "XFS",
    "f2fs": "F2FS",
    "btrfs": "Btrfs",
    "bcachefs": "Bcachefs",
    "zfs": "OpenZFS（vdev 是分区）",
    "raw-md": "裸 md raid1（镜像天花板）",
    "ext4-md": "ext4（md raid1）",
    "xfs-md": "XFS（md raid1）",
    "f2fs-md": "F2FS（md raid1）",
    "btrfs-raid1": "Btrfs raid1",
    "bcachefs-replicas2": "Bcachefs 双副本",
    "zfs-mirror": "OpenZFS mirror（vdev 是分区）",
    "singlefs": "singlefs（两盘）",
}
GROUPS = [
    ("大文件顺序写与顺序读（MiB/s，越大越好）", [
        ("write_sequential_mebibytes_per_second", "顺序写"),
        ("read_sequential_mebibytes_per_second", "顺序读"),
    ]),
    ("4K 随机读写（次/秒越大越好；p99 是完成延迟，µs，越小越好）", [
        ("read_random_4k_operations_per_second", "随机读 次/秒"),
        ("read_random_4k_percentile_99_microseconds", "随机读 p99"),
        ("write_random_4k_operations_per_second", "随机写 次/秒"),
        ("write_random_4k_percentile_99_microseconds", "随机写 p99"),
    ]),
    ("元数据 / 小文件（个/秒，越大越好）", [
        ("metadata_create_per_second", "建空文件"),
        ("metadata_stat_per_second", "stat"),
        ("metadata_delete_per_second", "删文件"),
        ("small_file_write_fsync_per_second", "4 KiB 写 + fsync"),
    ]),
    ("小文件持久化：新建 + 写 3000 字节 + fsync 文件 + fsync 目录（刚格式化的空文件系统上 100 次）", [
        ("persist_median_microseconds", "延迟中位 µs"),
        ("persist_percentile_99_microseconds", "延迟 p99 µs"),
        ("persist_mean_write_requests", "每次写请求"),
        ("persist_mean_write_bytes", "每次写字节"),
        ("persist_mean_flush_requests", "每次 FLUSH"),
        ("persist_first_write_bytes", "第一次写字节"),
        ("persist_first_write_requests", "第一次写请求"),
        ("persist_first_flush_requests", "第一次 FLUSH"),
    ]),
    ("冷挂载（卸载、清缓存之后再挂）", [
        ("cold_mount_milliseconds", "挂钟 ms"),
        ("cold_mount_read_bytes", "读字节"),
        ("cold_mount_read_requests", "读请求"),
    ]),
    ("格式化与可用容量（16 GiB 盘）", [
        ("format_milliseconds", "格式化 ms"),
        ("format_write_bytes", "格式化写字节"),
        ("format_flush_requests", "格式化 FLUSH"),
        ("capacity_total_percent", "总容量 %"),
        ("capacity_available_percent", "可用 %"),
    ]),
    ("singlefs（两块 16 GiB 盘，门禁 55 号的真设备二进制）", [
        ("singlefs_write_path_milliseconds", "mkfs + 取号 + 暖机 + 第一个事务 ms"),
        ("singlefs_second_transaction_milliseconds", "第二个事务（覆盖写 B）ms"),
        ("singlefs_second_transaction_inner_milliseconds", "发布 B（二进制自己计）ms"),
        ("singlefs_recovery_milliseconds", "冷恢复 ms"),
        ("singlefs_read_bytes_both_devices", "两盘读字节"),
        ("singlefs_read_requests_both_devices", "两盘读请求"),
        ("singlefs_second_transaction_writes_both_devices", "B 两盘写请求"),
        ("singlefs_second_transaction_written_bytes_both_devices", "B 两盘写字节"),
        ("singlefs_second_transaction_barriers_both_devices", "B 两盘屏障"),
        ("singlefs_second_transaction_force_unit_access_writes_both_devices", "B 两盘 FUA 写"),
    ]),
]


# 四个数据作业的「块层字节 ÷ fio 字节」：跑前没登记成指标，是从产物里每个作业前后的块层读数算的
AMPLIFICATION_JOBS = [
    ("write_sequential_large", "write", "顺序写"),
    ("read_sequential_large", "read", "顺序读"),
    ("read_random_4k", "read", "4K 随机读"),
    ("write_random_4k", "write", "4K 随机写"),
]


def read_amplification(text):
    """每个配置每个作业的放大倍数，取成功的那几次虚机跑（vm_exit=0）的中位。"""
    ratios = {}
    configuration, usable = None, False
    for line in text.splitlines():
        if line.startswith("E152RUN "):
            fields = parse_fields(line)
            configuration, usable = fields.get("configuration"), fields.get("vm_exit") == "0"
            continue
        if not usable or not line.startswith("E7RESULT name=fio "):
            continue
        fields = parse_fields(line)
        for job, side, _ in AMPLIFICATION_JOBS:
            if fields.get("job") == job:
                fio_bytes = int(fields[f"{side}_kibibytes"]) * 1024
                if fio_bytes > 0:
                    ratios.setdefault((configuration, job), []).append(int(fields[f"block_{side}_bytes"]) / fio_bytes)
    return {key: statistics.median(values) for key, values in ratios.items()}


def parse_fields(line):
    return dict(part.split("=", 1) for part in line.split()[1:] if "=" in part)


def read_summaries(text):
    summaries, excluded, failed, controls = {}, [], [], {}
    for line in text.splitlines():
        if not line.startswith("E7RESULT name=summary"):
            continue
        fields = parse_fields(line)
        name = fields.get("name")
        if name == "summary":
            summaries[(fields["configuration"], fields["metric"])] = fields
        elif name == "summary_excluded":
            excluded.append(fields)
        elif name == "summary_failed_round":
            failed.append(fields)
        elif name == "summary_control":
            controls[fields["configuration"]] = fields
    return summaries, excluded, failed, controls


def format_number(value):
    if value >= 100:
        return f"{value:,.0f}".replace(",", " ")
    if value >= 10:
        return f"{value:.1f}"
    return f"{value:.2f}"


def format_cell(fields):
    if fields is None:
        return "—"
    cell = format_number(float(fields["median"]))
    if fields["stability"] != "stable" and os.environ.get("E152_TABLES_HIDE_UNSTABLE") != "1":
        cell += f" ⚠{float(fields['spread_percent']):.0f}%"
    if fields["rounds"] != "5":
        cell += f"（{fields['rounds']} 轮）"
    return cell


def render(text):
    summaries, excluded, failed, controls = read_summaries(text)
    output = []
    for title, metrics in GROUPS:
        rows = [configuration for configuration in CONFIGURATION_ORDER
                if any((configuration, metric) in summaries for metric, _ in metrics)]
        if not rows:
            continue
        output.append(f"#### {title}\n")
        output.append("| 配置 | " + " | ".join(label for _, label in metrics) + " |")
        output.append("|---|" + "---|" * len(metrics))
        for configuration in rows:
            cells = [format_cell(summaries.get((configuration, metric))) for metric, _ in metrics]
            output.append(f"| {CONFIGURATION_LABELS[configuration]} | " + " | ".join(cells) + " |")
        output.append("")
    amplification = read_amplification(text)
    if amplification:
        output.append("#### 块层字节 ÷ fio 字节（5 轮中位；跑前没登记成指标，是从产物里每个作业前后的块层读数算的）\n")
        output.append("| 配置 | " + " | ".join(label for _, _, label in AMPLIFICATION_JOBS) + " |")
        output.append("|---|" + "---|" * len(AMPLIFICATION_JOBS))
        for configuration in CONFIGURATION_ORDER:
            if not any((configuration, job) in amplification for job, _, _ in AMPLIFICATION_JOBS):
                continue
            cells = [f"{amplification[(configuration, job)]:.2f}" if (configuration, job) in amplification else "—"
                     for job, _, _ in AMPLIFICATION_JOBS]
            output.append(f"| {CONFIGURATION_LABELS[configuration]} | " + " | ".join(cells) + " |")
        output.append("")
    output.append("#### 对照、排除与失败的轮\n")
    output.append("| 配置 | 对照有牙的轮 / 查过的轮 | 被排除的格子 | 两次都没跑成的轮 |")
    output.append("|---|---|---|---|")
    for configuration in CONFIGURATION_ORDER:
        control = controls.get(configuration)
        excluded_here = [f"{entry['metric']} 第 {entry['round']} 轮（{entry['reason']}）"
                         for entry in excluded if entry["configuration"] == configuration]
        failed_here = [entry["round"] for entry in failed if entry["configuration"] == configuration]
        if control is None and not excluded_here and not failed_here and not any(
                key[0] == configuration for key in summaries):
            continue
        control_cell = f"{control['rounds_with_teeth']} / {control['rounds_checked']}" if control else "—（没有对照）"
        output.append(f"| {CONFIGURATION_LABELS[configuration]} | {control_cell} | "
                      f"{'；'.join(excluded_here) or '无'} | {'、'.join(failed_here) or '无'} |")
    return "\n".join(output) + "\n"


def selftest():
    sample = "\n".join([
        "E7RESULT name=summary configuration=ext4 metric=write_sequential_mebibytes_per_second rounds=5 median=1536.000 minimum=1024.000 maximum=2048.000 spread_percent=66.7 stability=unstable values=1:1024.000",
        "E7RESULT name=summary configuration=zfs metric=write_sequential_mebibytes_per_second rounds=4 median=12.345 minimum=12.000 maximum=12.500 spread_percent=4.1 stability=stable values=1:12.000",
        "E7RESULT name=summary_control configuration=ext4 rounds_checked=5 rounds_with_teeth=5",
        "E7RESULT name=summary_excluded configuration=zfs metric=read_sequential_mebibytes_per_second round=3 reason=cache_substituted",
        "E7RESULT name=summary_failed_round configuration=zfs round=2 attempts=2",
        "E152RUN configuration=zfs round=1 attempt=1 host_load1=1.0 vm_exit=0",
        "E7RESULT name=fio configuration=zfs round=1 job=read_random_4k read_kibibytes=4 block_read_bytes=131072 write_kibibytes=0 block_write_bytes=0 verdict=counted",
        "E152RUN configuration=zfs round=2 attempt=1 host_load1=1.0 vm_exit=1",
        "E7RESULT name=fio configuration=zfs round=2 job=read_random_4k read_kibibytes=4 block_read_bytes=4096 write_kibibytes=0 block_write_bytes=0 verdict=counted",
    ])
    singlefs_sample = "\n".join([
        "E7RESULT name=summary configuration=singlefs metric=singlefs_second_transaction_milliseconds rounds=3 median=14.000 minimum=12.000 maximum=16.000 spread_percent=28.6 stability=unstable values=1:16.000,3:12.000,5:14.000",
        "E7RESULT name=summary configuration=singlefs metric=singlefs_second_transaction_inner_milliseconds rounds=5 median=6.412 minimum=6.000 maximum=6.500 spread_percent=7.8 stability=stable values=1:6.000",
        "E7RESULT name=summary_excluded configuration=singlefs metric=singlefs_second_transaction_milliseconds round=2 reason=outer_does_not_contain_inner",
    ])
    # 第一次、第二次正式跑的产物里没有二进制自己计的那个指标：那一格照缺值写「—」
    singlefs_without_inner_sample = singlefs_sample.splitlines()[0]
    rendered = render(sample) + render(singlefs_sample) + render(singlefs_without_inner_sample)
    expectations = [
        "| ext4 | 1 536 ⚠67% | — |",
        "| OpenZFS（vdev 是分区） | 12.3（4 轮） | — |",
        "| ext4 | 5 / 5 | 无 | 无 |",
        "| OpenZFS（vdev 是分区） | —（没有对照） | read_sequential_mebibytes_per_second 第 3 轮（cache_substituted） | 2 |",
        "| OpenZFS（vdev 是分区） | — | — | 32.00 | — |",
        "| 第二个事务（覆盖写 B）ms | 发布 B（二进制自己计）ms | 冷恢复 ms |",
        "| singlefs（两盘） | — | 14.0 ⚠29%（3 轮） | 6.41 | — | — | — | — | — | — | — |",
        "| singlefs（两盘） | — | 14.0 ⚠29%（3 轮） | — | — | — | — | — | — | — | — |",
        "| singlefs（两盘） | —（没有对照） | singlefs_second_transaction_milliseconds 第 2 轮（outer_does_not_contain_inner） | 无 |",
    ]
    missing = [expectation for expectation in expectations if expectation not in rendered]
    if missing:
        print("selftest: 渲染结果里缺这几行（不稳定标记、轮数、排除与失败、二进制自己计的那一列与它的缺值都要照报）：")
        for line in missing:
            print("  ", line)
        print(rendered)
        return 1
    print(f"selftest: 通过（{len(expectations)} 行渲染与期望逐字相同，不稳定的格子带了 ⚠）")
    return 0


def main():
    if len(sys.argv) != 2:
        print("用法：e152-tables.py <产物> | --selftest", file=sys.stderr)
        return 2
    if sys.argv[1] == "--selftest":
        return selftest()
    with open(sys.argv[1], encoding="utf-8", errors="replace") as product:
        sys.stdout.write(render(product.read()))
    return 0


if __name__ == "__main__":
    sys.exit(main())
