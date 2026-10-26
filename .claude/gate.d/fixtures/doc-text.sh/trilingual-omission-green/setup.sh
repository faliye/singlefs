#!/usr/bin/env bash
# 逐段译全的一组（README）与一份 kb 页，有几处三份写法不同而内容一样：
# 中文的三语切换行下面没空行、紧接着正文（E37 页那种写法；切换行不算一段，前后断开）；中文段落折行写成几行、英语日语一行；
# 列表项带续行；中文多一段跨空行的 HTML 注释（注释不算段）；围栏代码块里三份的注释与行数不同（围栏整块不比）；
# 反引号段与链接目标里带长串中文（三份照原样留，不算内容量）；「欠账」表首列编号三份次序不同（C3、C1、C2 对 C1、C2、C3，按编号配对），
# 三行长短差得多（按次序配会配错、比值出界）；一行中文不到 20 的短表格行比值偏低（不比）。判绿。
set -e
put() { mkdir -p "$(dirname "$1")"; printf '%s\n' "${@:2}" > "$1"; }
put .claude/trilingual-documents \
  'README.md  # 样本里的仓根说明，中文为默认版本' \
  '.claude/kb/*.md  # 样本里的 kb 页，中文为默认版本'
put README.md '# 样本' '' '**中文** · [English](README.en.md) · [日本語](README.ja.md)' \
  '**它问的是**：把日志实例代号放进序号的高位，' '是不是那张出路表里一条真出路，代价落在哪几个字段上。' '' \
  '<!-- 这一段只在中文那一份里，' '' '给写的人看的说明，读的人看不到 -->' '' \
  '- 第一条：装置连跑五遍，产物的校验和逐字节相同，' '  说明没有隐藏状态，但不说明统计上稳定。' \
  '- 第二条：变异表二十二条全抓，只说明写了的断言都在跑，不说明没写的断言不缺。' '' \
  '```text' '# 中文注释：照原样跑' 'bash replay.sh E37' '```' '' \
  '复跑入口是 `scripts/experiment-apparatus/replay.sh E37（日志实例代号的复跑登记与产物逐字节比对）`，记录在 [第一轮的经过](records/第一轮的经过与结论都写在这一份记录里面.md)。' '' \
  '## 欠账' '' '| 编号 | 欠什么 |' '|---|---|' \
  '| C3（序号高位放实例代号之后，回绕的那一刻没有单测，也没有变异去改坏回绕的判断） | 补一条会红的单测：序号回绕到零时实例代号加一，改坏加一那一行判红；再配一条变异，把回绕判断的比较符改反 |' \
  '| C1（实例代号的宽度没登记） | 登记宽度 |' \
  '| C2（产物里没报序号用了多少位） | 产物末尾报一行用了多少位，与登记的宽度比 |' \
  '| 短行 | 第一版的操作与回退 |'
put README.en.md '# Sample' '' '[中文](README.md) · **English** · [日本語](README.ja.md)' '' \
  '**What it asks**: whether putting the log instance identifier into the high bits of the sequence number is a real way out in that table, and which fields bear the cost.' '' \
  '- First: five consecutive runs of the apparatus give byte-identical artifact checksums, which shows there is no hidden state but not statistical stability.' \
  '- Second: all twenty-two rows of the mutation table are caught, which only shows that the written assertions run, not that no assertion is missing.' '' \
  '```text' '# English comment: run as is' '# (the same command as in the Chinese page)' 'bash replay.sh E37' '```' '' \
  'The replay entry is `scripts/experiment-apparatus/replay.sh E37（日志实例代号的复跑登记与产物逐字节比对）`, recorded in [how round one went](records/第一轮的经过与结论都写在这一份记录里面.md).' '' \
  '## Owed checks' '' '| Number | What is owed |' '|---|---|' \
  '| C1 (instance identifier width not registered) | Register the width |' \
  '| C2 (the artifact does not report how many bits the sequence number uses) | Report one line at the end of the artifact with the bits used, compared with the registered width |' \
  '| C3 (after the instance identifier goes into the high bits, there is no unit test for the moment of wraparound and no mutation that breaks the wraparound check) | Add a unit test that turns red: when the sequence number wraps to zero the instance identifier goes up by one, and breaking that line turns it red; also add a mutation that flips the comparison in the wraparound check |' \
  '| Short row | Rollback |'
put README.ja.md '# サンプル' '' '[中文](README.md) · [English](README.en.md) · **日本語**' '' \
  '**問うていること**：ログインスタンス識別子をシーケンス番号の上位ビットに入れることが、あの出口表の本当の出口の一つになるか、コストはどのフィールドに落ちるか。' '' \
  '- 第1項：装置を5回続けて走らせると成果物のチェックサムがバイト単位で一致する。隠れた状態がないことは示すが、統計的な安定は示さない。' \
  '- 第2項：変異表の22件をすべて捕捉した。書いたアサーションが走っていることだけを示し、書いていないアサーションが欠けていないことは示さない。' '' \
  '```text' '# 日本語のコメント：そのまま走らせる' 'bash replay.sh E37' '```' '' \
  '再実行の入口は `scripts/experiment-apparatus/replay.sh E37（日志实例代号的复跑登记与产物逐字节比对）` で、[第1ラウンドの経緯](records/第一轮的经过与结论都写在这一份记录里面.md) に記録した。' '' \
  '## 未完了のチェック' '' '| 番号 | 何が未完了か |' '|---|---|' \
  '| C1 (インスタンス識別子の幅が未登録) | 幅を登録する |' \
  '| C2 (成果物がシーケンス番号の使用ビット数を報告していない) | 成果物の末尾に使用ビット数を一行報告し、登録した幅と比べる |' \
  '| C3 (インスタンス識別子を上位ビットに入れた後、ラップアラウンドの瞬間の単体テストも、ラップアラウンド判定を壊す変異もない) | 赤になる単体テストを一つ加える：シーケンス番号がゼロに戻るときインスタンス識別子が一つ増え、その一行を壊すと赤になる。さらにラップアラウンド判定の比較演算子を反転させる変異を一つ加える |' \
  '| 短い行 | 初版の操作とロールバック |'
put .claude/kb/notes.md '# 说明页' '' '**中文** · [English](notes.en.md) · [日本語](notes.ja.md)' '' \
  '这一页只有一段：说明页记的是写译文时常见的三种写法差别，三份的段数要一样，内容量要对得上。'
put .claude/kb/notes.en.md '# Notes' '' '[中文](notes.md) · **English** · [日本語](notes.ja.md)' '' \
  'This page has a single paragraph: it records three common differences in how translations are written; the three versions must have the same number of paragraphs and matching amounts of content.'
put .claude/kb/notes.ja.md '# メモ' '' '[中文](notes.md) · [English](notes.en.md) · **日本語**' '' \
  'このページは一段だけである：訳文を書くときによくある三種類の書き方の違いを記す。三つの版の段数は同じで、内容量も見合っていなければならない。'
