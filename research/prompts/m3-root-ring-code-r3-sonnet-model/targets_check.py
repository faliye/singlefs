# 静态核：每行 --test 目标文件在不在、必须红的测试名在不在那个目标（或 --lib 的 src）里；不编译。
import re,glob,os,sys
root='/tmp/claude-1000/m3-root-ring-code-r3/tree/crates'
rows=[]
for i,l in enumerate(open(root+'/mutations.tsv',encoding='utf-8').read().split('\n'),1):
    if not l or l.startswith('#'): continue
    f=l.split('\t')
    if len(f)==6: rows.append((i,f))
tests={}
for p in glob.glob(root+'/*/tests/**/*.rs',recursive=True): tests.setdefault(os.path.basename(p)[:-3],[]).append(p)
bad=[]
for i,f in rows:
    name,path,orig,repl,args,red=f
    m=re.search(r'-p (\S+)',args); pkg=m.group(1) if m else None
    t=re.search(r'--test (\S+)',args)
    if t:
        cand=[p for p in tests.get(t.group(1),[]) if '/'+pkg+'/' in p] if pkg else tests.get(t.group(1),[])
        if not cand: bad.append((i,'no-test-file',t.group(1),name[:50])); continue
        txt=open(cand[0],encoding='utf-8').read()
        # 例：必须红的测试名里可能用 ; 或空格分隔多个
        for r in re.split(r'[;, ]+',red.strip()):
            if r and not re.search(r'fn\s+'+re.escape(r)+r'\b',txt) and r not in txt:
                bad.append((i,'red-test-not-in-file',r[:60],name[:40]))
print('rows',len(rows),'bad',len(bad))
for b in bad[:60]: print(b)
