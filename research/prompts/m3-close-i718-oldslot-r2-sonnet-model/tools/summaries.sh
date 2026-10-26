#!/usr/bin/env bash
# 用法：summaries.sh <输出目录> <落点目录>：把输出目录里各格的 .out 汇成 summary-rows.txt 与 summary-scans.txt（同一份输入次序：文件名排序）。
set -eu
OUT="$1"; DEST="$2"; HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
files=$(ls "$OUT"/S*-*.out | sort)
python3 "$HERE/summary.py" $files > "$DEST/summary-rows.txt"
python3 "$HERE/scan-summary.py" $files > "$DEST/summary-scans.txt"
