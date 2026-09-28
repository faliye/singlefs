#!/usr/bin/env bash
# admission: always 每次调都在这一刻的树上两台各跑一份、拷回、导入、对账，上一次的结论不替这一次作保
# run-condition: command cargo ssh rsync git
# 崩溃放量流水线的双机跑法（里程碑二「增补 4」第一、四、十一项；用户 2026-09-28 定四种跑法只由配置的 ENABLE_ACROSS_MACHINES、ENABLE_GPU 两个开关定）：
#
#   crash-amplification-two-hosts.sh <测试目标> [--expand-up-to <N>] [--memory-cap <本机上限>] [--peer-memory-cap <第二台上限>] [--label <标签>]
#   crash-amplification-two-hosts.sh --selftest      本机两个进程各领一份走同一条路（不碰第二台，不读配置）
#
# 测试目标是 crates/singlefs-checker-tier/tests/ 下一条走 crash_amplification::run_from_environment 的用例（例 crash_amplification_fixed_script_through_rollback）。
# 步骤：① 本机 SINGLEFS_CRASH_AMPLIFICATION_SHARE=0/2、库写到本机临时目录，后台跑（先 6 路并行只编、再按线程数跑）；
#       ② 第二台 SHARE=1/2、库写到那棵树副本树根下的 crash-amplification-library/（相对路径由测试按仓根解析），经 research/scripts/multi-host-run.sh --peer --fetch 跑并取回
#          （树怎么拷、内存上限、工具链比对、脱离 ssh 会话、私有字换成「第二台」都归它）；树副本不带 multi-host.env，
#          两个开关 ENABLE_ACROSS_MACHINES / ENABLE_GPU 随命令的环境送过去（测试在配置不在时读同名环境变量）；
#       ③ 等本机那一份；④ 本机再跑一趟 SHARE=0/2 带 SINGLEFS_CRASH_AMPLIFICATION_IMPORT=<取回的库>：导入第二台的块、对账、读违例——
#          用例在导入之后断言对账为空、违例为空，它退 0 这一趟才算全绿。
# 跑法：配置 ENABLE_GPU=1 两台都带 gpu 特性编、各按本机能用的卡的空闲显存再分一层（用例里的 run_from_environment 做），并照配置里的清场命令
#       先停第二台的本地服务、退出时复原（peer-host-lib.sh 的 peer_host_quiesce / peer_host_restore）；不是 1 只带 verdict-store、不清场。
# ENABLE_ACROSS_MACHINES 不是 1 由 multi-host-run.sh 拒（退 255），这里不偷偷退回单机。
# 输出：三趟各自的 CRASH_AMPLIFICATION 行原样打出（第二台那一趟经 multi-host-run.sh 已换掉私有的字）；日志放 ${TMPDIR:-/tmp}/singlefs-crash-amplification/<标签>/。
# 退出码：④ 那一趟 cargo test 的退出码；① 或 ② 失败退它们的退出码（先失败的那一个），一步都没跑退 2；250–254 是内存包装自己的结局。
# 重型：跑 checker 档用例，只在用户要求时跑（SINGLEFS_HEAVY_TESTS=user-request 由这里带给三趟 cargo）。
#
# gate-similar: layer0-shard-run.sh 它按 SINGLEFS_LAYER0_SHARD 切片跑登记的崩溃枚举用例、写 54 号全绿标记，账本与 merge 是层 0 那一套；这里跑的是判定存储那一套（各写各的库、导入并成一个）
# gate-similar: multi-host-run.sh 它只管把一条命令送到第二台跑并取回产物，不知道分片；这里调它送第二台那一份，本机那一份与导入那一趟自己跑
# gate-similar: mutation-shard-run.sh 它按用时把变异表分两堆、两台各跑门禁 checker-tier-crates-mutation-replay；分的是变异行，不是崩溃状态块
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

two_hosts_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repository_root="$(cd "$two_hosts_script_directory/../.." && pwd)"
BINDGEN_INCLUDE_ARGUMENTS="-I/usr/lib/gcc/x86_64-linux-gnu/13/include -I/usr/lib/gcc/x86_64-linux-gnu/14/include"  # librocksdb-sys 的 bindgen 找 stdbool.h；哪个版本的目录在就用哪个，不在的 -I 无害

usage() {
  echo "  ✗ 用法：$0 <测试目标> [--expand-up-to <N>] [--threads <两台各用的核对线程数>] [--library <本机库目录，跨次复用>] [--memory-cap <本机上限>] [--peer-memory-cap <第二台上限>] [--label <标签>]，或 --selftest" >&2
  echo "  → 怎么办：测试目标写 crates/singlefs-checker-tier/tests/ 下的文件名（不带 .rs），例 crash_amplification_fixed_script_through_rollback" >&2
  exit 2
}

