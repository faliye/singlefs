import os,sys
names=sorted(os.listdir('dump-device'))
for n in names:
    a=open('dump-device/'+n,'rb').read(); b=open('dump-impl/'+n,'rb').read()
    assert len(a)==len(b),n
    diffs=[i for i in range(len(a)) if a[i]!=b[i]]
    # group into runs
    runs=[]
    for i in diffs:
        if runs and i==runs[-1][1]+1: runs[-1][1]=i
        else: runs.append([i,i])
    print(f"{n}: len={len(a)} diff_bytes={len(diffs)} runs={[(r[0],r[1]-r[0]+1) for r in runs]}")
