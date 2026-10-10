# K3/K4：冻结副本里每个提到 InvariantVerdict::NotApplicable 或 ::Holds 的文件，有没有 PartiallyJudged；
# 再把每个 `match` 里提到 InvariantVerdict:: 的臂列出来，看有没有通配臂（`_ =>`）吞掉新成员。只读，不编译。
import re,glob,sys,os
root='/tmp/claude-1000/m3-root-ring-code-r3/tree'
files=[p for p in glob.glob(root+'/**/*.rs',recursive=True) if '/target/' not in p]
res=[]
for p in sorted(files):
    t=open(p,encoding='utf-8').read()
    n_holds=len(re.findall(r'InvariantVerdict::Holds',t))
    n_na=len(re.findall(r'InvariantVerdict::NotApplicable',t))
    n_v=len(re.findall(r'InvariantVerdict::Violated',t))
    n_pj=len(re.findall(r'PartiallyJudged',t))
    if n_holds+n_na+n_v:
        res.append((os.path.relpath(p,root),n_holds,n_na,n_v,n_pj))
print('files mentioning InvariantVerdict members:',len(res))
miss=[r for r in res if r[4]==0]
print('files with 0 PartiallyJudged:',len(miss))
for r in miss: print('  ',r)
# 通配臂：在 `match` 里同时出现 InvariantVerdict::X => 与随后的 `_ =>`
print('--- matches with a wildcard arm after InvariantVerdict arms')
for p in sorted(files):
    t=open(p,encoding='utf-8').read().split('\n')
    for i,l in enumerate(t):
        if re.match(r'\s*_\s*=>',l) or re.match(r'\s*_[a-z_]+\s*=>',l):
            ctx='\n'.join(t[max(0,i-12):i])
            if 'InvariantVerdict::' in ctx:
                print(os.path.relpath(p,root),i+1,l.strip()[:80])
