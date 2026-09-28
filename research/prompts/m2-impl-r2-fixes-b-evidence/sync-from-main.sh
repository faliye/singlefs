#!/usr/bin/env bash
# 把主工作区里不归我的文件同步进 base/ 与 work/：base 整份跟主工作区；work 跟主工作区，但 mine.txt 里列的（我改的）不覆盖。
# 同步前报出主工作区里我改的那几份自 base 之后被别人动过没有（动过就要手工合）。
set -uo pipefail
main=<仓根>
here=/tmp/claude-1000/impl-r2-fixes-b
while read -r path; do
  [ -z "$path" ] && continue
  if ! cmp -s "$main/$path" "$here/base/$path"; then echo "主工作区动过我的文件：$path"; fi
done < "$here/mine.txt"
rsync -a --delete --exclude target --exclude .git --exclude prove-red-logs "$main/" "$here/base/"
rsync -a --delete --exclude target --exclude .git --exclude prove-red-logs --exclude-from="$here/mine.txt" "$main/" "$here/work/"
echo synced
