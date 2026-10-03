#!/usr/bin/env python3
"""m3-conc-r1 攻方：D13（验证路线） 已定项 2 的 fsync 返回条件第三析取支「已确定实例切换或转只读」。

两个事务 A（号 n−1）、B（号 n），记录按事务号整段追加：A 占 jsn 1..a，B 占 a+1..a+b（甲整段 / 乙整段的形状）。
故障：A 的第 f 条记录写失败（设备报错，那一条不落盘）；B 的记录全部发出。
B 的 fsync 在第三析取支成立时返回，返回值取「成功」或「报错」两种读法。
恢复（今天的五条口径，D23（journal 的角色与格式） 已定项 14）：jsn 从 1 起严格连续，提交标记齐的事务才施加。
枚举：B 的记录与 A 失败那条之前的记录各自落不落盘（整写子集），之后崩溃或转只读后重读。
数：B 的 fsync 报了成功、而恢复出来没有 B 的状态数（问题单 Q3 翻面观测「fsync 已向 B 的调用方返回而恢复丢掉了 B」）。
实例切换那一支：新实例从前缀末 +1 接着写（D23 已定项 14 注 3），B 那几条落在断号之后，永远进不了前缀。
用法：fsync_return_model.py
"""
from itertools import product


def recover(persisted, a, b):
    """persisted: jsn -> 落没落盘。交回施加了哪些事务。"""
    applied = []
    jsn = 1
    while persisted.get(jsn, False):
        jsn += 1
    prefix_end = jsn - 1
    if prefix_end >= a:
        applied.append("A")
    if prefix_end >= a + b:
        applied.append("B")
    return applied


def main():
    for a, b in ((1, 1), (2, 1), (2, 2), (3, 2)):
        for failing in range(1, a + 1):
            issued = [j for j in range(1, a + b + 1) if j != failing]
            for fsync_reading in ("success", "error"):
                for branch in ("read_only", "instance_switch"):
                    states = lost_after_success = 0
                    for bits in product((False, True), repeat=len(issued)):
                        persisted = dict(zip(issued, bits))
                        persisted[failing] = False
                        states += 1
                        applied = recover(persisted, a, b)
                        if branch == "instance_switch":
                            # 新实例接着写，断号之后的 B 不再可达（不跨实例边界）
                            pass
                        if fsync_reading == "success" and "B" not in applied:
                            lost_after_success += 1
                    print(f"E name=third_disjunct a={a} b={b} failing_record={failing} branch={branch} "
                          f"fsync_returns={fsync_reading} states={states} b_lost_after_success={lost_after_success}")


if __name__ == "__main__":
    main()
