import re,sys,glob,collections
runs={}
for f in sorted(glob.glob('out/r7-*.log'))+sorted(glob.glob('out/fixed-*.log')):
    for line in open(f,errors='replace'):
        if not line.startswith('OPUS3-RUN'): continue
        m=re.match(r'OPUS3-RUN sp=(\S+) arm="([^"]*)" plan=(\S+) c554=(\S+) desc=(.*?) outcome=(\S+) hooks=(\d+) fired=(\d+)( k_mismatch)? red=(\S+)',line)
        if not m: print('unparsed',line[:120]); continue
        sp,arm,plan,c554,desc,outc,hooks,fired,km,red=m.groups()
        persist='persist=true' in desc or plan.endswith('persist')
        runs[(sp,arm or 'prod',plan)]=dict(outc=outc,red=red=='true',c554=c554=='true',km=bool(km),persist=persist,line=line)
tot=collections.Counter()
for (sp,arm,plan),r in runs.items():
    tot[(arm,'red_not_c554' if r['red'] and not r['c554'] else ('red_c554' if r['red'] else 'green'))]+=1
    if r['km']: tot[(arm,'k_mismatch')]+=1
for k,v in sorted(tot.items()): print('TOT',k,v)
trans=collections.Counter()
for (sp,arm,plan),r in runs.items():
    if arm!='prod': continue
    o=runs.get((sp,'reread_only',plan))
    if not o: continue
    def cls(x): return x['outc']+('/red' if x['red'] else '')+('/c554' if x['c554'] else '')
    trans[(cls(o),cls(r),'persist' if r['persist'] else 'transient')]+=1
for k,v in sorted(trans.items()): print('TRANS reread_only->prod',k,v)
for (sp,arm,plan),r in runs.items():
    if r['red'] and not r['c554'] and arm=='prod': print('PRODRED',r['line'][:600])
print('---')
c=collections.Counter()
for (sp,arm,plan),r in runs.items():
    if arm!='prod': continue
    o=runs.get((sp,'reread_only',plan))
    if not o: continue
    if r['outc'].startswith('Err(FloorRaise') and not o['outc'].startswith('Err(FloorRaise'):
        c[(sp,o['outc'])]+=1
        ex=(r['line'],o['line'])
for k,v in sorted(c.items()): print('FRF',k,v)
print('EX-prod',re.sub(r'desc=\[[^]]*\]','',ex[0])[:700]); print('EX-ro',re.sub(r'desc=\[[^]]*\]','',ex[1])[:500])
# steps refused in prod Ok runs
rc=collections.Counter()
for (sp,arm,plan),r in runs.items():
    m=re.search(r'refused_at=(\d+):(\w+)',r['line'])
    if m: rc[(arm,sp,m.group(2),'persist' if r['persist'] else 'transient')]+=1
for k,v in sorted(rc.items()): print('REFUSED',k,v)
print('--- refusal transitions for persist plans (reread_only -> prod)')
t=collections.Counter()
for (sp,arm,plan),r in runs.items():
    if arm!='prod' or not r['persist']: continue
    o=runs.get((sp,'reread_only',plan))
    if not o: continue
    def ref(x):
        m=re.search(r'refused_at=(\d+):(\w+)',x['line'])
        return x['outc']+(':'+m.group(2) if m else '')
    t[(sp,ref(o),ref(r))]+=1
for k,v in sorted(t.items()): print('PT',k,v)
