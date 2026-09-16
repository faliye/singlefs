#!/usr/bin/env bash
# gate-stage: QEMU 真设备上的第一个事务（两块 virtio 盘、设备侧独立录制、漏一道屏障与走页缓存两个对照必须判红）
# gate-covers: QEMU 真实负载
#
# C6（块层语义假设写错）要的：「程序以为发了什么」与「盘上实际收到了什么」由两条不共享代码的路比。
#   程序那一条 = 被测程序自己的录制器（宿主上同参数重跑，同字节）；
#   盘那一条   = QEMU 的 blklogwrites 过滤节点在来宾之外按 dm-log-writes 格式记下的每个写（带数据）与每个 FLUSH。
# 三次虚机跑（并行，本机合计十几秒）：
#   direct                          O_DIRECT 真写路：设备侧日志逐项等于程序的录制流（末尾至多一个关机 FLUSH），
#                                   来宾块层的 FLUSH 数 = 屏障 + FUA 写数，冷重开读回文件，段序列与 E142 产物逐字相同，宿主从盘镜像再读回一次
#   skip-first-transaction-barrier  盘 0 漏掉「单元 → journal 记录」那道屏障：设备侧比对必须判红，且红在盘 0
#   page-cache                      读写走页缓存（阳性对照）：回写合并 / 重排写，设备侧比对必须判红
# 本阶段没有 fixtures 样本：判红要真起虚机，装不进 fixtures 目录；两个必须判红的对照就是它自己的判别力自证。
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
[[ -f Cargo.toml && -d crates/singlefs-harness ]] || { echo "  ! 没有 crates/singlefs-harness，本阶段跳过（步 0 之前没有装置）"; exit 77; }

fail() { echo "  ✗ $1"; echo "     → 怎么办：$2"; exit 1; }

command -v qemu-system-x86_64 >/dev/null || fail "qemu-system-x86_64 缺失" "装 QEMU（qemu-system-x86），见 .claude/kb/vm-harness.md「三个前置」。"
[[ -r /dev/kvm && -w /dev/kvm ]] || fail "/dev/kvm 不可读写" "按 .claude/kb/vm-harness.md「三个前置」查 kvm 组成员身份；不许用 setfacl 补。"
bash research/scripts/vm-kernel.sh --check >/dev/null 2>&1 || fail "找不到可读的内核镜像" "跑 bash research/scripts/vm-kernel.sh，按它打印的路径设 SINGLEFS_KERNEL。"

if ! cargo build --release --target x86_64-unknown-linux-musl -p singlefs-harness --bin first_transaction_on_device >/dev/null 2>&1; then
  fail "虚机二进制（musl 静态）编不过" "单跑 cargo build --release --target x86_64-unknown-linux-musl -p singlefs-harness --bin first_transaction_on_device 看报错。"
fi
if ! cargo build -p singlefs-harness --bin first_transaction_device_log_check >/dev/null 2>&1; then
  fail "宿主一侧的设备日志检查编不过" "单跑 cargo build -p singlefs-harness --bin first_transaction_device_log_check 看报错。"
fi
BIN="target/x86_64-unknown-linux-musl/release/first_transaction_on_device"
CHECK="target/debug/first_transaction_device_log_check"
PRODUCT="research/results/$(awk -F'|' '$1=="E142"{print $4}' research/scripts/replay.sh)"
[[ -f "$PRODUCT" ]] || fail "从 replay.sh 的 E142 行解析不出产物文件（得到 $PRODUCT）" "看 research/scripts/replay.sh 里 E142 那一行的第 4 列。"
# 反向链从产物里现取，不写死：写死的那个数在格式常量一改就过期，而这道闸红起来像是写路错了。
# 实测 2026-09-16 树表条目 148 → 200：产物已经是新链值，闸里还写着旧的，红在一个与写路无关的地方。
EXPECTED_BACK_CHAIN="$(sed -n 's/.*name=root_record .*back_chain=\([0-9][0-9]*\).*/\1/p' "$PRODUCT" | head -1)"
[[ -n "$EXPECTED_BACK_CHAIN" ]] || fail "产物里取不出 name=root_record 的 back_chain（$PRODUCT）" "看 $PRODUCT 有没有 name=root_record 那一行、它的 back_chain 字段还在不在。"

work="$(mktemp -d "${TMPDIR:-/tmp}/singlefs-gate55.XXXXXX")"
trap 'rm -rf "${work:?}"' EXIT
MODES=(direct skip-first-transaction-barrier page-cache)
for mode in "${MODES[@]}"; do
  mkdir -p "$work/$mode"
  ( VM_DISKS=2 VM_DISK_MB=4096 VM_BLKLOGWRITES_DIR="$work/$mode" bash research/scripts/vm-bench.sh "$BIN" "$mode" >"$work/$mode/out.txt" 2>&1
    echo "$?" >"$work/$mode/vm-exit" ) &
done
wait

