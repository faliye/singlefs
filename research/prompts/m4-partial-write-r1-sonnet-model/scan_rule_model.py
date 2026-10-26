#!/usr/bin/env python3
"""正推腿的小模型（草稿）：部分写之后，扫描重建的几种认领规则能不能认回全部活单元、不认回已放掉的旧单元。

单位：文件大小 h 用「半个单元」计（h 偶数 = 恰在单元边界，奇数 = 在单元中间），位置 p 是单元号。
盘上状态：每个位置最新那份拷贝的出生 txg b[p]（None = 从没写过）；放掉的单元拷贝照留（最坏情形：没被别的东西覆盖）。
真值：活单元 = 位置 0..l-1（l = 现存单元数，洞只在尾部），期望单元数 expected(h) = max(1, ceil(h/2))。
入口（对应 crates 里已有的与要加的）：
  whole(h)   整份写（publish_sequential_write）：位置 0..m-1 全部重写
  trunc(h)   截断（publish_size_change 缩短）：边界不重写，单元中间或截到 0 重写新的最后一个单元；放了单元才写截断字段
  ext(h)     变长（publish_size_change 变长）：只有「最后一个存在的单元声明长度不够」时重写它，其后是洞
  pw(o)      部分写：位置 o（o < l）
  phole(j)   洞里的部分写：位置 j（l <= j < expected）；l..j-1 补零单元一起写
扫描规则（rebuild.rs 的三条，读法 Z 加写序 + 截断字段）与候选。
"""
import itertools
import sys

