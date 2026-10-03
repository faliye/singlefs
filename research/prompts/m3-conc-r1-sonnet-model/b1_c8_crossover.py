#!/usr/bin/env python3
"""C8（Q8，C58 报警）的算术：交点 N* = 合并速率 r × 一次发布的设备同步时长 S，三个交点来源在别的机器上什么时候永不响、什么时候总在响。

输入全部取自仓里的页，每个数的出处写在常量旁边；标「取样点」的不是量的，只用来看方向。
r：E17 页 17-writebuffer合并上限.md 第 8 行「29.9–31.8M 条目/秒」；C58 登记行用 31.2M。
S：E44 页 44-序号位宽的本机实测与代价.md 第 64 行 sync_1 = 2785 次/秒 → 1/2785 = 359.066 µs（一次 fdatasync）；
   D16 已定项 7 的屏障口径：每次发布四个序点（两道 FLUSH + 根槽 FUA + 系统配置轮换后一道屏障），所以一次发布的设备同步 ≥ 4 个序点，
   一次 fdatasync 的 359 µs 乘 4 只是「四个序点各等一次 fdatasync」的粗上限（取样点，不是量的）；
   E152 页：singlefs 发布 B「每次 6.6 ms」（虚机里、含设备与 CPU，不能拆）。
"""
R_LOW, R_C58, R_HIGH = 29.9e6, 31.2e6, 31.8e6          # 条目/秒
S_ONE_FDATASYNC = 1 / 2785                                # 秒，E44 sync_1
CONSTANT = 11_200                                         # C58 行「约 1.1 万」= 31.2e6 × 359e-6 = 11200.8


def matrix():
    print("N* = r × S（条目/次发布），r 取 E17 区间两端与 C58 用的点；S 取下面几档")
    s_points = [
        ("S=0.1×E44（取样点：设备同步快 10 倍，如带掉电保护的企业盘，没量）", 0.1 * S_ONE_FDATASYNC),
        ("S=1×E44（一次 fdatasync，C58 用的）", S_ONE_FDATASYNC),
        ("S=4×E44（D16 已定项 7 四个序点各等一次 fdatasync，取样点）", 4 * S_ONE_FDATASYNC),
        ("S=6.6 ms（E152 发布 B 每次，虚机，含 CPU）", 6.6e-3),
        ("S=10×E44（取样点：设备同步慢 10 倍，如旋转盘，没量）", 10 * S_ONE_FDATASYNC),
    ]
    print(f"  {'S 档':70s} {'N*(r=29.9M)':>12s} {'N*(r=31.2M)':>12s} {'N*(r=31.8M)':>12s}")
    for name, s in s_points:
        print(f"  {name:70s} {R_LOW * s:12.0f} {R_C58 * s:12.0f} {R_HIGH * s:12.0f}")
    print(f"\n编译期常量 {CONSTANT}（= 31.2e6 × 359e-6 取整，C58 行）在这几档机器上的错位（以 r=31.2M 为准）：")
    print("  真值 N* < 常量 ⇒ 每次发布条目数落在 (N*, 常量] 时 CPU 已是天花板而报警不响（永不响带）；")
    print("  真值 N* > 常量 ⇒ 条目数落在 (常量, N*] 时设备仍是瓶颈而报警在响（总在响带）。")
    for name, s in s_points:
        n_star = R_C58 * s
        if abs(n_star - CONSTANT) <= 0.01 * CONSTANT:
            band = f"无错位（N*={n_star:.0f}，与常量差 {abs(n_star - CONSTANT):.0f} 条，取整误差）"
        elif n_star < CONSTANT:
            band = f"永不响带 ({n_star:.0f}, {CONSTANT}]，宽 {CONSTANT - n_star:.0f} 条，占常量的 {100 * (CONSTANT - n_star) / CONSTANT:.1f}%"
        elif n_star > CONSTANT:
            band = f"总在响带 ({CONSTANT}, {n_star:.0f}]，宽 {n_star - CONSTANT:.0f} 条，是常量的 {n_star / CONSTANT - 1:.2f} 倍"
        else:
            band = "无错位"
        print(f"  {name:70s} {band}")


def stale_on_move():
    print("\n候选 a（mkfs 实测写进系统配置）的盘换机器：写进盘里的是 mkfs 那台机器的 N*，换到别的机器上就是上面的常量错位同一张表：")
    print("  从本机（S=1×E44）搬到 S=10×E44 的机器：盘里写 11200，真值 112000 ⇒ 总在响带 (11200, 112000]")
    print("  从 S=10×E44 的机器搬到本机：盘里写 112000，真值 11200 ⇒ 永不响带 (11200, 112000]")
    print(f"  两个方向的带宽都是 {112000 - 11200}。候选 a 比候选 c 多的只有「mkfs 那台机器上是对的」这一点。")


def unit_mismatch():
    print("\n条目的单位：E17 的「条目」是 sort-merge-sweep 的输入条目（D8 已定项 15：按每 4 KiB 用户写 3 条算）。")
    print("  crates/ 里没有 write buffer（grep -rniE 'write.?buffer|sort.?merge' crates/singlefs-core/src 零命中，见报告）；")
    print("  发布路径每次把每棵树整片重建（transaction.rs 的 publish_admitted），CPU 随「建树输入记录数」走，不随 E17 的合并条目走。")
    print("  ⇒ 把 11200 拿来当本实现的交点，单位对不上；第一步是标定：同一个计数器 + prepare 与 persist 两段的耗时，求本实现的 r_impl 与 S_pub。")


def entries_estimate():
    print("\n今天一次发布的建树输入记录数估计（推的：口径取自 publish_admitted 里各列表的长度；没有跑实现核对）：")
    print("  u = 这次发布写的数据单元数；两块盘；released = 被换下的单元数（覆盖写时约 u + 固定的十来个）")
    fixed_nodes = 12      # 取样点：extent 上段若干 + inode 叶根 + 分配记录树节点 + 记账树 + 映射树 + 树表，约十来个角色（没数）
    for u in (1, 3, 144, 1000, 1800):
        extent = u
        mapping = u + fixed_nodes
        allocation = 2 * (u + fixed_nodes) + 2 * (u + fixed_nodes)   # 新落点每盘一条 + 换下的每盘一条
        accounting = 15
        tree_table = 11
        inode = 1
        total = extent + mapping + allocation + accounting + tree_table + inode
        print(f"  u={u:5d}: extent {extent:5d} + mapping {mapping:5d} + allocation {allocation:5d} + accounting {accounting} "
              f"+ tree_table {tree_table} + inode {inode} = {total:6d}  （占 11200 的 {100 * total / 11200:5.1f}%）")
    print("  这张表只说明量级：一次发布写 ~1800 个 32 KiB 单元（~56 MiB）才摸到 11200；单个事务一次一个单元时差三个数量级。")
    print("  fixed_nodes=12 是取样点，不是数出来的；结论只用「差一到三个数量级」这个方向。")


if __name__ == "__main__":
    matrix()
    stale_on_move()
    unit_mismatch()
    entries_estimate()
