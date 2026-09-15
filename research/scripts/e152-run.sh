#!/usr/bin/env bash
# E152（按里程碑对比六家文件系统的文件性能）：八个配置轮流跑 N 轮，每轮一个新虚机、一块新盘。
#
#   bash research/scripts/e152-run.sh <产物路径>
#   E152_ROUNDS=1 E152_CONFIGURATIONS="ext4" bash research/scripts/e152-run.sh <产物路径>   # 冒烟
#
# 口径、判据与作废条款都在跑前登记 research/prompts/e152-preregistration.md；这里只照着跑、照着记。
# 纪律（.claude/singlefs-ai-sop/rules/command-safety.md）：
#   - 本轮输出先写 <产物>.partial，全部跑完、汇总行追加完才改名成产物：中途失败不留一份看着像产物的文件；
#   - 每一次虚机跑的退出码都记进产物（E152RUN 行的 vm_exit），不吞；退出码非 0 的那一次同参数重跑一次
#     （跑前登记第八节第 1 条），两次都记；
#   - 八个配置按轮交错跑（第 1 轮八个都跑完再跑第 2 轮），宿主上的漂移平摊到每一家（跑前登记第七节）。
set -uo pipefail
RESEARCH="$(cd "$(dirname "$0")/.." && pwd)"
REPOSITORY="$(cd "$RESEARCH/.." && pwd)"
OUTPUT="${1:?用法：e152-run.sh <产物路径>}"
ROUNDS="${E152_ROUNDS:-5}"
CONFIGURATIONS="${E152_CONFIGURATIONS:-raw ext4 xfs f2fs btrfs bcachefs zfs singlefs}"
RELEASE="${E152_KERNEL_RELEASE:-$(uname -r)}"

fail() { echo "  ✗ $1" >&2; echo "    → $2" >&2; exit 1; }

case "$OUTPUT" in /*) ;; *) OUTPUT="$PWD/$OUTPUT" ;; esac
cd "$RESEARCH" || fail "进不了 $RESEARCH" "从仓里跑"
KERNEL="$(bash scripts/vm-kernel.sh --release "$RELEASE")" \
  || fail "拿不到 $RELEASE 的内核镜像" "单跑 bash research/scripts/vm-kernel.sh --release $RELEASE 看报错"
cargo build --release --target x86_64-unknown-linux-musl --bin e152-file-system-benchmark >/dev/null 2>&1 \
  || fail "来宾二进制编不过" "cd research && cargo build --release --target x86_64-unknown-linux-musl --bin e152-file-system-benchmark"
cargo build --release --bin e152-file-system-benchmark >/dev/null 2>&1 \
  || fail "宿主上的汇总二进制编不过" "cd research && cargo build --release --bin e152-file-system-benchmark"
( cd "$REPOSITORY" && cargo build --release --target x86_64-unknown-linux-musl -p singlefs-harness --bin first_transaction_on_device >/dev/null 2>&1 ) \
  || fail "singlefs 的真设备二进制编不过" "在仓根跑 cargo build --release --target x86_64-unknown-linux-musl -p singlefs-harness --bin first_transaction_on_device"
GUEST_BINARY="$RESEARCH/target/x86_64-unknown-linux-musl/release/e152-file-system-benchmark"
HOST_BINARY="$RESEARCH/target/release/e152-file-system-benchmark"
SINGLEFS_BINARY="$REPOSITORY/target/x86_64-unknown-linux-musl/release/first_transaction_on_device"

WORK="$(mktemp -d "${TMPDIR:-/tmp}/singlefs-e152.XXXXXX")"
trap 'rm -rf "${WORK:?}"' EXIT
# 起跑时把要调的脚本复制一份，之后只调副本：bash 边读边执行脚本，仓里的原件中途被别的会话一改，
# 正在跑的那一次就会读到错位的内容（2026-09-15 第二次正式跑实测：vm-bench.sh 开头注释被改，那一次报语法错退出）。
mkdir -p "$WORK/scripts"
cp scripts/vm-bench.sh scripts/e152-stage-root.sh "$WORK/scripts/" || fail "复制不了测试台脚本" "看 $WORK 的权限"
bash "$WORK/scripts/e152-stage-root.sh" "$WORK/root" "$RELEASE" "$SINGLEFS_BINARY" \
  || fail "附加根没建成" "单跑 bash research/scripts/e152-stage-root.sh <新的空目录> $RELEASE $SINGLEFS_BINARY"

PARTIAL="$OUTPUT.partial"
rm -f "$OUTPUT" "$PARTIAL"   # 本轮输出不许跨轮复用
echo "E152RUN_HEADER kernel=$RELEASE rounds=$ROUNDS configurations=${CONFIGURATIONS// /,} started_jst=$(TZ=Asia/Tokyo date '+%Y-%m-%dT%H:%M')" > "$PARTIAL"
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
        bash "$WORK/scripts/vm-bench.sh" "$GUEST_BINARY" "$configuration" "$round" > "$run_output" 2>&1
      vm_exit=$?
      {
        echo "E152RUN configuration=$configuration round=$round attempt=$attempt host_load1=$host_load vm_exit=$vm_exit"
        # 只收行首的：vm-bench.sh 已经从控制台里抠干净了才打印；失败时它还会把控制台末尾缩进着再打一遍，那一份不收
        grep -a '^E7RESULT ' "$run_output" | tr -d '\r'
      } >> "$PARTIAL"
      echo "  $(TZ=Asia/Tokyo date '+%H:%M:%S') 第 $round 轮 $configuration 第 $attempt 次 vm_exit=$vm_exit 宿主负载 $host_load" >&2
      [[ $vm_exit -eq 0 ]] && break
      tail -25 "$run_output" | sed 's/^/      /' >&2
    done
  done
done

"$HOST_BINARY" summarize "$PARTIAL" > "$WORK/summary.txt" \
  || fail "汇总失败，产物留在 $PARTIAL" "$HOST_BINARY summarize $PARTIAL 看报错"
cat "$WORK/summary.txt" >> "$PARTIAL"
mv "$PARTIAL" "$OUTPUT"
echo "  ✓ E152 产物：$OUTPUT（$(wc -l < "$OUTPUT") 行，其中汇总 $(wc -l < "$WORK/summary.txt") 行）" >&2
