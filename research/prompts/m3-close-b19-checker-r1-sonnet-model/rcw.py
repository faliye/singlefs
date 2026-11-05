#!/usr/bin/env python3
"""RCW 对旧等式：在 lr5.py 的小世界（可加记录单落版本）上数误红 / 漏红。
用法：python3 rcw.py [recordonly]"""
import sys
import lr5
from lr5 import *

# 在 Root 上记「写者记这条根之后的回收门槛 t」：用发布之后的环现算（对照 mount.rs 的 reclaim_floor_when_the_latest_root_was_recorded）
_old_publish=lr5.publish
def publish_with_t(pool, base, version, refs, rows):
    p=_old_publish(pool,base,version,refs,rows)
    r=p.current
    r.t=reclaim_threshold([(standing(rows,x.version),x.version[1]) for x in p.ring.values()], pool.sysf)
    return p
lr5.publish=publish_with_t
publish=publish_with_t

# mkfs 根的 t：环里只有它自己 → 它的 txg 与 F 取大
_m=lr5.mkfs_pool
def mkfs_with_t():
    p=_m(); p.current.t=max(p.current.version[1],0); return p
lr5.mkfs_pool=mkfs_with_t

# 记录单落的版本 v 没经过 publish，也要有 t：它由它自己的写者记，环里没有它——用「环 + 它」现算（根没落盘，环不含它）
_old_make_root=lr5.make_root
def make_root_t(pool, base, version, refs, rows):
    r=_old_make_root(pool,base,version,refs,rows)
    ring=list(pool.ring.values())
    # 写者记这条根时把它算进环（盖掉同槽旧根）
    pos=version[1]%RING
    kept=[x for p,x in pool.ring.items() if p!=pos]
    r.t=reclaim_threshold([(standing(rows,x.version),x.version[1]) for x in kept]+[(standing(rows,version),version[1])], pool.sysf)
    return r
lr5.make_root=make_root_t
make_root=make_root_t

def E_of(newest, th):
    return {s for s,rec in newest.account.items() if rec[0]=='A' or rec[1]>th}

def verdicts(pool, un, bad=None):
    """返回 (旧等式红?, RCW 红?, 条件说明)；bad 是对最新根账的变异：('drop',slot) 删一条记录；('leak',slot) 凭空加一条未释放记录。"""
    readable=[r for p,r in sorted(pool.ring.items()) if p not in un]
    if not readable: return None
    newest=max(readable,key=lambda r:(r.version[1],r.version[0]))
    st=lambda v: standing(newest.rows,v)
    eff=pool.sysf
    for r in readable:
        if r.version==newest.version or st(r.version)==ABANDONED: eff=max(eff,r.floor)
    if newest.floor!=eff: return 'NA'
    acct=dict(newest.account)
    # 写者的计数器 C：按它自己的账与自己的 t 数（变异前）
    C=len({s for s,rec in acct.items() if rec[0]=='A' or rec[1]>newest.t})
    if bad:
        kind,slot=bad
        if kind=='drop':
            if slot not in acct: return 'NA'
            del acct[slot]            # 记账少记一条记录，计数器跟着自己的账走
            C=len({s for s,rec in acct.items() if rec[0]=='A' or rec[1]>newest.t})
        elif kind=='leak':
            if slot in acct: return 'NA'
            acct[slot]=('A',)
            C=len({s for s,rec in acct.items() if rec[0]=='A' or rec[1]>newest.t})
        elif kind=='short':           # 计数器少数 1，账不动
            C=C-1
    walked=set()
    for r in readable:
        if r.version==newest.version or (st(r.version)==VALID and r.version[1]>=eff): walked|=r.refs
    th_high=reclaim_threshold([(st(r.version),r.version[1]) for r in readable],eff)
    orig=set()
    for r in readable:
        if merge_orig(st(r.version),r.version[1],eff): orig|=r.refs
    rel_hi={s for s,rec in acct.items() if rec[0]=='R' and rec[1]>th_high}
    orig={x for x in orig if x not in walked and x in rel_hi}
    old_red = (C != len(walked|orig))
    def E(th): return {s for s,rec in acct.items() if rec[0]=='A' or rec[1]>th}
    low_ok = C >= len(E(th_high))
    slot_ok = walked <= E(th_high)
    th_low=eff
    U={s for s,rec in acct.items() if rec[0]=='R' and rec[1]>th_low and s not in walked}
    up_ok = C <= min(len(E(th_low)), len(walked|U))
    return old_red, not (low_ok and slot_ok and up_ok), (low_ok,slot_ok,up_ok)

def main(ext):
    seen=lr5.explore(ext)
    stats=dict(worlds=len(seen),judged=0,na=0,old_false_red=0,rcw_false_red=0,old_false_red_full=0,rcw_false_red_full=0)
    mut=dict(drop_n=0,drop_old=0,drop_rcw=0,leak_n=0,leak_old=0,leak_rcw=0,short_n=0,short_old=0,short_rcw=0)
    ex=[]
    for pool in seen.values():
        occ=set(pool.ring.keys())
        for un in subsets(occ):
            v=verdicts(pool,un)
            if v is None: continue
            if v=='NA': stats['na']+=1; continue
            stats['judged']+=1
            old,rcw,_=v
            if old: stats['old_false_red']+=1
            if rcw:
                stats['rcw_false_red']+=1
                if len(ex)<3: ex.append((pool,un,v))
            if not un:
                if old: stats['old_false_red_full']+=1
                if rcw: stats['rcw_false_red_full']+=1
                # 变异只在全可读世界上做
                for slot in range(UNIT_SLOTS):
                    for kind in ('drop','leak'):
                        w=verdicts(pool,un,(kind,slot))
                        if w is None or w=='NA': continue
                        mut[kind+'_n']+=1
                        if w[0]: mut[kind+'_old']+=1
                        if w[1]: mut[kind+'_rcw']+=1
                w=verdicts(pool,un,('short',0))
                if w not in (None,'NA'):
                    mut['short_n']+=1
                    if w[0]: mut['short_old']+=1
                    if w[1]: mut['short_rcw']+=1
    print(stats); print(mut)
    for pool,un,v in ex:
        print("RCW false red example: unreadable",sorted(un),v)
        for p,r in sorted(pool.ring.items()): print("  ring",p,r.version,"F",r.floor,"t",r.t,"refs",sorted(r.refs),"acct",r.account,"rows",r.rows)

if __name__=="__main__":
    main({a:True for a in sys.argv[1:]})
