import json, subprocess, sys
hook, cwd, agent = sys.argv[1], sys.argv[2], sys.argv[3]
for command in sys.argv[4:]:
    data = {"tool_name": "Bash", "tool_input": {"command": command}, "cwd": cwd, "hook_event_name": "PreToolUse"}
    if agent != "main":
        data.update({"agent_type": agent, "agent_id": "aprobe0000000000"})
    done = subprocess.run(["bash", hook], input=json.dumps(data), capture_output=True, text=True, timeout=60)
    lines = done.stderr.strip().split("\n")
    first = next((line for line in lines if line.lstrip().startswith(("✗", "→ 照抄"))), lines[0] if lines else "")
    print(f"{agent} 退 {done.returncode}：{command[:90]!r}　{first[:400]}")
