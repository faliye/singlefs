#!/usr/bin/env bash
# 红样本：没开 set -e，cd 没接 || 也没接 &&（进不去就在调用方的目录里接着跑）
cd "$(dirname "$0")/../.."
echo "在 $PWD 里"
