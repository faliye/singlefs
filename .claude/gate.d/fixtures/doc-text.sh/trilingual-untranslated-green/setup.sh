#!/usr/bin/env bash
# 已译好的一组：英语、日语两份只在该照中文原样留的段里带中文，每一种段各一行、各自都过门槛（英语 15 个以上汉字，日语 3 个以上日语不用的简体字）：
# 「」、"…"、“…”、『』、套着的「「」」、反引号、链接目标、带 ASCII 的路径、跨行的 HTML 注释、列表项里缩进的围栏代码块；
# 另有英语正好 14 个汉字的一行（候选名甲乙丙丁这一类）、日语正好 2 个的一行（较、记，门槛下面一档，判绿）。
# 术语表那一组照设计三语并列，英语、日语里都有过门槛的中文，登记在排除表里不判。判绿。
set -e
put() { mkdir -p "$(dirname "$1")"; printf '%s\n' "${@:2}" > "$1"; }
put .claude/trilingual-documents \
  'README.md  # 样本里的仓根说明，中文为默认版本' \
  '.claude/kb/*.md  # 样本里的 kb 页，中文为默认版本'
put .claude/gate.d/trilingual-untranslated-exempt.tsv \
  '# 样本里的排除登记：术语表整份照设计三语并列' \
  $'.claude/kb/translation-glossary.md\t样本里的术语表，表的中文列与说明列在三份里都是同一个中文原串'
put README.md '# 样本' '' '**中文** · [English](README.en.md) · [日本語](README.ja.md)' '' \
  '用户原话「这一格只判这次改动里新加的行，没改动过的旧行一律不报」。' \
  '说明 "门禁只看这次改动里新加的行，别的会话写的旧行不归它" 照原样留。' \
  '回复 “这一条照中文原样留着，因为它是用户在弹窗里选的那一项” 照留。' \
  '标题 『发布之前推抬回退下界的空发布没有检查，这一条还没立』 是简称。' \
  '套着的 「外层说明「这一格只判这次改动里新加的行」外层说明照留」 引文。' \
  '变异行 `E183（容量斜率读缓存小盘准入与清扫阈值的代价数）` 以中文名开头。' \
  '见 [第一轮](records/第一轮的经过与结论都写在这一份记录里面.md)。' \
  '细节见 records/样本第二轮的经过与结论都写在这一份记录里面.md。' \
  '<!-- 这一段说明照中文原样留着，' '不进译文的正文，读的人看不到它，这里还有一行 -->' \
  '- 照录的旧文：' '' '    ```text' '    这一段是照录的旧文件原文，没改动过的旧行一律不报，别的会话写的也不归这一格' '    ```' '' \
  '候选甲乙丙丁戊己庚辛壬癸子丑寅卯比过一遍。' '候选的比较记在记录里，结果之后再看。'
put README.en.md '# Sample' '' '[中文](README.md) · **English** · [日本語](README.ja.md)' '' \
  'The user said 「这一格只判这次改动里新加的行，没改动过的旧行一律不报」 verbatim.' \
  'The note "门禁只看这次改动里新加的行，别的会话写的旧行不归它" is kept as written.' \
  'The reply “这一条照中文原样留着，因为它是用户在弹窗里选的那一项” stays.' \
  'The heading 『发布之前推抬回退下界的空发布没有检查，这一条还没立』 is the short name.' \
  'Nested 「外层说明「这一格只判这次改动里新加的行」外层说明照留」 quote.' \
  'The mutation row `E183（容量斜率读缓存小盘准入与清扫阈值的代价数）` starts with the Chinese name.' \
  'See [round one](records/第一轮的经过与结论都写在这一份记录里面.md).' \
  'Details are in records/样本第二轮的经过与结论都写在这一份记录里面.md.' \
  '<!-- 这一段说明照中文原样留着，' '不进译文的正文，读的人看不到它，这里还有一行 -->' \
  '- Old text recorded verbatim:' '' '    ```text' '    这一段是照录的旧文件原文，没改动过的旧行一律不报，别的会话写的也不归这一格' '    ```' '' \
  'Candidates 甲 乙 丙 丁 戊 己 庚 辛 壬 癸 子 丑 寅 卯 were compared.' 'The comparison of candidates is recorded; the results are read later.'
put README.ja.md '# サンプル' '' '[中文](README.md) · [English](README.en.md) · **日本語**' '' \
  'ユーザーの原文は「这一格只判这次改动里新加的行，没改动过的旧行一律不报」である。' \
  '注記 "门禁只看这次改动里新加的行，别的会话写的旧行不归它" はそのまま残す。' \
  '返答 “这一条照中文原样留着，因为它是用户在弹窗里选的那一项” は残す。' \
  '見出し 『发布之前推抬回退下界的空发布没有检查，这一条还没立』 は略称である。' \
  '入れ子 「外层说明「这一格只判这次改动里新加的行」外层说明照留」 の引用。' \
  '変異行 `E183（容量斜率读缓存小盘准入与清扫阈值的代价数）` は中国語名で始まる。' \
  '[第一ラウンド](records/第一轮的经过与结论都写在这一份记录里面.md) を参照。' \
  '詳細は records/样本第二轮的经过与结论都写在这一份记录里面.md を参照。' \
  '<!-- 这一段说明照中文原样留着，' '不进译文的正文，读的人看不到它，这里还有一行 -->' \
  '- そのまま記録した旧文：' '' '    ```text' '    这一段是照录的旧文件原文，没改动过的旧行一律不报，别的会话写的也不归这一格' '    ```' '' \
  '候補甲乙丙丁戊己庚辛壬癸子丑寅卯を比べた。' '候補の比较は记録に残し、結果は後で見る。'
put .claude/kb/translation-glossary.md '# 翻译术语表' '' '**中文** · [English](translation-glossary.en.md) · [日本語](translation-glossary.ja.md)' '' \
  '| 中文 | English | 日本語 | 说明 / Note / 注 |' '|---|---|---|---|' \
  '| 对拍 | differential | 突き合わせ | 两条互不共享代码的路子算同一件事，再逐项对比结果<br>two code paths compute the same thing<br>二つの経路で同じものを計算する |'
put .claude/kb/translation-glossary.en.md '# Translation glossary' '' '[中文](translation-glossary.md) · **English** · [日本語](translation-glossary.ja.md)' '' \
  '| 中文 | English | 日本語 | 说明 / Note / 注 |' '|---|---|---|---|' \
  '| 对拍 | differential | 突き合わせ | 两条互不共享代码的路子算同一件事，再逐项对比结果<br>two code paths compute the same thing<br>二つの経路で同じものを計算する |'
put .claude/kb/translation-glossary.ja.md '# 翻訳用語集' '' '[中文](translation-glossary.md) · [English](translation-glossary.en.md) · **日本語**' '' \
  '| 中文 | English | 日本語 | 说明 / Note / 注 |' '|---|---|---|---|' \
  '| 对拍 | differential | 突き合わせ | 两条互不共享代码的路子算同一件事，再逐项对比结果<br>two code paths compute the same thing<br>二つの経路で同じものを計算する |'
