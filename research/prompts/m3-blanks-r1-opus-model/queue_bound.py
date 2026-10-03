#!/usr/bin/env python3
"""B4 / B8 / D23 已定项 14：deleted_inodes 队列在一次可写挂载里能长到多长。

只固定前缀（新池里 N 个 size 0 的活 inode、队列空），用户动作全部放开扫：
  unlink(i)        对现行那一版里 nlink >= 1 的每个 i
  rollback(k)      回退到候选集里的每一条根（候选集 = 根环里最近 CAND 条带文件的根，盘紧时 CAND = 4）
  remount          崩溃或卸载后可写挂载：mount_writable 交回之前把队列走到空（D8 已定项 16 第 6 条）
B8 的接口臂（unlink 一次调用里做什么）：
  OWN          意图(i) 之后只做 i 自己的批（size 0：一次最后一批）
  DRAIN_AFTER  意图(i) 之后把现行那一版队列里的全部号做完
  DRAIN_BEFORE 先把现行那一版队列里的号做完，再写意图(i)、做 i 的批
  OWN_RBCONT   同 OWN，另外回退那次发布之后在会话里接着把队列走到空
  DRAIN_ANY    会话里任何一次用户改动（unlink 或写）都先把现行那一版队列里的号做完，再做它自己
  ASYNC        只写意图就返回，批只在 remount 里做
另有一个用户动作 write：一次不碰删除结构的用户改动（覆盖写 / 建空 inode），只进根环。
第二个量 stuck：队列非空期间连续做了几次 write 而队列一直没空（D3 已定项 9 第 2 条「删掉的空间在有界步数内可用」的步数）。
回退取 R_old 的：inode 树（活与删除中的集合）与 deleted_inodes（D23 已定项 14）。
每次发布进根环；根环只留最近 RING 条。计的是任何一版（含会话现行）里 |队列| 的最大值。
"""
import sys
from collections import deque

def publish(ring, version, ring_len):
    ring = ring + (version,)
    return ring[-ring_len:]

def last_batch(version, j):
    live, dying, queue = version
    return (live, dying - {j}, queue - {j})

def intent(version, i):
    live, dying, queue = version
    return (live - {i}, dying | {i}, queue | {i})

def drain(ring, ring_len):
    for j in sorted(ring[-1][2]):
        ring = publish(ring, last_batch(ring[-1], j), ring_len)
    return ring

def do_write(ring, arm, ring_len):
    if arm == "DRAIN_ANY":
        ring = drain(ring, ring_len)
    return publish(ring, ring[-1], ring_len)

def do_unlink(ring, i, arm, ring_len):
    if arm in ("DRAIN_BEFORE", "DRAIN_ANY"):
        ring = drain(ring, ring_len)
    ring = publish(ring, intent(ring[-1], i), ring_len)
    if arm in ("OWN", "OWN_RBCONT", "DRAIN_BEFORE", "DRAIN_ANY"):
        ring = publish(ring, last_batch(ring[-1], i), ring_len)
    elif arm == "DRAIN_AFTER":
        ring = drain(ring, ring_len)
    return ring  # ASYNC：只有意图

def do_rollback(ring, k, arm, ring_len):
    target = ring[k]
    ring = publish(ring, target, ring_len)
    if arm == "OWN_RBCONT":
        ring = drain(ring, ring_len)
    return ring

def successors(ring, arm, ring_len, cand):
    current = ring[-1]
    for i in sorted(current[0]):
        yield ("unlink", i), do_unlink(ring, i, arm, ring_len)
    n = len(ring)
    for k in range(max(0, n - cand), n - 1):
        yield ("rollback", n - 1 - k), do_rollback(ring, k, arm, ring_len)
    yield ("write", 0), do_write(ring, arm, ring_len)
    yield ("remount", 0), drain(ring, ring_len)

def explore(arm, inodes, depth, ring_len, cand):
    start = (frozenset(range(1, inodes + 1)), frozenset(), frozenset())
    ring0 = (start,)
    seen = {(ring0, 0)}
    frontier = deque([(ring0, 0, ())])
    best = (0, ())
    stuck_best = (0, ())
    states = 0
    while frontier:
        ring, stuck, history = frontier.popleft()
        states += 1
        m = max(len(v[2]) for v in ring)
        if m > best[0]:
            best = (m, history)
        if stuck > stuck_best[0]:
            stuck_best = (stuck, history)
        if len(history) == depth:
            continue
        for op, nxt in successors(ring, arm, ring_len, cand):
            nxt_stuck = (stuck + 1 if op[0] == "write" else stuck) if nxt[-1][2] else 0
            key = (nxt, nxt_stuck)
            if key in seen:
                continue
            seen.add(key)
            frontier.append((nxt, nxt_stuck, history + (op,)))
    return best, stuck_best, states

def main():
    inodes = int(sys.argv[1]) if len(sys.argv) > 1 else 5
    depth = int(sys.argv[2]) if len(sys.argv) > 2 else 10
    ring_len = int(sys.argv[3]) if len(sys.argv) > 3 else 4
    cand = ring_len  # 候选集取满根环（盘不紧）或缩到最近 4 条（盘紧）：环外的根不再是候选，状态里不留
    for _ in (0,):
        for arm in ("OWN", "DRAIN_AFTER", "DRAIN_BEFORE", "OWN_RBCONT", "DRAIN_ANY", "ASYNC"):
            (m, history), (stuck, stuck_history), states = explore(arm, inodes, depth, ring_len, cand)
            text = " ".join(f"{op}({arg})" for op, arg in history)
            stuck_text = " ".join(f"{op}({arg})" for op, arg in stuck_history)
            print(f"E name=queue_bound arm={arm} inodes={inodes} depth={depth} ring={ring_len} "
                  f"candidates={cand} states={states} max_queue={m} witness=[{text}] "
                  f"max_writes_with_queue_nonempty={stuck} stuck_witness=[{stuck_text}]")

if __name__ == "__main__":
    main()
