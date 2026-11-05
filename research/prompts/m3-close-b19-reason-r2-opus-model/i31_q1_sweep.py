"""Q1 第 1 问（改记本身带出来的漏判）与 Q2（C944 归属候选在小世界上抓到什么）的槽级小世界，推的模型。
用法：python3 i31_q1_sweep.py mirrors
      python3 i31_q1_sweep.py sweep N
世界：一块盘 N 槽；单元（一槽 / 两槽，头都用得了）与全零槽的每种排法；每个单元走到不走到；每一槽上记录取
  无 / 未释放跨 k / 已释放跨 k 释放代 LOW / 已释放跨 k 释放代 MID / 已释放跨 k 释放代 HIGH / 未释放带隔离记号跨 1（k = 1, 2）；
门槛两种：t < θ_high（t = 1、θ_high = 2，MID = 2 落在 (t, θ_high]）与 t = θ_high = 2；LOW = 1 不高于 t（已回收、条目没删），HIGH = 3。
记账取 0..N+1。别的不变量按条文写成判据（I-5.4 按自述跨度、I-3.15「已死」、I-3.17 ②、I-3.9 已释放记录起在走到的单元起点、
I-3.11 用写者自洽的 defer 行 = 已释放且释放代高于 t 的记录跨度之和）。
"""
import itertools
import sys
from i31_today import Rec, covered, i31, set_to_unit_classes, quarantine

LOW, MID, HIGH = 1, 2, 3


def layouts(n):
    if n <= 0:
        yield []
        return
    for rest in layouts(n - 1):
        yield [None] + rest
    for c in (1, 2):
        if c <= n:
            for rest in layouts(n - c):
                yield [c] + [None] * (c - 1) + rest


def units_of(layout):
    out, s = {}, 0
    while s < len(layout):
        if layout[s] is None:
            s += 1
        else:
            out[s] = layout[s]
            s += layout[s]
    return out


def other_reds(records, units, walked, theta_high, t, acct):
    """交回别的不变量里红了的那几条的名字。"""
    walked_slots = {s for u in walked for s in range(u, u + units[u])}
    reds = []
    cover = {}
    for r in records:
        for s in covered(r):
            cover[s] = cover.get(s, 0) + 1
    if any(v > 1 for v in cover.values()):
        reds.append("I-5.4")
    if any((not r.released) and (not r.marked) and r.slot in units and r.slot not in walked for r in records):
        reds.append("I-3.15")
    if any(r.marked and any(s in walked_slots for s in covered(r)) for r in records):
        reds.append("I-3.17")
    if any(r.released and r.slot in walked for r in records):
        reds.append("I-3.9")
    defer = sum(r.span for r in records if r.released and r.gen > t)
    if acct - defer != len(walked_slots):
        reds.append("I-3.11")
    return reds


def i318_red(records, units):
    return [r.slot for r in records if (not r.released) and units.get(r.slot) is not None and units[r.slot] != r.span]


def overlap_after_rewrite(records, units):
    reset = set_to_unit_classes(records, units.get) or records
    cover = {}
    for r in reset:
        for s in covered(r):
            cover.setdefault(s, []).append(r.slot)
    return sorted(s for s, who in cover.items() if len(who) > 1)


def overlap_partners(records, units, theta_high, t):
    """改记之后与别的记录相交的那几条「别的记录」各属哪一类：between（已释放、释放代在 (t, θ_high]）、reclaimed（已释放、释放代不高于 t）、
    other（未释放或释放代高于 θ_high）。"""
    reset = set_to_unit_classes(records, units.get) or records
    kinds = set()
    for a, ra in zip(records, reset):
        if ra.span == a.span:
            continue
        added = set(range(ra.slot + a.span, ra.slot + ra.span))
        for b in records:
            if b is a or not (added & set(covered(b))):
                continue
            if b.released and t < b.gen <= theta_high:
                kinds.add("between")
            elif b.released and b.gen <= t:
                kinds.add("reclaimed")
            else:
                kinds.add("other")
    return kinds


def widened_into(records, units, walked, t):
    """改记加宽多出来的槽里落在隔离豁免或 U（已释放、释放代高于 t、没走到）里的。"""
    walked_slots = {s for u in walked for s in range(u, u + units[u])}
    q, _ = quarantine(records, set(walked))
    d = set()
    for r in records:
        if r.released and r.gen > t:
            d.update(s for s in covered(r) if s not in walked_slots)
    out = []
    for r in records:
        c = units.get(r.slot)
        if (not r.released) and c is not None and c > r.span:
            for s in range(r.slot + r.span, r.slot + c):
                if s in q or s in d:
                    out.append(s)
    return out


def c944_starts(records, units, walked, theta_high):
    """遍历走到的单元起点上没有一条起点就在那里、未释放或释放代高于 θ_high 的记录（候选丙判红的那几个起点）。"""
    starts = {r.slot for r in records if (not r.released) or r.gen > theta_high}
    return sorted(u for u in walked if u not in starts)


def show(name, records, units, walked, theta_high, t, acct):
    v, b, ub = i31(records, units, walked, theta_high, t, acct)
    print(f"name=mirror case={name} I-3.1={v} self=(E_high={b['e_h']},E_t={b['e_t']},walked_and_deferred={b['wc']},"
          f"walked_outside={b['walked_outside']}) unit_class="
          + (f"(E_high={ub['e_h']},E_t={ub['e_t']},walked_and_deferred={ub['wc']},walked_outside={ub['walked_outside']},"
             f"E_t_outside={ub['et_outside']})" if ub else "none")
          + f" I-3.18_red_at={i318_red(records, units)} others_red={other_reds(records, units, walked, theta_high, t, acct)}"
          f" overlap_after_rewrite={overlap_after_rewrite(records, units)} c944_starts={c944_starts(records, units, walked, theta_high)}")


