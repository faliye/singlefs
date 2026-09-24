#!/usr/bin/env bash
# 绿样本：-z 下 git 不转义路径
git ls-files -z --cached --others --exclude-standard | sort -z -u
