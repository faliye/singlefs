#!/usr/bin/env bash
# 样本：经共用库拷的、拷源整个是变量的、仓根条目只出现在目的地里的、注释与出路文字里的，都不点名
# rsync -a crates "$1/"（注释里的不算）
python3 scripts/hosts/minimal-copy.py . "$1" --with scripts/tool.py=副本里起它
rsync -a "$template/" "$1/repo/"
cp -- "$1/notes.txt" "$1/crates/notes.txt"
echo "  → 怎么办：rsync -a crates 进副本是改前的写法（出路文字，不点名）"
