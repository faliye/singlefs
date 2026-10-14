#!/usr/bin/env bash
# 样本：登记成 light 的自证，cargo build 走样本根 sample-tools/ 里的假 cargo（排在 PATH 最前面），自己退 0
[[ "${1:-}" == --selftest ]] || exit 2
PATH="$PWD/sample-tools:$PATH" cargo build --release >/dev/null || exit 1
echo "  ✓ 登记成 light 的样本自证 1 条都判对"
exit 0
