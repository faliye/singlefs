#!/usr/bin/env python3
# E183 跑前登记的独立锚点（H4 的环窗口与切换预留的链重写那一半、H5 的行数与清扫）。不 import 装置、不读 crates/。只用下列条款句与 kb 常量：
#   槽 16384 字节；journal 环从槽 1024 起（D23 已定项 2 的布局：系统配置、根环在前 16 MiB），单元区起点 = 环末尾的下一个槽（D3 已定项 10 ④）。
#   分配记录树：叶罩 812 槽、内部扇出 169（layout 登记的两个常量）；根层级 = 最小的 R ≥ 1 使 Σ_盘 ⌈盘上槽数 ÷ (812 × 169^(R−1))⌉ ≤ 169（D8 已定项 14）；
#   N_ℓ = 第 ℓ 层（0 … R−1）两块盘合计罩到的位置数，每块盘按单元区首末槽 ⌊末 ÷ 跨度⌋ − ⌊首 ÷ 跨度⌋ + 1。
#   W = 24 + (6 + H_退役) + Σ_ℓ min(N_ℓ, 2·(K + 26))，K = 144；环窗口 = (3S + 1) × W 槽（D28 已定项 1「环窗口」一条）。
#   切换预留的链重写那一半 = (N_switch + 1) × 32768 × max(1, ⌈(rows0 + N_switch) ÷ 369⌉)，N_switch = 3（D28 已定项 3）。
#   H5：每次可写挂载取一个新实例号、写一行（实例 1 之后每个实例一行，D18 已定项 11「每次可写挂载都写行」）；写行之后暖机，直到两块盘上
#   各有一条本实例的根（D16 已定项 8；txg u 的根落盘 [0,1,0][u % 3]）；挂载末尾行数 > θ 就清扫，删去根环里一条根都没有的实例的行（D18 已定项 11
#   「回收」），清扫发布占一个 txg；正常卸载推空发布到现行 txg：现行 txg 除以 3 余 1 时 3 次、否则 2 次（D16 已定项 1「正常卸载」一行）。
import sys

SLOT = 16384
LEAF, FAN, K, NSW = 812, 169, 144, 3
REGION = [0, 1, 0]

def unit_area(C, L):
    start = 1024 + L // SLOT
    return start, C // SLOT - 1

def root_level(C):
    slots = C // SLOT
    R = 1
    while 2 * -(-slots // (LEAF * FAN ** (R - 1))) > FAN:
        R += 1
    return R

def ring_window_slots(C, L, S, H=0):
    first, last = unit_area(C, L)
    R = root_level(C)
    N = [2 * (last // (LEAF * FAN ** l) - first // (LEAF * FAN ** l) + 1) for l in range(R)]
    W = 24 + 6 + H + sum(min(n, 2 * (K + 26)) for n in N)
    return (3 * S + 1) * W, W, N, R

def chain_bytes(rows0):
    return (NSW + 1) * 32768 * max(1, -(-(rows0 + NSW) // 369))

def h5(S, theta, mode, cap):
    P = 3 * S
    owner = {0: 1, 1: 1, 2: 1, 3: 1}      # txg -> 实例（mkfs 那个进程取号 1：暖机 1、2，首个文件 3）
    u = 3
    def unmount():
        nonlocal u
        for _ in range(3 if u % 3 == 1 else 2):
            u += 1; owner[u] = g
    g = 1
    unmount()                               # 公共前缀：首个文件之后正常卸载
    rows = set()
    sweeps, peak_rows0, residuals, mounts_at_sweep = [], 0, [], []
    for m in range(1, cap + 1):
        g += 1
        rows.add(g - 1)                     # 写行那次把上一个实例的行写进表（实例 1 起）
        u += 1; owner[u] = g
        covered = {REGION[u % 3]}
        while covered != {0, 1}:
            u += 1; owner[u] = g; covered.add(REGION[u % 3])
        rows0 = len(rows)
        peak_rows0 = max(peak_rows0, rows0)
        if len(rows) > theta:
            in_ring = {owner[t] for t in range(max(0, u - P + 1), u + 1)}
            deletable = {r for r in rows if r not in in_ring}
            rows -= deletable
            u += 1; owner[u] = g             # 清扫发布
            sweeps.append(len(deletable)); residuals.append(len(rows)); mounts_at_sweep.append(m)
            if len(sweeps) == 2:
                return dict(mounts=m, sweeps=sweeps, residuals=residuals, mounts_at_sweep=mounts_at_sweep,
                            peak_rows0=peak_rows0, peak_pages=max(1, -(-(peak_rows0 + NSW) // 369)), final_txg=u)
        if mode == 'unmount':
            unmount()
    return dict(mounts=cap, sweeps=sweeps, residuals=residuals, mounts_at_sweep=mounts_at_sweep,
                peak_rows0=peak_rows0, peak_pages=max(1, -(-(peak_rows0 + NSW) // 369)), final_txg=u)

if __name__ == '__main__':
    MiB, GiB = 1 << 20, 1 << 30
    for (C, L) in [(4 * GiB, 768 * MiB), (64 * GiB, 768 * MiB), (1 << 40, 768 * MiB), (4 << 40, 768 * MiB), ((16 << 40) - MiB, 768 * MiB)]:
        for S in (4, 8, 16):
            slots, W, N, R = ring_window_slots(C, L, S)
            first, last = unit_area(C, L)
            print(f"H4 C={C} L={L} S={S} unit_area_start_slot={first} unit_area_slots={last - first + 1} root_level={R} N={N} W={W} "
                  f"ring_window_slots={slots} ring_window_bytes={slots * SLOT}")
    print(f"H4 chain_bytes rows0=1:{chain_bytes(1)} rows0=366:{chain_bytes(366)} rows0=367:{chain_bytes(367)} rows0=370:{chain_bytes(370)} rows0=739:{chain_bytes(739)}")
    for S in (4, 8, 16):
        for theta in (64, 365, 369, 738):
            for mode in ('unmount', 'crash'):
                r = h5(S, theta, mode, 2 * theta + 3 * S + 20)
                print(f"H5 S={S} theta={theta} mode={mode} " + " ".join(f"{k}={v}" for k, v in r.items()))