def mirrors():
    # H1：两槽单元 X 起在 0、窄记录 A 跨 1；X 的尾槽 1 上一条已释放记录 B，释放代 2 落在 (t, θ_high] = (1, 2]；记账 2 槽
    show("tail_on_a_record_released_between_t_and_theta_high", [Rec(0, 1, False, 1, False), Rec(1, 1, True, MID, False)],
         {0: 2}, {0}, 2, 1, 2)
    show("same_with_release_generation_above_theta_high", [Rec(0, 1, False, 1, False), Rec(1, 1, True, HIGH, False)],
         {0: 2}, {0}, 2, 1, 2)
    show("same_with_t_equal_to_theta_high", [Rec(0, 1, False, 1, False), Rec(1, 1, True, MID, False)],
         {0: 2}, {0}, 2, 2, 2)
    # 第一轮两张（今天应判红）
    show("round1_wide_record_over_a_neighbor_and_a_free_slot", [Rec(0, 3, False, 1, False)], {0: 1, 1: 1}, {0, 1}, 2, 2, 2)
    show("round1_narrow_record_plus_headerless_leak", [Rec(0, 1, False, 1, False), Rec(5, 1, False, 1, False)],
         {0: 2}, {0}, 2, 2, 2)
    # C944：宽记录恰好罩住没有自己记录的邻居，记账照单元数
    show("c944_wide_record_exactly_over_a_neighbor_without_its_own_record", [Rec(0, 2, False, 1, False)], {0: 1, 1: 1},
         {0, 1}, 2, 2, 2)


def sweep(n):
    rec_opts = [None]
    for k in (1, 2):
        rec_opts += [(k, False, 1, False), (k, True, LOW, False), (k, True, MID, False), (k, True, HIGH, False)]
    rec_opts.append((1, False, 1, True))
    cnt = dict(worlds=0, partial=0, partial_overlap_after_rewrite=0, partial_overlap_after_rewrite_nothing_else_red=0,
               partial_overlap_with_a_record_released_between_t_and_theta=0,
               partial_overlap_with_a_record_released_between_t_and_theta_nothing_else_red=0,
               partial_overlap_with_a_reclaimed_record_only=0,
               partial_widened_into_quarantine_or_deferred=0, partial_widened_nothing_else_red=0,
               holds_c944=0, holds_c944_only_released_above_theta_at_the_start=0, holds_c944_nothing_else_red=0)
    example = {}
    for lay in layouts(n):
        units = units_of(lay)
        ustarts = sorted(units)
        for wmask in itertools.product((False, True), repeat=len(ustarts)):
            walked = {u for u, w in zip(ustarts, wmask) if w}
            per_slot = []
            for s in range(n):
                per_slot.append([o for o in rec_opts if o is None or s + o[0] <= n])
            for choice in itertools.product(*per_slot):
                records = [Rec(s, o[0], o[1], o[2], o[3]) for s, o in enumerate(choice) if o is not None]
                for theta_high, t in ((2, 1), (2, 2)):
                    for acct in range(n + 2):
                        cnt["worlds"] += 1
                        v, _, _ = i31(records, units, walked, theta_high, t, acct)
                        reds = other_reds(records, units, walked, theta_high, t, acct)
                        if v == "partial":
                            cnt["partial"] += 1
                            ov = overlap_after_rewrite(records, units)
                            if ov and "I-5.4" not in reds:
                                cnt["partial_overlap_after_rewrite"] += 1
                                partners = overlap_partners(records, units, theta_high, t)
                                if "between" in partners:
                                    cnt["partial_overlap_with_a_record_released_between_t_and_theta"] += 1
                                    if not reds:
                                        cnt["partial_overlap_with_a_record_released_between_t_and_theta_nothing_else_red"] += 1
                                        example.setdefault("between", (records, units, sorted(walked), theta_high, t, acct, ov))
                                elif partners == {"reclaimed"}:
                                    cnt["partial_overlap_with_a_reclaimed_record_only"] += 1
                                if not reds:
                                    cnt["partial_overlap_after_rewrite_nothing_else_red"] += 1
                                    example.setdefault("overlap", (records, units, sorted(walked), theta_high, t, acct, ov))
                            wi = widened_into(records, units, walked, t)
                            if wi:
                                cnt["partial_widened_into_quarantine_or_deferred"] += 1
                                if not reds:
                                    cnt["partial_widened_nothing_else_red"] += 1
                                    example.setdefault("widened", (records, units, sorted(walked), theta_high, t, acct, wi))
                        elif v == "holds" and "I-5.4" not in reds:
                            cs = c944_starts(records, units, walked, theta_high)
                            if cs:
                                cnt["holds_c944"] += 1
                                if not reds:
                                    cnt["holds_c944_nothing_else_red"] += 1
                                    example.setdefault("c944", (records, units, sorted(walked), theta_high, t, acct, cs))
                            starts_unrel = {r.slot for r in records if not r.released}
                            if any(u not in starts_unrel for u in walked) and not cs:
                                cnt["holds_c944_only_released_above_theta_at_the_start"] += 1
    print(f"name=i31_q1_sweep N={n} " + " ".join(f"{k}={v}" for k, v in cnt.items()))
    for k in sorted(example):
        print(f"name=i31_q1_sweep_example N={n} kind={k} world={example[k]}")


if __name__ == "__main__":
    if sys.argv[1] == "mirrors":
        mirrors()
    else:
        sweep(int(sys.argv[2]))
