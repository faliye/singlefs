import json, os, subprocess, sys, shlex, threading, queue
W=os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-lm-code-r1-sonnet')
F0=os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-lm-code-r1/tree')
R='/home/fy5090/code/singlefs/scripts'
# usage: drv.py <workers> <threads-each> <tag> <a:b> <json...>
nworkers=int(sys.argv[1]); thr=sys.argv[2]; tag=sys.argv[3]; a,b=map(int,sys.argv[4].split(':'))
jobs=[]
for p in sys.argv[5:]:
    for row in json.load(open(p,encoding='utf-8')):
        name,file,old,new,cmd,test=row
        jobs.append((name,file,old.replace('\\n','\n'),new.replace('\\n','\n'),cmd,test))
jobs=jobs[a:b]
q=queue.Queue()
for i,j in enumerate(jobs): q.put((a+i,j))
results={}
def worker(w):
    tree=f'{W}/tree-m{w}'; tgt=f'{W}/target-m{w}'
    while True:
        try: i,(name,file,old,new,cmd,test)=q.get_nowait()
        except queue.Empty: return
        path=f'{tree}/{file}'
        orig=open(f'{F0}/{file}',encoding='utf-8').read()
        if orig.count(old)!=1:
            results[i]=(name,'OLD-NOT-ONCE',str(orig.count(old)),''); continue
        open(path,'w',encoding='utf-8').write(orig.replace(old,new))
        env=dict(os.environ,CARGO_TARGET_DIR=tgt)
        c=['nice','-n','19','bash',f'{R}/run-with-memory-cap.sh','8G','bash',f'{R}/capped.sh',thr,'cargo','test','--offline']+shlex.split(cmd)
        r=subprocess.run(c,cwd=tree,env=env,capture_output=True,text=True)
        out=r.stdout+r.stderr
        os.makedirs(f'{W}/mut/logs-{tag}',exist_ok=True)
        open(f'{W}/mut/logs-{tag}/{i:02d}.log','w').write(out)
        tr=[l for l in out.splitlines() if l.startswith('test result:')]
        failed=[l[5:-9] for l in out.splitlines() if l.startswith('test ') and l.endswith('FAILED')]
        cerr='COMPILE-ERROR' if 'error[' in out or 'could not compile' in out else ''
        results[i]=(name,f'rc={r.returncode}{cerr}',' | '.join(tr)[:120],';'.join(x.split('::')[-1][:70] for x in failed)[:300])
        open(path,'w',encoding='utf-8').write(orig)
ts=[threading.Thread(target=worker,args=(w,)) for w in range(1,nworkers+1)]
[t.start() for t in ts]; [t.join() for t in ts]
for i in sorted(results): print(i,*results[i],sep='\t')
