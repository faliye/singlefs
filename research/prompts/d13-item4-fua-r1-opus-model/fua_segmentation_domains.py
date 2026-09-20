#!/usr/bin/env python3
"""按三种读法枚举「崩溃状态 = 哪些写已持久」的向量集合，逐向量比。

读法甲（＝ crates/singlefs-harness/src/crash.rs:327-366 今天的切法）：屏障关掉当前段；
  FUA 写先推进当前段再关掉当前段。状态 = 前若干段整段持久 + 当前段任意子集 + 之后全不持久。
读法乙（＝ .claude/kb/decisions/13-验证路线.md 定案句的字面）：FUA 自成一段，
  它前面那些普通写落在更早的一段里，按全序前缀必须整段持久。
读法丁（本腿提的第四个候选，被攻过零轮）：屏障仍给全序前缀；FUA 只给单向约束——
  它自己那一次写在它完成之后必然在介质上，所以「它之后发出的写已持久 ⇒ 它已持久」，
  而它**不**让同一个屏障区间里排在它前面的普通写持久。

三者都不枚举撕裂子集（D13 已定项 4）。
"""

import sys
from itertools import product

BARRIER = "barrier"


def parse(stream):
    """stream: 空格分隔的记号；B = 屏障，kind = 普通写，kind! = FUA 写。"""
    ops = []
    for token in stream.split():
        if token == "B":
            ops.append((BARRIER, None, False))
        elif token.endswith("!"):
            ops.append(("write", token[:-1], True))
        else:
            ops.append(("write", token, False))
    return ops


def write_kinds(ops):
    return [kind for (what, kind, _) in ops if what == "write"]


def segments_jia(ops):
    """crash.rs:327-366 的切法，逐行对应。"""
    segments, current, write_index = [], [], 0
    for what, _kind, is_force_unit_access in ops:
        if what == BARRIER:
            if current:
                segments.append(current)
                current = []
            continue
        current.append(write_index)
        write_index += 1
        if is_force_unit_access:
            segments.append(current)
            current = []
    if current:
        segments.append(current)
    return segments


def segments_yi(ops):
    """定案句字面：FUA 自成一段（它前面那些普通写先收成它们自己的一段）。"""
    segments, current, write_index = [], [], 0
    for what, _kind, is_force_unit_access in ops:
        if what == BARRIER:
            if current:
                segments.append(current)
                current = []
            continue
        if is_force_unit_access:
            if current:
                segments.append(current)
                current = []
            segments.append([write_index])
            write_index += 1
            continue
        current.append(write_index)
        write_index += 1
    if current:
        segments.append(current)
    return segments


def closed_form(segments):
    """crash.rs:370 的闭式：1 + Σ(2^|段| − 1)。"""
    return 1 + sum((1 << len(segment)) - 1 for segment in segments)


def vectors_from_segments(segments, write_count):
    """前若干段整段持久 + 当前段任意子集 + 之后全不持久；收成向量集合。"""
    vectors = set()
    for segment_index, segment in enumerate(segments):
        prefix = 0
        for earlier in segments[:segment_index]:
            for write in earlier:
                prefix |= 1 << write
        for mask in range(1 << len(segment)):
            vector = prefix
            for bit, write in enumerate(segment):
                if mask & (1 << bit):
                    vector |= 1 << write
            vectors.add(vector)
    everything = 0
    for segment in segments:
        for write in segment:
            everything |= 1 << write
    vectors.add(everything)
    _ = write_count
    return vectors


