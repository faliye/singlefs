#!/usr/bin/env bash
# admission: always 每趟都重编一次装置、各起一次虚机，上一次的输出不替这一次作保
# run-condition: command cargo rustc qemu-system-x86_64 busybox cpio git
# run-condition: check test -d "$(rustc --print sysroot)/lib/rustlib/x86_64-unknown-linux-musl/lib" :: 装 musl 目标：rustup target add x86_64-unknown-linux-musl
# run-condition: check test -r /dev/kvm -a -w /dev/kvm :: /dev/kvm 要可读写（不降级到软件模拟）：把当前用户加进 kvm 组并重新登录，或换到有 KVM 的机器
# run-condition: check case ",${E53_VM_BENCH_RUN_BREAK:-}," in *,kernel-image-unchecked,*) exit 0;; esac; if [ -n "${SINGLEFS_KERNEL:-}" ]; then test -r "$SINGLEFS_KERNEL"; else find /boot -maxdepth 1 -name 'vmlinuz-*' -readable 2> /dev/null | grep -q .; fi :: 要有可读的内核镜像（虚机驱动 scripts/devices/vm-bench.sh 的 find_kernel 同一个判法）：设 SINGLEFS_KERNEL=<可读的内核镜像路径>，或让 /boot/vmlinuz-* 里至少一份当前用户可读，或换到有可读内核镜像的机器
# E53（丢一整块盘之后根环还挂不挂得上）的虚机那一趟：把 e53_ring_failure 编成 musl 静态二进制，送进 QEMU/KVM 虚机里跑，结果行落到输出文件。
#
#   task-server/task submit -- env VM_DISKS=4 bash scripts/experiment-apparatus/e53-vm-bench-run.sh <输出文件>
#   task-server/task submit -- env VM_DISKS=4 VM_DISK_MB=64 bash scripts/experiment-apparatus/e53-vm-bench-run.sh <输出文件>   # 每块盘 64 MB
#   bash scripts/experiment-apparatus/e53-vm-bench-run.sh --selftest
#
# e53_ring_failure 不收参数：盘的路径（四块盘 /dev/vda … /dev/vdd）由虚机里 scripts/devices/vm-bench.sh 放在参数前面传给它，复跑只给输出文件；输出文件之后多给了东西，退 2、一步都不做。
# 盘数要是四块：环境里的 VM_DISKS 不等于 4（没设也算）退 2、一步都不做；装置自己只要两块以上就照跑，不核到四块，所以要入口来核。
# 输出文件的相对路径按调用方的工作目录算（先转成绝对路径再进装置目录）；输出文件所在的目录要已存在。
#
# 做两件事，先后次序固定：
#   构建  cargo build --release --target x86_64-unknown-linux-musl --bin e53_ring_failure，编译目录由环境给（任务服务给 CARGO_TARGET_DIR），脚本不设、不猜；
#         二进制的位置取 cargo 报的这一趟编出来的那一份（long-run-lib/binary-identity.sh 的 binary_identity_build），不拼仓里 target/ 下的路径
#   虚机  bash scripts/devices/vm-bench.sh <那份二进制> <参数…>。虚机内存、vCPU、盘数、盘大小、超时、内核镜像、QEMU 配置都只读 vm-bench.sh 自己的环境变量与配置
#         （VM_MEM、VM_CPUS、VM_DISKS、VM_DISK_MB、VM_TIMEOUT=900 秒、SINGLEFS_KERNEL 等，文件头在 scripts/devices/vm-bench.sh）；
#         调用方经 `-- env <名>=<值>` 给的 VM_* 原样到 vm-bench.sh（E53 要 VM_DISKS=4），本脚本不设、不改、不挑机器
# 虚机那一趟的耗时没有登记的实测；单次虚机运行的上限是 vm-bench.sh 的 VM_TIMEOUT 默认 900 秒（超时读不到退出标记，整轮作废）。
#
# 输出文件里只有 E7RESULT 行（含收尾行 name=done emitted=N）；vm-bench.sh 的全部输出打到这一趟的标准输出（任务日志里看）。
# 三道闸（command-safety-by-review.md「结果抓取要有完整性闸」「带闸的脚本，判完了才交出产物」）：
#   删旧闸  跑前删本轮输出，不许跨轮复用；已存在的输出文件只在 HEAD 里有同名的一份、工作区这一份逐字节与它相同（git hash-object --no-filters 等于 HEAD:<路径> 的 blob）时才删，
#         别的（不在 git 里、改过没提交，索引上标了 assume-unchanged 或 skip-worktree 的照样比内容）不删、退 2，那样的文件删了就找不回
#   条数闸  抓到的 E7RESULT 行数必须等于收尾行 emitted 声称的条数（没有收尾行也判红）；
#   交付闸  虚机驱动退出码、条数闸都过了，才把结果写进输出文件，写完回读行数；任何一道判红，输出文件不存在。
#
# 退出码：
#   0   跑完，结果行落在输出文件，行数与收尾行声称的相符
#   1   虚机驱动退非 0、抓不到收尾行、条数对不上、写出的输出回读行数不符（这一趟作废，输出文件不存在）
#   2   用法不对（含输出文件之后多给了参数）、VM_DISKS 不等于 4、输出文件所在目录不在或输出路径是目录、输出文件已存在而 HEAD 里没有它、或与 HEAD 里那一份内容不同、进不去装置目录、构建失败、写不出输出文件
#   78  运行条件没满足（缺 cargo / rustc / qemu-system-x86_64 / busybox / cpio / git、没装 musl 目标、/dev/kvm 读写不了、没有可读的内核镜像）；上面 run-condition 的出路句写了怎么办
#
# 自证：bash scripts/experiment-apparatus/e53-vm-bench-run.sh --selftest
#   每一格各造一棵临时树（本脚本、binary-identity.sh 的拷贝，.claude 指向真仓的，假的 scripts/devices/vm-bench.sh）、PATH 最前面放假 cargo（不编，
#   按 --target 把假二进制放进 CARGO_TARGET_DIR 并按 cargo 的机读格式报位置）、调用时的工作目录是另一个临时目录：不起真 cargo、不起 QEMU、不碰仓里的东西。
#   SINGLEFS_KERNEL 默认指到格里的假内核镜像；缺内核那一格不给它、改在 PATH 最前面放什么也找不到的假 find。输出文件在 git 里的格在调用目录里 git init 并提交（git 往上找仓拦在格目录里）。
#   各格互不依赖，按每台默认重活线程数并行起、按固定次序汇总。
# 弄坏开关 E53_VM_BENCH_RUN_BREAK=<项>（逗号分隔；只在 --selftest 起的那几趟里认：自证设 E53_VM_BENCH_RUN_SELFTEST_MARK 指到它建的标记文件，别处设了拒、退 2）：
#   E53_VM_BENCH_RUN_BREAK=arguments-accepted  输出文件之后给了参数也收、原样传到虚机驱动
#       「输出文件之后给了参数：退 2、不起构建与虚机驱动、不留输出」
#   E53_VM_BENCH_RUN_BREAK=vm-disks-unchecked  不核 VM_DISKS 等于 4
#       「VM_DISKS 不等于 4（给 2、没设）：退 2、不起构建与虚机驱动、不留输出」
#   E53_VM_BENCH_RUN_BREAK=vm-environment-cleared  起虚机驱动之前清掉调用方给的 VM_DISKS、VM_DISK_MB、VM_IOPS、VM_DISK_PREALLOCATE
#       「调用方给的 VM_* 环境变量原样到虚机驱动」
#   E53_VM_BENCH_RUN_BREAK=binary-from-repository-target  二进制路径拼装置目录下 target/ 的位置，不取 cargo 报的
#       「二进制路径取自 cargo 在环境给的编译目录里报的位置、不是仓里的 target」
#   E53_VM_BENCH_RUN_BREAK=build-for-host-target  构建不带 --target x86_64-unknown-linux-musl
#       「构建的是 musl 目标的 release 版 e53_ring_failure」
#   E53_VM_BENCH_RUN_BREAK=output-relative-to-apparatus  输出路径不先转绝对路径，进装置目录之后按相对路径写
#       「相对路径的输出落在调用时的工作目录下、不落在装置目录下」
#   E53_VM_BENCH_RUN_BREAK=build-failure-ignored  构建失败照样往下走
#       「构建失败：不起虚机驱动、不留输出、退 2」
#   E53_VM_BENCH_RUN_BREAK=driver-exit-code-ignored  虚机驱动退非 0 照样交输出
#       「虚机驱动退非 0：不留输出、退 1」
#   E53_VM_BENCH_RUN_BREAK=result-count-unchecked  条数闸不核收尾行的 emitted 与实际条数
#       「收尾行的 emitted 与抓到的条数对不上：不留输出、退 1、点名两个数」
#   E53_VM_BENCH_RUN_BREAK=output-delivered-before-judging  先把抓到的结果写进输出文件、后判（交付闸的次序倒过来）
#       「虚机驱动退非 0：不留输出、退 1」
#       「收尾行的 emitted 与抓到的条数对不上：不留输出、退 1、点名两个数」
#   E53_VM_BENCH_RUN_BREAK=missing-output-accepted  不给输出文件时落到调用目录下的默认名，不退 2
#       「不给输出文件：退 2、打用法」
#   E53_VM_BENCH_RUN_BREAK=missing-output-directory-unchecked  开跑前不核输出文件所在的目录在不在
#       「输出文件所在的目录不存在：开跑前退 2、不起构建」
#   E53_VM_BENCH_RUN_BREAK=temporary-files-kept  不挂清理临时目录的 trap
#       「成功与失败各一趟之后，临时目录里不留东西」
#   E53_VM_BENCH_RUN_BREAK=kernel-image-unchecked  运行条件里「有可读的内核镜像」那一条不判（它写在文件头 run-condition 的命令里，命令读这个变量）
#       「没有可读的内核镜像、也没设 SINGLEFS_KERNEL：退 78、不起构建与虚机驱动、不留输出」
#   E53_VM_BENCH_RUN_BREAK=existing-output-deleted-unchecked  删旧闸不比内容，已存在的输出一律照删
#       「输出文件已存在而在 git 里没有：退 2、文件原样留着、不起构建」
#       「输出文件在 git 里而有没提交的改动：退 2、文件原样留着、不起构建」
#       「输出文件名带方括号、没进 git 而名字按通配能匹配上一份提交了的：退 2、文件原样留着、不起构建」
#       「输出文件在 git 里、标了 assume-unchanged 而有没提交的改动：退 2、文件原样留着、不起构建」
#       「输出文件在 git 里、标了 skip-worktree 而有没提交的改动：退 2、文件原样留着、不起构建」
#       「调用环境带 GIT_ICASE_PATHSPECS=1、没进 git 的输出名与提交了的一份只差大小写：退 2、文件原样留着、不起构建」
#   E53_VM_BENCH_RUN_BREAK=existing-output-judged-by-git-status  删旧闸不比内容，改信 git 报的「没差」（ls-files --error-unmatch 与 diff --quiet HEAD，认索引上的 assume-unchanged / skip-worktree 标记，不清调用环境里的 GIT_ICASE_PATHSPECS）
#       「输出文件在 git 里、标了 assume-unchanged 而有没提交的改动：退 2、文件原样留着、不起构建」
#       「输出文件在 git 里、标了 skip-worktree 而有没提交的改动：退 2、文件原样留着、不起构建」
#       「调用环境带 GIT_ICASE_PATHSPECS=1、没进 git 的输出名与提交了的一份只差大小写：退 2、文件原样留着、不起构建」
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
APPARATUS_DIRECTORY="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCRIPT_PATH="$APPARATUS_DIRECTORY/$(basename "${BASH_SOURCE[0]}")"
SELFTEST_MARK_TEXT="e53-vm-bench-run-selftest"
MUSL_TARGET="x86_64-unknown-linux-musl"
USAGE_LINE="用法：e53-vm-bench-run.sh <输出文件>"
# shellcheck source=long-run-lib/binary-identity.sh
source "$APPARATUS_DIRECTORY/long-run-lib/binary-identity.sh"

