#!/usr/bin/env python3
# 草稿：在 sonnet 腿自己的副本上逐行应用 mutations.tsv 的一行，跑点名测试，看点名测试红不红，还原。
import sys, subprocess, shutil, os, re, time
TREE='/tmp/claude-1000/m3-unlink-code-r3-sonnet/tree/'
PRISTINE='/tmp/claude-1000/m3-unlink-code-r3/tree/'
rows=[l.rstrip('\n').split('\t') for l in open(PRISTINE+'crates/mutations.tsv',encoding='utf-8')]
env=dict(os.environ, CARGO_TARGET_DIR='/tmp/claude-1000/m3-unlink-code-r3-sonnet/target')
for n in map(int, sys.argv[1:]):
    r=rows[n-1]
    name,f,old,new,args,must=r
    old=old.replace('\\n','\n'); new=new.replace('\\n','\n')
    path=TREE+f
    orig=open(PRISTINE+f,encoding='utf-8').read()
    assert orig.count(old)==1,(n,'anchor')
    open(path,'w',encoding='utf-8').write(orig.replace(old,new))
    cmd=['cargo','test','--offline']+args.split()
    if '--' not in cmd: cmd.append('--')
    if must not in args: cmd.append(must)
    cmd+=['--test-threads','10']
    t=time.time()
    try:
        p=subprocess.run(cmd,cwd=TREE,env=env,capture_output=True,text=True,timeout=200)
        out=p.stdout+p.stderr; rc=p.returncode
    except subprocess.TimeoutExpired as e:
        out=(e.stdout or b'').decode() if isinstance(e.stdout,bytes) else (e.stdout or ''); rc='timeout'
    shutil.copyfile(PRISTINE+f,path)
    failed=[l for l in out.split('\n') if re.search(r'^test .* \.\.\. FAILED',l)]
    named=any(must in l for l in failed)
    compile_err='error[' in out or 'could not compile' in out
    print(f'ROW {n} rc={rc} named_test_red={named} other_failed={len(failed)-(1 if named else 0)} compile_err={compile_err} secs={time.time()-t:.0f} | {name[:70]}',flush=True)
    pan=[l for l in out.split('\n') if 'panicked at' in l]
    for l in pan[:2]: print('   ',l[:200])
    for i,l in enumerate(out.split('\n')):
        if 'panicked at' in l: print('    msg:', out.split('\n')[i+1][:260]); break
    if not named:
        print('   tail:', out.strip().split('\n')[-3:],flush=True)
