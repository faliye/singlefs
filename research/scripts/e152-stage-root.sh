#!/usr/bin/env bash
# E152（按里程碑对比六家文件系统的文件性能）的 initramfs 附加根：fio、六家的格式化与挂载工具连同依赖库、
# 解压过的内核模块与加载次序、singlefs 的真设备二进制，摆进一个目录，交给 vm-bench.sh 的 VM_EXTRA_ROOT。
#
#   e152-stage-root.sh <目标目录> <内核版本> <first_transaction_on_device 的路径>
#
# 本机没有 root，也没装 zfsutils-linux / bcachefs-tools：这两家的用户态从 Ubuntu 仓库下 .deb、核过 sha256 再解包，
# 不装进系统（缓存在 ${TMPDIR:-/tmp}/singlefs-e152-packages）。版本与跑前登记 research/prompts/e152-preregistration.md
# 第二节一致：ZFS 用户态 2.3.4-1ubuntu2 与 OEM 内核自带的模块逐字同版本；bcachefs-tools 取 Ubuntu 24.04 的包。
# 模块的加载次序交给 modprobe --show-depends 从内核自己的 modules.dep 里解（含软依赖），不手写。
set -uo pipefail

DESTINATION="${1:?用法：e152-stage-root.sh <目标目录> <内核版本> <first_transaction_on_device 的路径>}"
RELEASE="${2:?缺内核版本，例：$(uname -r)}"
SINGLEFS_BINARY="${3:?缺 first_transaction_on_device 的路径}"
PACKAGES="${E152_PACKAGE_CACHE:-${TMPDIR:-/tmp}/singlefs-e152-packages}"

fail() { echo "  ✗ $1" >&2; echo "    → $2" >&2; exit 1; }

[[ -d "/lib/modules/$RELEASE" ]] || fail "没有 /lib/modules/$RELEASE" "内核版本要与 /lib/modules 下的目录同名：ls /lib/modules"
[[ -x "$SINGLEFS_BINARY" ]] || fail "$SINGLEFS_BINARY 不可执行" "在仓根跑 cargo build --release --target x86_64-unknown-linux-musl -p singlefs-harness --bin first_transaction_on_device"
for tool in fio mkfs.ext4 mkfs.xfs mkfs.f2fs mkfs.btrfs sfdisk mdadm zstd dpkg-deb curl sha256sum ldd modprobe; do
  command -v "$tool" >/dev/null || fail "缺 $tool" "装上它（fio / e2fsprogs / xfsprogs / f2fs-tools / btrfs-progs / zstd / dpkg / curl / coreutils / libc-bin / kmod）"
done
mkdir -p "$DESTINATION" || fail "建不了 $DESTINATION" "换一个可写的目录"
[[ -z "$(ls -A "$DESTINATION")" ]] || fail "$DESTINATION 不是空目录" "给一个新建的空目录，这个脚本不往已有的东西上叠"

# ── 一、ZFS 与 bcachefs 的用户态：下载、核 sha256、解包（不装进系统） ──
ZFS_POOL=http://archive.ubuntu.com/ubuntu/pool/main/z/zfs-linux
BCACHEFS_POOL=http://archive.ubuntu.com/ubuntu/pool/universe/b/bcachefs-tools
# 每行：URL、本地文件名、sha256（2026-09-15 下载时算的）。对不上说明仓库里的包换了：改表就是换版本，要另写登记。
PACKAGE_TABLE="$ZFS_POOL/zfsutils-linux_2.3.4-1ubuntu2_amd64.deb zfsutils-linux_2.3.4-1ubuntu2_amd64.deb d3f6d9a528abab203c33a57524a79be8ab9ae90c8b919438c2f4eca50e7cfc88
$ZFS_POOL/libzfs6linux_2.3.4-1ubuntu2_amd64.deb libzfs6linux_2.3.4-1ubuntu2_amd64.deb 751e6d461c10ba41f041a8901e10027028b598c2b09e7aca2d03a9b9843f7704
$ZFS_POOL/libzpool6linux_2.3.4-1ubuntu2_amd64.deb libzpool6linux_2.3.4-1ubuntu2_amd64.deb a3b14b3ad2c44c4c19c402ec2759827eff0cd62f0d04ec3d30c1342a64c1c8a3
$ZFS_POOL/libnvpair3linux_2.3.4-1ubuntu2_amd64.deb libnvpair3linux_2.3.4-1ubuntu2_amd64.deb 39441d1aaa59d458e36710454cec40fa972485b243a010200433bca6352914a7
$ZFS_POOL/libuutil3linux_2.3.4-1ubuntu2_amd64.deb libuutil3linux_2.3.4-1ubuntu2_amd64.deb 9b01291a01c2c6832e9a25b1dcd428cfe2fb0886c3bd52725d8baf22ef4e0955
$BCACHEFS_POOL/bcachefs-tools_24%2breally1.3.4-2build2_amd64.deb bcachefs-tools_24+really1.3.4-2build2_amd64.deb 351dcfb3c14daf6967fe36bfe5b98742b53515dd2a2339cb0f1fd07d25832334"
mkdir -p "$PACKAGES/debs" || fail "建不了 $PACKAGES" "设 E152_PACKAGE_CACHE 指一个可写的目录"
while read -r url file_name expected_sha256; do
  [[ -n "$url" ]] || continue
  package_file="$PACKAGES/debs/$file_name"
  if [[ ! -f "$package_file" ]]; then
    curl -sf --max-time 300 -o "$package_file.partial" "$url" || fail "下载失败：$url" "看网络；仓库里路径变了就是版本变了，换版本要另写登记"
    mv "$package_file.partial" "$package_file"
  fi
  actual_sha256="$(sha256sum "$package_file" | cut -d' ' -f1)"
  [[ "$actual_sha256" == "$expected_sha256" ]] \
    || fail "$file_name 的 sha256 是 $actual_sha256，表里是 $expected_sha256" "删掉 $package_file 重下；仍对不上就是仓库里的包换了，不许改表了事"
