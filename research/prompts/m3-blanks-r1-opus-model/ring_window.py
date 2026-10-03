#!/usr/bin/env python3
"""B6：环窗口 = (根环里的根数 + 1) × W 的两种读法，在一段「先在根少的时候写满、再删一个大文件」的历史上各差多少。

读法甲：根数 = 根环槽数 3·S（系统配置每区槽数 S，4..16）。读法乙：根数 = 准入那一刻环里读得出的根条数 r。
删除的每一批新占至多 W 槽 / 盘，被换下的旧槽要等引用它的根全出候选集才回可分配集合（候选集平时是整个根环）：
第 b 批发布之后还钉着的删除新写 = W · min(b, 3S) + W（在飞那一批）。这就是 m3-delete-r3 判决 G1-窗口系数那一行说的顶点（第 3S 次发布附近）。
可用在写满那一刻 = 0；删除类发布不判式子、只判落点，落点不够时会话推抬 F（回退候选缩到最近 4 条）再发。
数这一段删除里落点不够、要靠抬 F 的批数，与抬 F 之后丢掉的回退深度。
"""

def run(S, r_at_fill, W, batches, reading):
    ring = 3 * S
    window = (ring + 1) * W if reading == "A" else (r_at_fill + 1) * W
    free = window  # 写满时可用 = 0：单元区里只剩窗口那一份是空的
    pinned = []  # 每一批被换下、还被候选集里的根钉着的槽
    short_batches = 0
    lost_depth = 0
    candidates = ring
    for b in range(batches):
        need = W
        if free < need:
            short_batches += 1
            # 抬 F：候选集缩到最近 4 条，更早那几批钉着的槽回来
            released = pinned[:-4] if len(pinned) > 4 else []
            free += sum(released)
            pinned = pinned[-4:]
            lost_depth = max(lost_depth, candidates - 4)
            candidates = 4
        free -= need
        pinned.append(need)  # 这一批换下的旧槽进钉住
        if len(pinned) > candidates:
            free += pinned.pop(0)
        candidates = min(ring, candidates + 1)
    return window, short_batches, lost_depth

def main():
    W = 368  # 4 GiB、K = 144 的 W 上界（m3-delete-r3 辩方报告第二节第 26 行，算的）
    for S in (4, 8, 16):
        for r in (4, 6, 3 * S):
            for reading in ("A", "B"):
                window, short, lost = run(S, r, W, 10 * 3 * S, reading)
                print(f"E name=ring_window S={S} ring_slots={3 * S} r_at_fill={r} reading={reading} W={W} "
                      f"window_slots={window} window_in_W={window // W} batches={30 * S} short_batches={short} "
                      f"rollback_depth_lost={lost}")

if __name__ == "__main__":
    main()