def expected(h):
    return max(1, (h + 1) // 2)

class State:
    __slots__ = ("h", "l", "b", "field", "tombs", "t")
    def __init__(self, h, l, b, field, tombs, t):
        self.h, self.l, self.b, self.field, self.tombs, self.t = h, l, tuple(b), field, tuple(tombs), t
    def key(self):
        # birth 值按名次归一，免得 t 不同的同形状状态各算一次
        vals = sorted({x for x in self.b if x is not None} | {self.field[0]} | {x[0] for x in self.tombs} | {self.t})
        rank = {v: i for i, v in enumerate(vals)}
        return (self.h, self.l,
                tuple(None if x is None else rank[x] for x in self.b),
                (rank[self.field[0]], self.field[1]),
                tuple((rank[x[0]], x[1]) for x in self.tombs),
                rank[self.t])

P = 4  # 位置数上限

def initial():
    # 起点：一个空文件，size 0，一个声明长度 0 的单元 0，出生 txg 1
    b = [None] * P
    b[0] = 1
    return State(0, 1, b, (0, 0), [], 1)

def step(s, op, cfg):
    kind, a = op
    t = s.t + 1
    b = list(s.b)
    h, l, field, tombs = s.h, s.l, s.field, list(s.tombs)
    if kind == "whole":
        hn = a
        m = expected(hn)
        if m > P: return None
        released = l > m
        for p in range(m): b[p] = t
        if cfg["whole_field"] == "clear":
            field = (0, 0)
        elif cfg["whole_field"] == "always":
            field = (t, m)
        elif cfg["whole_field"] == "if_released":
            field = (t, m) if released else field
        elif cfg["whole_field"] == "clear_unless_released":
            field = (t, m) if released else (0, 0)
        if released:
            tombs.append((t, m))
        return State(hn, m, b, field, tombs, t)
    if kind == "trunc":
        hn = a
        if hn >= h: return None
        m = expected(hn)
        released = l > m
        if (hn % 2 == 1 or hn == 0) and m - 1 < l:
            b[m - 1] = t
        ln = min(l, m)
        if released:
            field = (t, hn)
            tombs.append((t, m))
        return State(hn, ln, b, field, tombs, t)
    if kind == "ext":
        hn = a
        if hn <= h: return None
        if expected(hn) > P: return None
        if h % 2 == 1 and l == expected(h):
            b[l - 1] = t
        return State(hn, l, b, field, tombs, t)
    if kind == "pw":
        o = a
        if o >= l: return None
        if cfg["pw"] == "only":
            b[o] = t
        else:  # 从 o 重写到末尾
            for p in range(o, l): b[p] = t
        return State(h, l, b, field, tombs, t)
    if kind == "phole":
        j = a
        if not (l <= j < expected(h)): return None
        for p in range(l, j + 1): b[p] = t
        return State(h, j + 1, b, field, tombs, t)
    raise AssertionError(kind)

def claimed(s, rule):
    out = set()
    for p in range(expected(s.h)):
        if s.b[p] is None: continue
        if rule["order"]:
            if any(s.b[q] is not None and s.b[q] > s.b[p] for q in range(p)):
                continue
        if rule["field"]:
            T, hn = s.field
            if T != 0 and p >= expected(hn) and s.b[p] < T:
                continue
        if rule["tomb"]:
            if any(lo <= p and tt > s.b[p] for tt, lo in s.tombs):
                continue
        out.add(p)
    return out

NO_PARTIAL_WRITES = False

def ops_list():
    ops = [("whole", h) for h in range(0, 7)] + [("trunc", h) for h in range(0, 6)] + [("ext", h) for h in range(1, 7)]
    if not NO_PARTIAL_WRITES:
        ops += [("pw", o) for o in range(3)] + [("phole", j) for j in range(1, 3)]
    return ops

def explore(cfg, rule, depth, want_example=True):
    start = initial()
    frontier = [(start, [])]
    seen = {start.key()}
    bad = []
    n_states = 0
    for d in range(depth):
        nxt = []
        for s, path in frontier:
            for op in ops_list():
                ns = step(s, op, cfg)
                if ns is None: continue
                k = ns.key()
                if k in seen: continue
                seen.add(k)
                n_states += 1
                live = set(range(ns.l))
                if claimed(ns, rule) != live:
                    bad.append((path + [op], ns, claimed(ns, rule), live))
                nxt.append((ns, path + [op]))
        frontier = nxt
    return n_states, bad

def describe(ns):
    return f"h={ns.h} l={ns.l} b={list(ns.b)} field(T,h)={ns.field} tombs={list(ns.tombs)}"

VARIANTS = {
    # 名字: (操作语义, 认领规则)
    "today_rule_pw_only":        (dict(whole_field="clear", pw="only"), dict(order=True, field=True, tomb=False)),
    "today_rule_pw_tail_rewrite":(dict(whole_field="clear", pw="tail"), dict(order=True, field=True, tomb=False)),
    "field_only_whole_always":   (dict(whole_field="always", pw="only"), dict(order=False, field=True, tomb=False)),
    "field_only_whole_if_rel":   (dict(whole_field="if_released", pw="only"), dict(order=False, field=True, tomb=False)),
    "field_only_whole_clear":    (dict(whole_field="clear", pw="only"), dict(order=False, field=True, tomb=False)),
    "field_only_whole_clear_unless_released": (dict(whole_field="clear_unless_released", pw="only"), dict(order=False, field=True, tomb=False)),
    "range_tombstone":           (dict(whole_field="clear", pw="only"), dict(order=False, field=False, tomb=True)),
}

# 必红自证：今天的入口（没有部分写）下，去掉读法 Z 的哪一条都该红；全留着该绿。
SELF_CHECKS = {
    "today_ops_today_rule(should_be_0)":            (dict(whole_field="clear", pw="only"), dict(order=True, field=True, tomb=False)),
    "today_ops_without_field_clause(should_be_gt0)": (dict(whole_field="clear", pw="only"), dict(order=True, field=False, tomb=False)),
    "today_ops_without_order_clause(should_be_gt0)": (dict(whole_field="clear", pw="only"), dict(order=False, field=True, tomb=False)),
}

if __name__ == "__main__":
    depth = int(sys.argv[1]) if len(sys.argv) > 1 else 6
    NO_PARTIAL_WRITES = True
    for name, (cfg, rule) in SELF_CHECKS.items():
        n, bad = explore(cfg, rule, depth)
        print(f"selfcheck={name} depth={depth} states={n} mismatches={len(bad)}")
        for path, ns, got, live in bad[:1]:
            print(f"  example ops={path} -> {describe(ns)} scan_claims={sorted(got)} live={sorted(live)}")
    NO_PARTIAL_WRITES = False
    for name, spec in VARIANTS.items():
        if spec is None: continue
        cfg, rule = spec
        n, bad = explore(cfg, rule, depth)
        print(f"variant={name} depth={depth} states={n} mismatches={len(bad)}")
        for path, ns, got, live in bad[:2]:
            print(f"  example ops={path} -> {describe(ns)} scan_claims={sorted(got)} live={sorted(live)}")
