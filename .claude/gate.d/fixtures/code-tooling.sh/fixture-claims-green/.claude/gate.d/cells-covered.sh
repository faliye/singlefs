#!/usr/bin/env bash
# gate-stage: 样本阶段：两格的格名表，样本目录按 <格>-red / <格>-green 起名（没有裸 red / green），每格一红一绿
# gate-cell: alpha 判甲
# gate-cell: beta 判乙
ROOT="${1:-.}"
cd "$ROOT" || exit 2
exit 0
