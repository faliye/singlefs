import pickle,re,collections,json,datetime
out='/tmp/claude-1000/agent-analysis/scratch/'
events=pickle.load(open(out+'events.pkl','rb'))
def jst(ts):
    t=datetime.datetime.fromisoformat(ts.replace('Z','+00:00'))+datetime.timedelta(hours=9)
    return t.strftime('%m-%d %H:%M')
calls={}
agents={}  # agentId -> dict
denied=[]
for ts,cat,d in events:
    if cat=='call_Agent':
        calls[d['id']]=(ts,d['input'])
    if cat=='result_Agent':
        ts0,inp=calls.get(d['id'],(ts,{}))
        tur=d.get('tur') or {}
        aid=tur.get('agentId') if isinstance(tur,dict) else None
        if not aid:
            mm=re.search(r'agentId: (\w+)',d['t']); aid=mm.group(1) if mm else None
        if aid:
            agents[aid]={'ts':ts0,'type':inp.get('subagent_type','general-purpose'),'model':inp.get('model',''),'resolved':tur.get('resolvedModel','') if isinstance(tur,dict) else '','desc':inp.get('description',''),'prompt':inp.get('prompt',''),'bg':inp.get('run_in_background'),'notes':[],'sends':[],'stops':[],'handback':[], 'sync_result': None if (isinstance(tur,dict) and tur.get('status')=='async_launched') else d['t'][:300]}
            open(out+'prompts/'+aid+'.txt','w').write(inp.get('prompt',''))
        else:
            denied.append((ts0,inp.get('subagent_type'),inp.get('description'),d['t'][:600]))
# notifications
tn_re=re.compile(r'<task-notification>\s*<task-id>(\w+)</task-id>.*?<status>(\w+)</status>(.*?)</task-notification>',re.S)
for ts,cat,d in events:
    if cat=='user_text':
        t=d['t']
        for mm in tn_re.finditer(t):
            tid,st,rest=mm.groups()
            if tid in agents: agents[tid]['notes'].append((ts,st,rest[:1500]))
        for mm in re.finditer(r'<agent-message from="(\w+)">',t):
            if mm.group(1) in agents: agents[mm.group(1)]['handback'].append((ts,t[:6000]))
sm={}
for ts,cat,d in events:
    if cat=='call_SendMessage':
        sm[d['id']]=(ts,d['input'])
    if cat=='result_SendMessage':
        ts0,inp=sm.get(d['id'],(ts,{}))
        to=inp.get('to','')
        rec=(ts0,inp.get('summary',''),str(inp.get('message',''))[:3000],d.get('err'),d['t'][:500])
        if to in agents: agents[to]['sends'].append(rec)
        else: agents.setdefault('_other_sends',{'sends':[]})['sends'].append((to,)+rec)
tsk={}
for ts,cat,d in events:
    if cat=='call_TaskStop': tsk[d['id']]=(ts,d['input'])
    if cat=='result_TaskStop':
        ts0,inp=tsk.get(d['id'],(ts,{}))
        tid=inp.get('task_id') or inp.get('shell_id')
        if tid in agents: agents[tid]['stops'].append((ts0,d['t'][:300]))
        else: agents.setdefault('_other_stops',{'stops':[]})['stops'].append((tid,ts0,d['t'][:200]))
pickle.dump((agents,denied),open(out+'agents.pkl','wb'))
real={k:v for k,v in agents.items() if not k.startswith('_')}
print('agents',len(real),'denied dispatch',len(denied))
print('other sends',len(agents.get('_other_sends',{}).get('sends',[])),'other stops',len(agents.get('_other_stops',{}).get('stops',[])))
