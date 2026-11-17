#!/usr/bin/env bash
# 样本小阶段：每喂一份样本先睡 2 秒（几条开关趟叠得上，复跑那一格数得出同时在跑几条），再照样本根的 verdict 文件判：写 red 退 1，写 green 退 0
# 弄坏开关：
#   SLOT_STAGE_BREAK=flip-green  判绿的样本判成红
#       slot-green（期望退出 0，实测 1）
#   SLOT_STAGE_BREAK=swallow-red  判红的样本判成绿
#       slot-red（期望退出 1，实测 0）
#   SLOT_STAGE_BREAK=invert-all  红绿都判反
#       slot-green、slot-red
#   SLOT_STAGE_BREAK=green-crashes  判绿的样本退 3
#       slot-green（期望退出 0，实测 3）
cd "${1:-.}" || exit 2
sleep 2
verdict="$(cat verdict)"
case "${SLOT_STAGE_BREAK:-}" in
  flip-green) [[ "$verdict" == green ]] && verdict=red ;;
  swallow-red) [[ "$verdict" == red ]] && verdict=green ;;
  invert-all) if [[ "$verdict" == green ]]; then verdict=red; else verdict=green; fi ;;
  green-crashes) if [[ "$verdict" == green ]]; then echo "样本小阶段退 3"; exit 3; fi ;;
esac
echo "样本小阶段判：$verdict"
[[ "$verdict" == green ]]
