#!/usr/bin/env bash
# 崩溃放量的两个驱动共用的那几段：读配置的开关与显卡配置、组 cargo 的参数与环境、在本机跑一趟用例、从日志里取派活计划的数。
# 被 source 的函数库，只定义函数、不改 shell 选项，不单独调；research/scripts/crash-amplification-item.sh（一个分项的一趟）与
# research/scripts/crash-amplification-two-hosts.sh（两台各领一份）source 它。
# gate-similar: peer-host-lib.sh 它管第二台那几段（读多机配置、在第二台上跑命令、清场与复原）；这里管本机那一趟与派活计划的数，不连第二台
#
# 调用方先设好这几个变量再调 crash_amplification_prepare：
#   repository_root  仓根
#   test_target      crates/singlefs-checker-tier/tests/ 下的测试目标；test_filter 用例函数名（可空）
#   expand_up_to、threads  可空；memory_cap 本机那一趟的内存上限
#   library_is_temporary   1 表示库只活这一趟（自证用）
#   quiesced               1 表示清过场（卡上别的服务停了）：显卡的先后与额度取清场之后那一份
# crash_amplification_prepare 之后可用：
#   configuration_file、gpu_switch（0 / 1）、across_machines_switch（0 / 1）、features、
#   gpu_cards_configuration_file（显卡配置，不在是空串）、gpu_cards_by_priority（这一趟用的那一份，没写是空串）、
#   gpu_cards_by_priority_when_quiesced（清场之后那一份，没写是空串）、common_environment、cargo_build_arguments、cargo_test_arguments（数组）
#
# 显卡配置：${SINGLEFS_GPU_CARDS_CONFIG:-${XDG_CONFIG_HOME:-~/.config}/singlefs/gpu-cards.env}（这台开发机私有，不进仓；模板是仓根的 gpu-cards.env.example）。
# 两个键：GPU_CARDS_BY_PRIORITY（不清场时显卡的先后与每张卡的显存额度，读法在 crates/singlefs-checker-tier/src/crash_amplification.rs 的 parse_cards_by_priority）、
# GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED（清场之后的先后与额度，同一种写法；全量开跑就清场，用的是这一份）。
# 挑中的那一份经环境变量 GPU_CARDS_BY_PRIORITY 交给用例（本机与第二台同一份，各取点名给自己的卡）。

CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS="-I/usr/lib/gcc/x86_64-linux-gnu/13/include -I/usr/lib/gcc/x86_64-linux-gnu/14/include"  # librocksdb-sys 的 bindgen 找 stdbool.h；哪个版本的目录在就用哪个，不在的 -I 无害

crash_amplification_configuration_value() { # <配置文件> <键>：打那个键的值（去掉两头成对的一层引号）；文件不在、键没写打空串
  [[ -f "$1" ]] || return 0
  python3 - "$1" "$2" <<'PYTHON'
import sys
path, key = sys.argv[1], sys.argv[2]
for line in open(path, encoding="utf-8"):
    line = line.strip()
    if not line.startswith(key + "="):
        continue
    value = line[len(key) + 1:].strip()
    if len(value) >= 2 and value[0] == value[-1] and value[0] in "'\"":
        value = value[1:-1]
    print(value)
    break
PYTHON
}

crash_amplification_prepare() {
  configuration_file="${SINGLEFS_MULTI_HOST_CONFIG:-$repository_root/multi-host.env}"
  gpu_switch=0; across_machines_switch=0; features="verdict-store"
  [[ "$(crash_amplification_configuration_value "$configuration_file" ENABLE_GPU)" == 1 ]] && gpu_switch=1
  [[ "$(crash_amplification_configuration_value "$configuration_file" ENABLE_ACROSS_MACHINES)" == 1 ]] && across_machines_switch=1
  if [[ "${library_is_temporary:-0}" == 1 ]]; then gpu_switch=0; fi  # 自证不上卡
  (( gpu_switch )) && features="verdict-store,gpu"
  gpu_cards_configuration_file="${SINGLEFS_GPU_CARDS_CONFIG:-${XDG_CONFIG_HOME:-$HOME/.config}/singlefs/gpu-cards.env}"
  [[ -f "$gpu_cards_configuration_file" ]] || gpu_cards_configuration_file=""
  gpu_cards_by_priority=""; gpu_cards_by_priority_when_quiesced=""
  if [[ -n "$gpu_cards_configuration_file" ]]; then
    gpu_cards_by_priority="$(crash_amplification_configuration_value "$gpu_cards_configuration_file" GPU_CARDS_BY_PRIORITY)"
    gpu_cards_by_priority_when_quiesced="$(crash_amplification_configuration_value "$gpu_cards_configuration_file" GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED)"
    if [[ "${quiesced:-0}" == 1 ]]; then
      if [[ -z "$gpu_cards_by_priority_when_quiesced" ]]; then
        echo "  ✗ 说是清过场，显卡配置里却没写 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED" >&2
        echo "  → 怎么办：照仓根 gpu-cards.env.example 在显卡配置里写清场之后的显卡先后与额度" >&2
        return 2
      fi
      gpu_cards_by_priority="$gpu_cards_by_priority_when_quiesced"
    fi
  fi
  common_environment=(SINGLEFS_HEAVY_TESTS=user-request "BINDGEN_EXTRA_CLANG_ARGS=$CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS")
  [[ "${library_is_temporary:-0}" == 1 ]] && common_environment+=(SINGLEFS_CRASH_AMPLIFICATION_LIBRARY_IS_TEMPORARY=1)
  [[ -n "${expand_up_to:-}" ]] && common_environment+=("SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=$expand_up_to")
  local cores="${threads:-}"
  [[ -n "$cores" ]] || cores=$(( $(nproc) > 1 ? $(nproc) - 1 : 1 ))
  common_environment+=("SINGLEFS_CRASH_AMPLIFICATION_THREADS=$cores")
  cargo_build_arguments=(test --release -p singlefs-checker-tier --features "$features" --test "$test_target" --no-run)
  cargo_test_arguments=(test --release -p singlefs-checker-tier --features "$features" --test "$test_target" -- --nocapture)
  [[ -n "${test_filter:-}" ]] && cargo_test_arguments+=(--exact "$test_filter")
  return 0
}

