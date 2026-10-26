#!/usr/bin/env bash
# 改名登记表那一组：登记表的旧名、新名两列照中文原串写、三份相同，不算用到术语；同一份里别处用到的照核。
# 中文那份：「门禁」只在新名一列、「对拍」只在旧名一列（英语、日语里都没有 gate / ゲート、differential / 突き合わせ），不报；
# 表上方的正文用了「重放」，日语写成再生（不是术语表的リプレイ）；「是什么」那一列用了「崩溃点」，英语没写 crash point。判红，只点名这两处。
set -e
put() { mkdir -p "$(dirname "$1")"; printf '%s\n' "${@:2}" > "$1"; }
put .claude/trilingual-documents \
  '.claude/kb/term-renames.md  # 样本里的改名登记表，中文为默认版本'
put .claude/kb/translation-glossary.md \
  '# 翻译术语表' '' '样本里的术语表。' '' \
  '<!-- trilingual-glossary -->' \
  '| 中文 | English | 日本語 | 说明 / Note / 注 |' \
  '|---|---|---|---|' \
  '| 崩溃点 | crash point | クラッシュポイント | |' \
  '| 重放 | replay | リプレイ | |' \
  '| 门禁 | gate | ゲート | |' \
  '| 对拍 | differential | 突き合わせ | |'
put .claude/kb/term-renames.md '# 术语改名对照表' '' \
  '**中文** · [English](term-renames.en.md) · [日本語](term-renames.ja.md)' '' \
  '这张表登记重放之类的改名。' '' \
  '<!-- term-renames:table -->' \
  '| 旧 | 新 | 匹配 | 是什么 |' '|---|---|---|---|' \
  '| 关卡 | 门禁 | 整串 | 新名在新名一列 |' \
  '| 对拍 | 双路比对 | 整串 | 旧名在旧名一列 |' \
  '| 回放点 | 断点 | 整串 | 崩溃点那一类的叫法 |'
put .claude/kb/term-renames.en.md '# Term rename table' '' \
  '[中文](term-renames.md) · **English** · [日本語](term-renames.ja.md)' '' \
  'This table records renames such as replay.' '' \
  '<!-- term-renames:table -->' \
  '| Old | New | Match | What it is |' '|---|---|---|---|' \
  '| 关卡 | 门禁 | 整串 | The new name is in the new-name column |' \
  '| 对拍 | 双路比对 | 整串 | The old name is in the old-name column |' \
  '| 回放点 | 断点 | 整串 | A name for the point where a write is cut |'
put .claude/kb/term-renames.ja.md '# 用語改名対照表' '' \
  '[中文](term-renames.md) · [English](term-renames.en.md) · **日本語**' '' \
  'この表は再生などの改名を登録する。' '' \
  '<!-- term-renames:table -->' \
  '| 旧 | 新 | 一致 | 何か |' '|---|---|---|---|' \
  '| 关卡 | 门禁 | 整串 | 新名は新名の列にある |' \
  '| 对拍 | 双路比对 | 整串 | 旧名は旧名の列にある |' \
  '| 回放点 | 断点 | 整串 | クラッシュポイントの類の呼び方 |'
