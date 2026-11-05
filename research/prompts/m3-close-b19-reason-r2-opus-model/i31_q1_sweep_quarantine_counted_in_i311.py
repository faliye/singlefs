"""i31_q1_sweep.py 的 I-3.11 判据补上隔离豁免那一项之后重数，推的模型（承接腿 m3-close-b19-reason-r2-opus-c2 写）。
i31_q1_sweep.other_reds 的 I-3.11 写成「记账 − defer == 走到的槽数」，少了真 checker 加上的隔离豁免槽数
（开工快照 crates/singlefs-checker/src/walk.rs 21557–21571：最新根走读那一份 + quarantined_exempted_newest，再与 defer 相加比已分配）。
这里只把 I-3.11 换成「记账 == defer + 走到的槽数 + 隔离豁免槽数」（豁免照 i31_today.quarantine：按自述跨度、只收起点不被引用的带记号未释放记录），
别的不变量、世界取法、I-3.1 转写都原样从 i31_q1_sweep / i31_today 导入。每一类同时数原判据（model_）与补上之后（quarantine_counted_）两份，
原判据那一份应与 out/i31_sweep_4.txt 同名计数相同（锚）。
用法：python3 i31_q1_sweep_quarantine_counted_in_i311.py mirrors
      python3 i31_q1_sweep_quarantine_counted_in_i311.py sweep N
"""
import itertools
import sys
import i31_q1_sweep as base
from i31_today import Rec, i31, quarantine

LOW, MID, HIGH = base.LOW, base.MID, base.HIGH


def reds_with_quarantine_counted(records, units, walked, theta_high, t, acct):
    reds = [r for r in base.other_reds(records, units, walked, theta_high, t, acct) if r != "I-3.11"]
    walked_slots = {s for u in walked for s in range(u, u + units[u])}
    defer = sum(r.span for r in records if r.released and r.gen > t)
    _, q_count = quarantine(records, set(walked))
    if acct != defer + len(walked_slots) + q_count:
        reds.append("I-3.11")
    return reds


def between_on_walked_unit(records, units, walked, theta_high, t):
    """改记加宽多出来的槽与一条释放代在 (t, θ_high] 的已释放记录相交的那几条改记记录里，有没有起点单元被走到的。
    交回 (有走到的, 有没走到的)。"""
    reset = base.set_to_unit_classes(records, units.get) or records
    on_walked, on_unwalked = False, False
    for a, ra in zip(records, reset):
        if ra.span == a.span:
            continue
        added = set(range(ra.slot + a.span, ra.slot + ra.span))
        for b in records:
            if b is a or not b.released or not (t < b.gen <= theta_high):
                continue
            if added & set(range(b.slot, b.slot + b.span)):
                if a.slot in walked:
                    on_walked = True
                else:
                    on_unwalked = True
    return on_walked, on_unwalked


def show(name, records, units, walked, theta_high, t, acct):
    v, _, _ = i31(records, units, walked, theta_high, t, acct)
    print(f"name=mirror_quarantine_counted case={name} I-3.1={v} I-3.18_red_at={base.i318_red(records, units)}"
          f" others_red_model={base.other_reds(records, units, walked, theta_high, t, acct)}"
          f" others_red_quarantine_counted={reds_with_quarantine_counted(records, units, walked, theta_high, t, acct)}"
          f" overlap_after_rewrite={base.overlap_after_rewrite(records, units)}"
          f" between_on_walked_unit={between_on_walked_unit(records, units, walked, theta_high, t)}")


def mirrors():
    # out/i31_mirrors.txt 那六张
    show("tail_on_a_record_released_between_t_and_theta_high", [Rec(0, 1, False, 1, False), Rec(1, 1, True, MID, False)],
         {0: 2}, {0}, 2, 1, 2)
    show("same_with_release_generation_above_theta_high", [Rec(0, 1, False, 1, False), Rec(1, 1, True, HIGH, False)],
         {0: 2}, {0}, 2, 1, 2)
    show("same_with_t_equal_to_theta_high", [Rec(0, 1, False, 1, False), Rec(1, 1, True, MID, False)],
         {0: 2}, {0}, 2, 2, 2)
    show("round1_wide_record_over_a_neighbor_and_a_free_slot", [Rec(0, 3, False, 1, False)], {0: 1, 1: 1}, {0, 1}, 2, 2, 2)
    show("round1_narrow_record_plus_headerless_leak", [Rec(0, 1, False, 1, False), Rec(5, 1, False, 1, False)],
         {0: 2}, {0}, 2, 2, 2)
    show("c944_wide_record_exactly_over_a_neighbor_without_its_own_record", [Rec(0, 2, False, 1, False)], {0: 1, 1: 1},
         {0, 1}, 2, 2, 2)
    # out/i31_sweep_4.txt 第 2 行与第 5 行那两张例子（kind=between、kind=widened）
    show("sweep_example_between", [Rec(1, 1, False, 1, True), Rec(2, 1, False, 1, False), Rec(3, 1, True, MID, False)],
         {2: 2}, {2}, 2, 1, 3)
    show("sweep_example_widened", [Rec(0, 1, False, 1, True), Rec(1, 1, True, MID, False), Rec(2, 1, False, 1, False),
                                   Rec(3, 1, True, MID, False)], {0: 2, 2: 2}, {2}, 2, 1, 4)
    # 承接腿手造：隔离单元（没走到、带记号）的窄记录，尾槽落在释放代在 (t, θ_high] 的已释放记录上；另一个走到的窄记录让自述那一份不成立
    show("quarantined_unit_tail_on_a_record_released_between_t_and_theta_high",
         [Rec(0, 1, False, 1, False), Rec(2, 1, False, 1, True), Rec(3, 1, True, MID, False)], {0: 2, 2: 2}, {0}, 2, 1, 4)
    # 承接腿手造：隔离单元的窄记录，尾槽上另一条带记号的未释放记录（改宽罩进隔离豁免的槽）
    show("quarantined_unit_tail_on_another_quarantined_record",
         [Rec(0, 1, False, 1, False), Rec(2, 1, False, 1, True), Rec(3, 1, False, 1, True)], {0: 2, 2: 2}, {0}, 2, 1, 4)


