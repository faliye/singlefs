#!/usr/bin/env python3
"""C9 补一格（接 b4、b4b，不改它们）：串行提交下这一问还剩什么。
串行 = 每次发布的「计划 + 提交」不可分、发布一个接一个（b4b 的 A 臂）。W 插进 D 的三次发布（意图 I、批 B1、末批 B2）的 4 个位置各跑一遍，三种规则：
  ① 有 D8 已定项 16 第 4 条（写入口拒 nlink = 0 的 inode；b4.plan("W") 的现行写法）、没有第 6 条
  ② 没有第 4 条、有第 6 条（W 轮到时队列非空就先把 D 的剩余各批做完再做 W；意图还没提交则没有队列可走）
  ③ 两条都没有
目的：看串行下 Q9 的两个翻面观测（用户事务写进正在删的 inode；最后一批与意图不同事务）还成不成立。
"""
from itertools import permutations

import b4_c9_delete_interleave as b4

original_plan = b4.plan


def plan_without_reject(kind, base):
    inode = base.inode
    if kind == "W" and inode is not None:
        return {"kind": "W", "release": [inode["units"][0]], "units": ["u3"] + inode["units"][1:], "nlink": 1, "alloc": "u3"}
    return original_plan(kind, base)


def serial_run(sequence, drain_first):
    head = b4.Head()
    pending = ["I", "B1", "B2"]                      # D 还没做的发布，按次序
    for kind in sequence:
        if kind == "W" and drain_first and head.queue:
            while pending:                           # 队列非空：先把 D 的剩余各批做完
                step = pending.pop(0)
                p = b4.plan(step, head)
                if p is not None:
                    b4.apply(p, head)
        elif kind != "W" and kind in pending:
            pending.remove(kind)
        elif kind != "W":
            continue                                 # 这一步已被走空做过了
        p = b4.plan(kind, head)
        if p is not None:
            b4.apply(p, head)
    return b4.verdict(head)


def main():
    seqs = sorted(s for s in permutations(["I", "B1", "B2", "W"]) if s.index("I") < s.index("B1") < s.index("B2"))
    print(f"串行下 W 插进 D 三次发布的位置：{len(seqs)} 种")
    for label, plan_fn, drain in (("① 有第 4 条、无第 6 条", original_plan, False),
                                   ("② 无第 4 条、有第 6 条（W 先走空）", plan_without_reject, True),
                                   ("③ 两条都没有", plan_without_reject, False)):
        b4.plan = plan_fn
        bad = [s for s in seqs if serial_run(s, drain)]
        print(f"  {label}：违例 {len(bad)} 种 {[ '-'.join(s) for s in bad]}")
    b4.plan = original_plan


if __name__ == "__main__":
    main()
