#!/usr/bin/env bash
# T11：上游 diff_base（gate.sh 导出成 GATE_DIFF_BASE）与共用脚本 gate_diff_base gate 在八种仓状态下各取到哪个提交；
# 两个候选（不认 / 认 GATE_DIFF_BASE）各让哪道阶段判错；认它时真仓的基准怎样漏进样本自检。
# 用法：bash t11-bases.sh <真仓根> <草稿目录>   只读真仓；现场全建在草稿目录下的临时 git 仓里。
# 「认」这一臂用 GATE_BASE="$(diff_base)" 跑真阶段来模拟（共用脚本第一档就认 GATE_BASE，与「先认 GATE_DIFF_BASE」取到同一个提交）。
set -uo pipefail
REPO="$(cd "$1" && pwd)"; D="$2"; mkdir -p "$D"
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
unset GATE_BASE GATE_DIFF_BASE GATE_STAGED_FROM
LIB="$REPO/.claude/singlefs-ai-sop/scripts/lib.sh"; CP="$REPO/research/scripts/changed-paths.sh"
up_base() { bash -c 'source "$1" >/dev/null 2>&1; diff_base .' _ "$LIB" 2>/dev/null; }
cp_base() { bash -c 'source "$1"; gate_diff_base gate' _ "$CP"; }
name() { local h; h="$(git rev-parse -q --verify "$1^{commit}" 2>/dev/null)" || { echo "$1"; return; }; git log -1 --format=%s "$h"; }
scene() { rm -rf "$D/$1"; mkdir -p "$D/$1"; cd "$D/$1" && git init -q -b master . && for c in c1 c2 c3; do echo "$c" > f; git add f; git commit -qm "$c"; done; }
row() { printf '  %-44s diff_base → %-8s  gate_diff_base gate → %s\n' "$1" "$(name "$(up_base)")" "$(name "$(cp_base)")"; }

echo "== 两个取法在八种仓状态下取到的提交（提交名是提交说明）"
scene s1; git branch 上游 HEAD; git branch -q -u 上游 master; echo w >> f; row "S1 master 全推出去了，工作区有改动"
scene s2; git branch 上游 HEAD~1; git branch -q -u 上游 master; row "S2 master 本地多一个没推的提交 c3"
scene s3; row "S3 master 没配上游"
scene s4; git checkout -q -b 功能; echo c4 > f; git commit -qam c4; row "S4 功能分支 c4，没配上游"
scene s5; git checkout -q -b 功能; echo c4 > f; git commit -qam c4; git branch 远端功能 HEAD; git branch -q -u 远端功能; row "S5 功能分支 c4，上游是它自己、全推出去了"
scene s6; git checkout -q --detach HEAD; row "S6 游离 HEAD（在 c3 上）"
scene s7; git branch 上游 HEAD; git branch -q -u 上游 master; git update-ref refs/sop/gate-ok HEAD~1; row "S7 master 全推出去了，gate-ok 在 c2"
scene s8; git branch 上游 HEAD; git branch -q -u 上游 master; git reset -q --hard HEAD~1; row "S8 master 落后上游一个提交（fetch 了没合）"

run61() { local out rc=0; out="$(bash "$REPO/.claude/gate.d/61-settled-same-file.sh" "$PWD" 2>&1)" || rc=$?; echo "$rc $(grep -E '✗|!' <<<"$out" | head -1)"; }
dec() { mkdir -p .claude/kb/decisions; cp "$REPO/.claude/gate.d/fixtures/61-settled-same-file.sh/red/.claude/kb/decisions/97-样本.md" .claude/kb/decisions/; printf '# 决策索引\n' > .claude/kb/decisions.md; }
settle() { printf '\n### 已定项 2 —— 已定（2026-09-01）：取甲\n\n依据。\n' >> .claude/kb/decisions/97-样本.md; }
echo
echo "== 61 号（真仓现在的文件）：定案已提交、未定项没碰；工作区另改一个无关文件。「不认」= 直接跑，「认」= GATE_BASE=diff_base"
for st in S3 S4 S6; do
  scene "61-$st"; dec; git add -A; git commit -qm 决策; case $st in S4) git checkout -q -b 功能;; esac
  settle; git add -A; git commit -qm 定案; [[ $st == S6 ]] && git checkout -q --detach HEAD; echo 无关 > 无关.md
  printf '  %s  不认：%s\n' "$st" "$(run61)"; printf '  %s  认（GATE_BASE=%s）：%s\n' "$st" "$(name "$(up_base)")" "$(GATE_BASE="$(up_base)" run61)"
done

echo
echo "== 88 号：S1（全推出去了）。上一轮提交抄了产物行并已推；这一批照归档规则删掉上一轮的产物、换上新产物、页里点名新产物"
scene 88-s1; mkdir -p .claude/kb/experiments research/results; printf 'E7RESULT name=a value=1\n' > research/results/e1-r1.out
printf '## E1 样本 —— 已跑\n\n' > .claude/kb/experiments/01-样本.md; git add -A; git commit -qm 第一轮产物
printf '第一轮：\nE7RESULT name=a value=1\n' >> .claude/kb/experiments/01-样本.md; git add -A; git commit -qm 第一轮抄行
git branch 上游 HEAD; git branch -q -u 上游 master
git rm -q research/results/e1-r1.out; mkdir -p research/results; printf 'E7RESULT name=a value=2\n' > research/results/e1-r2.out; printf '第二轮产物 e1-r2.out。\n' >> .claude/kb/experiments/01-样本.md
r88() { local out rc=0; out="$(bash "$REPO/.claude/gate.d/88-quoted-result-lines.sh" "$PWD" 2>&1)" || rc=$?; echo "$rc $(grep -E '✗|!|✓' <<<"$out" | head -1)"; }
echo "  不认：$(r88)"; echo "  认（GATE_BASE=$(name "$(up_base)")）：$(GATE_BASE="$(up_base)" r88)"
