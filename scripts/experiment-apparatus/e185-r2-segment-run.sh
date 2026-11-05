#!/usr/bin/env bash
# admission: always 每一趟都是一次计时，上一趟的产物不替这一趟作保
# run-condition: command cargo python3 git sha256sum findmnt lsblk getconf
# E185（GC一批搬迁的挂钟拆分）第 2 次重跑一段的跑法（跑前登记 research/prompts/e185-r2-prereg.md 第五节「跑法」）：
#   写输入清单 → 在这一趟的编译目录（CARGO_TARGET_DIR：派到第二台的，multi-host-run.sh 在第二台本地盘上这一趟新建、跑完删；本机那一支，脚本里的 cargo 不在命令位置上，服务不设，是树里的 target）里编一次装置（crates/singlefs-verification/src/bin/e185_gc_batch_wall_clock_split.rs，
#   不带 gpu 特性）→ 跑这一段 → 产物写进产物目录。三趟先后跑，每趟一条命令，主 agent 在本机仓根起：
#     task-server/task submit -- env E185_RUN_DATE=<日期> bash scripts/experiment-apparatus/e185-r2-segment-run.sh <段> --disk-model <盘的型号>
#   这个入口整条交统一任务服务起，派哪台由服务定（records/2026-10-31-task-server巡检.md 问题 94：不登只派一台的表）；两台在同一棵共享的树里跑
#   （仓经 NFS 两台挂在同一绝对路径），产物（名字见「产物」）直接写进这棵树的 research/results/，两台都读得到，不拷回、不取；
#   输入清单里这一台本地盘上的目录（镜像目录），主 agent 换成「〔第二台的目录〕」「〔本机的目录〕」再留在 research/results/。
# 点盘（真盘臂量的是哪块盘）：--disk-model <型号> 必写，照 lsblk -s 的 MODEL 一栏逐字比（镜像目录所在设备往上各层里有一层是这个型号才算在这块盘上）。
#   点到的盘这一台没有（镜像目录不在那块盘上）就退 78、一步都不编不跑，出路写「在有那块盘的那一台跑，或把 E185_IMAGE_DIRECTORY 指到那块盘上的目录」，
#   不悄悄换一块盘量；量哪块盘照跑前登记第五节（登记写的是第二台那块盘的型号，主 agent 照登记填）。
#
# 用法：
#   bash scripts/experiment-apparatus/e185-r2-segment-run.sh geometry-check --disk-model <型号>    单点核：改法甲与旧造法各一条 geometry-check（P = 1024 两个占比）
#   bash scripts/experiment-apparatus/e185-r2-segment-run.sh wall-clock --disk-model <型号>        第一段
#   bash scripts/experiment-apparatus/e185-r2-segment-run.sh stage-split --disk-model <型号>       第二段
#   bash scripts/experiment-apparatus/e185-r2-segment-run.sh --selftest    自证：只判点盘那一步（假 findmnt、假 lsblk 放在 PATH 最前面），不编、不跑装置
# 弄坏开关（只在 --selftest 里认，别处设了退 2）。逐项写开关名、弄坏哪一处，下一行起写打开后红的格（自证打「✗ e185-r2 自证：<格名>（…）」）：
#   E185_R2_BREAK=disk-unchecked  点盘不比型号、一律算在那块盘上
#       「e185-r2 自证：点到的盘这一台没有：退 78、点名型号与出路」
# 产物（<日期> 取 E185_RUN_DATE，没设取起跑当天 TZ=Asia/Tokyo date +%F）：
#   输入清单 e185-r2-inputs-<段>-<日期>.txt；单点核两份 e185-gc-batch-wall-clock-split-geometry-check-r2-<日期>.out、
#   e185-gc-batch-wall-clock-split-geometry-check-slot-order-r2-<日期>.out；第一、二段各一份 e185-gc-batch-wall-clock-split-<段>-r2-<日期>.out。
#   装置的标准错误各写同名加 .err。
# 环境：E185_PRODUCT_DIRECTORY（产物目录，默认 research/results）、E185_IMAGE_DIRECTORY（真盘臂的镜像目录，默认 <MULTI_HOST_RUN_PEER_SCRATCH>/e185-images：
#   派到第二台时 multi-host-run.sh 给的第二台本地这一趟的目录，不在 NFS 上、不在开机清空的目录里，跑完连目录删；没有这个变量取 ${CARGO_TARGET_DIR:-target}/e185-images）、E185_RUN_DATE。
#   镜像目录要落在 --disk-model 点名的那块盘上（上面「点盘」）。
# 上限：装置 E185_MEMORY_CAP=40G（装置自己读的内存预算）；外层不套内存包装（额度由任务服务给）；线程不设，装置取登记值 .claude/heavy-thread-default。
# 退出码：0 跑完；2 段名认不出、没写 --disk-model、弄坏开关设在自证之外；78 点到的盘这一台没有（上面「点盘」）；别的照 cargo 与装置自己的退出码（装置退 3 是停机条款，见登记第十一节）。
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
cd "$(dirname "${BASH_SOURCE[0]}")/../.." || { echo "进不去仓根"; echo "→ 怎么办：从仓根照文件头的用法起"; exit 2; }

