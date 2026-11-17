#!/usr/bin/env bash
# 英语那份第 5 行在反引号里引了一处标记写法的开头（`<!-- format-const:`，后面没有闭合），第 7 行有一段没译的中文，第 9 行才是一处真的 HTML 注释：
# 反引号里的那一处不是注释的开头，第 7 行照判、判红。把它当成开头，第 5 行之后到真注释的闭合都算在注释里，第 7 行看不到。日语那份已译好。
set -e
put() { mkdir -p "$(dirname "$1")"; printf '%s\n' "${@:2}" > "$1"; }
put .claude/trilingual-documents \
  'README.md  # 样本里的仓根说明，中文为默认版本'
put README.md '# 样本' '' '**中文** · [English](README.en.md) · [日本語](README.ja.md)' '' \
  '登记行写成 `<!-- format-const:` 起头的一行，名字与值都跟在冒号后面。' '' \
  '候选的代价：提交时间多一倍，写吞吐少四分之一，读延迟没有变化。' '' \
  '<!-- 样本里的一处真注释 -->'
put README.en.md '# Sample' '' '[中文](README.md) · **English** · [日本語](README.ja.md)' '' \
  'A registration line starts with `<!-- format-const:`, with the name and the value after the colon.' '' \
  'The candidate costs 提交时间多一倍，写吞吐少四分之一，读延迟没有变化.' '' \
  '<!-- 样本里的一处真注释 -->'
put README.ja.md '# サンプル' '' '[中文](README.md) · [English](README.en.md) · **日本語**' '' \
  '登録行は `<!-- format-const:` で始まる一行として書き、名前と値はコロンの後ろに続ける。' '' \
  '候補のコスト：コミット時間が倍になり、書き込みスループットが四分の一減り、読み取り遅延は変わらない。' '' \
  '<!-- 样本里的一处真注释 -->'
