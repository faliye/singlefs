#!/usr/bin/env python3
"""G4：把命令文本按 PreToolUse 的 JSON 喂给真钩子 .claude/hooks/heavy-test-guard.sh（子 agent 与主 agent 两种身份），打退出码；
检出记录经 AGENT_HOOK_DETECTIONS 改写进暂存目录，不碰真工作根的记录。同一条命令里造的假拷贝放在暂存目录里。
用法：python3 -I hook_feed.py <仓根> <暂存目录>"""
import json, os, subprocess, sys

repo, scratch = os.path.abspath(sys.argv[1]), os.path.abspath(sys.argv[2])
R = "scripts/experiment-apparatus/replay.sh"
body = "#!/usr/bin/env bash\ncargo test --all\n"
for path in (f"{scratch}/fake/{R}", f"{scratch}/fake2/replay.sh"):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as handle:
        handle.write(body)
    os.chmod(path, 0o755)
WRITER = "implementation-writer"
CASES = [
    ("h01", WRITER, f"bash {R} --list-items", 0),
    ("h02", WRITER, f"bash {R} E142", 2),
    ("h03", WRITER, f"bash {R}", 2),
    ("h04", WRITER, f"bash {scratch}/fake/{R} --list-items", 0),
    ("h05", WRITER, f"bash {scratch}/fake2/replay.sh --list-items", 2),
    ("h06", WRITER, f"cp {R} {scratch}/r.sh; bash {scratch}/r.sh E142", 2),
    ("h07", WRITER, f"echo E142 | xargs bash {R}", 2),
    ("h08", WRITER, f"bash scripts/hosts/capped.sh 31 bash {R} --list-items", 0),
    ("h09", WRITER, f"bash -c 'bash {R} --list-items'", 0),
    ("h10", WRITER, f"bash {R} --list-items; bash {R} E142", 2),
    ("h11", WRITER, "bash scripts/experiments/replay.sh --list-items", 0),
    ("h12", WRITER, f"REPLAY_BREAK=x bash {R} --list-items", 0),
    ("h13", WRITER, f"bash {R} --force --list-items", 0),
    ("h14", WRITER, f"bash {R} --list-items E142", 2),
    ("h15", WRITER, f"bash scripts/hosts/run-with-memory-cap.sh 8G bash {R} --list-items", 0),
    ("h16", None, f"bash {R} --list-items", 0),
    ("h17", None, f"bash {R} E142", 2),
    ("h18", None, f"SINGLEFS_HEAVY_TESTS=commit bash {R} E142", 0),
    ("h19", None, f"bash {R}", 2),
]
bad = 0
for number, agent, command, want in CASES:
    hook_input = {"tool_name": "Bash", "cwd": repo, "tool_input": {"command": command}}
    if agent:
        hook_input["agent_type"] = agent
    env = dict(os.environ, CLAUDE_PROJECT_DIR=repo, AGENT_HOOK_DETECTIONS=os.path.join(scratch, "detections.jsonl"))
    done = subprocess.run(["bash", os.path.join(repo, ".claude/hooks/heavy-test-guard.sh")], input=json.dumps(hook_input), capture_output=True, text=True, env=env, cwd=repo)
    first = (done.stderr.strip().replace(scratch, "<scratch>").splitlines() or [""])[0][:120]
    verdict = "same" if done.returncode == want else "DIFF"
    bad += verdict == "DIFF"
    print(f"{number}\texit={done.returncode}\twant={want}\t{verdict}\tagent={agent or 'main'}\t{command.replace(scratch, '<scratch>')}\t{first.replace(scratch, '<scratch>')}")
print(f"cases={len(CASES)} diff={bad}")
