#!/usr/bin/env bash
# cl.sh <仓内相对路径> <固定串>：在今天的主工作区与冻结副本里各 grep -nF，打印行号
f="$1"; q="$2"
frozen=/tmp/claude-1000/m3-root-ring-code-r1/tree
a=$(grep -nF -- "$q" "/home/fy5090/code/singlefs/$f" | cut -d: -f1 | tr '\n' ',')
if [ -f "$frozen/$f" ]; then b=$(grep -nF -- "$q" "$frozen/$f" | cut -d: -f1 | tr '\n' ','); else b='(无冻结副本)'; fi
echo "$f | 今天:$a | 冻结:$b | $q"
