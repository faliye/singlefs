#!/usr/bin/env bash
# admission: always 每次调都在这一刻的树与库上排派活计划、再跑；哪些块要重判由库里的块键与判法版本定，不在这里
# run-condition: command bash cargo python3
# 崩溃放量一个分项的一趟：配置定几台、用不用 GPU，这里照配置跑，做不到就报错退出，不退回少一台、不退回 CPU
# （用户 2026-09-29 定：「配置中一台机器就要跑一台，配置中两台机器就要跑两台，配置中有gpu就一定要让GPU工作 如果不行就明确报错拒绝，不接受默认降级」）。
#
#   crash-amplification-item.sh <测试目标> [--test-filter <用例函数名>] [--library <库目录>] [--expand-up-to <N>] [--threads <N>]
#                               [--memory-cap <本机上限>] [--peer-memory-cap <第二台上限>] [--label <标签>] [--quiesced]
#   crash-amplification-item.sh --selftest      拿造出来的派活计划行走一遍「在哪跑」的判法与第二台失败时的去向（不编、不跑、不上卡、不连第二台）
#
# 测试目标是 crates/singlefs-checker-tier/tests/ 下一条走 crash_amplification::run_from_environment 的用例。
# 步骤：
#   ① 配置（${SINGLEFS_MULTI_HOST_CONFIG:-仓根 multi-host.env}）没开 GPU：开了双机交 crash-amplification-two-hosts.sh 两台跑，没开本机 CPU 判。
#   ② 开了 GPU：先核显卡配置（GPU_CARDS_BY_PRIORITY，--quiesced 时取 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED）与双机开关对得上——
#      双机开着，两台都要有点名的卡；双机没开，不许点名第二台的卡。对不上退 2。
#      再在本机跑一趟只排派活计划（SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1，不判），读它打的 CRASH_AMPLIFICATION_PLACEMENT 行
#      （一条用例里几条流各一行，逐个字段加起来）：要判的状态数 pending_states、本机的卡试判量到的速度 states_per_second_on_this_host。
#      一行都没有（那一趟没跑成：建不出表、点名的卡用不了、内核超了预算）就退 2，打出日志里 panicked 那几行。
#   ③ 双机没开：本机一台跑（SHARE 0/1）。双机开着：交 crash-amplification-two-hosts.sh，带 --local-states-per-second <本机量到的速度>
#      （它让第二台也量一次，两台按速度分）。第二台没量出速度（双机驱动退 249）、或它那一份失败：原样退它的退出码，不改在本机一台跑。
# 显卡的先后与额度：带 --quiesced（调用方清过场）用清场之后那一份，不带用平时那一份；清场与复原归调用方
#   （research/scripts/crash-amplification.sh 开跑就清、退出时复原）。
# 库与日志：库默认 ${SINGLEFS_CRASH_AMPLIFICATION_HOME:-~/.local/share/singlefs/crash-amplification}/library（开机不清空，重跑同一条命令就是续跑），
#   日志在 <库的上层>/runs/<标签>/：placement.log（排计划那一趟）、run.log（本机一台跑的那一趟）；两台跑时是双机驱动那几份。
# 输出：CRASH_AMPLIFICATION_PLACEMENT 行原样转打；一行 CRASH_AMPLIFICATION_DECISION where=<local|two-hosts|local-cpu|two-hosts-cpu> reason=<为什么> …；
#   那一趟的 CRASH_AMPLIFICATION、CRASH_AMPLIFICATION_GPU_CARD、CRASH_AMPLIFICATION_GPU_TOTAL 行原样转打。
# 退出码：跑的那一趟的退出码（250–254 是内存包装自己的结局，249 是第二台没量出速度）；2 用法错、配置对不上、或排计划那一趟没打出计划行。
# 重型：跑 checker 档用例，只在用户要求时跑（SINGLEFS_HEAVY_TESTS=user-request 由这里带给 cargo）。
# 弄坏开关 CRASH_AMPLIFICATION_ITEM_BREAK=<项>（只给 --selftest 证红用）：
#   stay-local-when-across       双机开着也本机一台跑（「配置两台就跑两台」判错）
#   ignore-across-switch         双机没开也动用第二台（「配置一台就跑一台」判错）
#   accept-peer-cards-without-across  双机没开、显卡配置却点名了第二台的卡也照跑（「配置自相矛盾就拒绝」判错）
#   accept-missing-peer-cards    双机开着、显卡配置里却没有第二台的卡也照跑（「两台都要有 GPU 干活」判错）
#   accept-missing-placement     计划行一行都没有也照本机跑（「没有计划行 ⇒ 退 2」判错）
#   local-run-when-the-second-host-fails  第二台没量出速度时改在本机一台跑（「不退回少一台」判错）
# 只供测试：CRASH_AMPLIFICATION_ITEM_DECIDE_ONLY=<造好的 placement.log> 时不编不跑，只拿那份日志判在哪跑、打 CRASH_AMPLIFICATION_DECISION 行就退；
#   CRASH_AMPLIFICATION_ITEM_SWITCHES=<gpu>,<双机> 代替读配置里的这两样；CRASH_AMPLIFICATION_ITEM_CARDS=<显卡配置的值> 代替读显卡配置；
#   CRASH_AMPLIFICATION_ITEM_LOCAL_RUN=<脚本> 换掉本机那一趟（调法：<脚本> <share> <库目录> <日志> [<附加环境 键=值>…]，自己写日志）；
#   CRASH_AMPLIFICATION_TWO_HOSTS_DRIVER=<脚本> 换掉双机驱动。
#
# gate-similar: crash-amplification-two-hosts.sh 它管两台各领一份、取回、导入那一趟；这里核配置、排派活计划、定一台还是两台，本机一台跑的那一趟自己跑
# gate-similar: crash-amplification.sh 它是分项登记表与全量的循环，清场与复原在它那里，每项调这里
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

