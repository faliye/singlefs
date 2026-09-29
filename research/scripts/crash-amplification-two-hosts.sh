#!/usr/bin/env bash
# admission: always 每次调都在这一刻的树上两台各跑一份、拷回、导入、对账，上一次的结论不替这一次作保
# run-condition: command cargo ssh rsync git
# 崩溃放量流水线的双机跑法（里程碑二「增补 4」第一、四、十一项；用户 2026-09-28 定四种跑法只由配置的 ENABLE_ACROSS_MACHINES、ENABLE_GPU 两个开关定）：
#
#   crash-amplification-two-hosts.sh <测试目标> [--test-filter <用例函数名>] [--library <本机库目录>] [--expand-up-to <N>] [--threads <N>] [--memory-cap <本机上限>] [--peer-memory-cap <第二台上限>] [--label <标签>]
#                                    [--host-weights <本机>,<第二台>] [--local-states-per-second <本机量到的速度>] [--quiesce] [--quiesced]
#   crash-amplification-two-hosts.sh --selftest      本机两个进程各领一份走同一条路（不碰第二台，不读配置）
#
# 测试目标是 crates/singlefs-checker-tier/tests/ 下一条走 crash_amplification::run_from_environment 的用例（例 crash_amplification_fixed_script_through_rollback）。
# 步骤：① 本机 SINGLEFS_CRASH_AMPLIFICATION_SHARE=0/2、库写到 --library（没给用 ${SINGLEFS_CRASH_AMPLIFICATION_HOME:-~/.local/share/singlefs/crash-amplification}/library，
#          在 /tmp 之外：重启不清、重跑同一条命令就是续跑；用例里的 run_from_environment 拒绝落在 /tmp、/var/tmp、/dev/shm、/run 下的库），后台跑（先 6 路并行只编、再按线程数跑）；
#       ② 第二台 SHARE=1/2、库写到那棵树副本树根下的 crash-amplification-library/（相对路径由测试按仓根解析），经 research/scripts/multi-host-run.sh --peer --fetch 跑并取回
#          （树怎么拷、内存上限、工具链比对、脱离 ssh 会话、私有字换成「第二台」都归它）；树副本不带 multi-host.env，
#          两个开关 ENABLE_ACROSS_MACHINES / ENABLE_GPU 随命令的环境送过去（测试在配置不在时读同名环境变量）；
#       ③ 等本机那一份；④ 本机再跑一趟 SHARE=0/2 带 SINGLEFS_CRASH_AMPLIFICATION_IMPORT=<取回的库>：导入第二台的块、对账、读违例——
#          用例在导入之后断言对账为空、违例为空，它退 0 这一趟才算全绿。② 失败（第二台被停、掉线、被杀）而库取回了一部分，也先导入再退：
#          第二台判到一半的块不丢，下一次同一条命令接着跑。--test-filter 只跑测试目标里点名的那一条用例（cargo test 的 --exact）。
# 跑法：配置 ENABLE_GPU=1 两台都带 gpu 特性编，各用显卡配置（GPU_CARDS_BY_PRIORITY）里点名给自己的卡、各在额度之内（用例里的 run_from_environment 做；
#       第二台的树副本不带配置，那个键与 SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE=peer 随命令的环境送过去）；不是 1 只带 verdict-store。
#       两台怎么分：--host-weights 给了照它；给了 --local-states-per-second（本机排派活计划时试判量到的速度）就先让第二台也排一趟计划、量它的速度，
#       按两个速度分；都没给就等分。第二台量速度的那段时间（它要拷树、从头编、编内核）本机不空等：本机先领整份判着，
#       第二台量完就叫它停（建停下文件，它判完手里那一块、写进库就收尾），再按速度分工——判过的块在库里，不重判。
#       这里默认不清场（卡在额度之内用）；全量由 crash-amplification.sh 开跑就清场、带 --quiesced 调到这里。单独要清场显式带 --quiesce：照配置里的清场命令先停、退出时复原
#       （peer-host-lib.sh 的 peer_host_quiesce / peer_host_restore）。--quiesced：调用方已经清过场、由它复原；这里只把显卡的先后与额度
#       换成清场之后那一份（显卡配置的 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED）。
# ENABLE_ACROSS_MACHINES 不是 1 由 multi-host-run.sh 拒（退 255），这里不偷偷退回单机。
# 输出：三趟各自的 CRASH_AMPLIFICATION 行原样打出（第二台那一趟经 multi-host-run.sh 已换掉私有的字）；日志放 <库目录的上层>/runs/<标签>/（--selftest 放 ${TMPDIR:-/tmp}/singlefs-crash-amplification/<标签>/）。
# 退出码：④ 那一趟 cargo test 的退出码；① 或 ② 失败退它们的退出码（先失败的那一个），一步都没跑退 2；250–254 是内存包装自己的结局；
#         249 给了 --local-states-per-second 而第二台没量出速度（打一行 CRASH_AMPLIFICATION_PEER_PLACEMENT_FAILED；本机先判着的块在库里，第二台一块都还没判）。
# 重型：跑 checker 档用例，只在用户要求时跑（SINGLEFS_HEAVY_TESTS=user-request 由这里带给三趟 cargo）。
#
# gate-similar: layer0-shard-run.sh 它按 SINGLEFS_LAYER0_SHARD 切片跑登记的崩溃枚举用例、写 54 号全绿标记，账本与 merge 是层 0 那一套；这里跑的是判定存储那一套（各写各的库、导入并成一个）
# gate-similar: multi-host-run.sh 它只管把一条命令送到第二台跑并取回产物，不知道分片；这里调它送第二台那一份，本机那一份与导入那一趟自己跑
# gate-similar: mutation-shard-run.sh 它按用时把变异表分两堆、两台各跑门禁 checker-tier-crates-mutation-replay；分的是变异行，不是崩溃状态块
# gate-similar: crash-amplification-item.sh 它排派活计划、定一个分项在本机一台跑还是动用第二台；动用第二台时调这里
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

