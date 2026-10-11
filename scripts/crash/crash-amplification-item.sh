#!/usr/bin/env bash
# admission: always 每次调都在这一刻的树与库上排派活计划、再跑；哪些块要重判由库里的块键与判法版本定，不在这里
# run-condition: command bash cargo python3
# run-condition: check source "$PREFLIGHT_SCRIPT_DIRECTORY/crash-amplification-lib.sh" && crash_amplification_run_condition_holds "$PREFLIGHT_SCRIPT_DIRECTORY/../.." :: 修好设备，或改配置：多机配置 multi-host.env 开了 GPU 而一份卡都没写、点名的卡与双机开关对不上；单跑 bash -c 'source scripts/crash-amplification-lib.sh && crash_amplification_run_condition_holds' 看是哪一处
# 崩溃放量一个分项的一趟：配置定几台、用不用 GPU，这里照配置跑，做不到就报错退出，不退回少一台、不退回 CPU
# （用户 2026-09-29 定：「配置中一台机器就要跑一台，配置中两台机器就要跑两台，配置中有gpu就一定要让GPU工作 如果不行就明确报错拒绝，不接受默认降级」）。
#
#   crash-amplification-item.sh <测试目标> [--test-filter <用例函数名>] [--library <库目录>] [--expand-up-to <N>] [--threads <N>]
#                               [--memory-cap <本机上限>] [--peer-memory-cap <第二台上限>] [--label <标签>] [--quiesced]
#   crash-amplification-item.sh --selftest      拿造出来的派活计划行与日志走一遍「在哪跑」的判法、运行条件与失败分类、第二台失败时的去向、排计划那一趟借不到显存的本次未跑（不编、不跑、不上卡、不连第二台）
#
# 测试目标是 crates/singlefs-checker-tier/tests/ 下一条走 crash_amplification::run_from_environment 的用例。
# 运行条件分两半：配置上判得了的（多机配置开了 GPU 却一份卡都没写、点名的卡与双机开关对不上；配置怎么找见 crash-amplification-lib.sh 文件头「配置」）由文件头的 run-condition: check 在开跑之前判，
#   带 --force 越过时在运行中照判、退 78；跑起来才知道的（点名的卡用不了、显存装不下、第二台连不上）在运行中退 78。出路都是「修好设备，或改配置」。
#   算法上判不了的（有状态 GPU 判不了、内核展开超预算、建不出表）不是运行条件，退 248。日志怎么分这几类见 crash-amplification-lib.sh 的 crash_amplification_failure_kind。
# 步骤：
#   ① 多机配置（${SINGLEFS_MULTI_HOST_CONFIG:-主工作树的根 multi-host.env}，按 git common-dir 找）没开 GPU：开了双机交 crash-amplification-two-hosts.sh 两台跑，没开本机 CPU 判。
#   ② 开了 GPU：先清场（见「清场」），再核清场之后那一份显卡配置（GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED）写了没有、与双机开关对不对得上——
#      双机开着，两台都要有点名的卡；双机没开，不许点名第二台的卡。没写或对不上退 78。
#      再在本机跑一趟只排派活计划（SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1，不判），读它打的 CRASH_AMPLIFICATION_PLACEMENT 行
#      （一条用例里几条流各一行，逐个字段加起来）：要判的状态数 pending_states、本机的卡试判量到的速度 states_per_second_on_this_host。
#      一行都没有就不跑，打出日志里 panicked 那几行，照日志分类退：设备用不上 78、算法上判不了 248、那一趟退了非 0 而认不出原因退它的退出码、退 0 却没打计划行 247。
#      只有一种例外是本次未跑：那一趟退 0、一行计划都没有，日志里有产品路径借不到显存的那一行
#      （NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment reason=<为什么>：起过卡池的进程里池里的卡等满了也登记不上、借不到）。
#      这时不跑、不交双机驱动，打 CRASH_AMPLIFICATION_DECISION where=not-run 与一行 CRASH_AMPLIFICATION_NOT_RUN，退 77（见「退出码」77）。
#      别的用例名的 NOT_RUN_NO_GPU_ADAPTER 行、那一趟退了非 0 的，都不算这一种，照上面分类。
#      有计划行、又有产品路径借不到显存的那一行（一条用例几条流，有的流借到了、有的没借到）：借到的照计划判；没借到的流逐条打一行
#      CRASH_AMPLIFICATION_NOT_RUN step=placement … flow=<第几条流>/<几条流> reason=…（第几条按排计划那一趟日志里计划行与那一行的先后数），
#      标准错误逐条说这不是通过；退出码照跑的那一趟，入口照这几行把这一项记 not-run、不算全量通过。
#   ③ 双机没开：本机一台跑（SHARE 0/1）。双机开着：交 crash-amplification-two-hosts.sh，带 --local-states-per-second <本机量到的速度>
#      （它让第二台也量一次，两台按速度分）。第二台没跑成：原样退双机驱动的退出码（78、248、249 见下），不改在本机一台跑。
#      第二台排计划那一趟借不到显存：双机驱动打 CRASH_AMPLIFICATION_NOT_RUN step=peer-placement 那一行、退 77，这里报本次未跑、原样退 77。
#   本机一台跑的那一趟退非 0，照 run.log 分类：设备用不上 78、算法上判不了 248，别的原样退它的退出码。
# 清场：开了 GPU 就清场，显卡的先后与额度一律用清场之后那一份。没带 --quiesced 时经 scripts/gpu-quiesce-run.sh 起自己（加 --quiesced）：
#   照多机配置清场并回读、跑，跑完、跑红或被打断都不复原（包装打 GPU_QUIESCE step=restore result=not-restored；要复原由用户手动跑私有复原脚本，
#   多机配置的 QUIESCE_START_COMMAND）；清场回读没过退 78（卡上有不归清场白名单的计算进程时就是这样：等那个进程跑完再起，这里不停它）。
#   带了 --quiesced 是调用方清过场（崩溃放量入口经同一份包装这样调），这里不再清。没开 GPU 不清场。
# 库与日志：库默认 ${SINGLEFS_CRASH_AMPLIFICATION_HOME:-~/.local/share/singlefs/crash-amplification}/library-by-conclusion-kind（开机不清空，重跑同一条命令就是续跑；
#   子目录名是 crash-amplification-lib.sh 的 CRASH_AMPLIFICATION_LIBRARY_SUBDIRECTORY，按结论种类存之前的旧库 <家目录>/library 引擎开库就拒，不删、不读），
#   日志在 <库的上层>/runs/<标签>/：placement.log（排计划那一趟）、run.log（本机一台跑的那一趟）；两台跑时是双机驱动那几份。
# 输出：CRASH_AMPLIFICATION_PLACEMENT 行原样转打；一行 CRASH_AMPLIFICATION_DECISION where=<local|two-hosts|local-cpu|two-hosts-cpu|not-run> reason=<为什么> …；
#   那一趟的 CRASH_AMPLIFICATION、CRASH_AMPLIFICATION_GPU_CARD、CRASH_AMPLIFICATION_GPU_TOTAL 行原样转打。
#   排计划那一趟借不到显存而本次未跑（步骤 ②）：一行 CRASH_AMPLIFICATION_NOT_RUN step=placement test=crash_amplification::run_from_environment reason=<那一行 reason= 之后原样>，
#   不打 CRASH_AMPLIFICATION 行；崩溃放量入口照这一行把这一项记 not-run（入口文件头「一项怎么判」）。几条流里只有几条没借到的，那几条各一行（带 flow=），
#   跑的那一趟的 CRASH_AMPLIFICATION 行照转打。
# 退出码（0、2、77、78 的含义照 .claude/singlefs-ai-sop/rules/test-script-discipline.md「入口脚本：不带参数就是剪枝跑全部项」那张表）：
#   0        跑的那一趟退 0。有流本次未跑也是 0（几条流里没借到的那几条打了 CRASH_AMPLIFICATION_NOT_RUN 行、真跑那一趟被叫停的行里 stopped_early=true）：
#            退 0 不等于全判过了，看有没有 CRASH_AMPLIFICATION_NOT_RUN 行与 stopped_early=true
#   77       一项都没跑到：排计划那一趟借不到显存、一块都没判（打了 CRASH_AMPLIFICATION_NOT_RUN 行），或双机驱动退 77（第二台排计划那一趟借不到显存）。
#            本次未跑，不是通过，也不是失败
#   2        用法错（参数认不得、缺测试目标、带值的选项缺值：--expand-up-to、--test-filter、--library、--threads、--memory-cap、
#            --peer-memory-cap、--label 后面没给值，或给的是空串、以 - 起头）
#   78       运行条件不满足：开了 GPU 却没在多机配置里写这一趟用的那份显卡先后、点名的卡与双机开关对不上
#            （开跑前由 run-condition: check 拒；带 --force 越过时 crash_amplification_prepare 照样退 78）、说是清过场却没写清场之后那一份，
#            点名的卡用不了或显存装不下、结论的版本与树对不上（CRASH_POINT_REPLAY_NOT_WRITTEN_CONCLUSION_VERSION_DIFFERS_FROM_THE_BUILD 那一行，整趟不写库）、
#            多机配置开了 GPU 而二进制没编进 gpu 特性（这三样都照日志分类，crash-amplification-lib.sh 的 crash_amplification_failure_kind），双机驱动退 78（第二台连不上、清场没过、库没取回），清场回读没过、多机配置没写全清场那两个键
#            （scripts/gpu-quiesce-run.sh 判的，它打 GPU_QUIESCE step=command result=not-started）。出路：修好设备，或改配置
#   247      排派活计划那一趟退 0，却一行 CRASH_AMPLIFICATION_PLACEMENT 都没打，日志里也认不出是设备还是算法，也没有产品路径借不到显存的那一行（那一种退 77，见上）
#   248      算法上判不了：有状态 GPU 判不了（表越过内核容量）、内核展开超预算、GPU 判器建不出表（照日志分类，或双机驱动退 248）
#   249      第二台排派活计划没量出速度、日志里认不出原因（双机驱动原样；第二台那一趟借不到显存的不在这里，见 77）
#   245      内存包装认出的显存不够（那一趟退了非 0、起止之间内核报了 NV_ERR_NO_MEMORY）：那一趟的结果不算数，原样退，出路单独一句
#   250–254  内存包装自己的结局（scripts/run-with-memory-cap.sh 文件头「退出码」）
#   其余     跑的那一趟（cargo test）的退出码，原样
# 重型：跑 checker 档用例，只在用户要求时跑（SINGLEFS_HEAVY_TESTS=user-request 由这里带给 cargo）。
# 弄坏开关 CRASH_AMPLIFICATION_ITEM_BREAK=<项>（只给 --selftest 证红用；格号是 run_selftest 里的第 1–25 格，判错的那一格打「✗ 自证第 <号> 格」）：
#   gpu-without-quiesce                开了 GPU、没带 --quiesced 也不经清场包装，照不清场直接跑                     第 7 格、第 11 格、第 15 格、第 16 格、第 17 格、第 18 格、第 19 格
#   （清场包装 scripts/gpu-quiesce-run.sh 的开关 GPU_QUIESCE_BREAK 带进这里的子进程也认：run-despite-failed-quiesce 红在第 16 格，
#    restore-after-run 红在第 7 格、第 11 格、第 15 格、第 16 格、第 17 格、第 18 格、第 19 格（这几格的调用次序里多出 quiesce-start））
#   stay-local-when-across             双机开着也本机一台跑（「配置两台就跑两台」判错）                         第 1 格、第 7 格、第 19 格
#   ignore-across-switch               双机没开也动用第二台（「配置一台就跑一台」判错）                         第 2 格、第 11 格、第 15 格、第 17 格、第 18 格、第 20 格
#   accept-peer-cards-without-across   双机没开、显卡配置却点名了第二台的卡也照跑（「配置自相矛盾就拒绝」判错）   第 3 格
#   accept-missing-peer-cards          双机开着、显卡配置里却没有第二台的卡也照跑（「两台都要有 GPU 干活」判错）   第 4 格
#   accept-missing-placement           计划行一行都没有也照本机跑（「没有计划行就不跑」判错）                   第 6 格、第 8 格、第 9 格、第 19 格
#   placement-not-run-as-refusal       排计划那一趟借不到显存的那一行不认，照没有计划行退 247（「本次未跑不是拒绝」判错）  第 19 格
#   accept-any-not-run-line            别的用例名的 NOT_RUN_NO_GPU_ADAPTER 行也当产品路径借不到显存（「只认产品路径那一行」判错）  第 19 格
#   accept-not-run-after-failed-placement  排计划那一趟退了非 0 也照那一行记本次未跑（「退了非 0 就照日志分类」判错）  第 19 格
#   not-run-exits-0                    一项都没跑到（排计划那一趟借不到显存、双机驱动退 77）照改前退 0（「一项都没跑到退 77」判错）  第 19 格
#   partial-not-run-unreported         有计划行时那几条借不到显存的流不报（改前：只照计划行往下跑）（「没借到的流逐条报本次未跑」判错）  第 20 格
#   local-run-when-the-second-host-fails  第二台没量出速度时改在本机一台跑（「不退回少一台」判错）              第 7 格
#   device-unusable-as-run-failure     日志里卡用不上的原话不认（不退 78）                                    第 8 格、第 11 格
#   algorithm-refusal-as-run-failure   日志里算法判不了的原话不认（不退 248）                                 第 9 格
#   accept-missing-card-list           开了 GPU、多机配置里一份卡都没写也照跑（「没写卡就退 78」判错）           第 10 格、第 13 格
#   missing-value-exits-1              带值的选项缺值时退 1（不按用法错退 2）                                    第 12 格
#   gpu-memory-unexplained             本机那一趟退 245（显存不够）时照别的退出码打通用出路、不说显存不够         第 18 格
#   （共用库 crash-amplification-lib.sh 里配置那几处的开关：cards-read-from-cards-file（显卡读退役的私有显卡配置 SINGLEFS_GPU_CARDS_CONFIG，不读多机配置）红在第 13 格、第 14 格；
#    configuration-from-tree-root（多机配置按树根找、不按 git common-dir 找）红在第 21 格）
#   （共用库的开关 given-states-run-condition-as-run-failure（给定状态表的判器借不到显存那一行 CRASH_POINT_REPLAY_GIVEN_STATES_RUN_CONDITION_NOT_MET 不认、不退 78）
#    打开时这里 25 格一格不红：这里的格都不造那一行；它由 scripts/crash-amplification-two-hosts.sh --selftest 证红，红在 local-share-given-states-run-condition 格）
#   version-differs-as-run-failure     日志里结论版本对不上那一行不认（不退 78）                               第 23 格
#   gpu-feature-missing-as-run-failure 日志里「这份二进制没有编进 gpu 特性」不认（不退 78）                    第 24 格
#   library-in-the-old-directory       没给 --library 时照改前用 <家目录>/library（按结论种类存之前的旧库）      第 25 格
#   （多机配置判法 scripts/layer0-shard-configuration-check.sh 的开关 MULTI_HOST_CONFIGURATION_BREAK=gpu-card-keys-unknown 红在第 22 格）
# 只供测试：CRASH_AMPLIFICATION_ITEM_DECIDE_ONLY=<造好的 placement.log> 时不编不跑，只拿那份日志判在哪跑、打 CRASH_AMPLIFICATION_DECISION 行就退
#   （一块都没判那一种另打 CRASH_AMPLIFICATION_NOT_RUN 行、退 77；有流没借到的照样逐条打那一行）；
#   CRASH_AMPLIFICATION_ITEM_SWITCHES=<gpu>,<双机> 代替读配置里的这两样；CRASH_AMPLIFICATION_ITEM_CARDS=<显卡配置的值> 代替读显卡配置；
#   CRASH_AMPLIFICATION_ITEM_LOCAL_RUN=<脚本> 换掉本机那一趟（调法：<脚本> <share> <库目录> <日志> [<附加环境 键=值>…]，自己写日志）；
#   CRASH_AMPLIFICATION_TWO_HOSTS_DRIVER=<脚本> 换掉双机驱动。换不掉的：真的那一趟由 crash-amplification-lib.sh 起 cargo，自证不碰它。
#
# gate-similar: crash-amplification-two-hosts.sh 它管两台各领一份、取回、导入那一趟；这里核配置、排派活计划、定一台还是两台，本机一台跑的那一趟自己跑
# gate-similar: crash-amplification.sh 它是分项登记表与全量的循环，全量经清场包装清一次、每项带 --quiesced 调这里
# gate-similar: gpu-quiesce-run.sh 它只管清场、回读、跑（不复原）；这里开了 GPU 而没带 --quiesced 时经它重起自己
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

