#!/usr/bin/env bash
# 键样本现场（情形 manifest-differs）：同 tier-key-samples-init-heredoc 的那棵小树与 environment.tsv、mode-parameters.tsv；清单不摆 initramfs-manifest.tsv，
# 改摆两样真跑那一条路读的原样：listings/<档>.txt 是 vm-bench.sh --initramfs-manifest 的标准输出（抬头、「✓ 内核 …」几行、kernel、qemu 两行在条目前面），
# runs/<档>/initramfs.manifest 是 run_one 打包之前打的清单（只有条目，/bench 是真二进制的内容摘要）。门禁头一趟核过清单再记；与 tier-key-samples-manifest-recorded 只差 second-instance 那一档的 run_one 清单里 /init 的内容摘要（开跑之后 /init 变了），这一档不记、别的五档记下；
# change.sh 什么都不改，第二趟 second-instance 现判（无记录）、别的五档复用。
set -euo pipefail
mkdir -p scripts/devices scripts/experiments research/results .claude/kb/layout crates/singlefs-checker-tier/src/bin .qemu-key-samples
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
printf '[package]\nname = "singlefs-checker-tier"\nversion = "0.1.0"\nedition = "2021"\n' > crates/singlefs-checker-tier/Cargo.toml
cat > crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs <<'RUST'
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
printf 'fn main() {\n    println!("check");\n}\n' > crates/singlefs-checker-tier/src/bin/new_pool_file_creation_device_log_check.rs
printf 'E142|@driver_e142||e142-trimmed.out|exact\nE142|@driver_e142_p8g_crates_dump||e142-r24-p8g-crates-dump-2026-10-05.out|exact\n' > scripts/experiments/replay.sh
cat > research/results/e142-trimmed.out <<'E142'
E7RESULT name=segments path=mkfs operations=23 segments=12+1+1+1+4 closed_form=4114 kinds=[zero_fill×8,unit_write×4,barrier×2]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier×2]
E7RESULT name=segments path=transaction operations=35 segments=24+2+1+2 closed_form=16777223 kinds=[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]
E7RESULT name=root_record checkpoint_txg=3 instance=1 tree_identifier_watermark=19 rollback_floor=0 unmount_marker=false record_bytes=4096 back_chain=3937788698
E142
cat > .claude/kb/layout/01-first-txn.md <<'LAYOUT'
## 八、根槽写路径的段序列登记表（键样本里的合成副本：只留门禁 checker-tier-qemu-device-streams 读的「mkfs 种根」一行）

| 根槽写路径 | 录制流的段序列 | 出处 | 崩溃点重放枚举 |
|---|---|---|---|
| mkfs 种根 | `12+1+1+1+4`，23 次操作、4114 个崩溃状态，种类 `[zero_fill×8,unit_write×4,barrier×2]\|[root_record_fua]\|[root_record_fua]\|[root_record_fua]\|[system_configuration_slot×4,barrier×2]` | 键样本照 green 样本抄的 | 不在 |
LAYOUT
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
printf '#!/usr/bin/env bash\n# 第二趟之前什么都不改：六档的键与头一趟相同，记下了的都复用\nexit 0\n' > .qemu-key-samples/change.sh
# second-instance 那一档跑的时候 /init 与开跑前算键用的不同：清单里 init 那一行的内容摘要换掉（恰好换中一行，换不中退 1）
sed -i "s/^init\tf\t775\t$(printf 'e%.0s' $(seq 64))\$/init\tf\t775\t$(printf '9%.0s' $(seq 64))/" .qemu-key-samples/runs/second-instance/initramfs.manifest
[[ "$(grep -c "^init	f	775	$(printf '9%.0s' $(seq 64))\$" .qemu-key-samples/runs/second-instance/initramfs.manifest)" == 1 ]]
