"""I-3.1 RCW 三判与「两种数法各取宽」的逐行转写（推的：照开工快照 crates/singlefs-checker/src/walk.rs 手抄，不是编出来的二进制）。

转写对象（快照 walk.rs 的行号）：
  allocated_statistic_bounds_of_a_device                                   10279 起
  allocated_statistic_lies_within_its_bounds                               10336 起
  records_with_the_spans_the_span_invariant_reddens_set_to_their_unit_classes 10360 起
  allocated_statistic_lies_within_the_bounds_of_either_span_reading       10391 起
  check_pool_image 里 I-3.1 那一段（三判不成立才取宽，取宽成立报部分没判） 21317 到 21362
槽数一律按槽算（Rust 里乘 SLOT_BYTES 再比，两边同乘，不改大小关系）。
"""
from collections import namedtuple

Record = namedtuple("Record", "device slot span generation released")


def bounds_of_a_device(device, records, walked_slots, quarantined_count, theta_high, t_root):
    """walked_slots：{(盘, 槽)}；交回 (|E(θ_high)|, |E(t)|, |Ω ∪ U|, 遍历走到而不在 E(θ_high) 里的槽(升序))。"""
    def counted_at(threshold):
        out = set()
        for r in records:
            if r.device == device and ((not r.released) or r.generation > threshold):
                out.update(range(r.slot, r.slot + r.span))
        return out
    e_high = counted_at(theta_high)
    still_deferred_outside = set()
    for r in records:
        if r.device == device and r.released and r.generation > t_root:
            still_deferred_outside.update(s for s in range(r.slot, r.slot + r.span)
                                          if (device, s) not in walked_slots)
    walked_on_device = [s for (d, s) in walked_slots if d == device]
    return (len(e_high), len(counted_at(t_root)),
            len(walked_on_device) + quarantined_count + len(still_deferred_outside),
            sorted(s for s in walked_on_device if s not in e_high))


def within_bounds(allocated, b):
    if allocated is None:
        return False
    e_high, e_t, walked_u, outside = b
    return allocated >= e_high and not outside and allocated <= min(e_t, walked_u)


def set_to_unit_classes(records, class_slots_at, comparison="ne"):
    """class_slots_at(record) -> 起点槽单元那一类的槽数，头用不了交回 None。今天的比较是 `!=`（10372）；
    comparison="gt" 是正文问的「只放记录比单元窄」那一改（类的槽数 > 记录跨度才改记）。"""
    changed = False
    out = []
    for r in records:
        if r.released:
            out.append(r)
            continue
        c = class_slots_at(r)
        if c is not None and (c != r.span if comparison == "ne" else c > r.span):
            changed = True
            out.append(r._replace(span=c))
        else:
            out.append(r)
    return out if changed else None


def within_either(allocated, rec, unit):
    if allocated is None:
        return False
    lower = allocated >= min(rec[0], unit[0])
    # 10408–10416：按记录自述走到而不在记录里的每个槽，都不在改记那一份的「走到而不在记录里」里
    every_walked = all(s not in unit[3] for s in rec[3])
    upper = allocated <= min(max(rec[1], unit[1]), rec[2])
    return lower and every_walked and upper


def i31_verdict(device, records, walked_slots, quarantined_count, theta_high, t_root,
                allocated, class_slots_at, comparison="ne"):
    """交回 'holds' / 'violated' / 'partial'（照 21317–21362 的分支）。"""
    b = bounds_of_a_device(device, records, walked_slots, quarantined_count, theta_high, t_root)
    if within_bounds(allocated, b):
        return "holds"
    on_device = [r for r in records if r.device == device]
    reset = set_to_unit_classes(on_device, class_slots_at, comparison)
    if reset is not None:
        ub = bounds_of_a_device(device, reset, walked_slots, quarantined_count, theta_high, t_root)
        if within_either(allocated, b, ub):
            return "partial"
    return "violated"