# 弄坏开关只在自证起的那几趟里认（标记文件在、内容对得上）；别处设了就拒。
break_is() {
  [[ ",${E53_VM_BENCH_RUN_BREAK:-}," == *",$1,"* ]]
}
if [[ -n "${E53_VM_BENCH_RUN_BREAK:-}" && "${1:-}" != --selftest ]] \
   && [[ -z "${E53_VM_BENCH_RUN_SELFTEST_MARK:-}" || "$(cat -- "${E53_VM_BENCH_RUN_SELFTEST_MARK}" 2> /dev/null)" != "$SELFTEST_MARK_TEXT" ]]; then
  echo "  ✗ E53_VM_BENCH_RUN_BREAK 只给 --selftest 证红用，这一趟不是自证起的" >&2
  echo "     → 怎么办：平常跑时不设它" >&2
  exit 2
fi

# ── 自证 ──────────────────────────────────────────────────────────────

# 一格的世界：临时树（本脚本与库的拷贝、指向真仓的 .claude、假虚机驱动）、假 cargo、调用目录、编译目录、临时目录；
# 调用方的 VM_* 先清掉，VM_DISKS 设成 4（入口只认四块盘），各格要别的在这之后设
selftest_world() { # <格目录>
  local cell="$1"
  unset VM_DISK_MB VM_IOPS VM_DISK_PREALLOCATE
  export VM_DISKS=4   # E53 的入口只认四块盘；要别的盘数的格自己改
  mkdir -p -- "$cell/tree/scripts/experiment-apparatus/long-run-lib" "$cell/tree/scripts/devices" "$cell/fakes" "$cell/caller" "$cell/cargo-target" "$cell/tmp"
  cp -- "$SCRIPT_PATH" "$cell/tree/scripts/experiment-apparatus/e53-vm-bench-run.sh"
  cp -- "$APPARATUS_DIRECTORY/long-run-lib/binary-identity.sh" "$cell/tree/scripts/experiment-apparatus/long-run-lib/binary-identity.sh"
  ln -s -- "$(cd "$APPARATUS_DIRECTORY/../.." && pwd)/.claude" "$cell/tree/.claude"
  echo "$SELFTEST_MARK_TEXT" > "$cell/mark"
  : > "$cell/kernel"   # 假的可读内核镜像：运行条件里「有可读的内核镜像」靠 SINGLEFS_KERNEL 指到它
  cat > "$cell/fakes/cargo" <<'FAKE_CARGO'
#!/usr/bin/env bash
# 假 cargo：build 记下参数；E53_SELFTEST_CARGO=fail 时编不过；否则像真 cargo 那样按 --target 把假二进制放进 ${CARGO_TARGET_DIR}[/<目标>]/release，并报它的位置
[[ "${1:-}" == build ]] || exit 0
printf '%s\n' "$*" >> "$E53_SELFTEST_CARGO_CALLS"
if [[ "${E53_SELFTEST_CARGO:-ok}" == fail ]]; then echo "error: could not compile e53_ring_failure (fake)" >&2; exit 101; fi
target=""; previous=""
for argument in "$@"; do
  [[ "$previous" == --target ]] && target="$argument"
  previous="$argument"
done
directory="$CARGO_TARGET_DIR${target:+/$target}/release"
mkdir -p -- "$directory"
printf '#!/bin/sh\nexit 0\n' > "$directory/e53_ring_failure"
chmod +x "$directory/e53_ring_failure"
printf '%s\n' "$directory/e53_ring_failure" > "$E53_SELFTEST_REPORTED"
printf '{"reason":"compiler-artifact","target":{"name":"e53_ring_failure"},"executable":"%s/e53_ring_failure"}\n' "$directory"
FAKE_CARGO
  cat > "$cell/tree/scripts/devices/vm-bench.sh" <<'FAKE_DRIVER'
#!/usr/bin/env bash
# 假虚机驱动：每个参数一行记下来、VM_* 四个变量记成一行；E53_SELFTEST_VM=ok 打两行结果（收尾行报 2 条）、short 打两行而收尾行报 3 条、fail 打两行结果之后退 1
printf '%s\n' "$@" > "$E53_SELFTEST_VM_CALLS"
printf 'VM_DISKS=%s VM_DISK_MB=%s VM_IOPS=%s VM_DISK_PREALLOCATE=%s\n' "${VM_DISKS-unset}" "${VM_DISK_MB-unset}" "${VM_IOPS-unset}" "${VM_DISK_PREALLOCATE-unset}" > "$E53_SELFTEST_VM_ENVIRONMENT"
echo ""; echo "══ 虚机 benchmark harness ══"
case "${E53_SELFTEST_VM:-ok}" in
  ok) printf 'E7RESULT name=row selftest=1\nE7RESULT name=done emitted=2\n'; echo "  ✓ 退出码 0，抓到 2 条结果（与程序声称的 2 条相符）" ;;
  short) printf 'E7RESULT name=row selftest=1\nE7RESULT name=done emitted=3\n' ;;
  fail) printf 'E7RESULT name=row selftest=1\nE7RESULT name=done emitted=2\n'; echo "  ✗ 退出码 7"; exit 1 ;;
