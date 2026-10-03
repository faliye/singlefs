#!/usr/bin/env python3
"""m3-conc-r1 攻方：C58 交点两半里「一次设备同步」与「拷一份分配器」的本机取样。

只量两样，都不碰 crates/：
1. 4 KiB pwrite + fdatasync 的单次耗时，在两个目录上各 5 轮 × 200 次：
   草稿目录（本机 NVMe 上的 ext4，与 E44 的 sync_1 同类）与 /dev/shm（tmpfs，测试里文件盘常落的那一类）。
   每轮报中位数与均值（微秒）。轮与轮之间的差就是「mkfs 时实测一次」会抓到的抖动。
2. 内存拷贝：bytes 拷 N 字节的耗时（中位数，20 次），N 取 450 KiB（transaction.rs 注释里第一版每次发布拷分配器的量）
   与 128 MiB（推的：1 TiB 每盘 6400 万槽、两盘各一字节一槽的 Vec<bool> 量级，C567 登记行写的是审阅员推的 190 MB）。
用法：fsync_probe.py <草稿目录>
输出：每行 `E name=... `，确定的只有行的种类与条数；数是本机此刻的取样，不复跑相等。
"""
import os
import statistics
import sys
import time

ROUNDS = 5
SYNCS_PER_ROUND = 200


def fsync_rounds(directory):
    path = os.path.join(directory, "m3-conc-r1-opus-fsync-probe.bin")
    block = os.urandom(4096)
    fd = os.open(path, os.O_RDWR | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        rounds = []
        for _ in range(ROUNDS):
            samples = []
            for i in range(SYNCS_PER_ROUND):
                os.pwrite(fd, block, (i % 64) * 4096)
                start = time.perf_counter_ns()
                os.fdatasync(fd)
                samples.append(time.perf_counter_ns() - start)
            rounds.append((statistics.median(samples) / 1000.0, statistics.mean(samples) / 1000.0))
        return rounds
    finally:
        os.close(fd)
        os.unlink(path)


def copy_median_seconds(n_bytes, repetitions=20):
    source = bytearray(os.urandom(min(n_bytes, 1 << 20))) * max(1, n_bytes // (1 << 20))
    source = source[:n_bytes] if len(source) >= n_bytes else source + bytearray(n_bytes - len(source))
    samples = []
    for _ in range(repetitions):
        start = time.perf_counter_ns()
        copied = bytes(source)
        samples.append(time.perf_counter_ns() - start)
        del copied
    return statistics.median(samples) / 1e9


def main():
    draft = sys.argv[1]
    for label, directory in (("nvme_ext4_draft_dir", draft), ("tmpfs_dev_shm", "/dev/shm")):
        for index, (median_us, mean_us) in enumerate(fsync_rounds(directory)):
            print(f"E name=fsync_round target={label} round={index} syncs={SYNCS_PER_ROUND} "
                  f"median_us={median_us:.1f} mean_us={mean_us:.1f}")
    for label, n_bytes in (("first_version_allocator_450KiB", 450 * 1024), ("one_tib_bitmaps_128MiB", 128 << 20)):
        seconds = copy_median_seconds(n_bytes)
        print(f"E name=copy target={label} bytes={n_bytes} median_us={seconds * 1e6:.1f} "
              f"bytes_per_second={n_bytes / seconds:.3e}")


if __name__ == "__main__":
    main()
