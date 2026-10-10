#!/usr/bin/env python3
# usage: mut.py apply|restore <row-line-number>   (works in ./work, rows from the frozen mutations.tsv)
import sys,shutil,os
mode,n=sys.argv[1],int(sys.argv[2])
rows=open('/tmp/claude-1000/m3-batch10-code-r2/tree/crates/mutations.tsv',encoding='utf-8').read().split('\n')
f=rows[n-1].split('\t')
path=os.path.join('/tmp/claude-1000/m3-batch10-code-r2-sonnet/work',f[1])
bak=path+'.orig'
if mode=='apply':
    src=open(path,encoding='utf-8').read()
    old=f[2].replace('\\n','\n'); new=f[3].replace('\\n','\n')
    assert src.count(old)==1, ('anchor hits',src.count(old))
    shutil.copy(path,bak)
    open(path,'w',encoding='utf-8').write(src.replace(old,new))
    print('applied',n,f[1]); print('test args:',f[4]); print('must red:',f[5])
else:
    shutil.move(bak,path); os.utime(path,None); print('restored',f[1])
