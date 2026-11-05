#!/usr/bin/env python3
"""喂钩子：probe.py <cases.py> --work 草稿目录 [--roots now,base] [--only 前缀] [--cwd 目录]
cases.py 里 CASES = [(标签, 钩子文件, 身份, 命令, 额外环境或省略)]；身份 "main" 是主 agent，别的是子 agent 的 agent_type。
每条样本一个进程组，超时杀整组。输出每个根的退出码、检出条数、第一行拒绝说明。"""
import concurrent.futures, json, os, signal, subprocess, sys, tempfile, time
argv = sys.argv[1:]
cases_file = argv[0]
def opt(name, default=None):
    return argv[argv.index(name) + 1] if name in argv else default
work = opt("--work"); roots = opt("--roots", "now,base").split(","); only = opt("--only", "")
cwd = opt("--cwd", os.path.join(work, "cwd")); os.makedirs(cwd, exist_ok=True)
ns = {"__file__": os.path.abspath(cases_file)}; exec(open(cases_file, encoding="utf-8").read(), ns)
cases = [c for c in ns["CASES"] if c[0].startswith(only)]
runs = os.path.join(work, "probe-tmp"); os.makedirs(runs, exist_ok=True)
def feed(root, hook, command, agent, env_extra, det, timeout=120):
    inp = {"tool_name": "Bash", "cwd": cwd, "session_id": "d137-r2-sonnet", "tool_input": {"command": command}}
    if agent != "main":
        inp["agent_type"] = agent; inp["agent_id"] = "probe"
    env = {k: v for k, v in os.environ.items() if not k.startswith(("SINGLEFS_", "HEAVY_TEST", "LIB_", "WORKSPACE_CARGO", "AGENT_HOOK", "CLAUDE_", "BASH_COMMAND"))}
    env.update({"CLAUDE_PROJECT_DIR": os.path.join(work, root), "AGENT_HOOK_DETECTIONS": det})
    env.update(env_extra or {})
    t0 = time.time()
    p = subprocess.Popen(["bash", os.path.join(work, root, ".claude", "hooks", hook)], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                         stderr=subprocess.PIPE, cwd=cwd, env=env, text=True, start_new_session=True)
    try:
        out, err = p.communicate(json.dumps(inp), timeout=timeout); rc = p.returncode
    except subprocess.TimeoutExpired:
        os.killpg(p.pid, signal.SIGKILL); out, err = p.communicate(); rc = 124
    try: os.killpg(p.pid, signal.SIGKILL)
    except (ProcessLookupError, PermissionError): pass
    notes = len(open(det, encoding="utf-8").read().splitlines()) if os.path.exists(det) else 0
    return rc, err, notes, time.time() - t0
def one(item):
    i, (label, hook, agent, command, *rest) = item
    env_extra = rest[0] if rest else {}
    res = {}
    for root in roots:
        det = os.path.join(runs, f"det-{os.getpid()}-{i}-{root}.jsonl")
        res[root] = feed(root, hook, command, agent, env_extra, det)
        if os.path.exists(det): os.remove(det)
    return i, label, hook, agent, command, res
with concurrent.futures.ThreadPoolExecutor(max_workers=int(os.environ.get("PROBE_THREADS", "8"))) as pool:
    results = sorted(pool.map(one, enumerate(cases)))
for i, label, hook, agent, command, res in results:
    cols = " ".join(f"{r}:rc={v[0]}/n={v[2]}/{v[3]:.1f}s" for r, v in res.items())
    flag = ""
    if "now" in res and "base" in res and res["now"][0] != res["base"][0]:
        flag = " CHANGED"
    short = hook.split("-")[0][:5]
    print(f"{cols}{flag} | {short} {agent} | {label} | {command!r}")
    if res.get("now", (0,))[0] not in (0,) :
        marks = [l for l in res["now"][1].splitlines() if "✗" in l]
        print("    now> " + (marks[0] if marks else res["now"][1].strip().splitlines()[0])[:300])
print(f"cases={len(results)}")
