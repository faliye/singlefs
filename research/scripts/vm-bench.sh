#!/usr/bin/env bash
# 在 QEMU/KVM 虚机里跑一个静态二进制，并给它一块真实的块设备（/dev/vda）。
#
#   vm-bench.sh <静态二进制> [参数...]    跑它，退出码传回宿主
#   vm-bench.sh --selftest                验证本 harness 能分辨成功与失败
#
# 为什么不用 singlefs-ai-sop/scripts/qemu/run.sh：那个 harness 不挂任何块设备，
# 也没有把额外文件塞进 initramfs 的钩子（可调环境变量只有 SINGLEFS_KERNEL 与 TMPDIR）。
# 索引 benchmark 要量的就是块设备行为，没有盘就没有被测对象。
#
# 纪律沿用 rules/command-safety.md：镜像落 TMPDIR 不落仓库；虚机 pid 写文件，
# 按字面量 pid 清理，不按名字匹配。读不到退出标记一律判「不明」，绝不当成 0。
set -uo pipefail

VM_MEM="${VM_MEM:-2048}"
VM_CPUS="${VM_CPUS:-4}"
VM_DISK_MB="${VM_DISK_MB:-4096}"
# 挂几块盘。默认 1（保持既有用法与 --selftest 不变）；>1 时来宾看到 /dev/vda /dev/vdb …，
# 全部路径按顺序作为**前缀参数**传给二进制。多盘是 E54（根环丢盘）要的：
# 「丢一整块盘之后还挂不挂得上」只有在真的有多块盘时才问得出来。
VM_DISKS="${VM_DISKS:-1}"
VM_TIMEOUT="${VM_TIMEOUT:-900}"

die() { printf '  \033[31m✗\033[0m %s\n' "$*" >&2; exit 1; }
ok()  { printf '  \033[32m✓\033[0m %s\n' "$*"; }
say() { printf '%s\n' "$*"; }

find_kernel() {
  [[ -n "${SINGLEFS_KERNEL:-}" ]] && { [[ -r "$SINGLEFS_KERNEL" ]] && { printf '%s' "$SINGLEFS_KERNEL"; return 0; }; return 1; }
  local k; for k in /boot/vmlinuz-*; do [[ -r "$k" ]] && { printf '%s' "$k"; return 0; }; done
  return 1
}

# 工作目录登记簿。**必须在 run_one 之前声明**：run_one 在命令替换（子 shell）里跑，
# 它只能继承已经存在的变量，且它对变量的赋值传不回来——所以走文件，不走变量。
VM_WORKLIST="${VM_WORKLIST:-}"

