#!/usr/bin/env bash
# 样本：拆出的库，经入口里定义的包装函数列路径
listed="$(git_quoted diff --name-only "$base")" || exit 1
