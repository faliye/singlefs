import json, os, shutil, subprocess, sys
# 用法：red-worlds.py <crate 目录> <行表> <工作目录> —— 逐行把「原文」换成「替换文」（原文须恰好命中一次），起 verus 只验新模块，报 errors 与落点
crate, table, work = sys.argv[1:4]
VERUS = os.path.expanduser("~/.local/share/singlefs/formal-tools/verus-0.2026.09.20/verus-x86-linux/verus")
FILE = "src/relocation_execution_proofs.rs"
for row in open(table, encoding="utf-8"):
    if not row.strip(): continue
    name, old, new = row.rstrip("\n").split("\t")
    old = old.replace("\\n", "\n"); new = new.replace("\\n", "\n")
    copy = os.path.join(work, "mutant")
    shutil.rmtree(copy, ignore_errors=True)
    shutil.copytree(crate, copy)
    path = os.path.join(copy, FILE)
    text = open(path, encoding="utf-8").read()
    hits = text.count(old)
    if hits != 1:
        print(f"{name}\t原文命中{hits}次，跳过"); continue
    open(path, "w", encoding="utf-8").write(text.replace(old, new))
    ran = subprocess.run([VERUS, "--no-cheating", "--crate-type=lib", "--edition=2021", "--crate-name", "singlefs_replay_prefix",
        "--num-threads", "31", "--output-json", "--rlimit", "10", "--verify-only-module", "relocation_execution_proofs", "src/lib.rs"],
        cwd=copy, capture_output=True, text=True)
    out = ran.stdout.strip()
    errors = verified = None
    if out.startswith("{"):
        r = json.loads(out).get("verification-results", {}); errors = r.get("errors"); verified = r.get("verified")
    lines = ran.stderr.splitlines()
    landings = []
    pending = False
    for i, line in enumerate(lines):
        if line.startswith("error") and "aborting" not in line and "Resource limit" not in line:
            pending = True
        elif pending and line.strip().startswith("-->"):
            landings.append(line.split("-->")[1].strip()); pending = False
    print(f"{name}\trc={ran.returncode}\terrors={errors}\tverified={verified}\t落点={';'.join(landings)}")
shutil.rmtree(os.path.join(work, "mutant"), ignore_errors=True)
