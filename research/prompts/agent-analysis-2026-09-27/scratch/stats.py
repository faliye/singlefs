import pickle,re,collections
out='/tmp/claude-1000/agent-analysis/scratch/'
agents,denied=pickle.load(open(out+'agents.pkl','rb'))
real={k:v for k,v in agents.items() if not k.startswith('_')}
by=collections.defaultdict(list)
for k,v in real.items(): by[v['type']].append((k,v))
print('type\tn\tfinal_status_counts\tany_failed\tany_killed\tsends\tsends_err\tagents_with_sends\tstops\thandback_n\tno_handback')
for t,lst in sorted(by.items(),key=lambda x:-len(x[1])):
    st=collections.Counter(); fail=kill=0; sends=0; senderr=0; aws=0; stops=0; hb=0; nohb=0
    for k,v in lst:
        s=[n[1] for n in v['notes']]
        st[s[-1] if s else 'none']+=1
        if 'failed' in s: fail+=1
        if 'killed' in s: kill+=1
        ok=[x for x in v['sends'] if not x[3]]
        sends+=len(ok); senderr+=len(v['sends'])-len(ok)
        if ok: aws+=1
        stops+=len(v['stops'])
        if v['handback']: hb+=1
        else: nohb+=1
    print(f"{t}\t{len(lst)}\t{dict(st)}\t{fail}\t{kill}\t{sends}\t{senderr}\t{aws}\t{stops}\t{hb}\t{nohb}")
print('denied by type', collections.Counter(d[1] for d in denied))
