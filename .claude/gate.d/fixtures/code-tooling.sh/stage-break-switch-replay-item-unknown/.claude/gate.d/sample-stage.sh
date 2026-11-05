#!/usr/bin/env bash
# 样本小阶段：样本根的 verdict 文件写 red 就退 1，写 green 就退 0（项写错时不该复跑）
# 弄坏开关：
#   SAMPLE_STAGE_BREAK=flip-green  判绿的样本判成红
#       sample-green（期望退出 0，实测 1）
cd "${1:-.}" || exit 2
verdict="$(cat verdict)"
[[ "${SAMPLE_STAGE_BREAK:-}" == flip-green && "$verdict" == green ]] && verdict=red
echo "样本小阶段判：$verdict"
[[ "$verdict" == green ]]