# 本机的一趟：先 6 路并行只编（librocksdb-sys 的 C++ 按并行度吃内存），再跑；跑的那一步用 threads 个核，没给就是本机核数减一
# （用户 2026-09-29 定：测试与真跑都用 31 核，留一个核给机器上别的事）。
# 要这一趟中途停下：附加环境里给 SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE=<文件>，建出那个文件它就不再领新的块、手里那一块判完写进库、
# 打一行汇总退 0；判过的块在库里，下一趟接着判。
crash_amplification_run_on_this_host() { # <share> <库目录> <日志> [<附加环境 键=值>…]
  local share="$1" library="$2" log="$3"
  shift 3
  local cores="${threads:-}"
  [[ -n "$cores" ]] || cores=$(( $(nproc) > 1 ? $(nproc) - 1 : 1 ))
  local local_environment=("${common_environment[@]}" "$@" "SINGLEFS_CRASH_AMPLIFICATION_SHARE=$share" "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY=$library")
  [[ -n "$gpu_cards_by_priority" ]] && local_environment+=("GPU_CARDS_BY_PRIORITY=$gpu_cards_by_priority")
  ( cd "$repository_root" \
    && env "${local_environment[@]}" bash research/scripts/run-with-memory-cap.sh "$memory_cap" bash research/scripts/capped.sh 6 cargo "${cargo_build_arguments[@]}" \
    && env "${local_environment[@]}" bash research/scripts/run-with-memory-cap.sh "$memory_cap" bash research/scripts/capped.sh "$cores" cargo "${cargo_test_arguments[@]}" ) >"$log" 2>&1
}

# 一台机器判这一趟的总速度（态/秒，取整）：各条流要判的状态加起来 ÷ 各条流估的用时加起来。一条用例里几条流各打一行，
# 各行的 states_per_second_on_this_host 是各自那条流的速度，直接相加没有意义（五条流各每秒 50 万，合起来仍是每秒 50 万上下）。
# 一行都没有打空串；估的用时合计是 0（没有一条流量出速度）打 0。
crash_amplification_placement_rate() { # <日志>
  python3 - "$1" <<'PYTHON'
import re, sys
pending, seconds, lines = 0, 0, 0
for line in open(sys.argv[1], encoding="utf-8", errors="replace"):
    if not line.startswith("CRASH_AMPLIFICATION_PLACEMENT "):
        continue
    lines += 1
    fields = dict(re.findall(r"(\w+)=([0-9]+)(?= |$)", line.strip()))
    pending += int(fields.get("pending_states", 0))
    seconds += int(fields.get("estimated_seconds_on_this_host", 0))
if lines:
    print(pending // seconds if seconds else 0)
PYTHON
}

# 日志里 CRASH_AMPLIFICATION_PLACEMENT 行某个字段的合计（一条用例里几条流各打一行）；一行都没有打空串。
crash_amplification_placement_total() { # <日志> <字段名>
  python3 - "$1" "$2" <<'PYTHON'
import re, sys
log, field = sys.argv[1], sys.argv[2]
total, lines = 0, 0
for line in open(log, encoding="utf-8", errors="replace"):
    if not line.startswith("CRASH_AMPLIFICATION_PLACEMENT "):
        continue
    lines += 1
    found = re.search(r"(?:^| )" + re.escape(field) + r"=([0-9]+)(?: |$)", line.strip())
    if found:
        total += int(found.group(1))
if lines:
    print(total)
PYTHON
}
