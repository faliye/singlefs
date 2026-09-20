#!/usr/bin/env bash
# 一份副本、若干处变异、跑一个测试二进制：run-one.sh 名字 "cargo test 参数" [文件 原文 替换文]...
set -u
repository=/home/fy5090/code/singlefs
scratch=/tmp/claude-1000/m2-supp3-item2-implementer
name=$1; arguments=$2; shift 2
directory=$scratch/copies/$name
rm -rf "$directory"
rsync -a --exclude target --exclude .git "$repository"/ "$directory"/
while [ $# -ge 3 ]; do
  python3 "$scratch/apply-mutation.py" "$directory" "$1" "$2" "$3" || exit 1
  shift 3
done
echo "── $name $(date -u +%H:%M:%S) UTC：cargo test $arguments"
(cd "$directory" && nice -n 19 cargo test $arguments) > "$scratch/$name.log" 2>&1
grep -E '^test result|^test .* FAILED$|^test .* ok$' "$scratch/$name.log"
