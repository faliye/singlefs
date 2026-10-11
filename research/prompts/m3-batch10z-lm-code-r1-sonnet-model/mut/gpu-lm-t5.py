import json, os, subprocess, re
W=os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-lm-code-r1-sonnet')
R='/home/fy5090/code/singlefs/scripts'
rows=json.load(open(W+'/mut/gpu-h.json'))
p=W+'/tree-gpu/crates/singlefs-harness/src/gpu_memory_ledger.rs'
orig=open(p,encoding='utf-8').read()
def run(label):
    env=dict(os.environ,CARGO_TARGET_DIR=W+'/target-gpu')
    c=['nice','-n','19','bash',R+'/run-with-memory-cap.sh','8G','bash',R+'/capped.sh','4','cargo','test','--offline','-p','singlefs-harness','--lib','--','--nocapture','gpu_memory_ledger::tests::lm_t5']
    r=subprocess.run(c,cwd=W+'/tree-gpu',env=env,capture_output=True,text=True)
    out=r.stdout+r.stderr
    m=[l for l in out.splitlines() if l.startswith('LM_T5 checked')]
    print(label,'|',(m[0] if m else 'NO-LM_T5-LINE rc=%d'%r.returncode),'|',[l for l in out.splitlines() if 'test result' in l][:1])
print('baseline'); run('baseline')
pick=lambda r: r[0].startswith('第10z批第十三块 显存账 K4-2 借之前看到留着的设备丢了不告诉账') or r[0].startswith('第10z批第十三块 显存账 K4-2 告诉账设备丢了只销那几笔') or r[0].split()[0] in ('Z3','Z5','Z6')
for name,file,old,new,cmd,test in rows:
    if not pick([name]): continue
    old=old.replace('\\n','\n'); new=new.replace('\\n','\n')
    assert orig.count(old)==1,name
    open(p,'w',encoding='utf-8').write(orig.replace(old,new))
    run(name[:60])
open(p,'w',encoding='utf-8').write(orig)
