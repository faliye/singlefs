#!/usr/bin/env python3
"""把命令写成钩子输入喂给某棵副本树里的两道钩子（不执行命令），打每道的退出码与 stderr 第一行。
用法：feed.py <树根> <cases.json> [名=值 …]；cases 每行 [名, agent_type 或 null, 命令, (可选)cwd 相对树根]。
名=值 设进钩子进程自己的环境（模拟 Claude Code 进程带着它）；命令里写在前面的 名=值 只是命令文本。
（第二轮攻方 feed.py 的同一写法，探针会话名改成 opusr3）"""
import json, os, subprocess, sys
TREE = os.path.abspath(sys.argv[1])
HOOKS = ["bash-command-detector.sh", "heavy-test-guard.sh"]
def run(agent_type, command, extra_env, cwd):
    payload = {"tool_name": "Bash", "tool_input": {"command": command, "run_in_background": False},
               "session_id": "opusr3-probe", "cwd": cwd, "transcript_path": "/dev/null"}
    if agent_type:
        payload["agent_type"] = agent_type
    env = dict(os.environ, AGENT_HOOK_DETECTIONS=os.path.join(os.environ.get("OPUSR3_SCRATCH", "/tmp"), "detections.jsonl"),
               CLAUDE_PROJECT_DIR=TREE, **extra_env)
    for k in ("SINGLEFS_HEAVY_TESTS", "BASH_COMMAND_DETECTOR_DETACHED_RUN_BY_NAME", "LIB_HEAVY_TESTS_BREAK", "LIB_SHELL_WORDS_BREAK",
              "BASH_COMMAND_DETECTOR_DETACHED_RUN_COPY_BY_CP_LN", "BASH_COMMAND_DETECTOR_DETACHED_RUN_START_DIRECTORY",
              "HEAVY_TEST_GUARD_IGNORE_HEREDOC_DETACHED_RUN"):
        if k not in extra_env:
            env.pop(k, None)
    out = []
    for hook in HOOKS:
        p = subprocess.run(["bash", os.path.join(TREE, ".claude/hooks", hook)], input=json.dumps(payload),
                           capture_output=True, text=True, cwd=cwd, env=env, timeout=120)
        lines = [l.strip() for l in p.stderr.splitlines() if l.strip()]
        out.append((hook.split("-")[0], p.returncode, lines[0][:160] if lines else ""))
    return out
cases = json.load(open(sys.argv[2]))
extra = dict(arg.split("=", 1) for arg in sys.argv[3:])
for case in cases:
    name, agent, command = case[0], case[1], case[2]
    cwd = os.path.join(TREE, case[3]) if len(case) > 3 and case[3] else TREE
    result = run(agent, command, extra, cwd)
    print(f"{name}\tagent={agent or 'main'}\tbash={result[0][1]}\theavy={result[1][1]}")
    for hook, rc, first in result:
        if rc or first:
            print(f"    {hook} rc={rc}: {first}")