# 跑一次。stdout 回显退出码；控制台日志路径留在 VM_LOG。
run_one() {
  local bin="$1"; shift
  local kernel="$1"; shift
  local work ird pidf logf disk
  work="$(mktemp -d "${TMPDIR:-/tmp}/singlefs-vmbench.XXXXXX")"
  ird="$work/irfs"; pidf="$work/qemu.pid"; logf="$work/console.log"; disk="$work/disk.img"
  mkdir -p "$ird"/{bin,proc,sys,dev}
  cp "$(command -v busybox)" "$ird/bin/busybox"
  ( cd "$ird/bin" && ./busybox --list | while read -r a; do ln -sf busybox "$a"; done ) 2>/dev/null || true
  cp "$bin" "$ird/bench"; chmod +x "$ird/bench"
  printf '%s\n' "$*" > "$ird/args"
  # N 块盘：disk.img / disk1.img …；来宾侧设备名按 virtio 顺序 vda vdb vdc …
  local d devs=""
  for ((d=0; d<VM_DISKS; d++)); do
    truncate -s "${VM_DISK_MB}M" "$work/disk$d.img"
    devs="$devs /dev/vd$(printf "\\$(printf '%03o' $((97+d)))")"
  done
  printf '%s\n' "${devs# }" > "$ird/devs"
  disk="$work/disk0.img"

  cat > "$ird/init" <<'INIT'
#!/bin/sh
/bin/busybox mount -t proc proc /proc 2>/dev/null
/bin/busybox mount -t sysfs sysfs /sys 2>/dev/null
/bin/busybox mdev -s 2>/dev/null
if [ ! -b /dev/vda ]; then
  echo "VMBENCH_NODISK"
  echo "SINGLEFS_EXIT=200"
  /bin/busybox poweroff -f
fi
/bench $(cat /devs) $(cat /args); rc=$?
echo "SINGLEFS_EXIT=$rc"
/bin/busybox poweroff -f
INIT
  chmod +x "$ird/init"
  ( cd "$ird" && find . | cpio -o -H newc --quiet | gzip -1 > "$work/initramfs.cpio.gz" )

  # 寻道主导模型：VM_IOPS 非空时给盘加一个「每次 I/O 固定代价、与大小无关」的上限。
  # 这正是旋转介质的本质不对称——一次 64 KiB 读和一次 4 KiB 读代价相同。
  # 本机没有旋转盘（lsblk 确认只有 nvme0n1 ROTA=0），所以这是**模型化设备上的实测**，
  # 不是旋转盘实测，引用时口径不许混。
  local throttle=() drivearg
  if [[ -n "${VM_IOPS:-}" ]]; then
    throttle=( -object "throttle-group,id=tg0,x-iops-total=${VM_IOPS}" )
    drivearg=( -blockdev "driver=raw,node-name=raw0,file.driver=file,file.filename=$disk,file.aio=native,file.cache.direct=on"
               -blockdev "driver=throttle,node-name=thr0,throttle-group=tg0,file=raw0"
               -device "virtio-blk-pci,drive=thr0" )
  else
    drivearg=()
    for ((d=0; d<VM_DISKS; d++)); do
      drivearg+=( -drive "file=$work/disk$d.img,if=virtio,format=raw,cache=none,aio=native" )
    done
  fi
  # VM_CPU 让上层控制来宾看到哪些 CPU 特性（例如屏蔽 AES-NI：VM_CPU="host,-aes"）。
  # 不设时用 host，否则默认 CPU 模型不暴露 AES-NI，会把「算法慢」和「没有指令集」混为一谈。
  local cpuarg=( -cpu "${VM_CPU:-host}" )
  timeout "$VM_TIMEOUT" qemu-system-x86_64 \
    -enable-kvm -m "$VM_MEM" -smp "$VM_CPUS" -no-reboot -nographic -serial mon:stdio -display none \
    "${cpuarg[@]}" \
    -kernel "$kernel" -initrd "$work/initramfs.cpio.gz" \
    "${throttle[@]}" "${drivearg[@]}" \
    -append "console=ttyS0 quiet panic=1" \
    -pidfile "$pidf" >"$logf" 2>&1 || true

  if [[ -s "$pidf" ]]; then
    local vp; vp="$(cat "$pidf")"
    [[ "$vp" =~ ^[0-9]+$ ]] && kill -0 "$vp" 2>/dev/null && kill -9 "$vp" 2>/dev/null || true
  fi

  # 子 shell 里的变量赋值传不回父进程——日志路径必须落文件传出去，
  # 否则父进程抓不到 E7RESULT，结果会静默变成「零条」。
  printf '%s' "$logf" > "$VM_LOGPTR"
  # **每次都登记工作目录**——`--selftest` 会连跑三次，只记最后一次会漏掉前两个
  # （2026-08-29 实测：修了「只删最后一个」之后仍剩 2 个）。追加写，父进程在 EXIT 里逐个删。
  printf '%s\n' "$work" >> "$VM_WORKLIST"
  local rc; rc="$(sed -n 's/.*SINGLEFS_EXIT=\([0-9]\+\).*/\1/p' "$logf" | tail -1)"
  [[ -n "$rc" ]] && { printf '%s' "$rc"; return 0; }
  return 1
}

