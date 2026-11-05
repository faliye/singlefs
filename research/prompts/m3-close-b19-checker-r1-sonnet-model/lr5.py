#!/usr/bin/env python3
"""LR5 小世界的 Python 移植（对照 crates/singlefs-checker/src/walk.rs 22102 起那条测试）。
基线应钉出 (180794, 2710980, 10204, 1082)。扩展由命令行开关打开。"""
import sys
from itertools import combinations

UNIT_SLOTS = 2
RING = 4
LIMIT = 5

VALID, ABANDONED = 0, 1

def freeze_dict(d):
    return tuple(sorted(d.items()))

# 记录：('A',) 或 ('R', gen, (inst,txg))
class Root:
    __slots__ = ("version","floor","refs","account","rows","key","t")
    def __init__(self, version, floor, refs, account, rows):
        self.version=version; self.floor=floor; self.refs=frozenset(refs)
        self.account=dict(account); self.rows=dict(rows)
        self.key=(version,floor,tuple(sorted(self.refs)),freeze_dict(self.account),freeze_dict(self.rows))

class Pool:
    __slots__=("ring","current","sysf","hi","key","extra")
    def __init__(self, ring, current, sysf, hi, extra=()):
        self.ring=dict(ring); self.current=current; self.sysf=sysf; self.hi=hi; self.extra=extra
        self.key=(tuple(sorted((p,r.key) for p,r in self.ring.items())),current.key,sysf,hi,extra)

def standing(rows, version):
    inst,txg=version
    if inst in rows and txg>rows[inst]: return ABANDONED
    return VALID

def reclaim_threshold(pairs, floor):
    valid=[t for s,t in pairs if s==VALID]
    if not valid: return floor
    return max(min(valid), floor)

def allocatable(pool, base, rows):
    th=reclaim_threshold([(standing(rows,r.version),r.version[1]) for r in pool.ring.values()], pool.sysf)
    iso=set()
    for r in pool.ring.values():
        if standing(rows,r.version)==ABANDONED: iso|=r.refs
    out=set()
    for s in range(UNIT_SLOTS):
        if s in base.refs or s in iso: continue
        rec=base.account.get(s)
        if rec is None: out.add(s)
        elif rec[0]=='A': pass
        else:
            if rec[1]<=th: out.add(s)
    return out

def subsets(c):
    m=sorted(c)
    return [frozenset(x for i,x in enumerate(m) if mask>>i&1) for mask in range(1<<len(m))]

def make_root(pool, base, version, refs, rows):
    acc=dict(base.account)
    for s in base.refs-refs: acc[s]=('R',version[1],base.version)
    for s in refs-base.refs: acc[s]=('A',)
    return Root(version,pool.sysf,refs,acc,rows)

def publish(pool, base, version, refs, rows):
    root=make_root(pool,base,version,refs,rows)
    ring=dict(pool.ring); ring[version[1]%RING]=root
    return Pool(ring,root,pool.sysf,pool.hi,pool.extra)

def successors(pool, ext):
    out=[]
    cur=pool.current
    nv=(cur.version[0],cur.version[1]+1)
    alloc=allocatable(pool,cur,cur.rows)
    reach=cur.refs|alloc
    for refs in subsets(reach):
        out.append(publish(pool,cur,nv,refs,cur.rows))
    for nf in range(pool.sysf+1,cur.version[1]+1):
        raised=Pool(pool.ring,pool.current,nf,pool.hi,pool.extra)
        out.append(publish(raised,cur,nv,cur.refs,cur.rows))
    hi_txg=max(r.version[1] for r in pool.ring.values())
    for landed in list(pool.ring.values()):
        lt=landed.version[1]
        if not (standing(cur.rows,landed.version)==VALID and lt>=pool.sysf): continue
        rows=dict(landed.rows)
        for newer in pool.ring.values():
            if newer.version[1]>lt:
                i=newer.version[0]
                rows[i]=min(rows.get(i,lt),lt)
        rec=Pool(pool.ring,landed,pool.sysf,pool.hi+1,pool.extra)
        rv=(rec.hi,hi_txg+1)
        a2=allocatable(rec,landed,rows)
        for refs in subsets(landed.refs|a2):
            out.append(publish(rec,landed,rv,refs,rows))
    if ext.get('recordonly'):
        out.extend(record_only_successors(pool,ext))
    return out

