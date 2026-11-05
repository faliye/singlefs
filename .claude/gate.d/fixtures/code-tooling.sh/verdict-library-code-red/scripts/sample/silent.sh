#!/usr/bin/env bash
# 弄坏开关 SAMPLE_BREAK=<项>（只给自证用）：
#   SAMPLE_BREAK=silent-run-tolerated  取不到库时照改前往下跑
#     自证「取不到库：退 2」
if ! library="$(sample_library)"; then
  echo "  ! 判定库没处放：这一趟每一条都照跑"
fi
echo "  ! 这一趟一行都不复用、也不记"
if [[ "${SAMPLE_MODE:-}" == unlisted-mode ]]; then
  echo "  ! 全部现判、不记"
fi
if [[ "${SAMPLE_BREAK:-}" == silent-run-tolerated ]]; then
  exit 2
else
  echo "  ! 没有判定库：照跑"
fi
