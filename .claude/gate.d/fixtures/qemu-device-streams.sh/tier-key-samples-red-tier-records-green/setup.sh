#!/usr/bin/env bash
# 键样本现场（情形 red-tier-records-green，一档判完就记）：同 tier-key-samples-manifest-recorded 的那棵小树、environment.tsv、mode-parameters.tsv、listings/ 与 runs/；
# 两份对照登记的合成副本（.claude/kb/layout/01-first-txn.md、crates/singlefs-verification/tests/new_pool_file_creation_publish.rs）与六档预录四件套不由这里写，
# 照 direct-divergence-red 原样摆在样本里（direct 的 check.txt 里盘 1 那一行改成 device=2，别的五档原样）。另放 .qemu-key-samples/judge-prerecorded：
# 门禁头一趟照真跑那一条路逐档判两段判据、一档判完就记，direct 红、当场记红，另五档核过清单记绿；change.sh 什么都不改，第二趟 direct 现判（「上次判红」）、
# 别的五档复用，之后整道判红。弄坏开关 GATE_QEMU_DEVICE_STREAMS_BREAK=red-tier-stops-the-rest 下判到 direct 就退，这份样本判错。
set -euo pipefail
mkdir -p scripts/devices crates/singlefs-verification/src/bin .qemu-key-samples
cat > scripts/devices/vm-bench.sh <<'VMBENCH'
#!/usr/bin/env bash
# 键样本里的小 vm-bench.sh：只为算宿主脚本单元的键，样本里不会起它
set -uo pipefail
stage_initramfs_root() {
  local ird="$1"
  mkdir -p "$ird/bin"
  cp "$(command -v busybox)" "$ird/bin/busybox"
  cat > "$ird/init" <<'INIT'
#!/bin/sh
/bench $(cat /devs) $(cat /args); rc=$?
echo "SINGLEFS_EXIT=$rc"
INIT
}
run_one() {
  stage_initramfs_root "$1/irfs"
  timeout 60 "$QEMU_SYSTEM_BINARY" -kernel "$2" -initrd "$1/initramfs.cpio.gz"
}
QEMU_SYSTEM_BINARY=qemu-system-x86_64
run_one "$@"
VMBENCH
printf '[package]\nname = "singlefs-verification"\nversion = "0.1.0"\nedition = "2021"\n' > crates/singlefs-verification/Cargo.toml
cat > crates/singlefs-verification/src/bin/new_pool_file_creation_on_device.rs <<'RUST'
fn publish_the_second_version() -> u32 {
    2
}

fn publish_the_fourth_version() -> u32 {
    4
}

fn run_the_direct_mode() {
    println!("{}", publish_the_second_version());
}

fn run_the_skip_new_pool_file_creation_barrier_mode() {
    println!("{}", publish_the_second_version());
}

fn run_the_page_cache_mode() {
    println!("{}", publish_the_second_version());
}

fn run_the_file_overwrite_mode() {
    println!("{}", publish_the_second_version());
}

fn run_the_second_instance_mode() {
    println!("{}", publish_the_second_version());
}

fn run_the_raise_rollback_floor_mode() {
    println!("{}", publish_the_second_version());
    println!("{}", publish_the_fourth_version());
}

fn entry_of_the_mode(mode: &str) -> fn() {
    match mode {
        "direct" => run_the_direct_mode,
        "skip-new-pool-file-creation-barrier" => run_the_skip_new_pool_file_creation_barrier_mode,
        "page-cache" => run_the_page_cache_mode,
        "file-overwrite" => run_the_file_overwrite_mode,
        "second-instance" => run_the_second_instance_mode,
        _ => run_the_raise_rollback_floor_mode,
    }
}

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_default();
    entry_of_the_mode(&mode)();
}
RUST
printf 'fn main() {\n    println!("check");\n}\n' > crates/singlefs-verification/src/bin/new_pool_file_creation_device_log_check.rs
printf '样本内核镜像\n' > .qemu-key-samples/kernel.img
printf 'kernel\t.qemu-key-samples/kernel.img\nbusybox\t%s\tBusyBox v1.36.1 (样本) multi-call binary.\nqemu-system-x86_64\t%s\tQEMU emulator version 8.2.2 (样本)\ntoolchain\tcargo 1.98.0 (样本)\ntoolchain\trustc 1.98.0 (样本)\n' \
  "$(printf 'a%.0s' $(seq 64))" "$(printf 'b%.0s' $(seq 64))" > .qemu-key-samples/environment.tsv
for mode in direct skip-new-pool-file-creation-barrier page-cache file-overwrite second-instance raise-rollback-floor; do
  printf '%s\tVM_DISKS=2 VM_DISK_MB=4096\n' "$mode"
done > .qemu-key-samples/mode-parameters.tsv
# 条目按 vm-bench.sh 的 initramfs_manifest 打（LC_ALL=C 按路径排）；/bench 开跑前是空的占位文件（e3b0… 是空串的 sha256），跑的时候是真二进制
mkdir -p .qemu-key-samples/listings
for mode in direct skip-new-pool-file-creation-barrier page-cache file-overwrite second-instance raise-rollback-floor; do
  entries="$(printf 'args\tf\t664\t%s\nbench\tf\t775\tBENCH\nbin\td\t775\t-\nbin/busybox\tf\t755\t%s\nbin/sh\tl\t-\tbusybox\ndev\td\t775\t-\ndevs\tf\t664\t%s\ninit\tf\t775\t%s\nproc\td\t775\t-\nsys\td\t775\t-\n' \
    "$(printf '%s\n' "$mode" | sha256sum | cut -d' ' -f1)" "$(printf 'c%.0s' $(seq 64))" "$(printf 'd%.0s' $(seq 64))" "$(printf 'e%.0s' $(seq 64))")"
  { printf '\n══ 虚机 benchmark harness ══\n'
    printf '  \033[32m✓\033[0m %s\n' '内核 /boot/vmlinuz-样本' '虚机 2048M 内存 / 4 vCPU / 4096M virtio 盘' \
      'blklogwrites 一支用补好的 QEMU：/opt/qemu-样本/bin/qemu-system-x86_64（开跑前 qemu-blklogwrites-check.sh 核过）'
    printf 'kernel\t/boot/vmlinuz-样本\nqemu\t/opt/qemu-样本/bin/qemu-system-x86_64\n'
    printf '%s\n' "$entries" | sed -e "s/BENCH\$/$(printf '' | sha256sum | cut -d' ' -f1)/" -e 's/^/entry\t/'; } > ".qemu-key-samples/listings/$mode.txt"
  mkdir -p ".qemu-key-samples/runs/$mode"
  printf '%s\n' "$entries" | sed "s/BENCH\$/$(printf 'f%.0s' $(seq 64))/" > ".qemu-key-samples/runs/$mode/initramfs.manifest"
done
printf '#!/usr/bin/env bash\n# 第二趟之前什么都不改：六档的键与头一趟相同，头一趟记绿的五档复用、记红的 direct 现判\nexit 0\n' > .qemu-key-samples/change.sh
printf '样本档标记：头一趟照真跑那一条路逐档判样本根里的六档预录四件套，一档判完就记（门禁文件头「样本」red-tier-records-green）\n' > .qemu-key-samples/judge-prerecorded
