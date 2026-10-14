#!/usr/bin/env bash
# V1 喂法：每个变体 = base.txt + 一段线程上限写法，经派发闸 --check-prompt（与钩子入口同一份 judge_dispatch）判，打印退出码与首行。
# 用法：v1_feed.sh <仓根> <类型> <变体表> <输出目录>
set -u
root=$1; type=$2; table=$3; out=$4
here=$(cd "$(dirname "$0")/.." && pwd)
mkdir -p "$out"; out=$(cd "$out" && pwd)
while IFS=$'\t' read -r name body; do
  [ -z "$name" ] && continue
  case "$name" in \#*) continue;; esac
  f="$out/$name.txt"
  { cat "${V1_BASE:-$here/feeds/v1/base.txt}"; printf '%b\n' "$body"; } > "$f"
  msg=$(cd "$root" && CLAUDE_PROJECT_DIR="$root" bash .claude/hooks/runner-dispatch-guard.sh --check-prompt "$f" --type "$type" --root "$root" 2>&1)
  rc=$?
  first=$(printf '%s\n' "$msg" | grep -m1 -E '✗|✓' )
  printf '%s\trc=%s\t%s\n' "$name" "$rc" "$first"
done < "$table"
