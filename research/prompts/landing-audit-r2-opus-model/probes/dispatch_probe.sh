#!/usr/bin/env bash
# 派发闸 ④⑰ 共用报告行解析之后：本会话两份真派发提示与三份最小探针，改前（暂存区那一版）与改后（工作区）逐份 --check-prompt。
# 用法：bash dispatch_probe.sh <仓根> <模型目录> <新的空目录>
set -uo pipefail
repo="$1" model="$2" work="$3"
mkdir -p "$work/hooks-before"
cp -a "$repo/.claude/hooks/." "$work/hooks-before/"
git -C "$repo" show :.claude/hooks/runner-dispatch-guard.sh > "$work/hooks-before/runner-dispatch-guard.sh.new" && mv "$work/hooks-before/runner-dispatch-guard.sh.new" "$work/hooks-before/runner-dispatch-guard.sh"
for pair in tooling-writer-hooks-fix:tooling-writer landing-audit-r1-attack:three-way-attack probe-clerk-prose:clerk probe-attack-prose:three-way-attack probe-tooling-writer-prose:tooling-writer; do
  name="${pair%%:*}" type="${pair##*:}"
  bash "$work/hooks-before/runner-dispatch-guard.sh" --check-prompt "$model/prompts/$name.txt" --type "$type" --root "$repo" > "$work/$name.before.txt" 2>&1; before=$?
  bash "$repo/.claude/hooks/runner-dispatch-guard.sh" --check-prompt "$model/prompts/$name.txt" --type "$type" --root "$repo" > "$work/$name.after.txt" 2>&1; after=$?
  echo "name=$name type=$type before_exit=$before after_exit=$after"
  grep -h '✗ 派\|✗ 提示要' "$work/$name.after.txt" | cut -c1-160 | sed 's/^/    改后拒：/'
done
exit 0
