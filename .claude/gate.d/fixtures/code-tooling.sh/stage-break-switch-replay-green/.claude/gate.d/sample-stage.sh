#!/usr/bin/env bash
# 样本小阶段：样本根的 verdict 文件写 red 就退 1，写 green 就退 0
# 弄坏开关：
#   SAMPLE_STAGE_BREAK=flip-green  判绿的样本判成红
#       sample-green（期望退出 0，实测 1）
#   SAMPLE_STAGE_BREAK=swallow-red  判红的样本判成绿
#       sample-red（期望退出 1，实测 0）
#   SAMPLE_STAGE_BREAK=noop  什么都不弄坏
#       （登记在豁免表，这一格不复跑它）
cd "${1:-.}" || exit 2
verdict="$(cat verdict)"
case "${SAMPLE_STAGE_BREAK:-}" in flip-green) [[ "$verdict" == green ]] && verdict=red ;; swallow-red) [[ "$verdict" == red ]] && verdict=green ;; esac
echo "样本小阶段判：$verdict"
[[ "$verdict" == green ]]
