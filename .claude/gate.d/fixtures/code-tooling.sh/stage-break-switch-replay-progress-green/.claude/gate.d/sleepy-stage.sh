#!/usr/bin/env bash
# 样本小阶段：每喂一份样本先睡 2 秒（比样本根的进度间隔 1 秒长，复跑那一格中途要报进度），再照样本根的 verdict 文件判：写 red 退 1，写 green 退 0
# 弄坏开关：
#   SLEEPY_STAGE_BREAK=flip-green  判绿的样本判成红
#       sleepy-green（期望退出 0，实测 1）
#   SLEEPY_STAGE_BREAK=swallow-red  判红的样本判成绿
#       sleepy-red（期望退出 1，实测 0）
cd "${1:-.}" || exit 2
sleep 2
verdict="$(cat verdict)"
case "${SLEEPY_STAGE_BREAK:-}" in flip-green) [[ "$verdict" == green ]] && verdict=red ;; swallow-red) [[ "$verdict" == red ]] && verdict=green ;; esac
echo "样本小阶段判：$verdict"
[[ "$verdict" == green ]]
