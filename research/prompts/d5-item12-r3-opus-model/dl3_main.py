#!/usr/bin/env python3
# D5（快照 / 空间记账机制） 未定项 12 第三轮攻方腿模型入口：python3 -B dl3_main.py <mode>
#   replay：固定几段历史喂给全部变体；bfs-batch / bfs-heads / bfs-crash / bfs-split：各攻击面上的最短历史；
#   fuzz：随机历史；minimize：各变体第一例缩到局部极小；geometry：U2 的改键 / 批数；lookup：U2 找下一侧 / 上一个快照要走过的树表条目数。
import sys, json
import dl2 as base
import dl3_search as search
from dl3_world import new_state, step

R3 = [("REF", {}), ("BING_R", {}), ("YI", {}), ("BING_T", {})]
KNOWN_BAD = [("BING_R", {"hi": "inf"}), ("YI", {"order": "any"}), ("BING_T", {"order": "any"})]   # 第二轮已知会中的对照
VARIANTS3 = R3 + [("BING_R", {"order": "lineage"}), ("BING_R", {"hi": "pub"})] + KNOWN_BAD + \
    [(name, {"inherit": mode}) for mode in ("nextpub", "nextpub_lost") for name in ("BING_R", "YI", "BING_T")]
SPLIT = [(name, {"inherit": mode}) for mode in ("nextpub", "nextpub_lost") for name in ("BING_R", "YI", "BING_T")]
LIM_SNAP = {"blocks": 2, "heads": 1, "snaps": 3, "txg": 4, "pss": True, "batch_ops": [("BS", 9)]}
LIM_SAFE = {"blocks": 2, "heads": 1, "snaps": 3, "txg": 4, "pss": True, "batch_ops": [("BA", 9), ("BG", 9)]}
LIM_HEADS = {"blocks": 2, "heads": 2, "snaps": 3, "txg": 4, "clone": True, "psc": True, "pss": True,
             "batch_ops": [("BA", 9)]}
LIM_CRASH = {"blocks": 2, "heads": 1, "snaps": 2, "txg": 5, "crash": True, "rbc": True, "pf": True}
LIM_SPLIT = {"blocks": 2, "heads": 1, "snaps": 2, "txg": 4, "crash": True}
FUZZ3 = {"blocks": 30, "heads": 4, "snaps": 6, "txg": 150, "clone": True, "psc": True, "pss": True, "crash": True,
         "rbc": True, "pf": True, "batch_ops": [("BA", 1), ("BA", 9), ("BS", 1), ("BS", 9), ("BG", 9)]}
FUZZ3_SEQ = dict(FUZZ3, batch_ops=[("BA", 1), ("BA", 9), ("BG", 9)])

H1 = [("PS", 11), ("W", 11, 1), ("PS", 11), ("K", 11, 0), ("D", 13), ("D", 12)]
H2 = [("W", 11, 1), ("W", 11, 1), ("PS", 11), ("C", 12), ("K", 11, 0), ("W", 13, 1), ("PS", 13), ("K", 13, 2),
      ("K", 13, 1), ("PS", 11), ("K", 11, 1), ("D", 14), ("D", 15), ("X", 13), ("D", 12), ("B", 2), ("B", 1), ("B", 0)]
H3 = [("W", 11, 1), ("PS", 11), ("W", 11, 1), ("PS", 11), ("K", 11, 1), ("D", 13), ("P",), ("PS", 11), ("K", 11, 0),
      ("RBC",), ("PS", 11), ("K", 11, 0), ("B", 0)]
H4 = [("W", 11, 1), ("PS", 11), ("W", 11, 1), ("PS", 11), ("K", 11, 1), ("D", 13), ("PF", "PS", 11), ("K", 11, 0),
      ("B", 0)]
H5 = [("W", 11, 1), ("PS", 11), ("D", 12), ("PS", 11), ("K", 11, 0)]
HISTORIES3 = {
    "H0 第二轮 2.1 那段（丙-区间上界取 ∞ 的阳性对照）": [("W", 11, 1), ("PS", 11), ("D", 12), ("PS", 11), ("K", 11, 0), ("B", 0)],
    "H1 同一个头上两个级联区间重叠，一批按批开始时的状态计划、施加时不复核": H1 + [("BS", 9)],
    "H1b 同一段历史，一批顺序施加": H1 + [("BA", 9)],
    "H1c 同一段历史，一批计划后施加、复核条目还在": H1 + [("BG", 9)],
    "H2 两个头的意图交错；克隆头的级联没做完就销毁克隆头、随即销毁 origin": H2,
    "H3 意图写在 checkpoint 3 之后回退、按候选根重发 txg 与树 ID": H3,
    "H3b 同一段历史，回退按全部根取号": [("RB",) if op == ("RBC",) else op for op in H3],
    "H4 写意图的 checkpoint 根槽写失败、推进一格再发": H4,
    "H5 prev 分到下一次发布才传": H5,
    "H5b 同一段历史，发布之后崩溃挂回": H5[:4] + [("CR",)] + H5[4:],
}


def geometry(shape, n, arm_specs):
    state = new_state(arm_specs, keep_ring=False)
    for op in base.geometry_ops(shape, n):
        step(state, op)
    search.drain(state, None)
    rows = []
    for (name, cfg), arm in zip(arm_specs, state["arms"]):
        over, double, leak = base.oracle(state["world"], arm)
        rows.append(dict(shape=shape, n=n, arm=search.label(name, cfg), over=len(over), double=double,
                         leak=len(leak), **arm.m))
    return rows


