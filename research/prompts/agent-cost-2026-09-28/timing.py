import json,glob,os,datetime,collections,re,statistics
ROOT='.'
SINCE=datetime.datetime(2026,9,20,tzinfo=datetime.timezone.utc)
def ts(o):
    t=o.get('timestamp')
    try: return datetime.datetime.fromisoformat(t.replace('Z','+00:00'))
    except: return None
def med(l): return statistics.median(l) if l else 0
# ---------- per subagent timing ----------
agents={}  # id -> dict
for meta in glob.glob(f'{ROOT}/*/subagents/agent-*.meta.json'):
    jl=meta.replace('.meta.json','.jsonl')
    if not os.path.exists(jl) or datetime.datetime.fromtimestamp(os.path.getmtime(jl),tz=datetime.timezone.utc)<SINCE: continue
    md=json.load(open(meta)); atype=md.get('agentType','?')
    aid=os.path.basename(jl)[6:-6]; session=jl.split('/')[1]
    first=None; last=None; asst=[]; handback=None; inj_idx=None; ctxs=[]
    seen=set()
    for line in open(jl, errors='replace'):
        try: o=json.loads(line)
        except: continue
        t=ts(o)
        if t: first=first or t; last=t
        typ=o.get('type')
        if typ=='attachment':
            a=o.get('attachment',{}); p=a.get('filename') or a.get('path') or a.get('file_path') or ''
            if 'singlefs-ai-sop/CLAUDE.md' in str(p) and inj_idx is None: inj_idx=len(seen)
        if typ=='assistant':
            m=o.get('message',{}); mid=m.get('id'); u=m.get('usage')
            if u and mid not in seen:
                seen.add(mid); asst.append(t)
                ctxs.append(u.get('cache_read_input_tokens',0)+u.get('cache_creation_input_tokens',0)+u.get('input_tokens',0))
            for b in m.get('content',[]) or []:
                if isinstance(b,dict) and b.get('type')=='tool_use' and b.get('name')=='SubagentHandback': handback=t
    if not asst: continue
    gaps=[(b-a).total_seconds() for a,b in zip(asst,asst[1:]) if a and b]
    waiting=sum(g for g in gaps if g>180); nwait=sum(1 for g in gaps if g>180)
    wall=(last-first).total_seconds() if first and last else 0
    agents[aid]=dict(type=atype,session=session,start=first,end=last,wall=wall,waiting=waiting,nwait=nwait,calls=len(asst),handback=handback,inj_idx=inj_idx,ctxs=ctxs)
print('subagents', len(agents))
# time per type
by=collections.defaultdict(list)
for a in agents.values(): by[a['type']].append(a)
print(f"\n{'type':26}{'n':>4}{'wall_med(min)':>14}{'wall_p90':>9}{'waiting_med(min)':>17}{'wait_share':>11}{'waits>3min med':>15}{'no_handback':>12}")
for t,l in sorted(by.items(), key=lambda kv:-len(kv[1])):
    walls=[a['wall']/60 for a in l]; waits=[a['waiting']/60 for a in l]
    share=sum(a['waiting'] for a in l)/max(1,sum(a['wall'] for a in l))
    nohb=sum(1 for a in l if a['handback'] is None)
    print(f"{t:26}{len(l):>4}{med(walls):>14.0f}{sorted(walls)[int(len(walls)*0.9)]:>9.0f}{med(waits):>17.0f}{share:>11.0%}{med([a['nwait'] for a in l]):>15.0f}{nohb:>12}")
allw=sum(a['wall'] for a in agents.values()); allwait=sum(a['waiting'] for a in agents.values())
print(f"TOTAL subagent wall {allw/3600:.0f} h, of which gaps>3min (waiting) {allwait/3600:.0f} h ({allwait/allw:.0%})")
nohb=[a for a in agents.values() if a['handback'] is None]
print(f"agents without handback: {len(nohb)}, their wall {sum(a['wall'] for a in nohb)/3600:.0f} h, calls {sum(a['calls'] for a in nohb)}")
# injection measurement
jumps=[]; after=[]
for a in agents.values():
    i=a['inj_idx']
    if i is None or i<1 or i>=len(a['ctxs']): continue
    jumps.append(a['ctxs'][i]-a['ctxs'][i-1]); after.append(len(a['ctxs'])-i)
