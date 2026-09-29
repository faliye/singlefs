import json, os, sys, glob, collections, datetime
ROOT='/home/fy5090/.claude/projects/-home-fy5090-code-singlefs'
SINCE=datetime.datetime(2026,9,20,tzinfo=datetime.timezone.utc)
def ts(o):
    t=o.get('timestamp')
    if not t: return None
    try: return datetime.datetime.fromisoformat(t.replace('Z','+00:00'))
    except: return None
def usage_of_file(path):
    seen={}
    first=None; last=None; calls=0
    attach_claude=0; attach_total=0; tool_results=0; tool_result_chars=0
    models=collections.Counter()
    for line in open(path, errors='replace'):
        try: o=json.loads(line)
        except: continue
        t=ts(o)
        if t:
            first=first or t; last=t
        typ=o.get('type')
        if typ=='attachment':
            attach_total+=1
            if 'CLAUDE.md' in line: attach_claude+=1
        if typ=='user':
            c=o.get('message',{}).get('content')
            if isinstance(c,list):
                for b in c:
                    if b.get('type')=='tool_result':
                        tool_results+=1
                        cc=b.get('content')
                        tool_result_chars+= len(cc) if isinstance(cc,str) else len(json.dumps(cc))
        if typ=='assistant':
            m=o.get('message',{})
            u=m.get('usage')
            if not u: continue
            mid=m.get('id') or o.get('uuid')
            seen[mid]=u  # last wins
            models[m.get('model')]+=1
    tot=collections.Counter()
    firstcall=None
    for mid,u in seen.items():
        cc=u.get('cache_creation') or {}
        tot['input']+=u.get('input_tokens',0)
        tot['cache_read']+=u.get('cache_read_input_tokens',0)
        tot['cache_write_5m']+=cc.get('ephemeral_5m_input_tokens', u.get('cache_creation_input_tokens',0) if not cc else 0)
        tot['cache_write_1h']+=cc.get('ephemeral_1h_input_tokens',0)
        tot['output']+=u.get('output_tokens',0)
        if firstcall is None:
            firstcall=u.get('input_tokens',0)+u.get('cache_read_input_tokens',0)+u.get('cache_creation_input_tokens',0)
    tot['calls']=len(seen)
    tot['first_context']=firstcall or 0
    tot['attach_claude']=attach_claude; tot['attach_total']=attach_total
    tot['tool_results']=tool_results; tot['tool_result_chars']=tool_result_chars
    wall=(last-first).total_seconds()/60 if first and last else 0
    return tot, wall, first, models
def folded(t):
    return t['input']+t['cache_read']*0.1+t['cache_write_5m']*1.25+t['cache_write_1h']*2+t['output']*5

