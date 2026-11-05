#!/usr/bin/env bash
# 样本自带的假数数命令（经 bash 起）：按参数打固定的数或固定的几行；不读仓里任何东西
case "${1:-}" in
  invariants) echo 6 ;;
  rows) printf 'I-1\nI-2\nI-3\nI-4\nI-5\nI-6\n' ;;
  items) echo "  4  " ;;
  *) echo "样本：认不出的参数 ${1:-}" >&2; exit 2 ;;
esac
