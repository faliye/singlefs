#!/usr/bin/env bash
# G3：核查员第 2 步「git log --since=<腿开工时刻> -- <文件>」「git status --short -- <文件>」两条现查，
# 在三种文件上看不看得见「腿开工之后被改过」。临时仓建在 <草稿目录> 下，提交时刻用 GIT_*_DATE 写死，不 sleep。
# 用法：bash g3-git-blind.sh <草稿目录>
set -uo pipefail
draft="$1"; repo="$draft/g3-repo"; rm -rf "$repo"; mkdir -p "$repo"; cd "$repo" || exit 1
git init -q .; git config user.email m@x; git config user.name m
mkdir -p .claude/singlefs-ai-sop/rules .claude/hooks research/scripts
printf '/.claude/singlefs-ai-sop/\n/research/scripts/memory-peaks.tsv\n' > .gitignore
seq 1 20 | sed 's/^/rule line /' > .claude/singlefs-ai-sop/rules/evidence-discipline.md
seq 1 20 | sed 's/^/peak /' > research/scripts/memory-peaks.tsv
seq 1 20 | sed 's/^/hook line /' > .claude/hooks/bash-command-detector.sh
seq 1 20 | sed 's/^/tracked line /' > .claude/hooks/tracked-changed-late.sh
GIT_AUTHOR_DATE='2026-09-26T15:00:00Z' GIT_COMMITTER_DATE='2026-09-26T15:00:00Z' git add -A  # clock-times:allow 造提交时间戳的自检输入
GIT_AUTHOR_DATE='2026-09-26T15:00:00Z' GIT_COMMITTER_DATE='2026-09-26T15:00:00Z' git commit -qm base  # clock-times:allow 造提交时间戳的自检输入
# 腿甲先开工、随后读到这几份文件的第 10 行；腿乙最后才派
leg_a_start='2026-09-26T15:15:00Z'; given_time='2026-09-26T15:30:00Z'  # clock-times:allow 喂给 git log --since 的两个查询时刻
# 之后别的会话同步上游 SOP（被忽略的目录）、跑了一次内存包装（被忽略的峰值表），各在第 3 行前插一行
sed -i '3i inserted by upstream sync' .claude/singlefs-ai-sop/rules/evidence-discipline.md
sed -i '3i peak of another run' research/scripts/memory-peaks.tsv
# 再之后别的会话在一份跟踪着的钩子第 3 行前插一行并提交
sed -i '3i inserted and committed' .claude/hooks/tracked-changed-late.sh
GIT_AUTHOR_DATE='2026-09-26T15:25:00Z' GIT_COMMITTER_DATE='2026-09-26T15:25:00Z' git commit -qam 'other session'  # clock-times:allow 造提交时间戳的自检输入
echo "腿引的是第 10 行；现在第 10 行是什么、那两条现查各出什么（核查员第 2 步 / 输入一节第 21 行的判法：没改过记 ✗）"
for f in .claude/singlefs-ai-sop/rules/evidence-discipline.md research/scripts/memory-peaks.tsv .claude/hooks/tracked-changed-late.sh .claude/hooks/bash-command-detector.sh; do
  for t in "$leg_a_start" "$given_time"; do
    log="$(git log --since="$t" --format=%h -- "$f" | tr '\n' ' ')"
    status="$(git status --short -- "$f")"
    line10="$(awk 'NR==10' "$f")"
    if [[ -z "$log" && -z "$status" ]]; then seen="没改过 ⇒ 对不上记 ✗"; else seen="改过 ⇒ 分不清"; fi
    case "$line10" in *" 10") truth="第 10 行没动";; *) truth="第 10 行已变成「$line10」";; esac
    printf '%s\tT=%s\tgit log=[%s]\tgit status=[%s]\t%s\t真相：%s\n' "$f" "$t" "$log" "$status" "$seen" "$truth"
  done
done
echo "看得见被忽略文件的两种现查（推的改法，不是定义里的）："
for f in .claude/singlefs-ai-sop/rules/evidence-discipline.md research/scripts/memory-peaks.tsv; do
  printf '%s\tgit status --short --ignored=[%s]\tgit check-ignore=[%s]\n' "$f" "$(git status --short --ignored -- "$f")" "$(git check-ignore -- "$f")"
done
