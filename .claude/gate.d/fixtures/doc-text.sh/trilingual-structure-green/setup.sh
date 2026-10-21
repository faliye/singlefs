#!/usr/bin/env bash
# 三份结构一致，但写法处处不同，逼判法认对边界：
# README：日文、中文里编号紧挨文字（ファイルE152、文件E152，不能用 \b）；链接带标题而标题三种语言不同（目标才算）；
#   围栏里的 # 开头行与 | 开头行三份个数不同（围栏里的不算标题和表格行）；三份顶上各有一行三语切换，每份不链自己（.en.md、.ja.md 归成 .md 之后三份相同）。
# 2026-09-28：四个反引号的围栏里夹着三个反引号的行（里面的不算开围栏、也不关外层），英语用它，中文、日语用普通围栏，围栏个数仍是 2；
#   三份各链同一种语言的下一份简报（2026-10-10.md、.en.md、.ja.md），归成 <名>.md 之后是同一个目标。
set -e
put() { mkdir -p "$(dirname "$1")"; printf '%s\n' "${@:2}" > "$1"; }
put .claude/trilingual-documents \
  'README.md  # 样本里的仓根说明，中文为默认版本' \
  'briefs/*.md  # 样本里的简报，每份 YYYY-MM-DD.md 是中文版本'
put README.en.md '# Sample' '[中文](README.md) · **English** · [日本語](README.ja.md)' '## Usage' 'See D5, file E152 and I-10.1.' \
  '| a | b |' '|---|---|' '| 1 | 2 |' \
  '[docs](docs/a.md "Docs")' \
  '```bash' '# install' '# build' '| not a table' '| nor this' '```'
put README.md '# 样本' '**中文** · [English](README.en.md) · [日本語](README.ja.md)' '## 用法' '见 D5、文件E152 与 I-10.1。' \
  '| a | b |' '|---|---|' '| 1 | 2 |' \
  '[文档](docs/a.md "文档")' \
  '```bash' '# 安装' '| 不是表' '```'
put README.ja.md '# サンプル' '[中文](README.md) · [English](README.en.md) · **日本語**' '## 使い方' 'D5、ファイルE152、I-10.1 を参照。' \
  '| a | b |' '|---|---|' '| 1 | 2 |' \
  '[ドキュメント](docs/a.md "ドキュメント")' \
  '```bash' '```'
put briefs/2026-09-28.en.md '# Brief' 'Next: [2026-10-10](2026-10-10.en.md).' '````md' '```' '# inside' '```' '````' '~~~' 'tilde' '~~~'
put briefs/2026-09-28.md '# 简报' '下一份见 [2026-10-10](2026-10-10.md)。' '```md' '里面' '```' '~~~' '波浪线' '~~~'
put briefs/2026-09-28.ja.md '# 概要' '次は [2026-10-10](2026-10-10.ja.md)。' '```md' '中身' '```' '~~~' 'チルダ' '~~~'