two_hosts_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repository_root="$(cd "$two_hosts_script_directory/../.." && pwd)"
source "$two_hosts_script_directory/crash-amplification-lib.sh"

usage() {
  echo "  ✗ 用法：$0 <测试目标> [--test-filter <用例函数名>] [--expand-up-to <N>] [--threads <两台各用的核对线程数>] [--library <本机库目录，跨次复用>] [--memory-cap <本机上限>] [--peer-memory-cap <第二台上限>] [--label <标签>] [--host-weights <本机>,<第二台>] [--local-states-per-second <数>] [--quiesce]，或 --selftest" >&2
  echo "  → 怎么办：测试目标写 crates/singlefs-checker-tier/tests/ 下的文件名（不带 .rs），例 crash_amplification_fixed_script_through_rollback" >&2
  exit 2
}

# 本机上限默认按机器空着的量给（用户 2026-09-28：有 40G 就给 40G，不给 12G）——但要留在 slice 总上限（整机 − 余量 20G = 40.1G）之内：
# 给满 40G 时 slice 里残留的几百 MiB 就让它排不进队、等满 3600 秒退 252（2026-09-28 到 E 全域第一次起跑本机那一半就这样卡了一小时），所以给 38G
test_target=""; expand_up_to=""; memory_cap="38G"; peer_memory_cap=""; label="crash-amplification-$(date -u +%Y%m%dT%H%M%SZ)-$$"; selftest=0; threads=""; library_directory=""; test_filter=""; quiesce=0
host_weights=""; local_states_per_second=""; quiesced=0
while (($# > 0)); do
  case "$1" in
    --expand-up-to) expand_up_to="${2:?--expand-up-to 要一个数}"; shift 2 ;;
    --test-filter) test_filter="${2:?--test-filter 要一个用例函数名}"; shift 2 ;;
    --quiesce) quiesce=1; quiesced=1; shift ;;
    --quiesced) quiesced=1; shift ;;
    --host-weights) host_weights="${2:?--host-weights 要 <本机>,<第二台> 两个正整数}"; shift 2 ;;
    --local-states-per-second) local_states_per_second="${2:?--local-states-per-second 要一个正整数}"; shift 2 ;;
    --library) library_directory="${2:?--library 要一个目录}"; shift 2 ;;
    --threads) threads="${2:?--threads 要一个数}"; shift 2 ;;
    --memory-cap) memory_cap="${2:?--memory-cap 要一个上限}"; shift 2 ;;
    --peer-memory-cap) peer_memory_cap="${2:?--peer-memory-cap 要一个上限}"; shift 2 ;;
    --label) label="${2:?--label 要一个标签}"; shift 2 ;;
    --selftest) selftest=1; shift ;;
    -*) usage ;;
    *) if [[ -n "$test_target" ]]; then usage; fi; test_target="$1"; shift ;;
  esac
