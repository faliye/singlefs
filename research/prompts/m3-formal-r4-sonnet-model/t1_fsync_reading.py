#!/usr/bin/env python3
"""T1 小模型：fsync(f) 返回之后「必须还在」取三种读法，同一段历史上允许集合多大、一个坏恢复放不放过。

读法（lo 集合 = 崩溃之后必须已施加的操作；其余已发出的操作按前缀可丢，即 P 读法的后缀丢）：
  a  只有 f 的内容与元数据（碰 f 的路径的、且不是目录项的操作；这里 create 算目录项，write 算内容）
  b  f 的内容与元数据，加 f 所在目录项（碰 f 的路径的全部操作）
  c  那一刻之前的全部操作
允许集合 = 必须集合之上、已发出操作的任一「依赖闭合」子集施加出来的状态（a、b），或 {1..j}, j ≥ fsync 位置（c）。
只用标准库；与 t2_allowed_sets.py 共用其 apply()（迷你 POSIX 状态机，不是实现的代码）。
"""
import itertools
import json
import sys

sys.dont_write_bytecode = True
from t2_allowed_sets import apply, paths_of  # noqa: E402

OPS = [("create", "a"), ("write", "a", 0, "x"), ("create", "c"), ("write", "c", 0, "z")]
FSYNC_POSITION = 4        # fsync(a) 发在 4 个操作之后
TARGET = "a"


def required(reading):
    n = FSYNC_POSITION
    if reading == "c":
        return set(range(1, n + 1))
    touching = {i for i in range(1, n + 1) if TARGET in paths_of(OPS[i - 1])}
    if reading == "b":
        return touching
    # a：内容与元数据，不含目录项（create 算目录项）；施加 write 需要它依赖的 create，这里按依赖闭合补上
    content = {i for i in touching if OPS[i - 1][0] != "create"}
    return content | {i for i in touching if OPS[i - 1][0] == "create" and any(j > i for j in content)}


def allowed(reading):
    must = required(reading)
    free = [i for i in range(1, FSYNC_POSITION + 1) if i not in must]
    states = set()
    for size in range(len(free) + 1):
        for extra in itertools.combinations(free, size):
            chosen = must | set(extra)
            state = apply(OPS, chosen)
            if state is None:
                continue
            # 依赖闭合：每个被选操作碰到的路径上，更早的同路径操作也被选
            ok = all(
                earlier in chosen
                for i in chosen
                for earlier in range(1, i)
                if set(paths_of(OPS[earlier - 1])) & set(paths_of(OPS[i - 1]))
            )
            if ok:
                states.add(state)
    return states


def main():
    bad = apply(OPS, {1, 2})          # 坏：fsync(a) 返回了，别的文件 c 的 create 与写全丢
    out = {}
    for reading in ("a", "b", "c"):
        if reading == "c":
            s = {apply(OPS, set(range(1, FSYNC_POSITION + 1)))}
        else:
            s = allowed(reading)
        out[reading] = {"必须集合": sorted(required(reading)), "允许状态数": len(s),
                        "坏恢复（只剩 a，c 全丢）": "放过" if bad in s else "查出"}
    json.dump(out, sys.stdout, ensure_ascii=False, indent=1, sort_keys=True)
    print()


if __name__ == "__main__":
    main()
