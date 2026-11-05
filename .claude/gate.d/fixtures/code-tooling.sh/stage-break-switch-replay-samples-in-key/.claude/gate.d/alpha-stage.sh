#!/usr/bin/env bash
# 样本小阶段：样本根的 verdict 文件写 red 就退 1，写 green 就退 0；复用三问的 change.sh 不动这一份，只改它的一份样本
# 弄坏开关：
#   ALPHA_STAGE_BREAK=flip-green  判绿的样本判成红
#       alpha-green（期望退出 0，实测 1）
#   ALPHA_STAGE_BREAK=swallow-red  判红的样本判成绿
#       alpha-red（期望退出 1，实测 0）
cd "${1:-.}" || exit 2
verdict="$(cat verdict)"
case "${ALPHA_STAGE_BREAK:-}" in flip-green) [[ "$verdict" == green ]] && verdict=red ;; swallow-red) [[ "$verdict" == red ]] && verdict=green ;; esac
echo "样本小阶段判：$verdict"
[[ "$verdict" == green ]]
