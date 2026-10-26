#!/usr/bin/env bash
# admission: always 每一趟都是一次计时，上一趟的产物不替这一趟作保
# run-condition: command cargo python3 git sha256sum findmnt lsblk getconf
# E185（GC一批搬迁的挂钟拆分）第 2 次重跑一段的跑法（跑前登记 research/prompts/e185-r2-prereg.md 第五节「跑法」）：
#   写输入清单 → 在这棵树自己的编译目录里编一次装置（crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs，
#   不带 gpu 特性）→ 跑这一段 → 产物写进产物目录。三趟在第二台先后跑，每趟一条命令，主 agent 在本机仓根起：
#     bash scripts/crash/gpu-quiesce-run.sh -- bash scripts/hosts/multi-host-run.sh --peer --memory-cap 46G --label e185-r2-<段>-<日期> \
#       --fetch research/results/<产物名>… -- env E185_RUN_DATE=<日期> bash scripts/experiment-apparatus/e185-r2-segment-run.sh <段>
#   --fetch 逐个点名这一段的产物（名字见「产物」）；取回的输入清单里第二台的目录，主 agent 换成「〔第二台的目录〕」再放进 research/results/。
#
# 用法：
#   bash scripts/experiment-apparatus/e185-r2-segment-run.sh geometry-check    单点核：改法甲与旧造法各一条 geometry-check（P = 1024 两个占比）
#   bash scripts/experiment-apparatus/e185-r2-segment-run.sh wall-clock        第一段
#   bash scripts/experiment-apparatus/e185-r2-segment-run.sh stage-split       第二段
# 产物（<日期> 取 E185_RUN_DATE，没设取起跑当天 TZ=Asia/Tokyo date +%F）：
#   输入清单 e185-r2-inputs-<段>-<日期>.txt；单点核两份 e185-gc-batch-wall-clock-split-geometry-check-r2-<日期>.out、
#   e185-gc-batch-wall-clock-split-geometry-check-slot-order-r2-<日期>.out；第一、二段各一份 e185-gc-batch-wall-clock-split-<段>-r2-<日期>.out。
#   装置的标准错误各写同名加 .err。
# 环境：E185_PRODUCT_DIRECTORY（产物目录，默认 research/results）、E185_IMAGE_DIRECTORY（真盘臂的镜像目录，默认 target/e185-images：
#   在树里的编译目录下，随 multi-host-run.sh 第 ⑦ 步连树删，落在第二台那块盘的 ext4 上、不在开机清空的目录里）、E185_RUN_DATE。
# 上限：装置 E185_MEMORY_CAP=40G（外层 multi-host-run.sh --memory-cap 46G）；线程不设，装置取登记值 .claude/heavy-thread-default。
# 退出码：0 跑完；2 段名认不出；别的照 cargo 与装置自己的退出码（装置退 3 是停机条款，见登记第十一节）。
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
cd "$(dirname "${BASH_SOURCE[0]}")/../.." || { echo "进不去仓根"; echo "→ 怎么办：从仓根照文件头的用法起"; exit 2; }

registered_apparatus_sha256=0bd2274383f12e88beab3f96f354148698cfcd2043263b5d5709e32e1946784d
segment=${1:-}
case "$segment" in
  geometry-check | wall-clock | stage-split) ;;
  *)
    echo "✗ 段名认不出：「$segment」"
    echo "→ 怎么办：第一个参数写 geometry-check、wall-clock、stage-split 之一（文件头的用法）"
    exit 2
    ;;
esac
product_directory=${E185_PRODUCT_DIRECTORY:-research/results}
image_directory=${E185_IMAGE_DIRECTORY:-target/e185-images}
run_date=${E185_RUN_DATE:-$(TZ=Asia/Tokyo date +%F)}
mkdir -p "$product_directory" "$image_directory"
eval "$(python3 scripts/hosts/rocksdb-prebuilt.py env)"

{
  sha256sum crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs \
    crates/singlefs-core/src/transaction.rs crates/singlefs-core/src/mounted_session.rs crates/singlefs-core/src/region_scan.rs \
    crates/singlefs-core/src/compaction_plan.rs crates/singlefs-core/src/unit.rs crates/singlefs-core/src/checksum.rs \
    crates/singlefs-core/src/block_device.rs crates/singlefs-core/src/allocator.rs crates/singlefs-format/src/lib.rs Cargo.lock
  echo "HEAD $(git rev-parse HEAD)（scripts/hosts/multi-host-run.sh --peer 拷树之后 git init 提交的那一次；树拷自主工作区）"
  cargo tree -p singlefs-checker-tier -e normal,build,features -i singlefs-core --offline
  image_device=$(findmnt -no SOURCE --target "$image_directory")
  echo "镜像目录所在盘（lsblk -s，NAME MODEL LOG-SEC）："
  lsblk -s -no NAME,MODEL,LOG-SEC "$image_device"
  echo "镜像目录所在文件系统：$(findmnt -no FSTYPE --target "$image_directory")"
  echo "nproc $(nproc)"
  echo "getconf CLK_TCK $(getconf CLK_TCK)"
  echo "上限：外层 multi-host-run.sh --memory-cap 46G；E185_MEMORY_CAP=40G"
  echo "E185_REGISTERED_APPARATUS_SHA256=$registered_apparatus_sha256"
} > "$product_directory/e185-r2-inputs-$segment-$run_date.txt" 2>&1

bash scripts/hosts/capped.sh 31 cargo build --release -p singlefs-checker-tier --bin e185_gc_batch_wall_clock_split
apparatus=target/release/e185_gc_batch_wall_clock_split
product_stem=$product_directory/e185-gc-batch-wall-clock-split

run_apparatus() {
  local output=$1
  shift
  E185_REGISTERED_APPARATUS_SHA256=$registered_apparatus_sha256 E185_MEMORY_CAP=40G \
    bash scripts/hosts/capped.sh 31 "$apparatus" "$@" > "$output" 2> "$output.err"
}

case "$segment" in
  geometry-check)
    run_apparatus "$product_stem-geometry-check-r2-$run_date.out" geometry-check --point 0.25:1024 --point 0.75:1024
    run_apparatus "$product_stem-geometry-check-slot-order-r2-$run_date.out" geometry-check --point 0.25:1024 --point 0.75:1024 --overwrite-order slot-order
    ;;
  wall-clock)
    run_apparatus "$product_stem-wall-clock-r2-$run_date.out" wall-clock --image-directory "$image_directory"
    ;;
  stage-split)
    run_apparatus "$product_stem-stage-split-r2-$run_date.out" stage-split
    ;;
esac
echo "✓ E185 第 2 次重跑 $segment 段跑完：产物在 $product_directory（日期 $run_date）"
