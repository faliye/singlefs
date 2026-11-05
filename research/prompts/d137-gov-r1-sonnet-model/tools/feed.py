#!/usr/bin/env python3
"""把一条命令按 hook 的 JSON 输入喂给草稿目录里拷来的钩子；每条样本放进自己的进程组，超时杀整组。"""
import json, os, signal, subprocess, sys, time
LEG = os.environ.get("D137_LEG_DIR") or os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TREE = os.path.join(LEG, "tree")
SANDBOX = os.path.join(LEG, "sandbox")
DETECT = os.path.join(LEG, "detections.jsonl")

def feed(hook, command, agent=None, env=None, cwd=None, extra=None, timeout=90, background=False, tool="Bash", tool_input=None):
    hook_path = os.path.join(TREE, ".claude", "hooks", hook)
    inp = {"tool_name": tool, "cwd": cwd or SANDBOX, "session_id": "d137-gov-r1-sonnet-probe"}
    inp["tool_input"] = tool_input if tool_input is not None else {"command": command}
    if background and tool == "Bash":
        inp["tool_input"]["run_in_background"] = True
    if agent:
        inp["agent_type"] = agent
        inp["agent_id"] = "probe"
    if extra:
        inp.update(extra)
    e = {k: v for k, v in os.environ.items() if not k.startswith(("SINGLEFS_", "HEAVY_TEST", "LIB_", "WORKSPACE_CARGO", "AGENT_HOOK", "CLAUDE_"))}
    e.update({"CLAUDE_PROJECT_DIR": TREE, "AGENT_HOOK_DETECTIONS": DETECT, "SINGLEFS_WORK_ROOT": os.path.join(LEG, "workroot"),
              "HOME": os.environ["HOME"]})
    if env:
        e.update(env)
    before = sum(1 for _ in open(DETECT, encoding="utf-8")) if os.path.exists(DETECT) else 0
    p = subprocess.Popen(["setsid", "--wait", "bash", hook_path], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                         cwd=cwd or SANDBOX, env=e, start_new_session=False, text=True, preexec_fn=os.setpgrp)
    try:
        out, err = p.communicate(json.dumps(inp), timeout=timeout)
        rc = p.returncode
    except subprocess.TimeoutExpired:
        try: os.killpg(p.pid, signal.SIGKILL)
        except ProcessLookupError: pass
        out, err = p.communicate(); rc = 124
    try: os.killpg(p.pid, signal.SIGKILL)
    except (ProcessLookupError, PermissionError): pass
    notes = []
    if os.path.exists(DETECT):
        lines = open(DETECT, encoding="utf-8").read().splitlines()[before:]
        for l in lines:
            try: notes.append(json.loads(l))
            except Exception: notes.append({"raw": l})
    return rc, err, notes

if __name__ == "__main__":
    hook, agent, command = sys.argv[1], (sys.argv[2] or None), sys.argv[3]
    t = time.time()
    rc, err, notes = feed(hook, command, agent)
    print("rc=%s %.1fs" % (rc, time.time() - t)); print(err[:1500]); print(json.dumps(notes, ensure_ascii=False)[:1500])
