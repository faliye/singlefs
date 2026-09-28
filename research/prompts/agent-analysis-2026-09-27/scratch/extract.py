import json,collections,re,datetime,os
p='~/.claude/projects/-home-fy5090-code-singlefs/d16a74c5-453c-44d5-8a19-7e71d116de72.jsonl'
out='/tmp/claude-1000/agent-analysis/scratch/'
os.makedirs(out+'prompts',exist_ok=True)
seen=set()
def jst(ts):
    t=datetime.datetime.fromisoformat(ts.replace('Z','+00:00'))+datetime.timedelta(hours=9)
    return t.strftime('%m-%d %H:%M')
events=[]  # (ts, cat, dict)
pending={}  # tool_use_id -> (name,input,ts)
def ttext(x):
    t=x.get('content')
    if isinstance(t,list): t=' '.join(y.get('text','') for y in t if isinstance(y,dict))
    return str(t or '')
for line in open(p):
    try: r=json.loads(line)
    except: continue
    u=r.get('uuid')
    if u in seen or u is None: continue
    seen.add(u)
    ts=r.get('timestamp')
    if not ts: continue
    m=r.get('message')
    if r.get('type')=='system' and r.get('subtype')=='compact_boundary':
        events.append((ts,'compact',{}))
    if r.get('type')=='system' and r.get('subtype')=='api_error':
        events.append((ts,'api_error',{'t':json.dumps(r,ensure_ascii=False)[:300]}))
    if r.get('type')=='assistant':
        for c in m.get('content',[]) or []:
            if isinstance(c,dict) and c.get('type')=='tool_use':
                if c['name'] in ('Agent','SendMessage','TaskStop','AskUserQuestion','Write','Edit','Read','ListAgents','ToolSearch'):
                    pending[c['id']]=(c['name'],c['input'],ts)
                    events.append((ts,'call_'+c['name'],{'id':c['id'],'input':c['input']}))
                elif c['name']=='Bash':
                    events.append((ts,'call_Bash',{'id':c['id'],'cmd':c['input'].get('command',''),'desc':c['input'].get('description',''),'bg':c['input'].get('run_in_background',False)}))
                    pending[c['id']]=('Bash',c['input'],ts)
    if r.get('type')=='user':
        c=m.get('content')
        if isinstance(c,str):
            events.append((ts,'user_text',{'t':c,'meta':r.get('isMeta'),'compact':r.get('isCompactSummary')}))
        elif isinstance(c,list):
            for x in c:
                if not isinstance(x,dict): continue
                if x.get('type')=='text':
                    events.append((ts,'user_text',{'t':x['text'],'meta':r.get('isMeta'),'compact':r.get('isCompactSummary')}))
                if x.get('type')=='tool_result':
                    tid=x.get('tool_use_id'); name=pending.get(tid,('?',))[0]
                    events.append((ts,'result_'+name,{'id':tid,'err':x.get('is_error'),'t':ttext(x),'tur':r.get('toolUseResult')}))
events.sort(key=lambda e:e[0])
import pickle
pickle.dump(events,open(out+'events.pkl','wb'))
print(len(events), collections.Counter(e[1] for e in events).most_common())
