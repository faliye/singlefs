#!/usr/bin/env bash
# admission: always 每次调都在这一刻的树与库上排派活计划、再跑；哪些块要重判由库里的块键与判法版本定，不在这里
# run-condition: command bash cargo python3
# run-condition: check source "$PREFLIGHT_SCRIPT_DIRECTORY/crash-amplification-lib.sh" && crash_amplification_run_condition_holds "$PREFLIGHT_SCRIPT_DIRECTORY/../.." :: 修好设备，或改配置：多机配置开了 GPU 而显卡配置一份卡都没写、点名的卡与双机开关对不上，或 GPU_COMPUTATION_CLASSES 没写、写了认不得的类名；单跑 bash -c 'source research/scripts/crash-amplification-lib.sh && crash_amplification_run_condition_holds' 看是哪一处
# 崩溃放量一个分项的一趟：配置定几台、用不用 GPU，这里照配置跑，做不到就报错退出，不退回少一台、不退回 CPU
# （用户 2026-09-29 定：「配置中一台机器就要跑一台，配置中两台机器就要跑两台，配置中有gpu就一定要让GPU工作 如果不行就明确报错拒绝，不接受默认降级」）。
#
#   crash-amplification-item.sh <测试目标> [--test-filter <用例函数名>] [--library <库目录>] [--expand-up-to <N>] [--threads <N>]
#                               [--memory-cap <本机上限>] [--peer-memory-cap <第二台上限>] [--label <标签>] [--quiesced]
#   crash-amplification-item.sh --selftest      拿造出来的派活计划行与日志走一遍「在哪跑」的判法、运行条件与失败分类、第二台失败时的去向（不编、不跑、不上卡、不连第二台）
#
# 测试目标是 crates/singlefs-checker-tier/tests/ 下一条走 crash_amplification::run_from_environment 的用例。
# 运行条件分两半：配置上判得了的（开了 GPU 却一份卡都没写、点名的卡与双机开关对不上、GPU_COMPUTATION_CLASSES 没写或写了认不得的类名）由文件头的 run-condition: check 在开跑之前判，
#   带 --force 越过时在运行中照判、退 78；跑起来才知道的（点名的卡用不了、显存装不下、第二台连不上）在运行中退 78。出路都是「修好设备，或改配置」。
#   算法上判不了的（有状态 GPU 判不了、内核展开超预算、建不出表）不是运行条件，退 248。日志怎么分这几类见 crash-amplification-lib.sh 的 crash_amplification_failure_kind。
# 步骤：
#   ① 配置（${SINGLEFS_MULTI_HOST_CONFIG:-仓根 multi-host.env}）没开 GPU：开了双机交 crash-amplification-two-hosts.sh 两台跑，没开本机 CPU 判。
#   ② 开了 GPU：先核这一趟用的那份显卡配置（GPU_CARDS_BY_PRIORITY，--quiesced 时取 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED）写了没有、与双机开关对不对得上——
#      双机开着，两台都要有点名的卡；双机没开，不许点名第二台的卡。没写或对不上退 78。
#      再在本机跑一趟只排派活计划（SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1，不判），读它打的 CRASH_AMPLIFICATION_PLACEMENT 行
#      （一条用例里几条流各一行，逐个字段加起来）：要判的状态数 pending_states、本机的卡试判量到的速度 states_per_second_on_this_host。
#      一行都没有就不跑，打出日志里 panicked 那几行，照日志分类退：设备用不上 78、算法上判不了 248、那一趟退了非 0 而认不出原因退它的退出码、退 0 却没打计划行 247。
#   ③ 双机没开：本机一台跑（SHARE 0/1）。双机开着：交 crash-amplification-two-hosts.sh，带 --local-states-per-second <本机量到的速度>
#      （它让第二台也量一次，两台按速度分）。第二台没跑成：原样退双机驱动的退出码（78、248、249 见下），不改在本机一台跑。
#   本机一台跑的那一趟退非 0，照 run.log 分类：设备用不上 78、算法上判不了 248，别的原样退它的退出码。
# 显卡的先后与额度：带 --quiesced（调用方清过场）用清场之后那一份，不带用平时那一份；清场与复原归调用方
#   （research/scripts/crash-amplification.sh 开跑就清、退出时复原）。
# 库与日志：库默认 ${SINGLEFS_CRASH_AMPLIFICATION_HOME:-~/.local/share/singlefs/crash-amplification}/library（开机不清空，重跑同一条命令就是续跑），
#   日志在 <库的上层>/runs/<标签>/：placement.log（排计划那一趟）、run.log（本机一台跑的那一趟）；两台跑时是双机驱动那几份。
# 输出：CRASH_AMPLIFICATION_PLACEMENT 行原样转打；一行 CRASH_AMPLIFICATION_DECISION where=<local|two-hosts|local-cpu|two-hosts-cpu> reason=<为什么> …；
#   那一趟的 CRASH_AMPLIFICATION、CRASH_AMPLIFICATION_GPU_CARD、CRASH_AMPLIFICATION_GPU_TOTAL 行原样转打。
# 退出码：
#   0        跑的那一趟退 0
#   2        用法错（参数认不得、缺测试目标、带值的选项缺值：--expand-up-to、--test-filter、--library、--threads、--memory-cap、
#            --peer-memory-cap、--label 后面没给值，或给的是空串、以 - 起头）
#   78       运行条件不满足：开了 GPU 却没写这一趟用的那份显卡配置、点名的卡与双机开关对不上、开了 GPU 而显卡配置的 GPU_COMPUTATION_CLASSES 没写或写了认不得的类名
#            （开跑前由 run-condition: check 拒；带 --force 越过时 crash_amplification_prepare 照样退 78）、说是清过场却没写清场之后那一份，
#            点名的卡用不了或显存装不下（照日志分类），双机驱动退 78（第二台连不上、清场没过、库没取回）。出路：修好设备，或改配置
#   247      排派活计划那一趟退 0，却一行 CRASH_AMPLIFICATION_PLACEMENT 都没打，日志里也认不出是设备还是算法
#   248      算法上判不了：有状态 GPU 判不了（表越过内核容量）、内核展开超预算、GPU 判器建不出表（照日志分类，或双机驱动退 248）
#   249      第二台排派活计划没量出速度、日志里认不出原因（双机驱动原样）
#   250–254  内存包装自己的结局（research/scripts/run-with-memory-cap.sh 文件头「退出码」）
#   其余     跑的那一趟（cargo test）的退出码，原样
# 重型：跑 checker 档用例，只在用户要求时跑（SINGLEFS_HEAVY_TESTS=user-request 由这里带给 cargo）。
# 弄坏开关 CRASH_AMPLIFICATION_ITEM_BREAK=<项>（只给 --selftest 证红用；格号是 run_selftest 里的第 1–14 格，判错的那一格打「✗ 自证第 <号> 格」）：
#   stay-local-when-across             双机开着也本机一台跑（「配置两台就跑两台」判错）                         第 1 格、第 7 格
#   ignore-across-switch               双机没开也动用第二台（「配置一台就跑一台」判错）                         第 2 格、第 11 格
#   accept-peer-cards-without-across   双机没开、显卡配置却点名了第二台的卡也照跑（「配置自相矛盾就拒绝」判错）   第 3 格
#   accept-missing-peer-cards          双机开着、显卡配置里却没有第二台的卡也照跑（「两台都要有 GPU 干活」判错）   第 4 格
#   accept-missing-placement           计划行一行都没有也照本机跑（「没有计划行就不跑」判错）                   第 6 格、第 8 格、第 9 格
#   local-run-when-the-second-host-fails  第二台没量出速度时改在本机一台跑（「不退回少一台」判错）              第 7 格
#   device-unusable-as-run-failure     日志里卡用不上的原话不认（不退 78）                                    第 8 格、第 11 格
#   algorithm-refusal-as-run-failure   日志里算法判不了的原话不认（不退 248）                                 第 9 格
#   accept-missing-card-list           开了 GPU、显卡配置一份都没写也照跑（「没写卡就退 78」判错）               第 10 格
#   missing-value-exits-1              带值的选项缺值时退 1（不按用法错退 2）                                    第 12 格
#   run-condition-skips-classes        开跑前的运行条件不判 GPU_COMPUTATION_CLASSES（没写、认不得的类名都放过去，退 0 照跑）  第 13 格、第 14 格
#   （共用库 crash-amplification-lib.sh 里同一个函数的开关：missing-classes-mean-all 打开时红在第 13 格，accept-unknown-computation-class 打开时红在第 14 格）
# 只供测试：CRASH_AMPLIFICATION_ITEM_DECIDE_ONLY=<造好的 placement.log> 时不编不跑，只拿那份日志判在哪跑、打 CRASH_AMPLIFICATION_DECISION 行就退；
#   CRASH_AMPLIFICATION_ITEM_SWITCHES=<gpu>,<双机> 代替读配置里的这两样；CRASH_AMPLIFICATION_ITEM_CARDS=<显卡配置的值> 代替读显卡配置；
#   CRASH_AMPLIFICATION_ITEM_LOCAL_RUN=<脚本> 换掉本机那一趟（调法：<脚本> <share> <库目录> <日志> [<附加环境 键=值>…]，自己写日志）；
#   CRASH_AMPLIFICATION_TWO_HOSTS_DRIVER=<脚本> 换掉双机驱动。换不掉的：真的那一趟由 crash-amplification-lib.sh 起 cargo，自证不碰它。
#
# gate-similar: crash-amplification-two-hosts.sh 它管两台各领一份、取回、导入那一趟；这里核配置、排派活计划、定一台还是两台，本机一台跑的那一趟自己跑
# gate-similar: crash-amplification.sh 它是分项登记表与全量的循环，清场与复原在它那里，每项调这里
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

