#!/usr/bin/env bash
# 给 QEMU harness 找一个**可读的**内核镜像，找不到就从 /boot 复制一份出来。
#
#   vm-kernel.sh            打印可用内核的路径（必要时先复制）
#   vm-kernel.sh --check    只检查，不复制；有可读内核回 0，没有回 1
#
# 为什么要它：`/boot/vmlinuz-*` 是 `-rw------- root root`，而 `vm-bench.sh` 的
# `find_kernel()` 只认可读的镜像。每次跑虚机实验都卡在这一步，所以做成脚本。
#
# 纪律（.claude/singlefs-ai-sop/rules/command-safety.md）：
#   - **口令不打印、不进命令行可见处**：从 `.env` 读进变量，用 `sudo -S` 走 stdin。
#   - **不用 echo 假装成功**：复制完回读确认可读且大小一致，不一致就报错退出。
#   - 目标路径固定在本会话 scratchpad 或 TMPDIR，不落仓库。
set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEST_DIR="${SINGLEFS_KERNEL_DIR:-${TMPDIR:-/tmp}}"
DEST="$DEST_DIR/singlefs-vmlinuz"

die() { printf '  \033[31m✗\033[0m %s\n' "$*" >&2; exit 1; }

# 已经有可读的就直接用——SINGLEFS_KERNEL 优先，其次已复制过的，再次 /boot 里碰巧可读的。
find_readable() {
  [[ -n "${SINGLEFS_KERNEL:-}" && -r "${SINGLEFS_KERNEL:-}" ]] && { printf '%s' "$SINGLEFS_KERNEL"; return 0; }
  [[ -r "$DEST" ]] && { printf '%s' "$DEST"; return 0; }
  local k; for k in /boot/vmlinuz-*; do [[ -r "$k" ]] && { printf '%s' "$k"; return 0; }; done
  return 1
}

if K="$(find_readable)"; then
  printf '%s\n' "$K"; exit 0
fi
[[ "${1:-}" == "--check" ]] && exit 1

# ── 到这里说明要复制一份 ──
SRC="/boot/vmlinuz-$(uname -r)"
[[ -e "$SRC" ]] || { SRC="$(ls -1 /boot/vmlinuz-* 2>/dev/null | tail -1)"; }
[[ -n "$SRC" && -e "$SRC" ]] || die "/boot 下找不到任何内核镜像"

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
[[ -n "$PASS" ]] || die ".env 里的口令都不通过 sudo 校验"

printf '%s\n' "$PASS" | sudo -S -p '' cp "$SRC" "$DEST" 2>/dev/null
printf '%s\n' "$PASS" | sudo -S -p '' chown "$(id -u):$(id -g)" "$DEST" 2>/dev/null

# **回读确认，不靠退出码**（command-safety：改状态的命令要回读实际状态）
[[ -r "$DEST" ]] || die "复制之后 $DEST 仍不可读"
SRC_SZ="$(printf '%s\n' "$PASS" | sudo -S -p '' stat -c%s "$SRC" 2>/dev/null)"
DST_SZ="$(stat -c%s "$DEST" 2>/dev/null)"
[[ -n "$SRC_SZ" && "$SRC_SZ" == "$DST_SZ" ]] || die "大小对不上：源 ${SRC_SZ:-?} 目标 ${DST_SZ:-?}"

printf '%s\n' "$DEST"
