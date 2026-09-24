#!/usr/bin/env bash
# T5：gate_added_lines 的输入形状，与 40 / 61 / 88 新旧两版在同一现场上的判定。
# 用法：bash t5-added-lines.sh <真仓根> <草稿目录>   只读真仓；现场全建在草稿目录下的临时 git 仓里。
set -uo pipefail
REPO="$(cd "$1" && pwd)"; D="$2"; mkdir -p "$D"
STAGES="${STAGES:-$REPO}"   # 阶段与共用脚本从哪一份仓取；t5-fix.sh 指到补过的副本
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
unset GATE_BASE GATE_DIFF_BASE GATE_STAGED_FROM
OLD="$D/old"; mkdir -p "$OLD"
for s in 40-results-cited 61-settled-same-file 88-quoted-result-lines; do
  git -C "$REPO" show "HEAD:.claude/gate.d/$s.sh" > "$OLD/$s.sh"
done
new() { local out rc=0; out="$(cd "$2" && bash "$STAGES/.claude/gate.d/$1.sh" "$2" 2>&1)" || rc=$?; echo "    新版 $1：退出码 $rc；$(grep -E '✗|✓|!' <<<"$out" | head -3 | tr '\n' ' ')"; }
old() { local out rc=0; out="$(cd "$2" && bash "$OLD/$1.sh" "$2" 2>&1)" || rc=$?; echo "    HEAD 版 $1：退出码 $rc；$(grep -E '✗|✓|!' <<<"$out" | head -3 | tr '\n' ' ')"; }
scene() { rm -rf "$D/$1"; mkdir -p "$D/$1"; cd "$D/$1" && git init -q -b master .; }
upstream_here() { git branch -f 上游 HEAD && git branch -q --set-upstream-to=上游 master; }
added() { local out rc=0; out="$(source "$STAGES/research/scripts/changed-paths.sh"; gate_added_lines "$@")" || rc=$?; [[ -z "$out" ]] || sed -n l <<<"$out" | sed 's/^/    | /'; echo "    gate_added_lines 退 $rc"; }
decision() { mkdir -p .claude/kb/decisions; cp "$REPO/.claude/gate.d/fixtures/61-settled-same-file.sh/red/.claude/kb/decisions/97-样本.md" "$1"; printf '# 决策索引\n' > .claude/kb/decisions.md; }

echo "== C1 决策文件路径带空格：定案加在工作区，未定项没碰（61 本该红）"
scene c1; decision '.claude/kb/decisions/97 样本.md'; git add -A; git commit -qm base; upstream_here
printf '\n### 已定项 2 —— 已定（2026-09-01）：取甲\n\n依据。\n' >> '.claude/kb/decisions/97 样本.md'
added HEAD .claude/kb/decisions; new 61-settled-same-file "$PWD"; old 61-settled-same-file "$PWD"

echo "== C2 实验页新增一行以「++ 」开头，其后新增一行抄错的 E7RESULT（88 本该红）"
scene c2; mkdir -p .claude/kb/experiments research/results; printf 'E7RESULT name=a value=1\n' > research/results/e1-x.out
printf '## E1 样本 —— 已跑\n\n旧正文。\n' > .claude/kb/experiments/01-样本.md; git add -A; git commit -qm base; upstream_here
printf '++ 注：下一行是抄的产物\nE7RESULT name=a value=2\n' >> .claude/kb/experiments/01-样本.md
added HEAD .claude/kb; new 88-quoted-result-lines "$PWD"; old 88-quoted-result-lines "$PWD"

echo "== C3 实验页新点名一份不存在的产物；同目录另有一份未跟踪的二进制文件（vim 交换文件，.gitignore 不管它）"
scene c3; mkdir -p .claude/kb/experiments research/results; printf '# 实验索引\n' > .claude/kb/experiments.md
printf '## E1 样本 —— 已跑\n\n产物 `research/results/e1-x.out`。\n' > .claude/kb/experiments/01-样本.md; printf 'x\n' > research/results/e1-x.out
git add -A; git commit -qm base; upstream_here
printf '重跑产物 `research/results/e1-never-2026-09-24.out`。\n' >> .claude/kb/experiments/01-样本.md
echo "  -- 没有交换文件"; new 40-results-cited "$PWD"; old 40-results-cited "$PWD"
printf 'b0VIM 9.1\0\0\0\0\0\0\0\0x\n' > .claude/kb/experiments/.01-样本.md.swp
echo "  -- 有交换文件（git status --porcelain：$(git -c core.quotepath=false status --porcelain | tr '\n' ' ')）"; new 40-results-cited "$PWD"; old 40-results-cited "$PWD"

