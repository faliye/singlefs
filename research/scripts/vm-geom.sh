#!/usr/bin/env bash
# 在 QEMU 里造几何量不同的多块 NVMe 命名空间，进来宾读回各自的几何，证明来宾真的看到不同。
#
#   vm-geom.sh            异构档：三个命名空间几何各不相同 —— 期望「看到 3 种几何」
#   vm-geom.sh --uniform  阴性对照：三个命名空间几何完全相同 —— 期望「只看到 1 种几何」
#
# 为什么要阴性对照：只跑异构档的话，「看到 3 种几何」可能只是脚本在回声我给的参数。
# 同一条读取路径在同构档上必须塌成 1 种，否则这条读取路径没有判别力。
#
# 纪律沿用 rules/command-safety.md 与 show-me-test.md：
# 镜像落 TMPDIR；pid 写文件按字面量杀；日志路径落文件传出子 shell；
# 完整性闸比对条数，对不上整轮作废。
set -uo pipefail

VM_MEM="${VM_MEM:-1024}"
VM_TIMEOUT="${VM_TIMEOUT:-180}"
UNIFORM=0
[[ "${1:-}" == "--uniform" ]] && UNIFORM=1

die() { printf '  \033[31m✗\033[0m %s\n' "$*" >&2; exit 1; }
ok()  { printf '  \033[32m✓\033[0m %s\n' "$*"; }
say() { printf '%s\n' "$*"; }

command -v qemu-system-x86_64 >/dev/null || die "qemu-system-x86_64 缺失"
[[ -r /dev/kvm && -w /dev/kvm ]] || die "/dev/kvm 不可读写 —— 不降级到软件模拟"
command -v busybox >/dev/null || die "busybox 缺失"
command -v cpio >/dev/null || die "cpio 缺失"
# 内核与模块树必须同版本：本机 /boot 里有 4 个内核而多数不可读，
# 「第一个可读的」挑出来的那个未必有配套模块树，insmod 会因 vermagic 不符全部失败，
# 表现为「来宾一个设备都看不到」——看上去像模拟不成立，其实是挑错了内核。
KERNEL=""; KVER=""
for k in /boot/vmlinuz-*; do
  [[ -r "$k" ]] || continue
  v="${k#/boot/vmlinuz-}"
  [[ -d "/lib/modules/$v" ]] || continue
  KERNEL="$k"; KVER="$v"; break
done
[[ -n "$KERNEL" ]] || die "找不到「可读且有配套模块树」的内核。候选：$(ls /boot/vmlinuz-* 2>/dev/null | tr '\n' ' ')"
say "  内核 $KERNEL（模块树 /lib/modules/$KVER）"

WORK="$(mktemp -d "${TMPDIR:-/tmp}/singlefs-vmgeom.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT
IRD="$WORK/irfs"; PIDF="$WORK/qemu.pid"; LOGF="$WORK/console.log"
mkdir -p "$IRD"/{bin,proc,sys,dev}
cp "$(command -v busybox)" "$IRD/bin/busybox"
( cd "$IRD/bin" && ./busybox --list | while read -r a; do ln -sf busybox "$a"; done ) 2>/dev/null || true

# NVMe 在本机内核里是模块（CONFIG_BLK_DEV_NVME=m），极简 initramfs 里没有它就一个盘也看不见。
# 按 modprobe --show-depends 给的顺序塞进去；busybox insmod 不认 .zst，宿主先解压。
command -v zstd >/dev/null || die "zstd 缺失（解压内核模块要用）"
mkdir -p "$IRD/mods"
modorder="$IRD/mods/order"
: > "$modorder"
while read -r kop _; do
  [[ -r "$kop" ]] || die "模块不可读：$kop"
  # 模块树可能是压缩的（.zst）也可能不是——本机两种都有，写死一种会在另一种上失败
  case "$kop" in
    *.zst) base="$(basename "$kop" .zst)"; zstd -dqf "$kop" -o "$IRD/mods/$base" || die "解压失败：$kop" ;;
    *.ko)  base="$(basename "$kop")";      cp "$kop" "$IRD/mods/$base" || die "拷贝失败：$kop" ;;
    *)     die "不认识的模块格式：$kop" ;;
  esac
  printf '%s\n' "$base" >> "$modorder"
done < <(modprobe -S "$KVER" --show-depends nvme 2>/dev/null | sed 's/^insmod //')
[[ -s "$modorder" ]] || die "拿不到 nvme 模块依赖链"
say "  initramfs 内核模块：$(tr '\n' ' ' < "$modorder")"

# 来宾侧：逐个块设备读 /sys 里的几何，每读一个报一条，收尾报总条数（完整性闸）
cat > "$IRD/init" <<'INIT'
#!/bin/sh
/bin/busybox mount -t proc proc /proc 2>/dev/null
/bin/busybox mount -t sysfs sysfs /sys 2>/dev/null
/bin/busybox mdev -s 2>/dev/null
# 按依赖顺序装 nvme，装不上要说话——静默失败会让下面看到 0 个设备而分不清原因
while read -r m; do
  err=$(/bin/busybox insmod "/mods/$m" 2>&1) || echo "GEOM insmod_failed=$m err=$err"
