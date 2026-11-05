#!/usr/bin/env bash
# 样本：不保留权限的 cp 与长得像的别的写法，一处都不点名（cp -a 写在注释里也不算）
cp -dR --preserve=timestamps,links "$source" "$target"
cp --preserve=timestamps first second
cp -R source-tree target-tree
cp -P link copied-link && cp --parents a/b target
echo "  → 怎么办：别写 cp -a 或 cp -p，写 cp -R"
case_text="样本命令 cp -a records/a.md copied/"
scp -p host:first second; rsync -a source/ target/; mkdir -p target
