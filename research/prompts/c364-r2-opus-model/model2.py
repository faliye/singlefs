#!/usr/bin/env python3
"""C364（整理意图记录没有字段表） 第二轮攻方腿的独立核实模型。只用标准库。

与第一轮模型（research/prompts/c364-r1-opus-model/model.py）的差别：
- 分配记录按副本逐条存：一个单元 = (种类, 出生代, 跨度, 副本组)，副本 = (设备, 槽号, 分配代)。
  mirror=True 时每个单元在每块盘上同号（crates/singlefs-core/src/allocator.rs 的 PoolAllocator::record 今天的样子）；
  mirror=False 时每个单元落其中两块盘、每块盘各取该盘内最低的合格槽（D3 已定项 8 第 1 条「在每一块被选中的设备上各自取」），
  两份副本槽号可以不同。
- 意图的四种落点：ROOT（根下的树，随根回退）、SB（根之外，每次发布根槽之后随超级块写）、
  JEACH（只在 journal 里，每次发布重记一条）、JCHG（只在 journal 里，只在意图变了的那次发布记一条）。
  状态里始终另存一份「随根的真值」（根环快照里的意图），恢复之后拿落点给出的意图与真值比。
- 事件多了：CRASH_SBLAG（根槽 FUA 之后、超级块槽之前崩）、ROOTLOSS（新实例的第一个根读不出，退到上一个实例的根）。
"""
from collections import namedtuple, deque
from functools import lru_cache
from itertools import combinations

Geometry = namedtuple("Geometry", "seg_slots nseg devices ring_k meta_per_publish max_user journal_keep mirror")
Config = namedtuple("Config", "R S P E SP place move exclusion wrap")
DEFAULT = Config(R="A", S="A", P="B", E="rec", SP="AC", place="ROOT", move="copy", exclusion="inflight", wrap=False)

# unit    = (kind, birth, span, copies)          copies = ((dev, slot, gen), ...) 按设备升序
# defer   = ((dev, slot, span, rel_txg), ...)
# intent  = (rspec, c0, segs0, rp, sign_counts)  rspec: ("A", seg) | ("B", dev, seg)；rp: None | (dev, slot)
# ring    = ((txg, snap, abandoned, first_of_instance), ...)   snap = (units, defer, intent)
# sb      = (cur, prev)                          两份：最后一次发布写的、再前一次写的
# jnl     = ((txg, intent_or_None), ...)         物理上最近 journal_keep 次发布留下的意图记录
State = namedtuple(
    "State",
    "txg units defer intent ghost run opens ring quarantine sb sb_pending jnl abandoned fresh_instance",
)


def r_cover(geo, rspec):
    if rspec[0] == "A":
        seg = rspec[1]
        return frozenset(range(geo.devices)), seg * geo.seg_slots, (seg + 1) * geo.seg_slots
    dev, seg = rspec[1], rspec[2]
    return frozenset([dev]), seg * geo.seg_slots, (seg + 1) * geo.seg_slots


def total_slots(geo):
    return geo.seg_slots * geo.nseg


@lru_cache(maxsize=400000)
def live_map(units):
    return frozenset((dev, s) for unit in units for dev, slot, _gen in unit[3] for s in range(slot, slot + unit[2]))


@lru_cache(maxsize=400000)
def defer_map(defer):
    return frozenset((dev, s) for dev, slot, span, _rel in defer for s in range(slot, slot + span))


def occupied(st, dev, slot, with_quarantine=True):
    key = (dev, slot)
    if key in live_map(st.units) or key in defer_map(st.defer):
        return True
    return with_quarantine and key in st.quarantine


