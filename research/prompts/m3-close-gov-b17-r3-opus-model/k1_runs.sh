#!/usr/bin/env bash
# K1 第三条路的四条时间线：A 原样 lib_function_units.py 两趟；B 改过的（E1）两趟；C 第一趟改过的、第二趟原样；D 同 C 但两趟都开 judging-files-included（改前：那两份进判法单元指纹）。
# 每条时间线：第一趟在 S0 上跑，第二趟把 add 的函数体改坏（S1：点名测试在没改坏的源码上就红）再跑。用法：k1_runs.sh <模型目录> <lfu0 副本> <lfu1 副本> <工作目录>
set -euo pipefail
model="$1"; lfu0="$2"; lfu1="$3"; work="$4"
run_line() {  # 时间线 第一趟副本 第二趟副本 开关
  local name="$1" first="$2" second="$3" switch="$4"
  local base="$work/$name"
  rm -rf "$base"; mkdir -p "$base"
  echo "name=timeline id=$name first=$(basename "$first") second=$(basename "$second") break=${switch:--}"
  python3 -I "$model/k1_timeline.py" --scripts "$first" --fixture "$base/repo" --state S0 --library "$base/lib" --work "$base/work-1" --break "$switch" | sed "s/^/$name.1 /"
  python3 -I "$model/k1_timeline.py" --scripts "$second" --fixture "$base/repo" --state S1 --library "$base/lib" --work "$base/work-2" --break "$switch" | sed "s/^/$name.2 /"
}
run_line A "$lfu0" "$lfu0" ""
run_line B "$lfu1" "$lfu1" ""
run_line C "$lfu1" "$lfu0" ""
run_line D "$lfu1" "$lfu0" "judging-files-included"
