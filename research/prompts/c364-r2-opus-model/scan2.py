#!/usr/bin/env python3
"""W5 的几何敏感性：64 槽段、两块盘全池同号，谓词只在意图照目标态删的那一刻判（AC），全空段数按记录（rec）或按占用（occ）。

每格（参数网格同第一轮 scan64，256 格）：
- 第 1 条意图删掉那一刻，两种定义下「全空段数 − segs0」（S甲 / S乙 在两盘同号时同一个条件），S丙 在那一刻判不判真、中不中 (a)；
- 之后只发布、不再整理，K 轮 defer 排空之后，同一个差曾不曾 ≥ 1（谓词不再判，这就是 b_fn_hz）；
- 多轮：谓词判假就签下一条意图（最低的、有活副本、不是开放段的段），最多 5 条，记第几条的删意图那一刻谓词第一次判真；
- 对照臂：同一布局只发布、不整理，同样多次发布里差的区间。
"""
import sys

from model2 import Geometry, DEFAULT, build, batch, publish, crash_recover, counts, copies_in_r, r_empty

CFG = DEFAULT._replace(R="A", S="C", E="rec", SP="AC", P="B", place="ROOT", move="unit")
MAX_ROUNDS = 5


def layout(holes, user_units, index_nodes, left):
    out = [("U", slot, 2, 1) for slot in range(0, 64 - 2 * holes, 2)]
    out += [("U", 64 + 2 * i, 2, 1) for i in range(user_units)]
    out += [("I", 64 + 2 * user_units + i, 1, 1) for i in range(index_nodes)]
    out += [("I", slot, 1, 1) for slot in range(128, 192 - left)]
    return out


def next_target(geo, st):
    open_seg = st.opens[0][0]
    for seg in range(geo.nseg):
        if seg == open_seg:
            continue
        if copies_in_r(geo, st.units, ("A", seg)):
            return seg
    return None


def sign(geo, st, seg):
    rspec = ("A", seg)
    now_rec, now_occ = counts(geo, st, "rec"), counts(geo, st, "occ")
    sign_devs = frozenset(k[0] for k, _u in copies_in_r(geo, st.units, rspec))
    intent = (rspec, st.txg, tuple((d, now_rec[d]) for d in range(geo.devices)), None, (now_rec, sign_devs))
    return st._replace(intent=intent, run="running"), now_rec[0], now_occ[0]


def run_intent(geo, st, per_publish, crash_mid):
    publishes, crashed = 0, False
    while st.intent is not None and publishes < 400:
        for _ in range(per_publish):
            if st.intent is not None:
                st, _w = batch(geo, CFG, st)
        if st.intent is None:
            break
        nxt = publish(geo, CFG, st)
        if nxt is None:
            return None, publishes
        st, publishes = nxt, publishes + 1
        if crash_mid and not crashed and publishes == 1:
            st, crashed = crash_recover(geo, CFG, st), True
    return st, publishes


def cell(meta, per_publish, left, holes, user_units, index_nodes, crash_mid):
    geo = Geometry(seg_slots=64, nseg=6, devices=2, ring_k=3, meta_per_publish=meta, max_user=10 ** 6,
                   journal_keep=2, mirror=True)
    lay = layout(holes, user_units, index_nodes, left)
    st = build(geo, lay, opens=((2, 192 - left), (2, 192 - left)))
    st, base_rec, base_occ = sign(geo, st, 1)
    st, publishes = run_intent(geo, st, per_publish, crash_mid)
    if st is None:
        return f"enospc_round=1 publishes={publishes}"
    d_rec = counts(geo, st, "rec")[0] - base_rec
    d_occ = counts(geo, st, "occ")[0] - base_occ
    horizon = 2 * (64 // meta) + geo.ring_k
    probe, hz_occ, hz_rec, r_occ_empty_at = st, d_occ, d_rec, None
    for step in range(horizon):
        nxt = publish(geo, CFG, probe)
        if nxt is None:
            break
        probe = nxt
        hz_occ = max(hz_occ, counts(geo, probe, "occ")[0] - base_occ)
        hz_rec = max(hz_rec, counts(geo, probe, "rec")[0] - base_rec)
        if r_occ_empty_at is None and r_empty(geo, probe, ("A", 1), "occ"):
            r_occ_empty_at = step + 1
    # 多轮：从第 1 条删掉那一刻接着签
    fire = {"rec": 1 if d_rec >= 1 else None, "occ": 1 if d_occ >= 1 else None}
    multi, rounds, enospc_round = st, 1, None
    while (fire["rec"] is None or fire["occ"] is None) and rounds < MAX_ROUNDS:
        seg = next_target(geo, multi)
        if seg is None:
            break
        multi, b_rec, b_occ = sign(geo, multi, seg)
        multi, _p = run_intent(geo, multi, per_publish, 0)
        rounds += 1
        if multi is None:
            enospc_round = rounds
            break
        if fire["rec"] is None and counts(geo, multi, "rec")[0] - b_rec >= 1:
            fire["rec"] = rounds
        if fire["occ"] is None and counts(geo, multi, "occ")[0] - b_occ >= 1:
            fire["occ"] = rounds
    control = build(geo, lay, opens=((2, 192 - left), (2, 192 - left)))
    c_base = counts(geo, control, "occ")[0]
    c_lo = c_hi = 0
    for _ in range(publishes + horizon):
        nxt = publish(geo, CFG, control)
        if nxt is None:
            break
        control = nxt
        delta = counts(geo, control, "occ")[0] - c_base
        c_lo, c_hi = min(c_lo, delta), max(c_hi, delta)
    return (f"publishes={publishes} d_rec_at_delete={d_rec} d_occ_at_delete={d_occ} "
            f"Sjia_fires_rec={d_rec >= 1} Sjia_fires_occ={d_occ >= 1} Sbing_a_rec={d_rec <= 0} Sbing_a_occ={d_occ <= 0} "
            f"hz_max_rec={hz_rec} hz_max_occ={hz_occ} b_fn_hz_rec={d_rec < 1 <= hz_rec} b_fn_hz_occ={d_occ < 1 <= hz_occ} "
            f"R_occ_empty_after_publishes={r_occ_empty_at} fire_round_rec={fire['rec']} fire_round_occ={fire['occ']} "
            f"rounds_run={rounds} enospc_round={enospc_round} control_occ=[{c_lo},{c_hi}]")


def scan_all():
    for meta in (1, 7):
        for per_publish in (1, 8):
            for left in (8, 56):
                for holes in (0, 16):
                    for user_units in (0, 8, 24):
                        for index_nodes in (0, 8, 16):
                            for crash_mid in (0, 1):
                                if user_units == 0 and index_nodes == 0:
                                    continue
                                text = cell(meta, per_publish, left, holes, user_units, index_nodes, crash_mid)
                                print(f"C364R2SCAN meta={meta} batches_per_publish={per_publish} open_left={left} "
                                      f"holes={holes} user_units_in_R={user_units} index_nodes_in_R={index_nodes} "
                                      f"crash_mid={crash_mid} {text}")
                                sys.stdout.flush()


if __name__ == "__main__":
    scan_all()
