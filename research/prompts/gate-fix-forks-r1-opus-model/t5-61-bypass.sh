#!/usr/bin/env bash
# T5 模型：61 号自带的基准取法（第 34–42 行）在「本地有没推的提交」时只看工作区，
# 同一处定案分两次提交就不被它判；对照：同一处定案还在工作区时它判红。跑的是真仓 61 号原文件（只读）。
# 用法：bash t5-61-bypass.sh <真仓根> <草稿目录>
set -uo pipefail
REPO="$(cd "$1" && pwd)"; DRAFT="$2"
work="$(mktemp -d "$DRAFT/t5b.XXXXXX")"
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1 GIT_AUTHOR_NAME=m GIT_AUTHOR_EMAIL=m@x.invalid GIT_COMMITTER_NAME=m GIT_COMMITTER_EMAIL=m@x.invalid
unset GATE_BASE
git init -q --bare "$work/origin.git"; git clone -q "$work/origin.git" "$work/c" 2>/dev/null; cd "$work/c" || exit 2
git checkout -q -b master 2>/dev/null
mkdir -p .claude/kb/decisions
printf '## D1 样例 —— 未定\n\n### 未定项\n\n1. 取多大，还没定\n' > .claude/kb/decisions/01-样例.md
git add -A; git commit -qm base; git push -q -u origin master 2>/dev/null
settle() { python3 -c "p='.claude/kb/decisions/01-样例.md'; t=open(p,encoding='utf-8').read(); open(p,'w',encoding='utf-8').write(t.replace('\n\n### 未定项', '\n\n### 已定（2026-09-23）\n\n取 16。\n\n### 未定项', 1))"; }
run61() { out="$(bash "$REPO/.claude/gate.d/61-settled-same-file.sh" 2>&1)"; rc=$?; printf '  61 号退出码 %s；%s\n' "$rc" "$(grep -m1 -E '✗|!' <<<"$out")"; }
echo "== 甲：定案还在工作区（未提交）"; settle; run61
echo "== 乙：同一处定案已提交在本地、没推；工作区另改一个无关文件"; git add -A; git commit -qm settle; echo z > other.txt; git add other.txt; run61
echo "   同一时刻 changed-paths.sh 的 gate 取法取到：$(source "$REPO/research/scripts/changed-paths.sh"; git log -1 --format=%s "$(gate_diff_base gate)")"
echo "现场：$work"