e185_r2_inside_selftest=0
# 点盘（文件头「点盘」）：<镜像目录> <型号>；在那块盘上退 0，不在打原因与出路退 78
disk_check() {
  local directory=$1 wanted=$2 device models
  if [[ "${E185_R2_BREAK:-}" == disk-unchecked ]] && (( e185_r2_inside_selftest )); then return 0; fi
  device=$(findmnt -no SOURCE --target "$directory" 2>/dev/null) || device=""
  models=$( [[ -n "$device" ]] && lsblk -s -no MODEL "$device" 2>/dev/null | sed 's/^[[:space:]]*//; s/[[:space:]]*$//' | grep -v '^$' | tr '\n' '|' ) || models=""
  if [[ "|$models" == *"|$wanted|"* ]]; then return 0; fi
  echo "✗ 点到的盘这一台没有：要型号「$wanted」的盘，镜像目录 $directory 所在设备往上各层的型号是「${models%|}」（没读到算空）：这一趟一步都没编、没跑（退 78，运行条件不满足）"
  echo "→ 怎么办：在有那块盘的那一台跑（同一条命令重交，派哪台由服务定；这一台没有就会再退 78），或把 E185_IMAGE_DIRECTORY 指到那块盘上的目录；不换一块盘量"
  return 78
}

disk_selftest() {
  local work exit_code failures=0
  e185_r2_inside_selftest=1
  work=$(mktemp -d "${TMPDIR:-/tmp}/e185-r2-selftest.XXXXXX")
  trap 'rm -rf -- "$work"' RETURN
  mkdir -p "$work/bin" "$work/images"
  printf '%s\n' '#!/usr/bin/env bash' 'echo /dev/selftest-partition' > "$work/bin/findmnt"
  printf '%s\n' '#!/usr/bin/env bash' 'printf "%s\n" "" "  SELFTEST DISK 2TB  "' > "$work/bin/lsblk"
  chmod +x "$work/bin/findmnt" "$work/bin/lsblk"
  exit_code=0; PATH="$work/bin:$PATH" disk_check "$work/images" "SELFTEST DISK 2TB" > "$work/match.out" || exit_code=$?
  if (( exit_code == 0 )); then echo "  ✓ e185-r2 自证：点到的盘在这一台：退 0"; else echo "  ✗ e185-r2 自证：点到的盘在这一台：退 0（退 $exit_code：$(cat "$work/match.out")）"; failures=$((failures + 1)); fi  # gate-lint:detail
  exit_code=0; PATH="$work/bin:$PATH" disk_check "$work/images" "OTHER DISK 4TB" > "$work/mismatch.out" || exit_code=$?
  if (( exit_code == 78 )) && grep -q '要型号「OTHER DISK 4TB」' "$work/mismatch.out" && grep -q '在有那块盘的那一台跑' "$work/mismatch.out"; then
    echo "  ✓ e185-r2 自证：点到的盘这一台没有：退 78、点名型号与出路"
  else
    echo "  ✗ e185-r2 自证：点到的盘这一台没有：退 78、点名型号与出路（退 $exit_code：$(cat "$work/mismatch.out")）"; failures=$((failures + 1))  # gate-lint:detail
  fi
  echo "e185-r2-segment-run.sh 自证：2 格里 $failures 格红"
  (( failures == 0 )) || echo "  → 怎么办：照上面 ✗ 那一格括号里的输出看 disk_check 的判法与假 lsblk 的样子"
  (( failures == 0 ))
}

