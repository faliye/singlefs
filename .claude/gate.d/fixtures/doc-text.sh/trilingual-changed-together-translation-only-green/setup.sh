#!/usr/bin/env bash
# 基准里登记的 README 与两组简报各有三份。这一轮只改译文的三组放行：2026-09-14 只改了日语（暂存）；2026-09-28 改了英语（工作区）、删了日语；
# 2026-10-12 只新写了英语（未跟踪，中文那一份没有，缺份归 trilingual-files）。README 三份都改了（改了中文的那一组碰全了）；other.txt 与登记无关。判绿，报放行 3 组。
set -e
put() { mkdir -p "$(dirname "$1")"; printf '%s\n' "${@:2}" > "$1"; }
git init -q .
git config user.email t@example.com; git config user.name t
put .claude/trilingual-documents \
  'README.md  # 样本里的仓根说明，中文为默认版本' \
  'briefs/*.md  # 样本里的简报，每份 YYYY-MM-DD.md 是中文版本'
put README.en.md '# Sample'
put README.md '# 样本'
put README.ja.md '# サンプル'
for name in 2026-09-14 2026-09-28; do
  put "briefs/$name.en.md" '# Brief'
  put "briefs/$name.md" '# 简报'
  put "briefs/$name.ja.md" '# 概要'
done
put other.txt 'base'
git add -A && git commit -qm base
printf '%s\n' 'more' >> README.en.md
printf '%s\n' '更多' >> README.md
printf '%s\n' '詳細' >> README.ja.md
printf '%s\n' '詳細' >> briefs/2026-09-14.ja.md
git add briefs/2026-09-14.ja.md
printf '%s\n' 'more' >> briefs/2026-09-28.en.md
rm briefs/2026-09-28.ja.md
put briefs/2026-10-12.en.md '# Brief'
printf '%s\n' 'changed' >> other.txt