esac
exit 0
FAKE_DRIVER
  chmod +x "$cell/fakes/cargo" "$cell/tree/scripts/devices/vm-bench.sh"
}

# 在调用目录里起树里那一份；退出码留在 exit_code，输出在 <格目录>/invoke.log（同一格里再起一趟会盖掉上一趟的）。
# SINGLEFS_KERNEL 默认指到格里那份假内核镜像；E53_SELFTEST_KERNEL=absent 时不给它（运行条件改去 PATH 最前面的假 find 看 /boot）。
# GIT_CEILING_DIRECTORIES 把 git 往上找仓的范围拦在格目录里：调用目录不是仓时，输出文件就是「不在 git 里」
selftest_invoke() { # <格目录> [脚本的参数…]
  local cell="$1" kernel_option=()
  shift
  if [[ "${E53_SELFTEST_KERNEL:-given}" == absent ]]; then kernel_option=(-u SINGLEFS_KERNEL); else kernel_option=(SINGLEFS_KERNEL="$cell/kernel"); fi
  if ( cd -- "$cell/caller" && env "${kernel_option[@]}" GIT_CEILING_DIRECTORIES="$cell" PATH="$cell/fakes:$PATH" TMPDIR="$cell/tmp" CARGO_TARGET_DIR="$cell/cargo-target" \
         E53_SELFTEST_CARGO_CALLS="$cell/cargo-calls" E53_SELFTEST_VM_CALLS="$cell/vm-calls" E53_SELFTEST_VM_ENVIRONMENT="$cell/vm-environment" \
         E53_SELFTEST_REPORTED="$cell/reported-binary" E53_VM_BENCH_RUN_SELFTEST_MARK="$cell/mark" \
         bash "$cell/tree/scripts/experiment-apparatus/e53-vm-bench-run.sh" "$@" > "$cell/invoke.log" 2>&1 ); then
    exit_code=0
  else
    exit_code=$?
  fi
}

