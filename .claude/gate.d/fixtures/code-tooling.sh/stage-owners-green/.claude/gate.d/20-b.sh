#!/usr/bin/env bash
# gate-stage: 样本阶段 20-b.sh（一格，登记着）
# gate-cell: b1 判乙一
ROOT="${1:-.}"
case " $* " in *" --list "*) printf 'b1\t判乙一\n'; exit 0 ;; esac
cd "$ROOT" || exit 2
exit 0
