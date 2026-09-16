#!/usr/bin/env python3
"""C364（整理意图记录没有字段表） 第一轮攻方腿的独立核实模型。只用标准库。

建模对象：两块（或更多）盘、每盘逐段的已用槽、分配记录（落点、跨度、分配代、已释放标志）、
一条整理意图（R / c0 / segs0 / 续做点按选项取）、分批搬迁（每批一个事务）、前台并发分配与释放、
发布（提交内生块从开放段 bump、上一次发布的元数据进 defer）、崩溃、续做、回退到根环里任意更早的根。
口径照 crates/singlefs-core/src/allocator.rs：两盘同号、开放段 = 最低全空段、bump 只在内存、挂载后新开一段；
全空段数两种定义都建（E=occ：defer 槽算占着，工作区 allocator.rs 的读法；E=rec：只看未释放记录，D3 已定项 10 ① 字面）。
"""
from collections import namedtuple, deque
from functools import lru_cache

Geometry = namedtuple("Geometry", "seg_slots nseg devices ring_k meta_per_publish max_user")
Config = namedtuple(
    "Config",
    "R S P E SP c0 plan_view req4 r3_view intent_rooted",
)
DEFAULT = Config(R="A", S="A", P="B", E="occ", SP="SD", c0="inflight",
                 plan_view="inflight", req4=True, r3_view="inflight", intent_rooted=True)

# unit = (kind, gen, birth, span, placements)   placements = ((dev, slot), ...) 按设备升序
# defer entry = (dev, slot, span, rel_txg)
# intent = (Rspec, c0, segs0, rp, sign_true)     Rspec: ("A", seg) | ("B", dev, seg) | ("C", start, n)
# ghost  = (Rspec, c0, segs0, sign_true)          被删掉的最后一条意图，只供判 (b)(c)
# ring entry = (txg, snap, abandoned)             snap = (units, defer, intent, ghost)
State = namedtuple(
    "State",
    "txg units defer intent ghost run open_seg cursor ring quarantine",
)


def r_range(geo, rspec):
    """返回 (覆盖的设备集合, 槽区间 [lo, hi))。"""
    if rspec[0] == "A":
        seg = rspec[1]
        return frozenset(range(geo.devices)), (seg * geo.seg_slots, (seg + 1) * geo.seg_slots)
    if rspec[0] == "B":
        dev, seg = rspec[1], rspec[2]
        return frozenset([dev]), (seg * geo.seg_slots, (seg + 1) * geo.seg_slots)
    start, count = rspec[1], rspec[2]
    return frozenset(range(geo.devices)), (start, start + count)


def total_slots(geo):
    return geo.seg_slots * geo.nseg


@lru_cache(maxsize=200000)
def live_map(units):
    return frozenset((pdev, s) for unit in units for pdev, pslot in unit[4] for s in range(pslot, pslot + unit[3]))


@lru_cache(maxsize=200000)
def defer_map(defer):
    return frozenset((ddev, s) for ddev, dslot, dspan, _rel in defer for s in range(dslot, dslot + dspan))


def occupied(geo, st, dev, slot, include_quarantine=True):
    key = (dev, slot)
    if key in live_map(st.units) or key in defer_map(st.defer):
        return True
    return include_quarantine and key in st.quarantine


