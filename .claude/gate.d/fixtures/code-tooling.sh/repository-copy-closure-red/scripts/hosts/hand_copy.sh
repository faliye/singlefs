#!/usr/bin/env bash
# 样本：手抄的拷贝清单（四处都该点名）
rsync -a --exclude target Cargo.toml Cargo.lock .cargo crates "$1/"
cp -- "$repository_root/.claude/heavy-thread-default" "$1/.claude/"
rsync -a --exclude target ./ "$1/workspace/"
cp -R "$REPOSITORY_ROOT" "$1/whole"
echo "  → 怎么办：rsync -a crates 进副本是改前的写法（出路文字，不点名）"
cp -- "$1/notes.txt" "$1/crates/notes.txt"
