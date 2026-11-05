#!/usr/bin/env bash
# 纯 Python / bash 装置：复跑行点名的脚本由这里造出来（样本里不放 .py / .sh），页上的几种写法点名的脚本与装置目录都在。
set -e
mkdir -p scripts/experiment-apparatus/e94_sample scripts/experiment-apparatus/e93_sample
for script_path in e94_sample/driver.py e94_sample/test_driver.py e94_sample/compile.sh e93_sample/full_run.py; do
  printf '%s\n' '# 样本里的占位脚本，不会被跑' > "scripts/experiment-apparatus/$script_path"
done
