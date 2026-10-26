#!/usr/bin/env bash
# admission: always 每一趟都是一次计时或一次计数，上一趟的产物不替这一趟作保
# run-condition: command cargo python3 git sha256sum getconf nproc
# E185（GC一批搬迁的挂钟拆分）第 3 次重跑一段的跑法（跑前登记 research/prompts/e185-r3-prereg.md 第五节「跑法」）：
#   写输入清单 → 在这一趟自己的编译目录里编一次装置（crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs；
#   第一、二段带 gpu 特性，第三段不带；编译目录经 scripts/hosts/run-build-directory.sh 建、跑完删）→ 跑那一段 → 产物写进产物目录。
# 准入与运行条件：
#   装置源码的 sha256 要等于登记第十二节写的那一个（下面 registered_apparatus_sha256）；不等就写完输入清单、退 3（停机 J-S7），不编、不跑。
#   输入清单里 `cargo tree` 那一段出现 lookup-work-counters（计数特性被并进来）就退 6，那一段整段作废（登记第十一节作废条款）。
#   第一、二段起卡：装置那一步经 scripts/crash/gpu-quiesce-run.sh 起（照多机配置清场、回读，跑完不复原）；清场回读没过它退 78，
#   这一段记「本次未跑」。第三段不起卡。
# 起法（主 agent 在仓根起）：
#   第一、二段在本机，先后起、不同时起：
#     bash scripts/hosts/run-with-memory-cap.sh 16G env E185_RUN_DATE=<日期> bash scripts/experiment-apparatus/e185-r3-segment-run.sh host-cpu-per-submission
#     跑完再起同一条换成 recompute-disagreement。
#   第三段（附带，在第二台；主 agent 定了要量才起）：
#     bash scripts/crash/gpu-quiesce-run.sh -- bash scripts/hosts/multi-host-run.sh --peer --memory-cap 46G --label e185-r3-function-split-<日期> \
#       --fetch research/results/<产物名>… -- env E185_RUN_DATE=<日期> bash scripts/experiment-apparatus/e185-r3-segment-run.sh function-split
#     --fetch 逐个点名这一段的产物（名字见「产物」）；取回的输入清单里第二台的目录，主 agent 换成「〔第二台的目录〕」再放进 research/results/。
#
# 用法：
#   bash scripts/experiment-apparatus/e185-r3-segment-run.sh host-cpu-per-submission   第一段（本机，两张卡）
#   bash scripts/experiment-apparatus/e185-r3-segment-run.sh recompute-disagreement    第二段（本机，两张卡）
#   bash scripts/experiment-apparatus/e185-r3-segment-run.sh function-split            第三段（附带，第二台，不起卡）
# 产物（<日期> 取 E185_RUN_DATE，没设取起跑当天 TZ=Asia/Tokyo date +%F；同一天同一段已有产物时三份一起加 -2、-3……，不盖已有的）：
#   e185-gc-batch-wall-clock-split-<段>-r3-<日期>.out（装置的标准错误写同名加 .err）；
#   输入清单 e185-r3-inputs-<段>-<日期>.txt；第一、二段另有 e185-r3-quiesce-<段>-<日期>.txt（清场包装打到标准错误的 GPU_QUIESCE 那几行）。
# 环境：E185_PRODUCT_DIRECTORY（产物目录，默认 research/results）、E185_RUN_DATE。线程不设，装置取登记值 .claude/heavy-thread-default（E185_THREADS
#   只在第二段造内容与 CPU 参照、第三段准备队列里用）；第三段装置内存上限 E185_MEMORY_CAP=40G（外层 multi-host-run.sh --memory-cap 46G）。
# 退出码：0 跑完；2 段名认不出；3 装置源码的 sha256 与登记不等（停机 J-S7），或装置自己的停机条款；6 计数特性被并进来（整段作废）；
#   78 清场或取卡没过（本次未跑）；别的照 cargo 与装置自己的退出码（见装置文件头）。
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
cd "$(dirname "${BASH_SOURCE[0]}")/../.." || { echo "进不去仓根"; echo "→ 怎么办：从仓根照文件头的用法起"; exit 2; }

registered_apparatus_sha256=4368c0e7a7870a09810fa5667173f9cf14625f4d64c4e0287ee10ec7ef80e6ea
apparatus_source=crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs
attack_ledger=research/prompts/m4-gpugc-design-r1-opus-model/out/g1.out
stage_split_product=research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out

refuse_an_unknown_segment_name() {
  echo "✗ 段名认不出：「$1」"
  echo "→ 怎么办：第一个参数写 host-cpu-per-submission、recompute-disagreement、function-split 之一（文件头的用法）"
  exit 2
}

segment=${1:-}
case "$segment" in
  host-cpu-per-submission | recompute-disagreement) build_features=(--features gpu) ;;
  function-split) build_features=() ;;
  *) refuse_an_unknown_segment_name "$segment" ;;
esac
product_directory=${E185_PRODUCT_DIRECTORY:-research/results}
run_date=${E185_RUN_DATE:-$(TZ=Asia/Tokyo date +%F)}
mkdir -p "$product_directory"

