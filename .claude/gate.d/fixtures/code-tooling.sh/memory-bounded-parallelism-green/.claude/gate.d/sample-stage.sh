#!/usr/bin/env bash
# gate-stage: 样本：起判法模块，代码行里写出它的路径（这一格照它找入口罩着的文件）
ROOT="${1:-.}"
python3 "$ROOT/scripts/mutation/sample_rows.py"
