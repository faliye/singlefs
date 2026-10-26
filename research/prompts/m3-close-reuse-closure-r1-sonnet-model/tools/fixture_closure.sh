#!/usr/bin/env bash
# V3：今天的引擎怎么处理函数位置参数。用法：fixture_closure.sh <仓根> <模型目录>
# 三份脚本：main.sh（"$@" 当命令的函数，调用处给命令词）、main_callback.sh（同，调用处第一个词不是命令）、main_text.sh（函数里 bash -c "$2"，调用处给串）
set -u
repo="$1"; model="$2"
for entry in scripts/main.sh scripts/main_callback.sh scripts/main_text.sh; do
  echo "== $entry"
  (cd "$model/fixtures" && python3 "$repo/scripts/gate/lib_function_units.py" closure . "$entry" 2>&1 | awk -F'\t' '$1=="refused" || $3 ~ /inner/ {print $1"\t"$2"\t"$3}' | cut -c1-160)
done
# 包装 `--` 之后的 bash -c "$3"（multi-host-run.sh 自证里 start_shared_driver 的形状）：今天既不拒算、也不把调用处的串连进来
echo "== scripts/main_after_wrapper.sh"
(cd "$model/fixtures" && python3 "$repo/scripts/gate/lib_function_units.py" closure . scripts/main_after_wrapper.sh 2>&1 | awk -F'\t' '$1=="refused" || $3 ~ /inner/ {print $1"\t"$2"\t"$3}' | cut -c1-160; echo "(退出码 ${PIPESTATUS[0]})")