done

if [[ -n "$host_weights" ]] && ! [[ "$host_weights" =~ ^[1-9][0-9]*,[1-9][0-9]*$ ]]; then
  echo "  ✗ --host-weights 要写 <本机>,<第二台> 两个正整数，读到「$host_weights」" >&2
  echo "  → 怎么办：例 --host-weights 3,1（本机领四分之三）" >&2
  exit 2
fi
if [[ -n "$local_states_per_second" ]] && ! [[ "$local_states_per_second" =~ ^[1-9][0-9]*$ ]]; then
  echo "  ✗ --local-states-per-second 要一个正整数，读到「$local_states_per_second」" >&2
  echo "  → 怎么办：给本机排派活计划那一趟 CRASH_AMPLIFICATION_PLACEMENT 行里 states_per_second_on_this_host 的合计" >&2
  exit 2
fi
library_is_temporary="$selftest"
if [[ "$selftest" == 1 ]]; then test_target="${test_target:-crash_amplification_fixed_script_through_rollback}"; fi
crash_amplification_prepare || exit 2

# 库与日志的落点：/tmp 开机会清空（tmpfiles 的 D /tmp），2026-09-29 到 E 全域的库与树就丢在那里、续不上；正式跑一律放家目录下的固定位置，
# 重跑同一条命令就是续跑（同一判法版本下判过的块全部复用）。--selftest 的库只活这一趟，放 TMPDIR 并向用例写明是一次性的。
crash_amplification_home="${SINGLEFS_CRASH_AMPLIFICATION_HOME:-$HOME/.local/share/singlefs/crash-amplification}"
if [[ "$selftest" == 1 ]]; then
  run_directory="${TMPDIR:-/tmp}/singlefs-crash-amplification/$label"
  local_library="$run_directory/local-library"
else
  run_directory="$crash_amplification_home/runs/$label"
  local_library="$crash_amplification_home/library"
fi
mkdir -p "$run_directory" || { echo "  ✗ 建不了 $run_directory" >&2; echo "  → 怎么办：看 ${crash_amplification_home} 可不可写（--selftest 时看 TMPDIR）" >&2; exit 2; }
# --library：本机的库跨次复用（已核过的块下一次不再核；第二台开跑前拿到它的一份快照，同样复用）
[[ -n "$library_directory" ]] && local_library="$library_directory"

# 本机的一趟（crash-amplification-lib.sh 的 crash_amplification_run_on_this_host）。<share> <库目录> <日志> [IMPORT=<库…>]；两台的分工随 weights_environment 带
weights_environment=()
run_local_share() {
  local share="$1" library="$2" log="$3" import="${4:-}"
  local extra=("${weights_environment[@]+"${weights_environment[@]}"}")
  [[ -n "$import" ]] && extra+=("SINGLEFS_CRASH_AMPLIFICATION_IMPORT=$import")
  crash_amplification_run_on_this_host "$share" "$library" "$log" "${extra[@]+"${extra[@]}"}"
}

