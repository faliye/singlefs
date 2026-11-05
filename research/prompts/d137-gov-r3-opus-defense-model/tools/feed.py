#!/usr/bin/env python3
"""喂钩子拷贝：feed.py <钩子树> <样本目录> <工作根> <检出记录> <cases.json>
只把命令文本当 JSON 喂给 heavy-test-guard.sh 与 bash-command-detector.sh，不执行被判的命令。
每份钩子放进自己的进程组起（start_new_session），超时对整组发 SIGKILL。
输出一行一条：id、身份、两份钩子的退出码、各自 stderr 里第一行带「✗」的原文。"""
import json, os, signal, subprocess, sys

tree, sample_dir, work_root, detections, cases_path = sys.argv[1:6]
cases = json.load(open(cases_path))
env = {k: v for k, v in os.environ.items() if not k.startswith(("SINGLEFS_", "CLAUDE_PROJECT_DIR", "CAPPED"))}
env["SINGLEFS_WORK_ROOT"] = work_root
env["AGENT_HOOK_DETECTIONS"] = detections


def run_hook(hook, payload, cwd):
    process = subprocess.Popen(["bash", os.path.join(tree, ".claude", "hooks", hook)], stdin=subprocess.PIPE,
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, env=env, cwd=cwd,
                               start_new_session=True)
    try:
        out, err = process.communicate(json.dumps(payload), timeout=150)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        process.communicate()
        return "timeout", ""
    first = next((line.strip() for line in err.splitlines() if "✗" in line), "")
    return process.returncode, first


for case in cases:
    cwd = os.path.join(sample_dir, case.get("cwd", "."))
    command = case["command"].replace("{S}", sample_dir)   # {S} 换成样本目录的绝对路径
    payload = {"hook_event_name": "PreToolUse", "tool_name": "Bash", "session_id": "s-d137-gov-r3-defense",
               "cwd": cwd, "tool_input": {"command": command, "run_in_background": False}}
    if case["role"] != "main":
        payload["agent_type"] = case["role"]
    results = []
    for hook in ("heavy-test-guard.sh", "bash-command-detector.sh"):
        rc, first = run_hook(hook, payload, cwd)
        results.append((hook, rc, first))
    print(f"name=feed id={case['id']} role={case['role']} htg_rc={results[0][1]} bcd_rc={results[1][1]}")
    print(f"  command: {json.dumps(case['command'], ensure_ascii=False)}")   # 原样打 {S}，不打样本目录
    for hook, rc, first in results:
        if first:
            print(f"  {hook}: {first[:600]}")
    sys.stdout.flush()
