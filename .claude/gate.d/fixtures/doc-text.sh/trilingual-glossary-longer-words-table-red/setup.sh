#!/usr/bin/env bash
# 更长词登记文件七行里六行写错，逐行点名判红：第 2 行词等于术语，第 3 行词不包含术语，第 4 行术语不在术语表里，
# 第 5 行与第 1 行同一对，第 6 行理由只有一个字，第 7 行只有两列；第 1 行写对。译文里没有任何术语。
set -e
put() { mkdir -p "$(dirname "$1")"; printf '%s\n' "${@:2}" > "$1"; }
put .claude/trilingual-documents \
  'README.md  # 样本里的仓根说明，中文为默认版本'
put .claude/kb/translation-glossary.md \
  '# 翻译术语表' '' '样本里的术语表。' '' \
  '<!-- trilingual-glossary -->' \
  '| 中文 | English | 日本語 | 说明 / Note / 注 |' \
  '|---|---|---|---|' \
  '| 合表 | merged table | 統合表 | |' \
  '| 产物 | artifact | 成果物 | |' \
  '| 门禁 | gate | ゲート | |' \
  '| 重放 | replay | リプレイ | |'
mkdir -p .claude/gate.d
printf '%s\t%s\t%s\n' \
  '合表' '集合表示' '「集合」是一个词，「合表」跨在词界上，不是术语合表' \
  '门禁' '门禁' '词和术语是同一个串，写了也没有意义' \
  '产物' '副产品' '这个词里没有「产物」两个字' \
  '缓存' '缓存池' '「缓存」不在术语表的中文列里' \
  '合表' '集合表示' '又写了一次同一对，应当删掉' \
  '产物' '副产物' '短' > .claude/gate.d/trilingual-glossary-longer-words.tsv
printf '%s\t%s\n' '重放' '重放队列' >> .claude/gate.d/trilingual-glossary-longer-words.tsv
put README.en.md '# Sample' 'Nothing to report.'
put README.md '# 样本' '没有可报的。'
put README.ja.md '# サンプル' '報告なし。'
