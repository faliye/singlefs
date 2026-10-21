#!/usr/bin/env bash
# 基准里登记的 README 与两组简报各有三份。这一轮：README 只改了英语和中文（工作区改动）；2026-09-14 只改了日语；
# 2026-09-28 三份都改了；2026-10-12 新写了英语（未跟踪）和中文（暂存），日语没写；other.txt 与登记无关。
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
printf '%s\n' '詳細' >> briefs/2026-09-14.ja.md
printf '%s\n' 'more' >> briefs/2026-09-28.en.md
printf '%s\n' '更多' >> briefs/2026-09-28.md
printf '%s\n' '詳細' >> briefs/2026-09-28.ja.md
put briefs/2026-10-12.en.md '# Brief'
put briefs/2026-10-12.md '# 简报'
git add briefs/2026-10-12.md
printf '%s\n' 'changed' >> other.txt