def empty_segments(geo, st, dev, edef):
    """全空聚簇段数。edef=occ：段内没有活单元也没有 defer 槽；edef=rec：段内没有未释放的分配记录（按记录起点）。"""
    if edef == "occ":
        busy_segments = {s // geo.seg_slots for d, s in live_map(st.units) | defer_map(st.defer) if d == dev}
    else:
        busy_segments = {pslot // geo.seg_slots for u in st.units for pdev, pslot in u[4] if pdev == dev}
    return geo.nseg - len(busy_segments)


def counts(geo, st, edef):
    return tuple(empty_segments(geo, st, dev, edef) for dev in range(geo.devices))


def units_in_r(geo, st, rspec, c0=None):
    """R 内（按记录 key：设备在覆盖集里、起点槽在区间里）的活落点；给 c0 时只取分配代 ≤ c0 的。"""
    devs, (lo, hi) = r_range(geo, rspec)
    found = []
    for unit in st.units:
        if any(pdev in devs and lo <= pslot < hi for pdev, pslot in unit[4]):
            if c0 is None or unit[1] <= c0:
                found.append(unit)
    return sorted(found, key=lambda u: min(ps for pd, ps in u[4] if pd in devs))


def r_physically_empty(geo, st, rspec):
    devs, (lo, hi) = r_range(geo, rspec)
    return not any(occupied(geo, st, dev, s, include_quarantine=False) and
                   any(pd == dev and ps <= s < ps + u[3] for u in st.units for pd, ps in u[4])
                   for dev in devs for s in range(lo, hi))


def target_reached(geo, st, rspec, c0):
    return not units_in_r(geo, st, rspec, c0)


def predicate(geo, cfg, st, rspec, segs0):
    """停机谓词。S=A：R 覆盖的每块盘 E − segs0 ≥ 1；S=B：全池之和；S=C：R 本身在每块盘上没有未释放记录。"""
    devs, _ = r_range(geo, rspec)
    if cfg.S == "A":
        now = counts(geo, st, cfg.E)
        return all(now[dev] - base >= 1 for dev, base in segs0)
    if cfg.S == "B":
        return sum(counts(geo, st, cfg.E)) - segs0 >= 1
    return not units_in_r(geo, st, rspec)


def segs0_value(geo, cfg, st, rspec):
    devs, _ = r_range(geo, rspec)
    now = counts(geo, st, cfg.E)
    if cfg.S == "A":
        return tuple((dev, now[dev]) for dev in sorted(devs))
    if cfg.S == "B":
        return sum(now)
    return None


# ---------------------------------------------------------------- 分配
def newest_valid(st):
    valid = [entry for entry in st.ring if not entry[2]]
    return max(valid, key=lambda entry: entry[0])


def snap_state(st, snap):
    units, defer, intent, ghost = snap
    return st._replace(units=units, defer=defer, intent=intent, ghost=ghost)


def alloc_exclusion(geo, cfg, st):
    """`R` 在意图期内不可作分配目的地（D26 已定项 1 第 3 条）；r3_view 决定分配器看在飞的意图还是已发布的意图。"""
    intent = st.intent if cfg.r3_view == "inflight" else newest_valid(st)[1][2]
    return None if intent is None else r_range(geo, intent[0])


def slot_blocked(geo, cfg, st, slot, exclusion):
    if any(occupied(geo, st, dev, slot) for dev in range(geo.devices)):
        return True
    if exclusion is not None:
        devs, (lo, hi) = exclusion
        if lo <= slot < hi:
            return True  # 两盘同号：落点在任何一块被覆盖的盘上落进 R 都不行
    return False


def alloc_user(geo, cfg, st):
    """D3 已定项 8 / 10 ③：最低的、偶数起点、两槽都空、不在开放段、不在 R。"""
    exclusion = alloc_exclusion(geo, cfg, st)
    for slot in range(0, total_slots(geo) - 1, 2):
        if st.open_seg is not None and st.open_seg * geo.seg_slots <= slot < (st.open_seg + 1) * geo.seg_slots:
            continue
        if not slot_blocked(geo, cfg, st, slot, exclusion) and not slot_blocked(geo, cfg, st, slot + 1, exclusion):
            return slot
    return None


def alloc_bump(geo, cfg, st):
    """D3 已定项 5 / 10 ①：提交内生块从开放段 bump；开放段满了或游标处不可用就开最低的全空段。返回 (槽, 开放段, 游标)。"""
    exclusion = alloc_exclusion(geo, cfg, st)
    open_seg, cursor = st.open_seg, st.cursor
    for _attempt in range(geo.nseg + 1):
        if open_seg is not None and cursor < (open_seg + 1) * geo.seg_slots and \
                not slot_blocked(geo, cfg, st, cursor, exclusion):
            return cursor, open_seg, cursor + 1
        open_seg = None
        for seg in range(geo.nseg):
            lo = seg * geo.seg_slots
            if all(not slot_blocked(geo, cfg, st, s, exclusion) for s in range(lo, lo + geo.seg_slots)):
                open_seg, cursor = seg, lo
                break
        if open_seg is None:
            return None
    return None


def release_unit(st, unit, rel_txg):
    units = tuple(sorted(u for u in st.units if u != unit))
    defer = tuple(sorted(st.defer + tuple((dev, slot, unit[3], rel_txg) for dev, slot in unit[4])))
    return st._replace(units=units, defer=defer)


def add_unit(st, unit):
    return st._replace(units=tuple(sorted(st.units + (unit,))))


def same_slot(geo, slot):
    return tuple((dev, slot) for dev in range(geo.devices))


# ---------------------------------------------------------------- 发布、崩溃、回退
def publish(geo, cfg, st):
    new = st
    for _ in range(geo.meta_per_publish):
        got = alloc_bump(geo, cfg, new)
        if got is None:
            return None
        slot, open_seg, cursor = got
        new = add_unit(new, ("M", st.txg, st.txg, 1, same_slot(geo, slot)))._replace(open_seg=open_seg, cursor=cursor)
    for unit in [u for u in new.units if u[0] == "M" and u[2] < st.txg]:
        new = release_unit(new, unit, st.txg)
    ring = (st.ring + ((st.txg, None, False),))[-geo.ring_k:]
    oldest_valid = min(entry[0] for entry in ring if not entry[2])
    defer = tuple(entry for entry in new.defer if entry[3] > oldest_valid)
    quarantine = new.quarantine if any(entry[2] for entry in ring) else frozenset()
    new = new._replace(defer=defer, quarantine=quarantine)
    snap = (new.units, new.defer, new.intent, new.ghost)
    ring = ring[:-1] + ((st.txg, snap, False),)
    return new._replace(ring=ring, txg=st.txg + 1)


def reload_intent(cfg, st, snap):
    if cfg.intent_rooted:
        return snap[2]
    newest_any = max(st.ring, key=lambda entry: entry[0])
    return newest_any[1][2]  # 意图不随根：取盘上最后一次写下的那份，不管它在哪条时间线上


def crash(geo, cfg, st):
    txg, snap, _ = newest_valid(st)
    intent = reload_intent(cfg, st, snap)
    return st._replace(txg=st.txg + 1, units=snap[0], defer=snap[1], intent=intent, ghost=snap[3],
                       run="running" if intent else "idle", open_seg=None, cursor=0)


def occupied_set(geo, st):
    return frozenset((dev, s) for dev in range(geo.devices) for s in range(total_slots(geo))
                     if occupied(geo, st, dev, s, include_quarantine=False))


def rollbacks(geo, cfg, st):
    valid = sorted((entry for entry in st.ring if not entry[2]), key=lambda entry: entry[0])
    for target_txg, snap, _ in valid[:-1]:
        ring = tuple((t, s, ab or t > target_txg) for t, s, ab in st.ring)
        old_state = snap_state(st, snap)
        abandoned_occupied = frozenset()
        for t, s, ab in ring:
            if ab:
                abandoned_occupied |= occupied_set(geo, snap_state(st, s))
        quarantine = st.quarantine | (abandoned_occupied - occupied_set(geo, old_state))
        probe = st._replace(ring=ring)
        intent = reload_intent(cfg, probe, snap)
        yield target_txg, st._replace(txg=st.txg + 1, units=snap[0], defer=snap[1], intent=intent, ghost=snap[3],
                                      run="running" if intent else "idle", open_seg=None, cursor=0,
                                      ring=ring, quarantine=quarantine)


# ---------------------------------------------------------------- 一批整理
def unit_start_in(geo, unit, rspec):
    devs, (lo, hi) = r_range(geo, rspec)
    return min(ps for pd, ps in unit[4] if pd in devs and lo <= ps < hi)


def quiesce_horizon(geo):
    """够开放段绕全池一圈、defer 排空：每段 seg_slots / meta 次发布 × 段数 + 环长。"""
    return geo.nseg * (geo.seg_slots // geo.meta_per_publish + 1) + geo.ring_k


def quiesce_predicate_never(geo, cfg, st, rspec, segs0, steps):
    """(b) 的「永远」按有界口径判：意图删掉之后只发布（元数据照常 bump），steps 次里停机谓词一次都没判真。"""
    probe = st
    for _ in range(steps):
        probe = publish(geo, cfg, probe)
        if probe is None:
            return False  # 发布本身 ENOSPC：卡死另记，不算 (b)
        if predicate(geo, cfg, probe, rspec, segs0):
            return False
    return True


def batch(geo, cfg, st):
    rspec, c0, segs0, rp, sign_true = st.intent
    devs, _ = r_range(geo, rspec)
    witnesses = []
    view = st
    if cfg.plan_view == "published":
        snap = newest_valid(st)[1]
        if snap[2] is not None and snap[2][:2] == st.intent[:2]:
            view = snap_state(st, snap)
    cands = units_in_r(geo, view, rspec, c0)
    if cfg.P in ("A_safe", "A_adv"):
        cands = [u for u in cands if unit_start_in(geo, u, rspec) >= rp]
    new = st
    if cands:
        unit = cands[0]
        planned = unit_start_in(geo, unit, rspec)
        done = False
        if unit not in st.units:
            if cfg.req4:
                done = True  # 硬要求 4：施加前先核，东西不在了就跳过
            else:
                witnesses.append(("d", f"槽 {planned} 的单元按计划再搬一次，而在飞态里它已经不在那儿（旧落点第二次进 defer）"))
                return None, witnesses
        else:
            got = alloc_user(geo, cfg, st) if unit[0] == "U" else alloc_bump(geo, cfg, st)
            if got is not None:
                if unit[0] == "U":
                    dest, open_seg, cursor = got, st.open_seg, st.cursor
                else:
                    dest, open_seg, cursor = got
                new = release_unit(st, unit, st.txg)
                new = add_unit(new, (unit[0], st.txg, unit[2], unit[3],
                                     tuple((dev, dest) for dev, _ in unit[4])))._replace(open_seg=open_seg, cursor=cursor)
                done = True
        if cfg.P == "A_safe":
            rp = planned + 1 if done else planned
        elif cfg.P == "A_adv":
            rp = planned + 1
    new = new._replace(intent=(rspec, c0, segs0, rp, sign_true))
    reached = target_reached(geo, new, rspec, c0)
    if cfg.P == "A_adv":
        delete_cond = not [u for u in units_in_r(geo, new, rspec, c0) if unit_start_in(geo, u, rspec) >= rp]
    else:
        delete_cond = reached
    pred = predicate(geo, cfg, new, rspec, segs0)
    now = counts(geo, new, cfg.E)

    def check_a():
        if any(now[dev] - sign_true[dev] <= 0 for dev in devs):
            witnesses.append(("a", f"谓词判有产出，而盘 {[d for d in devs if now[d] - sign_true[d] <= 0]} 的全空段数 "
                                   f"{[now[d] for d in sorted(devs)]} 不多于签发时 {[sign_true[d] for d in sorted(devs)]}"))

    def delete(state):
        if not target_reached(geo, state, rspec, c0):
            witnesses.append(("c", "意图被删而 R 里还有分配代 ≤ c0 的未释放落点"))
        elif not r_physically_empty(geo, state, rspec):
            witnesses.append(("c_overlap", "按记录 key 目标态已达成、意图被删，而一个起点在 R 外的活单元的跨度仍压在 R 里"
                                           "（「落点在 R 内」按重叠读就是 (c)）"))
        return state._replace(intent=None, ghost=(rspec, c0, segs0, sign_true), run="idle")

    if cfg.P in ("A_safe", "A_adv") and not cands and not reached:
        witnesses.append(("stuck", f"续做点 {rp} 之后一个要搬的都没有，而 R 里还有分配代 ≤ c0 的未释放落点"
                                   "（续做点与分配记录对不上：意图永不完成，R 一直被锁）"))

    def check_b(state):
        if reached and not units_in_r(geo, state, rspec) and not pred:
            horizon = quiesce_horizon(geo)
            if quiesce_predicate_never(geo, cfg, state, rspec, segs0, horizon):
                witnesses.append(("b", f"R 已无未释放记录，停机谓词在之后 {horizon} 次发布里一次都没判真"))

    if cfg.SP == "SD":
        if pred:
            check_a()
            if not reached:
                witnesses.append(("a_prime", "谓词判有产出时 R 里还有要搬的落点（提前停机）"))
            new = delete(new)
        elif delete_cond:
            new = delete(new)
            check_b(new)
    elif cfg.SP == "SK":
        if delete_cond:
            new = delete(new)
            if pred:
                check_a()
            else:
                check_b(new)
        elif pred:
            check_a()
            witnesses.append(("a_prime", "谓词判有产出时 R 里还有要搬的落点（提前停机）"))
            witnesses.append(("stall", "停机不删意图：R 继续被锁、续做再判一次谓词仍真，意图永不完成"))
            new = new._replace(run="stopped")
    else:  # AC：意图照目标态删，谓词只在删的那一刻判一轮停不停
        if delete_cond:
            new = delete(new)
            if pred:
                check_a()
            else:
                check_b(new)
    return new, witnesses


# ---------------------------------------------------------------- 事件与搜索
def rspec_for(cfg, seg, geo, dev=0):
    if isinstance(seg, tuple):
        return seg  # 场景直接给出的 R（例如 R丙 的非对齐区间）
    if cfg.R == "A":
        return ("A", seg)
    if cfg.R == "B":
        return ("B", dev, seg)
    return ("C", seg * geo.seg_slots, geo.seg_slots)


def events(geo, cfg, st, alphabet, sign_segments):
    if "SIGN" in alphabet and st.intent is None:
        for seg in sign_segments:
            rspec = rspec_for(cfg, seg, geo)
            if rspec[0] != cfg.R:
                continue
            if not units_in_r(geo, st, rspec):
                continue
            c0 = st.txg if cfg.c0 == "inflight" else st.txg - 1
            intent = (rspec, c0, segs0_value(geo, cfg, st, rspec), r_range(geo, rspec)[1][0] if cfg.P != "B" else None,
                      counts(geo, st, cfg.E))
            yield f"SIGN(R={rspec},c0={c0})", st._replace(intent=intent, run="running"), []
    if "BATCH" in alphabet and st.intent is not None and st.run == "running":
        new, wits = batch(geo, cfg, st)
        yield "BATCH", new, wits
    if "WRITE" in alphabet and sum(1 for u in st.units if u[0] == "U") < geo.max_user:
        slot = alloc_user(geo, cfg, st)
        if slot is not None:
            yield f"WRITE@{slot}", add_unit(st, ("U", st.txg, st.txg, 2, same_slot(geo, slot))), []
    if "DELETE" in alphabet:
        for unit in st.units:
            if unit[0] in ("U", "I"):
                yield f"DELETE {unit[0]}@{unit[4][0][1]}", release_unit(st, unit, st.txg), []
    if "PUBLISH" in alphabet:
        new = publish(geo, cfg, st)
        if new is not None:
            yield f"PUBLISH(txg={st.txg})", new, []
    if "CRASH" in alphabet:
        yield f"CRASH(新 txg={st.txg + 1})", crash(geo, cfg, st), []
    if "ROLLBACK" in alphabet:
        for target_txg, new in rollbacks(geo, cfg, st):
            yield f"ROLLBACK(→根 {target_txg},新 txg={st.txg + 1})", new, []
    for mode in ("SWITCH_KEEP", "SWITCH_RECOMPUTE"):
        if mode in alphabet and any(u[1] == st.txg for u in st.units):
            units = tuple(sorted((u[0], st.txg + 1 if u[1] == st.txg else u[1]) + u[2:] for u in st.units))
            defer = tuple(sorted(e[:3] + (st.txg + 1 if e[3] == st.txg else e[3],) for e in st.defer))
            intent = st.intent
            if mode == "SWITCH_RECOMPUTE" and intent is not None and intent[1] == st.txg:
                intent = (intent[0], st.txg + 1) + intent[2:]
            yield f"{mode}(在飞 checkpoint {st.txg} 按 {st.txg + 1} 重做)", st._replace(
                units=units, defer=defer, intent=intent, txg=st.txg + 1), []


def check_r3(geo, st):
    if st is None or st.intent is None:
        return []
    rspec, c0 = st.intent[0], st.intent[1]
    late = [u for u in units_in_r(geo, st, rspec) if u[1] > c0]
    if late:
        return [("R3", f"意图还活着，R 里有分配代 {late[0][1]} > c0={c0} 的活落点（目标态不罩它）")]
    return []


def search(geo, cfg, init, alphabet, sign_segments, depth, max_states=400000):
    found = {}
    visited = {init}
    frontier = deque([(init, ())])
    explored_depth = 0
    while frontier:
        st, path = frontier.popleft()
        if len(path) >= depth:
            continue
        explored_depth = max(explored_depth, len(path) + 1)
        for label, new, wits in events(geo, cfg, st, alphabet, sign_segments):
            for kind, detail in wits + check_r3(geo, new):
                if kind not in found:
                    found[kind] = (path + (label,), detail)
            if new is not None and new not in visited:
                if len(visited) >= max_states:
                    return found, len(visited), explored_depth, False
                visited.add(new)
                frontier.append((new, path + (label,)))
    return found, len(visited), explored_depth, True


def build(geo, layout, open_seg=None, cursor=0, txg=2):
    units = tuple(sorted((kind, gen, gen, span, same_slot(geo, slot)) for kind, slot, span, gen in layout))
    snap = (units, (), None, None)
    return State(txg=txg, units=units, defer=(), intent=None, ghost=None, run="idle", open_seg=open_seg,
                 cursor=cursor, ring=((txg - 1, snap, False),), quarantine=frozenset())
