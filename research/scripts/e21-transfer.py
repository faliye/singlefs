#!/usr/bin/env python3
"""E21 的**传输段**：主机 → 显存 → 主机的带宽。不含任何核函数。

为什么单独测这一段就有用：E21 判据 1 是
`CPU 16 线程耗时 ÷ (传输 + 计算 + 传回) 耗时`，小于 1 就否掉那一格。
⇒ **即使核函数耗时为零**，GPU 路径也至少要付「传输 + 传回」。
那一段单独就慢过 CPU 的话，这一格不必等核函数写出来就已经死了。
**这是必要条件，不是充分条件**——传输段够快只说明「还没被否掉」。

纪律：
  * 每次计时前后都 `torch.cuda.synchronize()`。不同步的话量到的是**异步启动**，
    不是传输，数字会好看一个数量级——这是这类测量最常见的错。
  * **阳性对照**：锁页内存必须明显快于可分页内存（可分页要多一次暂存拷贝）。
    测不出这个差别 ⇒ 计时没有判别力，整轮作废，不许把结果当结论。
  * N=5 取中位；先跑一轮丢弃（首次 CUDA 调用要建上下文）。
"""
import sys, time, statistics, torch

DEV = 0
SIZES_MIB = [1, 4, 16, 64, 256, 1024]
ROUNDS = 5

def timed(fn):
    torch.cuda.synchronize(DEV)
    t0 = time.perf_counter()
    fn()
    torch.cuda.synchronize(DEV)
    return time.perf_counter() - t0

def bench(mib, pinned):
    n = mib * 1024 * 1024
    host = torch.empty(n, dtype=torch.uint8, pin_memory=pinned)
    dev = torch.empty(n, dtype=torch.uint8, device=f'cuda:{DEV}')
    timed(lambda: dev.copy_(host))                      # 丢弃第一轮
    h2d = [timed(lambda: dev.copy_(host)) for _ in range(ROUNDS)]
    d2h = [timed(lambda: host.copy_(dev)) for _ in range(ROUNDS)]
    del dev, host
    torch.cuda.empty_cache()
    gb = n / 1e9
    return gb / statistics.median(h2d), gb / statistics.median(d2h)

def main():
    if not torch.cuda.is_available():
        sys.stderr.write("e21: 没有 CUDA —— 这不是「带宽为 0」，是没测\n"); sys.exit(2)
    free, total = torch.cuda.mem_get_info(DEV)
    print(f"E21RESULT name=config dev={torch.cuda.get_device_name(DEV)} "
          f"free_mib={free//2**20} total_mib={total//2**20} rounds={ROUNDS} torch={torch.__version__}")
    if free < 3 * 1024**3:
        sys.stderr.write("e21: 显存余量不足 3 GiB，不跑——挤爆别人的服务不值得\n"); sys.exit(2)

    rows = {}
    for pinned in (False, True):
        for mib in SIZES_MIB:
            h, d = bench(mib, pinned)
            rows[(pinned, mib)] = (h, d)
            print(f"E21RESULT name=transfer pinned={int(pinned)} mib={mib} "
                  f"h2d_gbps={h:.2f} d2h_gbps={d:.2f}")

    # 阳性对照：大块上锁页必须明显快于可分页
    big = max(SIZES_MIB)
    pg_h, _ = rows[(False, big)]
    pn_h, _ = rows[(True, big)]
    ratio = pn_h / pg_h if pg_h else float('nan')
    print(f"E21RESULT name=poscontrol mib={big} pageable_h2d={pg_h:.2f} "
          f"pinned_h2d={pn_h:.2f} speedup={ratio:.2f}")
    ok = ratio > 1.15
    print(f"E21RESULT name=done poscontrol_ok={int(ok)}")
    if not ok:
        sys.stderr.write(
            f"e21: 阳性对照失败——锁页只比可分页快 {ratio:.2f}×，计时没有判别力，整轮作废\n")
        sys.exit(4)

main()
