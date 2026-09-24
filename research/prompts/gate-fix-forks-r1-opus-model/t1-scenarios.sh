#!/usr/bin/env bash
# T1 模型：在临时 git 仓里造五段历史，甲用真仓 10 号原文件判（只读），乙 / 丙用 t1-judge.py 判。
# 用法：bash t1-scenarios.sh <真仓根> <草稿目录>
set -uo pipefail
REPO="$(cd "$1" && pwd)"; DRAFT="$2"; HERE="$(cd "$(dirname "$0")" && pwd)"
LIB="$REPO/research/scripts/changed-paths.sh"
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1 GIT_AUTHOR_NAME=m GIT_AUTHOR_EMAIL=m@x.invalid GIT_COMMITTER_NAME=m GIT_COMMITTER_EMAIL=m@x.invalid
unset GATE_BASE
root="$(mktemp -d "$DRAFT/t1.XXXXXX")"
EXP=.claude/kb/experiments/1-样例.md; D1=.claude/kb/decisions/01-甲.md; D2=.claude/kb/decisions/02-乙.md
fresh() {  # 新仓：c1 建 E1（状态 $1）与引用它的 D1、无关的 D2，同一个提交
  local d="$root/$2"; mkdir -p "$d"; cd "$d" || exit 2; git init -q -b master .
  mkdir -p .claude/kb/experiments .claude/kb/decisions
  printf '## E1 样例 —— %s\n\n正文。\n' "$1" > "$EXP"
  printf '## D1 甲 —— 已定\n\n依据 E1（样例）。\n' > "$D1"; printf '## D2 乙 —— 已定\n\n无关。\n' > "$D2"
  git add -A; git commit -qm c1
}
flip() { python3 -c "import sys,re; p=sys.argv[1]; t=open(p,encoding='utf-8').read(); open(p,'w',encoding='utf-8').write(re.sub(r'^(## E1 .*—— ).*$', r'\g<1>'+sys.argv[2], t, count=1, flags=re.M))" "$EXP" "$1"; }
judge() {
  local s10; s10="$(bash "$REPO/.claude/gate.d/10-kb-rot.sh" "$PWD" 2>&1 | sed -n '/── 3/,/── 4/p' | grep -E 'E1 |判了|没判|一个' | head -2)"
  printf '   甲（真 10 号）: %s\n' "${s10:-（第 3 段没有 E1 的行）}"
  python3 "$HERE/t1-judge.py" E1 "$LIB" | sed 's/^/   /'
}
echo "== S1 工作区把 E1 从待跑改成已跑，决策一份没动（还没提交）"; fresh 待跑 s1; flip '已跑（2026-09-23）'; judge
echo "== S1' 同上，提交成 c2 之后（工作区干净）"; git commit -qam c2; judge
echo "== S1'' 再提交一个 c3：在 E1 正文写一行「结论不改任何决策」（10 号 howto 第二条出路）"; printf '\n结论不改任何决策（零结论也是结论）。\n' >> "$EXP"; git commit -qam c3; judge
echo "== S2 c2 待跑→部分已跑 并动了 D1；c3 部分已跑→已跑、没动决策"; fresh 待跑 s2; flip '部分已跑（第一段）'; printf '补一句。\n' >> "$D1"; git commit -qam c2; flip '已跑（2026-09-23）'; git commit -qam c3; judge
echo "   —— 同一步还没提交时（c3 撤回到工作区）："; git reset -q --soft HEAD~1; judge
echo "== S3 工作区：别的会话改了 D2（无关决策）；这一批把 E1 改成已跑、没动 D1"; fresh 待跑 s3; printf '别的会话。\n' >> "$D2"; flip '已跑（2026-09-23）'; judge
echo "== S3' 工作区：别的会话改的正好是 D1（引用 E1 的那份）；这一批把 E1 改成已跑"; fresh 待跑 s3b; printf '别的会话。\n' >> "$D1"; flip '已跑（2026-09-23）'; judge
echo "== S4 搬家：c1 里 E1 已跑；c2 只把实验文件改名搬进另一份（内容逐字不变、没动决策）"; fresh '已跑（2026-09-01）' s4; git mv "$EXP" .claude/kb/experiments/001-样例.md; git commit -qm c2; EXP=.claude/kb/experiments/001-样例.md; judge
echo "== S5 拆分：c1 里 E1 已跑、住在 experiments.md（同文件另有 60 行的 E2，拆开后 git 的改名探测认不出 E1 那一份）；c2 拆成每实验一份、没动决策（357dbf0 的形状）"
d="$root/s5"; mkdir -p "$d"; cd "$d" || exit 2; git init -q -b master .; mkdir -p .claude/kb/decisions
{ printf '## E1 样例 —— 已跑（2026-09-01）\n\n正文。\n\n## E2 另一个 —— 待跑\n'; for i in $(seq 1 60); do echo "E2 正文第 $i 行"; done; } > .claude/kb/experiments.md
printf '## D1 甲 —— 已定\n\n依据 E1（样例）。\n' > "$D1"; git add -A; git commit -qm c1
mkdir -p .claude/kb/experiments; printf '## E1 样例 —— 已跑（2026-09-01）\n\n正文。\n' > .claude/kb/experiments/1-样例.md
{ printf '## E2 另一个 —— 待跑\n'; for i in $(seq 1 60); do echo "E2 正文第 $i 行"; done; } > .claude/kb/experiments/2-另一个.md; git rm -q .claude/kb/experiments.md; git add -A; git commit -qm c2; EXP=.claude/kb/experiments/1-样例.md; judge
echo "现场：$root"