item_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repository_root="$(cd "$item_script_directory/../.." && pwd)"
source "$item_script_directory/crash-amplification-lib.sh"

item_break_is() { [[ ",${CRASH_AMPLIFICATION_ITEM_BREAK:-}," == *",$1,"* ]]; }

usage() {
  echo "  ✗ 用法：$0 <测试目标> [--test-filter <用例函数名>] [--library <库目录>] [--expand-up-to <N>] [--threads <N>] [--memory-cap <本机上限>] [--peer-memory-cap <第二台上限>] [--label <标签>] [--quiesced]，或 --selftest" >&2
  echo "  → 怎么办：测试目标写 crates/singlefs-checker-tier/tests/ 下的文件名（不带 .rs），例 crash_amplification_fixed_script_through_rollback" >&2
  exit 2
}

# 本机上限：留在 slice 总上限（整机 − 余量 20G = 40.1G）之内，给 38G（给满 40G 时 slice 里残留的几百 MiB 就让它排不进队）
test_target=""; expand_up_to=""; memory_cap="38G"; peer_memory_cap=""; label="crash-amplification-$(date -u +%Y%m%dT%H%M%SZ)-$$"; selftest=0; threads=""; library_directory=""; test_filter=""; quiesced=0
forwarded_arguments=()
while (($# > 0)); do
  case "$1" in
    --expand-up-to) expand_up_to="${2:?--expand-up-to 要一个数}"; forwarded_arguments+=("$1" "$2"); shift 2 ;;
    --test-filter) test_filter="${2:?--test-filter 要一个用例函数名}"; forwarded_arguments+=("$1" "$2"); shift 2 ;;
    --library) library_directory="${2:?--library 要一个目录}"; forwarded_arguments+=("$1" "$2"); shift 2 ;;
    --threads) threads="${2:?--threads 要一个数}"; forwarded_arguments+=("$1" "$2"); shift 2 ;;
    --memory-cap) memory_cap="${2:?--memory-cap 要一个上限}"; forwarded_arguments+=("$1" "$2"); shift 2 ;;
    --peer-memory-cap) peer_memory_cap="${2:?--peer-memory-cap 要一个上限}"; forwarded_arguments+=("$1" "$2"); shift 2 ;;
    --label) label="${2:?--label 要一个标签}"; shift 2 ;;
    --quiesced) quiesced=1; forwarded_arguments+=("$1"); shift ;;
    --selftest) selftest=1; shift ;;
    -*) usage ;;
    *) if [[ -n "$test_target" ]]; then usage; fi; test_target="$1"; shift ;;
  esac
done

