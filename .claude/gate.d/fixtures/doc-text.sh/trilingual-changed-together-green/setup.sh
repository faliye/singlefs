#!/usr/bin/env bash
# 基准同上。这一轮：README 三份都改了（英语工作区改、中文暂存、日语工作区改）；2026-09-14 三份都删了（删也算碰）；
# 2026-10-12 新写三份（未跟踪）；2026-09-28 一份没碰；other.txt 与登记无关。
set -e
put() { mkdir -p "$(dirname "$1")"; printf '%s\n' "${@:2}" > "$1"; }
git init -q .
git config user.email t@example.com; git config user.name t
put .claude/trilingual-documents \
  'README.md  # 样本里的仓根说明，英语为默认版本' \
  'briefs/*.md  # 样本里的简报，每份 YYYY-MM-DD.md 是英语版本'
put README.md '# Sample'
put README.zh.md '# 样本'
put README.ja.md '# サンプル'
for name in 2026-09-14 2026-09-28; do
  put "briefs/$name.md" '# Brief'
  put "briefs/$name.zh.md" '# 简报'
  put "briefs/$name.ja.md" '# 概要'
done
put other.txt 'base'
git add -A && git commit -qm base
printf '%s\n' 'more' >> README.md
printf '%s\n' '更多' >> README.zh.md
git add README.zh.md
printf '%s\n' '詳細' >> README.ja.md
rm briefs/2026-09-14.md briefs/2026-09-14.ja.md
git rm -q briefs/2026-09-14.zh.md
put briefs/2026-10-12.md '# Brief'
put briefs/2026-10-12.zh.md '# 简报'
put briefs/2026-10-12.ja.md '# 概要'
printf '%s\n' 'changed' >> other.txt