# 一格的判定：条件成立打一行 ✓，不成立打 ✗ 与出路，返回 1
selftest_expect() { # <格目录> <格名> <条件>
  local cell="$1"
  if eval "$3"; then
    echo "  ✓ $2"
    return 0
  fi
  echo "  ✗ $2：条件「$3」不成立；输出尾部 $(tail -n 4 -- "$cell/invoke.log" 2> /dev/null | tr '\n' ' ')"
  echo "     → 怎么办：照这一格点名的条件改本脚本，再单独起 bash scripts/experiment-apparatus/e53-vm-bench-run.sh --selftest 到它退 0"
  return 1
}

# 把调用目录变成 git 仓、把一份输出文件提交进去（用户与全局配置都不读，提交者写死）
selftest_commit_output() { # <格目录> <输出文件名> <内容>
  local cell="$1"
  printf '%s\n' "$3" > "$cell/caller/$2"
  ( cd -- "$cell/caller" && export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null \
      && git init -q . && git add -- "$2" && git -c user.name=selftest -c user.email=selftest@example.invalid commit -q -m selftest ) > "$cell/git.log" 2>&1
}

# 把一份输出提交进去、在索引上给它标 <标记>（--assume-unchanged 或 --skip-worktree），再改它：标了之后 git 的 ls-files 与 diff 照样报它「没差」
selftest_commit_then_modify_under_index_flag() { # <格目录> <输出文件名> <标记>
  local cell="$1"
  selftest_commit_output "$cell" "$2" "提交了的输出" || return 1
  ( cd -- "$cell/caller" && GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null git update-index "$3" -- "$2" ) >> "$cell/git.log" 2>&1 || return 1
  echo "标了索引标记之后改过的输出" > "$cell/caller/$2"
}

cell_no_arguments() { # <格目录>
  local cell="$1" exit_code
  selftest_world "$cell"
  selftest_invoke "$cell" e53.out
  selftest_expect "$cell" "不给参数：虚机驱动只收到二进制、没有别的参数" \
    '((exit_code == 0)) && [[ "$(wc -l < "$cell/vm-calls")" == 1 ]]'
}

cell_arguments_refused() { # <格目录>
  local cell="$1" exit_code
  selftest_world "$cell"
  selftest_invoke "$cell" e53.out selfcheck
  selftest_expect "$cell" "输出文件之后给了参数：退 2、不起构建与虚机驱动、不留输出" \
    '((exit_code == 2)) && [[ ! -e "$cell/cargo-calls" && ! -e "$cell/vm-calls" && ! -e "$cell/caller/e53.out" ]] && grep -qF "'"$USAGE_LINE"'" "$cell/invoke.log"'
}

cell_vm_disks_not_four() { # <格目录>
  local cell="$1" exit_code two_disks_exit
  selftest_world "$cell"
  export VM_DISKS=2
  selftest_invoke "$cell" e53.out
  two_disks_exit=$exit_code
  unset VM_DISKS
  selftest_invoke "$cell" e53.out
  selftest_expect "$cell" "VM_DISKS 不等于 4（给 2、没设）：退 2、不起构建与虚机驱动、不留输出" \
    '((two_disks_exit == 2 && exit_code == 2)) && [[ ! -e "$cell/cargo-calls" && ! -e "$cell/vm-calls" && ! -e "$cell/caller/e53.out" ]] && grep -qF "VM_DISKS 要等于 4，现在是「没设」" "$cell/invoke.log"'
}

cell_environment_passthrough() { # <格目录>
  local cell="$1" exit_code
  selftest_world "$cell"
  export VM_DISKS=4 VM_DISK_MB=64 VM_IOPS=4000 VM_DISK_PREALLOCATE=1
  selftest_invoke "$cell" e53.out
  selftest_expect "$cell" "调用方给的 VM_* 环境变量原样到虚机驱动" \
    '((exit_code == 0)) && [[ "$(cat -- "$cell/vm-environment")" == "VM_DISKS=4 VM_DISK_MB=64 VM_IOPS=4000 VM_DISK_PREALLOCATE=1" ]]'
}

cell_binary_from_build_directory() { # <格目录>
  local cell="$1" exit_code
  selftest_world "$cell"
  selftest_invoke "$cell" e53.out
  selftest_expect "$cell" "二进制路径取自 cargo 在环境给的编译目录里报的位置、不是仓里的 target" \
    '((exit_code == 0)) && [[ "$(head -n 1 -- "$cell/vm-calls")" == "$(cat -- "$cell/reported-binary")" && "$(head -n 1 -- "$cell/vm-calls")" == "$cell/cargo-target/"* ]]'
}

cell_musl_release_build() { # <格目录>
  local cell="$1" exit_code
  selftest_world "$cell"
  selftest_invoke "$cell" e53.out
  selftest_expect "$cell" "构建的是 musl 目标的 release 版 e53_ring_failure" \
    '((exit_code == 0)) && grep -qF -- "--release" "$cell/cargo-calls" && grep -qF -- "--target x86_64-unknown-linux-musl" "$cell/cargo-calls" && grep -qF -- "--bin e53_ring_failure" "$cell/cargo-calls"'
}

cell_relative_output() { # <格目录>
  local cell="$1" exit_code
  selftest_world "$cell"
  selftest_invoke "$cell" e53-relative.out
  selftest_expect "$cell" "相对路径的输出落在调用时的工作目录下、不落在装置目录下" \
    '((exit_code == 0)) && [[ -f "$cell/caller/e53-relative.out" && ! -e "$cell/tree/scripts/experiment-apparatus/e53-relative.out" ]] && (($(grep -c "^E7RESULT " "$cell/caller/e53-relative.out") == 2))'
}

