import subprocess, sys, os, shutil, json
repo = os.path.expanduser('~/.local/share/singlefs/work/m3-batch10x-code-r1-sonnet/repo')
target = os.path.expanduser('~/.local/share/singlefs/work/m3-batch10x-code-r1-sonnet/target')
src = repo + '/crates/singlefs-harness/src/gpu_memory_ledger.rs'
orig = open(src).read()
mutants = json.load(open(sys.argv[1]))
only = sys.argv[2:] 
env = dict(os.environ, CARGO_TARGET_DIR=target)
for name, old, new in mutants:
    if only and name not in only: continue
    assert orig.count(old) == 1, (name, orig.count(old))
    open(src, 'w').write(orig.replace(old, new))
    try:
        r = subprocess.run(['nice','-n','19','bash','/home/fy5090/code/singlefs/scripts/capped.sh','10','bash','/home/fy5090/code/singlefs/scripts/run-with-memory-cap.sh','4G','cargo','test','-p','singlefs-harness','--lib','--offline','gpu_memory_ledger'],
                           cwd=repo, env=env, capture_output=True, text=True, timeout=200)
        out = r.stdout + r.stderr
        res = [l for l in out.splitlines() if l.startswith('test result') or 'error[' in l or l.startswith('error')]
        failed = [l for l in out.splitlines() if l.startswith('test ') and l.endswith('FAILED')]
        print(f"{name} | exit={r.returncode} | {res[:2]} | failed_tests={len(failed)}", flush=True)
    finally:
        open(src, 'w').write(orig)
