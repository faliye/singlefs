import sys,os,subprocess,shlex,re
W=os.path.expanduser('~/.local/share/singlefs/work/m3-batch52-54-code-r1-sonnet')
frozen=os.path.expanduser('~/.local/share/singlefs/work/m3-fixes-code-r1/repo')
rows=[l.rstrip('\n').split('\t') for l in open(frozen+'/crates/mutations.tsv',encoding='utf-8')]
idx=int(sys.argv[1]) if sys.argv[1].isdigit() else None
pat=sys.argv[1]
sel=[r for r in rows if (r[0].startswith(pat))]
assert len(sel)==1,(pat,len(sel))
r=sel[0]
name,f,o,rep,cmd,exp=r[:6]
o=o.replace('\\n','\n'); rep=rep.replace('\\n','\n')
p=os.path.join(W,'repo',f)
orig=open(p,encoding='utf-8').read()
assert orig.count(o)==1,'anchor count %d'%orig.count(o)
open(p,'w',encoding='utf-8').write(orig.replace(o,rep))
try:
    env=dict(os.environ); env['CARGO_TARGET_DIR']=W+'/target'
    c=['nice','-n','19','bash','/home/fy5090/code/singlefs/scripts/run-with-memory-cap.sh','10G','bash','/home/fy5090/code/singlefs/scripts/capped.sh','12','cargo','test','--offline','--no-fail-fast']+shlex.split(cmd)
    pr=subprocess.run(c,cwd=W+'/repo',env=env,capture_output=True,text=True,timeout=int(sys.argv[2]) if len(sys.argv)>2 else 230)
    out=pr.stdout+pr.stderr
    res=[l for l in out.splitlines() if l.startswith('test result') or l.endswith('FAILED') or l.startswith('error')]
    print(name[:90]); print('  expected:',exp[:100]); print('  rc',pr.returncode); [print('  ',l[:200]) for l in res[:8]]
finally:
    open(p,'w',encoding='utf-8').write(orig)
