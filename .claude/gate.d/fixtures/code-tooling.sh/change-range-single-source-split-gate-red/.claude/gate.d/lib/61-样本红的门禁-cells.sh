#!/usr/bin/env bash
# 样本：拆出的库里自己取 merge-base、列路径不带 quotepath
base="$(git merge-base HEAD origin/main)"
listed="$(git diff --name-only "$base")"