done < /mods/order
/bin/busybox mdev -s 2>/dev/null
/bin/busybox sleep 1
n=0
for d in /sys/block/nvme*; do
  [ -d "$d" ] || continue
  name=$(/bin/busybox basename "$d")
  q="$d/queue"
  lbs=$(cat "$q/logical_block_size" 2>/dev/null || echo NA)
  pbs=$(cat "$q/physical_block_size" 2>/dev/null || echo NA)
  zm=$(cat "$q/zoned" 2>/dev/null || echo NA)
  cs=$(cat "$q/chunk_sectors" 2>/dev/null || echo NA)
  nz=$(cat "$d/nr_zones" 2>/dev/null || cat "$q/nr_zones" 2>/dev/null || echo NA)
  sz=$(/bin/busybox blockdev --getsize64 "/dev/$name" 2>/dev/null || echo NA)
  echo "GEOM dev=$name lbs=$lbs pbs=$pbs zoned=$zm chunk_sectors=$cs nr_zones=$nz bytes=$sz"
  n=$((n+1))
done
echo "GEOM done emitted=$n"
echo "SINGLEFS_EXIT=0"
/bin/busybox poweroff -f
INIT
chmod +x "$IRD/init"
( cd "$IRD" && find . | cpio -o -H newc --quiet | gzip -1 > "$WORK/initramfs.cpio.gz" )

for i in 1 2 3; do truncate -s 512M "$WORK/d$i.img"; done

if [[ $UNIFORM -eq 1 ]]; then
  NS_ARGS=(
    -device nvme-ns,drive=d1,bus=nv0,nsid=1,zoned=true,zoned.zone_size=64M,logical_block_size=4096,physical_block_size=4096
    -device nvme-ns,drive=d2,bus=nv0,nsid=2,zoned=true,zoned.zone_size=64M,logical_block_size=4096,physical_block_size=4096
    -device nvme-ns,drive=d3,bus=nv0,nsid=3,zoned=true,zoned.zone_size=64M,logical_block_size=4096,physical_block_size=4096
  )
  MODE="同构（阴性对照）"
else
  NS_ARGS=(
    -device nvme-ns,drive=d1,bus=nv0,nsid=1,zoned=true,zoned.zone_size=64M,logical_block_size=4096,physical_block_size=4096
    -device nvme-ns,drive=d2,bus=nv0,nsid=2,zoned=true,zoned.zone_size=128M,logical_block_size=4096,physical_block_size=4096
    -device nvme-ns,drive=d3,bus=nv0,nsid=3,logical_block_size=512,physical_block_size=512
  )
  MODE="异构"
fi

say ""; say "══ 异构几何来宾侧验证：$MODE ══"
timeout "$VM_TIMEOUT" qemu-system-x86_64 \
  -enable-kvm -m "$VM_MEM" -smp 2 -no-reboot -nographic -serial mon:stdio -display none \
  -kernel "$KERNEL" -initrd "$WORK/initramfs.cpio.gz" \
  -drive file="$WORK/d1.img",if=none,id=d1,format=raw \
  -drive file="$WORK/d2.img",if=none,id=d2,format=raw \
  -drive file="$WORK/d3.img",if=none,id=d3,format=raw \
  -device nvme,serial=GEOM,id=nv0 "${NS_ARGS[@]}" \
  -append "console=ttyS0 quiet panic=1" \
  -pidfile "$PIDF" >"$LOGF" 2>&1 || true

if [[ -s "$PIDF" ]]; then
  vp="$(cat "$PIDF")"
  [[ "$vp" =~ ^[0-9]+$ ]] && kill -0 "$vp" 2>/dev/null && kill -9 "$vp" 2>/dev/null || true
fi

KEEP="${TMPDIR:-/tmp}/vmgeom-console.log"
cp "$LOGF" "$KEEP" 2>/dev/null && say "  控制台日志留在 $KEEP"
mapfile -t LINES < <(grep -ao 'GEOM dev=.*' "$LOGF" | tr -d '\r')
declared="$(grep -ao 'GEOM done emitted=[0-9]*' "$LOGF" | tr -d '\r' | sed -n 's/.*emitted=\([0-9]\+\).*/\1/p' | tail -1)"
n=${#LINES[@]}
[[ -n "$declared" ]] || { cp "$LOGF" "${TMPDIR:-/tmp}/vmgeom-fail.log"; die "读不到完成标记 —— 整轮作废。日志：${TMPDIR:-/tmp}/vmgeom-fail.log"; }
[[ "$n" -eq "$declared" ]] || die "完整性闸：抓到 $n 条，来宾声称发了 $declared 条 —— 整轮作废"
# 0/0 也要拦：读不到 ≠ 读到 0（rules/test-discipline.md）。来宾一个块设备都没看到，
# 说明驱动没起来或命名不对，而不是「几何都一样」。
[[ "$n" -ge 1 ]] || die "来宾一个 nvme 块设备都没看到（emitted=0）—— 整轮作废，不是阴性结果"
ok "完整性闸通过：$n/$declared 条"
say ""
printf '%s\n' "${LINES[@]}" | sed 's/^/  /'
say ""
distinct="$(printf '%s\n' "${LINES[@]}" | sed 's/^GEOM dev=[a-z0-9]* //' | sort -u | wc -l)"
say "  来宾看到的不同几何种数：$distinct"
if [[ $UNIFORM -eq 1 ]]; then
  [[ "$distinct" -eq 1 ]] && ok "阴性对照通过：同构档塌成 1 种几何 ⇒ 读取路径有判别力" \
    || die "阴性对照失败：同构档看到 $distinct 种几何 —— 读取路径在回声参数，异构档的结论作废"
else
  [[ "$distinct" -ge 2 ]] && ok "异构档：来宾确实看到 $distinct 种不同几何" \
    || die "异构档只看到 $distinct 种几何 —— 来宾没能区分，模拟不成立"
fi
