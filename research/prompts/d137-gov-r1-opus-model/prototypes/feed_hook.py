"""feed.py <hook 文件> <树> <agent_type|main> <命令文件> <检出记录>：照 PreToolUse 的输入喂一道钩子，打退出码与 stderr。样本放自己的进程组、超时杀整组。"""
import json, os, signal, subprocess, sys
hook, tree, agent_type, command_file, detections = sys.argv[1:6]
command = open(command_file, encoding="utf-8").read().rstrip("\n")
data = {"session_id": "probe-s1", "transcript_path": "/nonexistent/probe.jsonl", "cwd": tree,
        "hook_event_name": "PreToolUse", "tool_name": "Bash", "tool_input": {"command": command}}
if agent_type != "main":
    data.update(agent_id="probe-a1", agent_type=agent_type)
environment = dict(os.environ, CLAUDE_PROJECT_DIR=tree, AGENT_HOOK_DETECTIONS=detections)
process = subprocess.Popen(["bash", hook], cwd=tree, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                           env=environment, start_new_session=True)
try:
    out, err = process.communicate(json.dumps(data, ensure_ascii=False).encode(), timeout=90)
except subprocess.TimeoutExpired:
    os.killpg(process.pid, signal.SIGKILL); out, err = process.communicate()
    print("超时"); sys.exit(3)
finally:
    try:
        os.killpg(process.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
print(f"rc={process.returncode}")
print(err.decode(errors="replace").strip()[:600])
