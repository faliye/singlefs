#!/usr/bin/env bash
# 崩溃放量的两个驱动共用的那几段：读配置的开关与显卡配置、组 cargo 的参数与环境、在本机跑一趟用例、从日志里取派活计划的数。
# 被 source 的函数库，只定义函数、不改 shell 选项，不单独调；research/scripts/crash-amplification-item.sh（一个分项的一趟）与
# research/scripts/crash-amplification-two-hosts.sh（两台各领一份）source 它。
# gate-similar: peer-host-lib.sh 它管第二台那几段（读多机配置、在第二台上跑命令、清场，不复原）；这里管本机那一趟与派活计划的数，不连第二台
#
# 调用方先设好这几个变量再调 crash_amplification_prepare：
#   repository_root  仓根
#   test_target      crates/singlefs-checker-tier/tests/ 下的测试目标；test_filter 用例函数名（可空）
#   expand_up_to、threads  可空；memory_cap 本机那一趟的内存上限
#   library_is_temporary   1 表示库只活这一趟（自证用）
#   quiesced               1 表示清过场（卡上别的服务停了）：显卡的先后与额度取清场之后那一份
# crash_amplification_prepare 之后可用：
#   configuration_file（多机配置的路径，文件头「配置」一段怎么找）、gpu_switch（0 / 1）、across_machines_switch（0 / 1）、features、
#   gpu_cards_by_priority（这一趟用的那一份，没写是空串）、gpu_cards_by_priority_when_quiesced（清场之后那一份，没写是空串）、
#   common_environment、cargo_build_arguments、cargo_test_arguments（数组）
#   crash_amplification_prepare 交回非 0（都是 2）只有一种，是配置的事，调用方退 78：说是清过场、开了 GPU 或写了不清场那一份显卡先后，多机配置里却没写清场之后那一份。
# 交给本机那一趟的环境由 crash_amplification_environment_for_this_host 组（数组 this_host_environment，env 的参数）；
#   驱动自证里换掉本机那一趟时也经它组，只把起 cargo 换成假的。
#
# 运行条件与失败分类（崩溃放量入口与分项驱动共用）：
#   crash_amplification_run_condition_holds [<仓根>]   两个驱动文件头 run-condition: check 调它（preflight 用 bash -c 在仓根起）；只判配置：
#       开了 GPU 就要在多机配置里写了显卡、点名的卡与双机开关对得上。不连第二台、不碰卡：那两样跑起来才知道，用不上时驱动在运行中退 78。
#   crash_amplification_cards_match_the_hosts <双机 0/1> <显卡配置的值>   双机开着两台都要有点名的卡，双机没开不许点名第二台的卡
#   crash_amplification_failure_kind <日志…>           一趟没跑成，日志里说的是哪一类：device（显卡或显存用不上，含给定状态表的判器借不到显存那一行，运行条件，调用方退 78）、
#       build（这份二进制跑不了这一趟：结论的版本与树对不上、配置开了 GPU 而没编进 gpu 特性，运行条件，调用方退 78）、
#       algorithm（算法上判不了：有状态 GPU 判不了、内核展开超预算、建不出表，调用方退 248），都不是打空串；几类都有时先 device、再 build
#   crash_amplification_exit_code_of_a_failed_run <那一趟的退出码> <日志…>   打调用方该退的码：device 与 build 78、algorithm 248，别的原样
#     （内存包装退的 245 显存不够照原样，出路由 crash_amplification_explain_exit_code 的 245 那一支打）
#   弄坏开关读 CRASH_AMPLIFICATION_ITEM_BREAK（开关表与该红的格在 research/scripts/crash-amplification-item.sh 文件头）：
#   accept-missing-peer-cards、accept-peer-cards-without-across、accept-missing-card-list、device-unusable-as-run-failure、algorithm-refusal-as-run-failure、
#   version-differs-as-run-failure、gpu-feature-missing-as-run-failure、library-in-the-old-directory。
#   given-states-run-condition-as-run-failure（给定状态表借不到显存那一行不认、不退 78）只由 research/scripts/crash-amplification-two-hosts.sh --selftest 证红
#   （红在哪一格写在那份文件头「共用库 crash-amplification-lib.sh 的开关」一段）。
#   其中 device-unusable-as-run-failure、algorithm-refusal-as-run-failure 打开时 research/scripts/crash-amplification-two-hosts.sh --selftest 也红
#   （红在哪几格写在那份文件头「共用库 crash-amplification-lib.sh 的开关」一段）。
#   走 GPU 的计算类那几处的开关同样读 CRASH_AMPLIFICATION_ITEM_BREAK，只由 research/scripts/crash-amplification-two-hosts.sh --selftest 证红
#   （开关表与该红的格在那份文件头「共用库 crash-amplification-lib.sh 的开关」一段）：
#   inherited-classes-kept、classes-sent-again、configuration-from-tree-root。
#   配置那几处的开关 cards-read-from-cards-file 由 research/scripts/crash-amplification-item.sh --selftest 证红（表与该红的格在那份文件头）。
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
# 现算线程（crash_amplification_threads_now <local|peer> [<显式给的线程数>]）：本机只编那一步（crash_amplification_run_on_this_host）与
#   双机驱动送第二台的那两趟（排计划、分份）起 capped.sh 的线程数不写死，开起那一步之前现算：调用方显式给了就用它（要是正整数；
#   本机只编那一步读环境变量 CRASH_AMPLIFICATION_BUILD_THREADS，第二台那两趟读 CRASH_AMPLIFICATION_PEER_THREADS），没给就读
#   research/scripts/host-resources.py --json 里那一台的 cpu.available_threads（local 取 is_local 那一台、peer 取 is_peer 那一台；
#   那一份「可用线程」= 核数 − 此刻在用的核），不到 1 取 1。只供测试：CRASH_AMPLIFICATION_HOST_RESOURCES=<脚本> 换掉 host-resources.py
#   （python3 <脚本> --json 起，平常跑时设了也照认）。算不出（显式给的不是正整数、host-resources.py 退非 0、输出里读不出那一台）打 ✗ 与出路、交回 1，
#   不退回写死的数：本机只编那一步交回 78、不起 cargo；双机驱动退 78、第二台那一趟不起。
#   弄坏开关 CRASH_AMPLIFICATION_ITEM_BREAK=build-threads-fixed：本机只编那一步照改前写死 6 路，由 research/scripts/crash-amplification-two-hosts.sh --selftest 证红
#   （红在哪几格写在那份文件头「共用库 crash-amplification-lib.sh 的开关」一段）。
#
# 配置（用户 2026-10-10 定：多机配置合成一份，不零散配置；判决 research/prompts/m3-crash-enumeration-merge-r3-main-verification.md 第三节 J5、J11）：
#   只读一份多机配置 ${SINGLEFS_MULTI_HOST_CONFIG:-<主工作树的根>/multi-host.env}（git 忽略、这台机器一份；模板是仓根的 multi-host.env.example）。
#   主工作树的根按 git common-dir 取（crash_amplification_multi_host_configuration_file，找法与 research/scripts/layer0-shard-configuration-check.sh、
#   research/scripts/gpu-quiesce-run.sh 相同）：在 HEAD + 暂存区的临时 worktree 里跑也读主工作树那一份；树不是 git 工作树时退回 <树根>/multi-host.env。
#   开关 ENABLE_GPU、ENABLE_ACROSS_MACHINES 的判法不改（没写或写 0 算关，键的定义只在 layer0-shard-configuration-check.sh）。
#   显卡的两个键也写在这一份里：GPU_CARDS_BY_PRIORITY（不清场时显卡的先后与每张卡的显存额度，读法在
#   crates/singlefs-checker-tier/src/gpu_card_configuration.rs 的 parse_cards_by_priority）、GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED（清场之后的先后与额度，同一种写法；
#   全量开跑就清场，用的是这一份）。挑中的那一份显卡先后经环境变量 GPU_CARDS_BY_PRIORITY 交给用例（本机与第二台同一份，各取点名给自己的卡）。
#   私有显卡配置（SINGLEFS_GPU_CARDS_CONFIG 指的那一份、~/.config/singlefs/gpu-cards.env）退役：这里不读它，本机那一趟起 cargo 前 env -u 清掉这个变量，
#   用例也就读不到它；第二台那几趟本来就只看命令里送过去的环境。
# 走 GPU 的计算类：不再有正表，也不再经环境变量交。开了 GPU（ENABLE_GPU=1）引擎就三类全走 GPU，它不读 GPU_COMPUTATION_CLASSES
#   （用例经 CrashAmplificationEnvironment::cleared_then_set 设一个引擎一趟不读的变量会 panic），本机与第二台都不送它；
#   本机那一趟起 cargo 前照样用 env -u 清掉调用方环境里漏进来的同名变量。弄坏开关 classes-sent-again：开了 GPU 照改前往本机那一趟送三类。
#   不走 GPU 的那几截计算登记在仓内豁免表 .claude/gate.d/gpu-exemptions.tsv（每行用户点头），门禁 code-source-discipline 的 gpu-exemptions 格核它与代码标记逐项对上。
#   本机那一趟另交 SINGLEFS_MULTI_HOST_CONFIG=<这一份的绝对路径>：用例不设它时按自己的树根找 multi-host.env，在 worktree 里会找不到主工作树那一份（第二台不交，它的树副本不带配置）。
# 判定库（分项驱动与双机驱动共用；崩溃放量入口与崩溃点重放入口经它们用）：没给 --library 时用 <家目录>/<CRASH_AMPLIFICATION_LIBRARY_SUBDIRECTORY>，
#   家目录是 ${SINGLEFS_CRASH_AMPLIFICATION_HOME:-~/.local/share/singlefs/crash-amplification}，子目录今天是 library-by-conclusion-kind；
#   crash_amplification_default_library <家目录> 打这个路径。按结论种类存之前的旧库（<家目录>/library）引擎开库就拒，所以换成这个新子目录；
#   旧目录不删、不读。弄坏开关 library-in-the-old-directory：照改前打 <家目录>/library，由 research/scripts/crash-amplification-item.sh --selftest 证红（表与该红的格在那份文件头）。

CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS="-I/usr/lib/gcc/x86_64-linux-gnu/13/include -I/usr/lib/gcc/x86_64-linux-gnu/14/include"  # librocksdb-sys 的 bindgen 找 stdbool.h；哪个版本的目录在就用哪个，不在的 -I 无害
CRASH_AMPLIFICATION_LIBRARY_SUBDIRECTORY="library-by-conclusion-kind"  # 默认判定库在家目录下的子目录（文件头「判定库」）：按结论种类存的库；旧的 library 引擎开库就拒
# 给定状态表的判器借不到显存时引擎打的那一行的行首（crates/singlefs-checker-tier/src/crash_amplification.rs 的 GIVEN_STATES_RUN_CONDITION_NOT_MET_LINE_PREFIX）：
# 运行条件不满足，调用方退 78；crash_amplification_failure_kind 的 device 那一组与崩溃点重放入口 research/scripts/crash-point-replay.sh 的快档都照这一份认
CRASH_AMPLIFICATION_GIVEN_STATES_RUN_CONDITION_LINE_PREFIX="CRASH_POINT_REPLAY_GIVEN_STATES_RUN_CONDITION_NOT_MET"

crash_amplification_default_library() { # <家目录>：打没给 --library 时的判定库路径（文件头「判定库」）
  if crash_amplification_item_break_is library-in-the-old-directory; then printf '%s/library\n' "$1"; return 0; fi
  printf '%s/%s\n' "$1" "$CRASH_AMPLIFICATION_LIBRARY_SUBDIRECTORY"
}

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

