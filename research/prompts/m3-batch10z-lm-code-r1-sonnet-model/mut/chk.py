import os, subprocess, importlib.util, sys
W=os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-lm-code-r1-sonnet')
T=os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-lm-code-r1/tree')
R='/home/fy5090/code/singlefs/research/scripts'
GPU='crates/singlefs-checker-tier/src/gpu_unit_checks.rs'; CR='crates/singlefs-checker-tier/src/crash_amplification.rs'
src=open(W+'/mut/scanmut.py',encoding='utf-8').read()
ns={}
# 取 scanmut.py 里 ex 的定义
start=src.index('ex=['); end=src.index(']\nfor e in ex')+1
exec(src[start:end],{'GPU':GPU,'CR':CR},ns)
ex=ns['ex']
want=sys.argv[1:]
for name,file,old,new in ex:
    if name.split()[0] not in want: continue
    orig=open(T+'/'+file,encoding='utf-8').read()
    assert orig.count(old)==1,name
    p=W+'/tree-chk/'+file
    open(p,'w',encoding='utf-8').write(orig.replace(old,new))
    env=dict(os.environ,CARGO_TARGET_DIR=W+'/target-chk')
    r=subprocess.run(['nice','-n','19','bash',R+'/capped.sh','15','cargo','check','--offline','-p','singlefs-checker-tier','--features','gpu','--lib'],cwd=W+'/tree-chk',env=env,capture_output=True,text=True)
    out=(r.stdout+r.stderr).splitlines()
    print(name[:60],'rc=',r.returncode,'|',' / '.join(l[:140] for l in out if l.startswith('warning: unused') or l.startswith('error') or 'Finished' in l)[:500])
    open(p,'w',encoding='utf-8').write(orig)
