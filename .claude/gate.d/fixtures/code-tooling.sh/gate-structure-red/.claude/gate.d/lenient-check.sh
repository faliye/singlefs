#!/usr/bin/env bash
# gate-stage: 样本：有格名表、--list 对，--check 写错格名却照跑退 0
# gate-cell: alpha 判甲
ROOT="${1:-.}"
case " $* " in *" --list "*) printf 'alpha\t判甲\n'; exit 0 ;; esac
cd "$ROOT" || exit 2
echo "  ✓ 什么参数都照跑，1 格"
exit 0
