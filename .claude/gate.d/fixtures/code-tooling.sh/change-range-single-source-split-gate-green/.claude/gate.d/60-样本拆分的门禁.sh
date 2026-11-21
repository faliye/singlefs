#!/usr/bin/env bash
# 样本不是真阶段：入口里定义带 quotepath 的包装函数，列路径的调用在它拆出的库里
git_quoted() { git -c core.quotepath=false "$@"; }
