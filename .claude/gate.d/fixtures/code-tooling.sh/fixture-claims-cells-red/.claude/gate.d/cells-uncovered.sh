#!/usr/bin/env bash
# gate-stage: 样本阶段：三格的格名表，样本目录按 <格>-red / <格>-green 起名，有三格各缺一边
# gate-cell: alpha 判甲
# gate-cell: beta 判乙
# gate-cell: gamma 判丙
ROOT="${1:-.}"
cd "$ROOT" || exit 2
exit 0
