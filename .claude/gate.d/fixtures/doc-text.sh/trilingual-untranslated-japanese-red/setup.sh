#!/usr/bin/env bash
# 日语伪译文：README 日语那份第 5 行是中文原句只把术语表的词换成日语（「门禁」换成ゲート、「重放」换成リプレイ），有假名、结构与术语都对得上，
# 剩 7 个日语不用的简体字（checks-owed 日语那份约 793 行是这个样子）；第 7 行正好剩 3 个（确、记、录，门槛那一档，判红）。英语那份已译好。
set -e
put() { mkdir -p "$(dirname "$1")"; printf '%s\n' "${@:2}" > "$1"; }
put .claude/trilingual-documents \
  'README.md  # 样本里的仓根说明，中文为默认版本'
put README.md '# 样本' '' '**中文** · [English](README.en.md) · [日本語](README.ja.md)' '' \
  '| C509 | 门禁不重放旧文件 | 门禁只看这次改动里的文件，没改的旧文件里还留着的错不会被发现，要在全仓判一次。 |' \
  '' '主 agent 确认之后记录结果。'
put README.en.md '# Sample' '' '[中文](README.md) · **English** · [日本語](README.ja.md)' '' \
  '| C509 | The gate does not replay old files | The gate looks only at files in this change; errors left in old files that were not changed go unnoticed, so judge the whole repository once. |' \
  '' 'The main agent confirms it and then records the result.'
put README.ja.md '# サンプル' '' '[中文](README.md) · [English](README.en.md) · **日本語**' '' \
  '| C509 | ゲートは古いファイルをリプレイしない | ゲート只看这次改动里的文件，没改的旧文件里还留着的错不会被发现，要在全仓判一次。 |' \
  '' 'メイン agent が确認し、結果を记录する。'
