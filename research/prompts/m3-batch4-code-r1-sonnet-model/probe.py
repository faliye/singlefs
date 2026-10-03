#!/usr/bin/env python3
"""probe.py <label> <file-rel> <orig-file> <repl-file> <cargo-test-args...>
orig/repl are files holding the exact text (so no escaping troubles). Applies to the copy tree, runs the test, restores."""
import sys,subprocess,hashlib,os,shutil
label,rel,origf,replf=sys.argv[1:5]; targs=sys.argv[5:]
T='/tmp/claude-1000/m3-batch4-code-r1-sonnet/tree/'
F='/tmp/claude-1000/m3-batch5-freeze/tree/'
orig=open(origf,encoding='utf-8').read(); repl=open(replf,encoding='utf-8').read()
pass
src=open(T+rel,encoding='utf-8').read()
n=src.count(orig)
if n!=1:
    print(label,'ANCHOR HITS',n); sys.exit(3)
open(T+rel,'w',encoding='utf-8').write(src.replace(orig,repl))
env=dict(os.environ)
env['RUN_WITH_MEMORY_CAP_PARALLEL_REASON']='同一会话派出的几个 agent 各跑各的，不是同一条链重复起'
env['RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON']='变异之后要重编 core/checker 与测试二进制，峰值表里的实测值是不重编的'
cmd=['timeout','225','bash','/home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh','8G','bash','/home/fy5090/code/singlefs/research/scripts/capped.sh','7','nice','-n','19','cargo','test','--offline','-p','singlefs-harness']+targs
try:
    r=subprocess.run(cmd,cwd=T,env=env,capture_output=True,text=True)
    out=r.stdout+r.stderr
    code=r.returncode
finally:
    shutil.copyfile(F+rel,T+rel)
    assert hashlib.sha256(open(T+rel,'rb').read()).hexdigest()==hashlib.sha256(open(F+rel,'rb').read()).hexdigest()
open('/tmp/claude-1000/m3-batch4-code-r1-sonnet/probe-logs/%s.log'%label,'w').write(out)
fails=[l for l in out.split('\n') if ' FAILED' in l or 'error[' in l or 'could not compile' in l]
res=[l for l in out.split('\n') if l.startswith('test result')]
print(label,'exit',code,'| results:',' / '.join(x[:60] for x in res),'| failed:',[f[:150] for f in fails][:4])
