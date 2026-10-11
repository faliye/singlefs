#!/usr/bin/env bash
# 编一份打了 scripts/devices/qemu/blklogwrites-linearize-over-iov-max.patch 的 QEMU，装进私有目录；系统的 qemu-system-x86 包不动。
# 源码是 Ubuntu 源码包 qemu，版本与本机 qemu-system-x86 包相同（QEMU_SOURCE_VERSION），带 Ubuntu 自己的补丁，再打我们那一份。
# scripts/devices/vm-bench.sh 的 blklogwrites 一支经仓根 qemu.env（模板 qemu.env.example）用这里装的 qemu-system-x86_64。
#
#   build-patched-qemu.sh             装过、输入没变、装的那份过 qemu-blklogwrites-check.sh 就什么都不做；否则取源码、核、打补丁、编、核、装
#   build-patched-qemu.sh --rebuild   不管装没装过，照编照装
#   build-patched-qemu.sh --selftest  用手造的 .dsc、文件、目录与假源码树核每一项判法；不起网络、不编译、不装
#
# 取源码：本机没有 deb-src，从 launchpad（取不到换 Ubuntu 的 pool，QEMU_SOURCE_URL_BASES）取 .dsc，.dsc 自己的 sha256 钉在 QEMU_SOURCE_DSC_SHA256；.dsc 里 Checksums-Sha256 列的每个文件逐个核 sha256 与大小，
# 核不过拒绝。dpkg-source -x 解出（它自己也按 .dsc 核一遍），patch -p1 打补丁，回读补丁那一行在不在。
# 构建：configure --without-default-features，只开 vm-bench.sh 用得到的（kvm、linux-aio、slirp、pixman、tools），x86_64-softmmu，不编 tcg、文档；
# 固件路径与 Ubuntu 包相同（QEMU_FIRMWARE_PATH，用系统 seabios、ipxe 的固件，不装自带的）。本机没装 libaio-dev、libfdt-dev 时，apt-get download 取与已装运行库
# 同版本的开发包解进编译目录，只拿头文件（不 sudo、不装进系统），链接本机已装的运行库（系统 QEMU 也链这两个）。
# 装：make install 先装进编译目录里的暂存根，拿暂存的 qemu-io 跑 scripts/devices/qemu-blklogwrites-check.sh，绿了才换进私有目录，换完拿装好的再核一次。
# 私有目录里写 singlefs-build-stamp.txt（版本、.dsc 与补丁的 sha256、configure 参数）；已有的私有目录里没有这份就不动它、拒绝。
#
# 环境变量：
#   SINGLEFS_QEMU_HOME         私有目录，默认 $HOME/.local/share/singlefs/qemu；不许落在开机会清空的目录（/tmp、/run、/dev/shm、XDG_RUNTIME_DIR、tmpfs、ramfs）
#   SINGLEFS_THREAD_CAP        make 的并行数（scripts/hosts/capped.sh 设它），没设取 nproc
#   BUILD_PATCHED_QEMU_PATCH   补丁路径，默认仓里那一份；只给补丁还没进仓时用
#   BUILD_PATCHED_QEMU_KEEP=1  编译目录留着（默认装完、失败都删），打印路径
# 编译目录在 ${TMPDIR:-/tmp} 下，约 1 GiB。
# 退出码：0 装好（或已是最新）；1 拒绝或失败（原因与下一步写在 stderr）。
#
# 弄坏开关 BUILD_PATCHED_QEMU_BREAK=<项>（只给证红用；--selftest 里点名的那一格判错）：
#   skip-dsc-checksum     不核 .dsc 自己的 sha256          → 「.dsc 改了一个字节」那一格
#   skip-file-checksum    不核 .dsc 列出的文件             → 「tarball 改了一个字节」「tarball 短了一截」两格
#   allow-volatile-home   私有目录落在开机会清空的地方也放行 → 「私有目录在 /tmp 下」「私有目录在 tmpfs 上」两格
#   skip-version-match    不核系统 qemu-system-x86 的版本  → 「系统包版本不同」那一格
#   skip-patch-readback   打完补丁不回读                   → 「源码树里没有补丁那一行」那一格
#   stamp-ignores-patch   装过的戳不记补丁的 sha256        → 「补丁变了要重编」那一格
#
# admission: always 每次调都现判私有目录里装的是不是这一份源码与补丁、核不核得过，装好了它自己什么都不做就退出
# run-condition: single-instance
# run-condition: command curl dpkg-source dpkg-query dpkg-deb apt-get patch make meson ninja gcc pkg-config python3 sha256sum
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repository_root="$(cd "$script_directory/../.." && pwd)"

