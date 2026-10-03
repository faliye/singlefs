import sys,os,re,subprocess
root='/tmp/claude-1000/m3-formal-code-r1/tree'
tsv=sys.argv[1]; base=sys.argv[2]  # base dir for files
rows=[]
for i,l in enumerate(open(tsv,encoding='utf-8'),1):
    l=l.rstrip('\n')
    if not l or l.startswith('#'): continue
    f=l.split('\t'); rows.append((i,f))
bad=0
cache={}
for i,f in rows:
    if len(f)<4: print('SHORT',i,len(f)); continue
    name,file,orig,repl=f[0],f[1],f[2].replace('\\n','\n'),f[3].replace('\\n','\n')
    p=os.path.join(root,file)
    if p not in cache:
        cache[p]=open(p,encoding='utf-8').read() if os.path.exists(p) else None
    if cache[p] is None:
        print('NOFILE',i,file); continue
    n=cache[p].count(orig)
    t=''
    if len(f)>=6:
        t=f[5]
    out=[]
    if n!=1: out.append('anchor=%d'%n)
    if orig==repl: out.append('noop')
    if out: print(i,out,name[:60]); bad+=1
print('rows',len(rows),'bad',bad)
