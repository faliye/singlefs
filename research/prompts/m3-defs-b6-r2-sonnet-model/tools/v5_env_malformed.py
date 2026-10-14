#!/usr/bin/env python3
"""X3：畸形的 env -S 串、引号不闭合、转义、嵌套，喂 heavy-test-guard.sh 入口，看有没有崩（Traceback）、退出码是 0 还是 2。用法：v5_env_malformed.py <仓根>"""
import json, os, subprocess, sys
root = os.path.abspath(sys.argv[1])
cwd = root + "/crates/singlefs-harness"
cases = ["env -S \"cargo 'test\"", "env -S 'cargo \"test'", "env -S '\\c'", "env -S 'cargo ${X} test'", "env -S '\\_\\_'", "env --split-string=",
         "env -S '-S -S -S cargo test -p singlefs-checker-tier'", "env -S 'env -S \"env -S \\\"cargo test -p singlefs-checker-tier\\\"\"'",
         "env -S '#comment cargo test -p singlefs-checker-tier'", "env -S \"\\x\"", "env -S '$(cargo test -p singlefs-checker-tier)'", "env -S ';cargo test -p singlefs-checker-tier'",
         "env -i -- -S", "env -S 'FOO=\"a b\" cargo build'", "env -u", "env -S 'cargo test' --", "env -S ' '", "env -S $'cargo\\ttest -p singlefs-harness'"]
for command in cases:
    payload = json.dumps({"tool_name": "Bash", "cwd": cwd, "agent_type": "implementation-writer",
                          "tool_input": {"command": "bash ../../scripts/hosts/run-with-memory-cap.sh 8G " + command}})
    env = dict(os.environ, CLAUDE_PROJECT_DIR=root, PYTHONDONTWRITEBYTECODE="1")
    done = subprocess.run(["bash", root + "/.claude/hooks/heavy-test-guard.sh"], input=payload, capture_output=True, text=True, env=env, timeout=60)
    print(done.returncode, "Traceback" if "Traceback" in done.stderr else "-", command)
