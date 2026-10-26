#!/usr/bin/env bash
# 排除登记表写错：第 3 行指到不在的页，第 4 行只有一列，第 5 行理由不到 4 个字，第 6 行把术语表登记第二次，第 7 行登记的是译文那一份。
# 两组译文都已译好（术语表那一组也不带中文），只有登记表的五处毛病，逐行点名判红；登记表不读时判绿。
set -e
put() { mkdir -p "$(dirname "$1")"; printf '%s\n' "${@:2}" > "$1"; }
put .claude/trilingual-documents \
  'README.md  # 样本里的仓根说明，中文为默认版本' \
  '.claude/kb/*.md  # 样本里的 kb 页，中文为默认版本'
put .claude/gate.d/trilingual-untranslated-exempt.tsv \
  '# 样本里的排除登记，五行写错' \
  $'.claude/kb/translation-glossary.md\t样本里的术语表，整份三语并列' \
  $'.claude/kb/no-such-page.md\t这一份页不在样本里' \
  '.claude/kb/translation-glossary.md' \
  $'README.md\t短' \
  $'.claude/kb/translation-glossary.md\t又登记了一次这一份术语表' \
  $'README.en.md\t译文那一份不该登记'
put README.md '# 样本' '' '**中文** · [English](README.en.md) · [日本語](README.ja.md)' '' '过了门禁。'
put README.en.md '# Sample' '' '[中文](README.md) · **English** · [日本語](README.ja.md)' '' 'Passed the gate.'
put README.ja.md '# サンプル' '' '[中文](README.md) · [English](README.en.md) · **日本語**' '' 'ゲートを通過した。'
put .claude/kb/translation-glossary.md '# 翻译术语表' '' '**中文** · [English](translation-glossary.en.md) · [日本語](translation-glossary.ja.md)' '' '样本里的术语表。'
put .claude/kb/translation-glossary.en.md '# Translation glossary' '' '[中文](translation-glossary.md) · **English** · [日本語](translation-glossary.ja.md)' '' 'The glossary of the sample.'
put .claude/kb/translation-glossary.ja.md '# 翻訳用語集' '' '[中文](translation-glossary.md) · [English](translation-glossary.en.md) · **日本語**' '' 'サンプルの用語集。'
