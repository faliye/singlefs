#!/usr/bin/env bash
# feed.sh <钩子:guard|det> <agent:main|名字> <命令> : 用真的钩子入口（bash <钩子>，标准输入给 hook 的 JSON）喂一条命令，打出退出码与 stderr 前几行。
# 环境：SW_ROOT（放 tree、tree-orig、scratch 的目录）、TREE（tree 或 tree-orig）、CWD（hook 输入里的 cwd，默认 $SW_ROOT/scratch）
hook="$1"; agent="$2"; cmd="$3"
case "$hook" in guard) f=heavy-test-guard.sh;; det) f=bash-command-detector.sh;; esac
python3 -I - "$agent" "$cmd" "$SW_ROOT" "$f" "${TREE:-tree}" "${CWD:-$SW_ROOT/scratch}" <<'PY'
import json, subprocess, sys, os
agent, cmd, root, f, tree, cwd = sys.argv[1:7]
inp = {"tool_name": "Bash", "cwd": cwd, "tool_input": {"command": cmd}}
if agent != "main":
    inp["agent_type"] = agent
env = dict(os.environ)
env["AGENT_HOOK_DETECTIONS"] = root + "/scratch/detections.jsonl"
env.pop("SINGLEFS_HEAVY_TESTS", None)
p = subprocess.run(["bash", f"{root}/{tree}/.claude/hooks/{f}"], input=json.dumps(inp), capture_output=True, text=True, env=env, cwd=root + "/scratch")
print("rc=%d" % p.returncode)
if p.stderr.strip():
    print(p.stderr.strip()[:1500])
PY
