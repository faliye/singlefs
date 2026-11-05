#!/usr/bin/env bash
# admission: always 每一趟都是一次计时，上一趟的产物不替这一趟作保
# run-condition: command cargo python3 git sha256sum getconf nproc
# task-server-gpu-arguments: host-cpu-drift-control
# long-run-shards: 一段一条命令、一份产物：控漂移段（host-cpu-drift-control，本机两张卡）与写法与拆分段（cpu-writings，第二台）两段，两台上可以同时起；段内不再拆进几个进程：控漂移段的轮次比要同一个进程、同一条线程上前后几轮的比，写法与拆分段的计时窗口一条线程串行，准备队列在装置里自己按线程上限并行
# long-run-resume: 同一条命令重起就是续跑：这一段当天已有一份末行是 E7RESULT name=done 的产物就报出来、退 0、不重跑（设 E185_RUN_AGAIN=1 才另起一份带 -2、-3 的）；没有完整产物的整段重跑（段内不续，理由同分片那一行）
# long-run-hosts: 台数从 crash-point-replay-shard-configuration-check.sh --emit-hosts 读，每台线程取 .claude/heavy-thread-default 的登记值，不现算、不写常量
#   多机的读法：两段各交一条给服务，派哪台、清场与分卡由服务定（不自己连第二台；records/2026-10-31-task-server巡检.md 问题 94），跑在哪台写进输入清单；这一份把 --emit-hosts 读出的台数写进输入清单，
#   写法与拆分段只有一台（双机关着）时照样在本机跑、清单里写明；编译与装置的准备队列直接读服务交的线程变量（SINGLEFS_THREAD_CAP、CARGO_BUILD_JOBS），不写死线程数。
# E185（GC一批搬迁的挂钟拆分）第 4 次重跑一段的跑法（跑前登记 research/prompts/e185-r4-prereg.md 第五节「跑法」，第一节「主 agent 定」第 6 条撤了拆法乙）：
#   核装置 sha256 → 写输入清单 → 在这一趟自己的编译目录里编一次装置（控漂移段带 gpu 特性，写法与拆分段不带；编译目录经
#   scripts/hosts/run-build-directory.sh 建、跑完删）→ 跑那一段 → 产物写进产物目录。
# 准入与运行条件：
#   装置源码的 sha256 要等于登记第十二节写的那一个（下面 registered_apparatus_sha256）；不等就写完输入清单、退 3（停机 K-S6），不编、不跑。
#   输入清单里 `cargo tree` 那一段出现 lookup-work-counters（计数特性被并进来）就退 6，那一段整段作废（登记第十一节作废条款）。
#   控漂移段起卡：装置直接起，不套清场包装（任务服务对要卡的任务在最外层套清场；清场回读没过的由服务报，这一段记「本次未跑」）。
# 起法（主 agent 在仓根起；起之前核这几份脚本有没有变、两段能不能同时起，变了照新写法起、不能同时就先后起，记进登记第十二节一行）：
#   写法与拆分段（先起）：
#     task-server/task submit -- env E185_RUN_DATE=<日期> E185_MEMORY_CAP=40G bash scripts/experiment-apparatus/e185-r4-segment-run.sh cpu-writings
#     两台在同一棵共享的树里跑，这一段的四份产物（名字见「产物」）直接写进 research/results/，不取回；输入清单里这一台本地盘上的目录，主 agent 换成「〔第二台的目录〕」「〔本机的目录〕」再留在 research/results/。
#   控漂移段（上一段开跑之后再起；计时时那一台不起重活）：
#     task-server/task submit -- env E185_RUN_DATE=<日期> bash scripts/experiment-apparatus/e185-r4-segment-run.sh host-cpu-drift-control
#   重判段（本机，两段都交回之后；纯算术，不经这一份）：
#     task-server/task submit -- python3 -I scripts/experiment-apparatus/e185_r4_baseline_rejudge.py \
#       --host-cpu-product research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out \
#       --drift-control-product research/results/<控漂移段产物> --stage-split-product research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out \
#       --writings-product research/results/<写法与拆分段产物> --attack-ledger research/prompts/m4-gpugc-design-r1-opus-model/out/g1.out \
#       --writings-sha256 <它的 sha256> --drift-control-sha256 <它的 sha256> > research/results/e185-r4-baseline-rejudge-<日期>.out
#
# 用法：
#   bash scripts/experiment-apparatus/e185-r4-segment-run.sh host-cpu-drift-control   控漂移段（本机，两张卡）
#   bash scripts/experiment-apparatus/e185-r4-segment-run.sh cpu-writings             写法与拆分段（第二台，不起卡）
# 产物（<日期> 取 E185_RUN_DATE，没设取起跑当天 TZ=Asia/Tokyo date +%F；同一天同一段再跑时几份一起加 -2、-3……，不盖已有的）：
#   e185-gc-batch-wall-clock-split-<段>-r4-<日期>.out（装置的标准错误写同名加 .err）；输入清单 e185-r4-inputs-<段>-<日期>.txt；
# 环境：E185_PRODUCT_DIRECTORY（产物目录，默认 research/results）、E185_RUN_DATE、E185_RUN_AGAIN（设成 1 时当天已有完整产物也另起一份）、
#   E185_MEMORY_CAP（写法与拆分段装置的内存预算，装置按它定准备队列的并行条数，没设取 40G；外层的额度由任务服务给）。E185_THREADS 不设，装置取登记值。
# 退出码：0 跑完，或当天已有完整产物；2 段名认不出；3 装置源码的 sha256 与登记不等（停机 K-S6），或装置自己的停机条款；
#   6 计数特性被并进来（整段作废）；78 取卡没过、没编 gpu 特性、没有 sse4.2（本次未跑，装置退）；别的照 cargo 与装置自己的退出码（见装置文件头）。
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
cd "$(dirname "${BASH_SOURCE[0]}")/../.." || { echo "进不去仓根"; echo "→ 怎么办：从仓根照文件头的用法起"; exit 2; }