# 显卡配置与双机开关对不对得上：双机开着两台都要有点名的卡，双机没开不许点名第二台的卡。对得上交回 0，对不上打原因与出路、交回 2
check_the_cards_against_the_hosts() { # <双机开关 0/1> <显卡配置的值>
  local across="$1" cards="$2" names_local=0 names_peer=0 entry
  for entry in $cards; do
    [[ "$entry" == local:* ]] && names_local=1
    [[ "$entry" == peer:* ]] && names_peer=1
  done
  if [[ "$across" == 1 ]] && (( ! names_local || ! names_peer )) && ! item_break_is accept-missing-peer-cards; then
    echo "  ✗ 配置开了双机与 GPU，显卡配置「$cards」却没有两台各点名至少一张卡" >&2
    echo "  → 怎么办：照仓根 gpu-cards.env.example 给两台各点名卡（local:<序号>:<额度> 与 peer:<序号>:<额度>）；不想用第二台就把多机配置的 ENABLE_ACROSS_MACHINES 改成 0" >&2
    return 2
  fi
  if [[ "$across" != 1 ]] && (( names_peer )) && ! item_break_is accept-peer-cards-without-across; then
    echo "  ✗ 多机配置没开双机，显卡配置「$cards」却点名了第二台的卡：点名的卡用不上" >&2
    echo "  → 怎么办：要用第二台就把多机配置的 ENABLE_ACROSS_MACHINES 改成 1；不用就从显卡配置里删掉 peer: 那几项" >&2
    return 2
  fi
  return 0
}

# 在哪跑：读 placement.log 与双机开关，打 CRASH_AMPLIFICATION_DECISION 行；交回 0 本机一台跑、10 两台跑、2 没有计划行
decide_where_to_run() { # <placement.log> <双机开关 0/1>
  local placement_log="$1" across="$2"
  local pending local_speed
  pending="$(crash_amplification_placement_total "$placement_log" pending_states)"
  if [[ -z "$pending" ]]; then
    if item_break_is accept-missing-placement; then
      echo "CRASH_AMPLIFICATION_DECISION where=local reason=no-placement-line"
      return 0
    fi
    echo "  ✗ 排派活计划那一趟一行 CRASH_AMPLIFICATION_PLACEMENT 都没打；日志 $placement_log" >&2
    grep -h -A 3 'panicked at' "$placement_log" 2>/dev/null | head -8 | sed 's/^/    /' >&2
    echo "  → 怎么办：照上面 panicked 那几行改（建不出表、点名的卡用不了、内核超了预算都在那里说）；GPU_CARD_NOT_USABLE 行点名的卡用不了就查那张卡或改显卡配置" >&2
    return 2
  fi
  local_speed="$(crash_amplification_placement_rate "$placement_log")"
  local fields="pending_states=$pending states_per_second_on_this_host=$local_speed across_machines=$across quiesced=$quiesced"
  if { [[ "$across" != 1 ]] && ! item_break_is ignore-across-switch; } || item_break_is stay-local-when-across; then
    echo "CRASH_AMPLIFICATION_DECISION where=local reason=across-machines-is-off $fields"; return 0
  fi
  decided_local_states_per_second="$local_speed"
  echo "CRASH_AMPLIFICATION_DECISION where=two-hosts reason=across-machines-is-on $fields"
  return 10
}

