#!/usr/bin/env bash
# 样本脚本：divided.sh 拆出的部分文件，读 DIVIDED_BREAK；被 source 时平常跑设了就拒，直接起它不拦（库没有命令行入口）
if [[ "${BASH_SOURCE[0]}" != "$0" && -n "${DIVIDED_BREAK:-}" && "${1:-}" != --selftest ]]; then
  echo "  ✗ DIVIDED_BREAK 只给 --selftest 证红用，这一趟不是自证" >&2
  exit 2
fi