QEMU_SOURCE_VERSION='1:8.2.2+ds-0ubuntu1.18'
QEMU_SOURCE_DSC_NAME='qemu_8.2.2+ds-0ubuntu1.18.dsc'
QEMU_SOURCE_DSC_SHA256='0c7baad16818c5cd29d5031dc8c88cc632f4f8bdb864e80c5c7df9db53a365f2'
# 依次试这几处，取到为止：launchpad 留着每一版，Ubuntu 的 pool 只留当前版。传输不加密也行，.dsc 钉了 sha256、它列的文件逐个核
QEMU_SOURCE_URL_BASES=(
  'https://launchpad.net/ubuntu/+archive/primary/+sourcefiles/qemu/1:8.2.2+ds-0ubuntu1.18'
  'http://security.ubuntu.com/ubuntu/pool/main/q/qemu'
  'http://archive.ubuntu.com/ubuntu/pool/main/q/qemu'
)
SYSTEM_QEMU_PACKAGE='qemu-system-x86'
LIBAIO_RUNTIME_PACKAGE='libaio1t64'
LIBAIO_DEVELOPMENT_PACKAGE='libaio-dev'
LIBFDT_RUNTIME_PACKAGE='libfdt1'
LIBFDT_DEVELOPMENT_PACKAGE='libfdt-dev'
QEMU_FIRMWARE_PATH='/usr/share/qemu:/usr/share/seabios:/usr/lib/ipxe/qemu'
PATCH_MARKER_LINE='    if (qiov && niov + 2 > IOV_MAX) {'
STAMP_FILE_NAME='singlefs-build-stamp.txt'
INSTALLED_BINARIES=(qemu-system-x86_64 qemu-io qemu-img)
CONFIGURE_ARGUMENTS=(
  --target-list=x86_64-softmmu
  --without-default-features
  --enable-kvm --disable-tcg
  --enable-linux-aio --enable-slirp --enable-pixman --enable-tools
  --disable-docs --disable-download --disable-install-blobs
  "--firmwarepath=$QEMU_FIRMWARE_PATH"
)
BREAK_ITEMS=(skip-dsc-checksum skip-file-checksum allow-volatile-home skip-version-match skip-patch-readback stamp-ignores-patch)

patch_path="${BUILD_PATCHED_QEMU_PATCH:-$repository_root/scripts/devices/qemu/blklogwrites-linearize-over-iov-max.patch}"
install_home="${SINGLEFS_QEMU_HOME:-$HOME/.local/share/singlefs/qemu}"
build_threads="${SINGLEFS_THREAD_CAP:-$(nproc)}"
build_break="${BUILD_PATCHED_QEMU_BREAK:-}"
check_script="$script_directory/qemu-blklogwrites-check.sh"

die() {
  printf '  ✗ %s\n' "$1" >&2
  printf '     → 怎么办：%s\n' "$2" >&2
  exit 1
}
ok() { printf '  ✓ %s\n' "$*"; }

if [[ -n "$build_break" ]]; then
  known_break=0
  for item in "${BREAK_ITEMS[@]}"; do [[ "$item" == "$build_break" ]] && known_break=1; done
  (( known_break )) || die "BUILD_PATCHED_QEMU_BREAK=$build_break 认不出" "只认 ${BREAK_ITEMS[*]}；不证红就别设它"
fi

# ── 判法：每个函数只打印问题、用返回值报成不成立，不退出，--selftest 直接调它们 ──

# dsc_checksum_problem <.dsc 路径> <钉住的 sha256>
dsc_checksum_problem() {
  [[ "$build_break" == skip-dsc-checksum ]] && return 0
  local actual
  actual="$(sha256sum "$1" | cut -d' ' -f1)"
  [[ "$actual" == "$2" ]] && return 0
  printf '%s 的 sha256 是 %s，钉住的是 %s\n' "$1" "$actual" "$2"
  return 1
}

