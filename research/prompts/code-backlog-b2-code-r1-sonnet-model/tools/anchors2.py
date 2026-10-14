import sys,re
root=sys.argv[1]; diff=open(sys.argv[2],encoding='utf-8').read()
tsv=[l for l in open(f'{root}/crates/mutations.tsv',encoding='utf-8').read().split('\n') if l and not l.startswith('#')]
byname={}
for l in tsv: byname.setdefault(l.split('\t')[0],[]).append(l)
src=open(f'{root}/crates/singlefs-core/src/mount.rs',encoding='utf-8').read()
for sec in diff.split('\n## ')[1:]:
    if sec.startswith('三、') or sec.startswith('四、'):
        m=re.search(r'```text\n(.*?)\n```',sec,re.S)
        for l in m.group(1).split('\n'):
            n=l.split('\t')[0]
            rows=byname.get(n,[])
            print(len(rows), 'identical' if rows and rows[0]==l else 'DIFF', src.count(l.split('\t')[2].replace('\\n','\n')), n[:50])
