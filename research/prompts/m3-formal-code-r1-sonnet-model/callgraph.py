import re,sys
src=open('/tmp/claude-1000/m3-formal-code-r1/tree/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs',encoding='utf-8').read()
lines=src.split('\n')
clean='\n'.join(re.sub(r'//.*$','',l) for l in lines)
idx=clean.find('#[cfg(test)]')
main_part=clean[:idx] if idx>0 else clean
def funcs(text):
    res={}
    for m in re.finditer(r'\bfn\s+([a-zA-Z0-9_]+)',text):
        name=m.group(1)
        i=m.end(); dp=0
        while i<len(text):
            c=text[i]
            if c=='(' : dp+=1
            elif c==')': dp-=1
            elif c=='{' and dp==0: break
            elif c==';' and dp==0: i=None;break
            i+=1
        if i is None or i>=len(text): continue
        d=0;j=i
        while j<len(text):
            if text[j]=='{': d+=1
            elif text[j]=='}':
                d-=1
                if d==0: break
            j+=1
        res.setdefault(name,'')
        res[name]+=text[i:j+1]
    return res
F=funcs(main_part)
names=set(F)
calls={n:set(x for x in re.findall(r'\b([a-zA-Z0-9_]+)\s*(?:::<[^>]*>)?\s*\(',b) if x in names and x!=n) for n,b in F.items()}
impl={'replay_journal','scan_journal','choose_root','recover'}
direct={n for n,b in F.items() if any(re.search(r'\b'+k+r'\b',b) for k in impl)}
reach=set(direct)
ch=True
while ch:
    ch=False
    for n in names:
        if n not in reach and calls[n]&reach:
            reach.add(n); ch=True
print('tests idx',idx,'funcs',len(names),'direct',sorted(direct))
chk=[n for n in names if n.startswith('check_') or n.startswith('cross_check')]
print('check fns that reach impl:')
for n in sorted(chk):
    if n in reach:
        print(' ',n,'->',sorted(x for x in calls[n] if x in reach)[:6], 'DIRECT' if n in direct else '')
print('total check fns',len(chk))
