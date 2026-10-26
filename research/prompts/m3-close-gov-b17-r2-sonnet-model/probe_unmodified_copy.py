#!/usr/bin/env python3
"""H1（G4 的改法是不是只有一种）：判决第二节 G4 写「只认本仓那一份（与 --selftest 那一条一样比 sha256 或比真路径）」，修补 2 取了比真路径。
没改过一个字的 replay.sh 拷贝（路径尾部与本仓位置相同）带 --list-items，现在钩子怎么判。再喂本仓那一份、正文换成 cargo test --all 的拷贝作对照。
用法：python3 -I probe_unmodified_copy.py <仓根> <暂存目录>"""
import json, os, shutil, subprocess, sys

repo, scratch = os.path.abspath(sys.argv[1]), os.path.abspath(sys.argv[2])
R = "scripts/experiment-apparatus/replay.sh"
os.makedirs(os.path.join(scratch, "copy", os.path.dirname(R)), exist_ok=True)
shutil.copyfile(os.path.join(repo, R), os.path.join(scratch, "copy", R))
os.chmod(os.path.join(scratch, "copy", R), 0o755)
same = subprocess.run(["cmp", "-s", os.path.join(repo, R), os.path.join(scratch, "copy", R)]).returncode == 0
print(f"copy_identical_to_repo_file={same}")
CASES = [("本仓那一份 --list-items", f"bash {repo}/{R} --list-items"),
         ("没改过的拷贝（路径尾部相同）--list-items", f"bash {scratch}/copy/{R} --list-items"),
         ("没改过的拷贝 cd 进去相对路径 --list-items", f"cd {scratch}/copy && bash {R} --list-items")]
for label, command in CASES:
    hook_input = {"tool_name": "Bash", "cwd": repo, "agent_type": "implementation-writer", "tool_input": {"command": command}}
    env = dict(os.environ, CLAUDE_PROJECT_DIR=repo, AGENT_HOOK_DETECTIONS=os.path.join(scratch, "detections.jsonl"))
    done = subprocess.run(["bash", os.path.join(repo, ".claude/hooks/heavy-test-guard.sh")], input=json.dumps(hook_input), capture_output=True, text=True, env=env, cwd=repo)
    first = (done.stderr.strip().replace(scratch, "<scratch>").replace(repo, "<repo>").splitlines() or [""])[0][:170]
    print(f"{label}\texit={done.returncode}\t{first}")