VM_LOGPTR="$(mktemp "${TMPDIR:-/tmp}/singlefs-vmlog.XXXXXX")"
VM_WORKLIST="$(mktemp "${TMPDIR:-/tmp}/singlefs-vmwork.XXXXXX")"

# ⚠️ **工作目录必须收掉。** `run_one` 在命令替换里跑（子 shell），
# 它 `mktemp -d` 出来的路径传不回父进程——正是
# `.claude/singlefs-ai-sop/rules/command-safety.md`「子 shell 里的赋值传不回父进程」那一条。
# 实测后果：2026-08-29 清出 **154 个残留目录、4.5 GB**。
# 修法沿用本脚本已有的指针文件模式：日志路径在 `$VM_LOGPTR` 里，工作目录是它的父目录。
# 只删名字对得上 `singlefs-vmbench.*` 的那一个，删错目录的风险按模式挡住。
# `VM_KEEP=1` 时保留，供失败后翻现场。
cleanup_work() {
  if [[ "${VM_KEEP:-0}" == "1" ]]; then
    [[ -s "$VM_WORKLIST" ]] && { printf '  ! 工作目录保留：\n'; sed 's/^/      /' "$VM_WORKLIST"; }
    rm -f "$VM_LOGPTR" "$VM_WORKLIST"; return
  fi
  if [[ -s "$VM_WORKLIST" ]]; then
    local d
    while IFS= read -r d; do
      [[ -n "$d" ]] || continue
      # 只删名字对得上的那些，删错目录的风险按模式挡住
      case "$d" in
        */singlefs-vmbench.*) rm -rf "$d" ;;
        *) printf '  ! 工作目录名字不符合预期，未删：%s\n' "$d" >&2 ;;
      esac
    done < "$VM_WORKLIST"
  fi
  rm -f "$VM_LOGPTR" "$VM_WORKLIST"
}
trap cleanup_work EXIT

say ""; say "══ 虚机 benchmark harness ══"
command -v qemu-system-x86_64 >/dev/null || die "qemu-system-x86_64 缺失"
[[ -r /dev/kvm && -w /dev/kvm ]] || die "/dev/kvm 不可读写 —— 不降级到软件模拟"
command -v busybox >/dev/null || die "busybox 缺失（apt install busybox-static）"
command -v cpio >/dev/null || die "cpio 缺失"
KERNEL="$(find_kernel)" || die "找不到可读的内核镜像。SINGLEFS_KERNEL=/path/to/bzImage 指定，或 sudo chmod +r /boot/vmlinuz-\$(uname -r)"
ok "内核 $KERNEL"
ok "虚机 ${VM_MEM}M 内存 / ${VM_CPUS} vCPU / ${VM_DISK_MB}M virtio 盘"

