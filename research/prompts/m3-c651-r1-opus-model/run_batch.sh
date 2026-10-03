#!/usr/bin/env bash
# C651 攻方：每个候选一遍原型（副本里 core 与 checker 同一个开关）。用法：bash run_batch.sh <草稿目录> <批号>
# 草稿目录里要有 target/release/c651proto（见报告「复跑」一节）。每件退出码写 b<批号>-<件号>.rc。
set -uo pipefail
draft="$1"; batch="$2"
bin="$draft/target/release/c651proto"
rm -f "$draft"/b"$batch"-*.rc
# 件号 标签 环境
items=(
  "1 today"
  "2 const C651_CANDIDATE=const"
  "3 search C651_CANDIDATE=search"
  "4 refuse C651_CANDIDATE=refuse"
  "5 first C651_CANDIDATE=first"
  "6 unique C651_CANDIDATE=unique"
  "7 probe512 C651_CANDIDATE=probe C651_PROBED_IO_MIN=512"
  "8 probe8192 C651_CANDIDATE=probe C651_PROBED_IO_MIN=8192"
  "9 search-mkfsclears C651_CANDIDATE=search C651_MKFS_CLEARS=1"
)
for item in "${items[@]}"; do
  read -r number label environment <<<"$item"
  { env ${environment:-} nice -n 19 "$bin" > "$draft/b$batch-$number-$label.log" 2>&1; echo "$?" > "$draft/b$batch-$number.rc"; } &
  if [ "$number" = 8 ]; then wait; fi
done
wait
count=$(ls "$draft"/b"$batch"-*.rc | wc -l)
echo "rc_files=$count expected=9"
for item in "${items[@]}"; do
  read -r number label _ <<<"$item"
  echo "item=$number label=$label rc=$(cat "$draft/b$batch-$number.rc")"
done
