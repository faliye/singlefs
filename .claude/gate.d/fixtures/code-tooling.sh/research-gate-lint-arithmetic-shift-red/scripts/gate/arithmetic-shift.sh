#!/usr/bin/env bash
# 红样本：算术里的 << 是移位，不是 heredoc 起始；它们后面这一行裸 cd 没接 || 也没接 &&，要判。真 heredoc 正文里的 cd 不判
(( shift_bits = 1 << width ))
mask=$(( 1 << bits ))
cd "$(dirname "$0")"
cat <<'TEXT'
cd /nowhere
TEXT
echo "在 $PWD 里"