# 本机上限默认按机器空着的量给（用户 2026-09-28：有 40G 就给 40G，不给 12G）——但要留在 slice 总上限（整机 − 余量 20G = 40.1G）之内：
# 给满 40G 时 slice 里残留的几百 MiB 就让它排不进队、等满 3600 秒退 252（2026-09-28 到 E 全域第一次起跑本机那一半就这样卡了一小时），所以给 38G
test_target=""; expand_up_to=""; memory_cap="38G"; peer_memory_cap=""; label="crash-amplification-$(date -u +%Y%m%dT%H%M%SZ)-$$"; selftest=0; threads=""; library_directory=""
while (($# > 0)); do
  case "$1" in
    --expand-up-to) expand_up_to="${2:?--expand-up-to 要一个数}"; shift 2 ;;
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

configuration_file="${SINGLEFS_MULTI_HOST_CONFIG:-$repository_root/multi-host.env}"
features="verdict-store"
gpu_switch=0
if [[ "$selftest" == 0 ]] && grep -qE '^ENABLE_GPU=("?)1\1\s*$' "$configuration_file" 2>/dev/null; then features="verdict-store,gpu"; gpu_switch=1; fi

run_directory="${TMPDIR:-/tmp}/singlefs-crash-amplification/$label"
mkdir -p "$run_directory" || { echo "  ✗ 建不了 $run_directory" >&2; echo "  → 怎么办：看 TMPDIR 可不可写" >&2; exit 2; }
local_library="$run_directory/local-library"
# --library：本机的库跨次复用（已核过的块下一次不再核；第二台开跑前拿到它的一份快照，同样复用），没给就每次新建、只活这一趟
[[ -n "$library_directory" ]] && local_library="$library_directory"
common_environment=(SINGLEFS_HEAVY_TESTS=user-request "BINDGEN_EXTRA_CLANG_ARGS=$BINDGEN_INCLUDE_ARGUMENTS")
[[ -n "$expand_up_to" ]] && common_environment+=("SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=$expand_up_to")
[[ -n "$threads" ]] && common_environment+=("SINGLEFS_CRASH_AMPLIFICATION_THREADS=$threads")  # 两台都按这个数起核对线程（用户 2026-09-28 定全域两台各 31）

if [[ "$selftest" == 1 ]]; then test_target="${test_target:-crash_amplification_fixed_script_through_rollback}"; fi
cargo_test_arguments=(test --release -p singlefs-checker-tier --features "$features" --test "$test_target" -- --nocapture)

# 本机的一趟：先 6 路并行只编（librocksdb-sys 的 C++ 按并行度吃内存），再按本机核数跑。<share> <库目录> <日志> [IMPORT=<库…>]
run_local_share() {
  local share="$1" library="$2" log="$3" import="${4:-}"
  local extra=()
  [[ -n "$import" ]] && extra+=("SINGLEFS_CRASH_AMPLIFICATION_IMPORT=$import")
  ( cd "$repository_root" && env "${common_environment[@]}" "${extra[@]+"${extra[@]}"}" "SINGLEFS_CRASH_AMPLIFICATION_SHARE=$share" "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY=$library" \
      bash research/scripts/run-with-memory-cap.sh "$memory_cap" bash research/scripts/capped.sh 6 cargo "${cargo_test_arguments[@]:0:${#cargo_test_arguments[@]}-2}" --no-run \
    && env "${common_environment[@]}" "${extra[@]+"${extra[@]}"}" "SINGLEFS_CRASH_AMPLIFICATION_SHARE=$share" "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY=$library" \
      bash research/scripts/run-with-memory-cap.sh "$memory_cap" bash research/scripts/capped.sh "$(nproc)" cargo "${cargo_test_arguments[@]}" ) >"$log" 2>&1
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

# GPU 开着：第二台的卡被本地服务占着时空闲显存不够，照配置里的清场命令先停、退出时复原（做法同 layer0-shard-run.sh；命令与输出不打，里面有服务名）。
if [[ "$features" == *gpu* ]]; then
  source "$two_hosts_script_directory/peer-host-lib.sh"
  peer_host_load_configuration "$two_hosts_script_directory/layer0-shard-configuration-check.sh" "$repository_root" || { echo "  ✗ 多机配置判不过（上面是判法的原因）" >&2; echo "  → 怎么办：照判法打出的出路改 multi-host.env" >&2; exit 255; }
  if ! peer_host_quiesce; then
    echo "  ✗ 第二台清场没过（回读没通过）" >&2
    echo "  → 怎么办：看配置里 QUIESCE_STOP_COMMAND 与 QUIESCE_STOPPED_CHECK_COMMAND 在第二台上跑不跑得通" >&2
    exit 255
  fi
  trap 'peer_host_restore' EXIT
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
  env "${common_environment[@]}" ENABLE_ACROSS_MACHINES=1 "ENABLE_GPU=$gpu_switch" SINGLEFS_CRASH_AMPLIFICATION_SHARE=1/2 SINGLEFS_CRASH_AMPLIFICATION_LIBRARY=crash-amplification-library \
  bash research/scripts/capped.sh 8 cargo "${cargo_test_arguments[@]}" >"$run_directory/share-1.log" 2>&1
peer_exit=$?
wait "$local_pid"; local_exit=$?
grep -h '^CRASH_AMPLIFICATION ' "$run_directory/share-0.log" "$run_directory/share-1.log" 2>/dev/null
if [[ "$local_exit" != 0 ]]; then
  echo "  ✗ 本机那一份退 $local_exit；日志 $run_directory/share-0.log" >&2
  echo "  → 怎么办：看日志里 test result、panicked 与 run-with-memory-cap 那几行（250–254 是内存包装自己的结局）" >&2
  exit "$local_exit"
fi
if [[ "$peer_exit" != 0 ]]; then
  echo "  ✗ 第二台那一份退 $peer_exit；日志 $run_directory/share-1.log" >&2
  echo "  → 怎么办：255 是 multi-host-run.sh 自己没跑成（配置、工具链、拷树），别的看日志里第二台的 test result 与 panicked" >&2
  exit "$peer_exit"
fi
fetched_library="$fetch_directory/crash-amplification-library"
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
