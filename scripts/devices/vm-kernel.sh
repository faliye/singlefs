#!/usr/bin/env bash
# admission: always 每一次调都判此刻的仓与这一次的参数，上一次的结论不替这一次作保
# run-condition: none 找不到可读的内核镜像时它自己判红并打印怎么设 SINGLEFS_KERNEL
# 给 QEMU harness 找一个**可读的**内核镜像，找不到就从 /boot 复制一份出来。
#
#   vm-kernel.sh            打印可用内核的路径（必要时先复制）
#   vm-kernel.sh --check    只检查，不复制；有可读内核回 0，没有回 1
#   vm-kernel.sh --release <版本>   只认 /boot/vmlinuz-<版本>，必要时复制成 TMPDIR 下的 singlefs-vmlinuz-<版本>
#
# 为什么要它：`/boot/vmlinuz-*` 是 `-rw------- root root`，而 `vm-bench.sh` 的
# `find_kernel()` 只认可读的镜像。每次跑虚机实验都卡在这一步，所以做成脚本。
#
# 纪律（.claude/singlefs-ai-sop/rules/command-safety.md）：
#   - **口令不打印、不进命令行可见处**：从 `.env` 读进变量，用 `sudo -S` 走 stdin。
#   - **不用 echo 假装成功**：复制完回读确认可读且大小一致，不一致就报错退出。
#   - 目标路径固定在本会话 scratchpad 或 TMPDIR，不落仓库。
set -uo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEST_DIR="${SINGLEFS_KERNEL_DIR:-${TMPDIR:-/tmp}}"
DEST="$DEST_DIR/singlefs-vmlinuz"

die() { printf '  \033[31m✗\033[0m %s\n' "$*" >&2; exit 1; }

# 目录、空文件与复制中断留下的文件都不是可用内核。
kernel_readable() {
  [[ -f "$1" && -s "$1" && -r "$1" ]]
}
cached_kernel_matches() {
  kernel_readable "$1" || return 1
  local source_size cached_size
  source_size="$(stat -c %s "$2" 2>/dev/null)" || return 1
  cached_size="$(stat -c %s "$1")" || return 1
  [[ "$source_size" == "$cached_size" ]]
}

# 已经有可读的就直接用——SINGLEFS_KERNEL 优先，其次已复制过的，再次 /boot 里碰巧可读的。
find_readable() {
  [[ -n "${SINGLEFS_KERNEL:-}" ]] && kernel_readable "$SINGLEFS_KERNEL" && { printf '%s' "$SINGLEFS_KERNEL"; return 0; }
  kernel_readable "$DEST" && { printf '%s' "$DEST"; return 0; }
  local k; for k in /boot/vmlinuz-*; do kernel_readable "$k" && { printf '%s' "$k"; return 0; }; done
  return 1
}

if [[ "${1:-}" == "--release" ]]; then
  # 指定版本：E152（按里程碑对比六家文件系统的文件性能） 要与宿主同版本、不带 lockdep 的内核，
  # 而 find_readable 会先撞上 /boot 里唯一可读的那个 lockdep 内核（锁校验开销因文件系统而异，量出来的时间不能拿来比）。
  RELEASE="${2:-}"
  [[ -n "$RELEASE" ]] || die "--release 后面没有内核版本" "→ 例：vm-kernel.sh --release $(uname -r)"
  DEST="$DEST_DIR/singlefs-vmlinuz-$RELEASE"
  SRC="/boot/vmlinuz-$RELEASE"
  cached_kernel_matches "$DEST" "$SRC" && { printf '%s\n' "$DEST"; exit 0; }
  kernel_readable "$SRC" && { printf '%s\n' "$SRC"; exit 0; }
  [[ -e "$SRC" ]] || die "/boot 下没有 vmlinuz-$RELEASE" "→ ls /boot 看有哪些版本，把其中一个传给 --release"
else
  if K="$(find_readable)"; then
    printf '%s\n' "$K"; exit 0
  fi
  [[ "${1:-}" == "--check" ]] && exit 1

  # ── 到这里说明要复制一份 ──
  SRC="/boot/vmlinuz-$(uname -r)"
  [[ -e "$SRC" ]] || { SRC="$(ls -1 /boot/vmlinuz-* 2>/dev/null | tail -1)"; }
  [[ -n "$SRC" && -e "$SRC" ]] || die "/boot 下找不到任何内核镜像" "装一个内核（对应发行版的 linux-image 包），或者 SINGLEFS_KERNEL=/path/to/bzImage 指一个现成的。"
fi

[[ -f "$REPO/.env" ]] || die "需要 sudo 复制内核，但 $REPO/.env 不在。手动做一次：
       sudo cp $SRC $DEST && sudo chown \$USER $DEST
       或者 SINGLEFS_KERNEL=/path/to/bzImage 指一个可读的"

# 只取口令变量，不打印。set -a 让 .env 里的赋值进环境。
set -a; . "$REPO/.env"; set +a
PASS=""
for v in "${SUDO_PASS_A:-}" "${SUDO_PASS_B:-}"; do
  [[ -z "$v" ]] && continue
  if printf '%s\n' "$v" | sudo -S -p '' true 2>/dev/null; then PASS="$v"; break; fi
done
[[ -n "$PASS" ]] || die ".env 里的口令都不通过 sudo 校验" "检查 $REPO/.env 里 SUDO_PASS_A / SUDO_PASS_B 是不是最新口令；改完重跑，或者手动 sudo cp $SRC $DEST 再 sudo chown \$USER $DEST。"

# 私有临时文件验证完成后才发布；复制失败不会污染下一次读取的缓存。
COPY="$(mktemp "$DEST_DIR/.singlefs-vmlinuz.XXXXXX")" || die "建不了内核临时文件：$DEST_DIR"
trap 'rm -f -- "${COPY:?}"' EXIT
printf '%s\n' "$PASS" | sudo -S -p '' cp "$SRC" "$COPY" 2>/dev/null || die "复制内核失败：$SRC"
printf '%s\n' "$PASS" | sudo -S -p '' chown "$(id -u):$(id -g)" "$COPY" 2>/dev/null || die "内核副本改属主失败"
kernel_readable "$COPY" || die "内核副本为空、不可读或不是普通文件"
SRC_SZ="$(printf '%s\n' "$PASS" | sudo -S -p '' stat -c%s "$SRC" 2>/dev/null)" || die "读取源内核大小失败"
DST_SZ="$(stat -c%s "$COPY")" || die "读取副本大小失败"
[[ "$SRC_SZ" == "$DST_SZ" ]] || die "大小对不上：源 $SRC_SZ 目标 $DST_SZ"
mv -T -- "$COPY" "$DEST" || die "发布内核缓存失败：$DEST"
printf '%s\n' "$DEST"