run_selftest() {
  local work failures=0 cells=0
  work="$(mktemp -d "${TMPDIR:-/tmp}/crash-amplification-item-selftest.XXXXXX")"
  trap 'rm -rf -- "${work:?}"' EXIT
  selftest_cell_failed() { # <这一格判错的一句> <出路>
    echo "  ✗ 自证：$1" >&2
    echo "     → 怎么办：$2" >&2
    failures=$((failures + 1))
  }
  placement_line() { # <要判的> <本机速度>：估的用时 = 要判的 ÷ 速度
    echo "CRASH_AMPLIFICATION_PLACEMENT host=local judge=gpu pending_blocks=4 pending_states=$1 tables_mebibytes=10 cards=local:0:100:100,peer:2:50:50 states_placed_on_local=100 states_placed_on_peer=50 states_per_second_on_this_host=$2 estimated_seconds_on_this_host=$(( $1 / $2 ))"
  }
  decision_of() { # <placement.log> <gpu,双机> <显卡配置的值>：打 where= 的值与退出码
    local output exit_code
    output="$(CRASH_AMPLIFICATION_ITEM_DECIDE_ONLY="$1" CRASH_AMPLIFICATION_ITEM_SWITCHES="$2" CRASH_AMPLIFICATION_ITEM_CARDS="$3" bash "$0" some-target 2>/dev/null)"; exit_code=$?
    printf '%s %s\n' "$(grep -o 'where=[a-z-]*' <<<"$output" | head -1)" "$exit_code"
  }
  local both="local:0:15G peer:2:10G" local_only="local:0:15G"
  placement_line 1000 10 >"$work/plan.log"
  # ① 双机开着、两台都有卡 ⇒ 两台跑（不管本机多快）
  cells=$((cells + 1))
  [[ "$(decision_of "$work/plan.log" 1,1 "$both")" == "where=two-hosts 0" ]] \
    || selftest_cell_failed "双机开着却没两台跑：$(decision_of "$work/plan.log" 1,1 "$both")" "看 decide_where_to_run：双机开着一律交双机驱动"
  # ② 双机没开、只点名本机的卡 ⇒ 本机一台跑
  cells=$((cells + 1))
  [[ "$(decision_of "$work/plan.log" 1,0 "$local_only")" == "where=local 0" ]] \
    || selftest_cell_failed "双机没开却没本机一台跑：$(decision_of "$work/plan.log" 1,0 "$local_only")" "看 decide_where_to_run 里 across 那一支"
  # ③ 双机没开、显卡配置点名了第二台的卡 ⇒ 拒绝，退 2
  cells=$((cells + 1))
  [[ "$(decision_of "$work/plan.log" 1,0 "$both")" == " 2" ]] \
    || selftest_cell_failed "双机没开、点名了第二台的卡却没拒：$(decision_of "$work/plan.log" 1,0 "$both")" "看 check_the_cards_against_the_hosts 里 names_peer 那一支"
  # ④ 双机开着、显卡配置里没有第二台的卡 ⇒ 拒绝，退 2
  cells=$((cells + 1))
  [[ "$(decision_of "$work/plan.log" 1,1 "$local_only")" == " 2" ]] \
    || selftest_cell_failed "双机开着、第二台没有点名的卡却没拒：$(decision_of "$work/plan.log" 1,1 "$local_only")" "看 check_the_cards_against_the_hosts 里 across 那一支"
  # ⑤ 一条用例几条流：要判的加起来，速度是合计的状态 ÷ 合计的用时（两条流 400 态每秒 10 态、400 态每秒 40 态 ⇒ 800 态、50 秒、每秒 16 态；
  #   把各行的速度直接相加是 50，只取第一行是 10）
  cells=$((cells + 1)); { placement_line 400 10; placement_line 400 40; } >"$work/two-flows.log"
  local two_flows
  two_flows="$(CRASH_AMPLIFICATION_ITEM_DECIDE_ONLY="$work/two-flows.log" CRASH_AMPLIFICATION_ITEM_SWITCHES=1,1 CRASH_AMPLIFICATION_ITEM_CARDS="$both" bash "$0" some-target 2>/dev/null)"
  grep -q 'pending_states=800 states_per_second_on_this_host=16 ' <<<"$two_flows" \
    || selftest_cell_failed "两条流的要判状态没加起来、或速度没按合计的状态 ÷ 合计的用时算：$two_flows" "看 crash_amplification_placement_total 与 crash_amplification_placement_rate"
  # ⑥ 一行计划都没有 ⇒ 退 2，不猜
  cells=$((cells + 1)); echo "test result: FAILED" >"$work/none.log"
  [[ "$(decision_of "$work/none.log" 1,1 "$both")" == " 2" ]] \
    || selftest_cell_failed "没有计划行却没退 2：$(decision_of "$work/none.log" 1,1 "$both")" "看 decide_where_to_run 开头 pending 为空那一支"
  # ⑦ 两台跑、第二台没量出速度（双机驱动退 249）⇒ 原样退 249，本机不再单独跑
  cells=$((cells + 1))
  cat >"$work/local-run.sh" <<'STUB'
#!/usr/bin/env bash
# 假的本机那一趟：排计划的那一次写一行计划，真跑的那一次写一行汇总；每次调记一笔
log="$3"
if [[ " $* " == *" SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1 "* ]]; then
  echo "placement" >> "${STUB_CALLS:?}"
  echo "CRASH_AMPLIFICATION_PLACEMENT host=local judge=gpu pending_blocks=4 pending_states=1000 tables_mebibytes=10 cards=local:0:100:100,peer:2:50:50 states_placed_on_local=100 states_placed_on_peer=50 states_per_second_on_this_host=1 estimated_seconds_on_this_host=1000" > "$log"
else
  echo "local-run" >> "${STUB_CALLS:?}"
  echo "CRASH_AMPLIFICATION mode=two_hosts_cpu_with_gpus states_checked_here=1000 red=0" > "$log"
fi
STUB
  cat >"$work/two-hosts.sh" <<'STUB'
#!/usr/bin/env bash
# 假的双机驱动：第二台没量出速度
echo "two-hosts" >> "${STUB_CALLS:?}"
echo "CRASH_AMPLIFICATION_PEER_PLACEMENT_FAILED exit=250"
exit 249
STUB
  : > "$work/calls"
  local failed_output failed_exit
  failed_output="$(STUB_CALLS="$work/calls" CRASH_AMPLIFICATION_ITEM_SWITCHES=1,1 CRASH_AMPLIFICATION_ITEM_CARDS="$both" CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" \
    CRASH_AMPLIFICATION_TWO_HOSTS_DRIVER="$work/two-hosts.sh" bash "$0" some-target --library "$work/library" 2>/dev/null)"; failed_exit=$?
  if [[ "$failed_exit" != 249 ]] || [[ "$(tr '\n' ' ' <"$work/calls")" != "placement two-hosts " ]] || grep -q 'where=local' <<<"$failed_output"; then
    selftest_cell_failed "第二台没量出速度时没有原样退 249、或改在本机一台跑了（退 $failed_exit，调用次序「$(tr '\n' ' ' <"$work/calls")」）" "看末尾那个 case 里双机驱动退出之后那一段：原样退它的退出码"
  fi
  if (( failures > 0 )); then
    echo "  ✗ --selftest：$cells 格里 $failures 格判错" >&2
    echo "     → 怎么办：照上面每一格给的出路改，再跑 $0 --selftest" >&2
    exit 1
  fi
  echo "  ✓ --selftest：双机开着两台跑、双机没开本机跑、没开双机却点名第二台的卡拒、开了双机第二台没卡拒、几条流累加、没有计划行退 2、第二台量不出原样退 249，$cells 格都对"
  exit 0
}
if (( selftest )); then run_selftest; fi

