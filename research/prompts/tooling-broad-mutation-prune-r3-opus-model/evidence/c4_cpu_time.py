#!/usr/bin/env python3
"""C4 的口径：同一条测试在不同线程数、超订、SMT 兄弟核被占时的墙钟与 CPU 时间（用户 + 系统，wait4 的 rusage）。

用法：c4_cpu_time.py <多线程测试二进制> <单线程测试二进制> <cwd> <重复次数>
多线程测试：stale_snapshot_substitution_is_refused_by_every_entry_over_the_swept_user_steps（16 格，线程数取 SINGLEFS_THREAD_CAP）。
单线程测试：seventy_nine_devices_fill_the_accounting_node_exactly_and_still_publish。
CPU 编号：本机 k 与 k+16 是同一物理核的两个 SMT 线程（lscpu -e 的 CORE 列）。
"""
import os
import subprocess
import sys
import time

MULTI, SINGLE, CWD, REPS = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4])
MULTI_TEST = "stale_snapshot_substitution_is_refused_by_every_entry_over_the_swept_user_steps"
SINGLE_TEST = "seventy_nine_devices_fill_the_accounting_node_exactly_and_still_publish"


def run(binary, test, cpus, threads=None):
    env = dict(os.environ)
    if threads is not None:
        env["SINGLEFS_THREAD_CAP"] = str(threads)
    started = time.monotonic()
    child = subprocess.Popen(["taskset", "-c", cpus, binary, "--exact", test, "--test-threads=1"], cwd=CWD, env=env,
                             stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    _, status, usage = os.wait4(child.pid, 0)
    wall = time.monotonic() - started
    return wall, usage.ru_utime + usage.ru_stime, os.waitstatus_to_exitcode(status)


def spinner(cpu):
    return subprocess.Popen(["taskset", "-c", cpu, "python3", "-c", "while True: pass"])


def report(label, rows):
    walls = [r[0] for r in rows]
    cpus = [r[1] for r in rows]
    print(f"C4 {label} reps={len(rows)} wall-min={min(walls):.2f} wall-max={max(walls):.2f} "
          f"cpu-min={min(cpus):.2f} cpu-max={max(cpus):.2f} exit-codes={sorted(set(r[2] for r in rows))}", flush=True)


def main():
    # 单线程测试：独占物理核 8 / SMT 兄弟 24 被占 / 另一物理核 9 被占（对照）/ 兄弟与本核都挤两个
    report("single cpu=8 sibling-idle", [run(SINGLE, SINGLE_TEST, "8") for _ in range(REPS)])
    for label, cpu in (("single cpu=8 spinner-on-sibling-24", "24"), ("single cpu=8 spinner-on-other-core-9", "9")):
        spin = spinner(cpu)
        try:
            report(label, [run(SINGLE, SINGLE_TEST, "8") for _ in range(REPS)])
        finally:
            spin.kill()
            spin.wait()
    # 多线程测试（线程上限 12：本条腿派发的上限）：线程数 1 / 4 / 12，钉在 8 个逻辑 CPU（8–15，各在不同物理核）上；
    # 再把 12 线程挤到 2 个逻辑 CPU（8、9）上；再在 6 个逻辑 CPU（8–13）上跑 6 线程，SMT 兄弟（24–29）空着 / 各放一个空转
    for threads in (1, 4, 12):
        report(f"multi threads={threads} cpus=8-15", [run(MULTI, MULTI_TEST, "8-15", threads) for _ in range(REPS)])
    report("multi threads=12 cpus=8-9", [run(MULTI, MULTI_TEST, "8-9", 12) for _ in range(REPS)])
    report("multi threads=6 cpus=8-13 siblings-idle", [run(MULTI, MULTI_TEST, "8-13", 6) for _ in range(REPS)])
    spins = [spinner(str(c)) for c in range(24, 30)]
    try:
        report("multi threads=6 cpus=8-13 spinners-on-siblings-24-29", [run(MULTI, MULTI_TEST, "8-13", 6) for _ in range(REPS)])
    finally:
        for spin in spins:
            spin.kill()
            spin.wait()

if __name__ == "__main__":
    main()
