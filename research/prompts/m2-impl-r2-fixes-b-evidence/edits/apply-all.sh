#!/usr/bin/env bash
# 在一份仓副本上重放我的全部改动。
set -euo pipefail
root=$1
here=$(dirname "$(readlink -f "$0")")
cd "$here"
for script in 01_y4.py 02_d16.py 03_y2.py 04_matches.py; do python3 "$script" "$root"; done
bash 05_moved.sh "$root"
cp -r "$here/new/." "$root/"
echo "new files copied"
for script in $(ls 1[0-9]_*.py 2>/dev/null); do python3 "$script" "$root"; done
# 我改过的文件过一遍 rustfmt（带 `mod x;` 的会连子模块一起格式化：子模块本来就是格式化过的，不变；apply 之后另核不归我的文件与 base 相同）。
cd "$root"
while read -r path; do
  [ -f "$path" ] || continue
  rustfmt --edition 2021 "$path"
done < "$here/../mine.txt"
echo "rustfmt done"
