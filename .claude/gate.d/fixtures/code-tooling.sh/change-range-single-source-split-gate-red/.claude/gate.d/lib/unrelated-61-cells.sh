#!/usr/bin/env bash
# 样本：不在任何一道门禁的族里的库，同样的写法不扫
base="$(git merge-base HEAD origin/main)"
listed="$(git diff --name-only "$base")"
