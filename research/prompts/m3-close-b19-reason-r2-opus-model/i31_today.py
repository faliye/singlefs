"""I-3.1 取宽那一支今天的写法（只认整份改记读法 + 逐槽两判）的槽级转写，推的模型，不是编出来的二进制。

转写对象（开工快照 crates/singlefs-checker/src/walk.rs，sha256 b533596e…）：
  allocated_statistic_bounds_of_a_device                                       10398 起
  allocated_statistic_lies_within_its_bounds                                   10467 起
  records_with_the_spans_the_span_invariant_reddens_set_to_their_unit_classes  10491 起
  allocated_statistic_lies_within_the_bounds_of_the_unit_class_reading         10523 起
  check_pool_image 里 I-3.1 那一段                                             21389 到 21511
  quarantined_slots_exempted_per_device（只认记号，起点不被引用的未释放记录）  10180 起
一块盘、槽数按槽算（Rust 里乘 SLOT_BYTES 再比，两边同乘不改大小关系）。
记录是 (slot, span, released, gen, marked)；单元是 {起点: 类的槽数}，walked 是遍历走到的单元起点集合。
"""
from collections import namedtuple

Rec = namedtuple("Rec", "slot span released gen marked")


def covered(r):
    return range(r.slot, r.slot + r.span)


def quarantine(records, referenced_starts):
    """交回 (隔离豁免的槽集合, 豁免槽数)；按记录自述的跨度（10197–10215）。"""
    q, n = set(), 0
    for r in records:
        if r.released or r.slot in referenced_starts:
            continue
        if r.marked:
            q.update(covered(r))
            n += r.span
    return q, n


def bounds(records, walked_slots, walked_count, q_slots, q_count, theta_high, t):
    def counted_at(th):
        out = set()
        for r in records:
            if (not r.released) or r.gen > th:
                out.update(covered(r))
        return out
    e_h = counted_at(theta_high)
    e_t = counted_at(t)
    d = set()
    for r in records:
        if r.released and r.gen > t:
            d.update(s for s in covered(r) if s not in walked_slots)
    wc = walked_count + q_count + len(d)
    walked_outside = sorted(s for s in walked_slots if s not in e_h)
    et_outside = sorted(s for s in e_t if s not in walked_slots and s not in q_slots and s not in d)
    return dict(e_h=len(e_h), e_t=len(e_t), wc=wc, walked_outside=walked_outside,
                et_outside=et_outside, e_h_set=e_h, e_t_set=e_t, d=d)


def self_interval(b):
    """按记录自述三判成立的记账区间 [lo, hi]（槽数）；(P) 不成立交回 None。"""
    if b["walked_outside"]:
        return None
    lo, hi = b["e_h"], min(b["e_t"], b["wc"])
    return (lo, hi) if lo <= hi else None


def unit_class_interval(b):
    """整份改记读法四判成立的记账区间；逐槽两判有一判不成立交回 None。"""
    if b["walked_outside"] or b["et_outside"]:
        return None
    lo, hi = b["e_h"], min(b["e_t"], b["wc"])
    return (lo, hi) if lo <= hi else None


def set_to_unit_classes(records, class_at):
    """10491 起：未释放、起点头用得了、类的槽数 != 跨度的改记；一条都没改交回 None。"""
    changed, out = False, []
    for r in records:
        if r.released:
            out.append(r)
            continue
        c = class_at(r.slot)
        if c is not None and c != r.span:
            changed = True
            out.append(r._replace(span=c))
        else:
            out.append(r)
    return out if changed else None


def i31(records, units, walked_starts, theta_high, t, acct):
    """交回 ('holds'|'partial'|'violated', 自述那一份的界, 改记那一份的界或 None)。"""
    walked_slots = {s for u in walked_starts for s in range(u, u + units[u])}
    walked_count = sum(units[u] for u in walked_starts)
    q_slots, q_count = quarantine(records, set(walked_starts))
    b = bounds(records, walked_slots, walked_count, q_slots, q_count, theta_high, t)
    si = self_interval(b)
    if si is not None and si[0] <= acct <= si[1]:
        return "holds", b, None
    reset = set_to_unit_classes(records, units.get)
    if reset is None:
        return "violated", b, None
    ub = bounds(reset, walked_slots, walked_count, q_slots, q_count, theta_high, t)
    ui = unit_class_interval(ub)
    if ui is not None and ui[0] <= acct <= ui[1]:
        return "partial", b, ub
    return "violated", b, ub