# 多机配置在哪（文件头「配置」）：打路径。SINGLEFS_MULTI_HOST_CONFIG 设了照它；没设取 <树根> 的 git common-dir 的上一层（主工作树的根）下的 multi-host.env，
# 取不到 common-dir（不是 git 工作树）退回 <树根>/multi-host.env。文件在不在不判（不在就是两个开关都关、一张卡都没写）。
# 弄坏开关 configuration-from-tree-root：照改前按 <树根>/multi-host.env 找（在 worktree 里读不到主工作树那一份）。
crash_amplification_multi_host_configuration_file() { # <树根>
  local root="$1" common_directory=""
  if [[ -n "${SINGLEFS_MULTI_HOST_CONFIG:-}" ]]; then printf '%s\n' "$SINGLEFS_MULTI_HOST_CONFIG"; return 0; fi
  if ! crash_amplification_item_break_is configuration-from-tree-root \
     && common_directory="$(git -C "$root" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" && [[ -n "$common_directory" ]]; then
    printf '%s/multi-host.env\n' "$(dirname "$common_directory")"
    return 0
  fi
  printf '%s/multi-host.env\n' "$root"
}

# 显卡的两个键从哪一份读（文件头「配置」：多机配置那一份）；弄坏开关 cards-read-from-cards-file 下照改前读退役的私有显卡配置
crash_amplification_cards_source_file() { # <多机配置的路径>
  if crash_amplification_item_break_is cards-read-from-cards-file; then
    printf '%s\n' "${SINGLEFS_GPU_CARDS_CONFIG:-${XDG_CONFIG_HOME:-$HOME/.config}/singlefs/gpu-cards.env}"
    return 0
  fi
  printf '%s\n' "$1"
}