def record_only_successors(pool, ext):
    """扩展一：当前版本之后再发一版 V，记录落了、根槽没落；恢复施加 V（V 不在环里），
    新实例的写行发布从 V 起，txg = max(环里最大, V.txg) + 1。"""
    out=[]
    cur=pool.current
    V=(cur.version[0],cur.version[1]+1)
    alloc=allocatable(pool,cur,cur.rows)
    for vrefs in subsets(cur.refs|alloc):
        v=make_root(pool,cur,V,vrefs,cur.rows)   # 不进环
        rec=Pool(pool.ring,v,pool.sysf,pool.hi+1,pool.extra)
        rows=dict(v.rows)  # 同实例的根都 ≤ V.txg：行 (i,V.txg) 不改变任何有效性，省略
        hi_txg=max(max(r.version[1] for r in pool.ring.values()),V[1])
        rv=(rec.hi,hi_txg+1)
        a2=allocatable(rec,v,rows)
        for refs in subsets(v.refs|a2):
            out.append(publish(rec,v,rv,refs,rows))
    return out

def merge_orig(st,txg,eff):
    return txg<eff if st==VALID else False

def judge(pool, unreadable, ext):
    readable=[r for p,r in sorted(pool.ring.items()) if p not in unreadable]
    if not readable: return None
    newest=max(readable,key=lambda r:(r.version[1],r.version[0]))
    st=lambda v: standing(newest.rows,v)
    eff=pool.sysf
    for r in readable:
        if r.version==newest.version or st(r.version)==ABANDONED:
            eff=max(eff,r.floor)
    walked=set()
    for r in readable:
        if r.version==newest.version or (st(r.version)==VALID and r.version[1]>=eff):
            walked|=r.refs
    th=reclaim_threshold([(st(r.version),r.version[1]) for r in readable],eff)
    rel=set(s for s,rec in newest.account.items() if rec[0]=='R' and rec[1]>th)
    orig=set(); lr5=set()
    for r in readable:
        s=st(r.version)
        if merge_orig(s,r.version[1],eff): orig|=r.refs
        if s==ABANDONED or r.version[1]<eff: lr5|=r.refs
    orig={x for x in orig if x not in walked and x in rel}
    lr5={x for x in lr5 if x not in walked and x in rel}
    return orig,lr5,newest

def mkfs_pool():
    r=Root((1,1),0,{0},{0:('A',)},{})
    return Pool({1:r},r,0,1)

def explore(ext, limit=LIMIT):
    p0=mkfs_pool()
    seen={p0.key:p0}
    frontier=[p0]
    for _ in range(limit):
        nxt=[]
        for p in frontier:
            for s in successors(p,ext):
                if s.key not in seen:
                    seen[s.key]=s; nxt.append(s)
        frontier=nxt
    return seen

def run(ext):
    seen=explore(ext)
    judged=0; orig_nonempty=0; differ=0; bad=0
    examples=[]
    for pool in seen.values():
        occ=set(pool.ring.keys())
        for un in subsets(occ):
            res=judge(pool,un,ext)
            if res is None: continue
            orig,lr5,newest=res
            judged+=1
            if orig: orig_nonempty+=1
            if orig==lr5: continue
            # 断言：不同的槽上，换下它的那一版必须是「在环里、读不出」
            ok=True
            for (_,s) in []: pass
            for s in orig^lr5:
                rec=newest.account[s]
                vrel=rec[2]
                pos=vrel[1]%RING
                inring=pos in pool.ring and pool.ring[pos].version==vrel
                if not (inring and pos in un): ok=False
            if not ok:
                bad+=1
                if len(examples)<3: examples.append((pool,un,orig,lr5))
            differ+=1
    return len(seen),judged,orig_nonempty,differ,bad,examples

if __name__=="__main__":
    ext={a:True for a in sys.argv[1:]}
    n,j,o,d,b,ex=run(ext)
    print("worlds,judged,orig_nonempty,differ,assertion_violations =",(n,j,o,d,b))
    for pool,un,orig,lr5 in ex:
        print("EXAMPLE unreadable",sorted(un),"orig",orig,"lr5",lr5)
        for p,r in sorted(pool.ring.items()):
            print("  ring",p,r.version,"F",r.floor,"refs",sorted(r.refs),"acct",r.account,"rows",r.rows)
        print("  cur",pool.current.version,"sysf",pool.sysf)
