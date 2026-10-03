#!/usr/bin/env python3
"""C8 判别力自证的设计核对：把「计数器接线」写成一个纯函数，注入七种接线故障，看三类自证用例各抓到哪几种。
它核的是用例设计够不够（每种故障至少被一条用例抓到、每条用例至少抓到一种），不是核实现。

计数器 = 各建树输入记录数之和（报告 C8 节给的七项），项目口径同 transaction.rs publish_admitted 里的列表长度：
  extent 叶记录 = 写的数据单元数 u；inode 记录 = 新建 inode 数 n_i（+ 1 若写了文件）；分配记录 = 2 盘 × (新落点 + 换下)；
  记账 15；映射 = u + 固定节点；树表 11。固定项里的数是取样点（见 b1），故障判定只看「翻倍之后超不超门槛」。
"""
FIXED_NODES = 12


def terms(load):
    u, n_i = load.get("u", 0), load.get("inodes", 0)
    return {
        "extent": u,
        "inode": n_i + (1 if u else 0),
        "allocation": 2 * (u + FIXED_NODES) * 2 if u else 2 * FIXED_NODES,
        "accounting": 15,
        "mapping": (u + FIXED_NODES) if u else FIXED_NODES,
        "tree_table": 11,
    }


def counter(load, fault):
    t = terms(load)
    if fault == "F1 计数器没接（恒 0）":
        return 0
    if fault == "F2 漏 inode 项":
        t["inode"] = 0
    if fault == "F3 漏 extent 项":
        t["extent"] = 0
    if fault == "F4 按事务数而不是按发布求和（只取最后一个事务）":
        t = {k: v // max(1, load.get("transactions", 1)) for k, v in t.items()}
    return sum(t.values())


def red(entries, threshold, fault):
    if fault == "F5 门槛恒无穷":
        return False
    if fault == "F6 比较取反（<=）":
        return entries <= threshold
    if fault == "F7 读的是上一次发布的计数":
        return False   # 单次调用里读到初值 0
    return entries > threshold


FAULTS = ["F0 正确接线", "F1 计数器没接（恒 0）", "F2 漏 inode 项", "F3 漏 extent 项",
          "F4 按事务数而不是按发布求和（只取最后一个事务）", "F5 门槛恒无穷", "F6 比较取反（<=）", "F7 读的是上一次发布的计数"]
BASE_INODES = {"inodes": 5600}
BASE_UNITS = {"u": 1500, "transactions": 1500}     # 一次顺序写切成 1500 个事务（切分纪律一事务一单元）
THRESHOLD = 11_200


def tests(fault):
    """→ {用例名: 是否通过}。通过 = 用例的预期成立。"""
    out = {}
    for name, base in (("T1a 翻倍(新建 inode)", BASE_INODES), ("T1b 翻倍(写数据单元)", BASE_UNITS)):
        doubled = {k: v * 2 for k, v in base.items()}
        green = not red(counter(base, fault), THRESHOLD, fault)
        reds = red(counter(doubled, fault), THRESHOLD, fault)
        out[name] = green and reds
    low = 100
    out["T2 门槛挪动(基线负载在低门槛下必须红, 挪回必须绿)"] = (
        red(counter(BASE_INODES, fault), low, fault) and not red(counter({"inodes": 1}, fault), THRESHOLD, fault))
    # T3 钉绝对值：每种负载下的计数等于手算值（手算值取故障 F0 的结果在脚本外逐项加出来：这里写死两个数）
    out["T3 钉绝对值(两种负载各一个手算值)"] = (
        counter({"inodes": 100}, fault) == EXPECTED_INODES_100 and counter({"u": 10, "transactions": 10}, fault) == EXPECTED_UNITS_10)
    return out


# 手算（与 terms() 不共用代码，逐项加）：
#   inodes=100（没写文件）：extent 0 + inode 100 + allocation 2 盘 × 固定节点 12 = 24 + 记账 15 + 映射 固定节点 12 + 树表 11
EXPECTED_INODES_100 = 0 + 100 + 24 + 15 + 12 + 11
#   u=10：extent 10 + inode 1 + allocation 2*(10+12)*2=88 + 记账 15 + 映射 10+12=22 + 树表 11
EXPECTED_UNITS_10 = 10 + 1 + 88 + 15 + 22 + 11


def main():
    names = list(tests(FAULTS[0]))
    print("用例通过 = 预期成立；接线故障被抓到 = 至少一条用例不通过")
    print(f"  {'故障':50s} " + " | ".join(f"{n[:18]:18s}" for n in names) + " | 抓到?")
    caught_by = {n: 0 for n in names}
    for fault in FAULTS:
        result = tests(fault)
        fails = [n for n in names if not result[n]]
        for n in fails:
            caught_by[n] += 1
        print(f"  {fault:50s} " + " | ".join(f"{'通过' if result[n] else '不通过':18s}" for n in names)
              + f" | {'抓到' if fails else ('正确接线应全通过' if fault.startswith('F0') else '漏网')}")
    print("每条用例抓到的故障数：", caught_by)
    assert all(tests(FAULTS[0]).values()), "正确接线必须全部通过"
    missed = [f for f in FAULTS[1:] if all(tests(f).values())]
    print("漏网的故障：", missed or "无")


if __name__ == "__main__":
    main()
