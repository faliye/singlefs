#!/usr/bin/env python3
"""J1 的纸面模型：把 crash_judge_checker.wgsl 的 w_walk_instance_table_chain（第 1589 行起，冻结副本）的控制流逐支抄成 Python，
再写一份 CPU 版（walk.rs walk_instance_table_chain，没有片数上限），在链长 / 断点 / 重复片上对比。
只模拟「走没走到末片」「判不判不了」，不模拟行与校验和。不读仓、不依赖 SINGLEFS_REPO。

一条链用页描述表示：pages[k] 是第 k 片的状态，取值
  "ok_next"   读得出、身份对、链指针记录说有下一片（合法）
  "ok_last"   读得出、身份对、链指针记录说没有下一片（合法）
  "bad_chain" 读得出、身份对、链指针记录不合法
  "unreadable" 两份都读不出
  "bad_identity" 读得出但身份四元组不对
  "visited"   指着前一条根已经走过的单元
"""
GPU_MOST_PAGES = 256


def gpu_walk(pages):
    """返回 (走到末片, 判不了)。逐支对应 wgsl：每个提前出口都在 page 加一之前 break。"""
    reached_last = False
    page = 0
    while page < GPU_MOST_PAGES:           # for (; page < INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED; page = page + 1u)
        state = pages[page] if page < len(pages) else "unreadable"
        if state == "unreadable":
            break
        if state == "visited":
            break
        if state == "bad_identity":
            break
        chain = {"ok_next": 1, "ok_last": 0, "bad_chain": 2}[state]
        if chain == 0:
            reached_last = True
        if chain != 1:
            break
        page += 1
    undecidable = page == GPU_MOST_PAGES    # if (page == INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED)
    return reached_last, undecidable


def cpu_walk(pages):
    reached_last = False
    page = 0
    while True:
        state = pages[page] if page < len(pages) else "unreadable"
        if state in ("unreadable", "visited", "bad_identity"):
            break
        if state == "ok_last":
            reached_last = True
            break
        if state == "bad_chain":
            break
        page += 1
    return reached_last


def chain(n, last="ok_last"):
    return ["ok_next"] * (n - 1) + [last]


def main():
    cases = []
    for n in (1, 2, 255, 256, 257, 258, 1000):
        cases.append((f"chain-{n}-pages-valid", chain(n)))
    for n in (255, 256, 257):
        cases.append((f"chain-{n}-pages-last-record-invalid", chain(n, "bad_chain")))
    for n in (255, 256, 257, 300):
        for k in (1, 254, 255, 256, 257):
            if k < n:
                p = chain(n)
                p[k] = "unreadable"
                cases.append((f"chain-{n}-pages-page-{k}-unreadable", p))
    for n in (256, 257, 300):
        for k in (1, 255, 256):
            if k < n:
                p = chain(n)
                p[k] = "visited"
                cases.append((f"chain-{n}-pages-page-{k}-already-visited", p))
    violations = 0
    for name, pages in cases:
        g_last, g_undec = gpu_walk(pages)
        c_last = cpu_walk(pages)
        # 「判了的同判」：判得了就必须与 CPU 同；判不了的回落 CPU，自然同判
        decided_same = g_undec or g_last == c_last
        if not decided_same:
            violations += 1
        print(f"{name} gpu_reached_last={int(g_last)} gpu_undecidable={int(g_undec)} cpu_reached_last={int(c_last)} "
              f"decided_same_or_left_to_cpu={int(decided_same)}")
    print(f"cases={len(cases)} violations={violations}")


if __name__ == "__main__":
    main()
