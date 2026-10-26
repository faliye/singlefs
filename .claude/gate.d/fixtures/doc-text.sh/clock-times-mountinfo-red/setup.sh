#!/usr/bin/env bash
# 红样本现场：形态与 mountinfo 一行相近、其实是真钟点的三行，mountinfo 那条上下文规则不许放过它们：
# 前面有两个挂载号、后面却不是「 /根路径 /挂载点」，与后面是路径、前面只有一个数；
# 真钟点的冒号由变量拼进去——否则门禁 doc-text.sh 扫样本现场时会先把这份 setup.sh 判红，多红几行。
# 绿的一面（真 mountinfo 行不判）在 clock-times-green 的 records/colon-pairs.md。
set -e
colon=:
mkdir -p records
printf '%s\n' "26 32 0${colon}23 开始写，后面不是路径" \
  "行 32 0${colon}23 / /sys rw 前面只有一个数" \
  "26 32 10${colon}23 /usr 的会，挂载点那一段不是路径" > records/mountinfo-near.md
