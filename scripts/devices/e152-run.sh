#!/usr/bin/env bash
# E152（按里程碑对比六家文件系统的文件性能）：八个配置轮流跑 N 轮，每轮一个新虚机、一块新盘。
#
#   task-server/task submit -- bash scripts/devices/e152-run.sh <产物路径>                  不指定范围：八个配置、五轮，全部跑
#   task-server/task submit -- env E152_ROUNDS=1 E152_CONFIGURATIONS="ext4" bash scripts/devices/e152-run.sh <产物路径>   # 指定范围加缩域
#   bash scripts/devices/e152-run.sh --selftest                                              自证：缩过的那一趟产物头写「缩过、不当 E152 的产物」（不起虚机、不编译）
# E152_CONFIGURATIONS 当指定范围（点名哪几个配置），E152_ROUNDS 当缩域（少于登记的 5 轮）；设了与登记不同的值，产物头 E152RUN_HEADER 接
# reduced=yes registered_rounds=… registered_configurations=… not_e152_product=yes，这一份不当 E152 的产物。
# 弄坏开关 E152_RUN_BREAK=reduced-unmarked（只在 --selftest 起的那一趟里认，别处设了拒、退 2）：缩过的一趟产物头不接缩过那一段
#   打开后红的格：「缩过规模：轮数少于登记，产物头写缩过、不当 E152 的产物」「缩过规模：只点名一个配置，产物头写缩过」
#
# 不嵌套、经 task-server/task submit 交：三条 cargo build 直接起，编译并行度读服务交的 CARGO_BUILD_JOBS（每台默认重活线程数），
# 内存上限由服务给；脚本里不套 capped.sh、run-with-memory-cap.sh，也不写线程数。
#
# 口径、判据与作废条款都在跑前登记 research/prompts/e152-preregistration.md；这里只照着跑、照着记。
# 纪律（.claude/singlefs-ai-sop/rules/command-safety.md）：
#   - 本轮输出先写 <产物>.partial，全部跑完、汇总行追加完才改名成产物：中途失败不留一份看着像产物的文件；
#   - 每一次虚机跑的退出码都记进产物（E152RUN 行的 vm_exit），不吞；退出码非 0 的那一次同参数重跑一次
#     （跑前登记第八节第 1 条），两次都记；
#   - 八个配置按轮交错跑（第 1 轮八个都跑完再跑第 2 轮），宿主上的漂移平摊到每一家（跑前登记第七节）。
set -uo pipefail
RESEARCH="$(cd "$(dirname "$0")/../experiment-apparatus" && pwd)"
REPOSITORY="$(cd "$RESEARCH/../.." && pwd)"
REGISTERED_ROUNDS=5
REGISTERED_CONFIGURATIONS="raw ext4 xfs f2fs btrfs bcachefs zfs singlefs"

# 与登记不同的轮数或配置就是缩过的一趟：产物头接这一段，不当 E152 的产物；都是登记值时不接。
reduced_note() { # <轮数> <配置，空格隔开>
  if [[ "$1" == "$REGISTERED_ROUNDS" && "$2" == "$REGISTERED_CONFIGURATIONS" ]]; then return 0; fi
  if [[ "${E152_RUN_BREAK:-}" == reduced-unmarked ]]; then return 0; fi
  echo " reduced=yes registered_rounds=$REGISTERED_ROUNDS registered_configurations=${REGISTERED_CONFIGURATIONS// /,} not_e152_product=yes"
}

if [[ "${1:-}" == --selftest ]]; then
  failures=0
  expect() { # <名字> <条件>
    if eval "$2"; then echo "  ✓ $1"; else echo "  ✗ $1：条件「$2」不成立"; echo "     → 怎么办：改 reduced_note，再单独起 bash scripts/devices/e152-run.sh --selftest 到它退 0"; failures=$((failures + 1)); fi
  }
  expect "登记规模：产物头不接缩过那一段" '[[ -z "$(reduced_note "$REGISTERED_ROUNDS" "$REGISTERED_CONFIGURATIONS")" ]]'
  expect "缩过规模：轮数少于登记，产物头写缩过、不当 E152 的产物" '[[ "$(reduced_note 1 "$REGISTERED_CONFIGURATIONS")" == " reduced=yes registered_rounds=5 registered_configurations=raw,ext4,xfs,f2fs,btrfs,bcachefs,zfs,singlefs not_e152_product=yes" ]]'
  expect "缩过规模：只点名一个配置，产物头写缩过" '[[ "$(reduced_note "$REGISTERED_ROUNDS" ext4)" == *"reduced=yes"*"not_e152_product=yes" ]]'
  if ((failures > 0)); then echo "e152-run.sh 自证：$failures 格红"; exit 1; fi
  echo "e152-run.sh 自证：3 格全绿"
  exit 0
