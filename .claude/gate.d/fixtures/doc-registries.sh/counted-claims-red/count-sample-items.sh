#!/usr/bin/env bash
# 样本自带的假数数命令（经 bash 起）：按参数打一个固定的数，或故意退非 0、打不是整数的字；不读仓里任何东西
case "${1:-}" in
  invariants) echo 9 ;;
  items) echo 3 ;;
  broken) echo 4; echo "样本：故意退 3" >&2; exit 3 ;;
  garbage) echo "五个" ;;
  right) echo 2 ;;
  *) echo "样本：认不出的参数 ${1:-}" >&2; exit 2 ;;
esac
