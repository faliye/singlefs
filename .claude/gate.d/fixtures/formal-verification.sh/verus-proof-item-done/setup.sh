#!/usr/bin/env bash
# 样本现场：判定库的名字取被判那棵树的 git common-dir，样本拷进临时目录之后不是 git 工作树，判法退 2、一项都不判；这里先 git init。
set -euo pipefail
git init -q .
