#!/usr/bin/env python3
import json, os, subprocess, sys
W = "/tmp/claude-1000/tooling-detached-run-r3-sonnet/work"
HEREDOC = ("cat > dr-heredoc.sh <<'EOF2'\n#!/usr/bin/env bash\n# detached-run-script-signature: 探针\necho sample\nEOF2\n"
           "bash dr-heredoc.sh start --label p -- cargo test --all")
CASES = [
 ("h4 install 改名再起", "clerk", f"install -m 755 research/scripts/detached-run.sh {W}/tmp/dr4.sh && bash {W}/tmp/dr4.sh start --label m -- sleep 1", W),
 ("h4 mv 改名再起", "clerk", f"mv research/scripts/detached-run.sh {W}/tmp/dr5.sh && bash {W}/tmp/dr5.sh start --label m -- sleep 1", W),
 ("h4 cp 改名再起（旧）", "clerk", f"cp research/scripts/detached-run.sh {W}/tmp/dr6.sh && bash {W}/tmp/dr6.sh start --label m -- sleep 1", W),
 ("h7 cd 之后相对路径", "clerk", f"cd {W}/probe && bash ./dr-renamed.sh start --label m -- sleep 1", W),
 ("h5 heredoc 写出拷贝再起全量(主agent)", None, HEREDOC, W + "/tmp"),
 ("对照 子agent list 放行", "clerk", "bash research/scripts/detached-run.sh list", W),
]
extra = dict(a.split("=", 1) for a in sys.argv[1:])
for name, agent, cmd, cwd in CASES:
    payload = {"tool_name": "Bash", "tool_input": {"command": cmd, "run_in_background": False}, "session_id": "u1-probe", "cwd": cwd}
    if agent: payload["agent_type"] = agent
    env = {k: v for k, v in os.environ.items() if k != "SINGLEFS_HEAVY_TESTS"}
    env.update(extra, AGENT_HOOK_DETECTIONS=os.path.join(W, "detections.jsonl"), CLAUDE_PROJECT_DIR=W)
    got = [subprocess.run(["bash", os.path.join(W, ".claude/hooks", h)], input=json.dumps(payload), capture_output=True, text=True, cwd=cwd, env=env, timeout=120).returncode
           for h in ("bash-command-detector.sh", "heavy-test-guard.sh")]
    print(f"{name}\tdetector={got[0]}\theavy={got[1]}")