item_original_arguments=("$@")
item_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repository_root="$(cd "$item_script_directory/../.." && pwd)"
source "$item_script_directory/crash-amplification-lib.sh"

item_break_is() { crash_amplification_item_break_is "$1"; }
exit_code_is_explained() { # <要退的码>：78、248 与 245（显存不够）由 crash_amplification_explain_exit_code 打出路；弄坏开关 gpu-memory-unexplained 下 245 不算
  [[ "$1" == 78 || "$1" == 248 ]] || { [[ "$1" == 245 ]] && ! item_break_is gpu-memory-unexplained; }
}

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

# 产品路径借不到显存的那一行（文件头步骤 ②）：起过卡池的进程等满了也借不到时 Rust 打的，记号与产品路径的名字照
#   crates/singlefs-harness/src/gpu_memory_ledger.rs 的 GPU_NOT_RUN_MARKER 与 crates/singlefs-checker-tier/src/crash_amplification.rs 的 PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE
PLACEMENT_NOT_RUN_PREFIX='NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment reason='
placement_not_run_line() { # <placement.log> <排计划那一趟的退出码>：那一趟退 0、日志里有那一行就打第一行，别的打空串
  local prefix="$PLACEMENT_NOT_RUN_PREFIX"
  item_break_is placement-not-run-as-refusal && return 0
  [[ "$2" == 0 ]] || item_break_is accept-not-run-after-failed-placement || return 0
  item_break_is accept-any-not-run-line && prefix='NOT_RUN_NO_GPU_ADAPTER test='
  [[ -f "$1" ]] || return 0
  awk -v prefix="$prefix" 'index($0, prefix) == 1 { print; exit }' "$1"
}
# 本次未跑（文件头步骤 ②、「输出」）：打 CRASH_AMPLIFICATION_NOT_RUN 行，标准错误说一句这不是通过
report_placement_not_run() { # <那一行>
  local reason="${1#*reason=}"
  echo "CRASH_AMPLIFICATION_NOT_RUN step=placement test=crash_amplification::run_from_environment reason=$reason"
  echo "  ! 本次未跑（这不是通过）：排派活计划那一趟借不到显存，一块都没判：$reason" >&2
  echo "  · 等占着卡的 GPU 用例或测试进程跑完，再重跑同一条命令（判过的块在库里，接着判）" >&2
}
# 有计划行、又有那一行（文件头步骤 ②：一条用例几条流，有的借到有的没借到）：没借到的流逐条打 CRASH_AMPLIFICATION_NOT_RUN 行，
#   第几条按 placement.log 里计划行与那一行的先后数；标准错误逐条说这不是通过。没有这种行时什么都不打
report_placement_partial_not_run() { # <placement.log>
  local partial_lines partial_line
  item_break_is partial-not-run-unreported && return 0
  [[ -f "$1" ]] || return 0
  partial_lines="$(awk -v prefix="$PLACEMENT_NOT_RUN_PREFIX" '
    index($0, "CRASH_AMPLIFICATION_PLACEMENT ") == 1 { flows++; next }
    index($0, prefix) == 1 { flows++; missing++; ordinal[missing] = flows; reason[missing] = substr($0, length(prefix) + 1) }
    END { for (i = 1; i <= missing; i++) printf "CRASH_AMPLIFICATION_NOT_RUN step=placement test=crash_amplification::run_from_environment flow=%d/%d reason=%s\n", ordinal[i], flows, reason[i] }' "$1")"
  [[ -n "$partial_lines" ]] || return 0
  printf '%s\n' "$partial_lines"
  while IFS= read -r partial_line; do
    partial_line="${partial_line#* flow=}"
    echo "  ! 本次未跑（这不是通过）：排派活计划那一趟第 ${partial_line%% *} 条流借不到显存，没排进计划、这一趟不判它：${partial_line#*reason=}" >&2
  done <<<"$partial_lines"
  echo "  · 借到的流照计划判；没借到的那几条，等占着卡的 GPU 用例或测试进程跑完再重跑同一条命令（判过的块在库里，接着判）" >&2
}

