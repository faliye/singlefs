#!/usr/bin/env python3
# admission: always 量的是此刻这台机器的显卡，上一次量到的数不替这一次作保
# run-condition: check python3 -c 'import sys, torch; sys.exit(0 if torch.cuda.is_available() else 1)' :: 装好带 CUDA 的 torch，并确认显卡在、驱动起得来（nvidia-smi 能列出卡）
# task-server-card-lease: on-demand
"""证明 E21 那几个数确实是 GPU 跑出来的，不是 CPU 冒充的。

三条独立证据，任意一条单独都够：
  1. **物理不可能**：显存内吞吐 1041 GB/s 是本机内存带宽（64.9 GB/s）的 16 倍，
     CPU 造不出这个数。
  2. **占用率**：空载采样当阴性对照（应当 ~0%），持续跑时再采（应当明显抬起来）。
  3. **搬到 CPU 上跑同一段代码**，吞吐必须塌回内存带宽量级。

按需领卡（文件头 `task-server-card-lease: on-demand`；领还卡的逻辑与退出码在同目录 e21_card_lease.py 文件头）：整条经 task-server/task submit 交，服务起跑不分卡；
上卡这一段在子进程里跑，只看得到领到的那一张（torch 里序号 0），跑完、出错、被打断都还卡。没有领卡通道退 78，领卡没成退 1、不写产物。自证是
e21-card-lease-selftest.py（假客户端、假 torch，不起真卡）。
nvidia-smi 不受 CUDA_VISIBLE_DEVICES 管，占用率采样照领到的卡串里的序号点那一张；torch 里那一张的序号是 0。
"""
import subprocess, sys, threading, time, statistics, torch
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402
preflight(__file__)
import e21_card_lease  # noqa: E402

DEV = e21_card_lease.VISIBLE_DEVICE_INDEX

def sample(seconds, out, smi_index):
    end = time.time() + seconds
    while time.time() < end:
        r = subprocess.run(["nvidia-smi", f"--id={smi_index}",
                            "--query-gpu=utilization.gpu,memory.used",
                            "--format=csv,noheader,nounits"],
                           capture_output=True, text=True)
        if r.returncode == 0 and r.stdout.strip():
            u, m = r.stdout.strip().split(", ")
            out.append((int(u), int(m)))
        time.sleep(0.2)

def busy(seconds, dev, acc):
    end = time.time() + seconds
    while time.time() < end:
        for _ in range(200):
            torch.bitwise_xor(dev, acc, out=acc)
        torch.cuda.synchronize(DEV)

def main(card):
    n = 1024 * 1024 * 1024 // 8
    dev = torch.randint(0, 2**62, (n,), dtype=torch.int64, device=f'cuda:{DEV}')
    acc = torch.zeros_like(dev)
    torch.cuda.synchronize(DEV)

    idle = []
    sample(2.0, idle, card.index)
    print(f"E21RESULT name=idle samples={len(idle)} util_max={max(u for u,_ in idle)} "
          f"util_med={statistics.median([u for u,_ in idle])} mem_used_mib={idle[-1][1]}")

    live = []
    t = threading.Thread(target=sample, args=(6.0, live, card.index)); t.start()
    busy(6.0, dev, acc)
    t.join()
    print(f"E21RESULT name=busy samples={len(live)} util_max={max(u for u,_ in live)} "
          f"util_med={statistics.median([u for u,_ in live])} mem_used_mib={max(m for _,m in live)}")

    # 阴性对照：同一段代码搬到 CPU
    h = torch.randint(0, 2**62, (n,), dtype=torch.int64)
    ha = torch.zeros_like(h)
    t0 = time.perf_counter()
    for _ in range(3): torch.bitwise_xor(h, ha, out=ha)
    cpu = (time.perf_counter() - t0) / 3
    torch.cuda.synchronize(DEV); t0 = time.perf_counter()
    for _ in range(3): torch.bitwise_xor(dev, acc, out=acc)
    torch.cuda.synchronize(DEV); g = (time.perf_counter() - t0) / 3
    gb = n * 8 / 1e9 * 2
    print(f"E21RESULT name=cpu_vs_gpu cpu_gbps={gb/cpu:.1f} gpu_gbps={gb/g:.1f} ratio={cpu/g:.1f}")

    idle_med = statistics.median([u for u,_ in idle])
    busy_med = statistics.median([u for u,_ in live])
    ok = busy_med >= 50 and idle_med <= 10 and (cpu/g) > 3
    print(f"E21RESULT name=done proof_ok={int(ok)}")
    if not ok:
        sys.stderr.write("e21: 证明不成立——空载/满载占用率没分开，或 CPU 没塌下去\n"); sys.exit(4)

sys.exit(e21_card_lease.run_on_leased_card(main))
