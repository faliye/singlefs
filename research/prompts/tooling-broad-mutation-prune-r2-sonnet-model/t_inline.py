import json,os,collections
V=os.path.expanduser('~/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4')
rows={r['seq']:r for r in json.load(open('rows.json'))}
res={}
for l in list(open(V+'/results.tsv'))[1:]:
    p=l.rstrip('\n').split('\t');res[int(p[0])]=p
same=diff=harness=other=0
for s,p in res.items():
    if p[3]!='caught': continue
    T=p[5]
    if T.startswith('singlefs-core '):
        mod=T.split(' ',1)[1].split('::')[0]
        if mod==rows[s]['file'].split('/')[-1][:-3]: same+=1
        else: diff+=1
    elif T.startswith('singlefs-harness'): harness+=1
    else: other+=1
c=sum(1 for p in res.values() if p[3]=='caught')
print('caught',c,'| core-lib T in the same file as the mutated function',same,'| core-lib T in another file',diff,'| harness T',harness,'| other',other)
print('share of caught whose T file == mutated file: %.1f%%'%(100*same/c))
