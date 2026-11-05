#!/usr/bin/env bash
# 样本：带保留权限的 cp 六种写法（每行都该点名）
cp -a "$source" "$target"
cp -rp "$source/file" "$target/"
cp --archive source-tree target-tree
if true; then cp --preserve=mode,timestamps first second; fi
listing=$(cp --preserve first second && ls)
/bin/cp -p first second
