import json, os, subprocess, shlex
W=os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-lm-code-r1-sonnet')
R='/home/fy5090/code/singlefs/scripts'
rows=json.load(open(W+'/mut/gpu-h.json'))
want={'Z1','Z6','Z7','Z8'}
p=W+'/tree-gpu/crates/singlefs-harness/src/gpu_memory_ledger.rs'
orig=open(p,encoding='utf-8').read()
for name,file,old,new,cmd,test in rows:
    tag=name.split()[0]
    if tag not in want: continue
    old=old.replace('\\n','\n'); new=new.replace('\\n','\n')
    assert orig.count(old)==1,(tag,orig.count(old))
    open(p,'w',encoding='utf-8').write(orig.replace(old,new))
    env=dict(os.environ,CARGO_TARGET_DIR=W+'/target-gpu')
    c=['nice','-n','19','bash',R+'/run-with-memory-cap.sh','8G','bash',R+'/capped.sh','4','cargo','test','--offline','-p','singlefs-harness','--lib','--','gpu_memory_ledger::tests::lm_']
    r=subprocess.run(c,cwd=W+'/tree-gpu',env=env,capture_output=True,text=True)
    out=r.stdout+r.stderr
    res=[l for l in out.splitlines() if l.startswith('test ') and ('FAILED' in l or ' ok' in l)]
    print(tag,name[:50]); print('\n'.join('   '+x[:150] for x in res))
open(p,'w',encoding='utf-8').write(orig)
