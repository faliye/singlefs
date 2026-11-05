"""selftests.py : 修补报告「四」里的 14 份 --selftest，各放进自己的进程组（setsid），ulimit -v 16G、nice 19、220 秒超时杀整组；交回退出码、✗ 行数、末行。
在主工作区仓根里跑（与报告同一口径；五份钩子与冻结副本逐字节相同）。"""
import subprocess, os, sys, signal, time, resource, json
from concurrent.futures import ThreadPoolExecutor
REPO = os.environ.get("REPO_ROOT", "/home/fy5090/code/singlefs"); OUT = os.path.dirname(os.path.dirname(os.path.abspath(__file__))) + "/out/st"
os.makedirs(OUT, exist_ok=True)
items = [("detector", ["bash", ".claude/hooks/bash-command-detector.sh", "--selftest"]), ("heavyguard", ["bash", ".claude/hooks/heavy-test-guard.sh", "--selftest"]),
         ("heavylib", ["python3", ".claude/hooks/lib_heavy_tests.py", "--selftest"]), ("cargolock", ["python3", ".claude/hooks/lib_workspace_cargo_lock.py", "--selftest"]),
         ("runscope", ["bash", ".claude/hooks/run-scope-guard.sh", "--selftest"]), ("twologs", ["bash", ".claude/hooks/two-logs-guard.sh", "--selftest"]),
         ("handback", ["bash", ".claude/hooks/handback-guard.sh", "--selftest"]), ("timer", ["bash", ".claude/hooks/timer-only-remind-guard.sh", "--selftest"]),
         ("hostres", ["bash", ".claude/hooks/host-resources-reminder.sh", "--selftest"]), ("sharedindex", ["bash", ".claude/hooks/shared-index-stale-reminder.sh", "--selftest"]),
         ("detachedrun", ["bash", "scripts/hosts/detached-run.sh", "--selftest"]), ("agentwatch", ["python3", "scripts/agents/agent-watch.py", "--selftest"]),
         ("lightguard", ["python3", "scripts/gate/light-selftest-guard.py", "--selftest"]), ("memorypeaks", ["python3", "scripts/hosts/memory_peaks_lib.py", "--selftest"])]
compact = "--compact" in sys.argv[1:]; only = set(a for a in sys.argv[1:] if not a.startswith("--"))
def lim():
    resource.setrlimit(resource.RLIMIT_AS, (16 << 30, 16 << 30)); os.nice(19)
def run(it):
    name, cmd = it
    f = open(f"{OUT}/{name}.txt", "w")
    p = subprocess.Popen(cmd, cwd=REPO, stdout=f, stderr=subprocess.STDOUT, stdin=subprocess.DEVNULL, start_new_session=True, preexec_fn=lim)
    try: rc = p.wait(timeout=int(os.environ.get("SELFTEST_TIMEOUT", "220")))
    except subprocess.TimeoutExpired:
        rc = "超时"; 
    try: os.killpg(p.pid, signal.SIGKILL)
    except ProcessLookupError: pass
    p.wait(); f.close()
    txt = open(f"{OUT}/{name}.txt", errors="replace").read()
    last = txt.rstrip("\n").splitlines()[-1] if txt.strip() else ""
    return name, rc, sum(1 for l in txt.splitlines() if "✗" in l), last[:150]
with ThreadPoolExecutor(14) as ex:
    for name, rc, bad, last in ex.map(run, [i for i in items if not only or i[0] in only]):
        print(f"{name} rc={rc} ✗行={bad}" + ("" if compact else f" 末行：{last}"))