if [[ -n "${E185_R2_BREAK:-}" && "${1:-}" != --selftest ]]; then
  echo "✗ E185_R2_BREAK 只在 --selftest 里认（弄坏开关只给证红用），这一趟什么都没做"
  echo "→ 怎么办：去掉 E185_R2_BREAK 再跑；要证红就跑 E185_R2_BREAK=disk-unchecked bash scripts/experiment-apparatus/e185-r2-segment-run.sh --selftest"
  exit 2
fi
if [[ "${1:-}" == --selftest ]]; then
  disk_selftest
  exit $?
fi

registered_apparatus_sha256=a5b48b734f034114a6e2fe3a81f17ecd171dafba5e5b5ec1082e1b150fa5f082
segment=${1:-}
case "$segment" in
  geometry-check | wall-clock | stage-split) ;;
  *)
    echo "✗ 段名认不出：「$segment」"
    echo "→ 怎么办：第一个参数写 geometry-check、wall-clock、stage-split 之一（文件头的用法）"
    exit 2
    ;;
esac
if [[ "${2:-}" != --disk-model || -z "${3:-}" || $# -ne 3 ]]; then
  echo "✗ 没写要量的那块盘：段名后面要写 --disk-model <型号>（文件头「点盘」）"
  echo "→ 怎么办：照跑前登记第五节写那块盘的型号（lsblk -o MODEL 打的那一栏），例 bash scripts/experiment-apparatus/e185-r2-segment-run.sh $segment --disk-model <型号>"
  exit 2
fi
disk_model=$3
product_directory=${E185_PRODUCT_DIRECTORY:-research/results}
image_directory=${E185_IMAGE_DIRECTORY:-${MULTI_HOST_RUN_PEER_SCRATCH:-${CARGO_TARGET_DIR:-target}}/e185-images}
run_date=${E185_RUN_DATE:-$(TZ=Asia/Tokyo date +%F)}
mkdir -p "$product_directory" "$image_directory"
disk_exit=0; disk_check "$image_directory" "$disk_model" || disk_exit=$?
(( disk_exit == 0 )) || exit "$disk_exit"
eval "$(python3 scripts/hosts/rocksdb-prebuilt.py env)"

{
  sha256sum crates/singlefs-verification/src/bin/e185_gc_batch_wall_clock_split.rs \
    crates/singlefs-core/src/transaction.rs crates/singlefs-core/src/mounted_session.rs crates/singlefs-core/src/region_scan.rs \
    crates/singlefs-core/src/compaction_plan.rs crates/singlefs-core/src/unit.rs crates/singlefs-core/src/checksum.rs \
    crates/singlefs-core/src/block_device.rs crates/singlefs-core/src/allocator.rs crates/singlefs-format/src/lib.rs Cargo.lock
  echo "HEAD $(git rev-parse HEAD)（两台都在主工作区那棵共享的树里跑，同一棵树）"
  echo "点名的盘：型号 $disk_model"
  cargo tree -p singlefs-verification -e normal,build,features -i singlefs-core --offline
  image_device=$(findmnt -no SOURCE --target "$image_directory")
  echo "镜像目录所在盘（lsblk -s，NAME MODEL LOG-SEC）："
  lsblk -s -no NAME,MODEL,LOG-SEC "$image_device"
  echo "镜像目录所在文件系统：$(findmnt -no FSTYPE --target "$image_directory")"
  echo "nproc $(nproc)"
  echo "getconf CLK_TCK $(getconf CLK_TCK)"
  echo "上限：外层不套内存包装（额度由任务服务给）；E185_MEMORY_CAP=40G"
  echo "E185_REGISTERED_APPARATUS_SHA256=$registered_apparatus_sha256"
} > "$product_directory/e185-r2-inputs-$segment-$run_date.txt" 2>&1

cargo build --release -p singlefs-verification --bin e185_gc_batch_wall_clock_split
apparatus=${CARGO_TARGET_DIR:-target}/release/e185_gc_batch_wall_clock_split
product_stem=$product_directory/e185-gc-batch-wall-clock-split

run_apparatus() {
  local output=$1
  shift
  E185_REGISTERED_APPARATUS_SHA256=$registered_apparatus_sha256 E185_MEMORY_CAP=40G \
    "$apparatus" "$@" > "$output" 2> "$output.err"
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