item_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repository_root="$(cd "$item_script_directory/../.." && pwd)"
source "$item_script_directory/crash-amplification-lib.sh"

item_break_is() { crash_amplification_item_break_is "$1"; }

usage() { # [<哪里不对>]
  echo "  ✗ ${1:+$1；}用法：$0 <测试目标> [--test-filter <用例函数名>] [--library <库目录>] [--expand-up-to <N>] [--threads <N>] [--memory-cap <本机上限>] [--peer-memory-cap <第二台上限>] [--label <标签>] [--quiesced]，或 --selftest" >&2
  echo "  → 怎么办：测试目标写 crates/singlefs-checker-tier/tests/ 下的文件名（不带 .rs），例 crash_amplification_fixed_script_through_rollback；带值的选项后面紧跟它的值（不以 - 起头）" >&2
  exit 2
}

# 带值的选项：后面没给值、给的是空串或以 - 起头，按用法错退 2（文件头「退出码」）
require_value() { # <选项> <余下的参数个数（含选项自己）> [<值>]
  if (( $2 >= 2 )) && [[ -n "${3:-}" && "${3:-}" != -* ]]; then return 0; fi
  if item_break_is missing-value-exits-1; then
    echo "  ✗ $1 后面缺值" >&2
    echo "  → 怎么办：在 $1 后面给值" >&2
    exit 1
  fi
  usage "$1 后面缺值"
}

