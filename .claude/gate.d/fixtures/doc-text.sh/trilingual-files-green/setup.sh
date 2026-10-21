#!/usr/bin/env bash
# 登记 README.md 与 briefs/*.md，三组每组三份齐全。登记目录之外单有一份 .en.md（docs/notes.en.md）、
# briefs/ 里另一种语言后缀的文件（2026-09-28.fr.md）都不归这一格管。
set -e
put() { mkdir -p "$(dirname "$1")"; printf '%s\n' "${@:2}" > "$1"; }
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
put briefs/2026-09-28.fr.md '# Résumé'
put docs/notes.en.md '# 笔记'
# notes/**/*.md 跨子目录，名里带点（14-I-3.1…）：以 .<两个小写字母>.md 结尾的才算语言后缀，这一组照样三份齐全。
printf '%s\n' 'notes/**/*.md  # 样本里的笔记目录，名里带点、在子目录里' >> .claude/trilingual-documents
put 'notes/a/14-I-3.1的判别.md' '# 笔记'
put 'notes/a/14-I-3.1的判别.en.md' '# Note'
put 'notes/a/14-I-3.1的判别.ja.md' '# ノート'