cell_build_failure() { # <格目录>
  local cell="$1" exit_code
  selftest_world "$cell"
  export E53_SELFTEST_CARGO=fail
  selftest_invoke "$cell" e53.out
  selftest_expect "$cell" "构建失败：不起虚机驱动、不留输出、退 2" \
    '((exit_code == 2)) && [[ ! -e "$cell/vm-calls" && ! -e "$cell/caller/e53.out" ]] && grep -qF "e53_ring_failure 构建失败" "$cell/invoke.log"'
}

cell_driver_failure() { # <格目录>
  local cell="$1" exit_code committed=0
  selftest_world "$cell"
  export E53_SELFTEST_VM=fail
  selftest_commit_output "$cell" e53.out "上一趟留下的输出" || committed=$?   # 跑前删旧闸：在 git 里的旧输出照删，这一趟作废时它也不在
  selftest_invoke "$cell" e53.out
  selftest_expect "$cell" "虚机驱动退非 0：不留输出、退 1" \
    '((committed == 0 && exit_code == 1)) && [[ ! -e "$cell/caller/e53.out" ]] && grep -qF "虚机驱动退出码 1" "$cell/invoke.log"'
}

cell_existing_output_not_in_git() { # <格目录>
  local cell="$1" exit_code
  selftest_world "$cell"
  echo "还没进 git 的上一趟输出" > "$cell/caller/e53.out"
  selftest_invoke "$cell" e53.out
  selftest_expect "$cell" "输出文件已存在而在 git 里没有：退 2、文件原样留着、不起构建" \
    '((exit_code == 2)) && [[ "$(cat -- "$cell/caller/e53.out")" == "还没进 git 的上一趟输出" && ! -e "$cell/cargo-calls" && ! -e "$cell/vm-calls" ]] && grep -qF "换一个名字，或先挪走" "$cell/invoke.log"'
}

cell_existing_output_modified_after_commit() { # <格目录>
  local cell="$1" exit_code committed=0
  selftest_world "$cell"
  selftest_commit_output "$cell" e53.out "提交了的输出" || committed=$?
  echo "提交之后改过的输出" > "$cell/caller/e53.out"
  selftest_invoke "$cell" e53.out
  selftest_expect "$cell" "输出文件在 git 里而有没提交的改动：退 2、文件原样留着、不起构建" \
    '((committed == 0 && exit_code == 2)) && [[ "$(cat -- "$cell/caller/e53.out")" == "提交之后改过的输出" && ! -e "$cell/cargo-calls" && ! -e "$cell/vm-calls" ]] && grep -qF "换一个名字，或先挪走" "$cell/invoke.log"'
}

cell_existing_output_name_with_brackets() { # <格目录>
  local cell="$1" exit_code committed=0
  selftest_world "$cell"
  selftest_commit_output "$cell" e53-r2.out "提交了的输出" || committed=$?   # 按通配 e53-r[2].out 会匹配到这一份
  echo "还没进 git 的方括号名输出" > "$cell/caller/e53-r[2].out"
  selftest_invoke "$cell" 'e53-r[2].out'
  selftest_expect "$cell" "输出文件名带方括号、没进 git 而名字按通配能匹配上一份提交了的：退 2、文件原样留着、不起构建" \
    '((committed == 0 && exit_code == 2)) && [[ "$(cat -- "$cell/caller/e53-r[2].out")" == "还没进 git 的方括号名输出" && ! -e "$cell/cargo-calls" && ! -e "$cell/vm-calls" ]] && grep -qF "换一个名字，或先挪走" "$cell/invoke.log"'
}

cell_existing_output_modified_under_assume_unchanged() { # <格目录>
  local cell="$1" exit_code prepared=0
  selftest_world "$cell"
  selftest_commit_then_modify_under_index_flag "$cell" e53.out --assume-unchanged || prepared=$?
  selftest_invoke "$cell" e53.out
  selftest_expect "$cell" "输出文件在 git 里、标了 assume-unchanged 而有没提交的改动：退 2、文件原样留着、不起构建" \
    '((prepared == 0 && exit_code == 2)) && [[ "$(cat -- "$cell/caller/e53.out")" == "标了索引标记之后改过的输出" && ! -e "$cell/cargo-calls" && ! -e "$cell/vm-calls" ]] && grep -qF "换一个名字，或先挪走" "$cell/invoke.log"'
}

cell_existing_output_modified_under_skip_worktree() { # <格目录>
  local cell="$1" exit_code prepared=0
  selftest_world "$cell"
  selftest_commit_then_modify_under_index_flag "$cell" e53.out --skip-worktree || prepared=$?
  selftest_invoke "$cell" e53.out
  selftest_expect "$cell" "输出文件在 git 里、标了 skip-worktree 而有没提交的改动：退 2、文件原样留着、不起构建" \
    '((prepared == 0 && exit_code == 2)) && [[ "$(cat -- "$cell/caller/e53.out")" == "标了索引标记之后改过的输出" && ! -e "$cell/cargo-calls" && ! -e "$cell/vm-calls" ]] && grep -qF "换一个名字，或先挪走" "$cell/invoke.log"'
}

cell_existing_output_case_variant_under_icase_environment() { # <格目录>
  local cell="$1" exit_code committed=0
  selftest_world "$cell"
  selftest_commit_output "$cell" e53.out "提交了的输出" || committed=$?   # 带 GIT_ICASE_PATHSPECS=1 时按 pathspec 认，E53.out 会匹配到这一份
  echo "还没进 git 的大小写变体输出" > "$cell/caller/E53.out"
  export GIT_ICASE_PATHSPECS=1
  selftest_invoke "$cell" E53.out
  selftest_expect "$cell" "调用环境带 GIT_ICASE_PATHSPECS=1、没进 git 的输出名与提交了的一份只差大小写：退 2、文件原样留着、不起构建" \
    '((committed == 0 && exit_code == 2)) && [[ "$(cat -- "$cell/caller/E53.out")" == "还没进 git 的大小写变体输出" && "$(cat -- "$cell/caller/e53.out")" == "提交了的输出" && ! -e "$cell/cargo-calls" && ! -e "$cell/vm-calls" ]] && grep -qF "换一个名字，或先挪走" "$cell/invoke.log"'
}

