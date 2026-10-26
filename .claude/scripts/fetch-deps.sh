#!/usr/bin/env bash
# admission: always 取之前现查本机缺什么，已经取过的它自己认出来跳过，上一次的结论不替这一次作保
# run-condition: none 缺的工具它自己逐项查出来、打印装法（系统包不跑 sudo），取不了的那一项判红，不交给调用方预判
# 取测试依赖。安装期可以后台跑，也可以不跑——各测试脚本会在真要用时调它。
#
#   fetch-deps.sh              取全部能自动取的
#   fetch-deps.sh --kernel     只取内核树（lkmm.sh 内部用这个）
#   fetch-deps.sh --check      只报告缺什么，不装
#   参数至多一个，多给判用法错、退 1。
#
# 取内核树（fetch_kernel）：整段持 $CACHE/linux-memory-model.lock 的 flock，拿不到锁就等，至多等 SINGLEFS_KERNEL_FETCH_WAIT 秒（默认 600），
#   到点报「另一个进程正在取」、退 1；拿到锁先 clone 进 $CACHE 下 mktemp -d 建的临时目录，稀疏检出成功、tools/memory-model 在才改名换上
#   $KTREE_CACHE；失败只删临时目录，已有的 $KTREE_CACHE 不动。
#
# 弄坏开关（设进环境，只给证红用；几项用逗号隔开）：
#   FETCH_DEPS_BREAK=fetch-in-place  fetch_kernel 退回改前：不加锁，先 rm -rf $KTREE_CACHE 再往同一个路径 clone，失败也整个删掉
#       已有一份不全的 $KTREE_CACHE（里面放一份标记文件）、SINGLEFS_KERNEL_GIT 指到不存在的仓：期望退 1 而标记还在，实测标记被删；
#       两个进程相隔 1 秒同时取（假 git 在 clone 之前睡 3 秒）：期望两个都退 0、树在，实测后起的那个 clone 失败、删掉先起的那个取好的树；
#       另一个进程持着锁、SINGLEFS_KERNEL_FETCH_WAIT=1：期望退 1、报「正在取」，实测不等锁照取
#   FETCH_DEPS_BREAK=extra-arguments-ignored  参数多于一个不判，只读第一个（改前）
#       fetch-deps.sh --check --bogus：期望退 1、报用法，实测照 --check 跑完退 0
#
# 边界（rules/command-safety.md）：**本脚本不跑 sudo。**
# 系统包只打印那条命令，由人自己执行——静默改系统比缺个包危险得多。
# gate-lint:nocount 它取的是几样固定的依赖（内核树、系统包提示），不扫一批对象，成功句报的是取到的那一样在哪
# 准入经 .claude/scripts/preflight.sh 判（规范副本先找这棵树的、再找主仓的，两处都没有就拒绝；坑写在它的文件头），不直接 source 副本里的 lib.sh：
#   worktree 与 gate.sh --staged 的临时树里没有副本，直接 source 会落空，preflight 成了找不到的命令、参数被清空、不带参数照跑。
#   lib.sh（ok、bad、howto、warn、say、die、head1、project_root）取 preflight.sh 找到的那一份副本里的：它与 preflight.py 同在副本的 scripts/ 下（PREFLIGHT_PROGRAM）。
source "$(dirname "${BASH_SOURCE[0]}")/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
source "${PREFLIGHT_PROGRAM%/*}/lib.sh" || { echo "  ✗ 规范副本里没有 lib.sh：${PREFLIGHT_PROGRAM%/*}/lib.sh（preflight.py 在而 lib.sh 不在，副本装得不全）"
  echo "     → 怎么办：重装规范副本（bash .claude/singlefs-ai-sop/install.sh）再跑。"; exit 1; }

