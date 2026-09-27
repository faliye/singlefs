import json,os,glob,collections,pickle,datetime,re
d='/home/fy5090/.claude/projects/-home-fy5090-code-singlefs/d16a74c5-453c-44d5-8a19-7e71d116de72/subagents/'
out='/tmp/claude-1000/agent-analysis/scratch/'
stats={}
for f in glob.glob(d+'agent-*.jsonl'):
    aid=os.path.basename(f)[6:-6]
    meta={}
    try: meta=json.load(open(f[:-6]+'.meta.json'))
    except: pass
    seen=set(); calls=collections.Counter(); maxctx=0; first=last=None; handback=False; interrupted=False
    bashcmds=[]; denials=[]; nturns=0; incoming=0; errors=0
    for line in open(f):
        try: r=json.loads(line)
        except: continue
        u=r.get('uuid')
        if u in seen: continue
        seen.add(u)
        ts=r.get('timestamp')
        if ts:
            first=first or ts; last=ts
        m=r.get('message') or {}
        if r.get('type')=='assistant':
            us=m.get('usage') or {}
            ctx=us.get('input_tokens',0)+us.get('cache_read_input_tokens',0)+us.get('cache_creation_input_tokens',0)
            maxctx=max(maxctx,ctx)
            for c in m.get('content',[]) or []:
                if isinstance(c,dict) and c.get('type')=='tool_use':
                    calls[c['name']]+=1
                    if c['name']=='SubagentHandback': handback=True
                    if c['name']=='Bash': bashcmds.append(c['input'].get('command','')[:300])
        if r.get('type')=='user':
            c=m.get('content')
            if isinstance(c,list):
                for x in c:
                    if isinstance(x,dict) and x.get('type')=='tool_result' and x.get('is_error'):
                        t=x.get('content'); t=' '.join(y.get('text','') for y in t if isinstance(y,dict)) if isinstance(t,list) else str(t)
                        errors+=1
                        if 'hook' in t.lower(): denials.append(t[:400])
                    if isinstance(x,dict) and x.get('type')=='text' and '[Request interrupted' in x.get('text',''): interrupted=True
            elif isinstance(c,str) and '[Request interrupted' in c: interrupted=True
    def p(ts): return datetime.datetime.fromisoformat(ts.replace('Z','+00:00'))
    dur=(p(last)-p(first)).total_seconds()/60 if first else 0
    stats[aid]={'type':meta.get('agentType'),'desc':meta.get('description'),'calls':sum(calls.values()),'callsby':dict(calls),'maxctx':maxctx,'dur':dur,'first':first,'handback':handback,'interrupted':interrupted,'denials':denials,'errors':errors,'bash':bashcmds}
pickle.dump(stats,open(out+'substats.pkl','wb'))
print(len(stats))
