#!/usr/bin/env bash
# T5 两族：
#   G：61 号「本次新增了已定小节」改用共用脚本（带 --no-color --no-ext-diff），而「未定项这次碰没碰」仍是自己跑的
#      git diff -U0；使用者配了 color.ui=always 或 diff.external 时，前一半认得出定案、后一半一行都读不出 ⇒ 每条未定项都报没碰。
#   H：40、88 号把「HEAD 里还在、工作区删了」的产物算找得到；git commit --amend 把它从历史里整个抹掉时，
#      amend 之前那一次门禁放行，amend 之后同一批判红。
# 用法：bash t5-config-amend.sh <快照树> <真仓> <草稿目录>   真仓只用来 git show HEAD 那一版（只读）。
set -uo pipefail
SNAP="$(cd "$1" && pwd)"; REPO="$(cd "$2" && pwd)"; D="$3"
rm -rf "$D"; mkdir -p "$D/head/.claude/gate.d"
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1 GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
unset GATE_BASE GATE_DIFF_BASE GATE_STAGED_FROM GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE
for s in 61-settled-same-file 40-results-cited 88-quoted-result-lines; do git -C "$REPO" show "HEAD:.claude/gate.d/$s.sh" > "$D/head/.claude/gate.d/$s.sh"; done
printf '#!/bin/sh\necho "external diff: $1"\n' > "$D/fake-external-diff.sh"; chmod +x "$D/fake-external-diff.sh"
verdict() {  # $1 = 阶段，$2 = 仓 → 「退出码 首行判定」
  local out rc=0; out="$(cd "$2" && nice -n 19 bash "$1" "$2" 2>&1)" || rc=$?
  printf '退 %s | %s' "$rc" "$(grep -m1 -E '✗|✓|!' <<<"$out" | sed 's/^ *//' | python3 -c 'import sys; print(sys.stdin.read().rstrip("\\n")[:60])')"
}
echo "== G：61 号，D1 这一批新加「### 已定（…）」小节，同时在它唯一那条未定项上补了一句复核（该绿）"
mkdir -p "$D/g" && cd "$D/g" && git init -q -b master . && mkdir -p .claude/kb/decisions
printf '## D1 甲 —— 半定（一项未定）\n\n### 已定项\n\n1. **老规则**：一句。\n\n### 未定项\n\n1. **样本未定项**：还开着。\n\n## 历史版本\n' > .claude/kb/decisions/01-甲.md
git add -A && git commit -qm base
python3 - <<'PY'
p = '.claude/kb/decisions/01-甲.md'; t = open(p, encoding='utf-8').read()
t = t.replace('1. **老规则**：一句。\n', '1. **老规则**：一句。\n\n### 已定（2026-09-24，用户定案）\n\n2. **新规则**：一句。\n')
t = t.replace('1. **样本未定项**：还开着。', '1. **样本未定项**：还开着（2026-09-24 复核过，仍然开着，因为样本理由）。')
open(p, 'w', encoding='utf-8').write(t)
PY
for config in '' color.ui=always "diff.external=$D/fake-external-diff.sh"; do
  git config --unset-all color.ui 2>/dev/null; git config --unset-all diff.external 2>/dev/null
  [[ -z "$config" ]] || git config "${config%%=*}" "${config#*=}"
  printf '  %-26s 快照 61 %s\n  %-26s HEAD 61 %s\n' "${config:-（不配）}" "$(verdict "$SNAP/.claude/gate.d/61-settled-same-file.sh" "$D/g")" \
    "" "$(verdict "$D/head/.claude/gate.d/61-settled-same-file.sh" "$D/g")" | sed "s#$D#<草稿>#g"