CACHE="${XDG_CACHE_HOME:-$HOME/.cache}/singlefs"
KTREE_CACHE="$CACHE/linux-memory-model"
KERNEL_FETCH_LOCK="$CACHE/linux-memory-model.lock"
KERNEL_FETCH_WAIT="${SINGLEFS_KERNEL_FETCH_WAIT:-600}"
if (($# > 1)) && [[ ",${FETCH_DEPS_BREAK:-}," != *,extra-arguments-ignored,* ]]; then
  bad "参数多了：$*（只认一个）"
  howto "用法： fetch-deps.sh [--all|--kernel|--check]"
  exit 1
fi
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
  if [[ ",${FETCH_DEPS_BREAK:-}," == *,fetch-in-place,* ]]; then fetch_kernel_in_place; return; fi
  command -v flock >/dev/null || { bad "flock 缺失：取内核树要先锁住，免得两个进程同时取互相踩"; howto "装 util-linux（Debian/Ubuntu: sudo apt install util-linux）再来。"; return 1; }
  if ! mkdir -p "$CACHE" || ! touch "$KERNEL_FETCH_LOCK"; then
    bad "建不了缓存目录 $CACHE 或锁文件 $KERNEL_FETCH_LOCK"
    howto "看 \$CACHE 有没有写权限，修好再跑。"
    return 1
  fi
  local fetch_status=0
  # 锁住整段取树：锁文件的 fd 9 只在这个子 shell 里开着，子 shell 退出就放锁；等锁到点退 75
  (
    flock -w "$KERNEL_FETCH_WAIT" 9 || exit 75
    if p="$(kernel_tree_path)"; then ok "内核树 $p（等锁期间别的进程取好了，跳过）"; exit 0; fi
    staging="$(mktemp -d "$CACHE/linux-memory-model.fetch.XXXXXX")" || { bad "在 $CACHE 下建不了临时目录"; howto "看 $CACHE 满没满、有没有写权限，修好再跑。"; exit 1; }
    trap 'rm -rf "${staging:?}"' EXIT
    warn "稀疏检出 tools/memory-model 到临时目录 $staging，成功后换上 $KTREE_CACHE（只取那一个目录，几 MB）"
    if git clone -q --depth 1 --filter=blob:none --sparse \
         "${SINGLEFS_KERNEL_GIT:-https://github.com/torvalds/linux.git}" "$staging/tree" 2>/dev/null \
       && git -C "$staging/tree" sparse-checkout set tools/memory-model >/dev/null 2>&1 \
       && [[ -d "$staging/tree/tools/memory-model" ]]; then
      # 走到这里已有的 $KTREE_CACHE 一定不全（有 tools/memory-model 的上面就跳过了）：先挪进临时目录，换上新的之后随临时目录删
      if [[ -e "$KTREE_CACHE" ]] && ! mv -T "$KTREE_CACHE" "$staging/replaced"; then
        bad "挪不开已有的 $KTREE_CACHE"; howto "看它的权限；它不全，手动删掉再跑。"; exit 1
      fi
      mv -T "$staging/tree" "$KTREE_CACHE" || { bad "换不上 $KTREE_CACHE"; howto "看 $CACHE 的权限，修好再跑。"; exit 1; }
      ok "内核树 $KTREE_CACHE（$(du -sh "$KTREE_CACHE" | cut -f1)）"
      exit 0
    fi
    bad "取内核树失败（已有的 $KTREE_CACHE 没动，临时目录已删）"
    howto "网络不通，或 git 太老（稀疏检出要 2.25+）。指一棵已有的树：" \
          "SINGLEFS_KERNEL_TREE=/path/to/linux"
    exit 1
  ) 9>"$KERNEL_FETCH_LOCK" || fetch_status=$?
  if ((fetch_status == 75)); then
    bad "另一个进程正在取内核树：等锁 $KERNEL_FETCH_LOCK 等了 $KERNEL_FETCH_WAIT 秒还没拿到"
    howto "等它取完再跑（取完之后 $KTREE_CACHE/tools/memory-model 就在，这一步直接跳过）；" \
          "要等得更久设 SINGLEFS_KERNEL_FETCH_WAIT=<秒>"
  fi
  ((fetch_status == 0))
}

# 弄坏开关 FETCH_DEPS_BREAK=fetch-in-place 走这一份（改前的取法）：不加锁，先删再往同一个路径 clone
fetch_kernel_in_place() {
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
