import os, subprocess, sys, importlib.util
spec = importlib.util.spec_from_file_location("m", "/home/fy5090/code/singlefs/research/prompts/m3-batch10z-e-code-r1-opus-model/mutate_10ze.py")
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
copy, target, name = sys.argv[1], sys.argv[2], sys.argv[3]
path = os.path.join(copy, m.LEDGER)
old, new = m.MUTANTS[name]
original = open(path).read(); assert original.count(old) == 1
open(path, "w").write(original.replace(old, new, 1))
try:
    env = dict(os.environ, CARGO_TARGET_DIR=target, ATTACK_10ZE_SCRIPTS="5000")
    p = subprocess.run(["nice","-n","19","cargo","test","--offline","-p","singlefs-harness","--test","attack_10ze_rounds","--","k1_5_monotonic","--nocapture","--test-threads","1"], cwd=copy, env=env, capture_output=True, text=True)
    for l in (p.stdout+p.stderr).splitlines():
        if l.startswith("ATTACK") or "stopped_with" in l or "panic_or" in l or "judged_twice" in l or "short:" in l or "assertion" in l or l.startswith("  left") or l.startswith("  right"):
            print(l[:900])
finally:
    open(path, "w").write(original)
