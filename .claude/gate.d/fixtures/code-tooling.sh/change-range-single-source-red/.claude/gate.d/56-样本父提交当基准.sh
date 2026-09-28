#!/usr/bin/env bash
# 红样本：写死上一个提交当基准
parent="$(git -c core.quotepath=false diff --name-only HEAD^ --)" || exit 1
