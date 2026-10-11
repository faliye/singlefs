#!/usr/bin/env bash
# 用法：run_mutant.sh <名> <相对 repo 的文件> <旧串> <新串> <cargo test 参数…>
# 把冻结副本里的文件改一处写进我的拷贝，跑点名的测试，结束后把我的拷贝还原成冻结副本的样子
set -u
W=~/.local/share/singlefs/work/m3-batch57-code-r1-sonnet
FROZEN=~/.local/share/singlefs/work/m3-batch57-code-r1/repo
name=$1; file=$2; old=$3; new=$4; shift 4
python3 $W/model/mutate_walk.py "$FROZEN/$file" "$W/repo/$file" "$old" "$new" || exit 9
export CARGO_TARGET_DIR=$W/target
( cd $W/repo && nice -n 19 bash /home/fy5090/code/singlefs/scripts/run-with-memory-cap.sh 10G bash /home/fy5090/code/singlefs/scripts/capped.sh 10 cargo test --offline "$@" ) > $W/mut-$name.log 2>&1
rc=$?
cp "$FROZEN/$file" "$W/repo/$file"
echo "MUTANT $name rc=$rc"; grep -E '^test result|FAILED|failed' $W/mut-$name.log | head -8
