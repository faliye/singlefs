#!/usr/bin/env bash
# 产物 e99-normalized-2026-11-14.out 三行：前后各三个空格的一行；E7RESULT 与后面的空格之间隔着一个回车、行尾也有回车的一行；原样的一行。
# 去掉首尾空白、去掉回车之后三行都以 E7RESULT 加空格起头，页里整行抄的三行都在产物里逐字找得到：判绿。
# 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=result-lines-raw-prefix 不去空白就筛前缀，前后带空格的那一行收不进来；
# result-lines-prefilter-ignores-carriage-return 对有回车的产物也走「先筛含 E7RESULT 加一个空格的行」那一支，带回车的那一行收不进来：两条下判红。
set -e
mkdir -p research/results
printf '   E7RESULT name=indented arm=b count=4   \nE7RESULT\r name=carriage arm=c count=5\r\nE7RESULT name=plain arm=a count=3\n' > research/results/e99-normalized-2026-11-14.out
