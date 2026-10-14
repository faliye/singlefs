#!/usr/bin/env bash
# V6：派发闸 ⑥ 读不到 lib_abandoned_rounds.py 时不收任何叫停的轮（钩子文件头第 41 行的说法）。
# 用法：nolib.sh <冻结副本根> <树的根目录（make_gate_trees.sh 造的 other-name-unrecognised，库收这一轮）> <草稿目录（要不存在）>
# 先用完整的钩子喂一遍（⑥ 放行），再拷一份去掉库的钩子目录喂同一份提示（⑥ 应当拒、stderr 写读不到库）。
set -euo pipefail
frozen="${1:?冻结副本根}"; tree="${2:?树}"; work="${3:?草稿目录}"
[[ ! -e "$work" ]] || { echo "  ✗ $work 已存在" >&2; exit 2; }
mkdir -p "$work/nolib/.claude" "$work/nolib/scripts"
cp -r "$frozen/.claude/hooks" "$work/nolib/.claude/hooks"
cp -r "$frozen/.claude/singlefs-ai-sop" "$work/nolib/.claude/" 2>/dev/null || true
cp -r "$frozen/.claude/scripts" "$work/nolib/.claude/" 2>/dev/null || true
cp "$frozen/.claude/heavy-thread-default" "$work/nolib/.claude/"
cp -r "$frozen/scripts/hosts" "$work/nolib/scripts/hosts"
rm "$work/nolib/.claude/hooks/lib_abandoned_rounds.py"
mkdir -p "$tree/.claude"; cp -r "$frozen/.claude/agents" "$tree/.claude/agents"
printf '重型测试：不跑\n草稿目录：%s/x/\n报告：%s/x/r.md\n条款：x\n要动的 crates 文件：crates/foo/src/lib.rs\n出口：x\n预计挂钟：2 小时内\n线程上限：31\n' "$work" "$work" > "$work/prompt.md"
echo "--- 完整的钩子"; (cd / && bash "$frozen/.claude/hooks/runner-dispatch-guard.sh" --check-prompt "$work/prompt.md" --type implementation-writer --root "$tree" 2>&1 | grep -E '^(✗ 派|  ✓|!)' | python3 -I -c 'import sys; [print(line.rstrip("\n")[:140]) for line in sys.stdin]' || true)
echo "--- 去掉库的钩子"; (cd / && bash "$work/nolib/.claude/hooks/runner-dispatch-guard.sh" --check-prompt "$work/prompt.md" --type implementation-writer --root "$tree" 2>&1 | grep -E '^(✗ 派|  ✓|!)' | python3 -I -c 'import sys; [print(line.rstrip("\n")[:140]) for line in sys.stdin]' || true)