def lookup_rows():
    """树表按树 ID 升序（D8 已定项 8）。20 轮，每轮每个头各建一个快照，树 ID 依次发；头 0 在第 5 轮之后歇 gap 轮。
    销毁头 0 第 5 轮建的快照 S：往后找头 0 的下一个快照（下一侧）要走过几条、往前找上一个（丙′ 写意图时的目的段）要走过几条；
    头 0 最后一个快照要确认「下一侧是头」时往后走到表尾要走过几条。"""
    rows = []
    for heads in (1, 2, 8, 64, 1024):
        for gap in (0, 10):
            table = [head for round_index in range(20) for head in range(heads)
                     if not (head == 0 and 5 < round_index <= 5 + gap)]
            mine = [position for position, head in enumerate(table) if head == 0]
            s_at = mine[5]
            forward = mine[6] - s_at - 1
            backward = s_at - mine[4] - 1
            rows.append(dict(heads=heads, gap=gap, table=len(table), forward=forward, backward=backward,
                             latest_to_end=len(table) - mine[-1] - 1))
    return rows


def main():
    mode, rows = sys.argv[1], []

    def emit(section, payload):
        rows.append(1)
        print(json.dumps({"section": section, **payload}, ensure_ascii=False, default=str), flush=True)
    if mode == "replay":
        for title, path in HISTORIES3.items():
            emit("history", {"title": title, "path": path, "arms": search.replay_summary(VARIANTS3, path)})
            for line in search.trace(R3, path):
                emit("trace", {"title": title, "line": line})
    elif mode == "bfs-batch":
        for name, cfg in [("BING_R", {}), ("YI", {}), ("BING_T", {}), ("BING_R", {"order": "lineage"})]:
            emit("bfs-snap", search.bfs_shortest(name, cfg, LIM_SNAP, 8))
        for name, cfg in R3[1:]:
            emit("bfs-safe", search.bfs_shortest(name, cfg, LIM_SAFE, 8))
    elif mode == "bfs-heads":
        for name, cfg in R3[1:]:
            emit("bfs-heads", search.bfs_shortest(name, cfg, LIM_HEADS, 8, budget=1000000))
    elif mode == "bfs-heads-deep":      # 两个头、深度 11：两个头各挂一个未完成意图再加杀块要 10 步上下，深度 8 够不着
        for name, cfg in R3[1:]:
            emit("bfs-heads-deep", search.bfs_shortest(name, cfg, LIM_HEADS, 11, budget=6000000))
        for name, cfg in [("YI", {}), ("BING_T", {}), ("BING_R", {"order": "lineage"})]:
            limits = dict(LIM_HEADS, batch_ops=[("BS", 9)])
            emit("bfs-heads-deep-snap", search.bfs_shortest(name, cfg, limits, 11, budget=6000000))
    elif mode == "bfs-crash-deep":      # 意图活过一次回退再建快照、再杀块要 12 步上下，深度 8 够不着
        for name, cfg in R3[1:]:
            emit("bfs-crash-deep", search.bfs_shortest(name, cfg, LIM_CRASH, 11, budget=6000000))
    elif mode == "bfs-crash":
        for name, cfg in R3[1:] + [("BING_R", {"hi": "pub"})]:
            emit("bfs-crash", search.bfs_shortest(name, cfg, LIM_CRASH, 8, budget=1000000))
    elif mode == "bfs-split":
        for name, cfg in SPLIT:
            emit("bfs-split", search.bfs_shortest(name, cfg, LIM_SPLIT, 7))
    elif mode == "fuzz":
        emit("fuzz", {"seeds": 400, "length": 200, "limits": "FUZZ3", "result": search.fuzz(VARIANTS3, 400, 200, FUZZ3)})
        specs = R3 + [("BING_R", {"order": "lineage"})]
        emit("fuzz-seq", {"seeds": 400, "length": 200, "limits": "FUZZ3_SEQ", "result": search.fuzz(specs, 400, 200, FUZZ3_SEQ)})
    elif mode == "minimize":
        cases = [(("BING_R", {}), FUZZ3), (("BING_R", {"hi": "pub"}), FUZZ3_SEQ)] + [(spec, FUZZ3_SEQ) for spec in SPLIT]
        for spec, limits in cases:
            for seed in range(400):
                ops = search.record([spec], seed, 200, limits)
                result = search.verdict(spec, ops, limits)
                if result and any(result):
                    kind = result.index(True)
                    small = search.minimize(spec, ops, lambda flags, k=kind: flags[k], limits)
                    emit("minimized", {"arm": search.label(*spec), "seed": seed, "recorded": len(ops),
                                       "kind": ["over", "double", "leak"][kind], "path": small,
                                       "all_variants": search.replay_summary(VARIANTS3, small)})
                    break
    elif mode == "geometry":
        for shape in ("G2", "G3", "G4", "G5", "G6"):
            for row in geometry(shape, 1000, R3):
                emit("geometry", row)
    elif mode == "lookup":
        for row in lookup_rows():
            emit("lookup", row)
    else:
        raise SystemExit("mode ∈ replay / bfs-batch / bfs-heads / bfs-crash / bfs-split / fuzz / minimize / geometry / lookup")
    print(json.dumps({"section": "END", "rows": len(rows)}))


if __name__ == "__main__":
    main()