# 在哪跑：读 placement.log 与双机开关，打 CRASH_AMPLIFICATION_DECISION 行；交回 0 本机一台跑、10 两台跑、
# 20 本次未跑（那一行放进 decided_not_run_line：排计划那一趟退 0、没有计划行、有产品路径借不到显存的那一行）、
# 1 没有计划行（该退的码放进 refused_exit_code：设备用不上 78、算法上判不了 248、那一趟退了非 0 就是它的退出码、退 0 却没打计划行 247）
decide_where_to_run() { # <placement.log> <双机开关 0/1> <排计划那一趟的退出码>
  local placement_log="$1" across="$2" placement_exit="$3"
  local pending local_speed kind
  pending="$(crash_amplification_placement_total "$placement_log" pending_states)"
  if [[ -z "$pending" ]]; then
    decided_not_run_line="$(placement_not_run_line "$placement_log" "$placement_exit")"
    if [[ -n "$decided_not_run_line" ]]; then
      echo "CRASH_AMPLIFICATION_DECISION where=not-run reason=no-card-of-the-pool-had-room-in-time across_machines=$across quiesced=$quiesced"
      return 20
    fi
    if item_break_is accept-missing-placement; then
      echo "CRASH_AMPLIFICATION_DECISION where=local reason=no-placement-line"
      return 0
    fi
    echo "  ✗ 排派活计划那一趟（退 $placement_exit）一行 CRASH_AMPLIFICATION_PLACEMENT 都没打，不跑；日志 $placement_log" >&2
    grep -h -A 3 'panicked at' "$placement_log" 2>/dev/null | head -8 | sed 's/^/    /' >&2
    kind="$(crash_amplification_failure_kind "$placement_log")"
    case "$kind" in
      device|build) refused_exit_code=78 ;;
      algorithm) refused_exit_code=248 ;;
      *) if [[ "$placement_exit" != 0 ]]; then refused_exit_code="$placement_exit"; else refused_exit_code=247; fi ;;
    esac
    if exit_code_is_explained "$refused_exit_code"; then
      crash_amplification_explain_exit_code "$refused_exit_code"
    else
      echo "  → 怎么办：照上面 panicked 那几行与日志全文改（250–254 是内存包装自己的结局）；日志里认不出是设备还是算法，单跑那一趟看全文（退 $refused_exit_code）" >&2
    fi
    return 1
  fi
  report_placement_partial_not_run "$placement_log"
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
  # 子进程的运行条件只认这里给的：多机配置指到只写了假清场命令的一份（两个开关都关、一份卡都没写），开关与卡由各格的 CRASH_AMPLIFICATION_ITEM_* 给。
  #   退役的私有显卡配置（SINGLEFS_GPU_CARDS_CONFIG）指到一份写了只点名第二台的卡与一类计算的假文件：读到它就是读错了（第 13、14 格；弄坏开关
  #   cards-read-from-cards-file 下这几个子进程照它读，也不会去读这台机器真的私有配置）
  export SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host.env" SINGLEFS_GPU_CARDS_CONFIG="$work/retired-gpu-cards.env"
  printf '%s\n' "GPU_CARDS_BY_PRIORITY='peer:0:8G'" "GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED='peer:0:8G'" "GPU_COMPUTATION_CLASSES='crash_judge_gpu'" >"$work/retired-gpu-cards.env"
  # 多机配置里只写假的清场命令（开了 GPU 而没带 --quiesced 的格经清场包装重起这里）：清场与复原各往 STUB_CALLS 记一笔
  #   （包装不调复原，quiesce-start 那一笔只在它的弄坏开关 restore-after-run 下出现），STUB_STOPPED_CHECK 写成 fail 时清场回读不过；
  #   包装的登记与本机日志都放这里的临时目录，不落进真工作根
  printf '%s\n' \
    "QUIESCE_STOP_COMMAND='echo quiesce-stop >> \"\$STUB_CALLS\"'" \
    "QUIESCE_STOPPED_CHECK_COMMAND='[[ \"\${STUB_STOPPED_CHECK:-ok}\" == ok ]]'" \
    "QUIESCE_START_COMMAND='echo quiesce-start >> \"\$STUB_CALLS\"'" \
    "QUIESCE_STARTED_CHECK_COMMAND='[[ \"\${STUB_STARTED_CHECK:-ok}\" == ok ]]'" >"$work/multi-host.env"
  # 经清场包装重起、带着 --quiesced 的那一趟要清场之后那一份显卡先后（卡还是由 CRASH_AMPLIFICATION_ITEM_CARDS 给）：第 7、11、15–20 格指到这一份（多机配置里另写那一个键）
  { cat "$work/multi-host.env"; printf '%s\n' 'GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED="local:0:15G"'; } >"$work/multi-host-quiesced.env"
  export GPU_QUIESCE_STATE_DIRECTORY="$work/gpu-quiesce-state"
  export GPU_QUIESCE_LOG_DIRECTORY="$work/gpu-quiesce-logs"
  # 第 13、14 格的两份多机配置：开了 GPU、一份卡都没写（13）；开了 GPU、只点名本机的卡（14）
  printf '%s\n' ENABLE_GPU=1 >"$work/multi-host-gpu-on-without-cards.env"
  printf '%s\n' ENABLE_GPU=1 'GPU_CARDS_BY_PRIORITY="local:0:15G"' >"$work/multi-host-gpu-on-with-cards.env"
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
  if [[ "${STUB_PLACEMENT_NOT_RUN:-0}" == 1 ]]; then
    # 第 19 格：演起过卡池的产品路径借不到显存——照 Rust 那一行打（不打计划行），退 STUB_PLACEMENT_EXIT（没给退 0）
    printf '%s\n' "running 1 test" \
      "NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment reason=等了 1800 秒，池里没有一张卡空得出 2560 MiB（每张卡的账 [15360]；这台机器上别的 GPU 用例或测试进程一直占着）" \
      "test result: ok. 1 passed; 0 failed" > "$log"
    exit "${STUB_PLACEMENT_EXIT:-0}"
  fi
  if [[ "${STUB_PLACEMENT_PARTIAL:-0}" == 1 ]]; then
    # 第 20 格：一条用例三条流，第 2 条借不到显存——两行计划夹着 Rust 那一行；STUB_PLACEMENT_PARTIAL_TEST 换掉那一行的用例名（别的用例的那一行）
    printf '%s\n' "running 1 test" \
      "CRASH_AMPLIFICATION_PLACEMENT host=local judge=gpu pending_blocks=4 pending_states=1000 tables_mebibytes=10 cards=local:0:100:100 states_placed_on_local=1000 states_placed_on_peer=0 states_per_second_on_this_host=1 estimated_seconds_on_this_host=1000" \
      "NOT_RUN_NO_GPU_ADAPTER test=${STUB_PLACEMENT_PARTIAL_TEST:-crash_amplification::run_from_environment} reason=等了 1800 秒，池里没有一张卡空得出 5120 MiB（第 2 条流）" \
      "CRASH_AMPLIFICATION_PLACEMENT host=local judge=gpu pending_blocks=4 pending_states=1000 tables_mebibytes=10 cards=local:0:100:100 states_placed_on_local=1000 states_placed_on_peer=0 states_per_second_on_this_host=1 estimated_seconds_on_this_host=1000" \
      "test result: ok. 1 passed; 0 failed" > "$log"
    exit 0
  fi
  echo "CRASH_AMPLIFICATION_PLACEMENT host=local judge=gpu pending_blocks=4 pending_states=1000 tables_mebibytes=10 cards=local:0:100:100,peer:2:50:50 states_placed_on_local=100 states_placed_on_peer=50 states_per_second_on_this_host=1 estimated_seconds_on_this_host=1000" > "$log"
