#!/usr/bin/env bash
# Q2（第二轮 S4，判「站得住」）：仓副本里正文改过的、登记为 light 的自证，规则说「按提交时的检查办，主 agent 在集成副本里带 SINGLEFS_HEAVY_TESTS=commit 跑、不弹窗」。
# 喂法：stdin JSON 喂给 <那一版副本>/.claude/hooks/heavy-test-guard.sh（agent_type 有值是子 agent，空是主 agent），工作根设成 <工作目录>/root。
# 被起的脚本是登记为 light 的 scripts/gate/apply-writer-patch.py；「改过正文」是在副本那一份末尾加一行注释。
# 输出里这一版的工作目录先换成 <CASE>、那一版副本根换成 <TREE> 再截断（复跑换了草稿目录，截断点不动）。
# 用法：q2_selftest_copy.sh <名>=<那一版副本根> <工作目录>
set -euo pipefail
label="${1%%=*}" version="${1#*=}" work="$2/$label"
rm -rf "$work"; mkdir -p "$work/root" "$work/outside"
copy_tree() {   # 目录：从那一版副本拷登记表与 scripts/gate、.claude/scripts（脚本 source 的准入库）
  mkdir -p "$1/.claude/hooks" "$1/scripts"
  cp "$version/.claude/hooks/selftest-registry.tsv" "$1/.claude/hooks/"
  cp -r "$version/scripts/gate" "$1/scripts/gate"; cp -r "$version/.claude/scripts" "$1/.claude/scripts"
}
copy_tree "$work/root/same"; copy_tree "$work/root/edited"; copy_tree "$work/outside/edited"
printf '%s\n' '# 副本里改过的一行' >> "$work/root/edited/scripts/gate/apply-writer-patch.py"
printf '%s\n' '# 副本里改过的一行' >> "$work/outside/edited/scripts/gate/apply-writer-patch.py"
printf '%s\n' '#!/usr/bin/env python3' 'print("new")' > "$work/root/edited/scripts/gate/brand-new-tool.py"
feed() {   # 名 agent_type（-是主 agent） 工作目录 命令
  local input code
  input="$(python3 -c 'import json, sys; d = {"tool_name": "Bash", "cwd": sys.argv[2], "tool_input": {"command": sys.argv[3]}}
if sys.argv[1] != "-": d["agent_type"] = sys.argv[1]
print(json.dumps(d))' "$2" "$3" "$4")"
  code=0
  ( cd "$3" && SINGLEFS_WORK_ROOT="$work/root" CLAUDE_PROJECT_DIR="$version" PYTHONDONTWRITEBYTECODE=1 bash "$version/.claude/hooks/heavy-test-guard.sh" <<<"$input" ) > "$work/$1.out" 2>&1 || code=$?
  printf '%s\t%s\t退出=%s\t%s\n' "$label" "$1" "$code" "$(grep -m1 -E '✗|→ 怎么办|仓副本' "$work/$1.out" | python3 -c 'import sys; sys.stdout.write(sys.stdin.read().strip().replace(sys.argv[1], "<CASE>").replace(sys.argv[2], "<TREE>")[:150])' "$work" "$version")"
}
s="scripts/gate/apply-writer-patch.py --selftest"
feed 子agent-没改过的副本 tooling-writer "$work/root/same" "python3 $work/root/same/$s"
feed 子agent-改过正文 tooling-writer "$work/root/edited" "python3 $work/root/edited/$s"
feed 子agent-改过正文-自加commit前缀 tooling-writer "$work/root/edited" "SINGLEFS_HEAVY_TESTS=commit python3 $work/root/edited/$s"
feed 主agent-改过正文-不带前缀 - "$work/root/edited" "python3 $work/root/edited/$s"
feed 主agent-改过正文-带commit - "$work/root/edited" "SINGLEFS_HEAVY_TESTS=commit python3 $work/root/edited/$s"
feed 主agent-改过正文-带user-request - "$work/root/edited" "SINGLEFS_HEAVY_TESTS=user-request python3 $work/root/edited/$s"
feed 主agent-工作根外的副本-带commit - "$work/outside/edited" "SINGLEFS_HEAVY_TESTS=commit python3 $work/outside/edited/$s"
feed 子agent-副本里新脚本没登记 tooling-writer "$work/root/edited" "python3 $work/root/edited/scripts/gate/brand-new-tool.py --selftest"
feed 主agent-副本里新脚本没登记-带commit - "$work/root/edited" "SINGLEFS_HEAVY_TESTS=commit python3 $work/root/edited/scripts/gate/brand-new-tool.py --selftest"
