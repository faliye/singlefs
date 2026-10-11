#!/usr/bin/env bash
# restore.sh <relpath>：把自己拷贝里的文件还原成冻结副本的
set -e
W=/home/fy5090/.local/share/singlefs/work/m3-batch59-code-r1-sonnet
cp /home/fy5090/.local/share/singlefs/work/m3-batch59-code-r1/repo/"$1" $W/repo/"$1"
echo restored "$1"
