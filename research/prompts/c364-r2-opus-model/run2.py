#!/usr/bin/env python3
"""C364 第二轮攻方腿：跑搜索与扫描，逐行打印 C364R2RESULT / C364R2NONE / C364R2SCAN。

复跑：cd research/prompts/c364-r2-opus-model && PYTHONDONTWRITEBYTECODE=1 python3 run2.py SELF W3M W3X W4 W5 ATT4 > results-<组>.txt
（每组单独跑、单独重定向；并行进程数取 C364_WORKERS，默认 8；输出顺序与进程数无关）
"""
import os
import sys
from multiprocessing import Pool

from model2 import Geometry, DEFAULT, build, search

# ---------------------------------------------------------------- 几何与初始布局
# GM：两块盘、全池同号（crates 今天的 PoolAllocator），每段 4 槽 × 5 段，根环 K = 3，每次发布 1 个元数据块，
# journal 只留最近 2 次发布的记录（D23 已定项 18：环 196608 条、在飞上限 65536 条 ⇒ 大发布下环里只装得下约 3 次发布，而根环 24 个根）
GM = Geometry(seg_slots=4, nseg=5, devices=2, ring_k=3, meta_per_publish=1, max_user=4, journal_keep=2, mirror=True)
# LM：seg0 = I@0 U@2（槽 1 空）；seg1 = U@4 I@6（槽 7 空）；seg2 = U@8；seg3 = 开放段（M@12，游标 13）；seg4 全空（第一轮 L1）
LM = [("I", 0, 1, 1), ("U", 2, 2, 1), ("U", 4, 2, 1), ("I", 6, 1, 1), ("U", 8, 2, 1), ("M", 12, 1, 1)]
LM_OPENS = ((3, 13), (3, 13))
# LM2：R = seg1 里槽 4–5 空着、U@6 在后——整理自己的目的地可以落回 R 里续做点之前（排除读已发布意图时）
LM2 = [("I", 0, 1, 1), ("U", 2, 2, 1), ("U", 6, 2, 1), ("U", 8, 2, 1), ("M", 12, 1, 1)]
# GX：三块盘、每个单元落其中两块、两份副本槽号可以不同；每段 4 槽 × 4 段；元数据块按 txg 轮流落 (0,1)/(0,2)/(1,2)
GX = Geometry(seg_slots=4, nseg=4, devices=3, ring_k=3, meta_per_publish=1, max_user=3, journal_keep=2, mirror=False)
# LX：X = U 在 (0,4)/(1,6)（R = seg1 在 dev0、dev1 上有副本，dev2 上 seg1 签发时就空）；Y = U 在 (1,8)/(2,8)；
# seg0 在三块盘上都被索引节点占着；元数据在 (0,12)/(1,12)，dev0 / dev1 的开放段 seg3、dev2 没有开放段
LX = [("I", [(0, 0), (1, 0)], 1, 1), ("I", [(1, 1), (2, 0)], 1, 1), ("U", [(0, 4), (1, 6)], 2, 1),
      ("U", [(1, 8), (2, 8)], 2, 1), ("M", [(0, 12), (1, 12)], 1, 1)]
LX_OPENS = ((3, 13), (3, 13), (None, 0))

W3_KINDS = ("c", "c_lost", "c_lost_nowork", "never", "stuck", "rp_ahead", "resurrect", "r3")
S_KINDS = ("a", "b", "b_ac", "b_fn", "b_fn_hz", "blind", "c", "r3")


def fmt_cfg(cfg):
    return ",".join(f"{k}={v}" for k, v in cfg._asdict().items())


def one(job):
    exp, geo_name, layout_name, cfg, alphabet, targets, depth, max_states, kinds = job
    geo = {"GM": GM, "GX": GX}[geo_name]
    layout, opens = {"LM": (LM, LM_OPENS), "LM2": (LM2, LM_OPENS), "LX": (LX, LX_OPENS)}[layout_name]
    init = build(geo, layout, opens=opens)
    found, states, complete = search(geo, cfg, init, set(alphabet), targets, depth, max_states, set(kinds))
    lines = []
    for kind in kinds:
        if kind in found:
            path, detail = found[kind]
            lines.append(f"C364R2RESULT exp={exp} geo={geo_name}/{layout_name} cfg=[{fmt_cfg(cfg)}] "
                         f"alphabet={'+'.join(sorted(alphabet))} witness={kind} len={len(path)} "
                         f"path={' → '.join(path)} detail={detail}")
        else:
            lines.append(f"C364R2NONE exp={exp} geo={geo_name}/{layout_name} cfg=[{fmt_cfg(cfg)}] "
                         f"alphabet={'+'.join(sorted(alphabet))} witness={kind} depth={depth} states={states} "
                         f"complete={complete}")
    return "\n".join(lines)


