#!/usr/bin/env bash
# 辩方 U4 探针：runner-dispatch-guard.sh（挂 Agent|Task 的派发闸）尾部多一行没闭合的 if，一条合规的派发还过不过。
# 只把 JSON 从标准输入喂进拷贝里的钩子，派发一个子 agent 也不起。用法：bash probe-dispatch-guard.sh <草稿目录>（已存在拒绝）
set -uo pipefail
scratch="${1:?用法：probe-dispatch-guard.sh <草稿目录>}"
repository="${SINGLEFS_REPOSITORY:-/home/fy5090/code/singlefs}"
[[ ! -e "$scratch" ]] || { echo "草稿目录已存在：$scratch"; exit 2; }
mkdir -p "$scratch/root/.claude/hooks"
python3 - "$repository" "$scratch" <<'PY'
import os, sys
repository, scratch = sys.argv[1:3]
source = os.path.join(repository, ".claude", "hooks")
for name in os.listdir(source):
    path = os.path.join(source, name)
    if os.path.isfile(path) and name.endswith((".sh", ".py", ".tsv")):
        open(os.path.join(scratch, "root", ".claude", "hooks", name), "w", encoding="utf-8").write(open(path, encoding="utf-8").read())
text = open(os.path.join(source, "runner-dispatch-guard.sh"), encoding="utf-8").read() + "\nif true; then\n"
open(os.path.join(scratch, "root", ".claude", "hooks", "runner-dispatch-guard-broken.sh"), "w", encoding="utf-8").write(text)
os.symlink(os.path.join(repository, ".claude", "scripts"), os.path.join(scratch, "root", ".claude", "scripts"))
os.symlink(os.path.join(repository, ".claude", "singlefs-ai-sop"), os.path.join(scratch, "root", ".claude", "singlefs-ai-sop"))
PY
json='{"tool_name":"Agent","cwd":"/tmp","tool_input":{"subagent_type":"clerk","description":"x","prompt":"重型测试：不跑\n开工先读 .claude/agent-common.md\n做个小事"}}'
for hook in runner-dispatch-guard.sh runner-dispatch-guard-broken.sh; do
  printf '%s' "$json" | CLAUDE_PROJECT_DIR="$scratch/root" nice -n 19 bash "$scratch/root/.claude/hooks/$hook" >"$scratch/out.txt" 2>"$scratch/err.txt"
  printf '%s\texit=%s\t%s\n' "$hook" "$?" "$(head -1 "$scratch/err.txt" | cut -c1-120)"
done
