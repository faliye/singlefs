#!/usr/bin/env bash
# 英语漏了一段：README 英语那份「背景」一节少了中文第二段（这一节中文 3 段、英语 2 段，段数配不上）；
# 「回看」表里 D4 那一行只译了 2026-10-11 那一块、漏了 2026-10-21 那一块（照 E58 英语第 266 行那种漏法，内容量比不到门槛）；
# 「欠账」表里首列编号 C12 那一行英语整行没有（按编号配对，配不上）。日语那份逐段译全。
set -e
put() { mkdir -p "$(dirname "$1")"; printf '%s\n' "${@:2}" > "$1"; }
put .claude/trilingual-documents \
  'README.md  # 样本里的仓根说明，中文为默认版本'
put README.md '# 样本' '' '**中文** · [English](README.en.md) · [日本語](README.ja.md)' '' \
  '## 背景' '' \
  '这一页记一次实验的经过：先说要回答的问题，再说装置怎么搭、跑了几轮、每一轮各量了什么数。' '' \
  '第二轮把提交间隔从五秒放宽到三十秒，写吞吐涨了百分之十二，提交延迟的尾部拉长了一倍，读延迟没有变化。' '' \
  '结论只在这台机器上成立，换了盘或者换了内核都要重新量过，旧数只当参考，不拿来支撑新结论。' '' \
  '## 回看' '' '| 决策分项 | 关系 | 回看 |' '|---|---|---|' \
  '| D4（校验和位置） 已定项 1 | 支撑 | 2026-10-21 不受影响：只加了三语切换行；此前 2026-10-11 不受影响：只改措辞（时间词、位置指代与自称），结论、数与产物不变 |' \
  '| D8（核心索引结构） 已定项 2 | 不影响 | 2026-10-21 不受影响：只加了三语切换行；此前 2026-10-11 不受影响：只改措辞（时间词、位置指代与自称），结论、数与产物不变 |' \
  '' '## 欠账' '' '| 编号 | 欠什么 |' '|---|---|' \
  '| C11（提交间隔放宽之后读延迟没有单测） | 补一条会红的单测：提交间隔放宽到三十秒时读延迟的上界钉住，改坏间隔判红 |' \
  '| C12（换盘之后旧数没有重量的提醒） | 做成门禁里会红的一项：产物里的盘型号与登记不同就判红，出路是重量 |'
put README.en.md '# Sample' '' '[中文](README.md) · **English** · [日本語](README.ja.md)' '' \
  '## Background' '' \
  'This page records how one experiment went: first the question it answers, then how the apparatus was built, how many rounds were run, and which figures each round measured.' '' \
  'The conclusion holds only on this machine; a different disk or a different kernel needs a new measurement, and the old figures are only a reference, not support for a new conclusion.' '' \
  '## Review' '' '| Decision item | Relation | Review |' '|---|---|---|' \
  '| D4 (Checksum Location) settled item 1 | Supports | 2026-10-11 unaffected: wording only (temporal terms, locational references, self-referential terms), conclusions, figures, and artifacts unchanged |' \
  '| D8 (Core Index Structure) settled item 2 | Does not affect | 2026-10-21 unaffected: only the trilingual switch line was added; previously 2026-10-11 unaffected: wording only (temporal terms, locational references, self-referential terms), conclusions, figures, and artifacts unchanged |' \
  '' '## Owed checks' '' '| Number | What is owed |' '|---|---|' \
  '| C11 (no unit test for read latency after the commit interval was relaxed) | Add a unit test that turns red: pin the upper bound of read latency with the commit interval relaxed to thirty seconds, and break the interval to see it red |'
put README.ja.md '# サンプル' '' '[中文](README.md) · [English](README.en.md) · **日本語**' '' \
  '## 背景' '' \
  'このページは一回の実験の経緯を記す：まず答えるべき問いを述べ、次に装置の組み方、何ラウンド走らせたか、各ラウンドで何の数値を測ったかを述べる。' '' \
  '第2ラウンドではコミット間隔を5秒から30秒に緩め、書き込みスループットが12パーセント上がり、コミット遅延の裾が倍に伸びた。読み取り遅延は変わらなかった。' '' \
  '結論はこのマシンでのみ成り立つ。ディスクやカーネルを替えたら測り直す必要があり、古い数値は参考にとどめ、新しい結論の裏付けには使わない。' '' \
  '## 振り返り' '' '| 決定項目 | 関係 | 振り返り |' '|---|---|---|' \
  '| D4 (チェックサムの位置) 確定項目 1 | 支持 | 2026-10-21 影響なし：三言語切り替え行を加えただけ。それ以前の 2026-10-11 影響なし：表現の修正のみ（時間表現、位置の指示、自称）、結論・数値・成果物は不変 |' \
  '| D8 (コアインデックス構造) 確定項目 2 | 影響なし | 2026-10-21 影響なし：三言語切り替え行を加えただけ。それ以前の 2026-10-11 影響なし：表現の修正のみ（時間表現、位置の指示、自称）、結論・数値・成果物は不変 |' \
  '' '## 未完了のチェック' '' '| 番号 | 何が未完了か |' '|---|---|' \
  '| C11 (コミット間隔を緩めた後の読み取り遅延に単体テストがない) | 赤になる単体テストを一つ加える：コミット間隔を30秒に緩めたときの読み取り遅延の上限を固定し、間隔を壊すと赤になる |' \
  '| C12 (ディスク交換後に古い数値を測り直す注意がない) | ゲートで赤になる一項目にする：成果物のディスク型番が登録と違えば赤とし、出口は測り直し |'
