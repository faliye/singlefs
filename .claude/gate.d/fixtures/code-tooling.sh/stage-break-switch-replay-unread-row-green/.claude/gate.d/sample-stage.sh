#!/usr/bin/env bash
# 样本小阶段：样本根的 verdict 文件写 red 就退 1，写 green 就退 0；头注里有句中提到开关的、占位写法的，代码里有一行赋值，都不算读不出
# 弄坏开关 SAMPLE_STAGE_BREAK=flip-green 下判绿的样本判成红（句中提到，不算一条）
#   SAMPLE_STAGE_BREAK=<项>  占位写法，不算一条
# 弄坏开关：
#   SAMPLE_STAGE_BREAK=flip-green  判绿的样本判成红
#       sample-green（期望退出 0，实测 1）
#   SAMPLE_STAGE_BREAK=swallow-red  判红的样本判成绿
#       sample-red（期望退出 1，实测 0）
cd "${1:-.}" || exit 2
SAMPLE_STAGE_BREAK="${SAMPLE_STAGE_BREAK:-}"
verdict="$(cat verdict)"
case "$SAMPLE_STAGE_BREAK" in flip-green) [[ "$verdict" == green ]] && verdict=red ;; swallow-red) [[ "$verdict" == red ]] && verdict=green ;; esac
echo "样本小阶段判：$verdict"
[[ "$verdict" == green ]]
