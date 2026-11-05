#!/usr/bin/env bash
# 样本：库名只当名字，本地不建不读不写；自证不读不写真服务
# 改前的样子：取不到库就全部现判、不记（只是说明，不判）
# 弄坏开关 SAMPLE_BREAK=<项>（只给自证用）：
#   SAMPLE_BREAK=silent-run-tolerated  取不到库时照改前往下跑
#     自证「取不到库：退 2」
echo "  ✗ 按条复用失灵：全部现判、一条都不记库，这一格判红"
common="$(git rev-parse --git-common-dir)"   # 没有判定库就退出，不照跑（行尾注释，不判）
echo "$common"
if [[ "${SAMPLE_BREAK:-}" == silent-run-tolerated ]]; then
  echo "  ! 判定库没处放：照改前全部现判、不记"
  if [[ -n "$common" ]]; then
    echo "  ! 这一趟一行都不复用、也不记"
  fi
fi
