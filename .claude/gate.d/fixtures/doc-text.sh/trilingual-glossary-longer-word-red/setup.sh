#!/usr/bin/env bash
# 术语表与登记文件同 trilingual-glossary-longer-word-green。
# README：中文只有「集合表示」「副产物」，译文不带译名，不核。
# briefs/2026-09-14：中文有「副产物」，也有真用到术语的「这一批的产物」，英语写 deliverables、日语写成果：「产物」红在这一处（缺 artifact、缺成果物）。
# briefs/2026-09-15：中文先写「集合表示」（盖住第一处「合表」），后写「三条合表」（第二处不在任何登记的更长词里），译文不带译名：「合表」红在后一处。
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
put briefs/2026-09-14.en.md '# Brief' 'By-products do not count; the deliverables of this batch are not handed in yet.'
put briefs/2026-09-14.md '# 简报' '副产物不算，这一批的产物还没交。'
put briefs/2026-09-14.ja.md '# 概要' '副産物は数えない。この一括の成果はまだ出ていない。'
put briefs/2026-09-15.en.md '# Brief' 'The set representation has three tables.'
put briefs/2026-09-15.md '# 简报' '集合表示里有三条合表。'
put briefs/2026-09-15.ja.md '# 概要' '集合の表現には三つの表がある。'