def vectors_ding(ops):
    """崩溃点模型：崩溃点 p 之前的写都已发出且已返回，p 那个写在飞，p 之后的没发出。
    强制持久的 = 最后一道屏障之前的全部写，加上 p 之前的每一个 FUA 写；其余自由。"""
    writes = [index for index, (what, _k, _f) in enumerate(ops) if what == "write"]
    is_fua = [ops[index][2] for index in writes]
    # 每个写下标之前最后一道屏障覆盖到第几个写
    covered_by_barrier = []
    covered = 0
    seen = 0
    for what, _kind, _fua in ops:
        if what == BARRIER:
            covered = seen
        else:
            covered_by_barrier.append(covered)
            seen += 1
    covered_by_barrier.append(covered)  # 崩溃点落在全部写之后
    write_count = len(writes)
    vectors = set()
    for crash_point in range(write_count + 1):
        forced = 0
        for index in range(covered_by_barrier[crash_point]):
            forced |= 1 << index
        for index in range(crash_point):
            if is_fua[index]:
                forced |= 1 << index
        free = [index for index in range(crash_point) if not (forced >> index) & 1]
        if crash_point < write_count:
            free.append(crash_point)
        for bits in product((0, 1), repeat=len(free)):
            vector = forced
            for bit, index in zip(bits, free):
                if bit:
                    vector |= 1 << index
            vectors.add(vector)
    return vectors


def describe(vector, kinds):
    return "|".join(
        f"{kind}#{index}" for index, kind in enumerate(kinds) if (vector >> index) & 1
    ) or "(空)"


def report(name, stream):
    ops = parse(stream)
    kinds = write_kinds(ops)
    seg_jia, seg_yi = segments_jia(ops), segments_yi(ops)
    vec_jia = vectors_from_segments(seg_jia, len(kinds))
    vec_yi = vectors_from_segments(seg_yi, len(kinds))
    vec_ding = vectors_ding(ops)
    print(f"== {name}")
    print(f"   写数={len(kinds)}")
    print(f"   甲 段序列={'+'.join(str(len(s)) for s in seg_jia)} 闭式={closed_form(seg_jia)} 向量数={len(vec_jia)}")
    print(f"   乙 段序列={'+'.join(str(len(s)) for s in seg_yi)} 闭式={closed_form(seg_yi)} 向量数={len(vec_yi)}")
    print(f"   丁 向量数={len(vec_ding)}")
    print(f"   乙⊆甲={vec_yi <= vec_jia} 甲⊆丁={vec_jia <= vec_ding} 甲==丁={vec_jia == vec_ding} 甲==乙={vec_jia == vec_yi}")
    print(f"   丁\\甲 = {len(vec_ding - vec_jia)} 个向量；甲\\乙 = {len(vec_jia - vec_yi)} 个向量")
    missing = sorted(vec_ding - vec_jia)
    for vector in missing[:32]:
        print(f"     丁有甲无: {describe(vector, kinds)}")
    if len(missing) > 32:
        print(f"     …… 还有 {len(missing) - 32} 个")
    print()


STREAMS = {
    # layout/01-first-txn.md 八，整条流那一行：取号 + 暖机×2 + 第一个事务。
    # 段序列 2+2+1+2+2+1+18+2+1+2、262165 个状态。
    "第一条流（今天的代码）":
        "sb sb B  jr jr B  root!  sb sb B  jr jr B  root!  "
        "sb sb u u u u u u u u u u u u u u u u B  jr jr B  root!  sb sb",
    # 变异表第 67 行：零单元发布（暖机那两次）在记录与根之间少一道屏障。
    "第一条流（施加变异表第 67 行）":
        "sb sb B  jr jr root!  sb sb B  jr jr root!  "
        "sb sb u u u u u u u u u u u u u u u u B  jr jr B  root!  sb sb",
    # mkfs 种根：4 单元写 屏障 根 FUA ×3（彼此之间没有屏障） 4 超级块槽写 屏障。
    "mkfs 种根": "u u u u B  root! root! root!  sb sb sb sb B",
    # 假想：屏障挪到根之后（记录与根同段，根之后紧接超级块槽写）
    "假想：记录与根同段、根后紧跟超级块槽写": "B jr jr root! sb sb B",
}

if __name__ == "__main__":
    for name, stream in STREAMS.items():
        report(name, stream)
    sys.exit(0)