else
  echo "local-run" >> "${STUB_CALLS:?}"
  if [[ "${STUB_RUN_SELF_TERM:-0}" == 1 ]]; then kill -s TERM "$$"; fi
  if [[ "${STUB_RUN_DEVICE_FAILURE:-0}" == 1 ]]; then
    echo "GPU_CARD_NOT_USABLE index=0：配置点名了这张卡，本机却用不了它" > "$log"
    exit 101
  fi
  if [[ "${STUB_RUN_GPU_MEMORY:-0}" == 1 ]]; then
    # 第 18 格：演经内存包装跑的那一趟——测试红了、包装认出起止之间内核报了显存不够，退 245
    printf '%s\n' "test result: FAILED" "run-with-memory-cap: 显存不够（假的本机那一趟）" > "$log"
    exit 245
  fi
  echo "CRASH_AMPLIFICATION mode=two_hosts_cpu_with_gpus states_checked_here=1000 red=0" > "$log"
fi
STUB
  cat >"$work/two-hosts.sh" <<'STUB'
#!/usr/bin/env bash
# 假的双机驱动：第二台没量出速度；STUB_TWO_HOSTS_NOT_RUN=1 时演第二台排计划那一趟借不到显存（照真双机驱动打那一行、退 77）
echo "two-hosts" >> "${STUB_CALLS:?}"
if [[ "${STUB_TWO_HOSTS_NOT_RUN:-0}" == 1 ]]; then
  echo "CRASH_AMPLIFICATION_NOT_RUN step=peer-placement test=crash_amplification::run_from_environment reason=等了 1800 秒，第二台池里没有一张卡空得出 2560 MiB"
  exit 77