def empty_segments(geo, st, dev, edef):
    """全空聚簇段数。rec：段内没有未释放的分配记录（D3 已定项 10 ① 字面）；occ：段内没有活副本也没有 defer 槽
    （D26 已定项 1 ①「defer 段不算空」，工作区 DeviceFreeMap::mark_released 的读法）。影子账隔离的槽两种都不算占。"""
    if edef == "occ":
        busy = {s // geo.seg_slots for d, s in live_map(st.units) | defer_map(st.defer) if d == dev}
    else:
        busy = {slot // geo.seg_slots for u in st.units for d, slot, _g in u[3] if d == dev}
    return geo.nseg - len(busy)


def counts(geo, st, edef):
    return tuple(empty_segments(geo, st, dev, edef) for dev in range(geo.devices))


def copies_in_r(geo, units, rspec, c0=None):
    devs, lo, hi = r_cover(geo, rspec)
    found = []
    for unit in units:
        for dev, slot, gen in unit[3]:
            if dev in devs and lo <= slot < hi and (c0 is None or gen <= c0):
                found.append(((dev, slot), unit))
    return sorted(found)


def target_reached(geo, st, rspec, c0):
    return not copies_in_r(geo, st.units, rspec, c0)


def r_empty(geo, st, rspec, edef):
    devs, lo, hi = r_cover(geo, rspec)
    for dev in devs:
        for s in range(lo, hi):
            if edef == "occ" and ((dev, s) in live_map(st.units) or (dev, s) in defer_map(st.defer)):
                return False
    if edef == "rec":
        return not copies_in_r(geo, st.units, rspec)
    return True


def predicate(geo, cfg, st, intent):
    rspec, _c0, segs0, _rp, _sign = intent
    now = counts(geo, st, cfg.E)
    if cfg.S == "A":
        return all(now[dev] - base >= 1 for dev, base in segs0)
    if cfg.S == "B":
        return sum(now) - segs0 >= 1
    return not copies_in_r(geo, st.units, rspec)


def segs0_value(geo, cfg, st, rspec):
    devs, _lo, _hi = r_cover(geo, rspec)
    now = counts(geo, st, cfg.E)
    if cfg.S == "A":
        return tuple((dev, now[dev]) for dev in sorted(devs))
    if cfg.S == "B":
        return sum(now)
    return None


# ------------------------------------------------------------------ 分配
def newest_valid(st):
    return max((e for e in st.ring if not e[2]), key=lambda e: e[0])


def exclusions(geo, cfg, st):
    """「候选落点不在任何活整理意图的 R 里」。exclusion=inflight 读内存里的意图；published 读最新有效根里那份。"""
    intent = st.intent if cfg.exclusion == "inflight" else newest_valid(st)[1][2]
    if intent is None:
        return ()
    return (r_cover(geo, intent[0]),)


def excluded(excl, dev, slot):
    return any(dev in devs and lo <= slot < hi for devs, lo, hi in excl)


def blocked(geo, st, excl, dev, slot):
    return occupied(st, dev, slot) or excluded(excl, dev, slot)


def in_open(geo, opens, dev, slot):
    seg = opens[dev][0]
    return seg is not None and seg * geo.seg_slots <= slot < (seg + 1) * geo.seg_slots


def user_slot(geo, st, excl, devs):
    """D3 已定项 8 第 1 条：每块被选中的盘各自取最低的偶数起点、两槽都空、不在开放段、不在 R。mirror 时全池同号。"""
    if geo.mirror:
        for slot in range(0, total_slots(geo) - 1, 2):
            if all(not in_open(geo, st.opens, d, slot) and not blocked(geo, st, excl, d, slot)
                   and not blocked(geo, st, excl, d, slot + 1) for d in devs):
                return {d: slot for d in devs}
        return None
    chosen = {}
    for d in devs:
        for slot in range(0, total_slots(geo) - 1, 2):
            if not in_open(geo, st.opens, d, slot) and not blocked(geo, st, excl, d, slot) \
                    and not blocked(geo, st, excl, d, slot + 1):
                chosen[d] = slot
                break
        else:
            return None
    return chosen


def bump_one(geo, st, excl, opens, dev_list):
    """提交内生块（1 槽）：开放段 bump；游标处不可用或段满就开最低全空段；没有全空段回落到该盘最低空槽（D3 已定项 8 第 2 条）。
    mirror 时开放段全池一个（opens 每盘同值），槽号全池同号。返回 ({dev: slot}, opens)。"""
    opens = list(opens)
    if geo.mirror:
        seg, cur = opens[0]
        for _ in range(geo.nseg + 1):
            if seg is not None and cur < (seg + 1) * geo.seg_slots and \
                    all(not blocked(geo, st, excl, d, cur) for d in dev_list):
                opens = [(seg, cur + 1)] * geo.devices
                return {d: cur for d in dev_list}, tuple(opens)
            seg = None
            for cand in range(geo.nseg):
                lo = cand * geo.seg_slots
                if all(not blocked(geo, st, excl, d, s) for d in dev_list for s in range(lo, lo + geo.seg_slots)):
                    seg, cur = cand, lo
                    break
            if seg is None:
                break
        for s in range(total_slots(geo)):
            if all(not blocked(geo, st, excl, d, s) for d in dev_list):
                return {d: s for d in dev_list}, tuple(opens)
        return None
    chosen = {}
    for d in dev_list:
        seg, cur = opens[d]
        got = None
        for _ in range(geo.nseg + 1):
            if seg is not None and cur < (seg + 1) * geo.seg_slots and not blocked(geo, st, excl, d, cur):
                got = cur
                opens[d] = (seg, cur + 1)
                break
            seg = None
            for cand in range(geo.nseg):
                lo = cand * geo.seg_slots
                if all(not blocked(geo, st, excl, d, s) for s in range(lo, lo + geo.seg_slots)):
                    seg, cur = cand, lo
                    break
            if seg is None:
                break
        if got is None:
            for s in range(total_slots(geo)):
                if not blocked(geo, st, excl, d, s):
                    got = s
                    break
        if got is None:
            return None
        chosen[d] = got
    return chosen, tuple(opens)


def pairs(geo):
    if geo.mirror:
        return [tuple(range(geo.devices))]
    return list(combinations(range(geo.devices), 2))


def add_unit(st, unit):
    return st._replace(units=tuple(sorted(st.units + (unit,))))


def release_copies(st, unit, which, rel_txg):
    """把 unit 的 which 这几份副本放进 defer（记录改写成已释放 + 释放代）；全部释放时单元消失。"""
    keep = tuple(c for c in unit[3] if (c[0], c[1]) not in which)
    gone = tuple(c for c in unit[3] if (c[0], c[1]) in which)
    units = tuple(u for u in st.units if u != unit)
    if keep:
        units = tuple(sorted(units + ((unit[0], unit[1], unit[2], keep),)))
    defer = tuple(sorted(st.defer + tuple((d, s, unit[2], rel_txg) for d, s, _g in gone)))
    return st._replace(units=units, defer=defer)


# ------------------------------------------------------------------ 发布、恢复
def journal_value(cfg, st, prev_snap_intent):
    if cfg.place == "JEACH":
        return True
    if cfg.place == "JCHG":
        return st.intent != prev_snap_intent
    return False


def publish(geo, cfg, st):
    excl = exclusions(geo, cfg, st)
    new = st
    for index in range(geo.meta_per_publish):
        devs = pairs(geo)[(st.txg + index) % len(pairs(geo))]
        got = bump_one(geo, new, excl, new.opens, devs)
        if got is None:
            return None
        where, opens = got
        new = add_unit(new, ("M", st.txg, 1, tuple(sorted((d, s, st.txg) for d, s in where.items()))))
        new = new._replace(opens=opens)
    for unit in [u for u in new.units if u[0] == "M" and u[1] < st.txg]:
        new = release_copies(new, unit, {(d, s) for d, s, _g in unit[3]}, st.txg)
    prev_intent = newest_valid(st)[1][2] if st.ring else None
    ring = (st.ring + ((st.txg, None, False, st.fresh_instance),))[-geo.ring_k:]
    oldest_valid = min(e[0] for e in ring if not e[2])
    defer = tuple(e for e in new.defer if e[3] > oldest_valid)
    abandoned_live = any(e[2] for e in ring)
    quarantine = new.quarantine if abandoned_live else frozenset()
    new = new._replace(defer=defer, quarantine=quarantine)
    snap = (new.units, new.defer, new.intent)
    ring = ring[:-1] + ((st.txg, snap, False, st.fresh_instance),)
    sb = (new.intent, st.sb[0])
    jnl = st.jnl
    if journal_value(cfg, new, prev_intent):
        jnl = (jnl + ((st.txg, new.intent),))[-geo.journal_keep:]
    elif cfg.place in ("JEACH", "JCHG"):
        jnl = (jnl + ((st.txg, "nochange"),))[-geo.journal_keep:]  # 这次发布占了环，但没记意图
    live_txgs = {e[0] for e in ring} | {j[0] for j in jnl}
    abandoned = frozenset(t for t in new.abandoned if t in live_txgs)  # 被抛弃 / 读不出的根与记录还在盘上时，这个排除要留着
    return new._replace(ring=ring, txg=st.txg + 1, sb=sb, sb_pending=True, jnl=jnl, fresh_instance=False,
                        abandoned=abandoned)


def placed_intent(cfg, st, root_entry, sb_choice):
    """恢复时按落点读回意图。root_entry 是所选根；sb_choice 是超级块上实际留着的那一份。"""
    if cfg.place == "ROOT":
        return root_entry[1][2]
    if cfg.place == "SB":
        return sb_choice
    txg_r = root_entry[0]
    for txg, value in reversed(st.jnl):
        if txg <= txg_r and txg not in st.abandoned and value != "nochange":
            return value
    return None


def recover(geo, cfg, st, root_entry, ring, sb_choice, quarantine, abandoned):
    snap = root_entry[1]
    probe = st._replace(ring=ring, abandoned=abandoned)
    intent = placed_intent(cfg, probe, root_entry, sb_choice)
    empty_opens = ((None, 0),) * geo.devices
    new = st._replace(txg=st.txg + 1, units=snap[0], defer=snap[1], intent=intent, ghost=None,
                      run="running" if intent else "idle", opens=empty_opens, ring=ring,
                      quarantine=quarantine, sb=(sb_choice, sb_choice), sb_pending=False,
                      abandoned=abandoned, fresh_instance=True)
    return new, snap[2]


def occupied_set(geo, units, defer):
    return live_map(units) | defer_map(defer)


def recovery_events(geo, cfg, st, alphabet):
    if "CRASH" in alphabet:
        entry = newest_valid(st)
        yield f"CRASH(新 txg={st.txg + 1})", recover(geo, cfg, st, entry, st.ring, st.sb[0], st.quarantine, st.abandoned)
    if "CRASH_SBLAG" in alphabet and cfg.place == "SB" and st.sb_pending:
        entry = newest_valid(st)
        yield f"CRASH_SBLAG(新 txg={st.txg + 1})", recover(geo, cfg, st, entry, st.ring, st.sb[1], st.quarantine,
                                                           st.abandoned)
    if "ROOTLOSS" in alphabet:
        valid = sorted((e for e in st.ring if not e[2]), key=lambda e: e[0])
        if len(valid) >= 2 and valid[-1][3]:
            lost = valid[-1]
            ring = tuple(e for e in st.ring if e[0] != lost[0])
            yield f"ROOTLOSS(根 {lost[0]} 读不出→根 {valid[-2][0]},新 txg={st.txg + 1})", recover(
                geo, cfg, st, valid[-2], ring, st.sb[0], st.quarantine, st.abandoned | {lost[0]})
    if "ROLLBACK" in alphabet:
        valid = sorted((e for e in st.ring if not e[2]), key=lambda e: e[0])
        for target in valid[:-1]:
            ring = tuple((t, s, ab or t > target[0], f) for t, s, ab, f in st.ring)
            abandoned_occ = frozenset()
            for t, s, ab, _f in ring:
                if ab:
                    abandoned_occ |= occupied_set(geo, s[0], s[1])
            quarantine = st.quarantine | (abandoned_occ - occupied_set(geo, target[1][0], target[1][1]))
            abandoned = st.abandoned | frozenset(t for t, _s, ab, _f in ring if ab)
            yield f"ROLLBACK(→根 {target[0]},新 txg={st.txg + 1})", recover(
                geo, cfg, st, target, ring, st.sb[0], quarantine, abandoned)


# ------------------------------------------------------------------ 一批整理
def plan(geo, cfg, st):
    """排批读在飞态（硬要求 4 在内：计划就是施加前那一刻的在飞态）。返回 (键, 单元) 或 None。"""
    rspec, c0, _segs0, rp, _sign = st.intent
    cands = copies_in_r(geo, st.units, rspec, c0)
    if cfg.P == "A":
        cands = [c for c in cands if c[0] >= rp]
    return cands[0] if cands else None


def move_copies(geo, cfg, st, key, unit, excl):
    """把 unit 搬出 R。move=copy：只搬 key 那一份（mirror 时副本同号，一起搬）；move=unit：整个单元的副本都搬。"""
    if geo.mirror or cfg.move == "unit":
        which = {(d, s) for d, s, _g in unit[3]}
    else:
        which = {key}
    devs = sorted(d for d, _s in which)
    if unit[0] == "U":
        got = user_slot(geo, st, excl, devs)
        if got is None:
            return None
        opens = st.opens
    else:
        got = bump_one(geo, st, excl, st.opens, devs)
        if got is None:
            return None
        got, opens = got
    new = release_copies(st, unit, which, st.txg)
    remaining = [u for u in new.units if u[0] == unit[0] and u[1] == unit[1] and u[2] == unit[2] and
                 set((d, s) for d, s, _g in u[3]) == set((d, s) for d, s, _g in unit[3]) - which]
    moved = tuple((d, got[d], st.txg) for d in devs)
    if remaining:
        base = remaining[0]
        units = tuple(u for u in new.units if u != base)
        merged = (unit[0], unit[1], unit[2], tuple(sorted(base[3] + moved)))
        new = new._replace(units=tuple(sorted(units + (merged,))))
    else:
        new = add_unit(new, (unit[0], unit[1], unit[2], tuple(sorted(moved))))
    return new._replace(opens=opens)


def quiesce_publishes(geo):
    return geo.nseg * (geo.seg_slots // max(1, geo.meta_per_publish) + 1) + geo.ring_k


def batch(geo, cfg, st):
    rspec, c0, segs0, rp, sign = st.intent
    devs, lo, _hi = r_cover(geo, rspec)
    wits = []
    excl = exclusions(geo, cfg, st)
    new = st
    planned = plan(geo, cfg, st)
    if planned is not None:
        key, unit = planned
        moved = move_copies(geo, cfg, st, key, unit, excl)
        done = moved is not None
        if done:
            new = moved
        if cfg.P == "A":
            rp = (key[0], key[1] + 1) if done else key
    reached = target_reached(geo, new, rspec, c0)
    if cfg.P == "A":
        rest = [c for c in copies_in_r(geo, new.units, rspec, c0) if c[0] >= rp]
        if not rest and not reached:
            if cfg.wrap:
                rp = (min(devs), lo)
            else:
                wits.append(("stuck", f"续做点 {rp} 之后没有要搬的，而 R 里还有分配代 ≤ c0 的未释放副本 "
                                      f"{[c[0] for c in copies_in_r(geo, new.units, rspec, c0)]}"))
        delete_cond = not rest and reached
    else:
        delete_cond = reached
    intent = (rspec, c0, segs0, rp, sign)
    new = new._replace(intent=intent)
    pred = predicate(geo, cfg, new, intent)
    now = counts(geo, new, cfg.E)

    sign_counts, sign_devs = sign

    def check_a():
        low = [d for d in sorted(devs) if now[d] - sign_counts[d] <= 0]
        if low:
            wits.append(("a", f"谓词判有产出，而盘 {low} 的全空段数 {[now[d] for d in low]} 不多于签发时 "
                              f"{[sign_counts[d] for d in low]}"))

    def delete(state):
        if not r_empty(geo, state, rspec, cfg.E):
            wits.append(("blind", f"谓词判的那一刻 R 按 {cfg.E} 算还不空，R 自己腾出的段进不了这一次判定"))
        if not target_reached(geo, state, rspec, c0):
            wits.append(("c", f"意图被删而 R 里还有分配代 ≤ c0 的未释放副本 "
                              f"{[c[0] for c in copies_in_r(geo, state.units, rspec, c0)]}"))
        return state._replace(intent=None, ghost=(rspec, c0, segs0, sign, pred), run="idle")

    def gained(state):
        cur = counts(geo, state, cfg.E)
        return bool(sign_devs) and all(cur[d] - sign_counts[d] >= 1 for d in sign_devs)

    def check_b(state):
        horizon = quiesce_publishes(geo)
        probe = state
        ever_pred = False
        empty_seen = r_empty(geo, probe, rspec, cfg.E)
        gain_at_delete = gained(probe)
        gain_seen = gain_at_delete
        for _ in range(horizon):
            probe = publish(geo, cfg, probe)
            if probe is None:
                return
            ever_pred = ever_pred or predicate(geo, cfg, probe, intent)
            empty_seen = empty_seen or r_empty(geo, probe, rspec, cfg.E)
            gain_seen = gain_seen or gained(probe)
        if empty_seen and not ever_pred:
            wits.append(("b", f"R 在之后 {horizon} 次发布里变空（{cfg.E}），停机谓词在删意图那一刻判假、之后再判也一次都不真"))
        if empty_seen:
            wits.append(("b_ac", f"R 在之后 {horizon} 次发布里变空（{cfg.E}），而谓词只在删意图那一刻判、判的是假"))
        if gain_at_delete:
            wits.append(("b_fn", f"删意图那一刻，签发时 R 里有副本的每块盘 {sorted(sign_devs)} 全空段数都多了 ≥ 1，谓词判假"))
        if gain_seen:
            wits.append(("b_fn_hz", f"删意图之后 {horizon} 次发布内，签发时 R 里有副本的每块盘 {sorted(sign_devs)} "
                                    f"全空段数都多过 ≥ 1，而谓词只在删意图那一刻判、判的是假"))

    if cfg.SP == "SD":
        if pred:
            check_a()
            new = delete(new)
        elif delete_cond:
            new = delete(new)
            check_b(new)
    else:
        if delete_cond:
            new = delete(new)
            if pred:
                check_a()
            else:
                check_b(new)
    return new, wits


# ------------------------------------------------------------------ 事件与搜索
def sign_targets(geo, cfg, st, sign_segments):
    out = []
    for seg in sign_segments:
        if cfg.R == "A":
            rspec = ("A", seg)
            if copies_in_r(geo, st.units, rspec):
                out.append(rspec)
        else:
            for dev in range(geo.devices):
                rspec = ("B", dev, seg)
                if copies_in_r(geo, st.units, rspec):
                    out.append(rspec)
    return out


def events(geo, cfg, st, alphabet, sign_segments):
    if "SIGN" in alphabet and st.intent is None:
        for rspec in sign_targets(geo, cfg, st, sign_segments):
            devs, lo, _hi = r_cover(geo, rspec)
            rp = (min(devs), lo) if cfg.P == "A" else None
            sign_devs = frozenset(key[0] for key, _u in copies_in_r(geo, st.units, rspec))
            intent = (rspec, st.txg, segs0_value(geo, cfg, st, rspec), rp, (counts(geo, st, cfg.E), sign_devs))
            yield f"SIGN(R={rspec},c0={st.txg})", st._replace(intent=intent, run="running"), []
    if "BATCH" in alphabet and st.intent is not None and st.run == "running":
        new, wits = batch(geo, cfg, st)
        yield "BATCH", new, wits
    if "WRITE" in alphabet and sum(1 for u in st.units if u[0] == "U") < geo.max_user:
        excl = exclusions(geo, cfg, st)
        for devs in pairs(geo):
            got = user_slot(geo, st, excl, devs)
            if got is not None:
                unit = ("U", st.txg, 2, tuple(sorted((d, got[d], st.txg) for d in devs)))
                yield f"WRITE{tuple((d, got[d]) for d in devs)}", add_unit(st, unit), []
    if "DELETE" in alphabet:
        for unit in st.units:
            if unit[0] in ("U", "I"):
                yield f"DELETE {unit[0]}{tuple((d, s) for d, s, _g in unit[3])}", \
                    release_copies(st, unit, {(d, s) for d, s, _g in unit[3]}, st.txg), []
    if "PUBLISH" in alphabet:
        new = publish(geo, cfg, st)
        if new is not None:
            yield f"PUBLISH(txg={st.txg})", new, []
    for label, (new, truth) in recovery_events(geo, cfg, st, alphabet):
        yield label, new, recovery_checks(geo, cfg, new, truth)


def state_checks(geo, st):
    """每一步之后：意图活着而 R 里出现分配代 > c0 的活副本（目标态罩不到它，R 腾不空）。"""
    if st is None or st.intent is None:
        return []
    rspec, c0 = st.intent[0], st.intent[1]
    late = [c[0] for c in copies_in_r(geo, st.units, rspec) if c[0] not in {k for k, _u in copies_in_r(geo, st.units, rspec, c0)}]
    if late:
        return [("r3", f"意图活着，R 里有分配代 > c0={c0} 的活副本 {late}")]
    return []


def recovery_checks(geo, cfg, st, truth):
    wits = []
    eff = st.intent
    if truth is not None and eff is None and not target_reached(geo, st, truth[0], truth[1]):
        if work_done(st, truth[1]):
            wits.append(("c_lost", f"恢复后读不到意图，而所选根里那条意图 R={truth[0]} c0={truth[1]} 已经搬过、目标态没达成"))
        else:
            wits.append(("c_lost_nowork", f"恢复后读不到意图，所选根里那条意图 R={truth[0]} c0={truth[1]} 还一批都没搬"))
    if truth is None and eff is not None:
        wits.append(("resurrect", f"所选根里没有意图，落点读回一条 R={eff[0]} c0={eff[1]}"))
    if truth is not None and eff is not None and truth != eff:
        if cfg.P == "A" and eff[3] > truth[3]:
            wits.append(("rp_ahead", f"读回的续做点 {eff[3]} 在所选根那份 {truth[3]} 之后"))
        if eff[2] != truth[2]:
            wits.append(("segs0_mismatch", f"读回的 segs0 {eff[2]} 与所选根那份 {truth[2]} 不同"))
    if eff is not None:
        verdict = liveness(geo, cfg, st)
        if verdict == "never":
            wits.append(("never", "恢复之后不再有前台写入，连续整理加发布，意图在界内一直删不掉"))
    return wits


def work_done(st, c0):
    """这条意图签发之后有没有搬过东西：搬迁不改出生代、副本的分配代换成搬迁那次的 txg（≥ c0），前台新写的单元出生代 = 分配代。"""
    return any(gen >= c0 and gen != unit[1] for unit in st.units for _d, _s, gen in unit[3])


def liveness(geo, cfg, st):
    """有界活性：不再有前台写入，每轮一批整理加一次发布；R 里要搬的副本数 n，给 2n + 2K + 4 轮（续做点绕回一圈 + defer 排空的余量）。"""
    probe = st
    rspec, c0 = st.intent[0], st.intent[1]
    bound = 2 * len(copies_in_r(geo, st.units, rspec, c0)) + 2 * geo.ring_k + 4
    for _ in range(bound):
        if probe.intent is None:
            return "done"
        probe, _w = batch(geo, cfg, probe)
        if probe.intent is None:
            return "done"
        nxt = publish(geo, cfg, probe)
        if nxt is None:
            return "enospc"
        probe = nxt
    return "never"


def search(geo, cfg, init, alphabet, sign_segments, depth, max_states=3000000, kinds=None):
    found = {}
    visited = {init}
    frontier = deque([(init, ())])
    while frontier:
        st, path = frontier.popleft()
        if len(path) >= depth:
            continue
        for label, new, wits in events(geo, cfg, st, alphabet, sign_segments):
            for kind, detail in wits + state_checks(geo, new):
                if kind not in found and (kinds is None or kind in kinds):
                    found[kind] = (path + (label,), detail)
            if new is not None and new not in visited:
                if len(visited) >= max_states:
                    return found, len(visited), False
                visited.add(new)
                frontier.append((new, path + (label,)))
    return found, len(visited), True


def build(geo, layout, opens=None, txg=2):
    """layout: [(kind, [(dev, slot), ...], span, gen)]。mirror 时给一个槽号，自动铺到每块盘。"""
    units = []
    for kind, where, span, gen in layout:
        if geo.mirror:
            where = [(d, where) for d in range(geo.devices)]
        units.append((kind, gen, span, tuple(sorted((d, s, gen) for d, s in where))))
    units = tuple(sorted(units))
    opens = opens if opens is not None else ((None, 0),) * geo.devices
    snap = (units, (), None)
    return State(txg=txg, units=units, defer=(), intent=None, ghost=None, run="idle", opens=opens,
                 ring=((txg - 1, snap, False, False),), quarantine=frozenset(), sb=(None, None), sb_pending=False,
                 jnl=((txg - 1, None),), abandoned=frozenset(), fresh_instance=False)


def crash_recover(geo, cfg, st):
    """普通崩溃：在飞全丢，按落点读回意图（扫描用）。"""
    entry = newest_valid(st)
    return recover(geo, cfg, st, entry, st.ring, st.sb[0], st.quarantine, st.abandoned)[0]