def jobs_w3(geo_name, layout_name, depth, targets, foreground=True, moves=("unit",), places=("ROOT", "SB", "JEACH", "JCHG")):
    out = []
    # ROOTLOSS 只在新实例的第一个根上发生（16-发布语义.md:293），要先有一次崩溃造出新实例，所以那一组带 CRASH
    recoveries = {"crash": ["CRASH"], "sblag": ["CRASH_SBLAG"], "rootloss": ["CRASH", "ROOTLOSS"], "rollback": ["ROLLBACK"]}
    for place in places:
        for rec_name, rec in recoveries.items():
            if rec_name == "sblag" and place != "SB":
                continue
            for p_name, p, wrap in (("P甲", "A", False), ("P甲+绕回", "A", True), ("P乙", "B", False)):
                for move in moves:
                    cfg = DEFAULT._replace(R="A", S="C", E="rec", SP="AC", place=place, P=p, wrap=wrap, move=move)
                    alphabet = ["SIGN", "BATCH", "PUBLISH"] + (["WRITE", "DELETE"] if foreground else []) + rec
                    out.append((f"W3-{place}-{rec_name}-{p_name}-{move}-{'fg' if foreground else 'nofg'}", geo_name,
                                layout_name, cfg, alphabet, targets, depth, 1500000, W3_KINDS))
    return out


def jobs_w4(depth):
    out = []
    for r in "AB":
        for s in "AB":
            for e in ("rec", "occ"):
                for move in ("copy", "unit"):
                    for sp in ("AC", "SD"):
                        cfg = DEFAULT._replace(R=r, S=s, E=e, SP=sp, move=move, P="B", place="ROOT")
                        out.append((f"W4-R{r}-S{s}-{e}-{move}-{sp}", "GX", "LX", cfg,
                                    ["SIGN", "BATCH", "PUBLISH", "WRITE", "DELETE"], [1], depth, 4000000, S_KINDS))
    return out


def jobs_w5(depth):
    out = []
    alphabets = {"S-fg": ["SIGN", "BATCH", "DELETE", "PUBLISH", "WRITE"],
                 "S-freeonly": ["SIGN", "BATCH", "DELETE", "PUBLISH"],
                 "S-nofg": ["SIGN", "BATCH", "PUBLISH", "CRASH"]}
    for name, alphabet in alphabets.items():
        for s in "ABC":
            for e in ("rec", "occ"):
                cfg = DEFAULT._replace(R="A", S=s, E=e, SP="AC", P="B", place="ROOT", move="unit")
                out.append((f"W5-{name}-S{s}-{e}-AC", "GM", "LM", cfg, alphabet, [1], depth, 4000000, S_KINDS))
    return out


def jobs_att4(depth, foreground=True, geos=(("GM", "LM2", [1]), ("GM", "LM", [1]), ("GX", "LX", [1]))):
    out = []
    for geo_name, layout_name, targets in geos:
        for excl in ("inflight", "published"):
            for r in ("A", "B"):
                if geo_name == "GM" and r == "B":
                    continue
                for p_name, p, wrap in (("P甲", "A", False), ("P甲+绕回", "A", True), ("P乙", "B", False)):
                    for move in (("unit",) if geo_name == "GM" else ("copy", "unit")):
                        cfg = DEFAULT._replace(R=r, S="C", E="rec", SP="AC", place="ROOT", P=p, wrap=wrap,
                                               exclusion=excl, move=move)
                        alphabet = ["SIGN", "BATCH", "PUBLISH", "CRASH", "ROLLBACK"] + (["WRITE", "DELETE"] if foreground else [])
                        out.append((f"ATT4-{excl}-R{r}-{p_name}-{move}-{'fg' if foreground else 'nofg'}", geo_name,
                                    layout_name, cfg, alphabet,
                                    targets, depth, 1500000, W3_KINDS))
    return out


GROUPS = {
    "W3M": lambda: jobs_w3("GM", "LM", 7, [1]),
    "W3MD": lambda: jobs_w3("GM", "LM", 12, [1], foreground=False, places=("ROOT", "SB")),
    "W3X": lambda: jobs_w3("GX", "LX", 6, [1], moves=("copy", "unit")),
    "W4": lambda: jobs_w4(7),
    "W5": lambda: jobs_w5(8),
    "ATT4": lambda: jobs_att4(7),
    "ATT4D": lambda: jobs_att4(12, foreground=False, geos=(("GM", "LM2", [1]), ("GM", "LM", [1]))),
}


def run_group(name):
    jobs = GROUPS[name]()
    workers = int(os.environ.get("C364_WORKERS", "8"))
    with Pool(workers) as pool:
        for text in pool.imap(one, jobs):
            print(text)
            sys.stdout.flush()


if __name__ == "__main__":
    wanted = sys.argv[1:]
    for group in wanted:
        if group == "SELF":
            from selftest2 import selftest
            if not selftest():
                sys.exit(1)
        elif group == "SCAN":
            from scan2 import scan_all
            scan_all()
        else:
            run_group(group)
