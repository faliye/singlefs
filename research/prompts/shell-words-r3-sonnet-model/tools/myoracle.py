"""myoracle.py <cases.json> <out.json> [并行数] : 真 bash 当裁判，每条样本放进自己的进程组（start_new_session = setsid(2)），结束与超时都杀整组。
每条 {cmd, [env], ...}：沙盒 = 模板拷贝，PATH 前面放假 cargo，假 detached-run.sh 在沙盒 scripts/hosts/。
交回 [{heavy, detached, rc, err1, leftover, timeout}]，与输入同序；同 (cmd, env) 只跑一遍。
heavy = 日志里有 cargo test|t 带 --all 或 --workspace；detached = 日志里有 detached-run start。leftover = bash 退出后 0.8 秒进程组里还有活的（已杀）。"""
import sys, os, json, subprocess, tempfile, shutil, shlex, re, time, signal, resource
from concurrent.futures import ThreadPoolExecutor
HERE = os.path.dirname(os.path.abspath(__file__)); ROOT = os.path.dirname(HERE)
TEMPLATE, BIN = ROOT + "/template", ROOT + "/oracle/bin"
cases = json.load(open(sys.argv[1])); outp = sys.argv[2]; workers = int(sys.argv[3]) if len(sys.argv) > 3 else 16
base = tempfile.mkdtemp(prefix="myoracle-", dir=ROOT + "/tmp")
def lim():
    resource.setrlimit(resource.RLIMIT_AS, (1 << 30, 1 << 30))
def group_alive(pg):
    try: os.killpg(pg, 0); return True
    except (ProcessLookupError, PermissionError): return False
def kill_group(pg):
    try: os.killpg(pg, signal.SIGKILL)
    except (ProcessLookupError, PermissionError): pass
def run(key):
    cmd, envj, sub = key; extra = json.loads(envj)
    d = tempfile.mkdtemp(dir=base); sb = d + "/scratch"
    shutil.copytree(TEMPLATE, sb, symlinks=True); os.makedirs(d + "/tmp")
    log = d + "/.oracle-log"; open(log, "w").close()
    env = {"PATH": BIN + ":/usr/bin:/bin", "HOME": d, "LC_ALL": "C.UTF-8", "TIMEFORMAT": "", "ORACLE_LOG": log}
    env.update(extra)
    to = False; leftover = False
    po = open(d + "/out", "w"); pe = open(d + "/err", "w")
    p = subprocess.Popen(["bash", "-c", cmd.replace("/tmp/", d + "/tmp/")], cwd=(sb + "/sub" if sub else sb), env=env, stdin=subprocess.DEVNULL, stdout=po, stderr=pe, start_new_session=True, preexec_fn=lim)
    pg = p.pid
    try: rc = p.wait(timeout=20)
    except subprocess.TimeoutExpired:
        to = True; kill_group(pg); rc = p.wait()
    if not to:
        t0 = time.time()
        while group_alive(pg) and time.time() - t0 < 0.8: time.sleep(0.04)
        leftover = group_alive(pg)
    if re.search(r"setsid|(?<![&>|])&(?![&>])|coproc|nohup|disown|systemd-run", cmd): time.sleep(0.6)   # 脱离进程组或放后台的：等它们把日志写完
    kill_group(pg)
    t0 = time.time()
    while group_alive(pg) and time.time() - t0 < 2: time.sleep(0.02)
    po.close(); pe.close()
    err = open(d + "/err", errors="replace").read().strip().splitlines()[:1]
    e1 = err[0].replace(d, "<sbx>").replace(BIN, "<bin>")[:110] if err else ""
    lines = open(log, errors="replace").read().splitlines()
    heavy = False
    for l in lines:
        if l.startswith("cargo"):
            try: t = shlex.split(l)
            except ValueError: continue
            if t[1:2] and t[1] in ("test", "t") and ("--all" in t or "--workspace" in t): heavy = True
    det = any(l.startswith("detached-run start") for l in lines)
    shutil.rmtree(d, ignore_errors=True)
    return dict(heavy=heavy, detached=det, rc=("timeout" if to else rc), err1=e1, leftover=leftover, timeout=to, log=lines[:6])
keys = []; seen = {}
for c in cases:
    k = (c["cmd"], json.dumps(c.get("env") or {}, sort_keys=True), str(c.get("cwd") or "").endswith("/sub"))
    if c.get("noexec"): k = None
    keys.append(k)
    if k is not None and k not in seen: seen[k] = None
uk = list(seen)
with ThreadPoolExecutor(workers) as ex:
    for k, r in zip(uk, ex.map(run, uk)): seen[k] = r
out = [dict(seen[k]) if k is not None else dict(noexec=True) for k in keys]
json.dump(out, open(outp, "w"), ensure_ascii=False)
shutil.rmtree(base, ignore_errors=True)
print("cases", len(cases), "run", len(uk), "leftover", sum(1 for r in out if r.get("leftover")), "timeout", sum(1 for r in out if r.get("timeout")))
