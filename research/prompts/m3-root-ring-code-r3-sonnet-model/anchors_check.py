# 静态核：冻结副本 crates/mutations.tsv 每行的原文在文件里是不是恰好命中一次（不编译、不跑）。
import sys,collections
root='/tmp/claude-1000/m3-root-ring-code-r3/tree'
rows=[]
for i,l in enumerate(open(root+'/crates/mutations.tsv',encoding='utf-8').read().split('\n'),1):
    if not l or l.startswith('#'): continue
    f=l.split('\t')
    if len(f)!=6: print('BADCOLS',i,len(f)); continue
    rows.append((i,f))
cache={}
bad=[]
for i,f in rows:
    name,path,orig,repl,args,red=f
    if path not in cache:
        try: cache[path]=open(root+'/'+path,encoding='utf-8').read()
        except Exception as e: cache[path]=None
    txt=cache[path]
    if txt is None: bad.append((i,'nofile',name[:60])); continue
    o=orig.replace('\\n','\n')
    n=txt.count(o)
    if n!=1: bad.append((i,n,name[:80]))
print('rows',len(rows),'bad',len(bad))
for b in bad: print(b)
