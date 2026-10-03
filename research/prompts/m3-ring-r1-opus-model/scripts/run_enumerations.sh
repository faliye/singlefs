#!/usr/bin/env bash
# H2：层 0 枚举域逐个崩溃状态（恢复 + oracle + 池级 checker），再跟进下一次可写挂载 + 后缀 + 回退扫描
set -u
bin=${RING_HOLE_BIN_DIR:-/tmp/claude-1000/m3-ring-r1-opus/target/release}/ring_hole_crash_states
out=${RING_HOLE_OUT:-/tmp/claude-1000/m3-ring-r1-opus-2/h2}
one() { # 名字 环境… -- 参数…
  local name=$1; shift
  local envs=()
  while [ "$1" != "--" ]; do envs+=("$1"); shift; done; shift
  local start; start=$(date +%s)
  env "${envs[@]}" nice -n 19 "$bin" "$@" > "$out/$name.out" 2> "$out/$name.err"
  echo "$name rc=$? seconds=$(( $(date +%s) - start ))" >> "$out/progress.md"
}
for m in ${WITHOUT_FILE_MODES:-1 3 4}; do
  one "wf-warm-s4-m$m" PREFIX_WITHOUT_FILE=1 CRASH_THE_WARM_UP=1 -- 4 back $m 2 1
done
for m in ${WITH_FILE_MODES:-1 3 5}; do
  one "file-s4-back-m$m" -- 4 back $m 2 1
done
