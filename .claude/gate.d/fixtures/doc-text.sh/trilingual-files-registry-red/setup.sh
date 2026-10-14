#!/usr/bin/env bash
# 登记表三处不对：guide/*.md 指不到任何文件，briefs/*.md 没写理由，notes.zh.md 写成了中文那一份；文件本身三份都齐。
set -e
put() { mkdir -p "$(dirname "$1")"; printf '%s\n' "${@:2}" > "$1"; }
put .claude/trilingual-documents \
  'README.md  # 样本里的仓根说明，英语为默认版本' \
  'guide/*.md  # 样本里没有这个目录' \
  'briefs/*.md' \
  'notes.zh.md  # 写成了中文那一份'
put README.md '# Sample'
put README.zh.md '# 样本'
put README.ja.md '# サンプル'
put briefs/2026-09-14.md '# Brief'
put briefs/2026-09-14.zh.md '# 简报'
put briefs/2026-09-14.ja.md '# 概要'
