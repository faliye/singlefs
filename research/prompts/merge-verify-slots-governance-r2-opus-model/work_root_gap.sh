#!/usr/bin/env bash
# 攻方腿模型（正文判据格第 1 格「三样一起给」那一项）：冻结的 choose_slot 原样起，看只给 SINGLEFS_WORK_ROOT、不给自证那两样时，
# 清槽已不在的编译目录（sweep_slot_targets）会不会拿另一个工作根的槽那一层对着这一台的编译根删。
# 用法：bash work_root_gap.sh --frozen <冻结副本> --repo <仓根> --work <新目录>
#
# 每条臂各建一份互不相干的现场（都在 --work 下）：
#   「原来的工作根」W1 的槽那一层放四槽：slot.idle（空着）、slot.kept（留着：kept 与 repo/ 都在）、slot.held（样本进程拿着它的锁，演别的趟正占着、在两步 cargo 之间）、
#   slot.busy（空着，但样本进程开着它编译目录里的一份文件，演正在编）；编译根 B 下照冻结代码的布局 <B>/<那一层>/<槽名>/target 各放一份编译目录。
#   编译根：多机配置里这一台那一项由 SLOT_BUILD_ROOT_PROGRAM 读；这里不读真配置，slot_build_root 换成打 B 的替身（只换这一个函数，别的都是冻结代码）。
# 臂：
#   control       SINGLEFS_WORK_ROOT=W1、slots=3：挑槽、清编译目录都对着 W1，四份编译目录都该在
#   gap-take      SINGLEFS_WORK_ROOT=W2（另一个空的工作根）、slots=3、不给 MERGE_AND_VERIFY_SLOT_TABLE 与 MERGE_AND_VERIFY_SLOT_BUILD_ROOT：「三样」那一道不拦
#   gap-zero      同上，slots=0（只清、不占）
#   fix-namespace 改法（零轮，只在本模型上量过）：编译根下那一层按槽那一层真路径的 sha256 头 16 位再分一层，W1 的编译目录建在 W1 那一层，gap-take 的起法
#   fix-same-root 改法（零轮，只在本模型上量过）：没给 MERGE_AND_VERIFY_SLOT_BUILD_ROOT 时工作根的真路径要与「这一台缺省的工作根」（这里是 W1）相同，不同就不占也不清
# 每条臂打一行 name=work_root_gap arm=<臂> idle=<在|删了> kept=… held=… busy=… took=<占的槽名|none> fallback=<没占到的为什么>。
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/slot_model_lib.sh"
model_arguments "$@"
model_load

seed_arm() {   # <臂目录> <编译根下那一层的名字>
  local arm="$1" layer="$2" slot
  mkdir -p -- "$arm/w1/merge-and-verify" "$arm/w2" "$arm/build" "$arm/tree" "$arm/logs"
  printf 'slots=3\nslot-target-limit-gib=150\n' > "$arm/table.conf"
  for slot in slot.idle slot.kept slot.held slot.busy; do
    mkdir -p -- "$arm/w1/merge-and-verify/$slot/repo" "$arm/build/$layer/$slot/target/debug"
    printf '0\n' > "$arm/w1/merge-and-verify/$slot/used"
    printf 'artifact\n' > "$arm/build/$layer/$slot/target/debug/artifact"
  done
  printf 'log: model\ncreated: 2026-11-20\n' > "$arm/w1/merge-and-verify/slot.kept/kept"
}

namespace_of() {   # <槽那一层>：改法 fix-namespace 的那一层名字
  mkdir -p -- "$1"
  printf 'merge-and-verify-slots/%s' "$(printf '%s' "$(cd "$1" && pwd -P)" | sha256sum | cut -c1-16)"
}

state_of() { [[ -d "$1" ]] && echo 在 || echo 删了; }

run_arm() {   # <臂名> <给的 SINGLEFS_WORK_ROOT 是 w1 还是 w2> <slots> <改法：none|namespace|same-root>
  local name="$1" which="$2" limit="$3" fix="$4" arm="$MODEL_WORK/$1" layer="merge-and-verify-slots" locker opener took fallback
  [[ "$fix" != namespace ]] || layer="$(namespace_of "$arm/w1/merge-and-verify")"
  seed_arm "$arm" "$layer"
  setsid python3 -c "$MODEL_HOLDER_PROGRAM" lock "$arm/w1/merge-and-verify/slot.held/lock" "$arm/locked" &
  locker=$!
  setsid python3 -c "$MODEL_HOLDER_PROGRAM" open "$arm/build/$layer/slot.busy/target/debug/artifact" "$arm/opened" &
  opener=$!
  wait_ready "$arm/locked" && wait_ready "$arm/opened" || { stop_holder "$locker"; stop_holder "$opener"; return 1; }
  (
    export SINGLEFS_WORK_ROOT="$arm/$which"
    SLOT_LIMIT="$limit"; SLOT_TABLE="$arm/table.conf"; SOURCE_TREE="$arm/tree"; STAND_IN_BUILD_ROOT="$arm/build"
    SLOT_DIRECTORY=""; SLOT_TARGET=""; SLOT_FALLBACK_REASON=""; SLOT_ROOT_DIRECTORY=""; SLOT_HOLDER_PID=""; SLOT_HOLDER_FD=""
    slot_build_root() { printf '%s\n' "$STAND_IN_BUILD_ROOT"; }
    [[ "$fix" != namespace ]] || SLOT_BUILD_DIRECTORY_NAME="$(namespace_of "$SINGLEFS_WORK_ROOT/merge-and-verify")"
    if [[ "$fix" == same-root && -z "${MERGE_AND_VERIFY_SLOT_BUILD_ROOT:-}" \
          && "$(cd "$SINGLEFS_WORK_ROOT" && pwd -P)" != "$(cd "$arm/w1" && pwd -P)" ]]; then
      SLOT_FALLBACK_REASON="工作根不是这一台缺省的那一个，不占也不清（改法 fix-same-root）"
    else
      choose_slot "$arm/logs"
    fi
    printf '%s\n' "${SLOT_DIRECTORY:+$(basename -- "$SLOT_DIRECTORY")}" > "$arm/took"
    printf '%s\n' "$SLOT_FALLBACK_REASON" > "$arm/fallback"
    release_slot
  ) > "$arm/choose.out" 2>&1
  stop_holder "$locker"
  stop_holder "$opener"
  took="$(cat "$arm/took")"; fallback="$(cat "$arm/fallback")"
  echo "name=work_root_gap arm=$name idle=$(state_of "$arm/build/$layer/slot.idle/target") kept=$(state_of "$arm/build/$layer/slot.kept/target")" \
    "held=$(state_of "$arm/build/$layer/slot.held/target") busy=$(state_of "$arm/build/$layer/slot.busy/target") took=${took:-none} fallback=${fallback:-none}"
  sed 's/^/    /' "$arm/choose.out"
}

run_arm control w1 3 none
run_arm gap-take w2 3 none
run_arm gap-zero w2 0 none
run_arm fix-namespace w2 3 namespace
run_arm fix-same-root w2 3 same-root
echo "name=work_root_gap_done arms=5"
