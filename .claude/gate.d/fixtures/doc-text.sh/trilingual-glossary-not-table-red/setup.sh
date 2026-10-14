#!/usr/bin/env bash
# 标记在，但标记下面隔了一个空行才是表（要紧挨着）：判红。
set -e
put() { mkdir -p "$(dirname "$1")"; printf '%s\n' "${@:2}" > "$1"; }
put .claude/trilingual-documents \
  'README.md  # 样本里的仓根说明，英语为默认版本'
put .claude/kb/translation-glossary.md \
  '# 翻译术语表' '' \
  '<!-- trilingual-glossary -->' '' \
  '| 中文 | English | 日本語 | 说明 / Note / 注 |' \
  '|---|---|---|---|' \
  '| 崩溃点 | crash point | クラッシュポイント | |'
put README.md '# Sample'
put README.zh.md '# 样本'
put README.ja.md '# サンプル'