fi
echo "CRASH_AMPLIFICATION_PEER_PLACEMENT_FAILED exit=250"
exit 249
STUB
  : > "$work/calls"
  local failed_output failed_exit
  failed_output="$(STUB_CALLS="$work/calls" SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-quiesced.env" CRASH_AMPLIFICATION_ITEM_SWITCHES=1,1 CRASH_AMPLIFICATION_ITEM_CARDS="$both" CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" \
    CRASH_AMPLIFICATION_TWO_HOSTS_DRIVER="$work/two-hosts.sh" bash "$0" some-target --library "$work/library" 2>/dev/null)"; failed_exit=$?
  if [[ "$failed_exit" != 249 ]] || [[ "$(tr '\n' ' ' <"$work/calls")" != "quiesce-stop placement two-hosts " ]] || grep -q 'where=local' <<<"$failed_output"; then
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
  STUB_CALLS="$work/calls" STUB_RUN_DEVICE_FAILURE=1 SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-quiesced.env" CRASH_AMPLIFICATION_ITEM_SWITCHES=1,0 CRASH_AMPLIFICATION_ITEM_CARDS="$local_only" \
    CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" CRASH_AMPLIFICATION_TWO_HOSTS_DRIVER="$work/two-hosts.sh" \
    bash "$0" some-target --library "$work/library-device" >/dev/null 2>&1; device_run_exit=$?
  if [[ "$device_run_exit" != 78 ]] || [[ "$(tr '\n' ' ' <"$work/calls")" != "quiesce-stop placement local-run " ]]; then
    selftest_cell_failed "本机那一趟卡用不了却没退 78、或没照「清场、排计划、跑」走（退 $device_run_exit，调用次序「$(tr '\n' ' ' <"$work/calls")」）" "看 run_on_this_host_alone：退非 0 时经 crash_amplification_exit_code_of_a_failed_run 照 run.log 分类；清场看开了 GPU 而没带 --quiesced 时经 gpu-quiesce-run.sh 重起那一段"
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
  # 第 13、14 格：运行条件（文件头 run-condition: check 调的 crash_amplification_run_condition_holds，不经 preflight 直接调，看它自己的 ✗）读的是多机配置里的显卡先后，
  #   不读退役的私有显卡配置（那一份只点名第二台的卡）：
  #   13 开了 GPU、多机配置里一份卡都没写 ⇒ 交回 1、打「一份都没写」（读了私有那一份会报成「点名了第二台的卡」）；
  #   14 开了 GPU、多机配置只点名本机的卡、双机没开 ⇒ 交回 0（读了私有那一份会因为点名了第二台的卡交回 1）
  local cards_cell cards_configuration cards_exit cards_want
  for cards_cell in 13 14; do
    cells=$((cells + 1))
    if [[ "$cards_cell" == 13 ]]; then cards_configuration="$work/multi-host-gpu-on-without-cards.env"; cards_want=1; else cards_configuration="$work/multi-host-gpu-on-with-cards.env"; cards_want=0; fi
    env -u CRASH_AMPLIFICATION_ITEM_SWITCHES -u CRASH_AMPLIFICATION_ITEM_CARDS SINGLEFS_MULTI_HOST_CONFIG="$cards_configuration" \
      bash -c 'source "$1" && crash_amplification_run_condition_holds "$2"' run-condition "$item_script_directory/crash-amplification-lib.sh" "$work" \
      >"$work/cards-$cards_cell.out" 2>&1; cards_exit=$?
    if [[ "$cards_exit" != "$cards_want" ]] || { [[ "$cards_cell" == 13 ]] && ! grep -q 'GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED 一份都没写' "$work/cards-$cards_cell.out"; }; then
      selftest_cell_failed "开了 GPU、多机配置$( [[ "$cards_cell" == 13 ]] && echo 一份卡都没写 || echo 只点名本机的卡 )：运行条件应当交回 $cards_want$( [[ "$cards_cell" == 13 ]] && echo '、打「一份都没写」')，实际交回 $cards_exit（$(tr '\n' ' ' <"$work/cards-$cards_cell.out" | cut -c1-200)）" \
        "看 crash-amplification-lib.sh 的 crash_amplification_run_condition_holds：显卡先后从多机配置读（crash_amplification_cards_source_file），不读 SINGLEFS_GPU_CARDS_CONFIG"
    fi
  done
  # 第 15–17 格：开了 GPU、没带 --quiesced ⇒ 经清场包装：清场、回读、带 --quiesced 重起这里，跑完不复原（多机配置里是假的清场命令）
  quiesce_cell() { # <要的退出码> <要的调用次序> <这一格判错的一句> [<环境 键=值>…]：起一趟本机一台跑，退出码与调用次序都对交回 0
    local want_exit="$1" want_calls="$2" sentence="$3" quiesce_exit
    shift 3
    cells=$((cells + 1)); : > "$work/calls"
    env STUB_CALLS="$work/calls" SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-quiesced.env" CRASH_AMPLIFICATION_ITEM_SWITCHES=1,0 CRASH_AMPLIFICATION_ITEM_CARDS="$local_only" \
      CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" CRASH_AMPLIFICATION_TWO_HOSTS_DRIVER="$work/two-hosts.sh" "$@" \
      bash "$0" some-target --library "$work/library-quiesce-$cells" >"$work/quiesce-$cells.out" 2>&1; quiesce_exit=$?
    if [[ "$quiesce_exit" != "$want_exit" || "$(tr '\n' ' ' <"$work/calls")" != "$want_calls" ]]; then
      selftest_cell_failed "$sentence（退 $quiesce_exit，调用次序「$(tr '\n' ' ' <"$work/calls")」）" "看开了 GPU 而没带 --quiesced 时经 scripts/gpu-quiesce-run.sh 重起这里的那一段，与那份包装的 gpu_quiesce_leave"
      return 1
    fi
  }
  # 第 15 格：照常跑完 ⇒ 退 0，清场在排计划之前，重起的那一趟带着 --quiesced（DECISION 行 quiesced=1）
  if quiesce_cell 0 "quiesce-stop placement local-run " "开了 GPU、没带 --quiesced 却没有照「清场、排计划、跑」走"; then
    grep -q '^CRASH_AMPLIFICATION_DECISION where=local .* quiesced=1$' "$work/quiesce-$cells.out" \
      || selftest_cell_failed "经清场包装重起的那一趟没带 --quiesced（DECISION 行不是 quiesced=1）" "看重起的参数 item_reexec_arguments"
  fi
  # 第 16 格：清场回读不过 ⇒ 不排计划、不跑，退 78
  quiesce_cell 78 "quiesce-stop " "清场回读不过却起了这一项、或没退 78" STUB_STOPPED_CHECK=fail
  # 第 17 格：跑完不调复原命令：照常跑完（退 0）、本机那一趟被 TERM 打断（原样退 143）都只清一次场、不复原，
  #   清场包装打 GPU_QUIESCE step=restore result=not-restored
  cells=$((cells + 1))
  local no_restore_setting no_restore_exit no_restore_exits="" no_restore_calls="" no_restore_lines=0 no_restore_round=0
  for no_restore_setting in STUB_RUN_SELF_TERM=0 STUB_RUN_SELF_TERM=1; do
    no_restore_round=$((no_restore_round + 1)); : > "$work/calls"
    env STUB_CALLS="$work/calls" SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-quiesced.env" CRASH_AMPLIFICATION_ITEM_SWITCHES=1,0 CRASH_AMPLIFICATION_ITEM_CARDS="$local_only" \
      CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" CRASH_AMPLIFICATION_TWO_HOSTS_DRIVER="$work/two-hosts.sh" "$no_restore_setting" \
      bash "$0" some-target --library "$work/library-no-restore-$no_restore_round" >"$work/no-restore-$no_restore_round.out" 2>&1; no_restore_exit=$?
    no_restore_exits+="$no_restore_exit "
    no_restore_calls+="$(tr '\n' ' ' <"$work/calls")| "
    if grep -q '^GPU_QUIESCE step=restore result=not-restored ' "$work/no-restore-$no_restore_round.out"; then no_restore_lines=$((no_restore_lines + 1)); fi
  done
  if [[ "$no_restore_exits" != "0 143 " || "$no_restore_calls" != "quiesce-stop placement local-run | quiesce-stop placement local-run | " || "$no_restore_lines" != 2 ]]; then
    selftest_cell_failed "跑完调了复原命令、或没照「清场、排计划、跑」走（照常跑完与被 TERM 打断两趟退「${no_restore_exits% }」，调用次序「$no_restore_calls」，GPU_QUIESCE step=restore result=not-restored 行 $no_restore_lines 趟有）" \
      "看开了 GPU 而没带 --quiesced 时经 scripts/gpu-quiesce-run.sh 重起这里的那一段，与那份包装的 gpu_quiesce_leave：跑完不复原"
  fi
  # 第 18 格：本机一台跑的那一趟经内存包装退 245（显存不够）⇒ 原样退 245、打显存不够的出路（不当成卡用不了，也不只打通用的那一句）
  cells=$((cells + 1))
  : > "$work/calls"
  local gpu_memory_exit
  STUB_CALLS="$work/calls" STUB_RUN_GPU_MEMORY=1 SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-quiesced.env" CRASH_AMPLIFICATION_ITEM_SWITCHES=1,0 CRASH_AMPLIFICATION_ITEM_CARDS="$local_only" \
    CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" CRASH_AMPLIFICATION_TWO_HOSTS_DRIVER="$work/two-hosts.sh" \
    bash "$0" some-target --library "$work/library-gpu-memory" >"$work/gpu-memory.out" 2>&1; gpu_memory_exit=$?
  if [[ "$gpu_memory_exit" != 245 ]] || ! grep -q '显存不够（那一趟退了非 0' "$work/gpu-memory.out" || [[ "$(tr '\n' ' ' <"$work/calls")" != "quiesce-stop placement local-run " ]]; then
    selftest_cell_failed "本机那一趟经内存包装退 245 却没原样退 245、或没打显存不够的出路（退 $gpu_memory_exit，调用次序「$(tr '\n' ' ' <"$work/calls")」，输出尾部 $(tail -n 3 "$work/gpu-memory.out" | tr '\n' '|')）" \
      "看 run_on_this_host_alone 与 exit_code_is_explained：245 走 crash-amplification-lib.sh 的 crash_amplification_explain_exit_code 那一支"
  fi
  # 第 19 格：排计划那一趟借不到显存（起过卡池的产品路径打 NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment 那一行、退 0、没有计划行）⇒
  #   一项都没跑到：打 DECISION where=not-run 与 CRASH_AMPLIFICATION_NOT_RUN 行、退 77，不跑、不交双机驱动（双机开着也一样），不退 247、不退 0。
  #   两样不算这一种：别的用例名的 NOT_RUN_NO_GPU_ADAPTER 行照没有计划行退 247；那一趟退了非 0（101）照日志分类原样退 101、也不跑。
  #   第二台排计划那一趟借不到显存（双机驱动打 step=peer-placement 那一行、退 77）⇒ 原样退 77、报本次未跑，不报「两台跑那一趟退 77」那一句失败
  cells=$((cells + 1))
  local not_run_exit not_run_calls not_run_wrong=""
  : > "$work/calls"
  STUB_CALLS="$work/calls" STUB_PLACEMENT_NOT_RUN=1 SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-quiesced.env" CRASH_AMPLIFICATION_ITEM_SWITCHES=1,1 CRASH_AMPLIFICATION_ITEM_CARDS="$both" \
    CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" CRASH_AMPLIFICATION_TWO_HOSTS_DRIVER="$work/two-hosts.sh" \
    bash "$0" some-target --library "$work/library-not-run" >"$work/not-run.out" 2>&1; not_run_exit=$?
  not_run_calls="$(tr '\n' ' ' <"$work/calls")"
  if [[ "$not_run_exit" != 77 || "$not_run_calls" != "quiesce-stop placement " ]] \
     || ! grep -q '^CRASH_AMPLIFICATION_DECISION where=not-run ' "$work/not-run.out" \
     || ! grep -q '^CRASH_AMPLIFICATION_NOT_RUN step=placement test=crash_amplification::run_from_environment reason=等了 1800 秒，池里没有一张卡空得出 2560 MiB' "$work/not-run.out" \
     || ! grep -q '本次未跑（这不是通过）：排派活计划那一趟借不到显存' "$work/not-run.out" || grep -q '^CRASH_AMPLIFICATION ' "$work/not-run.out"; then
    not_run_wrong+="产品路径借不到显存那一趟退 $not_run_exit、调用次序「$not_run_calls」、输出尾部 $(tail -n 3 "$work/not-run.out" | tr '\n' '|')；"
  fi
  printf '%s\n' "NOT_RUN_NO_GPU_ADAPTER test=some_gpu_parity_test reason=本机没有 NVIDIA 卡" "test result: ok. 1 passed; 0 failed" >"$work/other-not-run.log"
  [[ "$(decision_of "$work/other-not-run.log" 1,0 "$local_only")" == " 247" ]] \
    || not_run_wrong+="别的用例名的 NOT_RUN_NO_GPU_ADAPTER 行没照没有计划行退 247（$(decision_of "$work/other-not-run.log" 1,0 "$local_only")）；"
  : > "$work/calls"
  STUB_CALLS="$work/calls" STUB_PLACEMENT_NOT_RUN=1 STUB_PLACEMENT_EXIT=101 SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-quiesced.env" CRASH_AMPLIFICATION_ITEM_SWITCHES=1,0 CRASH_AMPLIFICATION_ITEM_CARDS="$local_only" \
    CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" CRASH_AMPLIFICATION_TWO_HOSTS_DRIVER="$work/two-hosts.sh" \
    bash "$0" some-target --library "$work/library-not-run-failed" >"$work/not-run-failed.out" 2>&1; not_run_exit=$?
  not_run_calls="$(tr '\n' ' ' <"$work/calls")"
  if [[ "$not_run_exit" != 101 || "$not_run_calls" != "quiesce-stop placement " ]] || grep -q '^CRASH_AMPLIFICATION_NOT_RUN ' "$work/not-run-failed.out"; then
    not_run_wrong+="排计划那一趟退 101 而日志里有那一行，退 $not_run_exit、调用次序「$not_run_calls」（要原样退 101、不跑、不打 CRASH_AMPLIFICATION_NOT_RUN）；"
  fi
  : > "$work/calls"
  STUB_CALLS="$work/calls" STUB_TWO_HOSTS_NOT_RUN=1 SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-quiesced.env" CRASH_AMPLIFICATION_ITEM_SWITCHES=1,1 CRASH_AMPLIFICATION_ITEM_CARDS="$both" \
    CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" CRASH_AMPLIFICATION_TWO_HOSTS_DRIVER="$work/two-hosts.sh" \
    bash "$0" some-target --quiesced --library "$work/library-peer-not-run" >"$work/peer-not-run.out" 2>&1; not_run_exit=$?
  not_run_calls="$(tr '\n' ' ' <"$work/calls")"
  if [[ "$not_run_exit" != 77 || "$not_run_calls" != "placement two-hosts " ]] \
     || ! grep -q '^CRASH_AMPLIFICATION_NOT_RUN step=peer-placement ' "$work/peer-not-run.out" \
     || ! grep -q '本次未跑（这不是通过）：第二台排派活计划那一趟借不到显存' "$work/peer-not-run.out" || grep -q '两台跑那一趟退 77' "$work/peer-not-run.out"; then
    not_run_wrong+="第二台排计划那一趟借不到显存（双机驱动退 77），退 $not_run_exit、调用次序「$not_run_calls」、输出尾部 $(tail -n 3 "$work/peer-not-run.out" | tr '\n' '|')（要原样退 77、报本次未跑、不报两台跑失败）；"
  fi
  [[ -z "$not_run_wrong" ]] \
    || selftest_cell_failed "排计划那一趟借不到显存没按一项都没跑到退 77、或不该算的也算了：$not_run_wrong" \
      "看 decide_where_to_run 里 pending 为空那一支开头的 placement_not_run_line（只认那一趟退 0、行首是「$PLACEMENT_NOT_RUN_PREFIX」的那一行）、末尾 case 的 20 那一支与 10 那一支里双机驱动退 77 那一段"
  # 第 20 格：一条用例三条流、第 2 条借不到显存（两行计划夹着产品路径那一行，那一趟退 0）⇒ 借到的照判：本机一台跑（DECISION pending_states=2000、跑了 local-run、退 0），
  #   没借到的那一条打一行 CRASH_AMPLIFICATION_NOT_RUN step=placement … flow=2/3 reason=…、标准错误说这不是通过；只判不跑、双机开着也照打那一行。
  #   别的用例名的 NOT_RUN_NO_GPU_ADAPTER 行夹在计划行之间不算，不打 CRASH_AMPLIFICATION_NOT_RUN
  cells=$((cells + 1))
  local partial_exit partial_calls partial_wrong="" partial_decision
  : > "$work/calls"
  STUB_CALLS="$work/calls" STUB_PLACEMENT_PARTIAL=1 SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-quiesced.env" CRASH_AMPLIFICATION_ITEM_SWITCHES=1,0 CRASH_AMPLIFICATION_ITEM_CARDS="$local_only" \
    CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" CRASH_AMPLIFICATION_TWO_HOSTS_DRIVER="$work/two-hosts.sh" \
    bash "$0" some-target --quiesced --library "$work/library-partial" >"$work/partial.out" 2>&1; partial_exit=$?
  partial_calls="$(tr '\n' ' ' <"$work/calls")"
  if [[ "$partial_exit" != 0 || "$partial_calls" != "placement local-run " ]] \
     || ! grep -q '^CRASH_AMPLIFICATION_DECISION where=local .*pending_states=2000 ' "$work/partial.out" \
     || [[ "$(grep -c '^CRASH_AMPLIFICATION_NOT_RUN ' "$work/partial.out")" != 1 ]] \
     || ! grep -q '^CRASH_AMPLIFICATION_NOT_RUN step=placement test=crash_amplification::run_from_environment flow=2/3 reason=等了 1800 秒，池里没有一张卡空得出 5120 MiB（第 2 条流）$' "$work/partial.out" \
     || ! grep -q '本次未跑（这不是通过）：排派活计划那一趟第 2/3 条流借不到显存' "$work/partial.out" || ! grep -q '^CRASH_AMPLIFICATION mode=' "$work/partial.out"; then
    partial_wrong+="三条流第 2 条借不到显存那一趟退 $partial_exit、调用次序「$partial_calls」、CRASH_AMPLIFICATION_NOT_RUN $(grep -c '^CRASH_AMPLIFICATION_NOT_RUN ' "$work/partial.out") 行、输出尾部 $(tail -n 4 "$work/partial.out" | tr '\n' '|')；"
  fi
  printf '%s\n' "running 1 test" "$(placement_line 1000 10)" \
    "NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment reason=第 2 条流借不到" "$(placement_line 1000 10)" >"$work/partial-plan.log"
  partial_decision="$(CRASH_AMPLIFICATION_ITEM_DECIDE_ONLY="$work/partial-plan.log" CRASH_AMPLIFICATION_ITEM_SWITCHES=1,1 CRASH_AMPLIFICATION_ITEM_CARDS="$both" bash "$0" some-target 2>/dev/null)"
  grep -q '^CRASH_AMPLIFICATION_NOT_RUN step=placement test=crash_amplification::run_from_environment flow=2/3 reason=第 2 条流借不到$' <<<"$partial_decision" \
    || partial_wrong+="只判不跑、双机开着时没打 flow=2/3 那一行：$(tr '\n' '|' <<<"$partial_decision")；"
  printf '%s\n' "$(placement_line 1000 10)" "NOT_RUN_NO_GPU_ADAPTER test=some_gpu_parity_test reason=本机没有 NVIDIA 卡" "$(placement_line 1000 10)" >"$work/partial-other.log"
  partial_decision="$(CRASH_AMPLIFICATION_ITEM_DECIDE_ONLY="$work/partial-other.log" CRASH_AMPLIFICATION_ITEM_SWITCHES=1,0 CRASH_AMPLIFICATION_ITEM_CARDS="$local_only" bash "$0" some-target 2>/dev/null)"
  if grep -q '^CRASH_AMPLIFICATION_NOT_RUN ' <<<"$partial_decision"; then partial_wrong+="别的用例名的那一行夹在计划行之间也打了 CRASH_AMPLIFICATION_NOT_RUN：$(tr '\n' '|' <<<"$partial_decision")；"; fi
  [[ -z "$partial_wrong" ]] \
    || selftest_cell_failed "几条流里有的借不到显存，没借到的没逐条报本次未跑、或借到的没照判：$partial_wrong" \
      "看 decide_where_to_run 里 pending 不空那一支开头的 report_placement_partial_not_run（只认行首是「$PLACEMENT_NOT_RUN_PREFIX」的那一行，第几条按计划行与那一行的先后数）"
  # 第 21 格：没设 SINGLEFS_MULTI_HOST_CONFIG 时多机配置按 git common-dir 找主工作树那一份（在 worktree 里跑也读主工作树的 multi-host.env，判决 r3 第二节 M1-3）：
  #   临时目录里 git init 一个主仓、加一个 worktree，从 worktree 的树根找，要找到主仓根下的那一份；不是 git 工作树的目录退回它自己的树根
  cells=$((cells + 1))
  local common_found plain_found
  mkdir -p "$work/git-main" "$work/not-a-tree"
  if git -C "$work/git-main" init -q 2>/dev/null && git -C "$work/git-main" -c user.name=selftest -c user.email=selftest@invalid commit -q --allow-empty -m selftest 2>/dev/null \
     && git -C "$work/git-main" worktree add -q --detach "$work/git-worktree" 2>/dev/null; then
    common_found="$(env -u SINGLEFS_MULTI_HOST_CONFIG bash -c 'source "$1" && crash_amplification_multi_host_configuration_file "$2"' find "$item_script_directory/crash-amplification-lib.sh" "$work/git-worktree" 2>&1)"
    plain_found="$(env -u SINGLEFS_MULTI_HOST_CONFIG GIT_CEILING_DIRECTORIES="$work" bash -c 'source "$1" && crash_amplification_multi_host_configuration_file "$2"' find "$item_script_directory/crash-amplification-lib.sh" "$work/not-a-tree" 2>&1)"
    if [[ "$common_found" != "$(cd "$work/git-main" && pwd -P)/multi-host.env" && "$common_found" != "$work/git-main/multi-host.env" ]] || [[ "$plain_found" != "$work/not-a-tree/multi-host.env" ]]; then
      selftest_cell_failed "多机配置没按 git common-dir 找：从 worktree 找到「$common_found」（应当是 $work/git-main/multi-host.env），从不是 git 工作树的目录找到「$plain_found」（应当是 $work/not-a-tree/multi-host.env）" \
        "看 crash-amplification-lib.sh 的 crash_amplification_multi_host_configuration_file：取 git -C <树根> rev-parse --git-common-dir 的上一层"
    fi
  else
    selftest_cell_failed "临时目录里建不出主仓与 worktree（git init、commit、worktree add 有一步没成）" "看 git 在不在 PATH 上、TMPDIR 可不可写"
  fi
  # 第 22 格：多机配置判法（scripts/layer0-shard-configuration-check.sh）认显卡的两个键、不认退役的 GPU_COMPUTATION_CLASSES：
  #   八个必写键加 GPU_CARDS_BY_PRIORITY、GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED、双机没开 ⇒ 退 3（判得过、双机：关，不连第二台）；再加一行 GPU_COMPUTATION_CLASSES ⇒ 退 1、点名这个键
  cells=$((cells + 1))
  local shard_check_exit shard_check_retired_exit
  printf '%s\n' PEER_SSH_HOST=selftest-peer PEER_REPOSITORY_DIRECTORY=/selftest/peer PEER_CARGO_BIN_DIRECTORY=/selftest/cargo PEER_MEMORY_CAP=1G \
    QUIESCE_STOP_COMMAND= QUIESCE_STOPPED_CHECK_COMMAND= QUIESCE_START_COMMAND= QUIESCE_STARTED_CHECK_COMMAND= ENABLE_GPU=1 \
    'GPU_CARDS_BY_PRIORITY="local:0:15G"' 'GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED="local:0:24G"' >"$work/multi-host-merged.env"
  { cat "$work/multi-host-merged.env"; printf '%s\n' "GPU_COMPUTATION_CLASSES='crash_judge_gpu'"; } >"$work/multi-host-retired-key.env"
  SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-merged.env" bash "$item_script_directory/layer0-shard-configuration-check.sh" "$work" >"$work/shard-check.out" 2>&1; shard_check_exit=$?
  SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-retired-key.env" bash "$item_script_directory/layer0-shard-configuration-check.sh" "$work" >"$work/shard-check-retired.out" 2>&1; shard_check_retired_exit=$?
  if [[ "$shard_check_exit" != 3 || "$shard_check_retired_exit" != 1 ]] || ! grep -q 'GPU_COMPUTATION_CLASSES 不认得' "$work/shard-check-retired.out"; then
    selftest_cell_failed "多机配置写了显卡的两个键却判不过、或写了退役的 GPU_COMPUTATION_CLASSES 却判得过（两个键退 $shard_check_exit，应当 3；加退役的键退 $shard_check_retired_exit，应当 1、点名它：$(tr '\n' ' ' <"$work/shard-check-retired.out" | cut -c1-160)）" \
      "看 scripts/layer0-shard-configuration-check.sh 的 MULTI_HOST_GPU_CARD_KEYS 与认键那一段"
  fi
  # 第 23 格：排计划那一趟没打计划行，日志里是引擎打的结论版本对不上那一行（整趟不写库，引擎 E 报告第九节第 ① 条）⇒ 运行条件不满足，退 78
  cells=$((cells + 1))
  { echo "running 1 test"
    echo "CRASH_POINT_REPLAY_NOT_WRITTEN_CONCLUSION_VERSION_DIFFERS_FROM_THE_BUILD reason=table_differs entries=crash.rs（整趟不写库：重编再跑，或在提交那一趟的 worktree 里跑；退 78）"
    echo "thread 'the_case' panicked at crates/singlefs-checker-tier/src/crash_amplification.rs:1:1:"
    echo "test result: FAILED. 0 passed; 1 failed"; } >"$work/version-differs.log"
  [[ "$(decision_of "$work/version-differs.log" 1,0 "$local_only")" == " 78" ]] \
    || selftest_cell_failed "结论版本对不上那一行却没退 78：$(decision_of "$work/version-differs.log" 1,0 "$local_only")" "看 crash-amplification-lib.sh 的 crash_amplification_failure_kind 认不认行首 CRASH_POINT_REPLAY_NOT_WRITTEN_CONCLUSION_VERSION_DIFFERS_FROM_THE_BUILD（build 那一类），与 decide_where_to_run 里 device|build 那一支"
  # 第 24 格：排计划那一趟没打计划行，日志里是「这份二进制没有编进 gpu 特性」（多机配置开了 GPU 而编的不带 gpu 特性）⇒ 运行条件不满足，退 78
  cells=$((cells + 1))
  { echo "running 1 test"
    echo "thread 'the_case' panicked at crates/singlefs-checker-tier/src/crash_amplification.rs:1:1:"
    echo "配置写了 ENABLE_GPU=1，判器、核对与单元校验和都要走 GPU，这份二进制没有编进 gpu 特性（cargo … --features verdict-store,gpu）"
    echo "test result: FAILED. 0 passed; 1 failed"; } >"$work/gpu-feature-missing.log"
  [[ "$(decision_of "$work/gpu-feature-missing.log" 1,0 "$local_only")" == " 78" ]] \
    || selftest_cell_failed "开了 GPU 而二进制没编进 gpu 特性却没退 78：$(decision_of "$work/gpu-feature-missing.log" 1,0 "$local_only")" "看 crash-amplification-lib.sh 的 crash_amplification_failure_kind 认不认「这份二进制没有编进 gpu 特性」（build 那一类）"
  # 第 25 格：没给 --library ⇒ 判定库是 <家目录>/library-by-conclusion-kind（按结论种类存的新库），不碰旧的 <家目录>/library（文件头「库与日志」）
  cells=$((cells + 1))
  printf '%s\n' '#!/usr/bin/env bash' 'echo "library=$2" >> "${STUB_CALLS:?}"' 'echo "CRASH_AMPLIFICATION mode=single_host_cpu states_checked_here=1 red=0" > "$3"' >"$work/library-run.sh"
  : > "$work/calls"
  local library_exit
  STUB_CALLS="$work/calls" SINGLEFS_CRASH_AMPLIFICATION_HOME="$work/home-default-library" CRASH_AMPLIFICATION_ITEM_SWITCHES=0,0 \
    CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/library-run.sh" CRASH_AMPLIFICATION_TWO_HOSTS_DRIVER="$work/two-hosts.sh" \
    bash "$0" some-target >"$work/default-library.out" 2>&1; library_exit=$?
  if [[ "$library_exit" != 0 || "$(tr '\n' ' ' <"$work/calls")" != "library=$work/home-default-library/library-by-conclusion-kind " ]] || [[ -e "$work/home-default-library/library" ]]; then
    selftest_cell_failed "没给 --library 时判定库应当是 <家目录>/library-by-conclusion-kind、不碰旧的 <家目录>/library（退 $library_exit，调用记录「$(tr '\n' ' ' <"$work/calls")」）" \
      "看 local_library 那一行与 crash-amplification-lib.sh 的 crash_amplification_default_library"
  fi
  if (( failures > 0 )); then
    echo "  ✗ --selftest：$cells 格里 $failures 格判错" >&2
    echo "     → 怎么办：照上面每一格给的出路改，再跑 $0 --selftest" >&2
    exit 1
  fi
  echo "  ✓ --selftest：双机开着两台跑、双机没开本机跑、没开双机却点名第二台的卡退 78、开了双机第二台没卡退 78、几条流累加、没有计划行认不出原因退 247、第二台量不出原样退 249、卡用不了退 78、算法判不了退 248、开了 GPU 没写卡退 78、本机那一趟卡用不了退 78、本机那一趟显存不够原样退 245 并说显存不够、带值的选项缺值退 2、运行条件读多机配置里的显卡先后（不读退役的私有显卡配置）、开了 GPU 没带 --quiesced 经清场包装清场再跑（清场回读不过退 78 不跑；跑完、失败与被打断都不复原）、排计划那一趟借不到显存一项都没跑到退 77（别的用例名的那一行退 247、那一趟退了非 0 原样退、第二台借不到显存原样退 77）、几条流里没借到的逐条报本次未跑而借到的照判、没设 SINGLEFS_MULTI_HOST_CONFIG 时多机配置按 git common-dir 找、多机配置判法认显卡的两个键而不认退役的 GPU_COMPUTATION_CLASSES、结论版本对不上与没编进 gpu 特性都退 78、没给 --library 用按结论种类存的新库，$cells 格都对"
  exit 0
}
if (( selftest )); then run_selftest; fi

