#!/usr/bin/env bash
# 样本不是真阶段；stage-owners 格要每个阶段认第一个参数当项目根，这一行只为它：ROOT="${1:-.}"
# 绿样本：-z 下 git 不转义路径
git ls-files -z --cached --others --exclude-standard | sort -z -u
