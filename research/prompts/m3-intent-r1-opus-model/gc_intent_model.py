#!/usr/bin/env python3
"""m3-intent-r1 攻方计数模型：整理意图 × 删除 × 管理员回退 × 准入，按深度全枚举（跑前写死见 prereg.md）。

抽象（与 crates/ 今天的实现的差别写在报告第七节）：一块盘、NSEG 段 × SEGSZ 槽，一个单元占一个槽；
每个对象 = 一个元数据槽（extent 叶 / inode 叶的代表）+ 若干数据槽；每个操作 = 一次原子发布。
分配记录照 D3 已定项 7：(0, 分配代) 已分配、(1, 释放代) 已释放；已释放的在释放代 ≤ 环里最旧根的 txg 时可再分配。
"""
import sys
from collections import deque

ALLOC, REL = 0, 1


class Arm:
    def __init__(self, name, carrier="AB", lops_rb="old", cursor="recheck", batch="nlink",
                 admission="plain", gc=True, mutation=None, del_in_r=False):
        self.name, self.carrier, self.lops_rb, self.cursor = name, carrier, lops_rb, cursor
        self.del_in_r = del_in_r              # 删除类发布（unlink 意图、删除批）的落点可以落进活整理意图的 R
        self.batch, self.admission, self.gc, self.mutation = batch, admission, gc, mutation


class Geo:
    def __init__(self, nseg, segsz, ring, maxobj, maxdata):
        self.nseg, self.segsz, self.ring, self.maxobj, self.maxdata = nseg, segsz, ring, maxobj, maxdata
        self.nslot = nseg * segsz
        self.window = (ring + 1) * 1          # 环窗口 = (根环里的根数 + 1) × W，模型里一次删除类发布至多新占 1 槽

    def label(self):
        return f"nseg={self.nseg} segsz={self.segsz} ring={self.ring} maxobj={self.maxobj} maxdata={self.maxdata}"


# 版本 = (txg, objs, recs, dq, gc, lu)
#   objs: 排好序的 (oid, nlink, meta, data 元组, 写入时的数据单元数)；recs: 每槽 None / (ALLOC, 代) / (REL, 释放代)
#   dq: deleted_inodes（AB 臂）；lu: logged_ops 里的 unlink 条目（C 臂）；gc: None 或 (段号, c0, 续做点槽号)
def queue_of(arm, v):
    return v[3] if arm.carrier == "AB" else v[5]


def with_queue(arm, v, q):
    q = tuple(sorted(q))
    return (v[0], v[1], v[2], q, v[4], v[5]) if arm.carrier == "AB" else (v[0], v[1], v[2], v[3], v[4], q)


def floor_of(ring):
    return min(v[0] for v in ring)


def reclaimable(rec, fl):
    return rec is None or (rec[0] == REL and rec[1] <= fl)


def in_r(geo, slot, gc):
    return gc is not None and slot // geo.segsz == gc[0]


def place(geo, cur, fl, n, ignore_r=False):
    out = []
    for s in range(geo.nslot):
        if len(out) == n:
            break
        if (ignore_r or not in_r(geo, s, cur[4])) and reclaimable(cur[2][s], fl):
            out.append(s)
    return out if len(out) == n else None


def free_counts(geo, cur, fl):
    total = inside = 0
    for s in range(geo.nslot):
        if reclaimable(cur[2][s], fl):
            total += 1
            if in_r(geo, s, cur[4]):
                inside += 1
    return total, inside


def admitted(geo, arm, cur, fl, need):
    total, inside = free_counts(geo, cur, fl)
    avail = total - geo.window - (inside if arm.admission in ("rterm", "rterm_issue") else 0)
    return avail >= need


def owner_of(objs, slot):
    for i, o in enumerate(objs):
        if o[2] == slot:
            return i, "meta", None
        if slot in o[3]:
            return i, "data", o[3].index(slot)
    return None


