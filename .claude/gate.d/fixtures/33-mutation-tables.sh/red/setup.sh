#!/usr/bin/env bash
# 一张读不了的变异表：同名的实验二进制在，表却不是 UTF-8。现场在临时目录里现造，不把非 UTF-8 的文件摆进仓里。
# 锚点检查读它时 UnicodeDecodeError，老写法整段崩掉、一行报告都不剩；要的是这一份判红、别的照判。
set -e
printf 'fn main() {}\n' > research/e7-index-bench/src/bin/e3_latin1.rs
printf 'M1_\xe9chantillon\tfn main() {}\tfn main() { }\n' > research/mutations/e3_latin1.tsv
