#!/usr/bin/env bash
# defs-m2-closeout-r3 云端攻方 K3 / G6：照 bash-command-detector.sh ④ 的出路「a & b & wait」写，与照共用约束 ④ 写，父进程各看到什么。
# 两件活：一件退 3、一件退 0；只起 bash -c 'exit N'，不碰仓里任何东西。
# 用法：bash ampersand-demo.sh [临时目录]
set -uo pipefail
scratch="$(mktemp -d "${1:-${TMPDIR:-/tmp}}/ampersand-demo.XXXXXX")"
bash -c 'exit 3' & bash -c 'exit 0' & wait
echo "hook 出路的写法 a & b & wait：wait 之后 \$?=$?"
rm -f "$scratch"/b1-*.rc
{ bash -c 'exit 3'; echo "$?" > "$scratch/b1-1.rc"; } & { bash -c 'exit 0'; echo "$?" > "$scratch/b1-2.rc"; } &
wait  # shell-lint:exit-collected 下面按批号件号逐个读 .rc
count="$(find "$scratch" -maxdepth 1 -name 'b1-*.rc' | wc -l)"
if [[ "$count" -ne 2 ]]; then echo "共用约束 ④ 的写法：.rc 文件 $count 个，派出 2 件，整批作废"; else
  echo "共用约束 ④ 的写法：.rc 文件 $count 个，第 1 件退 $(cat "$scratch/b1-1.rc")，第 2 件退 $(cat "$scratch/b1-2.rc")"; fi
rm -rf -- "${scratch:?}"
