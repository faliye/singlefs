#!/usr/bin/env bash
# 照抄中文没译的行，都不到门槛，只有「与中文逐字相同」这一条判得出：英语那份第 3 行照抄了中文的语言切换行（剩「中文」「日本語」5 个汉字，门槛 15）；
# 日语那份第 5 行照抄中文表头（剩 1 个日语不用的简体字：编）、第 9 行照抄中文标题（剩 2 个：结、论，门槛 3）。别的行已译好。逐行点名判红。
set -e
put() { mkdir -p "$(dirname "$1")"; printf '%s\n' "${@:2}" > "$1"; }
put .claude/trilingual-documents \
  'README.md  # 样本里的仓根说明，中文为默认版本'
put README.md '# 样本' '' '**中文** · [English](README.en.md) · [日本語](README.ja.md)' '' \
  '| 编号 | 名称 |' '|---|---|' '| E165 | 门禁已过 |' '' '## 结论'
put README.en.md '# Sample' '' '**中文** · [English](README.en.md) · [日本語](README.ja.md)' '' \
  '| ID | Name |' '|---|---|' '| E165 | The gate passed |' '' '## Conclusion'
put README.ja.md '# サンプル' '' '[中文](README.md) · [English](README.en.md) · **日本語**' '' \
  '| 编号 | 名称 |' '|---|---|' '| E165 | ゲート通過 |' '' '## 结论'