def canonical(ring):
    txgs = set()
    oids = set()
    for v in ring:
        txgs.add(v[0])
        for r in v[2]:
            if r is not None:
                txgs.add(r[1])
        if v[4] is not None:
            txgs.add(v[4][1])
        for o in v[1]:
            oids.add(o[0])
        oids.update(v[3])
        oids.update(v[5])
    tm = {t: i for i, t in enumerate(sorted(txgs))}
    om = {o: i + 1 for i, o in enumerate(sorted(oids))}
    out = []
    for v in ring:
        objs = tuple(sorted((om[o[0]], o[1], o[2], o[3], o[4]) for o in v[1]))
        recs = tuple(None if r is None else (r[0], tm[r[1]]) for r in v[2])
        gc = None if v[4] is None else (v[4][0], tm[v[4][1]], v[4][2])
        out.append((tm[v[0]], objs, recs, tuple(sorted(om[x] for x in v[3])), gc,
                    tuple(sorted(om[x] for x in v[5]))))
    return tuple(out)


def next_oid(ring):
    m = 0
    for v in ring:
        for o in v[1]:
            m = max(m, o[0])
        m = max([m] + list(v[3]) + list(v[5]))
    return m + 1


def successors(geo, arm, ring):
    """→ [(操作名, 新版本或 None, [事件])]。None = 这一步没发布出去（被拒或落点给不出）。"""
    cur = ring[-1]
    fl = floor_of(ring)
    t = cur[0] + 1
    out = []
    objs = cur[1]
    q = queue_of(arm, cur)
    # 写：新对象 nlink = 1，n 个数据槽 + 1 个元数据槽，走式子
    if len(objs) < geo.maxobj:
        for n in range(1, geo.maxdata + 1):
            if not admitted(geo, arm, cur, fl, n + 1):
                continue
            slots = place(geo, cur, fl, n + 1)
            if slots is None:
                out.append((f"W{n}", None, ["false_enospc"]))
                continue
            recs = list(cur[2])
            for s in slots:
                recs[s] = (ALLOC, t)
            o = (next_oid(ring), 1, slots[0], tuple(slots[1:]), n)
            out.append((f"W{n}", (t, tuple(sorted(objs + (o,))), tuple(recs), cur[3], cur[4], cur[5]), []))
    # unlink：意图那次发布，nlink → 0，元数据槽 COW，进队列；删除类，不判式子，落点照判
    for i, o in enumerate(objs):
        if o[1] != 1:
            continue
        slots = place(geo, cur, fl, 1, arm.del_in_r)
        if slots is None:
            out.append((f"U{o[0]}", None, ["unlink_stuck"]))
            continue
        recs = list(cur[2])
        recs[o[2]] = (REL, t)
        recs[slots[0]] = (ALLOC, t)
        no = (o[0], 0, slots[0], o[3], o[4])
        nobjs = tuple(sorted(objs[:i] + (no,) + objs[i + 1:]))
        v = (t, nobjs, tuple(recs), cur[3], cur[4], cur[5])
        out.append((f"U{o[0]}", with_queue(arm, v, set(q) | {o[0]}), []))
    # 删除批：从尾部释放一个数据槽、元数据槽 COW；最后一批释放元数据槽、删对象与队列项
    for oid in q:
        idx = next((i for i, o in enumerate(objs) if o[0] == oid), None)
        nq = set(q) - {oid}
        if idx is None or (arm.batch == "nlink" and objs[idx][1] != 0):
            v = (t, objs, cur[2], cur[3], cur[4], cur[5])
            out.append((f"D{oid}", with_queue(arm, v, nq), []))
            continue
        o = objs[idx]
        recs = list(cur[2])
        if o[3]:
            slots = place(geo, cur, fl, 1, arm.del_in_r)
            if slots is None:
                out.append((f"D{oid}", None, ["delete_stuck"]))
                continue
            recs[o[3][-1]] = (REL, t)
            recs[o[2]] = (REL, t)
            recs[slots[0]] = (ALLOC, t)
            no = (o[0], o[1], slots[0], o[3][:-1], o[4])
            v = (t, tuple(sorted(objs[:idx] + (no,) + objs[idx + 1:])), tuple(recs), cur[3], cur[4], cur[5])
            out.append((f"D{oid}", with_queue(arm, v, set(q)), []))
        else:
            recs[o[2]] = (REL, t)
            v = (t, objs[:idx] + objs[idx + 1:], tuple(recs), cur[3], cur[4], cur[5])
            out.append((f"D{oid}", with_queue(arm, v, nq), []))
    # 签发整理意图：R = 一个段，c0 = 签发那次发布的 txg，续做点 = 段首
    if arm.gc and cur[4] is None:
        for seg in range(geo.nseg):
            lo = seg * geo.segsz
            if not any(cur[2][s] is not None and cur[2][s][0] == ALLOC for s in range(lo, lo + geo.segsz)):
                continue
            if arm.admission == "rterm_issue":
                total, _ = free_counts(geo, cur, fl)
                inside = sum(1 for s in range(lo, lo + geo.segsz) if reclaimable(cur[2][s], fl))
                if total - inside < geo.window:
                    continue
            out.append((f"G{seg}", (t, objs, cur[2], cur[3], (seg, t, lo), cur[5]), []))
    if cur[4] is not None:
        out.append(gc_batch(geo, arm, ring, cur, fl, t))
    # 管理员回退到环里任一旧根
    for k in range(len(ring) - 1):
        if all(r is None for r in ring[k][2]):
            continue                          # 回退候选集只收带文件的根（D23 已定项 14）：mkfs 那一版不收
        out.append(rollback(geo, arm, ring[k], cur, t, k))
    return out


