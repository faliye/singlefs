#!/usr/bin/env bash
# gate-stage: 样本阶段 10-a.sh（两格，逐格登记着）
# gate-cell: a1 判甲一
# gate-cell: a2 判甲二
ROOT="${1:-.}"
case " $* " in *" --list "*) printf 'a1\t判甲一\na2\t判甲二\n'; exit 0 ;; esac
cd "$ROOT" || exit 2
exit 0
