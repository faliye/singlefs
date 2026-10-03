#!/usr/bin/env bash
# 用法：run-verus-batch.sh <变体目录…>；每个目录里 src/lib.rs，逐个 verus 直调，输出落 <目录>/verus.out，判定行与退出码汇总到 summary.txt
set -u
VERUS="$HOME/.local/share/singlefs/formal-tools/verus-0.2026.09.20/verus-x86-linux/verus"
here="$(pwd)"
: > "$here/summary.txt"
for dir in "$@"; do
  ( cd "$dir" && nice -n 19 "$VERUS" --crate-type=lib --edition=2021 --crate-name singlefs_replay_prefix --no-cheating src/lib.rs > verus.out 2>&1; echo "exit=$?" >> verus.out )
  line="$(grep -E 'verification results::' "$dir/verus.out" | tail -1)"
  code="$(tail -1 "$dir/verus.out")"
  echo "$(basename "$dir")	${line:-没有判定行}	$code" >> "$here/summary.txt"
done