if [[ "$selftest" == 1 ]]; then
  # 本机两个进程各领一份（不碰第二台）：两个库、导入、对账。用例默认取固定脚本那条，小域。
  second_library="$run_directory/second-library"
  run_local_share 0/2 "$local_library" "$run_directory/share-0.log" &
  first_pid=$!
  run_local_share 1/2 "$second_library" "$run_directory/share-1.log"; second_exit=$?
  wait "$first_pid"; first_exit=$?
  if [[ "$first_exit" != 0 || "$second_exit" != 0 ]]; then
    echo "  ✗ --selftest：两份里有跑红的（0/2 退 $first_exit，1/2 退 $second_exit）；日志 $run_directory" >&2
    echo "  → 怎么办：看日志里 test result 与 panicked 那几行" >&2
    exit 1
  fi
  run_local_share 0/2 "$local_library" "$run_directory/import.log" "$second_library"; import_exit=$?
  grep -h '^CRASH_AMPLIFICATION ' "$run_directory/share-0.log" "$run_directory/share-1.log" "$run_directory/import.log"
  if [[ "$import_exit" != 0 ]] || ! grep -q 'unjudged_blocks=0 red=0' "$run_directory/import.log"; then
    echo "  ✗ --selftest：导入之后对账不为空或有红（退 $import_exit）；日志 $run_directory/import.log" >&2
    echo "  → 怎么办：看 import.log 里 CRASH_AMPLIFICATION 那一行的 unjudged_blocks 与 red" >&2
    exit 1
  fi
  echo "  ✓ --selftest：本机两份各录各核、导入之后对账为空、0 红（3 趟日志在 $run_directory）"
  exit 0
fi

[[ -n "$test_target" ]] || usage

# --quiesce：照配置里的清场命令先停、退出时复原（做法同 layer0-shard-run.sh；命令与输出不打，里面有服务名）。不带就不清场：卡在额度之内用。
if [[ "$quiesce" == 1 ]]; then
  source "$two_hosts_script_directory/peer-host-lib.sh"
  peer_host_load_configuration "$two_hosts_script_directory/layer0-shard-configuration-check.sh" "$repository_root" || { echo "  ✗ 多机配置判不过（上面是判法的原因）" >&2; echo "  → 怎么办：照判法打出的出路改 multi-host.env" >&2; exit 255; }
  if ! peer_host_quiesce; then
    echo "  ✗ 第二台清场没过（回读没通过）" >&2
    echo "  → 怎么办：看配置里 QUIESCE_STOP_COMMAND 与 QUIESCE_STOPPED_CHECK_COMMAND 在第二台上跑不跑得通" >&2
    exit 255
  fi
  trap 'peer_host_restore' EXIT
fi

# 第二台那一趟的环境：两个开关、它是第二台、显卡配置里的优先级（树副本不带配置）；三个内核一个一个编（驱动编大内核吃内存，一起编在第二台上撞过内存上限）；
# 编译目录共用树副本上一层的 crash-amplification-target：每一趟的树副本是新拷的，不共用就每趟从头编 rocksdb 等依赖（约 3 分钟）
peer_environment=("${common_environment[@]}" ENABLE_ACROSS_MACHINES=1 "ENABLE_GPU=$gpu_switch" SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE=peer SINGLEFS_GPU_KERNELS_ONE_AT_A_TIME=1 CARGO_TARGET_DIR=../crash-amplification-target)
[[ -n "$gpu_cards_by_priority" ]] && peer_environment+=("GPU_CARDS_BY_PRIORITY=$gpu_cards_by_priority")

