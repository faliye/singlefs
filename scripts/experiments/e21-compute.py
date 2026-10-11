#!/usr/bin/env python3
"""E21 的**计算段上界**：显存内的带宽受限算子能跑多快。

不需要 nvcc：scrub / EC / 压缩的内层循环都是**带宽受限**的，
带宽受限核函数的天花板就是显存带宽，而那个用现成的按位算子就能量到。
⇒ 量出来的是**核函数的上界**（真核函数只会更慢），足以回答
「端到端里传输段占多少」——那是 E21 判据 1 的分母构成。

纪律：
  * 每次计时前后 `torch.cuda.synchronize()`；不同步量到的是异步启动。
  * **阳性对照**：显存内算子必须远快于「含传输的同一个算子」。
    测不出这个差别 ⇒ 计时没有判别力（多半是没同步），整轮作废。
  * N=5 取中位，先跑一轮丢弃。
"""
import sys, time, statistics, torch

DEV, MIB, ROUNDS = 0, 1024, 5

def timed(fn):
    torch.cuda.synchronize(DEV); t0 = time.perf_counter()
    fn(); torch.cuda.synchronize(DEV)
    return time.perf_counter() - t0

def main():
    if not torch.cuda.is_available():
        sys.stderr.write("e21: 没有 CUDA —— 这不是「为 0」，是没测\n"); sys.exit(2)
    free, _ = torch.cuda.mem_get_info(DEV)
    if free < 4 * 1024**3:
        sys.stderr.write("e21: 显存余量不足 4 GiB，不跑\n"); sys.exit(2)
    n = MIB * 1024 * 1024 // 8
    print(f"E21RESULT name=config dev={torch.cuda.get_device_name(DEV)} mib={MIB} rounds={ROUNDS}")

    dev = torch.randint(0, 2**62, (n,), dtype=torch.int64, device=f'cuda:{DEV}')
    acc = torch.zeros_like(dev)
    gb = n * 8 / 1e9

    # 计算段：显存内的按位异或折叠（带宽受限，是任何此类核函数的上界）
    timed(lambda: torch.bitwise_xor(dev, acc, out=acc))
    comp = statistics.median([timed(lambda: torch.bitwise_xor(dev, acc, out=acc)) for _ in range(ROUNDS)])
    # 读 + 写各一遍 ⇒ 有效带宽是数据量的两倍
    print(f"E21RESULT name=compute op=xor gbps={2*gb/comp:.1f} ns={int(comp*1e9)}")

    # 含传输的同一个算子（锁页主存 → 显存 → 算 → 不回传）
    host = torch.empty(n, dtype=torch.int64, pin_memory=True)
    def with_xfer():
        dev.copy_(host); torch.bitwise_xor(dev, acc, out=acc)
    timed(with_xfer)
    both = statistics.median([timed(with_xfer) for _ in range(ROUNDS)])
    print(f"E21RESULT name=compute_plus_transfer gbps={gb/both:.1f} ns={int(both*1e9)}")

    ratio = both / comp
    print(f"E21RESULT name=poscontrol xfer_over_compute={ratio:.1f}")
    print(f"E21RESULT name=share transfer_pct={100*(both-comp)/both:.1f}")
    ok = ratio > 3.0
    print(f"E21RESULT name=done poscontrol_ok={int(ok)}")
    if not ok:
        sys.stderr.write(f"e21: 阳性对照失败——含传输只比纯计算慢 {ratio:.1f}×（要求 >3），"
                         "多半是没同步，整轮作废\n"); sys.exit(4)

main()