checks=0
for mode in "${MODES[@]}"; do
  out="$work/$mode/out.txt"
  if [[ "$(cat "$work/$mode/vm-exit")" != 0 ]]; then
    tail -15 "$out" | sed 's/^/        /'
    fail "虚机跑 $mode 没过（vm-bench.sh 的退出码或条数闸）" "单跑：VM_DISKS=2 VM_DISK_MB=4096 VM_BLKLOGWRITES_DIR=<目录> bash research/scripts/vm-bench.sh $BIN $mode"
  fi
  if ! diff <(grep -ao 'E7RESULT name=segments .*' "$PRODUCT" | tr -d '\r') <(grep -ao 'E7RESULT name=segments .*' "$out" | tr -d '\r') >/dev/null; then
    fail "虚机跑 $mode 的段序列与 E142 产物的 name=segments 行不一致" "对照 $PRODUCT 与虚机输出的 name=segments 行；真设备上的写路发出的步骤与装置不同，先查是哪一段。"
  fi
  grep -aq 'name=recover_cold outcome=file_read root=1:3 content_matches=true' "$out" \
    || fail "虚机跑 $mode 冷重开之后没读回文件" "看 $mode 的 name=recover_cold 行与来宾的 stderr（vm-bench.sh 保留现场用 VM_KEEP=1）。"
  grep -aq "name=transaction policy_mismatches=0 key_order_mismatches=0 root_txg=3 back_chain=$EXPECTED_BACK_CHAIN" "$out" \
    || fail "虚机跑 $mode 的第一个事务计数或反向链与产物不符" "产物 name=root_record 行的 back_chain 是 $EXPECTED_BACK_CHAIN；两个运行时计数要是 0。"
  # 来宾块层的 FLUSH 数 = 程序真正交给设备的屏障 + FUA 写（来宾内核是第三条独立的路）
  while IFS= read -r line; do
    barriers="$(sed -n 's/.* barriers=\([0-9]*\) .*/\1/p' <<<"$line")"
    fua="$(sed -n 's/.* force_unit_access_writes=\([0-9]*\) .*/\1/p' <<<"$line")"
    flushes="$(sed -n 's/.* flush_ios=\([0-9NA]*\).*/\1/p' <<<"$line")"
    [[ "$flushes" =~ ^[0-9]+$ && "$flushes" == "$((barriers + fua))" ]] \
      || fail "虚机跑 $mode 的来宾块层 FLUSH 数（$flushes）不等于屏障 + FUA 写（$barriers + $fua）" "看 name=device_calls 行；读不到（NA）也算红——读不到不等于读到 0。"
    checks=$((checks + 1))
  done < <(grep -ao 'E7RESULT name=device_calls .*' "$out" | tr -d '\r')
  geometry="$(grep -ao 'E7RESULT name=geometry .*' "$out" | tr -d '\r' | head -1)"
  bytes="$(sed -n 's/.*device_bytes=\([0-9]*\).*/\1/p' <<<"$geometry")"
  physical="$(sed -n 's/.*physical_block_size=\([0-9]*\).*/\1/p' <<<"$geometry")"
  minimum="$(sed -n 's/.*minimum_io=\([0-9]*\).*/\1/p' <<<"$geometry")"
  disks=()
  [[ "$mode" == direct ]] && disks=("$work/$mode/disk0.img" "$work/$mode/disk1.img")
  "$CHECK" "$work/$mode/log0.img" "$work/$mode/log1.img" "$bytes" "$physical" "$minimum" "${disks[@]}" >"$work/$mode/check.txt" 2>&1
  echo "$?" >"$work/$mode/check-exit"
  checks=$((checks + 4))
done

verdict() { cat "$work/$1/check-exit"; }
[[ "$(verdict direct)" == 0 ]] || { sed 's/^/        /' "$work/direct/check.txt"
  fail "direct：设备侧日志与程序的录制流对不上" "上面 divergence= 指着第一处不一致的下标、程序以为的与盘上收到的；先判是写路的错还是比对口径的错。"; }
grep -q 'name=host_recover outcome=file_read root=1:3 content_matches=true' "$work/direct/check.txt" \
  || fail "direct：宿主从虚机写出的盘镜像上读不回文件" "看 $work/direct/check.txt 的 name=host_recover 行。"
[[ "$(verdict skip-first-transaction-barrier)" == 1 ]] && grep -q 'name=device_log device=0 .*divergence=at=' "$work/skip-first-transaction-barrier/check.txt" \
  && grep -q 'name=device_log device=1 .*divergence=none' "$work/skip-first-transaction-barrier/check.txt" \
  || { sed 's/^/        /' "$work/skip-first-transaction-barrier/check.txt"
       fail "对照 skip-first-transaction-barrier 没有红在盘 0：这道比对分不出漏了一道屏障" "比对失去判别力，之前所有「对得上」作废；看 device_log.rs 的 expected_device_events 与解析。"; }
[[ "$(verdict page-cache)" == 1 ]] \
  || { sed 's/^/        /' "$work/page-cache/check.txt"
       fail "对照 page-cache 没有判红：走页缓存的回写与 O_DIRECT 的逐个写在这道比对里分不出来" "比对失去判别力；看设备侧日志里写的条数与长度。"; }
echo "  ✓ QEMU 真设备上的第一个事务：3 次虚机跑、$checks 项检查全过；direct 设备侧逐项对得上、两个对照都红在该红的地方"
