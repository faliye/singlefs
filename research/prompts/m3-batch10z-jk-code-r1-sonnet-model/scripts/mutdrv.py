import json, os, subprocess, sys, shutil, threading, queue, re, shlex
W=os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-jk-code-r1-sonnet')
R='/home/fy5090/code/singlefs/research/scripts'
def load(path):
    rows=[]
    for line in open(path,encoding='utf-8'):
        f=line.rstrip('\n').split('\t')
        if len(f)!=6: continue
        name,file,old,new,cmd,test=f
        rows.append((name,file,old.replace('\\n','\n'),new.replace('\\n','\n'),cmd))
    return rows
jobs=[]
for p in sys.argv[2:]:
    if p.endswith('.json'):
        for name,file,old,new,cmd in json.load(open(p,encoding='utf-8')): jobs.append((name,file,old,new,cmd))
    else: jobs+=load(p)
nworkers=int(sys.argv[1])
q=queue.Queue()
for i,j in enumerate(jobs): q.put((i,j))
results={}
def worker(w):
    tree=f'{W}/tree-m{w}'; tgt=f'{W}/target-m{w}'
    while True:
        try: i,(name,file,old,new,cmd)=q.get_nowait()
        except queue.Empty: return
        path=f'{tree}/{file}'
        orig=open(f'{W}/tree-a/{file}',encoding='utf-8').read()
        if orig.count(old)!=1:
            results[i]=(name,'OLD-NOT-ONCE',str(orig.count(old))); continue
        open(path,'w',encoding='utf-8').write(orig.replace(old,new))
        env=dict(os.environ,CARGO_TARGET_DIR=tgt)
        c=['nice','-n','19','bash',f'{R}/run-with-memory-cap.sh','8G','bash',f'{R}/capped.sh','4','cargo','test','--offline']+shlex.split(cmd)
        r=subprocess.run(c,cwd=tree,env=env,capture_output=True,text=True)
        out=r.stdout+r.stderr
        open(f'{W}/mutlogs/{i:02d}.log','w').write(out)
        tr=[l for l in out.splitlines() if l.startswith('test result:')]
        failed=[l for l in out.splitlines() if l.startswith('test ') and l.endswith('FAILED')]
        results[i]=(name,f'rc={r.returncode}',' | '.join(tr)[:200],';'.join(x[5:-9][:90] for x in failed)[:300])
        open(path,'w',encoding='utf-8').write(orig)
os.makedirs(f'{W}/mutlogs',exist_ok=True)
ts=[threading.Thread(target=worker,args=(w,)) for w in range(1,nworkers+1)]
[t.start() for t in ts]; [t.join() for t in ts]
for i in sorted(results): print(i,*results[i],sep='\t')