done <<< "$PACKAGE_TABLE"
PACKAGE_ROOT="$PACKAGES/root"
rm -rf "${PACKAGE_ROOT:?}"
mkdir -p "$PACKAGE_ROOT"
for package_file in "$PACKAGES"/debs/*.deb; do
  dpkg-deb -x "$package_file" "$PACKAGE_ROOT" || fail "解包 $package_file 失败" "删掉它重下"
done

# ── 二、二进制连同它们解析出来的全部动态库；包里解出来的库放到来宾的同名系统路径下 ──
PACKAGE_LIBRARY_PATH="$PACKAGE_ROOT/lib/x86_64-linux-gnu:$PACKAGE_ROOT/usr/lib/x86_64-linux-gnu"
install_binary_with_libraries() {   # <宿主上的路径> <来宾里的路径>
  local source_path="$1" guest_path="$2" resolution unresolved library library_guest_path
  resolution="$(LD_LIBRARY_PATH="$PACKAGE_LIBRARY_PATH" ldd "$source_path" 2>&1)" \
    || fail "ldd $source_path 失败：$resolution" "它得是一个动态链接的 ELF"
  unresolved="$(printf '%s\n' "$resolution" | grep -c 'not found')"
  [[ "$unresolved" == 0 ]] || fail "$source_path 有 $unresolved 个库解析不到" "LD_LIBRARY_PATH=$PACKAGE_LIBRARY_PATH ldd $source_path"
  install -D -m 0755 "$(readlink -f "$source_path")" "$DESTINATION$guest_path" || fail "装不进 $guest_path" "看 $DESTINATION 的权限"
  while read -r library; do
    [[ -n "$library" ]] || continue
    library_guest_path="${library#"$PACKAGE_ROOT"}"
    # 库一律带执行位：动态加载器 /lib64/ld-linux-x86-64.so.2 本身就是被内核执行的那个文件，
    # 没有执行位时来宾里每个动态程序都报 Permission denied（2026-09-15 冒烟跑七个配置全卡在这里）
    if [[ ! -e "$DESTINATION$library_guest_path" ]]; then
      install -D -m 0755 "$(readlink -f "$library")" "$DESTINATION$library_guest_path" || fail "装不进 $library_guest_path" "看 $DESTINATION 的权限"
    fi
  done < <(printf '%s\n' "$resolution" | awk '/=> \// {print $3} /^[[:space:]]*\/lib64\// {print $1}')
}
install_binary_with_libraries "$(command -v fio)" /usr/bin/fio
for tool in mkfs.ext4 mkfs.xfs mkfs.f2fs mkfs.btrfs; do
  # mkfs.ext4 是 mke2fs 的链接；按 mkfs.ext4 这个名字装进去，mke2fs 靠 argv[0] 认出要格式化成 ext4
  install_binary_with_libraries "$(command -v "$tool")" "/usr/sbin/$tool"
done
install_binary_with_libraries "$PACKAGE_ROOT/usr/sbin/bcachefs" /usr/sbin/bcachefs
install_binary_with_libraries "$PACKAGE_ROOT/usr/sbin/zpool" /usr/sbin/zpool
install_binary_with_libraries "$PACKAGE_ROOT/usr/sbin/zfs" /usr/sbin/zfs
# zfs 的 vdev 是事先分好的分区（登记第十节修订五），分区用本机 util-linux 的 sfdisk
install_binary_with_libraries "$(command -v sfdisk)" /usr/sbin/sfdisk
# 第二次正式跑的 md raid1（ext4 / XFS / F2FS 与裸阵列，跑前登记第十一节）
install_binary_with_libraries "$(command -v mdadm)" /usr/sbin/mdadm
install -D -m 0644 /etc/mke2fs.conf "$DESTINATION/etc/mke2fs.conf" || fail "拷不了 /etc/mke2fs.conf" "装 e2fsprogs"
ln -s /proc/self/mounts "$DESTINATION/etc/mtab"   # libzfs 挂载前要读 /etc/mtab
install -D -m 0755 "$SINGLEFS_BINARY" "$DESTINATION/usr/bin/first_transaction_on_device" || fail "拷不了 $SINGLEFS_BINARY" "看权限"

# ── 三、内核模块：modprobe 从 modules.dep 解出次序（含软依赖），zstd 解压成 .ko，来宾里 busybox insmod 直接吃 ──
MODULE_DIRECTORY="$DESTINATION/lib/modules/e152"
mkdir -p "$MODULE_DIRECTORY"
stage_modules() {   # <配置名> <模块名...>：写 <配置名>.order，一行一个来宾路径，依赖在前、同一个模块只写一次
  local configuration="$1" module plan kind module_path module_file order_file="$MODULE_DIRECTORY/$1.order"
  shift
  : > "$order_file"
  for module in "$@"; do
  plan="$(modprobe -S "$RELEASE" --show-depends "$module" 2>&1)" || fail "modprobe 解不出 $module：$plan" "modinfo -k $RELEASE $module 看它在不在"
  while read -r kind module_path _; do
    case "$kind" in
      insmod)
        module_file="$(basename "$module_path")"
        module_file="${module_file%.zst}"
        if [[ ! -f "$MODULE_DIRECTORY/$module_file" ]]; then
          case "$module_path" in
            *.ko.zst) zstd -q -dc "$module_path" > "$MODULE_DIRECTORY/$module_file" || fail "解压 $module_path 失败" "zstd -t $module_path 看它坏没坏" ;;
            *.ko) cp "$module_path" "$MODULE_DIRECTORY/$module_file" || fail "拷不了 $module_path" "看权限" ;;
            *) fail "不认识的模块文件：$module_path" "按 /boot/config-$RELEASE 的 CONFIG_MODULE_COMPRESS_* 补一条解压方式" ;;
          esac
        fi
        grep -qxF "/lib/modules/e152/$module_file" "$order_file" || echo "/lib/modules/e152/$module_file" >> "$order_file"
        ;;
      builtin) ;;
      *) fail "modprobe 输出了不认识的一行：$kind $module_path" "看 modprobe -S $RELEASE --show-depends $module 的原样输出" ;;
    esac
  done <<< "$plan"
  done
  [[ -s "$order_file" ]] || fail "$configuration 一个模块都没解出来" "modprobe -S $RELEASE --show-depends $*"
}
stage_modules xfs xfs
stage_modules f2fs f2fs
stage_modules btrfs btrfs
stage_modules bcachefs bcachefs
stage_modules zfs zfs
stage_modules md raid1          # 裸 md 阵列与 ext4-md（ext4 编进内核）
stage_modules xfs-md raid1 xfs
stage_modules f2fs-md raid1 f2fs

# ── 四、自检：用搭好的加载器、只给搭好的库，实际跑一次搭好的 fio ──
# 加载器没执行位、少一个库、库放错路径，这一步都当场红，不等进了虚机才发现（冒烟跑的毛病就是在这里漏过去的）
STAGED_LOADER="$DESTINATION/lib64/ld-linux-x86-64.so.2"
[[ -x "$STAGED_LOADER" ]] || fail "$STAGED_LOADER 没有执行位或不存在" "库要按 0755 装：来宾内核执行动态程序时先执行这个加载器"
staged_version="$("$STAGED_LOADER" --library-path "$DESTINATION/lib/x86_64-linux-gnu:$DESTINATION/usr/lib/x86_64-linux-gnu" "$DESTINATION/usr/bin/fio" --version 2>&1)" \
  || fail "用搭好的加载器与库跑不起搭好的 fio：$staged_version" "$STAGED_LOADER --list $DESTINATION/usr/bin/fio 看缺哪个库"
[[ "$staged_version" == fio-* ]] || fail "搭好的 fio 报的版本是「$staged_version」" "看 $DESTINATION/usr/bin/fio 是不是拷错了文件"

order_summary=""
for order_file in "$MODULE_DIRECTORY"/*.order; do
  order_summary="$order_summary $(basename "$order_file" .order)=$(wc -l < "$order_file")"
done
echo "  ✓ 附加根 $DESTINATION：$(find "$DESTINATION" -type f | wc -l) 个文件、$(du -sh "$DESTINATION" | cut -f1)；各配置的模块数：${order_summary# }"
