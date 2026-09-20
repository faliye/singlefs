#!/usr/bin/env python3
"""判别力自证：把三条规则各改坏一处，数必须动；不动就说明这个数不是这条规则算出来的。"""
import fua_segmentation_domains as model

STREAM_CLEAN = model.STREAMS["第一条流（今天的代码）"]
STREAM_HYPOTHETICAL = model.STREAMS["假想：记录与根同段、根后紧跟超级块槽写"]

failures = []


def check(name, actual, forbidden):
    verdict = "绿（数动了）" if actual != forbidden else "红（数没动，这一项没判别力）"
    print(f"  {name}: 改坏后 {actual}，改坏前 {forbidden} —— {verdict}")
    if actual == forbidden:
        failures.append(name)


ops_clean = model.parse(STREAM_CLEAN)
ops_hypothetical = model.parse(STREAM_HYPOTHETICAL)

print("== 自证 1：甲 不在 FUA 处关段（退回「只有屏障切段」）")
original_segments_jia = model.segments_jia
def segments_jia_broken(ops):
    segments, current, write_index = [], [], 0
    for what, _kind, _fua in ops:
        if what == model.BARRIER:
            if current:
                segments.append(current)
                current = []
            continue
        current.append(write_index)
        write_index += 1
    if current:
        segments.append(current)
    return segments
check("第一条流 甲 的闭式",
      model.closed_form(segments_jia_broken(ops_clean)),
      model.closed_form(original_segments_jia(ops_clean)))

print("== 自证 2：乙 不给 FUA 单独一段（退化成甲）")
check("假想流 乙 的闭式",
      model.closed_form(model.segments_jia(ops_hypothetical)),
      model.closed_form(model.segments_yi(ops_hypothetical)))

print("== 自证 3：丁 丢掉「FUA 之后的写已持久 ⇒ FUA 已持久」那一条")
def vectors_ding_broken(ops):
    writes = [index for index, (what, _k, _f) in enumerate(ops) if what == "write"]
    covered_by_barrier, covered, seen = [], 0, 0
    for what, _kind, _fua in ops:
        if what == model.BARRIER:
            covered = seen
        else:
            covered_by_barrier.append(covered)
            seen += 1
    covered_by_barrier.append(covered)
    vectors = set()
    from itertools import product
    for crash_point in range(len(writes) + 1):
        forced = 0
        for index in range(covered_by_barrier[crash_point]):
            forced |= 1 << index
        free = [i for i in range(crash_point) if not (forced >> i) & 1]
        if crash_point < len(writes):
            free.append(crash_point)
        for bits in product((0, 1), repeat=len(free)):
            vector = forced
            for bit, index in zip(bits, free):
                if bit:
                    vector |= 1 << index
            vectors.add(vector)
    return vectors
check("假想流 丁 的向量数",
      len(vectors_ding_broken(ops_hypothetical)),
      len(model.vectors_ding(ops_hypothetical)))

print()
if failures:
    print(f"自证判红：{failures}")
    raise SystemExit(1)
print("自证三项全绿：三个数各自对它那条规则敏感。")
