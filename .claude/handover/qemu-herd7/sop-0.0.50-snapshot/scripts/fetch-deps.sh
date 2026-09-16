#!/usr/bin/env bash
# 取测试依赖。安装期可以后台跑，也可以不跑——各测试脚本会在真要用时调它。
#
#   fetch-deps.sh              取全部能自动取的
#   fetch-deps.sh --kernel     只取内核树（lkmm.sh 内部用这个）
#   fetch-deps.sh --check      只报告缺什么，不装
#
# 边界（rules/command-safety.md）：**本脚本不跑 sudo。**
# 系统包只打印那条命令，由人自己执行——静默改系统比缺个包危险得多。
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

CACHE="${XDG_CACHE_HOME:-$HOME/.cache}/singlefs"
KTREE_CACHE="$CACHE/linux-memory-model"
MODE="${1:---all}"

kernel_tree_path() {   # 已有的树优先，其次缓存
  local c
  for c in "${SINGLEFS_KERNEL_TREE:-}" "$HOME/linux" "$HOME/linux-bug-fix/linux" "$KTREE_CACHE"; do
    [[ -n "$c" && -d "$c/tools/memory-model" ]] && { printf '%s' "$c"; return 0; }
  done
  return 1
}

fetch_kernel() {
  if p="$(kernel_tree_path)"; then ok "内核树 $p（已有，跳过）"; return 0; fi
  command -v git >/dev/null || { bad "git 缺失"; howto "先装 git 再来。"; return 1; }
  warn "稀疏检出 tools/memory-model 到 $KTREE_CACHE（只取那一个目录，几 MB）"
  rm -rf "$KTREE_CACHE"
  if git clone -q --depth 1 --filter=blob:none --sparse \
       "${SINGLEFS_KERNEL_GIT:-https://github.com/torvalds/linux.git}" "$KTREE_CACHE" 2>/dev/null \
     && git -C "$KTREE_CACHE" sparse-checkout set tools/memory-model >/dev/null 2>&1 \
     && [[ -d "$KTREE_CACHE/tools/memory-model" ]]; then
    ok "内核树 $KTREE_CACHE（$(du -sh "$KTREE_CACHE" | cut -f1)）"
  else
    rm -rf "$KTREE_CACHE"
    bad "取内核树失败"
    howto "网络不通，或 git 太老（稀疏检出要 2.25+）。指一棵已有的树：" \
          "SINGLEFS_KERNEL_TREE=/path/to/linux"
    return 1
  fi
}

fetch_herd7() {
  if command -v herd7 >/dev/null 2>&1; then ok "herd7 已在"; return 0; fi
  if command -v opam >/dev/null 2>&1; then
    export OPAMROOT="${OPAMROOT:-$HOME/.opam}"
    eval "$(opam env --root="$OPAMROOT" --set-root 2>/dev/null)" || true
    command -v herd7 >/dev/null 2>&1 && { ok "herd7 已在（opam env）"; return 0; }
    warn "opam install herdtools7（要编译，几分钟）"
    if opam install -y herdtools7 >/dev/null 2>&1; then ok "herd7 装好"; return 0; fi
    bad "herdtools7 安装失败"
    howto "手动跑一遍看报什么： opam install herdtools7"; return 1
  fi
  bad "herd7 与 opam 都缺"
  howto "先装 opam（系统包），再 opam install herdtools7"; return 1
}

report_system() {   # 只报告与打印命令，绝不代跑 sudo
  local miss=()
  for c in qemu-system-x86_64 busybox cpio dmsetup fio; do
    command -v "$c" >/dev/null 2>&1 || miss+=("$c")
  done
  [[ -e /dev/kvm && -r /dev/kvm && -w /dev/kvm ]] || miss+=("kvm-权限")
  if [[ ${#miss[@]} -eq 0 ]]; then ok "系统级依赖齐全"; return 0; fi
  warn "缺系统级依赖：${miss[*]}"
  howto "这些要 root，本脚本不代跑。自己执行：" \
        "sudo apt install qemu-system-x86 busybox-static cpio dmsetup fio" \
        "sudo usermod -aG kvm \$USER   （改完要重新登录）"
  return 0   # 不算失败：装不装由人决定，门禁那边会各自报
}

case "$MODE" in
  --kernel) head1 "取内核树"; fetch_kernel ;;
  --check)  head1 "依赖检查"
            if p="$(kernel_tree_path)"; then ok "内核树 $p"; else warn "内核树 未取"; fi
            # 先进 opam 环境再判——不然装在 opam 里的 herd7 会被误报成缺失
            if ! command -v herd7 >/dev/null 2>&1 && command -v opam >/dev/null 2>&1; then
              export OPAMROOT="${OPAMROOT:-$HOME/.opam}"
              eval "$(opam env --root="$OPAMROOT" --set-root 2>/dev/null)" || true
            fi
            command -v herd7 >/dev/null 2>&1 && ok "herd7 $(herd7 -version 2>&1 | head -1)" || warn "herd7 未装"
            report_system ;;
  --all|"") head1 "取测试依赖"
            rc=0; fetch_kernel || rc=1; fetch_herd7 || rc=1; report_system
            say ""
            [[ $rc -eq 0 ]] && ok "能自动取的都取到了" || { bad "有依赖没取到，看上面"; exit 1; } ;;
  *) bad "未知参数 $MODE"; howto "用法： fetch-deps.sh [--all|--kernel|--check]"; exit 1 ;;
esac