[[ -n "$test_target" ]] || usage

# 只判不跑（自证用）：拿造好的日志与开关判在哪跑
if [[ -n "${CRASH_AMPLIFICATION_ITEM_DECIDE_ONLY:-}" ]]; then
  IFS=, read -r decide_gpu decide_across <<<"${CRASH_AMPLIFICATION_ITEM_SWITCHES:?只判不跑要给 CRASH_AMPLIFICATION_ITEM_SWITCHES=<gpu>,<双机>}"
  check_the_cards_against_the_hosts "$decide_across" "${CRASH_AMPLIFICATION_ITEM_CARDS:-}" || exit 2
  decide_where_to_run "$CRASH_AMPLIFICATION_ITEM_DECIDE_ONLY" "$decide_across"; decision=$?
  if (( decision == 10 )); then exit 0; fi
  exit "$decision"
fi

library_is_temporary=0
crash_amplification_prepare || exit 2
if [[ -n "${CRASH_AMPLIFICATION_ITEM_SWITCHES:-}" ]]; then
  IFS=, read -r gpu_switch across_machines_switch <<<"$CRASH_AMPLIFICATION_ITEM_SWITCHES"
fi
[[ -n "${CRASH_AMPLIFICATION_ITEM_CARDS:-}" ]] && gpu_cards_by_priority="$CRASH_AMPLIFICATION_ITEM_CARDS"
if [[ -n "${CRASH_AMPLIFICATION_ITEM_LOCAL_RUN:-}" ]]; then
  crash_amplification_run_on_this_host() { bash "$CRASH_AMPLIFICATION_ITEM_LOCAL_RUN" "$@"; }