done
echo
echo "== H：40、88 号与 git commit --amend"
mkdir -p "$D/origin.git" && git init -q --bare -b master "$D/origin.git"
mkdir -p "$D/h" && cd "$D/h" && git init -q -b master . && mkdir -p .claude/kb/experiments research/results
printf '# 实验\n' > .claude/kb/experiments.md
printf '## E1 样本一 —— 已跑（2026-09-20）\n\n产物 e1-old-2026-09-20.out。\n\n## 历史版本\n' > .claude/kb/experiments/01-样本一.md
printf 'E7RESULT name=old v=1\n' > research/results/e1-old-2026-09-20.out
git add -A && git commit -qm c0 && git remote add origin "$D/origin.git" && git push -q -u origin master
# 本地第一次提交（没推）：新实验 E2 连同它的产物，页里点名产物、整行抄一行产物
printf '## E2 样本二 —— 已跑（2026-09-24）\n\n产物 e2-new-2026-09-24.out。\n\nE7RESULT name=new v=2\n\n## 历史版本\n' > .claude/kb/experiments/02-样本二.md
printf 'E7RESULT name=new v=2\n' > research/results/e2-new-2026-09-24.out
printf '| E2 | 样本二 | e2-new-2026-09-24.out |\n' >> .claude/kb/experiments.md
git add -A && git commit -qm c1
# 收拾工作区时把这一轮的产物误删了，git rm 进暂存区，准备 amend 进 c1
git rm -q research/results/e2-new-2026-09-24.out
for stage in 40-results-cited 88-quoted-result-lines; do
  printf '  amend 之前（产物只剩 HEAD 里那一份） 快照 %s %s\n' "${stage%%-*}" "$(verdict "$SNAP/.claude/gate.d/$stage.sh" "$D/h")"
  printf '  %-36s HEAD %s %s\n' '' "${stage%%-*}" "$(verdict "$D/head/.claude/gate.d/$stage.sh" "$D/h")"
done
git commit -q --amend --no-edit
printf '  amend 之后：产物在哪个提交里都没有了——git log --all --diff-filter=D 找到 %s 条、HEAD 里 %s\n' \
  "$(git log --all --diff-filter=D --format=%h -- research/results/e2-new-2026-09-24.out | wc -l)" \
  "$(git cat-file -e HEAD:research/results/e2-new-2026-09-24.out 2>/dev/null && echo 有 || echo 没有)"
for stage in 40-results-cited 88-quoted-result-lines; do
  printf '  amend 之后                            快照 %s %s\n' "${stage%%-*}" "$(verdict "$SNAP/.claude/gate.d/$stage.sh" "$D/h")"
done
echo
echo "== H2 对照（放开用户动作）：同一段历史，误删之后不 amend、另起一次提交"
cd "$D/h" && git reset -q --hard HEAD 2>/dev/null
git -C "$D/h" reset -q --hard "$(git -C "$D/h" rev-parse HEAD)"
rm -rf "$D/h2" && mkdir -p "$D/h2" && cd "$D/h2" && git init -q -b master . && git remote add origin "$D/origin.git" && git fetch -q origin && git reset -q --hard origin/master && git branch -q -u origin/master
mkdir -p research/results
printf '## E2 样本二 —— 已跑（2026-09-24）\n\n产物 e2-new-2026-09-24.out。\n\nE7RESULT name=new v=2\n\n## 历史版本\n' > .claude/kb/experiments/02-样本二.md
printf 'E7RESULT name=new v=2\n' > research/results/e2-new-2026-09-24.out
printf '| E2 | 样本二 | e2-new-2026-09-24.out |\n' >> .claude/kb/experiments.md
git add -A && git commit -qm c1 && git rm -q research/results/e2-new-2026-09-24.out && git commit -qm c2
printf '  另起一次提交之后：git log --all --diff-filter=D 找到 %s 条\n' "$(git log --all --diff-filter=D --format=%h -- research/results/e2-new-2026-09-24.out | wc -l)"
for stage in 40-results-cited 88-quoted-result-lines; do
  printf '  另起一次提交之后                    快照 %s %s\n' "${stage%%-*}" "$(verdict "$SNAP/.claude/gate.d/$stage.sh" "$D/h2")"
done
