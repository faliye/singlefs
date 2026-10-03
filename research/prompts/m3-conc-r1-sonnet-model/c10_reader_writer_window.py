#!/usr/bin/env python3
"""C10（Q10）：一个只读读者走活树（H 跳）与一个写者连发 W 次发布，在全部事件交错上数「无保护时读者读到可能已被释放并重用的块」的交错条数。
纯算术，不与 crates/ 共用代码。对应 C214：任何活树遍历读完父指针、再去读子块，这中间子块不许被释放并重用。

模型：读者每跳两个事件——Rp_i（读父指针，指向子块 B_i，此刻 B_i 在现行版本里）与 Rc_i（读子块）。写者的每次发布都可能把 B_i 换下（最坏情形：
每次发布都换下读者当前要读的那个子块——这样只看「两件事之间过去了几次发布」）。B_i 被换下后进 defer 队列，K 次发布之后可被重用。
违例（无保护）：Rp_i 与 Rc_i 之间过去的发布数 ≥ K。K 的口径有 ±1 的出入（C214 写「窗口 = K ÷ 发布率」），K 取 1、2、3、6 都列，结论只看方向。
这里只量「没有任何保护时暴露的交错有多少」。两种保护——运行期断言（越界中止遍历）与读者钉住版本（写者不重用被钉住的版本之后换下的块）——
按定义把这批违例变成中止或清零，不在模型里重复数；它们各自要什么，写在报告 C10 节。
"""
from itertools import combinations
from math import comb

H, W = 3, 6


def exposed(hops, publishes, k):
    count = 0
    for writer_slots in map(set, combinations(range(2 * hops + publishes), publishes)):
        publishes_done, parent_read_at, reader_events, violated = 0, 0, 0, False
        for position in range(2 * hops + publishes):
            if position in writer_slots:
                publishes_done += 1
            else:
                if reader_events % 2 == 0:
                    parent_read_at = publishes_done
                elif publishes_done - parent_read_at >= k:
                    violated = True
                reader_events += 1
        count += violated
    return count


def main():
    total = comb(2 * H + W, W)
    print(f"读者 {H} 跳（{2 * H} 个事件）× 写者 {W} 次发布：全部交错 {total} 条（C({2 * H + W},{W}) = {total}）")
    print(f"  {'K':>3s} {'无保护暴露的交错':>14s} {'占比':>7s}")
    for k in (1, 2, 3, 6):
        bad = exposed(H, W, k)
        print(f"  {k:3d} {bad:14d} {100 * bad / total:6.1f}%")
    print("读法：K 越大暴露越少，但 K 次发布一旦被读者的一跳超过就有违例；无保护时只能靠「一跳比 K 次发布短」这条没人守的余量（C214）。")


if __name__ == "__main__":
    main()
