#!/usr/bin/env bash
# 正文在反引号里引了一处标记写法的开头（`<!-- format-const:`，后面没有闭合），隔一节之后才有一处真的 HTML 注释：
# 反引号里的那一处不是注释的开头，「背景」一节照判。英语那份「背景」那一段压成一句（内容量比不到门槛），判红；日语那份译全。
# 把反引号里的那一处当成开头，会从它一直去到真注释的闭合，「背景」一节整节看不到（决策变更史三份各有约七千行这样被去掉过）。
set -e
put() { mkdir -p "$(dirname "$1")"; printf '%s\n' "${@:2}" > "$1"; }
put .claude/trilingual-documents \
  'README.md  # 样本里的仓根说明，中文为默认版本'
put README.md '# 样本' '' '**中文** · [English](README.en.md) · [日本語](README.ja.md)' '' \
  '## 登记' '' \
  '登记行写成 `<!-- format-const:` 起头的一行，名字与值都跟在冒号后面，门禁照这一行核源码里的常量有没有改过。' '' \
  '## 背景' '' \
  '第二轮把提交间隔从五秒放宽到三十秒，写吞吐涨了百分之十二，提交延迟的尾部拉长了一倍，读延迟没有变化。' '' \
  '<!-- 样本里的一处真注释 -->' '' \
  '## 结尾' '' \
  '结论只在这台机器上成立，换了盘或者换了内核都要重新量过，旧数只当参考，不拿来支撑新结论。'
put README.en.md '# Sample' '' '[中文](README.md) · **English** · [日本語](README.ja.md)' '' \
  '## Registration' '' \
  'A registration line starts with `<!-- format-const:`, with the name and the value after the colon, and the gate checks against that line whether the constant in the source has changed.' '' \
  '## Background' '' \
  'Round two changed the commit interval.' '' \
  '<!-- 样本里的一处真注释 -->' '' \
  '## Closing' '' \
  'The conclusion holds only on this machine; a different disk or a different kernel needs a new measurement, and the old figures are only a reference, not support for a new conclusion.'
put README.ja.md '# サンプル' '' '[中文](README.md) · [English](README.en.md) · **日本語**' '' \
  '## 登録' '' \
  '登録行は `<!-- format-const:` で始まる一行として書き、名前と値はコロンの後ろに続け、ゲートはこの行に照らしてソースの定数が変わっていないかを確かめる。' '' \
  '## 背景' '' \
  '第2ラウンドではコミット間隔を5秒から30秒に緩め、書き込みスループットが12パーセント上がり、コミット遅延の裾が倍に伸びた。読み取り遅延は変わらなかった。' '' \
  '<!-- 样本里的一处真注释 -->' '' \
  '## 結び' '' \
  '結論はこのマシンでのみ成り立つ。ディスクやカーネルを替えたら測り直す必要があり、古い数値は参考にとどめ、新しい結論の裏付けには使わない。'