fi
crash_amplification_home="${SINGLEFS_CRASH_AMPLIFICATION_HOME:-$HOME/.local/share/singlefs/crash-amplification}"
local_library="${library_directory:-$crash_amplification_home/library}"
run_directory="$(dirname "$local_library")/runs/$label"
mkdir -p "$run_directory" || { echo "  ✗ 建不了 $run_directory" >&2; echo "  → 怎么办：看库的上层目录可不可写" >&2; exit 2; }
two_hosts_driver="${CRASH_AMPLIFICATION_TWO_HOSTS_DRIVER:-$item_script_directory/crash-amplification-two-hosts.sh}"

run_on_this_host_alone() { # 本机一台跑整份，打它的汇总行，退它的退出码
  crash_amplification_run_on_this_host 0/1 "$local_library" "$run_directory/run.log"; local run_exit=$?
  grep -hE '^CRASH_AMPLIFICATION(_GPU_CARD|_GPU_TOTAL)? ' "$run_directory/run.log" 2>/dev/null
  if [[ "$run_exit" != 0 ]]; then
    echo "  ✗ 本机那一趟退 $run_exit；日志 $run_directory/run.log" >&2
    grep -h -A 3 'panicked at' "$run_directory/run.log" 2>/dev/null | head -8 | sed 's/^/    /' >&2
    echo "  → 怎么办：看日志里 test result、panicked 与 run-with-memory-cap 那几行（250–254 是内存包装自己的结局）；判过的块已经在库里，重跑同一条命令接着判" >&2
  fi
  return "$run_exit"
}

if (( ! gpu_switch )); then
  if (( across_machines_switch )); then
    echo "CRASH_AMPLIFICATION_DECISION where=two-hosts-cpu reason=gpu-is-off-and-across-machines-is-on"
    bash "$two_hosts_driver" "$test_target" "${forwarded_arguments[@]+"${forwarded_arguments[@]}"}" --label "$label"
    exit $?
  fi
  echo "CRASH_AMPLIFICATION_DECISION where=local-cpu reason=gpu-is-off-and-across-machines-is-off"
  run_on_this_host_alone
  exit $?
fi

# ② 开了 GPU：先核显卡配置与双机开关，再排派活计划
if [[ -z "$gpu_cards_by_priority" ]]; then
  echo "  ✗ 配置开了 GPU，显卡配置里却没写 GPU_CARDS_BY_PRIORITY$( (( quiesced )) && echo "_WHEN_QUIESCED")（显卡配置：${gpu_cards_configuration_file:-不在}）" >&2
  echo "  → 怎么办：照仓根 gpu-cards.env.example 写每台机器用哪几张卡、各多少显存" >&2
  exit 2
fi
check_the_cards_against_the_hosts "$across_machines_switch" "$gpu_cards_by_priority" || exit 2
crash_amplification_run_on_this_host 0/1 "$local_library" "$run_directory/placement.log" SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1
grep -h '^CRASH_AMPLIFICATION_PLACEMENT ' "$run_directory/placement.log" 2>/dev/null
decided_local_states_per_second=""
decide_where_to_run "$run_directory/placement.log" "$across_machines_switch"; decision=$?
case "$decision" in
  0) run_on_this_host_alone; exit $? ;;
  10)
    two_hosts_arguments=("$test_target" "${forwarded_arguments[@]+"${forwarded_arguments[@]}"}" --label "$label-two-hosts")
    [[ -n "$decided_local_states_per_second" && "$decided_local_states_per_second" != 0 ]] && two_hosts_arguments+=(--local-states-per-second "$decided_local_states_per_second")
    bash "$two_hosts_driver" "${two_hosts_arguments[@]}"; two_hosts_exit=$?
    if [[ "$two_hosts_exit" == 249 ]] && item_break_is local-run-when-the-second-host-fails; then
      echo "CRASH_AMPLIFICATION_DECISION where=local reason=the-second-host-did-not-calibrate"
      run_on_this_host_alone
      exit $?
    fi
    if [[ "$two_hosts_exit" != 0 ]]; then
      echo "  ✗ 两台跑那一趟退 $two_hosts_exit（249 是第二台没量出速度）：配置了两台就要两台都跑成，不改在本机一台跑" >&2
      echo "  → 怎么办：看双机驱动日志目录（$(dirname "$local_library")/runs/$label-two-hosts/）里第二台那几份的 panicked、GPU_CARD_NOT_USABLE 与 run-with-memory-cap 行，修好第二台再重跑同一条命令" >&2
    fi
    exit "$two_hosts_exit" ;;
  *) exit 2 ;;
esac
