#!/usr/bin/env bash
# 绿样本：同一行带着选项，配置键大小写不同也认
git -c core.quotePath=false log --all --diff-filter=D --format= --name-only -- research/results
