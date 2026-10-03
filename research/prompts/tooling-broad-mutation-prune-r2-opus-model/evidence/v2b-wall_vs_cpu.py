#!/usr/bin/env python3
"""V2②：同一条测试，单独跑与挤在同两个核上并发跑，比墙钟与 CPU 时间（用户 + 系统）。
用法：wall_vs_cpu.py <测试二进制> <测试名> <核列表> <并发数…>"""
import os, subprocess, sys, time
binary, test, cores = sys.argv[1:4]
for n in [int(x) for x in sys.argv[4:]]:
    started = {}
    for i in range(n):
        p = subprocess.Popen(["taskset", "-c", cores, binary, "--exact", test, "--test-threads=1"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        started[p.pid] = time.time()
    rows = []
    while started:
        pid, status, usage = os.wait4(-1, 0)
        if pid in started:
            rows.append((time.time() - started.pop(pid), usage.ru_utime + usage.ru_stime, os.waitstatus_to_exitcode(status)))
    walls = sorted(r[0] for r in rows); cpus = sorted(r[1] for r in rows)
    print(f"concurrent={n} cores={cores} wall-min={walls[0]:.2f}s wall-max={walls[-1]:.2f}s cpu-min={cpus[0]:.2f}s cpu-max={cpus[-1]:.2f}s exit-codes={sorted(set(r[2] for r in rows))}", flush=True)
