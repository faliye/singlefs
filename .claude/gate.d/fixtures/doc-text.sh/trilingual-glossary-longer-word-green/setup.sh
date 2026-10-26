#!/usr/bin/env bash
# 术语表三行（合表、产物、门禁），更长词登记文件登记了「集合表示」盖住「合表」、「副产物」盖住「产物」。
# README 中文只有「集合表示」「副产物」，英语、日语都不带术语的译名（merged table、artifact、統合表、成果物）：术语只是更长词的一部分，不核译名；
# 简报中文真用到「合表」（三条合表），英语、日语也用了术语表的译名：照核、核对了（出现 1 次）；「门禁」中文一处也没用。
set -e
put() { mkdir -p "$(dirname "$1")"; printf '%s\n' "${@:2}" > "$1"; }
put .claude/trilingual-documents \
  'README.md  # 样本里的仓根说明，中文为默认版本' \
  'briefs/*.md  # 样本里的简报，每份 YYYY-MM-DD.md 是中文版本'
put .claude/kb/translation-glossary.md \
  '# 翻译术语表' '' '样本里的术语表。' '' \
  '<!-- trilingual-glossary -->' \
  '| 中文 | English | 日本語 | 说明 / Note / 注 |' \
  '|---|---|---|---|' \
  '| 合表 | merged table | 統合表 | |' \
  '| 产物 | artifact | 成果物 | |' \
  '| 门禁 | gate | ゲート | |'
mkdir -p .claude/gate.d
printf '%s\t%s\t%s\n' \
  '合表' '集合表示' '「集合」是一个词，「合表」跨在词界上，不是术语合表' \
  '产物' '副产物' '「副产物」是一个词，指顺带得到的东西，不是术语产物' > .claude/gate.d/trilingual-glossary-longer-words.tsv
put README.en.md '# Sample' 'The set representation loses the birth generation; by-products are counted apart.'
put README.md '# 样本' '集合表示丢出生代，副产物另算。'
put README.ja.md '# サンプル' '集合の表現は出生世代を失い、副産物は別に数える。'
put briefs/2026-09-14.en.md '# Brief' 'Three merged tables were joined.'
put briefs/2026-09-14.md '# 简报' '三条合表已经并好。'
put briefs/2026-09-14.ja.md '# 概要' '三つの統合表を併合した。'