cell_kernel_image_missing() { # <格目录>
  local cell="$1" exit_code
  selftest_world "$cell"
  export E53_SELFTEST_KERNEL=absent
  printf '%s\n' '#!/usr/bin/env bash' 'exit 0' > "$cell/fakes/find"   # 假 find 什么也找不到：/boot 下没有可读的内核镜像
  chmod +x "$cell/fakes/find"
  selftest_invoke "$cell" e53.out
  selftest_expect "$cell" "没有可读的内核镜像、也没设 SINGLEFS_KERNEL：退 78、不起构建与虚机驱动、不留输出" \
    '((exit_code == 78)) && [[ ! -e "$cell/cargo-calls" && ! -e "$cell/vm-calls" && ! -e "$cell/caller/e53.out" ]] && grep -qF "SINGLEFS_KERNEL" "$cell/invoke.log"'
}

cell_result_count_mismatch() { # <格目录>
  local cell="$1" exit_code
  selftest_world "$cell"
  export E53_SELFTEST_VM=short
  selftest_invoke "$cell" e53.out
  selftest_expect "$cell" "收尾行的 emitted 与抓到的条数对不上：不留输出、退 1、点名两个数" \
    '((exit_code == 1)) && [[ ! -e "$cell/caller/e53.out" ]] && grep -qF "条数对不上：抓到 2，收尾行说 3" "$cell/invoke.log"'
}

cell_missing_output() { # <格目录>
  local cell="$1" exit_code
  selftest_world "$cell"
  selftest_invoke "$cell"
  selftest_expect "$cell" "不给输出文件：退 2、打用法" \
    '((exit_code == 2)) && grep -qF "'"$USAGE_LINE"'" "$cell/invoke.log"'
}

cell_missing_output_directory() { # <格目录>
  local cell="$1" exit_code
  selftest_world "$cell"
  selftest_invoke "$cell" no-such-directory/e53.out
  selftest_expect "$cell" "输出文件所在的目录不存在：开跑前退 2、不起构建" \
    '((exit_code == 2)) && [[ ! -e "$cell/cargo-calls" && ! -e "$cell/vm-calls" ]]'
}

cell_no_temporary_files() { # <格目录>
  local cell="$1" exit_code
  selftest_world "$cell"
  selftest_invoke "$cell" e53.out
  E53_SELFTEST_VM=fail selftest_invoke "$cell" e53-failed.out
  selftest_expect "$cell" "成功与失败各一趟之后，临时目录里不留东西" \
    '[[ -z "$(ls -A -- "$cell/tmp")" ]]'
}

# 一格放进后台之前：同时在跑的已满每台默认重活线程数就先等走一格；建这一格的目录（cell_dir）、记下它的名字。
# 用 run_selftest 的局部变量（work、threads、running、dispatched_cells、cell_dir）
selftest_begin_cell() { # <格名>
  if ((running >= threads)); then
    wait -n
    running=$((running - 1))
  fi
  running=$((running + 1))
  cell_dir="$work/$1"
  mkdir -p -- "$cell_dir"
  dispatched_cells+=("$1")
}

