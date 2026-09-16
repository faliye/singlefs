#!/usr/bin/env python3
"""C364 第一轮攻方腿：跑全部搜索与扫描，逐行打印 C364RESULT / C364NONE / C364SCAN。

复跑：cd research/prompts/c364-r1-opus-model && python3 run.py > results.txt
只跑某一组：python3 run.py S R P C0 RB G64 FUT SELF（任选）
"""
import sys
from model import (Geometry, DEFAULT, build, search, counts, predicate, segs0_value, publish, crash, events,
                   batch, units_in_r, target_reached, State)

KINDS = ("a", "b", "c", "c_overlap", "d", "R3", "a_prime", "stall", "stuck")
G1 = Geometry(seg_slots=4, nseg=5, devices=2, ring_k=3, meta_per_publish=1, max_user=4)
# L1：seg0 = I@0 U@2（槽 1 空）；seg1 = U@4 I@6（槽 7 空）；seg2 = U@8；seg3 = 开放段（M@12，游标 13）；seg4 全空
L1 = [("I", 0, 1, 1), ("U", 2, 2, 1), ("U", 4, 2, 1), ("I", 6, 1, 1), ("U", 8, 2, 1), ("M", 12, 1, 1)]
# L3：seg1 只有 U@4，最低的空偶数槽对就在 R 里（槽 6）——c0 那一格要前台能写进 R
L3 = [("I", 0, 1, 1), ("U", 2, 2, 1), ("U", 4, 2, 1), ("U", 8, 2, 1), ("I", 10, 1, 1), ("M", 12, 1, 1)]
G2 = Geometry(seg_slots=4, nseg=5, devices=2, ring_k=3, meta_per_publish=1, max_user=5)
# L2：盘紧——seg0 = U@0 U@2；seg1 = U@4 I@6；seg2 = U@8（槽 10–11 空）；seg3 = I@12 I@13 M@14，开放段游标 15；seg4 全空。
# 两次发布之后 seg4 成了开放段，用户数据只剩槽 10–11 一对
L2 = [("U", 0, 2, 1), ("U", 2, 2, 1), ("U", 4, 2, 1), ("I", 6, 1, 1), ("U", 8, 2, 1),
      ("I", 12, 1, 1), ("I", 13, 1, 1), ("M", 14, 1, 1)]


def fmt_cfg(cfg):
    return ",".join(f"{k}={v}" for k, v in cfg._asdict().items())


def report(exp, geo, cfg, init, alphabet, targets, depth, max_states=600000, kinds=KINDS):
    found, states, reached_depth, complete = search(geo, cfg, init, alphabet, targets, depth, max_states)
    for kind in kinds:
        if kind in found:
            path, detail = found[kind]
            print(f"C364RESULT exp={exp} cfg=[{fmt_cfg(cfg)}] witness={kind} len={len(path)} "
                  f"path={' → '.join(path)} detail={detail}")
        else:
            print(f"C364NONE exp={exp} cfg=[{fmt_cfg(cfg)}] witness={kind} depth={depth} states={states} "
                  f"complete={complete}")
    sys.stdout.flush()


def exp_s():
    init = build(G1, L1, open_seg=3, cursor=13)
    alphabets = {
        "S-fg": {"SIGN", "BATCH", "DELETE", "PUBLISH", "WRITE"},
        "S-freeonly": {"SIGN", "BATCH", "DELETE", "PUBLISH"},
        "S-nofg": {"SIGN", "BATCH", "PUBLISH", "CRASH"},
    }
    for name, alphabet in alphabets.items():
        for s_option in "ABC":
            for stop in ("SD", "SK", "AC"):
                for edef in ("occ", "rec"):
                    cfg = DEFAULT._replace(R="A", S=s_option, SP=stop, E=edef, P="B")
                    report(name, G1, cfg, init, alphabet, [1], 9 if name == "S-nofg" else 8)


