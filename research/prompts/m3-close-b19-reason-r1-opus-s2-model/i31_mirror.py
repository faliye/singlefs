"""Q3 窄方向：I-3.1 取宽后成立、而真病没被别的不变量说全的镜像（槽级模型，推的）。

先拿快照 walk.rs 25601–25720 那五条单测的输入复跑转写，输出要与单测断言逐条相同（校验转写没抄错）；
再判镜像与两张对照：每张列出 I-3.1、I-3.18、I-3.15、I-3.10、I-5.4、I-3.11、I-5.2、I-3.16 的判定。
I-3.1 之外几条按 05-空间记账.md / 07-分配器.md 的条文与 walk.rs 判法手写，只到槽级。
用法：python3 i31_mirror.py [--fix greater|per-slot]
"""
import sys
from i31_rules import Record, bounds_of_a_device, within_bounds, set_to_unit_classes, within_either, i31_verdict

FIX = sys.argv[2] if len(sys.argv) > 2 and sys.argv[1] == "--fix" else "none"


def rec(slot, span, gen=1, released=False):
    return Record(0, slot, span, gen, released)


COMPARISON = "gt" if FIX == "greater" else "ne"


def per_slot_fix_reddens(reset, walked):
    """改法「逐槽上沿」（攻方提的，只在本模型上量过、被攻过零轮）：取宽那一支另要改记那一份 E(t) 每个槽都被遍历走到。"""
    e_unit_t = {s for r in reset if (not r.released) or r.generation > 5 for s in range(r.slot, r.slot + r.span)}
    return FIX == "per-slot" and not e_unit_t <= set(walked)


def verdict_pair(records, walked, allocated, classes, theta=5, t=5):
    w = {(0, s) for s in walked}
    b = bounds_of_a_device(0, records, w, 0, theta, t)
    reset = set_to_unit_classes(records, lambda r: classes.get(r.slot), COMPARISON)
    either = None if reset is None else within_either(allocated, b, bounds_of_a_device(0, reset, w, 0, theta, t))
    if either and per_slot_fix_reddens(reset, walked):
        either = False
    return within_bounds(allocated, b), either


# ---- 转写校验：五条单测（快照 walk.rs 25638 起那一组）----
CLASSES = {10: 2, 12: 1, 30: 2}
calibration = [
    ("record_one_slot_short_..._counting_both_slots", [rec(10, 1), rec(12, 1)], [10, 11, 12], 3, (False, True)),
    ("record_one_slot_short_..._counting_one_slot", [rec(10, 1), rec(12, 1)], [10, 11, 12], 2, (False, True)),
    ("record_wider_than_its_two_slot_unit", [rec(30, 3)], [30, 31], 2, (False, True)),
    ("walked_slot_outside_every_record_..._stays_red", [rec(10, 1), rec(12, 1)], [10, 11, 12, 20], 3, (False, False)),
    ("walked_slot_without_a_record_..._stays_red", [rec(12, 1)], [10, 11, 12], 3, (False, None)),
]
for name, records, walked, allocated, expected in calibration:
    got = verdict_pair(records, walked, allocated, CLASSES)
    print(f"name=calibration test={name} expected={expected} got={got} match={got == expected}")


# ---- 判一张槽级盘面 ----
def judge_world(label, unit_area, records, walked, unit_headers, allocated, deferred_row, runs_row,
                empty_segments_row, newest_walk):
    """unit_headers：{起点槽: 类的槽数}，不在表里的槽头用不了（全零）。walked：全部候选版本走到的槽。
    newest_walk：最新根走读到的槽（I-3.11 用）。只有一个候选版本时两者相同。"""
    start, end = unit_area
    classes = lambda r: unit_headers.get(r.slot)
    w = {(0, s) for s in walked}
    v31 = i31_verdict(0, records, w, 0, 5, 5, allocated, classes, COMPARISON)
    if v31 == "partial" and per_slot_fix_reddens(set_to_unit_classes(records, classes, COMPARISON), walked):
        v31 = "violated"
    i318 = ["red" if (not r.released and r.slot in unit_headers and unit_headers[r.slot] != r.span)
            else ("no-object" if r.slot not in unit_headers else "holds") for r in records]
    i315 = ["holds" if r.slot in walked else ("no-object" if r.slot not in unit_headers else "red")
            for r in records if not r.released]
    i310 = ["judged" if r.slot in unit_headers else "no-object" for r in records if not r.released]
    spans = sorted((r.slot, r.slot + r.span) for r in records)
    i54 = "holds" if all(spans[i][1] <= spans[i + 1][0] for i in range(len(spans) - 1)) else "red"
    i311 = "holds" if allocated - deferred_row == len(newest_walk) else "red"
    i52 = "holds"  # 空闲行照「容量 − 已分配」写，构造上成立
    occupied = sorted(s for s in walked if start <= s < end)
    if len(occupied) != allocated:
        i316 = "partial"
    else:
        runs, nxt = 0, start
        for s in occupied:
            runs += s > nxt
            nxt = s + 1
        runs += end > nxt
        held = {s for r in records if not r.released for s in range(r.slot, r.slot + r.span)}
        segs = sum(1 for k in range((end - start) // 64)
                   if not any(start + 64 * k <= s < start + 64 * (k + 1) for s in held))
        i316 = "holds" if (runs, segs) == (runs_row, empty_segments_row) else f"red(runs {runs_row}/{runs} segs {empty_segments_row}/{segs})"
    print(f"name=world label={label} fix={FIX} I-3.1={v31} I-3.18={i318} I-3.15={i315} I-3.10={i310} "
          f"I-5.4={i54} I-3.11={i311} I-5.2={i52} I-3.16={i316}")


AREA = (0, 64)
HEADERS = {0: 2}          # 槽 0 起一个码 1 数据单元（两槽）；槽 63 全零
WALK = [0, 1]             # 引用按两槽走到 0、1
# 镜像：槽 0 那条记录只记一槽（I-3.18 判它），另有一条罩槽 63 的未释放记录，槽 63 全零、没人引用（泄漏）；
# 写者照记录数记已分配 2 槽、defer 0、runs 1、全空段 0。
judge_world("mirror_narrow_record_plus_headerless_leak", AREA, [rec(0, 1), rec(63, 1)], WALK, HEADERS,
            allocated=2, deferred_row=0, runs_row=1, empty_segments_row=0, newest_walk=WALK)
# 对照一：同一处泄漏，槽 0 的记录跨度对（两槽）：写者照记录数记 3 槽。
judge_world("control_leak_alone", AREA, [rec(0, 2), rec(63, 1)], WALK, HEADERS,
            allocated=3, deferred_row=0, runs_row=1, empty_segments_row=0, newest_walk=WALK)
# 对照二：只有窄记录、没有泄漏，写者照记录数记 1 槽（records-view runs：空闲 1..63 一段）。
judge_world("control_narrow_alone_counting_records", AREA, [rec(0, 1)], WALK, HEADERS,
            allocated=1, deferred_row=0, runs_row=1, empty_segments_row=0, newest_walk=WALK)
# 对照三：只有窄记录、记账照单元记 2 槽（单测「counting_both_slots」那一形放进整张盘面）。
judge_world("control_narrow_alone_counting_the_unit", AREA, [rec(0, 1)], WALK, HEADERS,
            allocated=2, deferred_row=0, runs_row=1, empty_segments_row=0, newest_walk=WALK)
