#!/usr/bin/env bash
# checker-tier-crates-mutation-replay 的判定库放在被判那棵树的 git common-dir 里（行键按函数单元算，不靠 git 列文件）：
# 样本拷进临时目录之后在那里建一个空仓，不提交，未跟踪的文件照样列得出来。
set -e
git init -q .
