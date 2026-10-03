#!/usr/bin/env python3
"""C9（Q9）：删除的续做批与一个用户事务在飞时的交错违例数。纯状态机枚举，不与 crates/ 共用代码。

被删对象 X：inode 记录（nlink、单元列表），三个数据单元 u0 u1 u2。删除流 D 是 D8 已定项 16 的形态：意图 I（nlink=0、入队）→ 批 B1（释放尾部
K=2 个单元 u1 u2）→ 最后一批 B2（释放剩下的 u0、删 inode 记录、出队）；每一发布 = 计划（读基版本）+ 提交（盲写计划里的值，或先校验读集）。
用户事务 W：覆盖写 X 的第 0 个单元（换成新单元 u3、释放 u0、整条 inode 记录按计划时读到的值写回，nlink 也写回去）。
读集 = X 的 inode 记录版本；O读 的校验：提交时读集版本变了就按现行版本重新计划（E166 的 O读 臂的形态）。

三种提交纪律：
  S    续做独占（Q9 候选甲，D8 已定项 16 第 6 条「每次用户改动先走空」的并行化）：W 要等队列走空才开始计划
  F    并行、提交时不校验（Q9 候选乙，盲写）
  V    并行、提交时校验读集（E166 的 O读）
违例：重复释放（同一单元释放两次）、悬空（inode 记录还指着已释放单元）、泄漏（单元既没释放也没被引用）、复活（意图之后 inode 的 nlink 又被写回 1）。
"""
from itertools import combinations


class Head:
    def __init__(self):
        self.inode = {"nlink": 1, "units": ["u0", "u1", "u2"], "ver": 0}
        self.queue = set()
        self.released = []
        self.allocated = ["u0", "u1", "u2"]
        self.resurrected = False
        self.intent_committed = False


def plan(kind, base):
    inode = base.inode
    if kind == "I":
        return None if inode is None or inode["nlink"] == 0 else {"kind": "I"}
    if kind == "B1":
        return None if inode is None or len(inode["units"]) < 2 else {"kind": "B1", "release": inode["units"][1:], "keep": inode["units"][:1]}
    if kind == "B2":
        return None if inode is None else {"kind": "B2", "release": list(inode["units"])}
    if kind == "W":
        if inode is None or inode["nlink"] == 0:
            return None          # 写入口拒 nlink = 0（D8 已定项 16 第 4 条）
        return {"kind": "W", "release": [inode["units"][0]], "units": ["u3"] + inode["units"][1:], "nlink": 1, "alloc": "u3"}


def apply(p, head):
    if p["kind"] == "I":
        head.inode["nlink"] = 0
        head.queue.add("X")
        head.intent_committed = True
    elif p["kind"] == "B1":
        head.released += p["release"]
        head.inode["units"] = p["keep"]
    elif p["kind"] == "B2":
        head.released += p["release"]
        head.inode = None
        head.queue.discard("X")
    elif p["kind"] == "W":
        head.allocated.append(p["alloc"])
        head.released += p["release"]
        if head.inode is None:
            head.inode = {"nlink": p["nlink"], "units": p["units"], "ver": 0}      # 盲写把已删的记录写了回来
        else:
            if head.intent_committed and p["nlink"] == 1:
                head.resurrected = True
            head.inode.update(nlink=p["nlink"], units=p["units"])
    if head.inode is not None:
        head.inode["ver"] += 1


def verdict(head):
    out = []
    if len(head.released) != len(set(head.released)):
        out.append("重复释放")
    referenced = set(head.inode["units"]) if head.inode else set()
    if referenced & set(head.released):
        out.append("悬空")
    if set(head.allocated) - set(head.released) - referenced:
        out.append("泄漏")
    if head.resurrected:
        out.append("复活")
    return out


def simulate(order, discipline):
    """order：事件序列，元素是 ("D", 步骤名) 或 ("W", 步骤名)；D 的步骤固定为 PI CI P1 C1 P2 C2。"""
    head = Head()
    plans, retries = {}, 0
    for who, step in order:
        if step.startswith("P"):
            kind = {"PI": "I", "P1": "B1", "P2": "B2", "PW": "W"}[step]
            plans[step] = (plan(kind, head), head.inode["ver"] if head.inode else None)
        else:
            key = "P" + step[1:]
            p, base_ver = plans[key]
            if p is None:
                continue
            if discipline == "V" and (head.inode is None or head.inode["ver"] != base_ver):
                kind = p["kind"]
                p = plan(kind, head)                      # 重新计划
                retries += 1
                if p is None:
                    continue
            apply(p, head)
    return verdict(head), retries


def main():
    d_steps = ["PI", "CI", "P1", "C1", "P2", "C2"]
    schedules = []
    for slots in combinations(range(8), 2):          # W 的两步（计划、提交）插进 D 的六步
        order, d_index = [], 0
        for position in range(8):
            if position == slots[0]:
                order.append(("W", "PW"))
            elif position == slots[1]:
                order.append(("W", "CW"))
            else:
                order.append(("D", d_steps[d_index]))
                d_index += 1
        schedules.append(order)
    print(f"D 六步（I、B1、B2 各计划 + 提交）与 W 两步（计划、提交）的全部交错：{len(schedules)} 条")
    for discipline, name in (("F", "并行、提交时不校验"), ("V", "并行、提交时校验读集")):
        bad = {}
        retries_total = 0
        for order in schedules:
            problems, retries = simulate(order, discipline)
            retries_total += retries
            if problems:
                bad[tuple(problems)] = bad.get(tuple(problems), 0) + 1
        print(f"  {name}：违例交错 {sum(bad.values())} 条 {bad}；重新计划合计 {retries_total} 次")
    # 续做独占：W 的计划排在 C2 之后（队列走空之后才开始），其余交错全被排除
    only = [o for o in schedules if o.index(("W", "PW")) > o.index(("D", "C2"))]
    bad_s = sum(1 for o in only if simulate(o, "F")[0])
    print(f"  续做独占：允许的交错 {len(only)} 条（W 的计划在 D 的 C2 之后），违例 {bad_s} 条；"
          f"被排除的 {len(schedules) - len(only)} 条交错对应「用户事务等队列走空」的等待")
    # 无违例的并行交错有多少是靠 W 被拒（nlink = 0）
    refused = sum(1 for o in schedules if plan("W", simulate_head_at(o)) is None)
    print(f"  W 计划时已看到 nlink = 0 而被写入口拒的交错：{refused} 条（D8 已定项 16 第 4 条挡住的那一半）")


def simulate_head_at(order):
    head = Head()
    plans = {}
    for who, step in order:
        if (who, step) == ("W", "PW"):
            return head
        if step.startswith("P"):
            kind = {"PI": "I", "P1": "B1", "P2": "B2"}[step]
            plans[step] = plan(kind, head)
        else:
            p = plans["P" + step[1:]]
            if p is not None:
                apply(p, head)
    return head


if __name__ == "__main__":
    main()
