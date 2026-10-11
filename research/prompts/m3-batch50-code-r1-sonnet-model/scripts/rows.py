#!/usr/bin/env python3
# 用法: rows.py <最多处理几行> [名字子串]
# 逐行重放本轮变异表新增 / 整行替换的行（mut-unstaged.diff 里的 + 行），只跑非 checker 档的：把 原文→替换文 套在 probe-tree 里，
# 跑那一行点名的 cargo 参数，记录点名的测试是否红。结果追加到 rows-results.tsv；已处理过的行（按名字）跳过。
import os, re, shutil, signal, subprocess, sys, time
root = os.path.expanduser("~/.local/share/singlefs/work/m3-batch50-code-r1-sonnet")
tree = f"{root}/probe-tree"
limit = int(sys.argv[1]); only = sys.argv[2] if len(sys.argv) > 2 else None
rows = []
for line in open(f"{root}/mut-unstaged.diff"):
    if line.startswith("+") and not line.startswith("+++"):
        f = line[1:].rstrip("\n").split("\t")
        if len(f) == 6:
            rows.append(f)
done = set()
res = f"{root}/rows-results.tsv"
if os.path.exists(res):
    done = {l.split("\t")[0] for l in open(res)}
bak = None; target = None
def restore():
    global bak, target
    if bak and os.path.exists(bak):
        shutil.move(bak, target); os.utime(target)
    bak = None
def on_term(s, fr):
    restore(); sys.exit(143)
signal.signal(signal.SIGTERM, on_term); signal.signal(signal.SIGINT, on_term)
n = 0
for index, f in enumerate(rows):
    name, rel, orig, repl, args, test = f
    if name in done: continue
    if only and only not in name: continue
    if "checker-tier" in args: continue
    if n >= limit: break
    n += 1
    target = f"{tree}/{rel}"
    text = open(target).read()
    o = orig.replace("\\n", "\n"); r = repl.replace("\\n", "\n")
    if text.count(o) != 1:
        open(res, "a").write(f"{name}\t{rel}\tANCHOR-{text.count(o)}\t\t\t0\n"); continue
    bak = target + ".rowbak"; shutil.copy(target, bak)
    open(target, "w").write(text.replace(o, r)); os.utime(target)
    t0 = time.time()
    cmd = ["nice", "-n", "19", "bash", "/home/fy5090/code/singlefs/scripts/hosts/run-with-memory-cap.sh", "10G", "bash",
           "/home/fy5090/code/singlefs/scripts/hosts/capped.sh", "8", "cargo", "test", "--offline",
           "--target-dir", f"{root}/target-h"] + args.split()
    try:
        p = subprocess.run(cmd, cwd=tree, capture_output=True, text=True, timeout=200)
        out = p.stdout + p.stderr; code = p.returncode
    except subprocess.TimeoutExpired:
        out = ""; code = 124
    finally:
        restore()
    failed = re.findall(r"^test (\S+) \.\.\. FAILED", out, re.M)
    comp = "could not compile" in out
    named = any(test in x for x in failed)
    status = "COMPILE-ERR" if comp else ("RED-NAMED" if named else ("RED-OTHER" if failed else ("TIMEOUT" if code == 124 else "GREEN")))
    open(res, "a").write(f"{name}\t{rel}\t{status}\t{test}\t{','.join(failed[:3])}\t{int(time.time()-t0)}\n")
    open(f"{root}/rows-last.log", "w").write(out[-3000:])
print("done", n)