if [[ "${1:-}" == "--selftest" ]]; then
  say ""
  say "  自检：跑一个必然成功的、一个必然失败的、一个查得到 /dev/vda 的。"
  say "  分辨不出失败的那个，说明这个 harness 会把失败当成成功。"
  say ""
  tmp="$(mktemp -d "${TMPDIR:-/tmp}/singlefs-vmself.XXXXXX")"
  # 用 busybox sh 当「二进制」：它在 initramfs 里一定跑得起来
  printf '#!/bin/sh\nexit 0\n'  > "$tmp/ok";   chmod +x "$tmp/ok"
  printf '#!/bin/sh\nexit 7\n'  > "$tmp/bad";  chmod +x "$tmp/bad"
  printf '#!/bin/sh\n[ -b "$1" ] || exit 9\nsz=$(/bin/busybox blockdev --getsize64 "$1" 2>/dev/null)\necho "VMBENCH_DISK_BYTES=$sz"\n[ "$sz" -gt 0 ] || exit 10\nexit 0\n' > "$tmp/disk"; chmod +x "$tmp/disk"
  fail=0
  g_ok="$(run_one "$tmp/ok" "$KERNEL")"    || { die "成功用例：读不到退出标记"; }
  [[ "$g_ok" == 0 ]] && ok "成功用例 → 0" || { printf '  ✗ 成功用例 → %s，期望 0\n' "$g_ok"; fail=1; }
  g_bad="$(run_one "$tmp/bad" "$KERNEL")"  || { die "失败用例：读不到退出标记"; }
  [[ "$g_bad" == 7 ]] && ok "失败用例 → 7（harness 认得出失败）" || { printf '  ✗ 失败用例 → %s，期望 7 —— harness 会把失败当成成功\n' "$g_bad"; fail=1; }
  g_dsk="$(run_one "$tmp/disk" "$KERNEL")" || { die "盘用例：读不到退出标记"; }
  if [[ "$g_dsk" == 0 ]]; then
    bytes="$(sed -n 's/.*VMBENCH_DISK_BYTES=\([0-9]\+\).*/\1/p' "$(cat "$VM_LOGPTR")" | tail -1)"
    # 读不到 ≠ 读到 0（rules/test-discipline.md）：抓不到这个数就是抓取路径坏了，
    # 而主路径抓 E7RESULT 用的是同一条路——这里放过去，benchmark 结果会静默变成零条。
    if [[ -n "$bytes" && "$bytes" -gt 0 ]]; then
      ok "盘用例 → 0，虚机里看到 /dev/vda $bytes 字节"
    else
      printf '  ✗ 盘用例退出码 0，但从控制台读不到 VMBENCH_DISK_BYTES —— 结果抓取路径坏了\n'; fail=1
    fi
  else
    printf '  ✗ 盘用例 → %s（9=不是块设备，10=大小为 0，200=根本没有 /dev/vda）\n' "$g_dsk"; fail=1
  fi
  rm -rf "$tmp"
  say ""
  [[ $fail -eq 0 ]] || die "harness 自检未通过"
  ok "自检通过：退出码如实传回，且虚机里确实有一块可用的盘"
  exit 0
fi

[[ $# -ge 1 ]] || die "用法：vm-bench.sh <静态二进制> [参数...]   或   vm-bench.sh --selftest"
BIN="$1"; shift
[[ -x "$BIN" ]] || die "二进制不可执行：$BIN"
rc="$(run_one "$BIN" "$KERNEL" "$@")" || { printf '  ✗ 读不到退出标记，判定不明，整轮作废\n'; [[ -s "$VM_LOGPTR" ]] && tail -20 "$(cat "$VM_LOGPTR")" | sed 's/^/        /'; exit 1; }
LOG="$(cat "$VM_LOGPTR")"
# 不锚定行首：第一行会被 BIOS 的控制台转义序列顶掉行首（实测 "Booting from ROM..^[c^[[?7l^[[2J"），
# 锚定 ^ 会把它静默漏掉。
mapfile -t LINES < <(grep -ao 'E7RESULT .*' "$LOG" | tr -d '\r')
n=${#LINES[@]}
printf '%s\n' "${LINES[@]}"

if [[ "$rc" != 0 ]]; then printf '  ✗ 退出码 %s\n' "$rc"; tail -30 "$LOG" | sed 's/^/        /'; exit 1; fi

# 完整性校验：被测程序在收尾行报出它发了多少条，抓到的条数必须相等。
# 少一条就说明控制台吞了结果——不设这道闸，实验会静默地少一项而没人知道。
declared="$(printf '%s\n' "${LINES[@]}" | sed -n 's/.*name=done emitted=\([0-9]\+\).*/\1/p' | tail -1)"
if [[ -z "$declared" ]]; then
  printf '  ✗ 抓到 %s 条结果，但没有收尾行（name=done）—— 判定不明，整轮作废\n' "$n"
  tail -20 "$LOG" | sed 's/^/        /'; exit 1
fi
if [[ "$n" -ne "$declared" ]]; then
  printf '  ✗ 结果条数对不上：程序声称发了 %s 条，宿主只抓到 %s 条 —— 控制台吞了结果，整轮作废\n' "$declared" "$n"
  exit 1
fi
ok "退出码 0，抓到 $n 条结果（与程序声称的 $declared 条相符）"