registered_apparatus_sha256=a5b48b734f034114a6e2fe3a81f17ecd171dafba5e5b5ec1082e1b150fa5f082
apparatus_source=crates/singlefs-verification/src/bin/e185_gc_batch_wall_clock_split.rs
attack_ledger=research/prompts/m4-gpugc-design-r1-opus-model/out/g1.out
stage_split_product=research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out
host_cpu_product=research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out
function_split_product=research/results/e185-gc-batch-wall-clock-split-function-split-r3-2026-10-24.out

refuse_an_unknown_segment_name() {
  echo "✗ 段名认不出：「$1」"
  echo "→ 怎么办：第一个参数写 host-cpu-drift-control 或 cpu-writings（文件头的用法）"
  exit 2
}

segment=${1:-}
case "$segment" in
  host-cpu-drift-control) build_features=(--features gpu) ;;
  cpu-writings) build_features=() ;;
  *) refuse_an_unknown_segment_name "$segment" ;;
esac
product_directory=${E185_PRODUCT_DIRECTORY:-research/results}
run_date=${E185_RUN_DATE:-$(TZ=Asia/Tokyo date +%F)}
mkdir -p "$product_directory"

# 续跑：当天已有一份末行是 name=done 的产物就不重跑（E185_RUN_AGAIN=1 另起一份）
# gate-lint:nocount 一次只跑一段装置、不扫一批对象，唯一的成功句报的是沿用的那一份产物
if [[ "${E185_RUN_AGAIN:-}" != 1 ]]; then
  for earlier in "$product_directory/e185-gc-batch-wall-clock-split-$segment-r4-$run_date"*.out; do
    [[ -f "$earlier" ]] || continue
    if [[ "$(tail -n 1 "$earlier")" == "E7RESULT name=done "* ]]; then
      echo "✓ 这一段当天已有完整产物：$earlier（末行 name=done），不重跑"
      echo "  要另起一份就设 E185_RUN_AGAIN=1 再起同一条命令（产物加 -2、-3）"
      exit 0
    fi
  done
fi

# 同一天同一段再跑：几份一起加 -2、-3……，取几份都还不在的头一个后缀
suffix=""
next_rerun=2
while [[ -e "$product_directory/e185-gc-batch-wall-clock-split-$segment-r4-$run_date$suffix.out" \
  || -e "$product_directory/e185-r4-inputs-$segment-$run_date$suffix.txt" ]]; do
  suffix="-$next_rerun"
  next_rerun=$((next_rerun + 1))
done
product="$product_directory/e185-gc-batch-wall-clock-split-$segment-r4-$run_date$suffix.out"
inputs="$product_directory/e185-r4-inputs-$segment-$run_date$suffix.txt"

eval "$(python3 scripts/hosts/rocksdb-prebuilt.py env)"
source scripts/hosts/run-build-directory.sh
run_directory=$(mktemp -d "${TMPDIR:-/tmp}/singlefs-e185-r4-$segment-XXXXXX")
finish_the_run() {
  run_build_directory_end || true
  rmdir "$run_directory" 2>/dev/null || true
}
trap finish_the_run EXIT
trap 'exit 143' TERM
trap 'exit 130' INT
run_build_directory_begin "$run_directory"

# 台数 = --emit-hosts 打的 MULTI_HOST_COUNT（双机关着、它不打或退非 0 时只有本机，记 1）；不抄主机名与目录
hosts_listed=$(bash scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-hosts 2>/dev/null \
  | sed -n 's/^MULTI_HOST_COUNT=\([0-9][0-9]*\)$/\1/p' || true)
