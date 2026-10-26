#!/usr/bin/env bash
# arg_limit_check.sh：本机上 execve 单个参数的上限（Linux 的 MAX_ARG_STRLEN，含结尾的 NUL）：试 100000、131071、131072、200000 字节。
getconf ARG_MAX | sed 's/^/ARG_MAX=/'
for n in 100000 131071 131072 200000; do
  s="$(head -c "$n" /dev/zero | tr '\0' x)"
  if /bin/true "$s" 2>/dev/null; then echo "单个参数 $n 字节: 通过"; else echo "单个参数 $n 字节: 失败(E2BIG)"; fi
done