def qualifying(rec, c0):
    return rec is not None and rec[0] == ALLOC and rec[1] <= c0


def gc_batch(geo, arm, ring, cur, fl, t):
    seg, c0, ptr = cur[4]
    lo, hi = seg * geo.segsz, (seg + 1) * geo.segsz
    objs = cur[1]
    s = next((x for x in range(ptr, hi) if qualifying(cur[2][x], c0)), None)
    if s is not None:
        if not admitted(geo, arm, cur, fl, 1):
            return ("B", None, ["gc_paused_admission"])
        dest = place(geo, cur, fl, 1)
        if dest is None:
            return ("B", None, ["gc_paused_placement"])
        own = owner_of(objs, s)
        if own is None:
            return ("B", None, ["gc_unreferenced"])
        i, kind, j = own
        o = objs[i]
        no = (o[0], o[1], dest[0], o[3], o[4]) if kind == "meta" else (o[0], o[1], o[2], o[3][:j] + (dest[0],) + o[3][j + 1:], o[4])
        recs = list(cur[2])
        if arm.mutation != "gc_forget_release":
            recs[s] = (REL, t)
        recs[dest[0]] = (ALLOC, t)
        nobjs = tuple(sorted(objs[:i] + (no,) + objs[i + 1:]))
        return ("B", (t, nobjs, tuple(recs), cur[3], (seg, c0, s + 1), cur[5]), [])
    left = any(qualifying(cur[2][x], c0) for x in range(lo, hi))
    if arm.cursor == "recheck" and left:
        return ("B", (t, objs, cur[2], cur[3], (seg, c0, lo), cur[5]), [])
    return ("B", (t, objs, cur[2], cur[3], None, cur[5]), ["gc_del"] if left else [])


def rollback(geo, arm, old, cur, t, k):
    if arm.lops_rb == "old":
        gc, lu = old[4], old[5]
    else:
        gc, lu = cur[4], cur[5]
    recs = list(cur[2])
    ref = {}
    for o in old[1]:
        for s in (o[2],) + o[3]:
            ref[s] = old[2][s][1]
    for s, g in ref.items():
        c = recs[s]
        if c is None:
            return (f"R{k}", None, ["rb_refused_no_record"])
        if c[0] == ALLOC and c[1] != g:
            return (f"R{k}", None, ["rb_refused_gen"])
        if c[0] == REL and arm.mutation != "rb_no_resurrect":
            recs[s] = (ALLOC, g)
    for s in range(geo.nslot):
        if recs[s] is not None and recs[s][0] == ALLOC and s not in ref:
            recs[s] = (REL, t)
    return (f"R{k}", (t, old[1], tuple(recs), old[3], gc, lu), [])


def state_flags(geo, arm, ring):
    v = ring[-1]
    flags = []
    refs = {}
    for o in v[1]:
        for s in (o[2],) + o[3]:
            refs[s] = refs.get(s, 0) + 1
            r = v[2][s]
            if r is None or r[0] != ALLOC:
                flags.append("live_lost" if o[1] >= 1 else "del_lost")
    if any(o[1] >= 1 and len(o[3]) < o[4] for o in v[1]):
        flags.append("live_truncated")
    if any(c > 1 for c in refs.values()):
        flags.append("double_ref")
    if any(r is not None and r[0] == ALLOC and s not in refs for s, r in enumerate(v[2])):
        flags.append("leak")
    q = set(queue_of(arm, v))
    nl = {o[0]: o[1] for o in v[1]}
    if any(nl.get(x, 0) >= 1 for x in q):
        flags.append("queue_has_live")
    if any(x not in nl for x in q):
        flags.append("queue_has_gone")
    if any(n == 0 and x not in q for x, n in nl.items()):
        flags.append("zero_not_queued")
    fl = floor_of(ring)
    if any(x in nl and next(o for o in v[1] if o[0] == x)[3] for x in q) and place(geo, v, fl, 1, arm.del_in_r) is None:
        flags.append("cont_blocked")
    if v[4] is not None:
        flags.append("gc_" + gc_only_outcome(geo, arm, ring))
    return flags