hosts_listed=${hosts_listed:-1}
apparatus_sha256=$(sha256sum "$apparatus_source" | cut -d' ' -f1)
{
  echo "$apparatus_sha256  $apparatus_source"
  sha256sum crates/singlefs-verification/src/gpu_unit_checks.rs crates/singlefs-core/src/mounted_session.rs \
    crates/singlefs-core/src/region_scan.rs crates/singlefs-core/src/transaction.rs crates/singlefs-core/src/checksum.rs \
    crates/singlefs-core/src/unit.rs crates/singlefs-core/src/compaction_plan.rs crates/singlefs-core/src/records.rs Cargo.lock
  echo "Cargo.lock 里 wgpu 的版本：$(grep -A1 '^name = "wgpu"$' Cargo.lock | sed -n 's/^version = "\(.*\)"$/\1/p')"
  echo "HEAD $(git rev-parse HEAD 2>/dev/null || echo none)"
  case "$segment" in
    host-cpu-drift-control)
      nvidia-smi --query-gpu=index,name,driver_version,pcie.link.gen.max,pcie.link.width.max,memory.total --format=csv ;;
    cpu-writings) echo "写法与拆分段不起卡，不查 nvidia-smi" ;;
  esac
  sha256sum "$attack_ledger" "$stage_split_product" "$host_cpu_product" "$function_split_product"
  cargo tree -p singlefs-verification -e normal,build,features -i singlefs-core --offline "${build_features[@]}"
  echo "nproc $(nproc)"
  echo "getconf CLK_TCK $(getconf CLK_TCK)"
  echo "RUSTFLAGS=${RUSTFLAGS:-}（拆法乙撤了，这一趟两段都不带 -C force-frame-pointers=yes）"
  echo "CPU 型号：$(grep -m1 '^model name' /proc/cpuinfo | cut -d: -f2- | sed 's/^ //')"
  for flag in sse4_2 pclmulqdq; do
    if grep -qw "$flag" /proc/cpuinfo; then echo "CPU 标志 $flag 在"; else echo "CPU 标志 $flag 不在"; fi
  done
  echo "scaling_governor（cpu0）：$(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || echo 读不到)"
  echo "perf_event_paranoid：$(cat /proc/sys/kernel/perf_event_paranoid 2>/dev/null || echo 读不到)"
  echo "多机配置里的台数（--emit-hosts 的 MULTI_HOST_COUNT，不抄主机名）：$hosts_listed"
  echo "上限：外层不套内存包装（额度由任务服务给）；写法与拆分段装置内存预算 E185_MEMORY_CAP=${E185_MEMORY_CAP:-40G}；"
  echo "  编译与装置准备队列的线程读服务交的变量：SINGLEFS_THREAD_CAP=${SINGLEFS_THREAD_CAP:-没设}，CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-没设}"
  echo "E185_REGISTERED_APPARATUS_SHA256=$registered_apparatus_sha256"
} > "$inputs" 2>&1

if [[ "$apparatus_sha256" != "$registered_apparatus_sha256" ]]; then
  echo "✗ 装置源码的 sha256 是 $apparatus_sha256，登记第十二节写的是 $registered_apparatus_sha256（停机 K-S6）"
  echo "→ 怎么办：核装置是不是换过；换过就交主 agent 改登记第十二节与这一份的 registered_apparatus_sha256，不照跑"
  exit 3
fi
if grep -q 'lookup-work-counters' "$inputs"; then
  echo "✗ 输入清单的 cargo tree 里有 lookup-work-counters：计数特性被并进来，这一段整段作废（登记第十一节）"
  echo "→ 怎么办：查是谁把 singlefs-core 的 lookup-work-counters 特性并进了 验证包的这一次编译，去掉再起"
  exit 6
fi

cargo build --release -p singlefs-verification "${build_features[@]}" --bin e185_gc_batch_wall_clock_split
apparatus="$CARGO_TARGET_DIR/release/e185_gc_batch_wall_clock_split"

case "$segment" in
  host-cpu-drift-control)
    E185_REGISTERED_APPARATUS_SHA256=$registered_apparatus_sha256 \
      "$apparatus" host-cpu-drift-control --cards 0,1 --stage-split-product "$stage_split_product" --attack-ledger "$attack_ledger" \
      > "$product" 2> "$product.err" && status=0 || status=$?
    ;;
  cpu-writings)
    E185_REGISTERED_APPARATUS_SHA256=$registered_apparatus_sha256 E185_MEMORY_CAP=${E185_MEMORY_CAP:-40G} \
      "$apparatus" cpu-writings \
      --point 0.25:1024 --point 0.25:8192 --point 0.75:1024 --point 0.75:8192 > "$product" 2> "$product.err" && status=0 || status=$?
    ;;
esac
echo "E185 第 4 次重跑 $segment 段退出码 $status：产物 $product（输入清单 $inputs）"
exit "$status"