def exp_r():
    init = build(G1, L1, open_seg=3, cursor=13)
    alphabet = {"SIGN", "BATCH", "DELETE", "PUBLISH", "WRITE"}
    targets = {"A": [1], "B": [("B", 0, 1)], "C-seg": [("C", 4, 4)], "C-odd": [("C", 5, 4)], "C-part": [("C", 4, 3)]}
    for label, target in targets.items():
        for s_option in "AC":
            for edef in ("occ", "rec"):
                cfg = DEFAULT._replace(R=label[0], S=s_option, SP="AC", E=edef, P="B")
                report(f"R-{label}", G1, cfg, init, alphabet, target, 8)


def exp_p():
    init = build(G2, L2, open_seg=3, cursor=15)
    alphabet = {"SIGN", "BATCH", "PUBLISH", "CRASH", "ROLLBACK", "WRITE", "DELETE"}
    for p_option in ("A_safe", "A_adv", "B"):
        for view in ("inflight", "published"):
            for req4 in (True, False):
                for rooted in (True, False):
                    cfg = DEFAULT._replace(R="A", S="C", SP="AC", E="occ", P=p_option, plan_view=view,
                                           req4=req4, intent_rooted=rooted)
                    report("P", G2, cfg, init, alphabet, [1], 7, kinds=("c", "d", "R3", "stall", "stuck"))


def exp_c0():
    init = build(G1, L3, open_seg=3, cursor=13)
    for switch in ("SWITCH_KEEP", "SWITCH_RECOMPUTE"):
        alphabet = {"SIGN", "BATCH", "PUBLISH", "WRITE", "CRASH", "ROLLBACK", switch}
        for c0 in ("inflight", "published"):
            for r3_view in ("inflight", "published"):
                cfg = DEFAULT._replace(R="A", S="C", SP="AC", E="occ", P="B", c0=c0, r3_view=r3_view)
                report(f"C0-{switch}", G1, cfg, init, alphabet, [1], 7, kinds=("R3", "c", "d"))


def exp_rb():
    init = build(G1, L1, open_seg=3, cursor=13)
    alphabet = {"SIGN", "BATCH", "PUBLISH", "ROLLBACK", "CRASH", "WRITE", "DELETE"}
    for s_option in ("A", "C"):
        for rooted in (True, False):
            for p_option in ("B", "A_safe", "A_adv"):
                cfg = DEFAULT._replace(R="A", S=s_option, SP="AC", E="occ", P=p_option, intent_rooted=rooted)
                report("RB", G1, cfg, init, alphabet, [1], 8, max_states=800000,
                       kinds=("R3", "c", "d", "a", "b", "stuck"))


def scan64():
    """几何敏感性：64 槽段、真实量级的 bump，整理一个段的净产出随参数怎么变（C317 那一型）。"""
    for meta in (1, 7):
        for per_publish in (1, 8):
            for left in (8, 56):
                for holes in (0, 16):
                    for user_units in (0, 8, 24):
                        for index_nodes in (0, 8, 16):
                            for crash_mid in (0, 1):
                                if user_units == 0 and index_nodes == 0:
                                    continue
                                scan64_one(meta, per_publish, left, holes, user_units, index_nodes, crash_mid)


