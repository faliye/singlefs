#!/usr/bin/env bash
# 复跑行点名的脚本只有一部分在：e92_sample 里只有 driver.py（gone_driver.py、removed.py 不造），e91_sample 里有 driver.py、装置目录 e91_gone 不造。
set -e
mkdir -p scripts/experiment-apparatus/e92_sample scripts/experiment-apparatus/e91_sample
for script_path in e92_sample/driver.py e91_sample/driver.py; do
  printf '%s\n' '# 样本里的占位脚本，不会被跑' > "scripts/experiment-apparatus/$script_path"
done
