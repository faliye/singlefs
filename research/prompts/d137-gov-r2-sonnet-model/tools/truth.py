#!/usr/bin/env python3
"""真起一遍：把 cases 里每条命令在真 bash 里跑，PATH 里放一个只记日志的假 cargo，看假 cargo 有没有被起。
每条样本 setsid --wait 起、自己的进程组、超时与结束对整组发 KILL；cwd 是样本目录。用法：truth.py <cases.py> <样本目录> [--only 前缀]
命令里有会无限递归的不放进 cases（这份语料里没有）。"""
import json, os, signal, subprocess, sys, tempfile, time
cases_file, sandbox = sys.argv[1], os.path.abspath(sys.argv[2])
only = sys.argv[sys.argv.index("--only") + 1] if "--only" in sys.argv else ""
ns = {"__file__": os.path.abspath(cases_file)}; exec(open(cases_file, encoding="utf-8").read(), ns)
raw = ns.get("RAW") or [(c[0], c[3]) for c in ns["CASES"]]
shim = tempfile.mkdtemp(prefix="truth-shim-", dir=os.path.dirname(sandbox))
log = os.path.join(shim, "calls.log")
with open(os.path.join(shim, "cargo"), "x") as handle:
    handle.write('#!/bin/sh\necho "cargo $*" >> "' + log + '"\nexit 0\n')
os.chmod(os.path.join(shim, "cargo"), 0o755)
for label, command in raw:
    if not label.startswith(only):
        continue
    if os.path.exists(log): os.remove(log)
    env = {"PATH": shim + ":/usr/local/bin:/usr/bin:/bin", "HOME": os.environ["HOME"], "LANG": "C.UTF-8"}
    p = subprocess.Popen(["setsid", "--wait", "bash", "-c", command], cwd=sandbox, env=env, stdin=subprocess.DEVNULL,
                         stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, start_new_session=True)
    try:
        p.communicate(timeout=10)
    except subprocess.TimeoutExpired:
        pass
    time.sleep(0.15)
    try: os.killpg(p.pid, signal.SIGKILL)
    except (ProcessLookupError, PermissionError): pass
    calls = open(log, encoding="utf-8").read().splitlines() if os.path.exists(log) else []
    print(f"truth cargo_called={'yes' if calls else 'no '} | {label} | {command!r}")
import shutil; shutil.rmtree(shim)