def sweep(n):
    rec_opts = [None]
    for k in (1, 2):
        rec_opts += [(k, False, 1, False), (k, True, LOW, False), (k, True, MID, False), (k, True, HIGH, False)]
    rec_opts.append((1, False, 1, True))
    names = ["worlds", "partial"]
    for judge in ("model", "quarantine_counted"):
        names += [f"{judge}_partial_overlap_after_rewrite_nothing_else_red",
                  f"{judge}_partial_overlap_with_a_record_released_between_t_and_theta_nothing_else_red",
                  f"{judge}_between_nothing_else_red_on_a_walked_unit",
                  f"{judge}_between_nothing_else_red_on_an_unwalked_unit_only",
                  f"{judge}_partial_widened_nothing_else_red",
                  f"{judge}_partial_widened_into_quarantine_nothing_else_red",
                  f"{judge}_partial_widened_into_deferred_nothing_else_red",
                  f"{judge}_holds_c944_nothing_else_red"]
    cnt = {k: 0 for k in names}
    example = {}
    for lay in base.layouts(n):
        units = base.units_of(lay)
        ustarts = sorted(units)
        for wmask in itertools.product((False, True), repeat=len(ustarts)):
            walked = {u for u, w in zip(ustarts, wmask) if w}
            walked_slots = {s for u in walked for s in range(u, u + units[u])}
            per_slot = [[o for o in rec_opts if o is None or s + o[0] <= n] for s in range(n)]
            for choice in itertools.product(*per_slot):
                records = [Rec(s, o[0], o[1], o[2], o[3]) for s, o in enumerate(choice) if o is not None]
                q_slots, _ = quarantine(records, set(walked))
                for theta_high, t in ((2, 1), (2, 2)):
                    for acct in range(n + 2):
                        cnt["worlds"] += 1
                        v, _, _ = i31(records, units, walked, theta_high, t, acct)
                        if v not in ("partial", "holds"):
                            continue
                        reds = {"model": base.other_reds(records, units, walked, theta_high, t, acct),
                                "quarantine_counted": reds_with_quarantine_counted(records, units, walked, theta_high, t, acct)}
                        if v == "partial":
                            cnt["partial"] += 1
                            ov = base.overlap_after_rewrite(records, units)
                            partners = base.overlap_partners(records, units, theta_high, t) if ov else set()
                            wi = base.widened_into(records, units, walked, t)
                            for judge, rd in reds.items():
                                if rd:
                                    continue
                                world = (records, units, sorted(walked), theta_high, t, acct)
                                if ov:
                                    cnt[f"{judge}_partial_overlap_after_rewrite_nothing_else_red"] += 1
                                    if "between" in partners:
                                        cnt[f"{judge}_partial_overlap_with_a_record_released_between_t_and_theta_nothing_else_red"] += 1
                                        on_w, on_u = between_on_walked_unit(records, units, walked, theta_high, t)
                                        if on_w:
                                            cnt[f"{judge}_between_nothing_else_red_on_a_walked_unit"] += 1
                                            example.setdefault((judge, "between_walked"), world)
                                        elif on_u:
                                            cnt[f"{judge}_between_nothing_else_red_on_an_unwalked_unit_only"] += 1
                                            example.setdefault((judge, "between_unwalked"), world)
                                if wi:
                                    cnt[f"{judge}_partial_widened_nothing_else_red"] += 1
                                    if any(s in q_slots for s in wi):
                                        cnt[f"{judge}_partial_widened_into_quarantine_nothing_else_red"] += 1
                                        example.setdefault((judge, "widened_quarantine"), world + (wi,))
                                    if any(s not in q_slots for s in wi):
                                        cnt[f"{judge}_partial_widened_into_deferred_nothing_else_red"] += 1
                                        example.setdefault((judge, "widened_deferred"), world + (wi,))
                        elif "I-5.4" not in reds["model"]:
                            cs = base.c944_starts(records, units, walked, theta_high)
                            if cs:
                                for judge, rd in reds.items():
                                    if not rd:
                                        cnt[f"{judge}_holds_c944_nothing_else_red"] += 1
                                        example.setdefault((judge, "c944"), (records, units, sorted(walked), theta_high, t, acct, cs))
    print(f"name=i31_q1_sweep_quarantine_counted N={n} " + " ".join(f"{k}={cnt[k]}" for k in names))
    for k in sorted(example):
        print(f"name=i31_q1_sweep_quarantine_counted_example N={n} judge={k[0]} kind={k[1]} world={example[k]}")


if __name__ == "__main__":
    if sys.argv[1] == "mirrors":
        mirrors()
    else:
        sweep(int(sys.argv[2]))
