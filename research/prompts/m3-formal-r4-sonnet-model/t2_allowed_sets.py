#!/usr/bin/env python3
"""T2 小模型：同一段短历史、同一个崩溃时刻，三种「丢法」给出的允许集合有多大。

只用标准库。模型自己的东西：一个迷你 POSIX 状态机（文件 = 路径到字节串）、操作序列、
fsync 标记、崩溃时刻。不读 crates/，不用实现的任何代码；这是 RefFS 允许集合的玩具版，
用来量「几种读法在条款面前相容、而给出不同集合」的大小，不是 RefFS 的实现。

三种丢法（崩溃之后只允许看到下面这些操作子集施加出来的状态；lo = 最后一次已返回的 fsync
覆盖到第几个操作，hi = 崩溃时刻已发出的操作数，操作按 1..hi 编号）：
  P  前缀（op 级）：{1..j}，j 取 [lo, hi]                      —— 只丢一个后缀
  Q  发布级前缀：{1..j}，j 取 [lo, hi] 里恰是某次发布边界的那几个 —— 边界由 RefFS 之外的人告诉它
  S  任意子集：{1..lo} 并上 (lo, hi] 里任一子集                  —— 按任意子集丢
  Sd 依赖闭合的子集：S 里挑出「每个被选的操作，它依赖的前一个同路径操作也被选」的那些
状态按内容去重后再数（两个不同子集可以落到同一个 POSIX 状态）。
"""
import itertools
import json
import sys


def apply(ops, chosen):
    """按原次序施加被选的操作；施加不了的（没有这个路径）判「非法」，返回 None。"""
    files = {}
    for index, op in enumerate(ops, 1):
        if index not in chosen:
            continue
        kind = op[0]
        if kind == "create":
            if op[1] in files:
                return None
            files[op[1]] = ""
        elif kind == "write":
            if op[1] not in files:
                return None
            old = files[op[1]]
            offset, data = op[2], op[3]
            old = old.ljust(offset, "\0")
            files[op[1]] = old[:offset] + data + old[offset + len(data):]
        elif kind == "rename":
            if op[1] not in files or op[2] in files:
                return None
            files[op[2]] = files.pop(op[1])
        elif kind == "unlink":
            if op[1] not in files:
                return None
            del files[op[1]]
    return tuple(sorted(files.items()))


def paths_of(op):
    return [p for p in op[1:3] if isinstance(p, str)]


def dependency_closed(ops, chosen):
    """被选的每个操作，它碰到的每个路径上更早的操作也被选（同路径的先后次序保留）。"""
    for index in chosen:
        for earlier in range(1, index):
            if set(paths_of(ops[earlier - 1])) & set(paths_of(ops[index - 1])) and earlier not in chosen:
                return False
    return True


def allowed(ops, lo, hi, boundaries):
    base = set(range(1, lo + 1))
    tail = list(range(lo + 1, hi + 1))
    out = {"P": set(), "Q": set(), "S": set(), "Sd": set()}
    for j in range(lo, hi + 1):
        state = apply(ops, set(range(1, j + 1)))
        out["P"].add(state)
        if j == lo or j in boundaries:
            out["Q"].add(state)
    for size in range(len(tail) + 1):
        for subset in itertools.combinations(tail, size):
            chosen = base | set(subset)
            state = apply(ops, chosen)
            if state is None:
                continue
            out["S"].add(state)
            if dependency_closed(ops, chosen):
                out["Sd"].add(state)
    return out


def moments(ops, fsyncs):
    """崩溃时刻：每个操作发出之前、与每个 fsync 在途（调用了没返回）。
    返回 [(名字, lo, hi)]；lo = 已返回的 fsync 里覆盖到的最大操作号，hi = 已发出的操作数。"""
    out = []
    returned = 0
    for index in range(0, len(ops) + 1):
        out.append((f"发出 {index} 个操作后", returned, index))
        if index in fsyncs:
            out.append((f"第 {index} 个操作后的 fsync 在途", returned, index))
            returned = index
            out.append((f"第 {index} 个操作后的 fsync 已返回", returned, index))
    return out