fi
if [[ -n "${E152_RUN_BREAK:-}" ]]; then echo "  ✗ E152_RUN_BREAK 只给 --selftest 证红用，这一趟不是自证" >&2; echo "    → 平常跑时不设它" >&2; exit 2; fi

OUTPUT="${1:?用法：e152-run.sh <产物路径>}"
ROUNDS="${E152_ROUNDS:-$REGISTERED_ROUNDS}"
CONFIGURATIONS="${E152_CONFIGURATIONS:-$REGISTERED_CONFIGURATIONS}"
RELEASE="${E152_KERNEL_RELEASE:-$(uname -r)}"

fail() { echo "  ✗ $1" >&2; echo "    → $2" >&2; exit 1; }

case "$OUTPUT" in /*) ;; *) OUTPUT="$PWD/$OUTPUT" ;; esac
cd "$RESEARCH" || fail "进不了 $RESEARCH" "从仓里跑"
KERNEL="$(bash ../devices/vm-kernel.sh --release "$RELEASE")" \
  || fail "拿不到 $RELEASE 的内核镜像" "单跑 bash scripts/devices/vm-kernel.sh --release $RELEASE 看报错"
cargo build --release --target x86_64-unknown-linux-musl --bin e152-file-system-benchmark >/dev/null 2>&1 \
  || fail "来宾二进制编不过" "cd scripts/experiment-apparatus && cargo build --release --target x86_64-unknown-linux-musl --bin e152-file-system-benchmark"
cargo build --release --bin e152-file-system-benchmark >/dev/null 2>&1 \
  || fail "宿主上的汇总二进制编不过" "cd scripts/experiment-apparatus && cargo build --release --bin e152-file-system-benchmark"
# singlefs 的真设备二进制与门禁 qemu-device-streams 同一种编法：glibc 目标加 +crt-static（验证包默认编 RocksDB 的 C++，本机没有 musl 的 C/C++ 编译器）
( cd "$REPOSITORY" && cargo build --release --target x86_64-unknown-linux-gnu --config 'target.x86_64-unknown-linux-gnu.rustflags=["-C","target-feature=+crt-static"]' -p singlefs-verification --bin new_pool_file_creation_on_device >/dev/null 2>&1 ) \
  || fail "singlefs 的真设备二进制编不过" "在仓根跑 cargo build --release --target x86_64-unknown-linux-gnu --config 'target.x86_64-unknown-linux-gnu.rustflags=[\"-C\",\"target-feature=+crt-static\"]' -p singlefs-verification --bin new_pool_file_creation_on_device"
GUEST_BINARY="$RESEARCH/target/x86_64-unknown-linux-musl/release/e152-file-system-benchmark"
HOST_BINARY="$RESEARCH/target/release/e152-file-system-benchmark"
SINGLEFS_BINARY="$REPOSITORY/target/x86_64-unknown-linux-gnu/release/new_pool_file_creation_on_device"

WORK="$(mktemp -d "${TMPDIR:-/tmp}/singlefs-e152.XXXXXX")"
trap 'rm -rf "${WORK:?}"' EXIT
# 起跑时把要调的脚本复制一份，之后只调副本：bash 边读边执行脚本，仓里的原件中途被别的会话一改，
# 正在跑的那一次就会读到错位的内容（2026-09-15 第二次正式跑实测：vm-bench.sh 开头注释被改，那一次报语法错退出）。
mkdir -p "$WORK/scripts/devices"
cp "$REPOSITORY/scripts/devices/vm-bench.sh" "$REPOSITORY/scripts/devices/e152-stage-root.sh" "$WORK/scripts/devices/" || fail "复制不了测试台脚本" "看 $WORK 的权限"
bash "$WORK/scripts/devices/e152-stage-root.sh" "$WORK/root" "$RELEASE" "$SINGLEFS_BINARY" \
  || fail "附加根没建成" "单跑 bash scripts/devices/e152-stage-root.sh <新的空目录> $RELEASE $SINGLEFS_BINARY"

PARTIAL="$OUTPUT.partial"
[[ ! -d "$OUTPUT" ]] || fail "产物路径是目录：$OUTPUT" "指定一个文件路径"
rm -f -- "$PARTIAL" || fail "无法清理 $PARTIAL" "检查产物目录权限"   # 成功发布前保留上一份产物
echo "E152RUN_HEADER kernel=$RELEASE rounds=$ROUNDS configurations=${CONFIGURATIONS// /,} started_jst=$(TZ=Asia/Tokyo date '+%Y-%m-%dT%H:%M')$(reduced_note "$ROUNDS" "$CONFIGURATIONS")" > "$PARTIAL" || fail "写不了 $PARTIAL" "检查父目录是否存在、空间及权限"
for ((round = 1; round <= ROUNDS; round++)); do
  for configuration in $CONFIGURATIONS; do
    case "$configuration" in
      singlefs) disks=2 ;;   # D2（RAID 条带策略） 已定项 9：第一版跑 2 块盘
      raw-md | ext4-md | xfs-md | f2fs-md | btrfs-raid1 | bcachefs-replicas2 | zfs-mirror) disks=2 ;;   # 跑前登记第十一节：两盘镜像
      *) disks=1 ;;
    esac
    for attempt in 1 2; do
      host_load="$(cut -d' ' -f1 /proc/loadavg)"
      run_output="$WORK/$configuration-$round-$attempt.txt"
      SINGLEFS_KERNEL="$KERNEL" VM_EXTRA_ROOT="$WORK/root" VM_DISKS="$disks" VM_DISK_MB=16384 VM_MEM=4096 VM_CPUS=4 \
        VM_DISK_PREALLOCATE=1 VM_TIMEOUT=1500 \
        bash "$WORK/scripts/devices/vm-bench.sh" "$GUEST_BINARY" "$configuration" "$round" > "$run_output" 2>&1
      vm_exit=$?
      {
        echo "E152RUN configuration=$configuration round=$round attempt=$attempt host_load1=$host_load vm_exit=$vm_exit" || fail "写入运行记录失败：$PARTIAL" "检查空间及权限，本轮未发布"
        # 只收行首的：vm-bench.sh 已经从控制台里抠干净了才打印；失败时它还会把控制台末尾缩进着再打一遍，那一份不收
        sed -n '/^E7RESULT /p' "$run_output" | tr -d '\r'
      } >> "$PARTIAL" || fail "追加结果失败：$PARTIAL" "检查空间及权限，本轮未发布"
      echo "  $(TZ=Asia/Tokyo date '+%H:%M:%S') 第 $round 轮 $configuration 第 $attempt 次 vm_exit=$vm_exit 宿主负载 $host_load" >&2
      [[ $vm_exit -eq 0 ]] && break
      tail -25 "$run_output" | sed 's/^/      /' >&2
    done
  done
done

"$HOST_BINARY" summarize "$PARTIAL" > "$WORK/summary.txt" \
  || fail "汇总失败，产物留在 $PARTIAL" "$HOST_BINARY summarize $PARTIAL 看报错"
cat "$WORK/summary.txt" >> "$PARTIAL" || fail "追加汇总失败：$PARTIAL" "检查空间及权限，本轮未发布"
output_lines="$(wc -l < "$PARTIAL")" || fail "读不了 $PARTIAL" "检查产物权限，本轮未发布"
summary_lines="$(wc -l < "$WORK/summary.txt")" || fail "读不了汇总" "检查工作目录权限，本轮未发布"
mv -T -- "$PARTIAL" "$OUTPUT" || fail "发布失败，产物保留在 $PARTIAL" "检查目标路径、空间及权限"
echo "  ✓ E152 产物：$OUTPUT（$output_lines 行，其中汇总 $summary_lines 行）" >&2
