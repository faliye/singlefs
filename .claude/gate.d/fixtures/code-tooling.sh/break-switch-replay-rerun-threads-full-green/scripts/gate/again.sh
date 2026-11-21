#!/usr/bin/env bash
# 样本脚本：核每一趟拿到的 SINGLEFS_THREAD_CAP（样本根 .break-switch-replay-jobs 给线程 8；一份两条开关，开关趟那一段外层 3 条 × 里层 2）：
# 基准趟要是 8，开关趟的头一趟要是 2，AGAIN_BREAK=late 重跑的那一趟（样本根里已有 .seen-late）要是 8；哪一趟不对就在样本根记下 .cap-wrong，
# 此后每一趟都多打一行没点名的失败行（各趟写上自己的开关值，不被认成汇总行）。AGAIN_BREAK=late 的头一趟没红、退 0，开关复跑要重跑它；重跑那一趟红在点名的失败行上。
# 弄坏开关：
#   AGAIN_BREAK=late  判迟到那一处不判
#       again-late（「✗ 自检：判迟到那一格」）
#   AGAIN_BREAK=plain  判照常那一处不判
#       again-plain（「✗ 自检：判照常那一格」）
if [[ "${1:-}" == --selftest ]]; then
  cap="${SINGLEFS_THREAD_CAP:-读不到}"
  case "${AGAIN_BREAK:-}" in
    "") wanted=8 ;;
    late) if [[ -e .seen-late ]]; then wanted=8; else wanted=2; fi ;;
    *) wanted=2 ;;
  esac
  [[ "$cap" == "$wanted" ]] || : > .cap-wrong
  [[ ! -e .cap-wrong ]] || echo "  ✗ 自检：这一趟核到的线程数不对（这一趟：${AGAIN_BREAK:-不设开关}，核到 ${cap}、要 ${wanted}）"
  case "${AGAIN_BREAK:-}" in
    late)
      if [[ ! -e .seen-late ]]; then
        : > .seen-late
        echo "  ✓ 自检通过（查了 2 格）"
        exit 0
      fi
      echo "  ✗ 自检：判迟到那一格：应当红，实际绿"
      exit 1
      ;;
    plain)
      echo "  ✗ 自检：判照常那一格：应当红，实际绿"
      exit 1
      ;;
  esac
  [[ ! -e .cap-wrong ]] || exit 1
  echo "  ✓ 自检通过（查了 2 格）"
fi
