#!/usr/bin/env bash
# 攻方腿模型（正文判据格第 1 格「关掉增量」那一项）：冻结的 slot_incremental_off 原样起，量四种调用方环境下它导出什么、删哪几份 incremental，
# 与有进程开着 incremental 里的文件时的出路（run_build_directory.py remove 不删、打一行 !、照往下编）。
# 用法：bash incremental_holder.sh --frozen <冻结副本> --repo <仓根> --work <新目录>
# 每条臂一份新的槽编译目录 <臂>/target：debug/incremental/a、x86_64-unknown-linux-gnu/debug/incremental/b、release/incremental/c 三份，
#   另起一个样本进程开着 debug/incremental/a 里的一份文件（新会话起，收尾停它的组、核组里没人）。
# 臂（调用方给的 CARGO_INCREMENTAL）：unset 不设；empty 设成空串；one 设成 1；zero 设成 0。
# 每条臂打一行 name=incremental_holder arm=<臂> exported=<收尾时 CARGO_INCREMENTAL 的值|unset> debug=<在|删了> triple=… release=… bang_lines=<! 行数> outlet=<有|没有>
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/slot_model_lib.sh"
model_arguments "$@"
model_load

state_of() { [[ -d "$1" ]] && echo 在 || echo 删了; }

run_arm() {   # <臂名> <unset|值>
  local name="$1" given="$2" arm="$MODEL_WORK/$1" opener
  mkdir -p -- "$arm/target/debug/incremental/a" "$arm/target/x86_64-unknown-linux-gnu/debug/incremental/b" "$arm/target/release/incremental/c"
  printf 'cache\n' > "$arm/target/debug/incremental/a/query-cache.bin"
  printf 'cache\n' > "$arm/target/x86_64-unknown-linux-gnu/debug/incremental/b/query-cache.bin"
  printf 'cache\n' > "$arm/target/release/incremental/c/query-cache.bin"
  setsid python3 -c "$MODEL_HOLDER_PROGRAM" open "$arm/target/debug/incremental/a/query-cache.bin" "$arm/opened" &
  opener=$!
  wait_ready "$arm/opened" || { stop_holder "$opener"; return 1; }
  (
    if [[ "$given" == unset ]]; then unset CARGO_INCREMENTAL; else export CARGO_INCREMENTAL="$given"; fi
    SLOT_TARGET="$arm/target"
    slot_incremental_off
    printf '%s\n' "${CARGO_INCREMENTAL-unset}" > "$arm/exported"
    bash -c 'printf "%s\n" "${CARGO_INCREMENTAL-unset}"' > "$arm/child-sees"
  ) > "$arm/off.out" 2>&1
  stop_holder "$opener"
  echo "name=incremental_holder arm=$name exported=$(cat "$arm/exported") child_sees=$(cat "$arm/child-sees")" \
    "debug=$(state_of "$arm/target/debug/incremental") triple=$(state_of "$arm/target/x86_64-unknown-linux-gnu/debug/incremental")" \
    "release=$(state_of "$arm/target/release/incremental") bang_lines=$(grep -c '^  ! ' "$arm/off.out")" \
    "outlet=$(grep -qF '→ 怎么办' "$arm/off.out" && echo 有 || echo 没有)"
  sed 's/^/    /' "$arm/off.out"
}

run_arm unset unset
run_arm empty ""
run_arm one 1
run_arm zero 0
echo "name=incremental_holder_done arms=4"