# 各格的函数名都写成字面的命令词、一格一行（不经变量当命令词：门禁的算键要从这几行认出每一格调了哪个函数）；
# 每格放后台、输出与退出码各写进自己的 out.txt 与 status
run_selftest() {
  local work threads failures=0 passed=0 name running=0 dispatched=0 collected=0 defined_names dispatched_names cell_dir
  local -a dispatched_cells=()
  work=$(mktemp -d "${TMPDIR:-/tmp}/singlefs-e53vmself.XXXXXX") || return 1
  SELFTEST_WORK="$work"
  trap 'rm -rf -- "${SELFTEST_WORK:?}"' EXIT
  # shellcheck source=../hosts/heavy-threads.sh
  source "$APPARATUS_DIRECTORY/../hosts/heavy-threads.sh"
  threads=$(heavy_thread_default) || return 2
  selftest_begin_cell cell_no_arguments; { cell_no_arguments "$cell_dir" > "$cell_dir/out.txt" 2>&1; echo "$?" > "$cell_dir/status"; } &
  selftest_begin_cell cell_arguments_refused; { cell_arguments_refused "$cell_dir" > "$cell_dir/out.txt" 2>&1; echo "$?" > "$cell_dir/status"; } &
  selftest_begin_cell cell_vm_disks_not_four; { cell_vm_disks_not_four "$cell_dir" > "$cell_dir/out.txt" 2>&1; echo "$?" > "$cell_dir/status"; } &
  selftest_begin_cell cell_environment_passthrough; { cell_environment_passthrough "$cell_dir" > "$cell_dir/out.txt" 2>&1; echo "$?" > "$cell_dir/status"; } &
  selftest_begin_cell cell_binary_from_build_directory; { cell_binary_from_build_directory "$cell_dir" > "$cell_dir/out.txt" 2>&1; echo "$?" > "$cell_dir/status"; } &
  selftest_begin_cell cell_musl_release_build; { cell_musl_release_build "$cell_dir" > "$cell_dir/out.txt" 2>&1; echo "$?" > "$cell_dir/status"; } &
  selftest_begin_cell cell_relative_output; { cell_relative_output "$cell_dir" > "$cell_dir/out.txt" 2>&1; echo "$?" > "$cell_dir/status"; } &
  selftest_begin_cell cell_build_failure; { cell_build_failure "$cell_dir" > "$cell_dir/out.txt" 2>&1; echo "$?" > "$cell_dir/status"; } &
  selftest_begin_cell cell_driver_failure; { cell_driver_failure "$cell_dir" > "$cell_dir/out.txt" 2>&1; echo "$?" > "$cell_dir/status"; } &
  selftest_begin_cell cell_result_count_mismatch; { cell_result_count_mismatch "$cell_dir" > "$cell_dir/out.txt" 2>&1; echo "$?" > "$cell_dir/status"; } &
  selftest_begin_cell cell_missing_output; { cell_missing_output "$cell_dir" > "$cell_dir/out.txt" 2>&1; echo "$?" > "$cell_dir/status"; } &
  selftest_begin_cell cell_missing_output_directory; { cell_missing_output_directory "$cell_dir" > "$cell_dir/out.txt" 2>&1; echo "$?" > "$cell_dir/status"; } &
  selftest_begin_cell cell_no_temporary_files; { cell_no_temporary_files "$cell_dir" > "$cell_dir/out.txt" 2>&1; echo "$?" > "$cell_dir/status"; } &
  selftest_begin_cell cell_existing_output_not_in_git; { cell_existing_output_not_in_git "$cell_dir" > "$cell_dir/out.txt" 2>&1; echo "$?" > "$cell_dir/status"; } &
  selftest_begin_cell cell_existing_output_modified_after_commit; { cell_existing_output_modified_after_commit "$cell_dir" > "$cell_dir/out.txt" 2>&1; echo "$?" > "$cell_dir/status"; } &
  selftest_begin_cell cell_existing_output_name_with_brackets; { cell_existing_output_name_with_brackets "$cell_dir" > "$cell_dir/out.txt" 2>&1; echo "$?" > "$cell_dir/status"; } &
  selftest_begin_cell cell_existing_output_modified_under_assume_unchanged; { cell_existing_output_modified_under_assume_unchanged "$cell_dir" > "$cell_dir/out.txt" 2>&1; echo "$?" > "$cell_dir/status"; } &
  selftest_begin_cell cell_existing_output_modified_under_skip_worktree; { cell_existing_output_modified_under_skip_worktree "$cell_dir" > "$cell_dir/out.txt" 2>&1; echo "$?" > "$cell_dir/status"; } &
  selftest_begin_cell cell_existing_output_case_variant_under_icase_environment; { cell_existing_output_case_variant_under_icase_environment "$cell_dir" > "$cell_dir/out.txt" 2>&1; echo "$?" > "$cell_dir/status"; } &
  selftest_begin_cell cell_kernel_image_missing; { cell_kernel_image_missing "$cell_dir" > "$cell_dir/out.txt" 2>&1; echo "$?" > "$cell_dir/status"; } &
  wait
  dispatched=${#dispatched_cells[@]}
  for name in "${dispatched_cells[@]}"; do
    if [[ -e "$work/$name/status" ]]; then collected=$((collected + 1)); fi
    cat -- "$work/$name/out.txt" 2> /dev/null
    if [[ "$(cat -- "$work/$name/status" 2> /dev/null)" != 0 ]]; then failures=$((failures + 1)); else passed=$((passed + 1)); fi
  done
  defined_names="$(compgen -A function cell_ | sort | tr '\n' ' ')"
  dispatched_names="$(printf '%s\n' "${dispatched_cells[@]}" | sort | tr '\n' ' ')"
  if ((collected != dispatched)) || [[ "$defined_names" != "$dispatched_names" ]]; then
    echo "  ✗ e53-vm-bench-run.sh 自证：派出 $dispatched 格、收回 $collected 格；定义了的格是「$defined_names」，派出的是「$dispatched_names」"
    echo "     → 怎么办：看 run_selftest 里一格一行的派活：每个 cell_ 函数都要有一行，每行的格名与调的函数同名；没写出 status 的格是被打断了"
    return 1
  fi
  if ((failures > 0)); then
    echo "  ✗ e53-vm-bench-run.sh 自证：$failures 格红（共 $dispatched 格，并行 $threads 路）"
    echo "     → 怎么办：看上面红的那几格各自的出路；E53_VM_BENCH_RUN_BREAK 设着的话点名的那一格本来就该红"
    return 1
  fi
  echo "  ✓ e53-vm-bench-run.sh 自证：$passed 格全绿（共 $dispatched 格，并行 $threads 路）"
  return 0
}
if [[ "${1:-}" == --selftest ]]; then
  run_selftest
  exit $?
fi

# ── 正文 ──────────────────────────────────────────────────────────────

if [[ -z "${1:-}" ]] && ! break_is missing-output-accepted; then
  echo "$USAGE_LINE"; echo "→ 怎么办：照文件头的用法给一个输出文件路径（相对路径按调用方的工作目录算）"; exit 2
fi
output_file="${1:-e53-vm-bench.out}"
if (($# > 0)); then shift; fi
# 先转绝对路径再进装置目录：相对路径按调用方的工作目录算，不落到装置目录底下
if ! break_is output-relative-to-apparatus; then
  case "$output_file" in /*) ;; *) output_file="$PWD/$output_file" ;; esac
fi

# e53_ring_failure 不收参数、盘数要是四块：开跑前核，不合的什么都没做
if (($# > 0)) && ! break_is arguments-accepted; then
  echo "$USAGE_LINE"; echo "→ 怎么办：输出文件之后什么都不给（e53_ring_failure 不收参数，盘的路径由虚机驱动放在它前面）"; exit 2
fi
if [[ "${VM_DISKS:-}" != 4 ]] && ! break_is vm-disks-unchecked; then
  echo "  ✗ E53 要四块盘：VM_DISKS 要等于 4，现在是「${VM_DISKS:-没设}」，这一趟什么都没做"
  echo "     → 怎么办：照文件头的用法起：task-server/task submit -- env VM_DISKS=4 bash scripts/experiment-apparatus/e53-vm-bench-run.sh <输出文件>"; exit 2
fi
vm_arguments=("$@")

output_directory="$(dirname -- "$output_file")"
if [[ -d "$output_file" ]]; then
  echo "  ✗ 输出路径是个目录：$output_file"; echo "     → 怎么办：给一个文件路径"; exit 2
fi
if [[ ! -d "$output_directory" ]] && ! break_is missing-output-directory-unchecked; then
  echo "  ✗ 输出文件所在的目录不存在：$output_directory"; echo "     → 怎么办：先建好这个目录，或给一个已有目录下的路径；构建与虚机都还没起"; exit 2
fi
# 删旧闸：跑前删本轮输出。已存在的输出文件只有 HEAD 里有同名的一份、工作区这一份逐字节与它相同（删了凭 git show HEAD:<路径> 原样找得回）时才删；别的拒，不碰它。
# 按内容比、不信 git 报的「有没有改动」：索引上标了 assume-unchanged 或 skip-worktree 的文件改过没提交，ls-files 与 diff 照样报「没差」。
# 两条 git 都不经 pathspec（HEAD:./<名> 按树里的路径逐字认，名字里的 [ ] * ? 不当通配、大小写不折），起之前清掉调用环境里的 GIT_ICASE_PATHSPECS；
# hash-object 带 --no-filters：比的是盘上的原字节，不经 autocrlf 与 clean 过滤
if [[ -e "$output_file" || -L "$output_file" ]] && ! break_is existing-output-deleted-unchecked; then
  output_name="$(basename -- "$output_file")"
  output_matches_head=false
  if break_is existing-output-judged-by-git-status; then
    if git -C "$output_directory" ls-files --error-unmatch -- ":(literal)$output_name" > /dev/null 2>&1 \
       && git -C "$output_directory" diff --quiet HEAD -- ":(literal)$output_name" > /dev/null 2>&1; then output_matches_head=true; fi
  else
    committed_blob="$(env -u GIT_ICASE_PATHSPECS git -C "$output_directory" rev-parse --verify --quiet "HEAD:./$output_name" 2> /dev/null)"   # HEAD 里没有它、或不在仓里时为空
    worktree_blob="$(env -u GIT_ICASE_PATHSPECS git -C "$output_directory" hash-object --no-filters -- "$output_name" 2> /dev/null)"   # 读不了（悬空的符号链接这类）时为空
    if [[ -n "$committed_blob" && "$worktree_blob" == "$committed_blob" ]]; then output_matches_head=true; fi
  fi
  if [[ "$output_matches_head" != true ]]; then
    echo "  ✗ 输出文件已存在，而 HEAD 里没有它、或与 HEAD 里那一份内容不同（改过没提交的，索引上标了 assume-unchanged / skip-worktree 也算改过）：$output_file，删了就找不回，这一趟什么都没做"
    echo "     → 怎么办：换一个名字，或先挪走；进了 git、与 HEAD 里那一份逐字节相同的输出才会被删"
    exit 2
  fi
fi
rm -f -- "$output_file"

cd -- "$APPARATUS_DIRECTORY" || { echo "  ✗ 进不去装置目录 $APPARATUS_DIRECTORY"; echo "     → 怎么办：从仓根照文件头的用法起"; exit 2; }
TMP="$(mktemp -d "${TMPDIR:-/tmp}/singlefs-e53vm.XXXXXX")" || { echo "  ✗ 建不出临时目录（${TMPDIR:-/tmp} 下）"; echo "     → 怎么办：看 ${TMPDIR:-/tmp} 满没满、可不可写"; exit 2; }
if ! break_is temporary-files-kept; then trap 'rm -rf -- "${TMP:?}"' EXIT; fi

build_arguments=(--release --target "$MUSL_TARGET" --manifest-path Cargo.toml --bin e53_ring_failure)
if break_is build-for-host-target; then build_arguments=(--release --manifest-path Cargo.toml --bin e53_ring_failure); fi
if ! binary_identity_build e53_ring_failure "${build_arguments[@]}"; then
  if ! break_is build-failure-ignored; then
    echo "  ✗ e53_ring_failure 构建失败，或编过了 cargo 没报出它编在哪（上面是 cargo 与构建库的输出）"
    echo "     → 怎么办：没装 musl 目标的先 rustup target add $MUSL_TARGET；别的照上面的编译错误改，再重交 task-server/task submit -- env VM_DISKS=4 bash scripts/experiment-apparatus/e53-vm-bench-run.sh <输出文件>"
    exit 2
  fi
fi
binary="$BUILT_BINARY"
if break_is binary-from-repository-target; then binary="$APPARATUS_DIRECTORY/target/$MUSL_TARGET/release/e53_ring_failure"; fi

if break_is vm-environment-cleared; then unset VM_DISKS VM_DISK_MB VM_IOPS VM_DISK_PREALLOCATE; fi
if bash "$APPARATUS_DIRECTORY/../devices/vm-bench.sh" "$binary" ${vm_arguments[@]+"${vm_arguments[@]}"} > "$TMP/vm.log" 2>&1; then driver_exit=0; else driver_exit=$?; fi
cat -- "$TMP/vm.log"
grep -a '^E7RESULT ' "$TMP/vm.log" > "$TMP/result.txt"   # 没有结果行时 grep 退 1，下面按条数 0 判
got=$(wc -l < "$TMP/result.txt")
if break_is output-delivered-before-judging; then cat -- "$TMP/result.txt" > "$output_file"; fi

if ((driver_exit != 0)) && ! break_is driver-exit-code-ignored; then
  echo "  ✗ 虚机驱动退出码 $driver_exit，这一趟作废、不留输出（vm-bench.sh 的输出在上面）"
  echo "     → 怎么办：这是虚机驱动或被测程序自己的失败，看上面 vm-bench.sh 打出的控制台尾部定位；修好后重交 task-server/task submit -- env VM_DISKS=4 bash scripts/experiment-apparatus/e53-vm-bench-run.sh <输出文件>"
  exit 1
fi
declared=$(sed -n 's/^E7RESULT .*name=done emitted=\([0-9][0-9]*\).*/\1/p' "$TMP/result.txt" | tail -n 1)
if ! break_is result-count-unchecked; then
  if [[ -z "$declared" ]]; then
    echo "  ✗ 抓到 $got 条结果行，但没有收尾行 name=done，判定不明，这一趟作废、不留输出"
    echo "     → 怎么办：被测程序要在收尾时发 E7RESULT name=done emitted=<总条数>；看上面 vm-bench.sh 的输出确认它发了、格式对"
    exit 1
  fi
  if [[ "$got" != "$declared" ]]; then
    echo "  ✗ 条数对不上：抓到 $got，收尾行说 $declared，这一趟作废、不留输出"
    echo "     → 怎么办：控制台吞了结果或某行被转义序列顶掉了行首；看上面 vm-bench.sh 的输出，修好抓取后重交"
    exit 1
  fi
fi

cat -- "$TMP/result.txt" > "$output_file" || { echo "  ✗ 写不出输出文件 $output_file"; echo "     → 怎么办：核对目录可写、盘没满，再重交"; exit 2; }
delivered=$(wc -l < "$output_file")
if [[ "$delivered" != "$got" ]]; then
  rm -f -- "$output_file"
  echo "  ✗ 写出的输出文件回读是 $delivered 行，应是 $got 行，已删，这一趟作废"
  echo "     → 怎么办：核对盘没满、目录可写，再重交"
  exit 1
fi
echo "E53（丢一整块盘之后根环还挂不挂得上）虚机复跑完成：$output_file（$delivered 条结果行，与收尾行声称的 ${declared:-无} 条相符）"