def scan64_one(meta, per_publish, left, holes, user_units, index_nodes, crash_mid):
    geo = Geometry(seg_slots=64, nseg=6, devices=2, ring_k=3, meta_per_publish=meta, max_user=10 ** 6)
    layout = [("U", slot, 2, 1) for slot in range(0, 64 - 2 * holes, 2)]
    layout += [("U", 64 + 2 * i, 2, 1) for i in range(user_units)]
    layout += [("I", 64 + 2 * user_units + i, 1, 1) for i in range(index_nodes)]
    layout += [("I", slot, 1, 1) for slot in range(128, 192 - left)]
    cfg = DEFAULT._replace(R="A", S="C", SP="AC", E="occ", P="B")
    st = build(geo, layout, open_seg=2, cursor=192 - left)
    sign_occ, sign_rec = counts(geo, st, "occ"), counts(geo, st, "rec")
    st = st._replace(intent=(("A", 1), st.txg, None, None, sign_occ), run="running")
    publishes, crashed, enospc = 0, False, False
    while st.intent is not None and publishes < 200:
        for _ in range(per_publish):
            if st.intent is not None:
                st, _w = batch(geo, cfg, st)
        if st.intent is None:
            break
        nxt = publish(geo, cfg, st)
        if nxt is None:
            enospc = True
            break
        st, publishes = nxt, publishes + 1
        if crash_mid and not crashed and publishes == 1:
            st, crashed = crash(geo, cfg, st), True
    done_occ, done_rec = counts(geo, st, "occ")[0] - sign_occ[0], counts(geo, st, "rec")[0] - sign_rec[0]
    fired, low, high, final = False, done_occ, done_occ, done_occ
    for _ in range(2 * (64 // meta) + geo.ring_k):
        nxt = publish(geo, cfg, st)
        if nxt is None:
            enospc = True
            break
        st = nxt
        final = counts(geo, st, "occ")[0] - sign_occ[0]
        low, high, fired = min(low, final), max(high, final), fired or final >= 1
    control = build(geo, layout, open_seg=2, cursor=192 - left)
    control_sign = counts(geo, control, "occ")[0]
    control_low = control_high = 0
    for _ in range(publishes + 2 * (64 // meta) + geo.ring_k):
        nxt = publish(geo, cfg, control)
        if nxt is None:
            break
        control = nxt
        delta = counts(geo, control, "occ")[0] - control_sign
        control_low, control_high = min(control_low, delta), max(control_high, delta)
    print(f"C364SCAN meta={meta} batches_per_publish={per_publish} open_left={left} holes={holes} "
          f"user_units_in_R={user_units} index_nodes_in_R={index_nodes} crash_mid={crash_mid} publishes={publishes} "
          f"enospc={enospc} done_delta_occ={done_occ} done_delta_rec={done_rec} "
          f"Sjia_fires_at_done_occ={done_occ >= 1} Sjia_fires_at_done_rec={done_rec >= 1} "
          f"Sbing_a_at_done_occ={done_occ <= 0} Sbing_a_at_done_rec={done_rec <= 0} "
          f"quiesce_min={low} quiesce_max={high} quiesce_final={final} Sjia_fires_in_quiesce_occ={fired} "
          f"control_no_compaction_min={control_low} control_no_compaction_max={control_high}")
    sys.stdout.flush()


def exp_future():
    """三块盘、一个单元只落两块、槽号可以不同：R甲 / R乙 与 S甲 / S乙 分不分得开（scripted，不搜索）。"""
    geo = Geometry(seg_slots=4, nseg=3, devices=3, ring_k=3, meta_per_publish=1, max_user=10)

    def unit(kind, placements):
        return (kind, 1, 1, 1, tuple(sorted(placements)))
    # 签发前：dev0 seg0 占、seg1 = X、seg2 空；dev1 seg0 占、seg1 = X、seg2 = Y；dev2 seg0 占、seg1 空、seg2 = Y
    base = [unit("I", [(0, 0), (1, 0)]), unit("I", [(2, 1)]), unit("I", [(0, 4), (1, 5)]), unit("I", [(1, 8), (2, 9)])]
    st = State(txg=2, units=tuple(sorted(base)), defer=(), intent=None, ghost=None, run="idle", open_seg=None,
               cursor=0, ring=(), quarantine=frozenset())
    for s_option in ("A", "B"):
        cfg = DEFAULT._replace(S=s_option, E="rec")
        rspec = ("A", 1)
        segs0 = segs0_value(geo, cfg, st, rspec)
        sign = counts(geo, st, "rec")
        after_free_y = st._replace(units=tuple(u for u in st.units if u[4] != ((1, 8), (2, 9))))
        now = counts(geo, after_free_y, "rec")
        pred = predicate(geo, cfg, after_free_y, rspec, segs0)
        print(f"C364FUT case=F1_前台释放只动两块盘 R={rspec} S={s_option} sign_counts={sign} now_counts={now} "
              f"predicate={pred} R_target_reached={target_reached(geo, after_free_y, rspec, 1)} "
              f"T1a_hit={pred and any(now[d] - sign[d] <= 0 for d in range(geo.devices))}")
        after_compact = st._replace(units=tuple(u for u in st.units if u[4] != ((0, 4), (1, 5))) +
                                    (unit("I", [(0, 2), (1, 3)]),))
        now = counts(geo, after_compact, "rec")
        pred = predicate(geo, cfg, after_compact, rspec, segs0)
        print(f"C364FUT case=F2_R在dev2上签发时就空 R={rspec} S={s_option} sign_counts={sign} now_counts={now} "
              f"predicate={pred} R_target_reached={target_reached(geo, after_compact, rspec, 1)}")
    for dev in range(3):
        cfg = DEFAULT._replace(R="B", S="A", E="rec")
        rspec = ("B", dev, 1)
        segs0 = segs0_value(geo, cfg, st, rspec)
        after_compact = st._replace(units=tuple(u for u in st.units if u[4] != ((0, 4), (1, 5))) +
                                    (unit("I", [(0, 2), (1, 3)]),))
        print(f"C364FUT case=F2_R乙逐盘 R={rspec} S=A units_in_R_at_sign={len(units_in_r(geo, st, rspec))} "
              f"predicate_after={predicate(geo, cfg, after_compact, rspec, segs0)}")
    # F3：加盘。签发时两块盘，意图期内加一块全空的盘
    cfg = DEFAULT._replace(S="B", E="rec")
    geo2 = Geometry(seg_slots=4, nseg=3, devices=2, ring_k=3, meta_per_publish=1, max_user=10)
    two = st._replace(units=(unit("I", [(0, 0), (1, 0)]), unit("I", [(0, 4), (1, 4)])))
    segs0 = segs0_value(geo2, cfg, two, ("A", 1))
    three_now = counts(geo, two, "rec")
    print(f"C364FUT case=F3_加盘 S=B segs0_sum={segs0} sum_after_add={sum(three_now)} "
          f"predicate={sum(three_now) - segs0 >= 1} R_target_reached={target_reached(geo, two, ('A', 1), 1)}")


def selftest():
    """模型自己的判别力自证：每条检查在一个已知该红的配置上红、在它的改法上不红。任何一条对不上退出码 1。"""
    failures = []

    def found(geo, cfg, init, alphabet, targets, depth):
        return search(geo, cfg, init, alphabet, targets, depth)[0]

    init1 = build(G1, L1, open_seg=3, cursor=13)
    free_alpha = {"SIGN", "BATCH", "DELETE", "PUBLISH"}
    sd = found(G1, DEFAULT._replace(S="A", SP="SD", E="rec"), init1, free_alpha, [1], 5)
    ac = found(G1, DEFAULT._replace(S="A", SP="AC", E="rec"), init1, free_alpha, [1], 5)
    if "c" not in sd:
        failures.append("(c) 在 S甲 + 停机即删意图 + 前台释放上应当红")
    if "c" in ac:
        failures.append("(c) 在 S甲 + 意图照目标态删 上不应红")
    nofg = {"SIGN", "BATCH", "PUBLISH"}
    if "b" not in found(G1, DEFAULT._replace(S="A", SP="AC", E="occ"), init1, nofg, [1], 3):
        failures.append("(b) 在 S甲 + occ 的 SIGN→BATCH→BATCH 上应当红")
    if "a" not in found(G1, DEFAULT._replace(S="C", SP="AC", E="occ"), init1, nofg, [1], 3):
        failures.append("(a) 在 S丙 + occ 的 SIGN→BATCH→BATCH 上应当红")
    init2 = build(G2, L2, open_seg=3, cursor=15)
    p_alpha = {"SIGN", "BATCH", "PUBLISH", "WRITE"}
    adv = found(G2, DEFAULT._replace(S="C", SP="AC", P="A_adv"), init2, p_alpha, [1], 6)
    safe = found(G2, DEFAULT._replace(S="C", SP="AC", P="A_safe"), init2, p_alpha, [1], 6)
    if "c" not in adv:
        failures.append("(c) 在 P甲（续做点越过没搬成的落点、扫到尾就删）上应当红")
    if "c" in safe:
        failures.append("(c) 在 P甲（续做点只越过搬成的落点、删前全量核目标态）上不应红")
    d_alpha = {"SIGN", "BATCH", "PUBLISH"}
    blind = found(G1, DEFAULT._replace(S="C", SP="AC", plan_view="published", req4=False), init1, d_alpha, [1], 4)
    checked = found(G1, DEFAULT._replace(S="C", SP="AC", plan_view="published", req4=True), init1, d_alpha, [1], 4)
    if "d" not in blind:
        failures.append("(d) 在 按已发布视图排批 + 不核 上应当红")
    if "d" in checked:
        failures.append("(d) 在 按已发布视图排批 + 硬要求 4 上不应红")
    init3 = build(G1, L3, open_seg=3, cursor=13)
    keep = found(G1, DEFAULT._replace(S="C", SP="AC"), init3, {"WRITE", "SIGN", "SWITCH_KEEP"}, [1], 3)
    recompute = found(G1, DEFAULT._replace(S="C", SP="AC"), init3, {"WRITE", "SIGN", "SWITCH_RECOMPUTE"}, [1], 3)
    if "R3" not in keep:
        failures.append("R3 在 切换重做改了分配代而 c0 照旧 上应当红")
    if "R3" in recompute:
        failures.append("R3 在 切换重做时 c0 跟着改 上不应红")
    rb_alpha = {"SIGN", "BATCH", "PUBLISH", "ROLLBACK"}
    loose = found(G1, DEFAULT._replace(S="C", SP="AC", P="A_safe", intent_rooted=False), init1, rb_alpha, [1], 7)
    rooted = found(G1, DEFAULT._replace(S="C", SP="AC", P="A_safe", intent_rooted=True), init1, rb_alpha, [1], 7)
    if "stuck" not in loose:
        failures.append("stuck 在 P甲 + 意图不随根回退 上应当红")
    if "stuck" in rooted:
        failures.append("stuck 在 P甲 + 意图随根回退 上不应红")
    ov_alpha = {"SIGN", "BATCH", "DELETE"}
    odd = found(G1, DEFAULT._replace(R="C", S="C", SP="AC"), init1, ov_alpha, [("C", 5, 4)], 3)
    seg = found(G1, DEFAULT._replace(R="A", S="C", SP="AC"), init1, ov_alpha, [1], 3)
    if "c_overlap" not in odd:
        failures.append("c_overlap 在 R丙 起点落在一个两槽单元中间 上应当红")
    if "c_overlap" in seg:
        failures.append("c_overlap 在 R甲（段对齐） 上不应红")
    for failure in failures:
        print(f"C364SELFTEST FAIL {failure}")
    print(f"C364SELFTEST {'ok' if not failures else 'red'} checks=14 failures={len(failures)}")
    return not failures


GROUPS = {"S": exp_s, "R": exp_r, "P": exp_p, "C0": exp_c0, "RB": exp_rb, "G64": scan64, "FUT": exp_future}

if __name__ == "__main__":
    wanted = sys.argv[1:] or ["SELF", "S", "R", "P", "C0", "RB", "G64", "FUT"]
    for group in wanted:
        if group == "SELF":
            if not selftest():
                sys.exit(1)
        else:
            GROUPS[group]()
