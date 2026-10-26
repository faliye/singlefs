#!/usr/bin/env bash
# 编号不数链接目标里的，链接文字照数：
# experiments 索引与 E14 那一页：译文链的是 translation-paths.json 登记的译文文件名，中文文件名里的 I-3.1 在日语文件名里写成 I-3-1（数出来是 I-3）、
#   英语文件名里写成小写；链接文字与正文三份都写 I-3.1。链接目标归名之后三份相同，编号只差在链接目标里，这两组不报。
# decisions 索引：日语那份的链接文字丢了 D5（链接目标三份相同、里面没有编号），照报。判红，只点名 decisions 一组。
set -e
put() { mkdir -p "$(dirname "$1")"; printf '%s\n' "${@:2}" > "$1"; }
put .claude/trilingual-documents \
  '.claude/kb/**/*.md  # 样本里的 kb 页，中文为默认版本'
put .claude/kb/translation-paths.json \
  '{' \
  '  ".claude/kb/experiments/14-I-3.1的判别力.en.md": ".claude/kb/experiments/14-discriminative-power-of-i-3-1.en.md",' \
  '  ".claude/kb/experiments/14-I-3.1的判别力.ja.md": ".claude/kb/experiments/14-I-3-1の判別力.ja.md"' \
  '}'
put .claude/kb/experiments.md '# 实验索引' '' '| 实验 | 页 |' '|---|---|' \
  '| E14（I-3.1 的判别力） | [14-I-3.1的判别力.md](experiments/14-I-3.1的判别力.md) |'
put .claude/kb/experiments.en.md '# Experiment index' '' '| Experiment | Page |' '|---|---|' \
  '| E14 (discriminative power of I-3.1) | [14-Discriminative power of I-3.1.en.md](experiments/14-discriminative-power-of-i-3-1.en.md) |'
put .claude/kb/experiments.ja.md '# 実験索引' '' '| 実験 | ページ |' '|---|---|' \
  '| E14（I-3.1 の判別力） | [14-I-3.1の判別力.ja.md](experiments/14-I-3-1の判別力.ja.md) |'
put .claude/kb/experiments/14-I-3.1的判别力.md '# I-3.1 的判别力' '' \
  '**中文** · [English](14-discriminative-power-of-i-3-1.en.md) · [日本語](14-I-3-1の判別力.ja.md)' '' 'I-3.1 对增量记账没有判别力。'
put .claude/kb/experiments/14-discriminative-power-of-i-3-1.en.md '# Discriminative power of I-3.1' '' \
  '[中文](14-I-3.1的判别力.md) · **English** · [日本語](14-I-3-1の判別力.ja.md)' '' 'I-3.1 has no discriminative power on incremental accounting.'
put .claude/kb/experiments/14-I-3-1の判別力.ja.md '# I-3.1 の判別力' '' \
  '[中文](14-I-3.1的判别力.md) · [English](14-discriminative-power-of-i-3-1.en.md) · **日本語**' '' 'I-3.1 は増分記帳に対して判別力を持たない。'
put .claude/kb/decisions.md '# 决策索引' '' '见 [D5（快照）](decisions/05-快照.md)。'
put .claude/kb/decisions.en.md '# Decision index' '' 'See [D5 (snapshots)](decisions/05-快照.md).'
put .claude/kb/decisions.ja.md '# 決定索引' '' '[スナップショットの決定](decisions/05-快照.md) を参照。'
