import subprocess, os, re, sys
W=os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-e-code-r1-sonnet')
exec(open(W+'/mut/mutants.py').read())
target=W+'/tree-mut/crates/singlefs-harness/src/gpu_memory_ledger.rs'
src=open(W+'/mut/gpu_memory_ledger.orig.rs').read()
cut=src.index("#[cfg(test)]\nmod tests {")
env=dict(os.environ, CARGO_TARGET_DIR=W+'/target-mut')
cmd=['nice','-n','19','bash','/home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh','12G','bash','/home/fy5090/code/singlefs/research/scripts/capped.sh','6','cargo','test','--offline','-p','singlefs-harness','--test','killers_new','--','--test-threads','4']
ids=sys.argv[1:]
for i in ['BASE']+ids:
    if i=='BASE':
        open(target,'w').write(src); d='未变异'
    else:
        x=[t for t in M if t[0]==i][0]; d=x[1]
        open(target,'w').write(src[:cut].replace(x[2],x[3],1)+src[cut:])
    p=subprocess.run(cmd,cwd=W+'/tree-mut',env=env,capture_output=True,text=True,timeout=200)
    out=p.stdout+p.stderr
    failed=re.findall(r'^test (\S+) \.\.\. FAILED',out,re.M)
    res=re.findall(r'^test result: .*$',out,re.M)
    print(i,'rc=',p.returncode,'failed=',failed,res[-1:] if res else ('COMPILE?' if 'error' in out else ''),'|',d,flush=True)
open(target,'w').write(src)
