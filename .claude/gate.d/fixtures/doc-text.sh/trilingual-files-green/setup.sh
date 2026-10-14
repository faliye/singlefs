#!/usr/bin/env bash
# 登记 README.md 与 briefs/*.md，三组每组三份齐全。登记目录之外单有一份 .zh.md（docs/notes.zh.md）、
# briefs/ 里另一种语言后缀的文件（2026-09-28.fr.md）都不归这一格管。
set -e
put() { mkdir -p "$(dirname "$1")"; printf '%s\n' "${@:2}" > "$1"; }
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
put briefs/2026-09-28.fr.md '# Résumé'
put docs/notes.zh.md '# 笔记'
