#!/usr/bin/env python3
"""m3-conc-r1 攻方：环窗口 (3S + 1) × W（D28（挂载期承诺量） 已定项 1「环窗口」）在多次发布同时在飞时够不够。

槽级模拟，不引 crates/：
- 一串删除批发布 1..N，每一次重写同一组 W 个节点（尾叶、祖先、分配记录树那几片），每个节点写一份新副本、换下上一份。
- 副本 p（第 p 次发布写的）只被根 p 引用（下一次发布把 W 个节点全换掉）。
- 根环里留最近 R = 3S 个已落盘的根；副本 p 在根 p 离开根环之后才可回收（回收不早于换下它的那次发布落盘）。
- 同时在飞 k 次：第 n+1..n+k 次发布已经取了落点、写了副本，根还没落盘（问题单 Q1「多次发布同时在飞」；k = 1 就是今天的串行）。
- 量：任一时刻占着的删除副本槽数（含当前那一版的 W 个），对照环窗口 (3S + 1) × W。
用法：ring_window_inflight.py
"""


def peak_live_slots(slots_per_region, w, k, publishes):
    ring = 3 * slots_per_region
    live = {}          # 发布号 -> 它写的副本槽数
    committed = 0      # 已落盘的最大发布号
    issued = 0
    peak = 0
    while committed < publishes:
        # 发到在飞 k 个为止
        while issued < min(publishes, committed + k):
            issued += 1
            live[issued] = w
            peak = max(peak, sum(live.values()))
        # 最早那次落盘：根进环，挤出的根所引用的副本释放
        committed += 1
        leaving = committed - ring
        if leaving >= 1:
            live.pop(leaving, None)
        peak = max(peak, sum(live.values()))
    return peak


def main():
    # W 取样：24 是小文件的 W；87 是 1 万单元文件最大一批（m3-delete-r3 判决 G1-W 上界一行，攻方量的）
    for s in (4, 8, 16):
        for w in (24, 87):
            for k in (1, 2, 3, 4, 8, 16):
                peak = peak_live_slots(s, w, k, publishes=10 * 3 * s + 50)
                window = (3 * s + 1) * w
                print(f"E name=ring_window slots_per_region={s} w={w} in_flight={k} peak_live_slots={peak} "
                      f"window_slots={window} closed_form_peak={(3 * s + k) * w} "
                      f"peak_le_window={'true' if peak <= window else 'false'} deficit_slots={max(0, peak - window)}")


if __name__ == "__main__":
    main()
