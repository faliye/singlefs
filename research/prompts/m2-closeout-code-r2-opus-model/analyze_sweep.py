import re,sys,collections
rows=[]
for line in open(sys.argv[1]):
    if not line.startswith('SWEEP entry=') or ' kind=' not in line: continue
    d={}
    for k in ['entry','kind','n','writes_before','writes_after','barriers_after','frozen','checker','checker_after_remount']:
        m=re.search(r'\b'+k+r'=(\S+)',line); d[k]=m.group(1) if m else None
    d['outcome']=re.search(r'outcome=(.*?) writes_before=',line).group(1)
    d['resend']=re.search(r'resend=(.*?) checker=',line).group(1)
    d['remount']=re.search(r'remount=(.*?) checker_after_remount=',line).group(1)
    d['line']=line.strip()
    rows.append(d)
print('runs',len(rows))
cat=collections.Counter()
ex={}
for d in rows:
    reasons=[]
    if d['checker']!='0': reasons.append('checker_red_after_entry')
    if d['remount']!='Ok': reasons.append('remount_failed')
    if d['checker_after_remount'] not in ('0',None): reasons.append('checker_red_after_remount')
    if d['kind']=='read' and d['outcome'].startswith('Err') and int(d['writes_before'])>0: reasons.append('read_fault_refusal_after_writes')
    if d['kind']!='read' and int(d['writes_after'])>0: reasons.append('writes_after_injected_'+d['kind']+'_failure')
    if d['kind']=='read' and d['outcome'].startswith('Err'): reasons.append('read_fault_err')
    if d['frozen']=='true' and d['resend']!='Ok': reasons.append('resend_not_ok')
    for r in reasons:
        key=(d['entry'],r)
        cat[key]+=1
        ex.setdefault(key,[]).append(d)
for k,v in sorted(cat.items()):
    print(k,v)
    for d in ex[k][:3]:
        print('   ', d['line'][:420])
# outcome classes per entry/kind
oc=collections.Counter()
for d in rows:
    o=re.sub(r'[\s{(].*','',d['outcome'].replace('Err ','Err:'))
    oc[(d['entry'],d['kind'],o,d['frozen'])]+=1
for k,v in sorted(oc.items()): print('OUT',k,v)
