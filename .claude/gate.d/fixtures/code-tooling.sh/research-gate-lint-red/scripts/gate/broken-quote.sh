#!/usr/bin/env bash
# 红样本：少了 .. 后面那个引号（搬迁时替换吃掉的那一种）
ROOT="$(cd "$(dirname "$0")/../.. && pwd)"
echo "$ROOT"
