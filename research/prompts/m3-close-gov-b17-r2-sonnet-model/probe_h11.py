#!/usr/bin/env python3
"""H5：正推腿 hook_feed 第一次复跑「diff= 计数多 1」的一个候选原因——h11 那条命令写的是改名之前的旧路径 scripts/experiments/replay.sh，
它的结局取决于当前目录里那个旧路径在不在。造两个当前目录（一个有旧路径的真 replay.sh 拷贝，一个没有），各喂真钩子一次，打退出码。
用法：python3 -I probe_h11.py <仓根> <暂存目录>（暂存目录要不存在或空；只写这里）"""
import json, os, shutil, subprocess, sys

repo, scratch = os.path.abspath(sys.argv[1]), os.path.abspath(sys.argv[2])
os.makedirs(scratch, exist_ok=True)
real = os.path.join(repo, "scripts/experiment-apparatus/replay.sh")
with_old = os.path.join(scratch, "with-old-path")
without_old = os.path.join(scratch, "without-old-path")
os.makedirs(os.path.join(with_old, "scripts/experiments"), exist_ok=True)
os.makedirs(without_old, exist_ok=True)
shutil.copyfile(real, os.path.join(with_old, "scripts/experiments/replay.sh"))
os.chmod(os.path.join(with_old, "scripts/experiments/replay.sh"), 0o755)
command = "bash scripts/experiments/replay.sh --list-items"
for label, cwd in (("old-path-present", with_old), ("old-path-absent", without_old)):
    hook_input = {"tool_name": "Bash", "cwd": cwd, "agent_type": "implementation-writer", "tool_input": {"command": command}}
    env = dict(os.environ, CLAUDE_PROJECT_DIR=repo, AGENT_HOOK_DETECTIONS=os.path.join(scratch, "detections.jsonl"))
    done = subprocess.run(["bash", os.path.join(repo, ".claude/hooks/heavy-test-guard.sh")], input=json.dumps(hook_input), capture_output=True, text=True, env=env, cwd=repo)
    first = (done.stderr.strip().replace(scratch, "<scratch>").replace(repo, "<repo>").splitlines() or [""])[0][:150]
    print(f"{label}\texit={done.returncode}\t{first}")