[[ -n "$test_target" ]] || usage

# 只判不跑（自证用）：拿造好的日志与开关判在哪跑
if [[ -n "${CRASH_AMPLIFICATION_ITEM_DECIDE_ONLY:-}" ]]; then
  IFS=, read -r decide_gpu decide_across <<<"${CRASH_AMPLIFICATION_ITEM_SWITCHES:?只判不跑要给 CRASH_AMPLIFICATION_ITEM_SWITCHES=<gpu>,<双机>}"
  crash_amplification_cards_match_the_hosts "$decide_across" "${CRASH_AMPLIFICATION_ITEM_CARDS:-}" || exit 78
  refused_exit_code=""; decided_not_run_line=""
  decide_where_to_run "$CRASH_AMPLIFICATION_ITEM_DECIDE_ONLY" "$decide_across" 0; decision=$?
  case "$decision" in
    0|10) exit 0 ;;
    20) report_placement_not_run "$decided_not_run_line"; item_break_is not-run-exits-0 && exit 0; exit 77 ;;
    *) exit "${refused_exit_code:-247}" ;;
  esac
fi

# 开了 GPU 而调用方没清场：经清场包装带 --quiesced 重起自己，交回包装的退出码（文件头「清场」）
if (( ! quiesced )) && ! item_break_is gpu-without-quiesce; then
  item_gpu_switch="$(crash_amplification_configuration_value "$(crash_amplification_multi_host_configuration_file "$repository_root")" ENABLE_GPU)"
  if [[ -n "${CRASH_AMPLIFICATION_ITEM_SWITCHES:-}" ]]; then IFS=, read -r item_gpu_switch _ <<<"$CRASH_AMPLIFICATION_ITEM_SWITCHES"; fi
  if [[ "$item_gpu_switch" == 1 ]]; then
    item_reexec_arguments=("${item_original_arguments[@]}" --quiesced)
    if [[ "${PREFLIGHT_FORCE_GIVEN:-0}" == 1 ]]; then item_reexec_arguments+=(--force); fi
    echo "  · 开了 GPU、没带 --quiesced：经 scripts/gpu-quiesce-run.sh 清场，带 --quiesced 重起这一项，跑完不复原" >&2
    bash "$item_script_directory/gpu-quiesce-run.sh" -- bash "$0" "${item_reexec_arguments[@]}"
    exit $?
  fi
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
local_library="${library_directory:-$(crash_amplification_default_library "$crash_amplification_home")}"
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
  if exit_code_is_explained "$exit_code"; then
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
  echo "  ✗ 配置开了 GPU，多机配置里却没写 GPU_CARDS_BY_PRIORITY$( (( quiesced )) && echo "_WHEN_QUIESCED")（多机配置：$configuration_file）" >&2
  echo "  → 怎么办：修好设备，或改配置：照仓根 multi-host.env.example 在多机配置里写每台机器用哪几张卡、各多少显存；不用 GPU 就把多机配置的 ENABLE_GPU 改成 0" >&2
  exit 78
