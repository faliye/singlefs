#!/usr/bin/env bash
# K1 第三条路续：四趟（S0 建、S1 把 add 改坏、S2 改回、S3 再改坏），第一趟用改过的 lib_function_units.py（E1），之后三趟用原样的。
# E：原样驱动、两份进判法单元指纹（judging-files-included）；F：覆盖号也带判法单元指纹（只在我的副本上改，零轮）+ judging-files-included；
# G：覆盖号带判法单元指纹、不开 judging-files-included。用法：k1_runs_long.sh <模型目录> <副本目录前缀> <工作目录>
set -euo pipefail
model="$1"; prefix="$2"; work="$3"
long_line() {  # 时间线 副本后缀 开关
  local name="$1" suffix="$2" switch="$3" base="$work/$1"
  rm -rf "$base"; mkdir -p "$base"
  echo "name=timeline id=$name copies=lfu1$suffix,lfu0$suffix break=${switch:--}"
  local step state copy
  for step in 1 2 3 4; do
    state="S$((step - 1))"; copy="${prefix}lfu0$suffix"
    [[ "$step" == 1 ]] && copy="${prefix}lfu1$suffix"
    python3 -I "$model/k1_timeline.py" --scripts "$copy" --fixture "$base/repo" --state "$state" --library "$base/lib" --work "$base/work-$step" --break "$switch" \
      | grep -E 'name=(run|keys|coverage_record|cargo_calls)|计数：|基线：' | sed "s/^/$name.$step /"
  done
}
long_line E "" "judging-files-included"
long_line F "-fix" "judging-files-included"
long_line G "-fix" ""
