#!/usr/bin/env bash
# 用法：run-harness.sh <构建日志> <结果目录> <预算秒>：按构建日志里的 Executable 次序逐个跑 harness 测试二进制，跑过的跳过；到预算就停。
log="$1"; out="$2"; budget="$3"; mkdir -p "$out"; start=$(date +%s)
grep -o '(/[^)]*)' "$log" | tr -d '()' | while read -r bin; do
  name=$(basename "$bin"); [ -e "$out/$name.done" ] && continue
  now=$(date +%s); [ $((now - start)) -ge "$budget" ] && { echo "budget-stop"; break; }
  ( cd /tmp/claude-1000/m3-formal-code-r1-attack/tree/crates/singlefs-harness && "$bin" --test-threads=5 > "$out/$name.log" 2>&1 ); rc=$?
  echo "$rc" > "$out/$name.done"
  echo "$name rc=$rc $(grep -E '^test result' "$out/$name.log" | tail -1)"
done
