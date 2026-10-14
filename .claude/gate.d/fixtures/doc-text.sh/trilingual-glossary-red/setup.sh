#!/usr/bin/env bash
# 术语表写对了，译文没照它用词：README 中文用了「崩溃点」「重放」，英语写 Crash-point（连字符，不是术语表的 crash point），
# 日语写リプレー（不是术语表的リプレイ）；2026-09-14 中文用了「门禁」，日语写関門（不是ゲート）。
set -e
put() { mkdir -p "$(dirname "$1")"; printf '%s\n' "${@:2}" > "$1"; }
put .claude/trilingual-documents \
  'README.md  # 样本里的仓根说明，英语为默认版本' \
  'briefs/*.md  # 样本里的简报，每份 YYYY-MM-DD.md 是英语版本'
put .claude/kb/translation-glossary.md \
  '# 翻译术语表' '' '样本里的术语表。' '' \
  '<!-- trilingual-glossary -->' \
  '| 中文 | English | 日本語 | 说明 / Note / 注 |' \
  '|---|---|---|---|' \
  '| 崩溃点 | crash point | クラッシュポイント | 把盘上状态切在一次写的中间<br>The disk state cut in the middle of a write<br>書き込みの途中で切ったディスク状態 |' \
  '| 重放 | replay | リプレイ | |' \
  '| 门禁 | gate | ゲート | |'
put README.md '# Sample' 'Crash-point replay has no problem.'
put README.zh.md '# 样本' '崩溃点重放时没有问题。'
put README.ja.md '# サンプル' 'クラッシュポイントのリプレーに問題はない。'
put briefs/2026-09-14.md '# Brief' 'Passed the gate.'
put briefs/2026-09-14.zh.md '# 简报' '过了门禁。'
put briefs/2026-09-14.ja.md '# 概要' '関門を通過。'
