#!/usr/bin/env bash
# q.sh <钩子:guard|det> <agent> <命令> : 改前（tree-orig）、改后（tree）各用真钩子入口喂一次，打 rc 与第一条 ✗ 行
# 环境：CWD_IN_TREE=1 时 hook 输入里的 cwd 是各自那棵树的根（命令里有 scripts/hosts/… 这类仓内路径时用），否则是 $SW_ROOT/scratch
trunc() { python3 -I -c 'import sys; print(sys.stdin.read().rstrip("\n")[:int(sys.argv[1])])' "$1"; }   # 按字符截（GNU cut -c 其实按字节，会把汉字截在中间）
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; export SW_ROOT="${SW_ROOT:-$(cd "$here/../.." && pwd)}" SINGLEFS_WORK_ROOT=/var/empty/swr
for t in tree-orig tree; do
  if [[ -n "${CWD_IN_TREE:-}" ]]; then export CWD="$SW_ROOT/$t"; else export CWD="$SW_ROOT/scratch"; fi
  out=$(TREE=$t "$here/feed.sh" "$1" "$2" "$3" 2>&1)
  echo "  $([[ $t == tree-orig ]] && echo 改前 || echo 改后)：$(printf '%s\n' "$out" | head -1) $(printf '%s\n' "$out" | grep -m1 '✗' | trunc 110)"
done
