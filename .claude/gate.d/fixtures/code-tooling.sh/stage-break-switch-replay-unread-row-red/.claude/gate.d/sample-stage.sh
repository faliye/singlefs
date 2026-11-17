#!/usr/bin/env bash
# 样本小阶段：样本根的 verdict 文件写 red 就退 1，写 green 就退 0；头注里有一行开关写法读不出（值后面只隔一个空格接着写字）
# 弄坏开关：
#   SAMPLE_STAGE_BREAK=flip-green  判绿的样本判成红
#       sample-green（期望退出 0，实测 1）
#   SAMPLE_STAGE_BREAK=swallow-red  判红的样本判成绿
#       sample-red（期望退出 1，实测 0）
#   SAMPLE_STAGE_BREAK=one-space 值后面只隔一个空格，这一条读不出、悄悄不复跑
cd "${1:-.}" || exit 2
verdict="$(cat verdict)"
case "${SAMPLE_STAGE_BREAK:-}" in flip-green) [[ "$verdict" == green ]] && verdict=red ;; swallow-red) [[ "$verdict" == red ]] && verdict=green ;; esac
echo "样本小阶段判：$verdict"
[[ "$verdict" == green ]]
