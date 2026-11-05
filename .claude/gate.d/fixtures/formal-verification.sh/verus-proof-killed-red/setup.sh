#!/usr/bin/env bash
# 样本 verus-proof-killed-red 的现场：样本拷进临时目录之后，把登记表 release 行第五列（安装目录）改成这里的替身安装 verus-stand-in/ 的绝对路径
# （verus-proof-check.py 拿 ${XDG_DATA_HOME:-~/.local/share} 与第五列拼路径，第五列是绝对路径时就是它自己），替身 verus 补上执行位。
# 设了 SINGLEFS_VERUS_ROOT 的环境里它盖过登记表、起的是真 Verus，这份样本判错（与别的 Verus 样本起真 Verus 同一个前提，写在阶段文件头的样本一段）。
set -euo pipefail
chmod +x verus-stand-in/verus
awk -F'\t' -v OFS='\t' -v install="$PWD/verus-stand-in" '$1 == "release" { $5 = install } { print }' .claude/gate.d/verus-pins.tsv > .claude/gate.d/verus-pins.tsv.new
mv .claude/gate.d/verus-pins.tsv.new .claude/gate.d/verus-pins.tsv
# 回读：release 行第五列真改成了替身安装
awk -F'\t' -v install="$PWD/verus-stand-in" '$1 == "release" && $5 == install { found = 1 } END { exit !found }' .claude/gate.d/verus-pins.tsv
