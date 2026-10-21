#!/usr/bin/env bash
# 术语表本身六处不对：表头第四列写成「说明」、一行只有三列、一行 English 列是空的、中文列「崩溃点」重复、
# 一行说明列只有两段、一行说明列三段而中间一段是空的。三份 README 里没用任何术语，译文那一半没有毛病。
set -e
put() { mkdir -p "$(dirname "$1")"; printf '%s\n' "${@:2}" > "$1"; }
put .claude/trilingual-documents \
  'README.md  # 样本里的仓根说明，中文为默认版本'
put .claude/kb/translation-glossary.md \
  '# 翻译术语表' '' \
  '<!-- trilingual-glossary -->' \
  '| 中文 | English | 日本語 | 说明 |' \
  '|---|---|---|---|' \
  '| 崩溃点 | crash point | クラッシュポイント | |' \
  '| 重放 | replay | リプレイ |' \
  '| 门禁 |  | ゲート | |' \
  '| 崩溃点 | crash point again | クラッシュポイント | |' \
  '| 日志 | journal | ジャーナル | 一段<br>two parts |' \
  '| 回滚 | rollback | ロールバック | 甲<br><br>丙 |'
put README.en.md '# Sample' 'Nothing here.'
put README.md '# 样本' '这里没有术语。'
put README.ja.md '# サンプル' 'ここには何もない。'
