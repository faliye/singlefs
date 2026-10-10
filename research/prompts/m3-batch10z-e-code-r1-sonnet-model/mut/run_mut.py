import subprocess, sys, shutil, os, time, re
W=os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-e-code-r1-sonnet')
exec(open(W+'/mut/mutants.py').read())
target=W+'/tree-mut/crates/singlefs-harness/src/gpu_memory_ledger.rs'
orig=W+'/mut/gpu_memory_ledger.orig.rs'
src=open(orig).read()
cut=src.index("#[cfg(test)]\nmod tests {")
filt=os.environ.get('FILT','gpu_memory_ledger')
a=int(sys.argv[1]); b=int(sys.argv[2])
env=dict(os.environ, CARGO_TARGET_DIR=W+'/target-mut')
cmd=['nice','-n','19','bash','/home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh','12G','bash','/home/fy5090/code/singlefs/research/scripts/capped.sh','6','cargo','test','--offline','-p','singlefs-harness','--lib','--']+([filt] if filt else [])
def run(label, tmp):
    t=time.time()
    try:
        p=subprocess.run(cmd,cwd=W+'/tree-mut',env=env,capture_output=True,text=True,timeout=int(os.environ.get('TMO','200')))
        out=p.stdout+p.stderr; rc=p.returncode
    except subprocess.TimeoutExpired as e:
        out=(e.stdout or b'').decode() if isinstance(e.stdout,bytes) else (e.stdout or ''); rc='timeout'
    dt=time.time()-t
    failed=re.findall(r'^test (\S+) \.\.\. FAILED',out,re.M)
    res=re.findall(r'^test result: .*$',out,re.M)
    cerr='error[' in out or 'could not compile' in out
    open(W+f'/mut/logs/{label}.log','w').write(out)
    return rc,failed,res,cerr,dt
os.makedirs(W+'/mut/logs',exist_ok=True)
for i,d,o,n in M[a:b]:
    new=src[:cut].replace(o,n,1)+src[cut:]
    open(target,'w').write(new)
    rc,failed,res,cerr,dt=run(i+('' if not filt else '') + ('-full' if not filt else ''),None)
    verdict='COMPILE_ERROR' if cerr else ('TIMEOUT' if rc=='timeout' else ('RED' if failed or rc not in (0,) else 'GREEN'))
    line=f"{i}\t{verdict}\trc={rc}\tfailed={len(failed)}\t{';'.join(f.split('::')[-1] for f in failed[:6])}\t{dt:.0f}s\t{d}"
    print(line,flush=True)
    open(W+'/mut/results.tsv','a').write(line+'\n')
shutil.copy(orig,target)
