#!/usr/bin/env bash
# 样本：必红的一条自证——并行跑的时候它的退出码不许被吃掉，它的输出要原样列出来
[[ "${1:-}" == --selftest ]] || exit 2
echo "  ✗ b 的自证判错：样本故意红"
echo "     → 怎么办：这是 runner-red 的必红样本，不用修"
exit 1