echo "== C4 一页带一行早先抄下、已对不上产物的 E7RESULT；只把它搬个家（不改一个字）"
scene c4; mkdir -p .claude/kb/experiments research/results; printf 'E7RESULT name=a value=1\n' > research/results/e1-x.out
printf '## E1 样本 —— 已跑\n\nE7RESULT name=a value=0\n' > .claude/kb/experiments/01-旧名.md; git add -A; git commit -qm base; upstream_here
mv .claude/kb/experiments/01-旧名.md .claude/kb/experiments/01-新名.md
echo "  -- mv 之后、git add 之前"; new 88-quoted-result-lines "$PWD"; old 88-quoted-result-lines "$PWD"
git add -A
echo "  -- git add -A 之后（暂存区里是一次改名）"; new 88-quoted-result-lines "$PWD"; old 88-quoted-result-lines "$PWD"
git config diff.renames false
echo "  -- 同上，本仓配了 diff.renames=false"; new 88-quoted-result-lines "$PWD"; git config --unset diff.renames

echo "== C5 路径带制表符、未跟踪路径带制表符"
scene c5; mkdir -p k; printf 'a\n' > "k/t	x.md"; git add -A; git commit -qm base; printf 'a\nb\n' > "k/t	x.md"; printf 'c\n' > "k/u	y.md"
added HEAD k

echo "== C6 git 配置扫一遍：只看 gate_added_lines 的输出与不配时逐字相同否"
scene c6; mkdir -p k; printf 'a\nb\nc\n' > k/f.md; printf 'x\n' > k/g.md; git add -A; git commit -qm base
printf 'a\nB\nc\nd\n' > k/f.md; git mv k/g.md k/h.md; printf 'x\ny\n' > k/h.md
ref="$(added HEAD k)"
for cfg in diff.noprefix=true diff.mnemonicPrefix=true diff.relative=true color.ui=always color.diff=always \
           diff.renames=false diff.renames=copies diff.algorithm=patience diff.algorithm=histogram diff.context=5 \
           diff.suppressBlankEmpty=true diff.interHunkContext=10 core.quotepath=true diff.external=/bin/false; do
  git config "${cfg%%=*}" "${cfg#*=}"; got="$(added HEAD k)"; git config --unset "${cfg%%=*}"
  [[ "$got" == "$ref" ]] && echo "    $cfg：相同" || { echo "    $cfg：不同"; diff <(echo "$ref") <(echo "$got") | sed 's/^/      /'; }
done

echo "== C7 没有上游（本地分支没设 upstream、没有 gate-ok）：实验页已提交，点名一份不存在的产物、抄了一行对不上的 E7RESULT；工作区干净"
scene c7; mkdir -p .claude/kb/experiments research/results; printf '# 实验索引\n' > .claude/kb/experiments.md
printf 'E7RESULT name=a value=1\n' > research/results/e1-x.out
printf '## E1 样本 —— 已跑\n\n产物 `research/results/e1-x.out`。\n' > .claude/kb/experiments/01-样本.md; git add -A; git commit -qm base
git checkout -q -b 功能分支
printf '重跑产物 `research/results/e1-never-2026-09-24.out`。\nE7RESULT name=a value=2\n' >> .claude/kb/experiments/01-样本.md; git add -A; git commit -qm 页
echo "  -- 分支「功能分支」，没有上游；上游版 lib.sh diff_base 在这里取到 $(cd "$PWD" && bash -c 'source "$1"; diff_base .' _ "$REPO/.claude/singlefs-ai-sop/scripts/lib.sh" 2>/dev/null | cut -c1-7)（master 的 merge-base），共用脚本 gate 取法取到 $(source "$REPO/research/scripts/changed-paths.sh"; gate_diff_base gate)"
new 40-results-cited "$PWD"; old 40-results-cited "$PWD"; new 88-quoted-result-lines "$PWD"; old 88-quoted-result-lines "$PWD"

echo "== C8 git 失败时（索引文件坏了：git diff 退 128）：共用脚本退非 0，调用方判不判退出码"
scene c8; mkdir -p crates/a/src; printf 'pub fn f() {}\n' > crates/a/src/lib.rs; git add -A; git commit -qm base; upstream_here
printf 'pub fn f() { let _ = 1; }\n' > crates/a/src/lib.rs
for s in 56-crates-adversarial-review; do
  echo "  -- 索引完好"; new "$s" "$PWD"
  cp .git/index "$D/c8.index"; printf 'DIRC garbage' > .git/index
  echo "  -- 索引坏了（git diff --name-only HEAD 退 $(git diff --name-only HEAD >/dev/null 2>&1; echo $?)；gate_changed_paths 退 $( (source "$REPO/research/scripts/changed-paths.sh"; gate_changed_paths HEAD untracked >/dev/null 2>&1); echo $?)）"; new "$s" "$PWD"
  cp "$D/c8.index" .git/index
done
echo "  调 gate_changed_paths 而不判它退出码的地方（真仓现查）："
grep -nE '^\s*[a-z_]+="\$\(gate_changed_paths .*\)"\s*$' "$REPO"/.claude/gate.d/*.sh | sed "s|$REPO/||; s/^/    /"
