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
#   gpu_cards_by_priority_when_quiesced（清场之后那一份，没写是空串）、
#   gpu_computation_classes（走 GPU 的计算类，空白隔开；没开 GPU 是空串）、common_environment、cargo_build_arguments、cargo_test_arguments（数组）
#   crash_amplification_prepare 交回非 0（都是 2）只有这几种，都是配置的事，调用方退 78：说是清过场、显卡配置里却没写清场之后那一份；
#   开了 GPU、显卡配置里却没写 GPU_COMPUTATION_CLASSES，或写了认不得的类名。
# 交给本机那一趟的环境由 crash_amplification_environment_for_this_host 组（数组 this_host_environment，env 的参数）；
#   驱动自证里换掉本机那一趟时也经它组，只把起 cargo 换成假的。
#
# 运行条件与失败分类（崩溃放量入口与分项驱动共用）：
#   crash_amplification_run_condition_holds [<仓根>]   两个驱动文件头 run-condition: check 调它（preflight 用 bash -c 在仓根起）；只判配置：
#       开了 GPU 就要写了显卡、点名的卡与双机开关对得上、GPU_COMPUTATION_CLASSES 写了且类名都认得（判法就是 crash_amplification_prepare 用的
#       crash_amplification_read_gpu_computation_classes，同一个函数两处调）。不连第二台、不碰卡：那两样跑起来才知道，用不上时驱动在运行中退 78。
#   crash_amplification_cards_match_the_hosts <双机 0/1> <显卡配置的值>   双机开着两台都要有点名的卡，双机没开不许点名第二台的卡
#   crash_amplification_failure_kind <日志…>           一趟没跑成，日志里说的是哪一类：device（显卡或显存用不上，运行条件，调用方退 78）、
#       algorithm（算法上判不了：有状态 GPU 判不了、内核展开超预算、建不出表，调用方退 248），都不是打空串；两类都有按 device
#   crash_amplification_exit_code_of_a_failed_run <那一趟的退出码> <日志…>   打调用方该退的码：device 78、algorithm 248，别的原样
#   弄坏开关读 CRASH_AMPLIFICATION_ITEM_BREAK（开关表与该红的格在 research/scripts/crash-amplification-item.sh 文件头）：
#   accept-missing-peer-cards、accept-peer-cards-without-across、accept-missing-card-list、device-unusable-as-run-failure、algorithm-refusal-as-run-failure。
#   其中 device-unusable-as-run-failure、algorithm-refusal-as-run-failure 打开时 research/scripts/crash-amplification-two-hosts.sh --selftest 也红
#   （红在哪几格写在那份文件头「共用库 crash-amplification-lib.sh 的开关」一段）。
#   走 GPU 的计算类那几处的开关同样读 CRASH_AMPLIFICATION_ITEM_BREAK，只由 research/scripts/crash-amplification-two-hosts.sh --selftest 证红
#   （开关表与该红的格在那份文件头「共用库 crash-amplification-lib.sh 的开关」一段）：
#   missing-classes-mean-all、accept-unknown-computation-class、read-classes-when-gpu-off、inherited-classes-kept、classes-not-sent-on-this-host。
#   run-condition-skips-classes（运行条件那一段不判 GPU_COMPUTATION_CLASSES）由 research/scripts/crash-amplification-item.sh --selftest 证红，
#   表与该红的格在那份文件头；missing-classes-mean-all、accept-unknown-computation-class 打开时分项驱动自证里开跑前判这一类的格也红。
#
# 弄坏开关轮（崩溃放量入口与双机驱动的 --selftest 末尾共用；.claude/singlefs-ai-sop/rules/test-script-discipline.md「测试脚本自己也要被测试证明有效」：
# 复跑的一方逐个打开开关、看它红在点名的那一项）：
#   crash_amplification_read_switch_table <脚本文件> <变量名…>
#       读那份文件头的开关表（第一行代码之前的注释块），一个开关打一行「<变量>=<开关名> <TAB> <点名的格，空格隔开>」，照表里的次序。表的写法（B 写法，
#       research/scripts/mutate.sh 文件头「弄坏开关」一段是样板）：一行「#   <变量>=<开关名>  <弄坏哪一处>」（变量是给的变量名之一），下一行起以「#       」
#       开头写点名的格（、隔开；括号里是说明与失败行的句首，不读，括号外的每一段都算一格）。「第 N 格」读成 N（崩溃放量入口的格是编号的）；
#       写错的格名照样读进来，开关轮比格时红。崩溃放量入口与双机驱动的 --selftest 都经它读表。
#   crash_amplification_switch_wheel <放输出的目录> <取格的正则> <只比退出码 0/1> -- <自证命令…>
#       调用方先从自己文件头的开关表现读出开关（crash_amplification_read_switch_table，不在代码里另抄名单），填两个同序数组：switch_wheel_assignments 每个开关一项「<变量>=<开关名>」；
#       switch_wheel_expected 那个开关点名的格，空白隔开，写成「-」表示表里没列它、打开后应当退 0、一格不红。
#       每个开关一个子进程：先 env -u 清掉数组里出现过的每个变量、只设这一个，再设 CRASH_AMPLIFICATION_SWITCH_WHEEL_INNER（指到这一轮写的标记文件；
#       调用方用 crash_amplification_switch_wheel_inner 认它，认出来就不再起开关轮），跑自证命令，标准输出与标准错误并进 <目录>/switch-wheel-<序号>.out，
#       一批一批起、逐个 wait 收退出码。并行度：CRASH_AMPLIFICATION_SELFTEST_JOBS（正整数），没给取 SINGLEFS_THREAD_CAP（research/scripts/capped.sh 设的），再没给取 nproc。
#       对上的逐个打 ·（标准输出），对不上的逐个打 ✗（标准错误，出路由调用方的汇总句给）；交回之后 switch_wheel_failures 是对不上的处数
#       （含派出收回对不上）、switch_wheel_collected 是收回的个数。
#   crash_amplification_switch_wheel_verdict <点名的格> <退出码> <输出文件> <取格的正则> <只比退出码 0/1>
#       一个开关对不对得上：打实测红的格（取格的正则是 sed -E 的写法、一组括号括住格名；去重、sort -V 排好、空格隔开）。点名了格的要退 1、
#       红的格与点名的逐格相等；「-」的要退 0、一格不红；点名的格是空串（表里那一行没写格）一律对不上。
#       第五个参数是 1 时不比格、只比退出码：调用方的弄坏开关用（双机驱动的 wheel-ignores-cells），由它自证的 switch-wheel-judges 格拿造出来的输出证红。
#
# 显卡配置：${SINGLEFS_GPU_CARDS_CONFIG:-${XDG_CONFIG_HOME:-~/.config}/singlefs/gpu-cards.env}（这台开发机私有，不进仓；模板是仓根的 gpu-cards.env.example）。
# 三个键：GPU_CARDS_BY_PRIORITY（不清场时显卡的先后与每张卡的显存额度，读法在 crates/singlefs-checker-tier/src/crash_amplification.rs 的 parse_cards_by_priority）、
# GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED（清场之后的先后与额度，同一种写法；全量开跑就清场，用的是这一份）、
# GPU_COMPUTATION_CLASSES（走 GPU 的计算类，空白隔开；认得的类名是下面 CRASH_AMPLIFICATION_GPU_COMPUTATION_CLASSES_KNOWN 那三个）。
# 挑中的那一份显卡先后经环境变量 GPU_CARDS_BY_PRIORITY 交给用例（本机与第二台同一份，各取点名给自己的卡）。
# 走 GPU 的计算类只在多机配置开了 GPU（ENABLE_GPU=1）时读：没写（键不在、值是空的、显卡配置整份不在）或写了认不得的类名，
# crash_amplification_prepare 打 ✗ 与出路、交回 2，不默认全开；读到的经环境变量 GPU_COMPUTATION_CLASSES 交给用例（本机与第二台两边都交）。
# 没开 GPU 时不读、置空，也不交：本机那一趟起 cargo 前用 env -u 清掉调用方环境里漏进来的同名变量。

CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS="-I/usr/lib/gcc/x86_64-linux-gnu/13/include -I/usr/lib/gcc/x86_64-linux-gnu/14/include"  # librocksdb-sys 的 bindgen 找 stdbool.h；哪个版本的目录在就用哪个，不在的 -I 无害
CRASH_AMPLIFICATION_GPU_COMPUTATION_CLASSES_KNOWN="gpu_unit_checks crash_verify_gpu crash_judge_gpu"  # 认得的走 GPU 的计算类（仓根 gpu-cards.env.example 的 GPU_COMPUTATION_CLASSES 一段逐个写了是什么）

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

# 走 GPU 的计算类：读显卡配置的 GPU_COMPUTATION_CLASSES，空白隔开的类名逐个核过、以一个空格连起来放进 gpu_computation_classes。只在开了 GPU 时调。
# 两处调同一份判法：crash_amplification_prepare（开跑后读、核、留下值）与 crash_amplification_run_condition_holds（开跑前只核，调用方先 local 一个同名变量接住值）。
# 没写（键不在、值是空的、显卡配置整份不在）或有认不得的类名：打 ✗ 与出路、交回 2（不默认全开）；读到了交回 0。
crash_amplification_read_gpu_computation_classes() { # <显卡配置文件，不在可以是空串>
  local cards_file="$1" known="$CRASH_AMPLIFICATION_GPU_COMPUTATION_CLASSES_KNOWN" value word unknown="" words=()
  value="$(crash_amplification_configuration_value "$cards_file" GPU_COMPUTATION_CLASSES)"
  read -r -a words <<<"$value"
  if (( ${#words[@]} == 0 )); then
    if crash_amplification_item_break_is missing-classes-mean-all; then gpu_computation_classes="$known"; return 0; fi
    echo "  ✗ 配置开了 GPU（ENABLE_GPU=1），显卡配置里却没写 GPU_COMPUTATION_CLASSES（哪几类计算走 GPU；显卡配置：${cards_file:-不在}）：不默认全开" >&2
    echo "  → 怎么办：修好设备，或改配置：照仓根 gpu-cards.env.example 的 GPU_COMPUTATION_CLASSES 一段写走 GPU 的计算类（认得的：$known）；不用 GPU 就把多机配置的 ENABLE_GPU 改成 0" >&2
    return 2
  fi
  for word in "${words[@]}"; do
    [[ " $known " == *" $word "* ]] || unknown+="${unknown:+ }$word"
  done
  if [[ -n "$unknown" ]] && ! crash_amplification_item_break_is accept-unknown-computation-class; then
    echo "  ✗ 显卡配置的 GPU_COMPUTATION_CLASSES「$value」里有认不得的类名：$unknown（显卡配置：${cards_file:-不在}）" >&2
    echo "  → 怎么办：修好设备，或改配置：只写认得的类名（$known，各是什么见仓根 gpu-cards.env.example），拼错的改正、不要的删掉" >&2
    return 2
  fi
  gpu_computation_classes="${words[*]}"
  return 0
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
        echo "  → 怎么办：修好设备，或改配置：照仓根 gpu-cards.env.example 在显卡配置里写清场之后的显卡先后与额度" >&2
        return 2
      fi
      gpu_cards_by_priority="$gpu_cards_by_priority_when_quiesced"
    fi
  fi
  # 走 GPU 的计算类：开了 GPU 才读，没开置空、不读（文件头「显卡配置」一段）
  gpu_computation_classes=""
  if (( gpu_switch )); then
    crash_amplification_read_gpu_computation_classes "$gpu_cards_configuration_file" || return 2
  elif crash_amplification_item_break_is read-classes-when-gpu-off; then
    gpu_computation_classes="$(crash_amplification_configuration_value "$gpu_cards_configuration_file" GPU_COMPUTATION_CLASSES)"
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

# 交给本机那一趟的环境（env 的参数）放进数组 this_host_environment：先 -u 清掉调用方环境里漏进来的 GPU_COMPUTATION_CLASSES，
# 再是 common_environment、附加环境、份与库，挑中的那份显卡先后与额度、走 GPU 的计算类（这两样空的不交）。
crash_amplification_environment_for_this_host() { # <share> <库目录> [<附加环境 键=值>…]
  local share="$1" library="$2"
  shift 2
  this_host_environment=()
  crash_amplification_item_break_is inherited-classes-kept || this_host_environment+=(-u GPU_COMPUTATION_CLASSES)
  this_host_environment+=("${common_environment[@]}" "$@" "SINGLEFS_CRASH_AMPLIFICATION_SHARE=$share" "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY=$library")
  [[ -n "${gpu_cards_by_priority:-}" ]] && this_host_environment+=("GPU_CARDS_BY_PRIORITY=$gpu_cards_by_priority")
  if [[ -n "${gpu_computation_classes:-}" ]] && ! crash_amplification_item_break_is classes-not-sent-on-this-host; then
    this_host_environment+=("GPU_COMPUTATION_CLASSES=$gpu_computation_classes")
  fi
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
  crash_amplification_environment_for_this_host "$share" "$library" "$@"
  local local_environment=("${this_host_environment[@]}")
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

crash_amplification_item_break_is() { [[ ",${CRASH_AMPLIFICATION_ITEM_BREAK:-}," == *",$1,"* ]]; }

# 显卡配置与双机开关对不对得上：双机开着两台都要有点名的卡，双机没开不许点名第二台的卡。对得上交回 0，对不上打原因与出路、交回 1
crash_amplification_cards_match_the_hosts() { # <双机开关 0/1> <显卡配置的值>
  local across="$1" cards="$2" names_local=0 names_peer=0 entry
  for entry in $cards; do
    [[ "$entry" == local:* ]] && names_local=1
    [[ "$entry" == peer:* ]] && names_peer=1
  done
  if [[ "$across" == 1 ]] && (( ! names_local || ! names_peer )) && ! crash_amplification_item_break_is accept-missing-peer-cards; then
    echo "  ✗ 配置开了双机与 GPU，显卡配置「$cards」却没有两台各点名至少一张卡" >&2
    echo "  → 怎么办：修好设备，或改配置：照仓根 gpu-cards.env.example 给两台各点名卡（local:<序号>:<额度> 与 peer:<序号>:<额度>）；不想用第二台就把多机配置的 ENABLE_ACROSS_MACHINES 改成 0" >&2
    return 1
  fi
  if [[ "$across" != 1 ]] && (( names_peer )) && ! crash_amplification_item_break_is accept-peer-cards-without-across; then
    echo "  ✗ 多机配置没开双机，显卡配置「$cards」却点名了第二台的卡：点名的卡用不上" >&2
    echo "  → 怎么办：修好设备，或改配置：要用第二台就把多机配置的 ENABLE_ACROSS_MACHINES 改成 1；不用就从显卡配置里删掉 peer: 那几项" >&2
    return 1
  fi
  return 0
}

# 运行条件：配置里开了的硬件在配置上对不对得上。开关读多机配置（CRASH_AMPLIFICATION_ITEM_SWITCHES=<gpu>,<双机> 只供测试，给了照它），
# 显卡读显卡配置的两份（不清场的与清场之后的，这里不知道调用方清没清场，写了的每一份都要对得上；CRASH_AMPLIFICATION_ITEM_CARDS 只供测试，给了只判它）。
crash_amplification_run_condition_holds() { # [<仓根>]：对得上交回 0；对不上打原因与出路、交回 1
  local root="${1:-$PWD}" configuration gpu=0 across=0 cards_file key list lists=() gpu_computation_classes=""
  configuration="${SINGLEFS_MULTI_HOST_CONFIG:-$root/multi-host.env}"
  [[ "$(crash_amplification_configuration_value "$configuration" ENABLE_GPU)" == 1 ]] && gpu=1
  [[ "$(crash_amplification_configuration_value "$configuration" ENABLE_ACROSS_MACHINES)" == 1 ]] && across=1
  if [[ -n "${CRASH_AMPLIFICATION_ITEM_SWITCHES:-}" ]]; then IFS=, read -r gpu across <<<"$CRASH_AMPLIFICATION_ITEM_SWITCHES"; fi
  [[ "$gpu" == 1 ]] || return 0
  cards_file="${SINGLEFS_GPU_CARDS_CONFIG:-${XDG_CONFIG_HOME:-$HOME/.config}/singlefs/gpu-cards.env}"
  if [[ -n "${CRASH_AMPLIFICATION_ITEM_CARDS:-}" ]]; then
    lists=("$CRASH_AMPLIFICATION_ITEM_CARDS")
  else
    for key in GPU_CARDS_BY_PRIORITY GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED; do
      list="$(crash_amplification_configuration_value "$cards_file" "$key")"
      [[ -n "$list" ]] && lists+=("$list")
    done
  fi
  if (( ${#lists[@]} == 0 )) && ! crash_amplification_item_break_is accept-missing-card-list; then
    echo "  ✗ 配置开了 GPU（ENABLE_GPU=1），显卡配置里 GPU_CARDS_BY_PRIORITY 与 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED 一份都没写（显卡配置：$cards_file）" >&2
    echo "  → 怎么办：修好设备，或改配置：照仓根 gpu-cards.env.example 写每台机器用哪几张卡、各多少显存；不用 GPU 就把多机配置的 ENABLE_GPU 改成 0" >&2
    return 1
  fi
  for list in ${lists[@]+"${lists[@]}"}; do
    crash_amplification_cards_match_the_hosts "$across" "$list" || return 1
  done
  # 走 GPU 的计算类：与 crash_amplification_prepare 同一个函数，开跑前就拒（没写或写了认不得的类名；出路同一句），不等跑到 prepare 才退 78
  if ! crash_amplification_item_break_is run-condition-skips-classes; then
    crash_amplification_read_gpu_computation_classes "$cards_file" || return 1
  fi
  return 0
}

# 一趟没跑成，日志里说的是哪一类（判据是用例与判器报错的原话，出处见各串旁边）：
#   device：显卡或显存用不上——GPU_CARD_NOT_USABLE 行（crates/singlefs-checker-tier/src/gpu_unit_checks.rs）、
#     「这台机器却没有一张用得了的卡」「没有一张卡装得下这条流的表」「GPU_CARDS_BY_PRIORITY 里没有」（crates/singlefs-checker-tier/src/crash_amplification.rs）
#   algorithm：算法上判不了——「表越过了内核的容量」（有状态 GPU 判不了）、「全部调用展开之后 <数> 个表达式，超过预算」（内核展开超预算，crash_judge_dispatch.rs）、
#     「GPU 判器建不出这条流的表」（crash_amplification.rs）
# 不在的日志跳过；两类都有按 device（设备用不上时那一趟判的不算数）；都不是打空串。
crash_amplification_failure_kind() { # <日志…>
  local device_pattern='GPU_CARD_NOT_USABLE|这台机器却没有一张用得了的卡|没有一张卡装得下这条流的表|GPU_CARDS_BY_PRIORITY 里没有'
  local algorithm_pattern='表越过了内核的容量|全部调用展开之后 [0-9]+ 个表达式，超过预算|GPU 判器建不出这条流的表'
  local logs=() log
  for log in "$@"; do [[ -f "$log" ]] && logs+=("$log"); done
  (( ${#logs[@]} > 0 )) || return 0
  if ! crash_amplification_item_break_is device-unusable-as-run-failure && grep -qE -- "$device_pattern" "${logs[@]}"; then echo device; return 0; fi
  if ! crash_amplification_item_break_is algorithm-refusal-as-run-failure && grep -qE -- "$algorithm_pattern" "${logs[@]}"; then echo algorithm; return 0; fi
  return 0
}

crash_amplification_exit_code_of_a_failed_run() { # <那一趟的退出码> <日志…>：打调用方该退的码
  local run_exit="$1" kind
  shift
  kind="$(crash_amplification_failure_kind "$@")"
  case "$kind" in
    device) echo 78 ;;
    algorithm) echo 248 ;;
    *) echo "$run_exit" ;;
  esac
}

crash_amplification_explain_exit_code() { # <调用方要退的码>：78 与 248 打对应的出路（标准错误），别的不打
  case "$1" in
    78) echo "  → 怎么办：修好设备，或改配置：日志里 GPU_CARD_NOT_USABLE 行点名的卡用不了（没有这个序号、空闲显存不够、上下文起不来）、显存装不下这条流的表，或第二台连不上；查那张卡或那台机器，或改显卡配置与多机配置（退 78：运行条件不满足）" >&2 ;;
    248) echo "  → 怎么办：算法上判不了（有状态 GPU 判不了、内核展开超预算、建不出表），不是设备的事、也不是红：照日志里 panicked 那几行改判器的表或内核的容量再跑（退 248）" >&2 ;;
  esac
}

# 弄坏开关轮（文件头「弄坏开关轮」一段）
CRASH_AMPLIFICATION_SWITCH_WHEEL_MARK_CONTENT="crash_amplification_switch_wheel 起的子进程"

crash_amplification_read_switch_table() { # <脚本文件> <变量名…>：读文件头的开关表（文件头「弄坏开关轮」一段的写法），一个开关打一行「<变量>=<开关名> <TAB> <点名的格>」
  python3 - "$@" <<'PYTHON'
import re, sys
variables = "|".join(re.escape(name) for name in sys.argv[2:])
start = re.compile(r"^#   (" + variables + r")=([a-z0-9-]+)(\s|$)")
rows, current = [], None
with open(sys.argv[1], encoding="utf-8") as handle:
    for line in handle:
        line = line.rstrip("\n")
        if not line.startswith("#"):
            break
        found = start.match(line)
        if found:
            current = [found.group(1) + "=" + found.group(2), ""]
            rows.append(current)
        elif current is not None and line.startswith("#       "):
            current[1] += " " + line[1:]
        else:
            current = None
for assignment, text in rows:
    previous = None
    while previous != text:
        previous = text
        text = re.sub(r"（[^（）]*）", " ", text)
        text = re.sub(r"\([^()]*\)", " ", text)
    text = re.sub(r"第\s*([0-9]+)\s*格", r" \1 ", text)
    print(assignment + "\t" + " ".join(part for part in re.split(r"[\s、，,；;]+", text) if part))
PYTHON
}

crash_amplification_switch_wheel_inner() { # 这一趟是不是开关轮起的子进程：标记文件在、内容对得上交回 0
  [[ -n "${CRASH_AMPLIFICATION_SWITCH_WHEEL_INNER:-}" && -f "${CRASH_AMPLIFICATION_SWITCH_WHEEL_INNER:-}" ]] \
    && [[ "$(cat -- "$CRASH_AMPLIFICATION_SWITCH_WHEEL_INNER")" == "$CRASH_AMPLIFICATION_SWITCH_WHEEL_MARK_CONTENT" ]]
}

crash_amplification_switch_wheel_verdict() { # <点名的格> <退出码> <输出文件> <取格的正则> <只比退出码 0/1>：打实测红的格，对得上交回 0
  local expected="$1" exit_code="$2" output="$3" pattern="$4" exit_only="$5" observed="" wanted=""
  if [[ -f "$output" ]]; then
    observed="$(sed -nE "s/.*${pattern}.*/\\1/p" -- "$output" | LC_ALL=C sort -V | uniq | tr '\n' ' ')"
  fi
  printf '%s' "${observed% }"
  if [[ "$expected" == - ]]; then
    [[ "$exit_code" == 0 ]] && { [[ "$exit_only" == 1 ]] || [[ -z "$observed" ]]; }
    return
  fi
  wanted="$(tr -s ' \t' '\n\n' <<<"$expected" | sed '/^$/d' | LC_ALL=C sort -V | uniq | tr '\n' ' ')"
  [[ -n "$wanted" && "$exit_code" == 1 ]] || return 1
  [[ "$exit_only" == 1 || "$observed" == "$wanted" ]]
}

crash_amplification_switch_wheel() { # <放输出的目录> <取格的正则> <只比退出码 0/1> -- <自证命令…>（两个数组与交回的变量见文件头）
  local directory="$1" pattern="$2" exit_only="$3" jobs batch index count assignment variable observed shown
  local variables=" " unset_arguments=() pids=() exits=()
  shift 3
  [[ "${1:-}" == -- ]] && shift
  switch_wheel_failures=0; switch_wheel_collected=0
  count="${#switch_wheel_assignments[@]}"
  jobs="${CRASH_AMPLIFICATION_SELFTEST_JOBS:-${SINGLEFS_THREAD_CAP:-$(nproc)}}"
  [[ "$jobs" =~ ^[1-9][0-9]*$ ]] || jobs=1
  for assignment in ${switch_wheel_assignments[@]+"${switch_wheel_assignments[@]}"}; do
    variable="${assignment%%=*}"
    [[ "$variables" == *" $variable "* ]] && continue
    variables+="$variable "; unset_arguments+=(-u "$variable")
  done
  printf '%s\n' "$CRASH_AMPLIFICATION_SWITCH_WHEEL_MARK_CONTENT" >"$directory/switch-wheel-mark"
  for ((batch = 0; batch < count; batch += jobs)); do
    for ((index = batch; index < batch + jobs && index < count; index++)); do
      env "${unset_arguments[@]}" "${switch_wheel_assignments[index]}" CRASH_AMPLIFICATION_SWITCH_WHEEL_INNER="$directory/switch-wheel-mark" "$@" \
        >"$directory/switch-wheel-$index.out" 2>&1 &
      pids[index]=$!
    done
    for ((index = batch; index < batch + jobs && index < count; index++)); do
      wait "${pids[index]}"; exits[index]=$?
    done
  done
  for ((index = 0; index < count; index++)); do
    [[ -n "${exits[index]:-}" ]] && switch_wheel_collected=$((switch_wheel_collected + 1))
    if observed="$(crash_amplification_switch_wheel_verdict "${switch_wheel_expected[index]:-}" "${exits[index]:-}" "$directory/switch-wheel-$index.out" "$pattern" "$exit_only")"; then
      shown="${observed// /、}"
      echo "  · 开关轮：${switch_wheel_assignments[index]} 红在「${shown:-（一格不红，表里没列它）}」，与文件头相同"
    else
      shown="${switch_wheel_expected[index]:-}"
      [[ "$shown" == - ]] && shown="（表里没列它：应当退 0、一格不红）"
      echo "  ✗ 开关轮：${switch_wheel_assignments[index]} 打开后 --selftest 退 ${exits[index]:-（没收到）}、红在「${observed:-（没有）}」，文件头点名的是「${shown:-（没写）}」" >&2 # gate-lint:detail
      switch_wheel_failures=$((switch_wheel_failures + 1))
    fi
  done
  if (( switch_wheel_collected != count )); then
    echo "  ✗ 开关轮：派出 $count 个开关、收回 $switch_wheel_collected 个" >&2 # gate-lint:detail
    switch_wheel_failures=$((switch_wheel_failures + 1))
  fi
  return 0
}
