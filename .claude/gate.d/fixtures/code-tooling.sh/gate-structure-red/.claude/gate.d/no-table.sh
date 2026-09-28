#!/usr/bin/env bash
# gate-stage: 样本：文件头没有格名表的阶段（老样子一个大函数，不认 --list）
ROOT="${1:-.}"
cd "$ROOT" || exit 2
echo "  ✓ 什么参数都照跑，1 项"
exit 0
