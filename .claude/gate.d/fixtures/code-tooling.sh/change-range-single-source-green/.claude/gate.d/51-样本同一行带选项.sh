#!/usr/bin/env bash
# 样本不是真阶段；stage-owners 格要每个阶段认第一个参数当项目根，这一行只为它：ROOT="${1:-.}"
# 绿样本：同一行带着选项，配置键大小写不同也认
git -c core.quotePath=false log --all --diff-filter=D --format= --name-only -- research/results
