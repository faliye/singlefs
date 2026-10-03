#!/usr/bin/env python3
"""X3 在今天的 crates 里：被变异的两个包（core、checker）的 const fn，哪些在 const / static 项的初始化式里被叫（只认这两种常量上下文；数组长度、常量泛型、const {} 块没认）。在仓根下跑。"""
import re,sys,glob,collections
files=[f for f in glob.glob('crates/**/*.rs',recursive=True)]
src={f:open(f).read() for f in files}
names=set()
for f in files:
    if 'checker-tier' in f: continue
    if not (f.startswith('crates/singlefs-core/src') or f.startswith('crates/singlefs-checker/src')): continue
    for m in re.finditer(r'\bconst fn (\w+)',src[f]): names.add(m.group(1))
pat=re.compile(r'^\s*(?:pub(?:\([^)]*\))?\s+)?(?:const|static)\s+([A-Z_0-9]+)\s*:[^=]*=',re.M)
hits=collections.defaultdict(list)
for f,s in src.items():
    for m in pat.finditer(s):
        # expression until ; at depth 0
        i=m.end(); d=0
        while i<len(s):
            c=s[i]
            if c in '([{': d+=1
            elif c in ')]}': d-=1
            elif c==';' and d==0: break
            i+=1
        expr=s[m.end():i]
        for n in names:
            if re.search(r'\b'+n+r'\s*\(',expr):
                line=s.count('\n',0,m.start())+1
                hits[n].append(f'{f}:{line}:{m.group(1)}')
for n in sorted(hits):
    print(n, len(hits[n]), hits[n][:4])