# 本机上限：留在 slice 总上限（整机 − 余量 20G = 40.1G）之内，给 38G（给满 40G 时 slice 里残留的几百 MiB 就让它排不进队）
test_target=""; expand_up_to=""; memory_cap="38G"; peer_memory_cap=""; label="crash-amplification-$(date -u +%Y%m%dT%H%M%SZ)-$$"; selftest=0; threads=""; library_directory=""; test_filter=""; quiesced=0
forwarded_arguments=()
while (($# > 0)); do
  case "$1" in
    --expand-up-to) require_value "$1" $# "${2:-}"; expand_up_to="$2"; forwarded_arguments+=("$1" "$2"); shift 2 ;;
    --test-filter) require_value "$1" $# "${2:-}"; test_filter="$2"; forwarded_arguments+=("$1" "$2"); shift 2 ;;
    --library) require_value "$1" $# "${2:-}"; library_directory="$2"; forwarded_arguments+=("$1" "$2"); shift 2 ;;
    --threads) require_value "$1" $# "${2:-}"; threads="$2"; forwarded_arguments+=("$1" "$2"); shift 2 ;;
    --memory-cap) require_value "$1" $# "${2:-}"; memory_cap="$2"; forwarded_arguments+=("$1" "$2"); shift 2 ;;
    --peer-memory-cap) require_value "$1" $# "${2:-}"; peer_memory_cap="$2"; forwarded_arguments+=("$1" "$2"); shift 2 ;;
    --label) require_value "$1" $# "${2:-}"; label="$2"; shift 2 ;;
    --quiesced) quiesced=1; forwarded_arguments+=("$1"); shift ;;
    --selftest) selftest=1; shift ;;
    -*) usage ;;
    *) if [[ -n "$test_target" ]]; then usage; fi; test_target="$1"; shift ;;
  esac