# 给了本机的速度：第二台先排一趟派活计划、量它的速度，两台按速度分（四舍五入到整数，至少 1）
if [[ -z "$host_weights" && -n "$local_states_per_second" ]]; then
  # 第二台量速度的那段时间本机先判着（整份），量完叫它停
  warm_stop_file="$run_directory/stop-the-run-on-this-host"
  rm -f -- "$warm_stop_file"
  crash_amplification_run_on_this_host 0/1 "$local_library" "$run_directory/warm-local.log" "SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE=$warm_stop_file" &
  warm_job=$!
  placement_arguments=(--peer --label "$label-placement")
  [[ -n "$peer_memory_cap" ]] && placement_arguments+=(--memory-cap "$peer_memory_cap")
  bash "$two_hosts_script_directory/multi-host-run.sh" "${placement_arguments[@]}" -- \
    env "${peer_environment[@]}" SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1 SINGLEFS_CRASH_AMPLIFICATION_SHARE=0/1 SINGLEFS_CRASH_AMPLIFICATION_LIBRARY=crash-amplification-library \
    bash research/scripts/capped.sh 8 cargo "${cargo_test_arguments[@]}" >"$run_directory/peer-placement.log" 2>&1
  peer_placement_exit=$?
  : > "$warm_stop_file"
  wait "$warm_job"; warm_exit=$?
  rm -f -- "$warm_stop_file"
  grep -hE '^CRASH_AMPLIFICATION(_GPU_CARD|_GPU_TOTAL|_STOPPED)? ' "$run_directory/warm-local.log" 2>/dev/null
  if [[ "$warm_exit" != 0 ]]; then
    echo "  ✗ 本机先判着的那一趟退 $warm_exit；日志 $run_directory/warm-local.log" >&2
    echo "  → 怎么办：看日志里 test result、panicked 与 run-with-memory-cap 那几行；判过的块已经在库里，重跑同一条命令接着判" >&2
    exit "$warm_exit"
  fi
  grep -h '^CRASH_AMPLIFICATION_PLACEMENT ' "$run_directory/peer-placement.log" 2>/dev/null
  peer_states_per_second="$(crash_amplification_placement_rate "$run_directory/peer-placement.log")"
  if [[ "$peer_placement_exit" != 0 || -z "$peer_states_per_second" || "$peer_states_per_second" == 0 ]]; then
    echo "CRASH_AMPLIFICATION_PEER_PLACEMENT_FAILED exit=$peer_placement_exit"
    echo "  ✗ 第二台排派活计划那一趟没量出速度（退 $peer_placement_exit）；日志 $run_directory/peer-placement.log" >&2
    echo "  → 怎么办：看日志里 GPU_CARD_NOT_USABLE、panicked、run-with-memory-cap 与 multi-host-run.sh 那几行（250 是第二台那一趟撞了内存上限）；配置了两台就要两台都跑成，修好第二台再重跑同一条命令" >&2
    exit 249
  fi
  host_weights="$local_states_per_second,$peer_states_per_second"
fi
if [[ -n "$host_weights" ]]; then
  weights_environment=("SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS=$host_weights")
  echo "CRASH_AMPLIFICATION_HOST_WEIGHTS local,peer=$host_weights"
fi

peer_arguments=(--peer --label "$label" --fetch crash-amplification-library)
if [[ -f "$local_library/CURRENT" ]]; then
  # 库开着的时候不能拷（RocksDB 的文件在变）：本机那一份开跑之前先照一份快照，送给第二台当它的起点
  library_for_peer="$run_directory/library-for-peer"
  cp -a -- "$local_library" "$library_for_peer" || { echo "  ✗ 拷不了库的快照给第二台" >&2; echo "  → 怎么办：看 $run_directory 可不可写" >&2; exit 2; }
  peer_arguments+=(--send "$library_for_peer" crash-amplification-library)