print(f"\ninjection: agents measured {len(jumps)}, ctx jump at injection median {med(jumps):.0f} p25 {sorted(jumps)[len(jumps)//4]:.0f} p75 {sorted(jumps)[3*len(jumps)//4]:.0f}; calls after injection median {med(after):.0f} sum {sum(after)}; est. injected read tokens {med(jumps)*sum(after)/1e9:.1f}B")

# ---------- main sessions: dispatch batches, reaction latency, concurrency ----------
batch_sizes=[]; react=[]; queue_delay=[]; serial_after_hb=0; total_disp=0
conc_max=[]; alone_share=[]
rounds=collections.defaultdict(dict)
for jl in sorted(glob.glob(f'{ROOT}/*.jsonl')):
    if datetime.datetime.fromtimestamp(os.path.getmtime(jl),tz=datetime.timezone.utc)<SINCE: continue
    sid=os.path.basename(jl)[:-6]
    disp=[]; hb_notif=[]; asst_ts=[]; verdict_write={}
    pending={}
    for line in open(jl, errors='replace'):
        try: o=json.loads(line)
        except: continue
        t=ts(o); typ=o.get('type'); m=o.get('message',{}); c=m.get('content')
        if typ=='assistant' and t: 
            asst_ts.append(t)
            if isinstance(c,list):
                for b in c:
                    if b.get('type')=='tool_use':
                        if b.get('name') in ('Agent','Task'):
                            pending[b['id']]=(t,b['input'].get('subagent_type'),b['input'].get('prompt',''))
                        else:
                            s=json.dumps(b.get('input',{}),ensure_ascii=False)
                            for r in set(re.findall(r'([A-Za-z0-9][A-Za-z0-9-]*-r\d+)-main-verification\.md', s)):
                                verdict_write.setdefault(r,t)
        if typ=='user' and isinstance(c,list):
            for b in c:
                if b.get('type')=='tool_result' and b.get('tool_use_id') in pending:
                    t0,st,pr=pending.pop(b['tool_use_id'])
                    r=o.get('toolUseResult'); aid=r.get('agentId') if isinstance(r,dict) else None
                    if aid: 
                        disp.append((t0,st,aid,pr))
        if typ=='user':
            txt=c if isinstance(c,str) else json.dumps(c) if c else ''
            if '<task-notification>' in txt and t:
                mm=re.search(r'<task-id>(a[0-9a-f]{16})</task-id>', txt)
                if mm: hb_notif.append((t,mm.group(1)))
    total_disp+=len(disp)
    # batches: dispatches within 90s of previous
    disp.sort()
    cur=1
    for a,b in zip(disp,disp[1:]):
        if (b[0]-a[0]).total_seconds()<=90: cur+=1
        else: batch_sizes.append(cur); cur=1
    if disp: batch_sizes.append(cur)
    # reaction latency: subagent end -> notification arrival in main -> next assistant message
    asst_ts.sort()
    import bisect
    for t,aid in hb_notif:
        a=agents.get(aid)
        if a and a['end']:
            qd=(t-a['end']).total_seconds()
            if 0<=qd<6*3600: queue_delay.append(qd)
        i=bisect.bisect_right(asst_ts,t)
        if i<len(asst_ts): react.append((asst_ts[i]-t).total_seconds())
    # dispatch right after a handback (serial): dispatch within 10 min after a notification
    hbt=sorted(t for t,_ in hb_notif)
    for t0,st,aid,pr in disp:
        j=bisect.bisect_right(hbt,t0)
        if j>0 and (t0-hbt[j-1]).total_seconds()<=600: serial_after_hb+=1
    # concurrency profile
    ags=[agents[aid] for _,_,aid,_ in disp if aid in agents]
    if ags:
        s0=min(a['start'] for a in ags); e0=max(a['end'] for a in ags)
        span=(e0-s0).total_seconds(); step=300; alone=0; mx=0; n=0
        tcur=s0
        while tcur<e0:
            running=sum(1 for a in ags if a['start']<=tcur<=a['end'])
            mx=max(mx,running); alone+= (running==0); n+=1; tcur+=datetime.timedelta(seconds=step)
        conc_max.append(mx); alone_share.append(alone/max(1,n))
    # three-way rounds
    for t0,st,aid,pr in disp:
        if not st or not st.startswith('three-way'): continue
        rs=re.findall(r'([A-Za-z0-9][A-Za-z0-9-]*-r\d+)', pr)
        if not rs: continue
        rn=collections.Counter(rs).most_common(1)[0][0]
        a=agents.get(aid)
        if not a: continue
        rounds[(sid,rn)].setdefault(st,[]).append((t0,a['end']))
    for rn,t in verdict_write.items(): rounds[(sid,rn)]['verdict']=t