done

# 在哪跑：读 placement.log 与双机开关，打 CRASH_AMPLIFICATION_DECISION 行；交回 0 本机一台跑、10 两台跑、
# 1 没有计划行（该退的码放进 refused_exit_code：设备用不上 78、算法上判不了 248、那一趟退了非 0 就是它的退出码、退 0 却没打计划行 247）
decide_where_to_run() { # <placement.log> <双机开关 0/1> <排计划那一趟的退出码>
  local placement_log="$1" across="$2" placement_exit="$3"
  local pending local_speed kind
  pending="$(crash_amplification_placement_total "$placement_log" pending_states)"
  if [[ -z "$pending" ]]; then
    if item_break_is accept-missing-placement; then
      echo "CRASH_AMPLIFICATION_DECISION where=local reason=no-placement-line"
      return 0
    fi
    echo "  ✗ 排派活计划那一趟（退 $placement_exit）一行 CRASH_AMPLIFICATION_PLACEMENT 都没打，不跑；日志 $placement_log" >&2
    grep -h -A 3 'panicked at' "$placement_log" 2>/dev/null | head -8 | sed 's/^/    /' >&2
    kind="$(crash_amplification_failure_kind "$placement_log")"
    case "$kind" in
      device) refused_exit_code=78 ;;
      algorithm) refused_exit_code=248 ;;
      *) if [[ "$placement_exit" != 0 ]]; then refused_exit_code="$placement_exit"; else refused_exit_code=247; fi ;;
    esac
    if [[ "$refused_exit_code" == 78 || "$refused_exit_code" == 248 ]]; then
      crash_amplification_explain_exit_code "$refused_exit_code"
    else
      echo "  → 怎么办：照上面 panicked 那几行与日志全文改（250–254 是内存包装自己的结局）；日志里认不出是设备还是算法，单跑那一趟看全文（退 $refused_exit_code）" >&2
    fi
    return 1
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
  # 子进程的运行条件只认这里给的：多机配置与显卡配置指到不在的文件（两个开关都关、一份卡都没写），开关与卡由各格的 CRASH_AMPLIFICATION_ITEM_* 给
  #   显卡配置里只写 GPU_COMPUTATION_CLASSES（开了 GPU 的格要它过运行条件那一关），卡还是一份都没写；第 13、14 格另指到缺这个键与写了认不得类名的两份
  export SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host.env" SINGLEFS_GPU_CARDS_CONFIG="$work/gpu-cards.env"
  printf '%s\n' "GPU_COMPUTATION_CLASSES='gpu_unit_checks crash_verify_gpu crash_judge_gpu'" >"$work/gpu-cards.env"
  printf '%s\n' 'GPU_CARDS_BY_PRIORITY="local:0:15G"' >"$work/gpu-cards-without-classes.env"
  printf '%s\n' 'GPU_CARDS_BY_PRIORITY="local:0:15G"' "GPU_COMPUTATION_CLASSES='gpu_unit_checks crash_verify_cpu'" >"$work/gpu-cards-unknown-class.env"
  selftest_cell_failed() { # <这一格判错的一句> <出路>
    echo "  ✗ 自证第 $cells 格：$1" >&2
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
  # 第 1 格：双机开着、两台都有卡 ⇒ 两台跑（不管本机多快）
  cells=$((cells + 1))
  [[ "$(decision_of "$work/plan.log" 1,1 "$both")" == "where=two-hosts 0" ]] \
    || selftest_cell_failed "双机开着却没两台跑：$(decision_of "$work/plan.log" 1,1 "$both")" "看 decide_where_to_run：双机开着一律交双机驱动"
  # 第 2 格：双机没开、只点名本机的卡 ⇒ 本机一台跑
  cells=$((cells + 1))
  [[ "$(decision_of "$work/plan.log" 1,0 "$local_only")" == "where=local 0" ]] \
    || selftest_cell_failed "双机没开却没本机一台跑：$(decision_of "$work/plan.log" 1,0 "$local_only")" "看 decide_where_to_run 里 across 那一支"
  # 第 3 格：双机没开、显卡配置点名了第二台的卡 ⇒ 运行条件不满足，退 78（preflight 先拒；带 --force 时脚本里照判）
  cells=$((cells + 1))
  [[ "$(decision_of "$work/plan.log" 1,0 "$both")" == " 78" ]] \
    || selftest_cell_failed "双机没开、点名了第二台的卡却没退 78：$(decision_of "$work/plan.log" 1,0 "$both")" "看 crash-amplification-lib.sh 的 crash_amplification_cards_match_the_hosts 里 names_peer 那一支"
  # 第 4 格：双机开着、显卡配置里没有第二台的卡 ⇒ 退 78
  cells=$((cells + 1))
  [[ "$(decision_of "$work/plan.log" 1,1 "$local_only")" == " 78" ]] \
    || selftest_cell_failed "双机开着、第二台没有点名的卡却没退 78：$(decision_of "$work/plan.log" 1,1 "$local_only")" "看 crash_amplification_cards_match_the_hosts 里 across 那一支"
  # 第 5 格：一条用例几条流：要判的加起来，速度是合计的状态 ÷ 合计的用时（两条流 400 态每秒 10 态、400 态每秒 40 态 ⇒ 800 态、50 秒、每秒 16 态；
  #   把各行的速度直接相加是 50，只取第一行是 10）
  cells=$((cells + 1)); { placement_line 400 10; placement_line 400 40; } >"$work/two-flows.log"
  local two_flows
  two_flows="$(CRASH_AMPLIFICATION_ITEM_DECIDE_ONLY="$work/two-flows.log" CRASH_AMPLIFICATION_ITEM_SWITCHES=1,1 CRASH_AMPLIFICATION_ITEM_CARDS="$both" bash "$0" some-target 2>/dev/null)"
  grep -q 'pending_states=800 states_per_second_on_this_host=16 ' <<<"$two_flows" \
    || selftest_cell_failed "两条流的要判状态没加起来、或速度没按合计的状态 ÷ 合计的用时算：$two_flows" "看 crash_amplification_placement_total 与 crash_amplification_placement_rate"
  # 第 6 格：一行计划都没有、日志里认不出原因、那一趟退 0 ⇒ 不跑，退 247
  cells=$((cells + 1)); echo "test result: FAILED" >"$work/none.log"
  [[ "$(decision_of "$work/none.log" 1,1 "$both")" == " 247" ]] \
    || selftest_cell_failed "没有计划行、认不出原因却没退 247：$(decision_of "$work/none.log" 1,1 "$both")" "看 decide_where_to_run 开头 pending 为空那一支：分不出类、那一趟退 0 时 refused_exit_code=247"
  # 第 7 格：两台跑、第二台没量出速度（双机驱动退 249）⇒ 原样退 249，本机不再单独跑
  cells=$((cells + 1))
  cat >"$work/local-run.sh" <<'STUB'
#!/usr/bin/env bash
# 假的本机那一趟：排计划的那一次写一行计划，真跑的那一次写一行汇总（STUB_RUN_DEVICE_FAILURE=1 时写一行卡用不了、退 101）；每次调记一笔
log="$3"
if [[ " $* " == *" SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1 "* ]]; then
  echo "placement" >> "${STUB_CALLS:?}"
  echo "CRASH_AMPLIFICATION_PLACEMENT host=local judge=gpu pending_blocks=4 pending_states=1000 tables_mebibytes=10 cards=local:0:100:100,peer:2:50:50 states_placed_on_local=100 states_placed_on_peer=50 states_per_second_on_this_host=1 estimated_seconds_on_this_host=1000" > "$log"
else
  echo "local-run" >> "${STUB_CALLS:?}"
  if [[ "${STUB_RUN_DEVICE_FAILURE:-0}" == 1 ]]; then
    echo "GPU_CARD_NOT_USABLE index=0：配置点名了这张卡，本机却用不了它" > "$log"
    exit 101
  fi
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
  # 第 8 格：排计划那一趟没打计划行，日志里有卡用不了的原话 ⇒ 运行条件不满足，退 78
  cells=$((cells + 1))
  { echo "GPU_CARD_NOT_USABLE index=3：配置点名了这张卡，本机却用不了它（没有这个序号、空闲显存低于 1024 MiB、或上下文起不来）"
    echo "thread 'the_case' panicked at crates/singlefs-checker-tier/src/crash_amplification.rs:1:1:"
    echo "配置点名给 local 这台机器的卡里，序号 [3] 用不了（上面 GPU_CARD_NOT_USABLE 那几行）"
    echo "test result: FAILED"; } >"$work/device.log"
  [[ "$(decision_of "$work/device.log" 1,0 "$local_only")" == " 78" ]] \
    || selftest_cell_failed "卡用不了却没退 78：$(decision_of "$work/device.log" 1,0 "$local_only")" "看 crash-amplification-lib.sh 的 crash_amplification_failure_kind 认不认 GPU_CARD_NOT_USABLE，与 decide_where_to_run 里 device 那一支"
  # 第 9 格：排计划那一趟没打计划行，日志里是算法判不了的原话（表越过内核容量）⇒ 退 248，不算运行条件
  cells=$((cells + 1))
  { echo "thread 'the_case' panicked at crates/singlefs-checker-tier/src/crash_amplification.rs:1:1:"
    echo "卡 0 判不了点 p 块内第 5 个状态（恢复内核第 1 字 7、checker 内核第 3 字 9：表越过了内核的容量）。配置开了 GPU，不退回 CPU 判器"
    echo "test result: FAILED"; } >"$work/algorithm.log"
  [[ "$(decision_of "$work/algorithm.log" 1,0 "$local_only")" == " 248" ]] \
    || selftest_cell_failed "算法判不了却没退 248：$(decision_of "$work/algorithm.log" 1,0 "$local_only")" "看 crash_amplification_failure_kind 认不认「表越过了内核的容量」，与 decide_where_to_run 里 algorithm 那一支"
  # 第 10 格：开了 GPU、显卡配置一份都没写 ⇒ 运行条件不满足，退 78
  cells=$((cells + 1))
  [[ "$(decision_of "$work/plan.log" 1,0 "")" == " 78" ]] \
    || selftest_cell_failed "开了 GPU、一份卡都没写却没退 78：$(decision_of "$work/plan.log" 1,0 "")" "看 crash-amplification-lib.sh 的 crash_amplification_run_condition_holds 里 lists 为空那一支"
  # 第 11 格：本机一台跑的那一趟退 101、日志里有卡用不了的原话 ⇒ 退 78，不原样退 101
  cells=$((cells + 1))
  : > "$work/calls"
  local device_run_exit
  STUB_CALLS="$work/calls" STUB_RUN_DEVICE_FAILURE=1 CRASH_AMPLIFICATION_ITEM_SWITCHES=1,0 CRASH_AMPLIFICATION_ITEM_CARDS="$local_only" \
    CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" CRASH_AMPLIFICATION_TWO_HOSTS_DRIVER="$work/two-hosts.sh" \
    bash "$0" some-target --library "$work/library-device" >/dev/null 2>&1; device_run_exit=$?
  if [[ "$device_run_exit" != 78 ]] || [[ "$(tr '\n' ' ' <"$work/calls")" != "placement local-run " ]]; then
    selftest_cell_failed "本机那一趟卡用不了却没退 78（退 $device_run_exit，调用次序「$(tr '\n' ' ' <"$work/calls")」）" "看 run_on_this_host_alone：退非 0 时经 crash_amplification_exit_code_of_a_failed_run 照 run.log 分类"
  fi
  # 第 12 格：带值的选项缺值（放在末尾、或后面紧跟 --quiesced）：退 2（用法错）；各个带值的选项各试两次。
  #   给了只判不跑的日志与开关：万一参数照收了，走的也是只判不跑那一支，不编不跑
  cells=$((cells + 1))
  local option missing_value_exit missing_value_wrong=""
  for option in --expand-up-to --test-filter --library --threads --memory-cap --peer-memory-cap --label; do
    CRASH_AMPLIFICATION_ITEM_DECIDE_ONLY="$work/plan.log" CRASH_AMPLIFICATION_ITEM_SWITCHES=0,0 bash "$0" some-target "$option" >/dev/null 2>&1; missing_value_exit=$?
    [[ "$missing_value_exit" == 2 ]] || missing_value_wrong+="$option 放末尾退 $missing_value_exit；"
    CRASH_AMPLIFICATION_ITEM_DECIDE_ONLY="$work/plan.log" CRASH_AMPLIFICATION_ITEM_SWITCHES=0,0 bash "$0" some-target "$option" --quiesced >/dev/null 2>&1; missing_value_exit=$?
    [[ "$missing_value_exit" == 2 ]] || missing_value_wrong+="$option 后面紧跟 --quiesced 退 $missing_value_exit；"
  done
  [[ -z "$missing_value_wrong" ]] \
    || selftest_cell_failed "带值的选项缺值没按用法错退 2：$missing_value_wrong" "看参数那一段的 require_value：缺值、空串、以 - 起头都交 usage（退 2）"
  # 第 13、14 格：开了 GPU、卡写对了，显卡配置里 GPU_COMPUTATION_CLASSES 没写（13）、写了认不得的类名（14）⇒ 开跑前（run-condition: check）就退 78、打出点名这个键的出路，
  #   一趟都不起（假的本机那一趟与双机驱动的调用记录是空的）；不等跑到 crash_amplification_prepare。
  #   开关 run-condition-skips-classes 打开时开跑前放过去：这里的 prepare 读的是没开 GPU 的真配置（不判类），那一趟照跑退 0，格就红
  local classes_cell classes_cards classes_pattern classes_exit
  for classes_cell in 13 14; do
    cells=$((cells + 1))
    # preflight 不转打判据自己的 stderr，只打文件头 :: 之后的那一句出路，所以两格都认那一句里点名的键
    classes_pattern='GPU_COMPUTATION_CLASSES 没写、写了认不得的类名'
    if [[ "$classes_cell" == 13 ]]; then classes_cards="$work/gpu-cards-without-classes.env"; else classes_cards="$work/gpu-cards-unknown-class.env"; fi
    : > "$work/calls"
    STUB_CALLS="$work/calls" SINGLEFS_GPU_CARDS_CONFIG="$classes_cards" CRASH_AMPLIFICATION_ITEM_SWITCHES=1,0 CRASH_AMPLIFICATION_ITEM_CARDS="$local_only" \
      CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" CRASH_AMPLIFICATION_TWO_HOSTS_DRIVER="$work/two-hosts.sh" \
      bash "$0" some-target --library "$work/library-classes-$classes_cell" >"$work/classes-$classes_cell.out" 2>&1; classes_exit=$?
    if [[ "$classes_exit" != 78 ]] || [[ -s "$work/calls" ]] || ! grep -qE -- "$classes_pattern" "$work/classes-$classes_cell.out"; then
      selftest_cell_failed "开了 GPU、GPU_COMPUTATION_CLASSES $( [[ "$classes_cell" == 13 ]] && echo 没写 || echo 写了认不得的类名 )，开跑前却没退 78、点名这个键、一趟都不起（退 $classes_exit，调用次序「$(tr '\n' ' ' <"$work/calls")」）" \
        "看 crash-amplification-lib.sh 的 crash_amplification_run_condition_holds 末尾调 crash_amplification_read_gpu_computation_classes 那一段"
    fi
  done
  if (( failures > 0 )); then
    echo "  ✗ --selftest：$cells 格里 $failures 格判错" >&2
    echo "     → 怎么办：照上面每一格给的出路改，再跑 $0 --selftest" >&2
    exit 1
  fi
  echo "  ✓ --selftest：双机开着两台跑、双机没开本机跑、没开双机却点名第二台的卡退 78、开了双机第二台没卡退 78、几条流累加、没有计划行认不出原因退 247、第二台量不出原样退 249、卡用不了退 78、算法判不了退 248、开了 GPU 没写卡退 78、本机那一趟卡用不了退 78、带值的选项缺值退 2、开了 GPU 而 GPU_COMPUTATION_CLASSES 没写或写了认不得的类名开跑前就退 78，$cells 格都对"
  exit 0
}
if (( selftest )); then run_selftest; fi