def gc_only_outcome(geo, arm, ring):
    for _ in range(2 * geo.segsz + 2):
        cur = ring[-1]
        if cur[4] is None:
            return "completes"
        label, v, events = gc_batch(geo, arm, ring, cur, floor_of(ring), cur[0] + 1)
        if v is None:
            return "paused"
        ring = (ring + (v,))[-geo.ring:]
    return "completes" if ring[-1][4] is None else "livelock"


ARMS = {a.name: a for a in [
    Arm("nogc", gc=False),
    Arm("AB-old-recheck"), Arm("AB-old-skip", cursor="skip"),
    Arm("AB-cur-recheck", lops_rb="cur"), Arm("AB-cur-skip", lops_rb="cur", cursor="skip"),
    Arm("C-old-recheck", carrier="C"), Arm("C-old-recheck-naive", carrier="C", batch="naive"),
    Arm("C-cur-recheck", carrier="C", lops_rb="cur"),
    Arm("C-cur-recheck-naive", carrier="C", lops_rb="cur", batch="naive"),
    Arm("adm-plain"), Arm("adm-rterm", admission="rterm"), Arm("adm-rterm_issue", admission="rterm_issue"),
    Arm("adm-delexempt", del_in_r=True),
    Arm("mut-gc_forget_release", mutation="gc_forget_release"),
    Arm("mut-rb_no_resurrect", mutation="rb_no_resurrect"),
]}

COUNTERS = ["live_lost", "live_truncated", "del_lost", "double_ref", "leak", "queue_has_live", "queue_has_gone", "zero_not_queued",
            "cont_blocked", "gc_completes", "gc_paused", "gc_livelock",
            "ev_gc_del", "ev_false_enospc", "ev_unlink_stuck", "ev_delete_stuck",
            "ev_rb_refused_no_record", "ev_rb_refused_gen", "ev_gc_paused_admission", "ev_gc_paused_placement",
            "ev_gc_unreferenced"]


def path_to(parent, ring):
    labels = []
    while parent[ring] is not None:
        ring, label = parent[ring]
        labels.append(label)
    return ".".join(reversed(labels)) or "-"


def explore(geo, arm, depth):
    nslot = geo.nslot
    start = canonical(((0, (), (None,) * nslot, (), None, ()),))
    parent = {start: None}
    frontier = [start]
    counts = {c: 0 for c in COUNTERS}
    witness = {}
    for d in range(depth):
        nxt = []
        for ring in frontier:
            for label, v, events in successors(geo, arm, ring):
                for e in events:
                    counts["ev_" + e] += 1
                    witness.setdefault("ev_" + e, path_to(parent, ring) + "." + label)
                if v is None:
                    continue
                nring = canonical((ring + (v,))[-geo.ring:])
                if nring in parent:
                    continue
                parent[nring] = (ring, label)
                nxt.append(nring)
                for f in state_flags(geo, arm, nring):
                    counts[f] += 1
                    witness.setdefault(f, path_to(parent, nring))
        frontier = nxt
        if not frontier:
            break
    return len(parent), counts, witness


def main(argv):
    if argv[1:2] == ["list"]:
        print(" ".join(ARMS))
        return 0
    arm = ARMS[argv[1]]
    nseg, segsz, ring, maxobj, maxdata, depth = (int(x) for x in argv[2:8])
    geo = Geo(nseg, segsz, ring, maxobj, maxdata)
    states, counts, witness = explore(geo, arm, depth)
    print(f"E name={arm.name} {geo.label()} window={geo.window} depth={depth} states={states} "
          + " ".join(f"{c}={counts[c]}" for c in COUNTERS))
    for c in COUNTERS:
        if c in witness and c not in ("gc_completes", "gc_paused"):
            print(f"W name={arm.name} {geo.label()} depth={depth} flag={c} path={witness[c]}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
