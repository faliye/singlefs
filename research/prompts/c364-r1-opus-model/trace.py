#!/usr/bin/env python3
"""把一条最短序列逐步重放，打印每一步之后两盘全空段数（occ / rec 两种定义）、R 内分配记录、停机谓词与目标态。

复跑：python3 trace.py > traces.txt
"""
from model import DEFAULT, build, events, counts, units_in_r, target_reached, predicate
from run import G1, G2, L1, L2, L3

TRACES = [
    ("Q1 前台释放在 R 外腾出全空段（S甲，停机即删意图，rec）", G1, L1, (3, 13),
     DEFAULT._replace(S="A", SP="SD", E="rec"), {"SIGN", "BATCH", "DELETE", "PUBLISH", "WRITE"},
     ["SIGN(R=('A', 1),c0=2)", "DELETE I@0", "DELETE U@2", "BATCH"]),
    ("Q1 同一序列（S丙）", G1, L1, (3, 13),
     DEFAULT._replace(S="C", SP="SD", E="rec"), {"SIGN", "BATCH", "DELETE", "PUBLISH", "WRITE"},
     ["SIGN(R=('A', 1),c0=2)", "DELETE I@0", "DELETE U@2", "BATCH", "BATCH"]),
    ("共用序列：不带前台，搬完 R（S甲，occ）", G1, L1, (3, 13),
     DEFAULT._replace(S="A", SP="AC", E="occ"), {"SIGN", "BATCH", "PUBLISH"},
     ["SIGN(R=('A', 1),c0=2)", "BATCH", "BATCH", "PUBLISH(txg=2)", "PUBLISH(txg=3)", "PUBLISH(txg=4)",
      "PUBLISH(txg=5)"]),
    ("共用序列：不带前台，搬完 R（S丙，occ）", G1, L1, (3, 13),
     DEFAULT._replace(S="C", SP="AC", E="occ"), {"SIGN", "BATCH", "PUBLISH"},
     ["SIGN(R=('A', 1),c0=2)", "BATCH", "BATCH"]),
    ("P甲 续做点越过没搬成的落点、扫到尾就删", G2, L2, (3, 15),
     DEFAULT._replace(S="C", SP="AC", P="A_adv"), {"SIGN", "BATCH", "PUBLISH", "WRITE"},
     ["SIGN(R=('A', 1),c0=2)", "BATCH", "WRITE@16", "PUBLISH(txg=2)", "BATCH"]),
    ("同一序列，P甲 只越过搬成的落点", G2, L2, (3, 15),
     DEFAULT._replace(S="C", SP="AC", P="A_safe"), {"SIGN", "BATCH", "PUBLISH", "WRITE"},
     ["SIGN(R=('A', 1),c0=2)", "BATCH", "WRITE@16", "PUBLISH(txg=2)", "BATCH"]),
    ("c0：切换重做把在飞分配代改成新 txg，c0 照旧", G1, L3, (3, 13),
     DEFAULT._replace(S="C", SP="AC"), {"WRITE", "SIGN", "SWITCH_KEEP"},
     ["WRITE@6", "SIGN(R=('A', 1),c0=2)", "SWITCH_KEEP(在飞 checkpoint 2 按 3 重做)"]),
    ("c0 取上一次已发布的 txg", G1, L3, (3, 13),
     DEFAULT._replace(S="C", SP="AC", c0="published"), {"WRITE", "SIGN"},
     ["WRITE@6", "SIGN(R=('A', 1),c0=1)"]),
    ("Q1 序列，S甲，谓词只在意图照目标态删的那一刻判（AC）", G1, L1, (3, 13),
     DEFAULT._replace(S="A", SP="AC", E="rec"), {"SIGN", "BATCH", "DELETE", "PUBLISH", "WRITE"},
     ["SIGN(R=('A', 1),c0=2)", "DELETE I@0", "DELETE U@2", "BATCH", "BATCH"]),
    ("P甲（只越过搬成的）+ 意图不随根回退", G1, L1, (3, 13),
     DEFAULT._replace(S="C", SP="AC", P="A_safe", intent_rooted=False), {"SIGN", "BATCH", "PUBLISH", "ROLLBACK"},
     ["SIGN(R=('A', 1),c0=2)", "PUBLISH(txg=2)", "BATCH", "PUBLISH(txg=3)", "ROLLBACK(→根 2,新 txg=5)", "BATCH",
      "BATCH"]),
    ("同一序列，意图随根回退", G1, L1, (3, 13),
     DEFAULT._replace(S="C", SP="AC", P="A_safe", intent_rooted=True), {"SIGN", "BATCH", "PUBLISH", "ROLLBACK"},
     ["SIGN(R=('A', 1),c0=2)", "PUBLISH(txg=2)", "BATCH", "PUBLISH(txg=3)", "ROLLBACK(→根 2,新 txg=5)", "BATCH",
      "BATCH"]),
    ("R丙 起点落在两槽单元中间（R=[5,9)）", G1, L1, (3, 13),
     DEFAULT._replace(R="C", S="C", SP="AC"), {"SIGN", "BATCH", "DELETE"},
     ["SIGN(R=('C', 5, 4),c0=2)", "DELETE I@6", "BATCH"], [("C", 5, 4)]),
]


TRACE_R = [("A", 1)]


def show(geo, cfg, st, label):
    if st is None:
        print(f"  {label:<44} → 终止（本步即 (d)）")
        return
    intent = st.intent
    rspec = intent[0] if intent else (st.ghost[0] if st.ghost else TRACE_R[0])
    in_r = [(u[0], u[4][0][1], u[1]) for u in units_in_r(geo, st, rspec)]
    line = (f"  {label:<44} txg={st.txg} occ={counts(geo, st, 'occ')} rec={counts(geo, st, 'rec')} "
            f"R内活记录(种类,槽,分配代)={in_r} 意图={'有' if intent else '无'}")
    if intent:
        line += (f" c0={intent[1]} segs0={intent[2]} 续做点={intent[3]} "
                 f"谓词={predicate(geo, cfg, st, rspec, intent[2])} 目标态={target_reached(geo, st, rspec, intent[1])}")
    elif st.ghost:
        line += f" 已删意图的目标态={target_reached(geo, st, rspec, st.ghost[1])}"
    print(line)


for title, geo, layout, (open_seg, cursor), cfg, alphabet, path, *rest in TRACES:
    targets = rest[0] if rest else [1]
    TRACE_R[0] = targets[0] if isinstance(targets[0], tuple) else ("A", targets[0])
    print(f"== {title}")
    st = build(geo, layout, open_seg=open_seg, cursor=cursor)
    show(geo, cfg, st, "初始")
    for label in path:
        for event_label, new, wits in events(geo, cfg, st, alphabet, targets):
            if event_label == label:
                st = new
                break
        else:
            raise SystemExit(f"序列里的一步在这个状态下不可达：{label}")
        show(geo, cfg, st, label)
        for kind, detail in wits:
            print(f"      ↑ 这一步判出 {kind}：{detail}")
        if st is None:
            break