print(f"\nMAIN: dispatches with agent launched {total_disp}; batch sizes (<=90s apart): median {med(batch_sizes):.0f}, share of size-1 batches {sum(1 for b in batch_sizes if b==1)/len(batch_sizes):.0%}, max {max(batch_sizes)}")
print(f"dispatches made within 10 min after some handback notification: {serial_after_hb} ({serial_after_hb/total_disp:.0%})")
print(f"handback -> notification delivered to main: median {med(queue_delay)/60:.1f} min, p90 {sorted(queue_delay)[int(len(queue_delay)*0.9)]/60:.1f} min (n={len(queue_delay)})")
print(f"notification -> next main assistant message: median {med(react):.0f} s, p90 {sorted(react)[int(len(react)*0.9)]:.0f} s")
print(f"per-session max concurrency: {sorted(conc_max)}; share of time with zero subagents running (median over sessions): {med(alone_share):.0%}")
# three-way round timeline
print("\nthree-way rounds (materials -> legs -> verdict), minutes:")
rows=[]
for (sid,rn),d in rounds.items():
    mats=d.get('three-way-materials',[]); legs=[x for k,v in d.items() if k.startswith('three-way') and k!='three-way-materials' for x in v]
    if not legs: continue
    t_first=min([x[0] for x in mats]+[x[0] for x in legs])
    mat_dur=max([(e-s).total_seconds() for s,e in mats],default=0)/60
    legs_start=min(x[0] for x in legs); legs_end=max(x[1] for x in legs)
    leg_durs=[(e-s).total_seconds()/60 for s,e in legs]
    v=d.get('verdict')
    total=((v-t_first).total_seconds()/60) if v else None
    gap_verdict=((v-legs_end).total_seconds()/60) if v else None
    rows.append((rn,len(mats),mat_dur,len(legs),max(leg_durs),med(leg_durs),gap_verdict,total))
rows.sort(key=lambda r:-(r[7] or 0))
print(f"{'round':34}{'mats':>5}{'mat_min':>8}{'legs':>5}{'leg_max':>8}{'leg_med':>8}{'legs_end->verdict':>18}{'total':>7}")
for r in rows[:25]:
    print(f"{r[0][:34]:34}{r[1]:>5}{r[2]:>8.0f}{r[3]:>5}{r[4]:>8.0f}{r[5]:>8.0f}{(f'{r[6]:.0f}' if r[6] is not None else '-'):>18}{(f'{r[7]:.0f}' if r[7] is not None else '-'):>7}")
tot=[r[7] for r in rows if r[7] is not None]; gv=[r[6] for r in rows if r[6] is not None]
print(f"rounds with verdict located: {len(tot)} of {len(rows)}; total median {med(tot):.0f} min p90 {sorted(tot)[int(len(tot)*0.9)]:.0f}; legs_end->verdict median {med(gv):.0f} min; materials median {med([r[2] for r in rows if r[1]]):.0f} min; leg max median {med([r[4] for r in rows]):.0f} min")