mode=sys.argv[1] if len(sys.argv)>1 else 'sub'
if mode=='sub':
    by=collections.defaultdict(lambda: collections.Counter())
    per=[]
    for meta in glob.glob(f'{ROOT}/*/subagents/agent-*.meta.json'):
        jl=meta.replace('.meta.json','.jsonl')
        if not os.path.exists(jl): continue
        mt=datetime.datetime.fromtimestamp(os.path.getmtime(jl),tz=datetime.timezone.utc)
        if mt<SINCE: continue
        try: md=json.load(open(meta))
        except: md={}
        atype=md.get('agentType') or 'unknown'
        tot,wall,first,models=usage_of_file(jl)
        if first and first<SINCE: continue
        tot['n']=1; tot['wall_min']=wall; tot['folded']=folded(tot)
        by[atype].update(tot)
        per.append((atype, os.path.basename(jl), tot, wall, list(models.keys())))
    print(f"{'agentType':26}{'n':>4}{'calls':>7}{'firstctx_avg':>13}{'cache_read(M)':>14}{'w5m(M)':>9}{'w1h(M)':>9}{'input(M)':>10}{'out(K)':>8}{'folded(M)':>11}{'wall_avg_min':>13}{'claudemd_attach':>16}")
    rows=sorted(by.items(), key=lambda kv:-kv[1]['folded'])
    G=collections.Counter()
    for a,t in rows:
        n=t['n']
        print(f"{a:26}{n:>4}{t['calls']:>7}{t['first_context']/n:>13.0f}{t['cache_read']/1e6:>14.1f}{t['cache_write_5m']/1e6:>9.2f}{t['cache_write_1h']/1e6:>9.2f}{t['input']/1e6:>10.2f}{t['output']/1e3:>8.0f}{t['folded']/1e6:>11.2f}{t['wall_min']/n:>13.0f}{t['attach_claude']:>16}")
        G.update(t)
    t=G; n=t['n']
    print(f"{'TOTAL':26}{n:>4}{t['calls']:>7}{t['first_context']/n:>13.0f}{t['cache_read']/1e6:>14.1f}{t['cache_write_5m']/1e6:>9.2f}{t['cache_write_1h']/1e6:>9.2f}{t['input']/1e6:>10.2f}{t['output']/1e3:>8.0f}{t['folded']/1e6:>11.2f}{t['wall_min']/n:>13.0f}{t['attach_claude']:>16}")
    per.sort(key=lambda r:-r[2]['folded'])
    print("\nTop 15 single subagents by folded:")
    for a,f,t,w,models in per[:15]:
        print(f"  {a:24}{f:32} calls={t['calls']:4d} cache_read={t['cache_read']/1e6:6.1f}M w5m={t['cache_write_5m']/1e6:5.2f}M out={t['output']/1e3:5.0f}K folded={t['folded']/1e6:6.2f}M wall={w:5.0f}min {models}")
    # distribution of calls per subagent
    calls=[r[2]['calls'] for r in per]
    calls.sort()
    print("\ncalls per subagent: n=%d median=%d p90=%d max=%d" % (len(calls), calls[len(calls)//2], calls[int(len(calls)*0.9)], calls[-1]))
    fc=[r[2]['first_context'] for r in per]; fc.sort()
    print("first-call context: median=%d p90=%d max=%d" % (fc[len(fc)//2], fc[int(len(fc)*0.9)], fc[-1]))
    # per-call average context (cache_read/calls)
    avgctx=sorted([(r[2]['cache_read']/max(1,r[2]['calls'])) for r in per])
    print("avg cache_read per call: median=%d p90=%d" % (avgctx[len(avgctx)//2], avgctx[int(len(avgctx)*0.9)]))
elif mode=='main':
    print(f"{'session':40}{'calls':>7}{'cache_read(M)':>14}{'w5m(M)':>8}{'w1h(M)':>8}{'input(M)':>9}{'out(K)':>8}{'folded(M)':>11}{'avgctx(K)':>10}{'wall_h':>7}{'subagents':>10}")
    G=collections.Counter()
    for jl in sorted(glob.glob(f'{ROOT}/*.jsonl')):
        mt=datetime.datetime.fromtimestamp(os.path.getmtime(jl),tz=datetime.timezone.utc)
        if mt<SINCE: continue
        tot,wall,first,models=usage_of_file(jl)
        if tot['calls']==0: continue
        sid=os.path.basename(jl)[:8]
        nsub=len(glob.glob(jl[:-6]+'/subagents/*.jsonl'))
        f=folded(tot); tot['folded']=f; tot['nsub']=nsub; tot['n']=1
        G.update(tot)
        print(f"{sid:40}{tot['calls']:>7}{tot['cache_read']/1e6:>14.1f}{tot['cache_write_5m']/1e6:>8.2f}{tot['cache_write_1h']/1e6:>8.2f}{tot['input']/1e6:>9.2f}{tot['output']/1e3:>8.0f}{f/1e6:>11.2f}{tot['cache_read']/max(1,tot['calls'])/1e3:>10.0f}{wall/60:>7.1f}{nsub:>10}")
    t=G
    print(f"{'TOTAL':40}{t['calls']:>7}{t['cache_read']/1e6:>14.1f}{t['cache_write_5m']/1e6:>8.2f}{t['cache_write_1h']/1e6:>8.2f}{t['input']/1e6:>9.2f}{t['output']/1e3:>8.0f}{t['folded']/1e6:>11.2f}{'':>10}{'':>7}{t['nsub']:>10}")
