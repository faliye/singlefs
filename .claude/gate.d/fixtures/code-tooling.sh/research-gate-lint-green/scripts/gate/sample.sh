#!/usr/bin/env bash
source "$(dirname "$0")/lib.sh"
[[ -f input.txt ]] || die "找不到 input.txt" "先跑 make-input.sh 生成它"
# 这几种 cd 都接了出路：|| 接拒绝、&& 接下一步、命令替换里接 && pwd、续行之后接 &&
cd "$(dirname "$0")" || die "进不去 $(dirname "$0")" "从仓根照用法起"
here="$(cd "$(dirname "$0")" && pwd)"
( cd "$here" \
  && ls >/dev/null )
