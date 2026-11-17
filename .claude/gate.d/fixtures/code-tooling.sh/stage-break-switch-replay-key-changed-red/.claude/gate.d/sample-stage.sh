#!/usr/bin/env bash
# 样本小阶段：样本根的 verdict 文件写 red 就退 1，写 green 就退 0；每被喂一次，往这一棵树里自己的样本目录 fixtures/sample-stage.sh/ 下追加一行（键里的样本摘要在跑的过程中变了）
# 弄坏开关：
#   SAMPLE_STAGE_BREAK=flip-green  判绿的样本判成红
#       sample-green（期望退出 0，实测 1）
#   SAMPLE_STAGE_BREAK=swallow-red  判红的样本判成绿
#       sample-green（这一条点名点错了）
echo touched >> "${0%/*}/fixtures/sample-stage.sh/touched.txt"
cd "${1:-.}" || exit 2
verdict="$(cat verdict)"
case "${SAMPLE_STAGE_BREAK:-}" in flip-green) [[ "$verdict" == green ]] && verdict=red ;; swallow-red) [[ "$verdict" == red ]] && verdict=green ;; esac
echo "样本小阶段判：$verdict"
[[ "$verdict" == green ]]