crash_amplification_prepare() {
  configuration_file="$(crash_amplification_multi_host_configuration_file "$repository_root")"
  gpu_switch=0; across_machines_switch=0; features="verdict-store"
  [[ "$(crash_amplification_configuration_value "$configuration_file" ENABLE_GPU)" == 1 ]] && gpu_switch=1
  [[ "$(crash_amplification_configuration_value "$configuration_file" ENABLE_ACROSS_MACHINES)" == 1 ]] && across_machines_switch=1
  if [[ "${library_is_temporary:-0}" == 1 ]]; then gpu_switch=0; fi  # 自证不上卡
  (( gpu_switch )) && features="verdict-store,gpu"
  local cards_file
  cards_file="$(crash_amplification_cards_source_file "$configuration_file")"
  gpu_cards_by_priority="$(crash_amplification_configuration_value "$cards_file" GPU_CARDS_BY_PRIORITY)"
  gpu_cards_by_priority_when_quiesced="$(crash_amplification_configuration_value "$cards_file" GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED)"
  # 清过场：用清场之后那一份；开了 GPU、或写了不清场那一份显卡先后（要用卡）却没写清场之后那一份，交回 2。没开 GPU 也没写卡的（只清场跑 CPU）照跑
  if [[ "${quiesced:-0}" == 1 ]] && { (( gpu_switch )) || [[ -n "$gpu_cards_by_priority" ]]; }; then
    if [[ -z "$gpu_cards_by_priority_when_quiesced" ]]; then
      echo "  ✗ 说是清过场，多机配置里却没写 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED（多机配置：$configuration_file）" >&2
      echo "  → 怎么办：修好设备，或改配置：照仓根 multi-host.env.example 在多机配置里写清场之后的显卡先后与额度" >&2
      return 2
    fi
    gpu_cards_by_priority="$gpu_cards_by_priority_when_quiesced"
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

# 交给本机那一趟的环境（env 的参数）放进数组 this_host_environment：先 -u 清掉调用方环境里漏进来的 GPU_COMPUTATION_CLASSES（引擎不读它，文件头「配置」）
# 与退役的 SINGLEFS_GPU_CARDS_CONFIG，再是 common_environment、附加环境、份与库、多机配置的路径（文件在才交，文件头「配置」），挑中的那份显卡先后与额度（空的不交）。
crash_amplification_environment_for_this_host() { # <share> <库目录> [<附加环境 键=值>…]
  local share="$1" library="$2"
  shift 2
  this_host_environment=(-u SINGLEFS_GPU_CARDS_CONFIG)
  crash_amplification_item_break_is inherited-classes-kept || this_host_environment+=(-u GPU_COMPUTATION_CLASSES)
  this_host_environment+=("${common_environment[@]}" "$@" "SINGLEFS_CRASH_AMPLIFICATION_SHARE=$share" "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY=$library")
  [[ -n "${configuration_file:-}" && -f "${configuration_file:-}" ]] && this_host_environment+=("SINGLEFS_MULTI_HOST_CONFIG=$configuration_file")
  [[ -n "${gpu_cards_by_priority:-}" ]] && this_host_environment+=("GPU_CARDS_BY_PRIORITY=$gpu_cards_by_priority")
  if (( ${gpu_switch:-0} )) && crash_amplification_item_break_is classes-sent-again; then
    this_host_environment+=("GPU_COMPUTATION_CLASSES=gpu_unit_checks crash_verify_gpu crash_judge_gpu")   # 弄坏开关（文件头「配置」）：改前送的那三类
  fi
  return 0
}

# 这一刻那一台能开多少线程（文件头「现算线程」）：打一个正整数；算不出打 ✗ 与出路、交回 1
crash_amplification_threads_now() { # <local|peer> [<显式给的线程数>]
  local host="$1" given="${2:-}" name=本机 report=""
  local resources="${CRASH_AMPLIFICATION_HOST_RESOURCES:-$repository_root/research/scripts/host-resources.py}"
  [[ "$host" == peer ]] && name=第二台
  if [[ -n "$given" ]]; then
    if [[ "$given" =~ ^[1-9][0-9]*$ ]]; then echo "$given"; return 0; fi
    echo "  ✗ 显式给的${name}线程数「$given」不是正整数" >&2
    echo "  → 怎么办：写成正整数，或不给、按 research/scripts/host-resources.py --json 现算（crash-amplification-lib.sh 文件头「现算线程」）" >&2
    return 1
  fi
  if ! report="$(python3 "$resources" --json 2>/dev/null)"; then
    echo "  ✗ 现算${name}线程数：python3 $resources --json 退了非 0" >&2
    echo "  → 怎么办：单跑 python3 research/scripts/host-resources.py 看哪一台、哪一项没读到（第二台连不上就修好它或改多机配置）；要绕开现算就显式给线程数（crash-amplification-lib.sh 文件头「现算线程」）" >&2
    return 1
  fi
  python3 -c '
import json, sys
field = "is_local" if sys.argv[1] == "local" else "is_peer"
try:
    hosts = json.loads(sys.stdin.read()).get("hosts") or []
except (ValueError, AttributeError):
    sys.exit(1)
for entry in hosts:
    if isinstance(entry, dict) and entry.get(field) and entry.get("read"):
        threads = (entry.get("cpu") or {}).get("available_threads")
        if isinstance(threads, (int, float)) and not isinstance(threads, bool):
            print(max(1, int(threads)))
            sys.exit(0)
sys.exit(1)' "$host" <<<"$report" && return 0
  echo "  ✗ 现算${name}线程数：$resources --json 的输出里读不出${name}此刻的可用线程（hosts[] 里 $([[ "$host" == peer ]] && echo is_peer || echo is_local) 那一台的 cpu.available_threads；那一台没读到也是这样）" >&2
  echo "  → 怎么办：单跑 python3 research/scripts/host-resources.py --json 看那一台的 read 与 problem；要绕开现算就显式给线程数（crash-amplification-lib.sh 文件头「现算线程」）" >&2
  return 1
}

# 本机的一趟：先只编（并行度按文件头「现算线程」现算：librocksdb-sys 的 C++ 按并行度吃内存，由内存包装的上限兜着），再跑；
# 跑的那一步用 threads 个核，没给就是本机核数减一（用户 2026-09-29 定：测试与真跑都用 31 核，留一个核给机器上别的事）。
# 只编那一步的线程数算不出：打 ✗ 与出路（标准错误，同一句写进日志），交回 78、不起 cargo。
# 要这一趟中途停下：附加环境里给 SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE=<文件>，建出那个文件它就不再领新的块、手里那一块判完写进库、
# 打一行汇总退 0；判过的块在库里，下一趟接着判。
crash_amplification_run_on_this_host() { # <share> <库目录> <日志> [<附加环境 键=值>…]
  local share="$1" library="$2" log="$3"
  shift 3
  local cores="${threads:-}"
  [[ -n "$cores" ]] || cores=$(( $(nproc) > 1 ? $(nproc) - 1 : 1 ))
  local build_threads=""
  if crash_amplification_item_break_is build-threads-fixed; then
    build_threads=6
  elif ! build_threads="$(crash_amplification_threads_now local "${CRASH_AMPLIFICATION_BUILD_THREADS:-}" 2>"$log")"; then
    cat -- "$log" >&2
    return 78
  fi
  crash_amplification_environment_for_this_host "$share" "$library" "$@"
  local local_environment=("${this_host_environment[@]}")
  ( cd "$repository_root" \
    && env "${local_environment[@]}" bash research/scripts/run-with-memory-cap.sh "$memory_cap" bash research/scripts/capped.sh "$build_threads" cargo "${cargo_build_arguments[@]}" \
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
    echo "  ✗ 配置开了双机与 GPU，显卡先后「$cards」却没有两台各点名至少一张卡" >&2
    echo "  → 怎么办：修好设备，或改配置：照仓根 multi-host.env.example 在多机配置里给两台各点名卡（local:<序号>:<额度> 与 peer:<序号>:<额度>）；不想用第二台就把多机配置的 ENABLE_ACROSS_MACHINES 改成 0" >&2
    return 1
  fi
  if [[ "$across" != 1 ]] && (( names_peer )) && ! crash_amplification_item_break_is accept-peer-cards-without-across; then
    echo "  ✗ 多机配置没开双机，显卡先后「$cards」却点名了第二台的卡：点名的卡用不上" >&2
    echo "  → 怎么办：修好设备，或改配置：要用第二台就把多机配置的 ENABLE_ACROSS_MACHINES 改成 1；不用就从多机配置的显卡先后里删掉 peer: 那几项" >&2
    return 1
  fi
  return 0
}

# 运行条件：配置里开了的硬件在配置上对不对得上。开关与显卡都读多机配置（文件头「配置」；CRASH_AMPLIFICATION_ITEM_SWITCHES=<gpu>,<双机> 只供测试，给了照它），
# 显卡读两份先后（不清场的与清场之后的，这里不知道调用方清没清场，写了的每一份都要对得上；CRASH_AMPLIFICATION_ITEM_CARDS 只供测试，给了只判它）。
crash_amplification_run_condition_holds() { # [<仓根>]：对得上交回 0；对不上打原因与出路、交回 1
  local root="${1:-$PWD}" configuration gpu=0 across=0 cards_file key list lists=()
  configuration="$(crash_amplification_multi_host_configuration_file "$root")"
  [[ "$(crash_amplification_configuration_value "$configuration" ENABLE_GPU)" == 1 ]] && gpu=1
  [[ "$(crash_amplification_configuration_value "$configuration" ENABLE_ACROSS_MACHINES)" == 1 ]] && across=1
  if [[ -n "${CRASH_AMPLIFICATION_ITEM_SWITCHES:-}" ]]; then IFS=, read -r gpu across <<<"$CRASH_AMPLIFICATION_ITEM_SWITCHES"; fi
  [[ "$gpu" == 1 ]] || return 0
  cards_file="$(crash_amplification_cards_source_file "$configuration")"
  if [[ -n "${CRASH_AMPLIFICATION_ITEM_CARDS:-}" ]]; then
    lists=("$CRASH_AMPLIFICATION_ITEM_CARDS")
  else
    for key in GPU_CARDS_BY_PRIORITY GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED; do
      list="$(crash_amplification_configuration_value "$cards_file" "$key")"
      [[ -n "$list" ]] && lists+=("$list")
    done
  fi
  if (( ${#lists[@]} == 0 )) && ! crash_amplification_item_break_is accept-missing-card-list; then
    echo "  ✗ 配置开了 GPU（ENABLE_GPU=1），多机配置里 GPU_CARDS_BY_PRIORITY 与 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED 一份都没写（多机配置：$cards_file）" >&2
    echo "  → 怎么办：修好设备，或改配置：照仓根 multi-host.env.example 在多机配置里写每台机器用哪几张卡、各多少显存；不用 GPU 就把多机配置的 ENABLE_GPU 改成 0" >&2
    return 1
  fi
  for list in ${lists[@]+"${lists[@]}"}; do
    crash_amplification_cards_match_the_hosts "$across" "$list" || return 1
  done
  return 0
}

# 一趟没跑成，日志里说的是哪一类（判据是用例与判器报错的原话，出处见各串旁边）：
#   device：显卡或显存用不上——GPU_CARD_NOT_USABLE 行（crates/singlefs-checker-tier/src/gpu_unit_checks.rs）、
#     「这台机器却没有一张用得了的卡」「没有一张卡装得下这条流的表」「GPU_CARDS_BY_PRIORITY 里没有」（crates/singlefs-checker-tier/src/crash_amplification.rs）、
#     行首（可带空白）是 CRASH_POINT_REPLAY_GIVEN_STATES_RUN_CONDITION_NOT_MET 的那一行（给定状态表的判器起过卡池而借不到显存，引擎打这一行后报错；
#     CRASH_AMPLIFICATION_GIVEN_STATES_RUN_CONDITION_LINE_PREFIX）
#   build：这份二进制跑不了这一趟——行首是 CRASH_POINT_REPLAY_NOT_WRITTEN_CONCLUSION_VERSION_DIFFERS_FROM_THE_BUILD 的那一行（结论的版本编译时与跑时对不上、
#     整趟不写库；crates/singlefs-checker-tier/src/crash_amplification.rs 的 CONCLUSION_VERSION_NOT_WRITTEN_LINE_PREFIX）、
#     「这份二进制没有编进 gpu 特性」（多机配置开了 GPU 而编的不带 gpu 特性，crash_amplification.rs）
#   algorithm：算法上判不了——「表越过了内核的容量」（有状态 GPU 判不了）、「全部调用展开之后 <数> 个表达式，超过预算」（内核展开超预算，crash_judge_dispatch.rs）、
#     「GPU 判器建不出这条流的表」（crash_amplification.rs）
# 不在的日志跳过；几类都有时先 device、再 build（这两类时那一趟判的不算数）；都不是打空串。
# 弄坏开关 version-differs-as-run-failure、gpu-feature-missing-as-run-failure：build 那两句各自不认（表与该红的格在 crash-amplification-item.sh 文件头）。
# 弄坏开关 given-states-run-condition-as-run-failure：给定状态表借不到显存那一行不认（红在 crash-amplification-two-hosts.sh --selftest 的 local-share-given-states-run-condition）。
crash_amplification_failure_kind() { # <日志…>
  local device_pattern='GPU_CARD_NOT_USABLE|这台机器却没有一张用得了的卡|没有一张卡装得下这条流的表|GPU_CARDS_BY_PRIORITY 里没有'
  local given_states_run_condition_pattern="^[[:space:]]*${CRASH_AMPLIFICATION_GIVEN_STATES_RUN_CONDITION_LINE_PREFIX}( |\$)"
  local version_differs_pattern='^CRASH_POINT_REPLAY_NOT_WRITTEN_CONCLUSION_VERSION_DIFFERS_FROM_THE_BUILD( |$)'
  local gpu_feature_missing_pattern='这份二进制没有编进 gpu 特性'
  local algorithm_pattern='表越过了内核的容量|全部调用展开之后 [0-9]+ 个表达式，超过预算|GPU 判器建不出这条流的表'
  local logs=() log
  for log in "$@"; do [[ -f "$log" ]] && logs+=("$log"); done
  (( ${#logs[@]} > 0 )) || return 0
  if ! crash_amplification_item_break_is device-unusable-as-run-failure && grep -qE -- "$device_pattern" "${logs[@]}"; then echo device; return 0; fi
  if ! crash_amplification_item_break_is given-states-run-condition-as-run-failure && grep -qE -- "$given_states_run_condition_pattern" "${logs[@]}"; then echo device; return 0; fi
  if ! crash_amplification_item_break_is version-differs-as-run-failure && grep -qE -- "$version_differs_pattern" "${logs[@]}"; then echo build; return 0; fi
  if ! crash_amplification_item_break_is gpu-feature-missing-as-run-failure && grep -qF -- "$gpu_feature_missing_pattern" "${logs[@]}"; then echo build; return 0; fi
  if ! crash_amplification_item_break_is algorithm-refusal-as-run-failure && grep -qE -- "$algorithm_pattern" "${logs[@]}"; then echo algorithm; return 0; fi
  return 0
}

crash_amplification_exit_code_of_a_failed_run() { # <那一趟的退出码> <日志…>：打调用方该退的码
  local run_exit="$1" kind
  shift
  kind="$(crash_amplification_failure_kind "$@")"
  case "$kind" in
    device|build) echo 78 ;;
    algorithm) echo 248 ;;
    *) echo "$run_exit" ;;
  esac
}

crash_amplification_explain_exit_code() { # <调用方要退的码>：78、248 与 245（内存包装认出的显存不够）打对应的出路（标准错误），别的不打
  case "$1" in
    245) echo "  → 怎么办：显存不够（那一趟退了非 0，内存包装认出它起止之间内核报了 NV_ERR_NO_MEMORY，退 245），这一趟的结果不算数：nvidia-smi 看谁在用卡（同一段时间别的进程撞的也算进来），等它跑完、或少给这一趟显存，再重跑同一条命令接着判（判过的块已经在库里）" >&2 ;;
    78) echo "  → 怎么办：修好设备，或改配置：日志里 GPU_CARD_NOT_USABLE 行点名的卡用不了（没有这个序号、空闲显存不够、上下文起不来）、显存装不下这条流的表、给定状态表的判器借不到显存（CRASH_POINT_REPLAY_GIVEN_STATES_RUN_CONDITION_NOT_MET 那一行，等卡上占着显存的跑完再跑），或第二台连不上；查那张卡或那台机器，或改显卡配置与多机配置；日志里有 CRASH_POINT_REPLAY_NOT_WRITTEN_CONCLUSION_VERSION_DIFFERS_FROM_THE_BUILD 那一行的，是二进制与盘上的树对不上、整趟没写库，重编再跑（或在提交那一趟的 worktree 里跑）；有「这份二进制没有编进 gpu 特性」的，带 --features verdict-store,gpu 重编，或把多机配置的 ENABLE_GPU 改成 0（退 78：运行条件不满足）" >&2 ;;
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
