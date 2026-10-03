#!/usr/bin/env python3
"""C6（Q6）：第一版并行形态下的同步点清单，与现有罩（litmus / Verus M2 / 入库计数模型 / 无）对账，数每个候选各罩到几个。
表是人列的（出处在每行的「依据」里，行号见报告 C6 节）；脚本只做对账与计数，不判对错。

层：进程内 = 线程之间的交接与互斥（Rust 里由 std 的锁 / 通道给先后，crates 里 forbid(unsafe_code)，core/src 今天零 Mutex/Atomic）；
    设备序 = 屏障 / FUA 排出的落盘先后（litmus 里用 smp_wmb 当它的代理）；恢复侧 = 崩溃后取前缀。
需要（甲 / 乙 / 丙）：Q1 的三个候选里这一点是不是新出现或要守。
罩：litmus = 现有 8 个 .litmus 里的 3 条 Never；verus = replay-prefix 的 M2；model = 已入库的计数模型（E166 / E169 / E167）；none = 没有。
"""
POINTS = [
    # id, 同步点, 层, 甲, 乙, 丙, 现有罩
    ("S1", "取号：事务号 / txg / jsn 计数器 / 树 ID 水位只有一个持有者", "进程内", 1, 1, 1, "model"),
    ("S2", "准入与取落点在同一个临界区（Q4 的 A 读法、Q5）", "进程内", 1, 1, 1, "model"),
    ("S3", "记录按事务号顺序追加（Q2：jsn 次序 = 事务号次序）", "进程内", 1, 1, 1, "model"),
    ("S4", "单元写 → 屏障 → journal 记录写", "设备序", 1, 1, 1, "litmus"),
    ("S5", "记录写 → 屏障 → 根槽 FUA", "设备序", 1, 1, 1, "litmus"),
    ("S6", "根槽 → 系统配置轮换 → 屏障 → 发布返回", "设备序", 1, 1, 1, "litmus"),
    ("S7", "事务号 N 的 fsync 返回前等 < N 的事务已追加 ∨ 已作废 ∨ 已确定切换或转只读（D13 已定项 2）", "进程内", 1, 1, 1, "model"),
    ("S8", "作废对并发后继可见（D13 已定项 2 括注）", "进程内", 1, 1, 1, "model"),
    ("S9", "跨发布的根次序：后一次发布的根不先于前一次发布的单元与记录持久（E169 根序 / 根乱）", "设备序", 0, 0, 1, "none"),
    ("S10", "写失败的级联：N 冻结时 N+1 .. 已在它的分配器状态上准备好的发布全部作废", "进程内", 1, 0, 1, "none"),
    ("S11", "恢复取前缀：施加的是整数次整发布（M2）", "恢复侧", 1, 1, 1, "verus"),
    ("S12", "恢复取前缀：相邻两次施加的发布 txg 连号（M2 今天没写）", "恢复侧", 1, 1, 1, "none"),
]
CANDIDATES = (("甲 准备并行落盘串行", 3), ("乙 一次发布装多个事务", 4), ("丙 多次发布同时在飞", 5))


def main():
    print(f"同步点 {len(POINTS)} 个；「需要」按候选数，「罩」按现有罩分")
    for name, column in CANDIDATES:
        need = [p for p in POINTS if p[column]]
        by = {}
        for p in need:
            by.setdefault(p[6], []).append(p[0])
        in_process = [p[0] for p in need if p[2] == "进程内"]
        print(f"\n{name}：需要 {len(need)} 个")
        for kind in ("litmus", "verus", "model", "none"):
            ids = by.get(kind, [])
            print(f"   {kind:7s} {len(ids):2d} 个 {ids}")
        print(f"   其中进程内 {len(in_process)} 个 {in_process}（litmus 用的是 LKMM 的 smp_wmb 代理设备序，进程内的锁与通道它不建模）")
        litmus_possible = [p[0] for p in need if p[2] == "设备序"]
        print(f"   设备序 {len(litmus_possible)} 个 {litmus_possible}；现有 litmus 罩 {sum(1 for p in need if p[2] == '设备序' and p[6] == 'litmus')} 个")
    print("\n现有 litmus 一个写者一个读者：grep -c 'P2' litmus/*.litmus 全 0（事实表 e.1）；新增两发布并行的 litmus 要三个进程（两个写者加恢复）。")


if __name__ == "__main__":
    main()
