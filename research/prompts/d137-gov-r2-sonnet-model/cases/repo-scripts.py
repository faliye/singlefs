# 本仓每一份 shell 脚本，各以 `bash <脚本> --help` 与 `<脚本> --help`（直接执行）起，喂两个钩子；比修补前后的退出码与检出条数。
# 仓根从 REPO 环境变量接（rerun.sh 设）。
import os, subprocess
repo = os.environ["REPO"]
out = subprocess.run(["git", "-C", repo, "ls-files", "--", "scripts", ".claude/gate.d", ".claude/hooks", ".claude/scripts"], capture_output=True, text=True).stdout.split()
files = [f for f in out if f.endswith(".sh") and "/fixtures/" not in f and os.path.exists(os.path.join(repo, f))]
CASES = []
for f in sorted(files):
    for hook in ("heavy-test-guard.sh", "bash-command-detector.sh"):
        for agent in ("main", "three-way-defense"):
            CASES.append(("repo-sh", hook, agent, f"bash {f} --help"))
