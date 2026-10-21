#!/usr/bin/env bash
# 登记表和三份 README 都在，翻译术语表 .claude/kb/translation-glossary.md 不在：判红（不是 77），出路写建它。
set -e
put() { mkdir -p "$(dirname "$1")"; printf '%s\n' "${@:2}" > "$1"; }
put .claude/trilingual-documents \
  'README.md  # 样本里的仓根说明，中文为默认版本'
put README.en.md '# Sample'
put README.md '# 样本'
put README.ja.md '# サンプル'