fi
run_local_share 0/2 "$local_library" "$run_directory/share-0.log" &
local_pid=$!
[[ -n "$peer_memory_cap" ]] && peer_arguments+=(--memory-cap "$peer_memory_cap")
fetch_directory="$run_directory/fetched"
MULTI_HOST_RUN_FETCH_DIRECTORY="$fetch_directory" bash "$two_hosts_script_directory/multi-host-run.sh" "${peer_arguments[@]}" -- \
  env "${peer_environment[@]}" "${weights_environment[@]+"${weights_environment[@]}"}" SINGLEFS_CRASH_AMPLIFICATION_SHARE=1/2 SINGLEFS_CRASH_AMPLIFICATION_LIBRARY=crash-amplification-library \
  bash research/scripts/capped.sh 8 cargo "${cargo_test_arguments[@]}" >"$run_directory/share-1.log" 2>&1
peer_exit=$?
wait "$local_pid"; local_exit=$?
grep -hE '^CRASH_AMPLIFICATION(_GPU_CARD|_GPU_TOTAL)? ' "$run_directory/share-0.log" "$run_directory/share-1.log" 2>/dev/null
fetched_library="$fetch_directory/crash-amplification-library"
if [[ "$local_exit" != 0 ]]; then
  echo "  ✗ 本机那一份退 $local_exit；日志 $run_directory/share-0.log" >&2
  if [[ -f "$fetched_library/CURRENT" ]]; then
    # 第二台那一份判过的块已经取回来了：先导进本机库再退，不因为本机这边红了就把第二台判的丢掉；下一次同一条命令接着跑
    run_local_share 0/2 "$local_library" "$run_directory/import.log" "$fetched_library"; salvage_exit=$?
    grep -h '^CRASH_AMPLIFICATION ' "$run_directory/import.log" 2>/dev/null
    echo "  · 第二台取回的库已导入本机库（导入那一趟退 $salvage_exit）" >&2
  fi
  echo "  → 怎么办：看日志里 test result、panicked 与 run-with-memory-cap 那几行（250–254 是内存包装自己的结局）" >&2
  exit "$local_exit"
fi
if [[ "$peer_exit" != 0 ]]; then
  echo "  ✗ 第二台那一份退 $peer_exit；日志 $run_directory/share-1.log" >&2
  if [[ -f "$fetched_library/CURRENT" ]]; then
    # 第二台判到一半的库取回来了：先导进本机库再退，它判过的块不丢；下一次同一条命令接着跑
    run_local_share 0/2 "$local_library" "$run_directory/import.log" "$fetched_library"; salvage_exit=$?
    grep -h '^CRASH_AMPLIFICATION ' "$run_directory/import.log" 2>/dev/null
    echo "  · 第二台判到一半的库已导入本机库（导入那一趟退 $salvage_exit）：重跑同一条命令接着判" >&2
  fi
  echo "  → 怎么办：255 是 multi-host-run.sh 自己没跑成（配置、工具链、拷树），别的看日志里第二台的 test result 与 panicked" >&2
  exit "$peer_exit"
fi
if [[ ! -d "$fetched_library" ]]; then
  echo "  ✗ 第二台的库没取回：$fetched_library 不在" >&2
  echo "  → 怎么办：看 share-1.log 里 multi-host-run.sh --fetch 那几行" >&2
  exit 255
fi
run_local_share 0/2 "$local_library" "$run_directory/import.log" "$fetched_library"; import_exit=$?
grep -h '^CRASH_AMPLIFICATION ' "$run_directory/import.log"
if [[ "$import_exit" != 0 ]]; then
  echo "  ✗ 导入第二台的库之后那一趟退 $import_exit（对账不为空或有红）；日志 $run_directory/import.log" >&2
  echo "  → 怎么办：看 import.log 里 CRASH_AMPLIFICATION 那一行的 unjudged_blocks 与 red，与 panicked 那几行" >&2
  exit "$import_exit"
fi
echo "  ✓ 双机各录各核、第二台的库拷回导入、对账为空、0 红（3 趟日志在 $run_directory）"
