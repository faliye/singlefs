#!/usr/bin/env bash
# 术语表三行；README 里中文用了「崩溃点」「重放」，英语写 Crash points are replayed（大小写与术语表的 crash point 不同，不分大小写才对得上），
# 日语用了术语表的译名；简报三份里中文一个术语也没用，英语、日语也没有译名（中文没用的术语不核别的两份）。
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
  '| 崩溃点 | crash point | クラッシュポイント | 把盘上状态切在一次写的中间<br>The disk state cut in the middle of a write<br>書き込みの途中で切ったディスク状態 |' \
  '| 重放 | replay | リプレイ | |' \
  '| 门禁 | gate | ゲート | |'
put README.en.md '# Sample' 'Crash points are replayed.'
put README.md '# 样本' '崩溃点会被重放。'
put README.ja.md '# サンプル' 'クラッシュポイントはリプレイされる。'
put briefs/2026-09-14.en.md '# Brief' 'Nothing to report.'
put briefs/2026-09-14.md '# 简报' '没有可报的。'
put briefs/2026-09-14.ja.md '# 概要' '報告なし。'