fi
crash_amplification_cards_match_the_hosts "$across_machines_switch" "$gpu_cards_by_priority" || exit 78
crash_amplification_run_on_this_host 0/1 "$local_library" "$run_directory/placement.log" SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1; placement_exit=$?
grep -h '^CRASH_AMPLIFICATION_PLACEMENT ' "$run_directory/placement.log" 2>/dev/null
decided_local_states_per_second=""; refused_exit_code=""; decided_not_run_line=""
decide_where_to_run "$run_directory/placement.log" "$across_machines_switch" "$placement_exit"; decision=$?
case "$decision" in
  0) run_on_this_host_alone; exit $? ;;
  20) # 一项都没跑到（文件头步骤 ②、退出码 77）：不跑、不交双机驱动；弄坏开关 not-run-exits-0 下照改前退 0
    report_placement_not_run "$decided_not_run_line"
    item_break_is not-run-exits-0 && exit 0
    exit 77 ;;
  10)
    two_hosts_arguments=("$test_target" "${forwarded_arguments[@]+"${forwarded_arguments[@]}"}" --label "$label-two-hosts")
    [[ -n "$decided_local_states_per_second" && "$decided_local_states_per_second" != 0 ]] && two_hosts_arguments+=(--local-states-per-second "$decided_local_states_per_second")
    bash "$two_hosts_driver" "${two_hosts_arguments[@]}"; two_hosts_exit=$?
    if [[ "$two_hosts_exit" == 249 ]] && item_break_is local-run-when-the-second-host-fails; then
      echo "CRASH_AMPLIFICATION_DECISION where=local reason=the-second-host-did-not-calibrate"
      run_on_this_host_alone
      exit $?
    fi
    if [[ "$two_hosts_exit" == 77 ]]; then
      # 第二台排计划那一趟借不到显存（文件头步骤 ③、退出码 77）：双机驱动打了 CRASH_AMPLIFICATION_NOT_RUN 那一行；本次未跑，不改在本机一台跑
      echo "  ! 本次未跑（这不是通过）：第二台排派活计划那一趟借不到显存，两台都没分份判（双机驱动退 77，见上面 CRASH_AMPLIFICATION_NOT_RUN 那一行）" >&2
      echo "  · 等第二台上占着卡的 GPU 用例或测试进程跑完，再重跑同一条命令（本机先判着的块在库里，接着判）" >&2
      item_break_is not-run-exits-0 && exit 0
      exit 77
    fi
    if [[ "$two_hosts_exit" != 0 ]]; then
      echo "  ✗ 两台跑那一趟退 $two_hosts_exit（78 第二台或卡用不上、248 算法上判不了、249 第二台没量出速度）：配置了两台就要两台都跑成，不改在本机一台跑" >&2
      if exit_code_is_explained "$two_hosts_exit"; then
        crash_amplification_explain_exit_code "$two_hosts_exit"
      else
        echo "  → 怎么办：看双机驱动日志目录（$(dirname "$local_library")/runs/$label-two-hosts/）里第二台那几份的 panicked、GPU_CARD_NOT_USABLE 与 run-with-memory-cap 行，修好第二台再重跑同一条命令" >&2
      fi
    fi
    exit "$two_hosts_exit" ;;
  *) exit "${refused_exit_code:-247}" ;;
esac