# dsc_listed_files <.dsc 路径>：打印 Checksums-Sha256 一节的每一行「sha256 大小 文件名」；文件名带 / 或以 . 开头、一行都没有，都算不成立
dsc_listed_files() {
  local in_section=0 line listed=0 file_sha256 file_size file_name
  while IFS= read -r line; do
    if [[ "$line" == 'Checksums-Sha256:' ]]; then in_section=1; continue; fi
    (( in_section )) || continue
    [[ "$line" == ' '* ]] || break
    read -r file_sha256 file_size file_name <<<"$line"
    if [[ ! "$file_sha256" =~ ^[0-9a-f]{64}$ || ! "$file_size" =~ ^[0-9]+$ || -z "$file_name" || "$file_name" == */* || "$file_name" == .* ]]; then
      printf '%s 的 Checksums-Sha256 一节有认不出的行：%s\n' "$1" "$line"
      return 1
    fi
    printf '%s %s %s\n' "$file_sha256" "$file_size" "$file_name"
    listed=$((listed + 1))
  done < "$1"
  (( listed > 0 )) && return 0
  printf '%s 里没有 Checksums-Sha256 一节，或那一节是空的\n' "$1"
  return 1
}

# listed_file_problem <目录> <sha256> <大小> <文件名>
listed_file_problem() {
  [[ "$build_break" == skip-file-checksum ]] && return 0
  local path="$1/$4" actual_size actual_sha256
  [[ -f "$path" ]] || { printf '%s 不在\n' "$path"; return 1; }
  actual_size="$(stat -c %s "$path")"
  [[ "$actual_size" == "$3" ]] || { printf '%s 是 %s 字节，.dsc 写的是 %s\n' "$path" "$actual_size" "$3"; return 1; }
  actual_sha256="$(sha256sum "$path" | cut -d' ' -f1)"
  [[ "$actual_sha256" == "$2" ]] || { printf '%s 的 sha256 是 %s，.dsc 写的是 %s\n' "$path" "$actual_sha256" "$2"; return 1; }
  return 0
}

# install_home_problem <路径> [<文件系统类型>]：不给类型就取它自己或最近一层存在的上级目录的（stat -f）
install_home_problem() {
  local path="$1" filesystem_type="${2:-}" resolved existing volatile
  [[ "$path" == /* ]] || { printf '私有目录 %s 不是绝对路径\n' "$path"; return 1; }
  [[ "$build_break" == allow-volatile-home ]] && return 0
  resolved="$(realpath -m "$path")"
  for volatile in /tmp /run /var/run /dev/shm ${XDG_RUNTIME_DIR:+"$XDG_RUNTIME_DIR"}; do
    if [[ "$resolved" == "$volatile" || "$resolved" == "$volatile"/* ]]; then
      printf '私有目录 %s 在 %s 下，开机会清空\n' "$resolved" "$volatile"
      return 1
    fi
  done
  if [[ -z "$filesystem_type" ]]; then
    existing="$resolved"
    while [[ ! -e "$existing" ]]; do existing="$(dirname "$existing")"; done
    filesystem_type="$(stat -f -c %T "$existing")"
  fi
  case "$filesystem_type" in
    tmpfs|ramfs) printf '私有目录 %s 在 %s 上，开机会清空\n' "$resolved" "$filesystem_type"; return 1 ;;
  esac
  return 0
}

# system_version_problem <本机 qemu-system-x86 的版本>
system_version_problem() {
  [[ "$build_break" == skip-version-match ]] && return 0
  [[ "$1" == "$QEMU_SOURCE_VERSION" ]] && return 0
  printf '本机 %s 是「%s」，这里钉的源码是 %s\n' "$SYSTEM_QEMU_PACKAGE" "$1" "$QEMU_SOURCE_VERSION"
  return 1
}

# patched_tree_problem <源码树>
patched_tree_problem() {
  [[ "$build_break" == skip-patch-readback ]] && return 0
  local source_file="$1/block/blklogwrites.c" rejects
  [[ -f "$source_file" ]] || { printf '%s 不在\n' "$source_file"; return 1; }
  grep -qxF -- "$PATCH_MARKER_LINE" "$source_file" || { printf '%s 里没有补丁那一行「%s」\n' "$source_file" "$PATCH_MARKER_LINE"; return 1; }
  rejects="$(find "$1/block" -name '*.rej' -print)"
  [[ -z "$rejects" ]] || { printf '打补丁留下了没合上的块：%s\n' "$rejects"; return 1; }
  return 0
}

# stamp_text <补丁的 sha256>：私有目录里那份戳该有的内容
stamp_text() {
  printf 'qemu-source-version=%s\n' "$QEMU_SOURCE_VERSION"
  printf 'qemu-source-dsc-sha256=%s\n' "$QEMU_SOURCE_DSC_SHA256"
  [[ "$build_break" == stamp-ignores-patch ]] || printf 'patch-sha256=%s\n' "$1"
  printf 'configure-arguments=%s\n' "${CONFIGURE_ARGUMENTS[*]}"
}

# stamp_is_current <私有目录> <补丁的 sha256>：戳与这一次的输入相同、二进制都在
stamp_is_current() {
  local binary
  [[ -f "$1/$STAMP_FILE_NAME" ]] || return 1
  [[ "$(cat "$1/$STAMP_FILE_NAME")" == "$(stamp_text "$2")" ]] || return 1
  for binary in "${INSTALLED_BINARIES[@]}"; do [[ -x "$1/bin/$binary" ]] || return 1; done
  return 0
}

# install_home_takeover_problem <私有目录>：已在而不是本脚本装的（没有戳、又不是空目录），不动它
install_home_takeover_problem() {
  [[ -e "$1" ]] || return 0
  [[ -d "$1" ]] || { printf '%s 已在，而且不是目录\n' "$1"; return 1; }
  [[ -f "$1/$STAMP_FILE_NAME" ]] && return 0
  [[ -z "$(ls -A "$1")" ]] && return 0
  printf '%s 已在、不是空的，里面没有 %s，不是这个脚本装的\n' "$1" "$STAMP_FILE_NAME"
  return 1
}

# ── 自证 ──
selftest_right=0
selftest_wrong=()
# selftest_case <格名> <该红：1 该红，0 该绿> <弄坏哪个开关这一格判错，没有写 -> <命令…>
selftest_case() {
  local name="$1" should_be_red="$2" guarding_break="$3" output status
  shift 3
  if output="$("$@" 2>&1)"; then status=0; else status=$?; fi
  if { [[ "$should_be_red" == 1 && $status -ne 0 ]] || [[ "$should_be_red" == 0 && $status -eq 0 ]]; }; then
    selftest_right=$((selftest_right + 1))
    printf '  ✓ %s：%s\n' "$name" "$([[ $status -ne 0 ]] && printf '判红（%s）' "${output//$'\n'/ }" || printf '判绿')"
  else
    selftest_wrong+=("$name")
    printf '  ✗ %s：该%s，判成%s（%s）\n' "$name" "$([[ "$should_be_red" == 1 ]] && echo 红 || echo 绿)" "$([[ $status -ne 0 ]] && echo 红 || echo 绿)" "${output//$'\n'/ }"  # gate-lint:detail
    [[ "$guarding_break" == - ]] || printf '     → BUILD_PATCHED_QEMU_BREAK=%s 设着的话这一格本来就该判错；没设就看这一格调的那个判法函数\n' "$guarding_break"
  fi
}

# dsc_listed_files_equals <.dsc> <期望的输出>
dsc_listed_files_equals() {
  local actual
  actual="$(dsc_listed_files "$1")" || { printf '%s\n' "$actual"; return 1; }
  [[ "$actual" == "$2" ]] && return 0
  printf '解析出「%s」，该是「%s」\n' "${actual//$'\n'/ | }" "${2//$'\n'/ | }"
  return 1
}

# stamp_is_stale <私有目录> <补丁的 sha256>：戳不是这一次的就成立（给「补丁变了要重编」那一格用）
stamp_is_stale() {
  if stamp_is_current "$1" "$2"; then printf '%s 的戳被当成这一次的，补丁 sha256 %s 变了也不重编\n' "$1" "$2"; return 1; fi
  return 0
}

run_selftest() {
  local scratch fake_tarball fake_debian fake_dsc dsc_sha256 tarball_sha256 debian_sha256 tarball_size debian_size
  scratch="$(mktemp -d "${TMPDIR:-/tmp}/singlefs-build-patched-qemu-selftest.XXXXXX")"
  trap 'rm -rf "${scratch:?}"' EXIT
  mkdir -p "$scratch/download"
  fake_tarball="qemu_0.0.0+ds.orig.tar.xz"; fake_debian="qemu_0.0.0+ds-0ubuntu0.debian.tar.xz"
  printf 'fake upstream tarball\n' > "$scratch/download/$fake_tarball"
  printf 'fake debian tarball\n' > "$scratch/download/$fake_debian"
  tarball_sha256="$(sha256sum "$scratch/download/$fake_tarball" | cut -d' ' -f1)"; tarball_size="$(stat -c %s "$scratch/download/$fake_tarball")"
  debian_sha256="$(sha256sum "$scratch/download/$fake_debian" | cut -d' ' -f1)"; debian_size="$(stat -c %s "$scratch/download/$fake_debian")"
  fake_dsc="$scratch/download/qemu_0.0.0.dsc"
  printf 'Format: 3.0 (quilt)\nSource: qemu\nChecksums-Sha1:\n 0000000000000000000000000000000000000000 1 %s\nChecksums-Sha256:\n %s %s %s\n %s %s %s\nFiles:\n 00000000000000000000000000000000 1 %s\n' \
    "$fake_tarball" "$tarball_sha256" "$tarball_size" "$fake_tarball" "$debian_sha256" "$debian_size" "$fake_debian" "$fake_tarball" > "$fake_dsc"
  dsc_sha256="$(sha256sum "$fake_dsc" | cut -d' ' -f1)"
  cp "$fake_dsc" "$scratch/tampered.dsc"; printf 'X' | dd of="$scratch/tampered.dsc" bs=1 seek=8 conv=notrunc status=none
  printf 'Format: 3.0 (quilt)\nChecksums-Sha256:\n %s %s ../%s\n' "$tarball_sha256" "$tarball_size" "$fake_tarball" > "$scratch/traversal.dsc"
  mkdir -p "$scratch/tampered-download" "$scratch/short-download"
  cp "$scratch/download/$fake_tarball" "$scratch/tampered-download/"; printf 'Y' | dd of="$scratch/tampered-download/$fake_tarball" bs=1 seek=0 conv=notrunc status=none
  head -c 5 "$scratch/download/$fake_tarball" > "$scratch/short-download/$fake_tarball"
  mkdir -p "$scratch/patched-tree/block" "$scratch/unpatched-tree/block" "$scratch/home-current/bin" "$scratch/home-foreign"
  printf 'static int f(void)\n{\n%s\n}\n' "$PATCH_MARKER_LINE" > "$scratch/patched-tree/block/blklogwrites.c"
  printf 'static int f(void)\n{\n    qemu_iovec_init(&log_qiov, niov + 2);\n}\n' > "$scratch/unpatched-tree/block/blklogwrites.c"
  for binary in "${INSTALLED_BINARIES[@]}"; do printf '#!/bin/sh\nexit 0\n' > "$scratch/home-current/bin/$binary"; chmod +x "$scratch/home-current/bin/$binary"; done
  stamp_text "$(printf '%064d' 1)" > "$scratch/home-current/$STAMP_FILE_NAME"
  printf 'someone else\n' > "$scratch/home-foreign/README"

  selftest_case ".dsc 的 sha256 与钉住的相同" 0 - dsc_checksum_problem "$fake_dsc" "$dsc_sha256"
  selftest_case ".dsc 改了一个字节" 1 skip-dsc-checksum dsc_checksum_problem "$scratch/tampered.dsc" "$dsc_sha256"
  selftest_case ".dsc 的 Checksums-Sha256 一节解析成两行" 0 - dsc_listed_files_equals "$fake_dsc" \
    "$tarball_sha256 $tarball_size $fake_tarball"$'\n'"$debian_sha256 $debian_size $fake_debian"
  selftest_case ".dsc 列的文件名带 ../" 1 - dsc_listed_files "$scratch/traversal.dsc"
  selftest_case "tarball 与 .dsc 列的相同" 0 - listed_file_problem "$scratch/download" "$tarball_sha256" "$tarball_size" "$fake_tarball"
  selftest_case "tarball 改了一个字节" 1 skip-file-checksum listed_file_problem "$scratch/tampered-download" "$tarball_sha256" "$tarball_size" "$fake_tarball"
  selftest_case "tarball 短了一截" 1 skip-file-checksum listed_file_problem "$scratch/short-download" "$tarball_sha256" "$tarball_size" "$fake_tarball"
  selftest_case "私有目录在家目录下的 ext4 上" 0 - install_home_problem /home/someone/.local/share/singlefs/qemu ext2/ext3
  selftest_case "私有目录在 /tmp 下" 1 allow-volatile-home install_home_problem /tmp/singlefs-qemu ext2/ext3
  selftest_case "私有目录在 /dev/shm 下" 1 allow-volatile-home install_home_problem /dev/shm/qemu tmpfs
  selftest_case "私有目录在 tmpfs 上" 1 allow-volatile-home install_home_problem /srv/ramdisk/qemu tmpfs
  selftest_case "私有目录写成相对路径" 1 - install_home_problem relative/qemu ext2/ext3
  selftest_case "系统包版本与钉住的相同" 0 - system_version_problem "$QEMU_SOURCE_VERSION"
  selftest_case "系统包版本不同" 1 skip-version-match system_version_problem '1:8.2.2+ds-0ubuntu1.19'
  selftest_case "源码树里有补丁那一行" 0 - patched_tree_problem "$scratch/patched-tree"
  selftest_case "源码树里没有补丁那一行" 1 skip-patch-readback patched_tree_problem "$scratch/unpatched-tree"
  selftest_case "戳与这一次的输入相同" 0 - stamp_is_current "$scratch/home-current" "$(printf '%064d' 1)"
  selftest_case "补丁变了要重编" 0 stamp-ignores-patch stamp_is_stale "$scratch/home-current" "$(printf '%064d' 2)"
  selftest_case "私有目录已在、不是本脚本装的" 1 - install_home_takeover_problem "$scratch/home-foreign"
  selftest_case "私有目录是本脚本装过的" 0 - install_home_takeover_problem "$scratch/home-current"

  local total=$((selftest_right + ${#selftest_wrong[@]}))
  if (( ${#selftest_wrong[@]} > 0 )); then
    printf '  ✗ 自证 %d 格里 %d 格判错：%s\n' "$total" "${#selftest_wrong[@]}" "${selftest_wrong[*]}"
    printf '     → 怎么办：看上面每一格 ✗ 的说明；设了 BUILD_PATCHED_QEMU_BREAK 的话，判错的应当正是它点名的那几格\n'
    exit 1
  fi
  ok "自证 $total 格全部判对（不起网络、不编译）"
  exit 0
}

# ── 主流程 ──
rebuild=0
case "${1:-}" in
  --selftest) [[ $# -eq 1 ]] || die "--selftest 不带别的参数" "单独跑 build-patched-qemu.sh --selftest"; run_selftest ;;
  --rebuild) rebuild=1; shift ;;
  '') ;;
  *) die "认不出的参数 $1" "用法：build-patched-qemu.sh [--rebuild]，或 build-patched-qemu.sh --selftest" ;;
esac
[[ $# -eq 0 ]] || die "多给了参数：$*" "用法：build-patched-qemu.sh [--rebuild]，或 build-patched-qemu.sh --selftest"
[[ "$build_threads" =~ ^[1-9][0-9]*$ ]] || die "并行数「$build_threads」不是正整数" "SINGLEFS_THREAD_CAP 设成正整数，或经 scripts/hosts/capped.sh <上限> 起这个脚本"

system_version="$(dpkg-query -W -f='${Version}' "$SYSTEM_QEMU_PACKAGE" 2>/dev/null)" || system_version="（没装）"
problem="$(system_version_problem "$system_version")" \
  || die "$problem" "系统包升级过的话，把 QEMU_SOURCE_VERSION、QEMU_SOURCE_DSC_NAME、QEMU_SOURCE_DSC_SHA256、QEMU_SOURCE_URL_BASES 一起改成新版本（.dsc 的 sha256 现取现核），确认补丁还打得上再跑"
problem="$(install_home_problem "$install_home")" \
  || die "$problem" "把 SINGLEFS_QEMU_HOME 设到开机不清空的地方（默认 \$HOME/.local/share/singlefs/qemu），qemu.env 里写同一个路径"
problem="$(install_home_takeover_problem "$install_home")" \
  || die "$problem" "换一个 SINGLEFS_QEMU_HOME，或确认那个目录没人用之后自己挪走它"
[[ -f "$patch_path" ]] || die "补丁 $patch_path 不在" "补丁在仓里 scripts/devices/qemu/blklogwrites-linearize-over-iov-max.patch；还没进仓时设 BUILD_PATCHED_QEMU_PATCH 指到它"
[[ -f "$check_script" ]] || die "核查脚本 $check_script 不在" "它与本脚本在同一个目录（scripts/devices/qemu-blklogwrites-check.sh），从仓里取回来再跑"
patch_sha256="$(sha256sum "$patch_path" | cut -d' ' -f1)"

if (( ! rebuild )) && stamp_is_current "$install_home" "$patch_sha256"; then
  if bash "$check_script" "$install_home/bin/qemu-io"; then
    ok "$install_home 里装的就是这一份（$QEMU_SOURCE_VERSION，补丁 sha256 ${patch_sha256:0:12}），核过了，不重编；要重编带 --rebuild"
    exit 0
  fi
  printf '  ! %s 的戳是这一份，但核不过，照编照装\n' "$install_home"
fi

for tool in curl dpkg-source dpkg-query dpkg-deb apt-get patch make meson ninja gcc pkg-config python3 sha256sum; do
  command -v "$tool" >/dev/null || die "缺 $tool" "装上它（构建依赖：meson ninja-build libglib2.0-dev libpixman-1-dev python3-venv libslirp-dev dpkg-dev），装完重跑"
done

work_directory="$(mktemp -d "${TMPDIR:-/tmp}/singlefs-qemu-build.XXXXXX")"
cleanup_work_directory() {
  if [[ "${BUILD_PATCHED_QEMU_KEEP:-0}" == 1 ]]; then
    printf '  ! 编译目录留着：%s\n' "$work_directory" >&2
  else
    rm -rf "${work_directory:?}"
  fi
}
trap cleanup_work_directory EXIT
download_directory="$work_directory/download"
log_directory="$work_directory/logs"
mkdir -p "$download_directory" "$log_directory"

# fail_with_log <日志> <卡在哪> <下一步>
fail_with_log() {
  tail -n 40 "$1" | sed 's/^/        /' >&2
  die "$2（日志末尾 40 行在上面）" "$3"
}

# fetch_source_file <文件名>：依次从 QEMU_SOURCE_URL_BASES 取进下载目录，连不上、半分钟连不通、一分钟不到 10 KiB/s 就换下一处
fetch_source_file() {
  local base
  for base in "${QEMU_SOURCE_URL_BASES[@]}"; do
    if curl -fsSL --connect-timeout 30 --speed-limit 10240 --speed-time 60 --retry 2 -o "$download_directory/$1" "$base/$1"; then
      printf '  取到 %s（%s）\n' "$1" "$base"
      return 0
    fi
    printf '  ! %s 取不到 %s，换下一处\n' "$base" "$1" >&2
  done
  return 1
}

echo "── 取源码 $QEMU_SOURCE_VERSION ──"
fetch_source_file "$QEMU_SOURCE_DSC_NAME" \
  || die "${QEMU_SOURCE_URL_BASES[*]} 几处都取不到 $QEMU_SOURCE_DSC_NAME" "先确认本机连得上它们（curl -I 那几个地址），连上了重跑"
problem="$(dsc_checksum_problem "$download_directory/$QEMU_SOURCE_DSC_NAME" "$QEMU_SOURCE_DSC_SHA256")" \
  || die "$problem" "取到的 .dsc 与钉住的不同：不用它。确认 URL 指的是 $QEMU_SOURCE_VERSION；版本要换就连同 QEMU_SOURCE_DSC_SHA256 一起改"
listed_files="$(dsc_listed_files "$download_directory/$QEMU_SOURCE_DSC_NAME")" \
  || die "$listed_files" ".dsc 的 sha256 核过了却解析不出来，看 dsc_listed_files 的判法与 .dsc 原文"
listed_count=0
while read -r file_sha256 file_size file_name; do
  fetch_source_file "$file_name" \
    || die "${QEMU_SOURCE_URL_BASES[*]} 几处都取不到 $file_name" "先确认本机连得上它们，连上了重跑"
  problem="$(listed_file_problem "$download_directory" "$file_sha256" "$file_size" "$file_name")" \
    || die "$problem" "取到的文件与 .dsc 列的不同：不用它。重跑一次；还不同就是源头变了，停下查"
  listed_count=$((listed_count + 1))
done <<<"$listed_files"
ok ".dsc 与它列的 $listed_count 个文件都核过 sha256"

source_tree="$work_directory/source"
nice -n 19 dpkg-source -x "$download_directory/$QEMU_SOURCE_DSC_NAME" "$source_tree" > "$log_directory/dpkg-source.log" 2>&1 \
  || fail_with_log "$log_directory/dpkg-source.log" "dpkg-source -x 解不出源码" "看日志里是哪一份 Ubuntu 补丁打不上，或哪个文件核不过"
patch -p1 --forward --batch -d "$source_tree" < "$patch_path" > "$log_directory/patch.log" 2>&1 \
  || fail_with_log "$log_directory/patch.log" "补丁 $patch_path 打不上" "补丁是照 $QEMU_SOURCE_VERSION 的 block/blklogwrites.c 写的；源码版本换过就照新源码重写补丁"
problem="$(patched_tree_problem "$source_tree")" || die "$problem" "补丁 patch 退 0 却没改进去：看 $log_directory/patch.log，设 BUILD_PATCHED_QEMU_KEEP=1 重跑留下源码树对照"
ok "解出源码、打上补丁（回读到补丁那一行）"

# borrow_development_headers <头文件> <运行库包> <开发包> <库名>：本机 /usr/include 里没有那个头文件时，apt-get download 取与已装运行库同版本的开发包、
# 解进编译目录，-I 指它的 usr/include，lib<库名>.so 链到本机已装的运行库，-L 指那里；有头文件就什么都不做
development_root="$work_directory/development"
development_library_directory="$development_root/lib"
borrowed_development_packages=()
borrow_development_headers() {
  local header="$1" runtime_package="$2" development_package="$3" library_name="$4" runtime_version runtime_library
  [[ -f "/usr/include/$header" ]] && return 0
  runtime_version="$(dpkg-query -W -f='${Version}' "$runtime_package" 2>/dev/null)" \
    || die "本机没有 $development_package，也没装 $runtime_package 运行库" "装 $development_package（它会带上运行库），装完重跑"
  runtime_library="$(dpkg -L "$runtime_package" | grep -E "/lib${library_name}\.so\.[^/]*$" | sort | head -n 1)"
  [[ -n "$runtime_library" && -e "$runtime_library" ]] || die "$runtime_package 里找不到 lib$library_name.so.*" "装 $development_package，装完重跑"
  mkdir -p "$development_root/deb/$development_package" "$development_library_directory"
  ( cd "$development_root/deb/$development_package" && apt-get download "$development_package=$runtime_version" ) > "$log_directory/$development_package.log" 2>&1 \
    || fail_with_log "$log_directory/$development_package.log" "apt-get download $development_package=$runtime_version 取不到" "apt 源里要有与已装运行库同版本的 $development_package；取不到就装 $development_package 再跑"
  dpkg-deb -x "$development_root/deb/$development_package/${development_package}_"*.deb "$development_root/root" >> "$log_directory/$development_package.log" 2>&1 \
    || fail_with_log "$log_directory/$development_package.log" "解不开 $development_package 的包" "看日志；或者装 $development_package 再跑"
  [[ -f "$development_root/root/usr/include/$header" ]] || die "$development_package 包里没有 usr/include/$header" "装 $development_package 再跑"
  ln -sfn "$runtime_library" "$development_library_directory/lib$library_name.so"
  borrowed_development_packages+=("$development_package $runtime_version → $runtime_library")
}
borrow_development_headers libaio.h "$LIBAIO_RUNTIME_PACKAGE" "$LIBAIO_DEVELOPMENT_PACKAGE" aio
borrow_development_headers libfdt.h "$LIBFDT_RUNTIME_PACKAGE" "$LIBFDT_DEVELOPMENT_PACKAGE" fdt
extra_compiler_flags=""
extra_linker_flags=""
if (( ${#borrowed_development_packages[@]} > 0 )); then
  extra_compiler_flags="-I$development_root/root/usr/include"
  extra_linker_flags="-L$development_library_directory"
  ok "借了 ${#borrowed_development_packages[@]} 个开发包的头文件（只在编译目录里，链接本机已装的运行库）：$(printf '%s；' "${borrowed_development_packages[@]}")"
fi

stage_root="$work_directory/stage"
configure_command=(./configure "--prefix=$install_home" "--with-pkgversion=singlefs blklogwrites patch ${patch_sha256:0:12}" "${CONFIGURE_ARGUMENTS[@]}")
[[ -n "$extra_compiler_flags" ]] && configure_command+=("--extra-cflags=$extra_compiler_flags" "--extra-ldflags=$extra_linker_flags")
echo "── configure、编（$build_threads 路并行）、装进暂存根 ──"
( cd "$source_tree" && nice -n 19 "${configure_command[@]}" ) > "$log_directory/configure.log" 2>&1 \
  || fail_with_log "$log_directory/configure.log" "configure 失败" "多半是缺开发包：照日志里点名的依赖装上再跑"
nice -n 19 make -C "$source_tree" -j "$build_threads" > "$log_directory/make.log" 2>&1 \
  || fail_with_log "$log_directory/make.log" "编译失败" "看日志里第一处 error；是补丁那几行就改补丁"
nice -n 19 make -C "$source_tree" install DESTDIR="$stage_root" > "$log_directory/install.log" 2>&1 \
  || fail_with_log "$log_directory/install.log" "make install 失败" "看日志"
staged_home="$stage_root$install_home"
for binary in "${INSTALLED_BINARIES[@]}"; do
  [[ -x "$staged_home/bin/$binary" ]] || die "暂存根里没有 bin/$binary" "看 $log_directory/install.log（设 BUILD_PATCHED_QEMU_KEEP=1 重跑留下它）；configure 参数里 --enable-tools 与 x86_64-softmmu 还在不在"
done
staged_version="$("$staged_home/bin/qemu-io" --version | head -n 1)"
[[ "$staged_version" == *"singlefs blklogwrites patch ${patch_sha256:0:12}"* ]] \
  || die "暂存的 qemu-io 报的版本「$staged_version」里没有这一份补丁的标记" "看 configure 的 --with-pkgversion 有没有生效"
ok "编好：$staged_version"

echo "── 拿暂存的 qemu-io 核 blklogwrites ──"
bash "$check_script" "$staged_home/bin/qemu-io" \
  || die "补好的 qemu-io 核不过，不装" "看上面判红的那几条；补丁打上了却没修好，就照判红的那一项改补丁"

echo "── 换进 $install_home ──"
install_parent="$(dirname "$install_home")"
mkdir -p "$install_parent"
incoming="$(mktemp -d "$install_parent/.singlefs-qemu-incoming.XXXXXX")"
cp -a "$staged_home/." "$incoming/" || { rm -rf "${incoming:?}"; die "拷不进 $incoming" "看 $install_parent 的空间与权限"; }
stamp_text "$patch_sha256" > "$incoming/$STAMP_FILE_NAME"
previous=""
if [[ -e "$install_home" ]]; then
  previous="$(mktemp -u "$install_parent/.singlefs-qemu-previous.XXXXXX")"
  mv "$install_home" "$previous" || { rm -rf "${incoming:?}"; die "挪不开旧的 $install_home" "看权限；旧的没动"; }
fi
mv "$incoming" "$install_home" || die "换不进 $install_home（旧的在 ${previous:-（没有）}）" "把旧的挪回 $install_home，看权限后重跑"
[[ -n "$previous" ]] && rm -rf "${previous:?}"
for binary in "${INSTALLED_BINARIES[@]}"; do
  [[ -x "$install_home/bin/$binary" ]] || die "装完 $install_home/bin/$binary 不在" "带 --rebuild 重跑"
done
bash "$check_script" "$install_home/bin/qemu-io" \
  || die "装好的 $install_home/bin/qemu-io 核不过（暂存的那份核过了）" "带 --rebuild 重跑；还不过就看是不是装进去之后动态库换了"
ok "装好：$install_home（$QEMU_SOURCE_VERSION + 补丁 sha256 ${patch_sha256:0:12}）"
echo "  仓根 qemu.env 里写：SINGLEFS_QEMU_HOME=$install_home"