# 同一天同一段再跑：三份一起加 -2、-3……，取三份都还不在的头一个后缀
suffix=""
next_rerun=2
while [[ -e "$product_directory/e185-gc-batch-wall-clock-split-$segment-r3-$run_date$suffix.out" \
  || -e "$product_directory/e185-r3-inputs-$segment-$run_date$suffix.txt" \
  || -e "$product_directory/e185-r3-quiesce-$segment-$run_date$suffix.txt" ]]; do
  suffix="-$next_rerun"
  next_rerun=$((next_rerun + 1))
done
product="$product_directory/e185-gc-batch-wall-clock-split-$segment-r3-$run_date$suffix.out"
inputs="$product_directory/e185-r3-inputs-$segment-$run_date$suffix.txt"
quiesce="$product_directory/e185-r3-quiesce-$segment-$run_date$suffix.txt"

eval "$(python3 scripts/hosts/rocksdb-prebuilt.py env)"
source scripts/hosts/run-build-directory.sh
run_directory=$(mktemp -d "${TMPDIR:-/tmp}/singlefs-e185-r3-$segment-XXXXXX")
finish_the_run() {
  run_build_directory_end || true
  rmdir "$run_directory" 2>/dev/null || true
}
trap finish_the_run EXIT
trap 'exit 143' TERM
trap 'exit 130' INT
run_build_directory_begin "$run_directory"

apparatus_sha256=$(sha256sum "$apparatus_source" | cut -d' ' -f1)
{
  echo "$apparatus_sha256  $apparatus_source"
  sha256sum crates/singlefs-checker-tier/src/gpu_unit_checks.rs crates/singlefs-core/src/mounted_session.rs \
    crates/singlefs-core/src/region_scan.rs crates/singlefs-core/src/transaction.rs crates/singlefs-core/src/checksum.rs Cargo.lock
  echo "Cargo.lock 里 wgpu 的版本：$(grep -A1 '^name = "wgpu"$' Cargo.lock | sed -n 's/^version = "\(.*\)"$/\1/p')"
  echo "HEAD $(git rev-parse HEAD 2>/dev/null || echo none)"
  case "$segment" in
    host-cpu-per-submission | recompute-disagreement)
      nvidia-smi --query-gpu=index,name,driver_version,pcie.link.gen.max,pcie.link.width.max,memory.total --format=csv ;;
    function-split) echo "第三段不起卡，不查 nvidia-smi" ;;
  esac
  if [[ "$segment" == host-cpu-per-submission ]]; then
    sha256sum "$attack_ledger" "$stage_split_product"
  fi
  cargo tree -p singlefs-checker-tier -e normal,build,features -i singlefs-core --offline "${build_features[@]}"
  echo "nproc $(nproc)"
  echo "getconf CLK_TCK $(getconf CLK_TCK)"
  echo "上限：第一、二段外层 run-with-memory-cap.sh 16G；第三段外层 multi-host-run.sh --memory-cap 46G、E185_MEMORY_CAP=40G；编译线程 31，装置线程取登记值"
  echo "E185_REGISTERED_APPARATUS_SHA256=$registered_apparatus_sha256"
} > "$inputs" 2>&1

if [[ "$apparatus_sha256" != "$registered_apparatus_sha256" ]]; then
  echo "✗ 装置源码的 sha256 是 $apparatus_sha256，登记第十二节写的是 $registered_apparatus_sha256（停机 J-S7）"
  echo "→ 怎么办：核装置是不是换过；换过就交主 agent 改登记第十二节与这一份的 registered_apparatus_sha256，不照跑"
  exit 3
fi
if grep -q 'lookup-work-counters' "$inputs"; then
  echo "✗ 输入清单的 cargo tree 里有 lookup-work-counters：计数特性被并进来，这一段整段作废（登记第十一节）"
  echo "→ 怎么办：查是谁把 singlefs-core 的 lookup-work-counters 特性并进了 checker 档的这一次编译，去掉再起"
  exit 6
fi

bash scripts/hosts/capped.sh 31 cargo build --release -p singlefs-checker-tier "${build_features[@]}" --bin e185_gc_batch_wall_clock_split
apparatus="$CARGO_TARGET_DIR/release/e185_gc_batch_wall_clock_split"

case "$segment" in
  host-cpu-per-submission)
    E185_REGISTERED_APPARATUS_SHA256=$registered_apparatus_sha256 bash scripts/crash/gpu-quiesce-run.sh -- \
      "$apparatus" host-cpu-per-submission --cards 0,1 --stage-split-product "$stage_split_product" --attack-ledger "$attack_ledger" \
      > "$product" 2> "$product.err" && status=0 || status=$?
    grep '^GPU_QUIESCE' "$product.err" > "$quiesce" || true
    ;;
  recompute-disagreement)
    E185_REGISTERED_APPARATUS_SHA256=$registered_apparatus_sha256 bash scripts/crash/gpu-quiesce-run.sh -- \
      "$apparatus" recompute-disagreement --cards 0,1 > "$product" 2> "$product.err" && status=0 || status=$?
    grep '^GPU_QUIESCE' "$product.err" > "$quiesce" || true
    ;;
  function-split)
    E185_REGISTERED_APPARATUS_SHA256=$registered_apparatus_sha256 E185_MEMORY_CAP=40G "$apparatus" function-split \
      --point 0.25:1024 --point 0.25:8192 --point 0.75:1024 --point 0.75:8192 > "$product" 2> "$product.err" && status=0 || status=$?
    ;;
esac
echo "E185 第 3 次重跑 $segment 段退出码 $status：产物 $product（输入清单 $inputs）"
exit "$status"
