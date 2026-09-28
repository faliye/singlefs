#!/usr/bin/env bash
# 红样本的现场，先 mutation-tables 那一格的、再 reading-discipline 那一格的（后者要把前面造好的文件一起提交进基准）。
#
# mutation-tables：一张读不了的变异表：同名的实验二进制在，表却不是 UTF-8。现场在临时目录里现造，不把非 UTF-8 的文件摆进仓里。
# 锚点检查读它时 UnicodeDecodeError，老写法整段崩掉、一行报告都不剩；要的是这一份判红、别的照判。
# 变异表末尾追加一行「必须红的测试名」空着的（第 10 行）：六段里只有替换文可以空，这一行要判不成形。在这里现写，
# 是因为那一段空着时行尾是一个制表符，摆进仓里的文件会被编辑工具与去行尾空白的习惯吃掉，变成五段、判的就不是这一样了。
#
# reading-discipline：豁免表进了基准提交，再把基准里那一版的 blob 从对象库里删掉——git 取不到基准那一版，
# 「只缩不涨」必须判红，不许当成「基准里还没有这张表、初次登记」放过去。
set -euo pipefail
printf 'M_测试名空着\tcrates/demo/src/lib.rs\tpub fn published_generation(\tpub fn published_generation_emptied(\t-p demo --lib\t\n' >> crates/mutations.tsv
printf 'fn main() {}\n' > research/e7-index-bench/src/bin/e3_latin1.rs
printf 'M1_\xe9chantillon\tfn main() {}\tfn main() { }\n' > research/mutations/e3_latin1.tsv
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
git add -A
git commit -qm '样本基准：一张挂在认不出的欠账上的豁免表'
blob="$(git rev-parse HEAD:.claude/gate.d/experiment-seed-fold-lag.tsv)"
rm -f ".git/objects/${blob:0:2}/${blob:2}"