HISTORIES = {
    # 今天实现面上有的：一个文件、内容写、fsync（发布返回）
    "H1 一个文件的内容写": {
        "ops": [("create", "f"), ("write", "f", 0, "AAAA"), ("write", "f", 2, "BB"),
                ("write", "f", 0, "C"), ("write", "f", 1, "DD")],
        "fsyncs": {2},          # 第 2 个操作之后 fsync
        "boundaries": {2, 4},   # 发布边界：fsync 触发一次，之后某次时间触发的发布落在第 4 个操作之后
        "bad2": (4, 5),         # 坏2：第 4 个没落、第 5 个落了
    },
    # 实现今天没有、条款面还没写的：目录操作与数据写的次序（经典的「rename 落了、数据没落」）
    "H2 目录操作与数据写": {
        "ops": [("create", "a"), ("write", "a", 0, "x"), ("create", "c"),
                ("write", "a", 1, "y"), ("rename", "a", "b"), ("write", "c", 0, "z")],
        "fsyncs": {2},
        "boundaries": {2, 5},
        "bad2": (4, 5),         # 坏2：写 a 的第 4 个没落、rename 的第 5 个落了
    },
    # 一次写请求被切成三个一单元事务（D23 已定项 14：崩溃原子性的单位是一个单元，不是一次写请求）
    "H3 一次写请求切三个单元": {
        "ops": [("create", "f"), ("write", "f", 0, "AAA"), ("write", "f", 3, "BBB"), ("write", "f", 6, "CCC")],
        "fsyncs": {1},
        "boundaries": {1, 4},   # 这里的「边界」是写请求的两端：整个请求整体落或整体不落
        "bad2": (2, 3),
    },
}

# 三个「实现真会出的坏恢复」：恢复落在哪个状态上，各种读法判不判它出局（出局 = 被查出）
def probes(ops, lo, hi, bad2):
    out = {}
    if lo >= 1:
        out["坏1 丢了已返回的 fsync"] = apply(ops, set(range(1, lo)))
    skipped, landed = bad2
    chosen = (set(range(1, hi + 1)) - {skipped})
    out[f"坏2 乱序（第 {skipped} 个没落、第 {landed} 个落了，其余全落）"] = apply(ops, chosen)
    return out


def main():
    result = {}
    for name, h in HISTORIES.items():
        ops, fsyncs, boundaries = h["ops"], h["fsyncs"], h["boundaries"]
        rows = []
        for label, lo, hi in moments(ops, fsyncs):
            sets = allowed(ops, lo, hi, boundaries)
            rows.append({"时刻": label, "lo": lo, "hi": hi,
                         **{k: len(v) for k, v in sets.items()}})
        # 三个坏恢复在「全部操作已发出、最后一次 fsync 已返回」时刻与「最后」时刻上的判定
        lo_end = max(fsyncs)
        hi_end = len(ops)
        sets_end = allowed(ops, lo_end, hi_end, boundaries)
        verdict = {}
        for probe, state in probes(ops, lo_end, hi_end, h["bad2"]).items():
            verdict[probe] = {"状态": (None if state is None else [list(p) for p in state]), **{k: ("放过" if state in v else "查出") for k, v in sets_end.items()}}
        # 坏3：切开一次发布 —— 取 boundaries 之外、在 (lo,hi) 之内的 j
        for j in range(lo_end + 1, hi_end):
            if j not in boundaries:
                state = apply(ops, set(range(1, j + 1)))
                verdict[f"坏3 切开一次整体边界（落在第 {j} 个操作后）"] = {
                    "状态": [list(p) for p in state],
                    **{k: ("放过" if state in v else "查出") for k, v in sets_end.items()}}
        result[name] = {"rows": rows, "end_moment": [lo_end, hi_end], "verdict": verdict,
                        "boundaries": sorted(boundaries)}
    json.dump(result, sys.stdout, ensure_ascii=False, indent=1, sort_keys=True)
    print()


def selftest():
    """必红自检：弄坏下界 lo（强行取 0），「丢了已返回的 fsync」那个坏恢复必须由查出变成放过；
    再核封闭式：|P| = hi - lo + 1，Q 不大于 P，S 不小于 P。"""
    for name, h in HISTORIES.items():
        ops, fsyncs, boundaries = h["ops"], h["fsyncs"], h["boundaries"]
        for label, lo, hi in moments(ops, fsyncs):
            sets = allowed(ops, lo, hi, boundaries)
            assert len(sets["P"]) <= hi - lo + 1, (name, label)
            assert len(sets["Q"]) <= len(sets["P"]) <= len(sets["S"]), (name, label)
        lo_end, hi_end = max(fsyncs), len(ops)
        bad1 = apply(ops, set(range(1, lo_end)))
        real = allowed(ops, lo_end, hi_end, boundaries)
        broken = allowed(ops, 0, hi_end, boundaries)   # 弄坏：下界取 0
        assert all(bad1 not in v for v in real.values()), "下界正确时坏1必须被查出"
        assert bad1 in broken["P"], "下界弄坏后坏1必须被放过（自检会红的证据）"
    print("selftest ok")


if __name__ == "__main__":
    selftest() if "--selftest" in sys.argv else main()
