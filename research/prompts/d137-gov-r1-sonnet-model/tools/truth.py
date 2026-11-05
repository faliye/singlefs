#!/usr/bin/env python3
"""真 bash 量：命令在沙箱里真跑一遍（cargo 是只记调用的垫片），看垫片收到了什么。每条样本放进自己的进程组，超时杀整组。"""
import os, signal, subprocess, time
LEG = os.environ.get("D137_LEG_DIR") or os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SANDBOX = os.path.join(LEG, "sandbox")
LOG = os.path.join(SANDBOX, "shim.log")
def truth(command, timeout=10, interpreter=("bash", "-c")):
    if os.path.exists(LOG): os.remove(LOG)
    env = {"PATH": os.path.join(SANDBOX, "bin") + ":/usr/local/bin:/usr/bin:/bin", "HOME": os.environ["HOME"], "SHIM_LOG": LOG, "LANG": "C.UTF-8"}
    p = subprocess.Popen(["setsid", "--wait", *interpreter, command], cwd=SANDBOX, env=env, stdin=subprocess.DEVNULL,
                         stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, preexec_fn=os.setpgrp)
    try:
        out, _ = p.communicate(timeout=timeout); rc = p.returncode
    except subprocess.TimeoutExpired:
        try: os.killpg(p.pid, signal.SIGKILL)
        except ProcessLookupError: pass
        out, _ = p.communicate(); rc = 124
    time.sleep(0.25)   # 后台起的 &
    try: os.killpg(p.pid, signal.SIGKILL)
    except (ProcessLookupError, PermissionError): pass
    calls = open(LOG, encoding="utf-8").read().splitlines() if os.path.exists(LOG) else []
    return rc, calls, out