[[ -n "$test_target" ]] || usage

# 只判不跑（自证用）：拿造好的日志与开关判在哪跑
if [[ -n "${CRASH_AMPLIFICATION_ITEM_DECIDE_ONLY:-}" ]]; then
  IFS=, read -r decide_gpu decide_across <<<"${CRASH_AMPLIFICATION_ITEM_SWITCHES:?只判不跑要给 CRASH_AMPLIFICATION_ITEM_SWITCHES=<gpu>,<双机>}"
  crash_amplification_cards_match_the_hosts "$decide_across" "${CRASH_AMPLIFICATION_ITEM_CARDS:-}" || exit 78
  refused_exit_code=""
  decide_where_to_run "$CRASH_AMPLIFICATION_ITEM_DECIDE_ONLY" "$decide_across" 0; decision=$?
  case "$decision" in
    0|10) exit 0 ;;
    *) exit "${refused_exit_code:-247}" ;;
  esac
fi

library_is_temporary=0
crash_amplification_prepare || exit 78
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

run_on_this_host_alone() { # 本机一台跑整份，打它的汇总行；退 0，或照 run.log 分类之后该退的码
  crash_amplification_run_on_this_host 0/1 "$local_library" "$run_directory/run.log"; local run_exit=$?
  grep -hE '^CRASH_AMPLIFICATION(_GPU_CARD|_GPU_TOTAL)? ' "$run_directory/run.log" 2>/dev/null
  if [[ "$run_exit" == 0 ]]; then return 0; fi
  local exit_code
  exit_code="$(crash_amplification_exit_code_of_a_failed_run "$run_exit" "$run_directory/run.log")"
  echo "  ✗ 本机那一趟退 $run_exit；日志 $run_directory/run.log" >&2
  grep -h -A 3 'panicked at' "$run_directory/run.log" 2>/dev/null | head -8 | sed 's/^/    /' >&2
  if [[ "$exit_code" == 78 || "$exit_code" == 248 ]]; then
    crash_amplification_explain_exit_code "$exit_code"
  else
    echo "  → 怎么办：看日志里 test result、panicked 与 run-with-memory-cap 那几行（250–254 是内存包装自己的结局）；判过的块已经在库里，重跑同一条命令接着判" >&2
  fi
  return "$exit_code"
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

# ② 开了 GPU：先核这一趟用的那份显卡配置与双机开关，再排派活计划
if [[ -z "$gpu_cards_by_priority" ]] && ! item_break_is accept-missing-card-list; then
  echo "  ✗ 配置开了 GPU，显卡配置里却没写 GPU_CARDS_BY_PRIORITY$( (( quiesced )) && echo "_WHEN_QUIESCED")（显卡配置：${gpu_cards_configuration_file:-不在}）" >&2
  echo "  → 怎么办：修好设备，或改配置：照仓根 gpu-cards.env.example 写每台机器用哪几张卡、各多少显存；不用 GPU 就把多机配置的 ENABLE_GPU 改成 0" >&2
  exit 78
fi
crash_amplification_cards_match_the_hosts "$across_machines_switch" "$gpu_cards_by_priority" || exit 78
crash_amplification_run_on_this_host 0/1 "$local_library" "$run_directory/placement.log" SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1; placement_exit=$?
grep -h '^CRASH_AMPLIFICATION_PLACEMENT ' "$run_directory/placement.log" 2>/dev/null
decided_local_states_per_second=""; refused_exit_code=""
decide_where_to_run "$run_directory/placement.log" "$across_machines_switch" "$placement_exit"; decision=$?
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
      echo "  ✗ 两台跑那一趟退 $two_hosts_exit（78 第二台或卡用不上、248 算法上判不了、249 第二台没量出速度）：配置了两台就要两台都跑成，不改在本机一台跑" >&2
      if [[ "$two_hosts_exit" == 78 || "$two_hosts_exit" == 248 ]]; then
        crash_amplification_explain_exit_code "$two_hosts_exit"
      else
        echo "  → 怎么办：看双机驱动日志目录（$(dirname "$local_library")/runs/$label-two-hosts/）里第二台那几份的 panicked、GPU_CARD_NOT_USABLE 与 run-with-memory-cap 行，修好第二台再重跑同一条命令" >&2
      fi
    fi
    exit "$two_hosts_exit" ;;
  *) exit "${refused_exit_code:-247}" ;;
esac
