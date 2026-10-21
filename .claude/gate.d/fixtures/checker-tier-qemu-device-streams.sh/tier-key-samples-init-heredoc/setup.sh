#!/usr/bin/env bash
# 键样本现场（情形 init-heredoc）：一棵小树（宿主侧 vm-bench.sh、来宾与宿主检查两个 bin、布局表八与钉反向链的用例文件）与 .qemu-key-samples/ 那几份；
# 门禁头一趟六档照绿记下，跑 change.sh 改 vm-bench.sh 里 /init 的 here-doc，第二趟六档都作废（宿主脚本单元变了）。
set -euo pipefail
mkdir -p scripts/devices .claude/kb/layout crates/singlefs-checker-tier/src/bin crates/singlefs-harness/tests .qemu-key-samples
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
cat > crates/singlefs-harness/tests/new_pool_file_creation_publish.rs <<'RUST'
// 键样本里的合成副本：门禁 checker-tier-qemu-device-streams 只读下面这一条常量（新池新建文件那条记录反向链的对照），值照 green 样本
const E142_BACK_CHAIN_OF_NEW_POOL_FILE_CREATION: u32 = 3_937_788_698;
RUST
cat > .claude/kb/layout/01-first-txn.md <<'LAYOUT'
## 八、根槽写路径的段序列登记表（键样本里的合成副本：照 green 样本抄的，只留门禁 checker-tier-qemu-device-streams 读的那几处登记）

产物的 `name=segments` 四行（`path=instance_acquisition / warm_up / transaction / post_mkfs_stream`）；四样登记值都照 2026-10-02 录样本那一趟虚机输出（PROVENANCE.txt）。

| 根槽写路径 | 录制流的段序列 | 出处 | 崩溃点重放枚举 |
|---|---|---|---|
| mkfs 种根 | `12+1+1+1+4`，23 次操作、4114 个崩溃状态，种类 `[zero_fill×8,unit_write×4,barrier×2]\|[root_record_fua]\|[root_record_fua]\|[root_record_fua]\|[system_configuration_slot×4,barrier×2]` | 2026-10-02 录样本那一趟虚机输出的 path=mkfs 行（PROVENANCE.txt） | 不在 |
| 第一次可写挂载取号 | `2`，2 次操作、4 个崩溃状态，种类 `[system_configuration_slot×2]` | 产物 `name=segments path=instance_acquisition`（照 2026-10-02 录样本那一趟虚机输出，PROVENANCE.txt） | 在 |
| 空发布（暖机，第一版 2 次） | `2+1+2+2+1+2`，20 次操作、15 个崩溃状态，种类 `[journal_record×2,barrier×4]\|[root_record_fua]\|[system_configuration_slot×2,barrier×2]\|[journal_record×2,barrier×2]\|[root_record_fua]\|[system_configuration_slot×2,barrier×2]` | 产物 `name=segments path=warm_up`（照 2026-10-02 录样本那一趟虚机输出，PROVENANCE.txt） | 在 |
| 普通发布（新池新建文件） | `24+2+1+2`，35 次操作、16777223 个崩溃状态，种类 `[unit_write×24,barrier×2]\|[journal_record×2,barrier×2]\|[root_record_fua]\|[system_configuration_slot×2,barrier×2]` | 产物 `name=segments path=transaction`（照 2026-10-02 录样本那一趟虚机输出，PROVENANCE.txt） | 在 |

⚠️ 新池新建文件那条流是整条流 `2+2+1+2+2+1+2+24+2+1+2`、16777240 个状态（种类 `[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]`；57 次操作）。
LAYOUT
printf '样本内核镜像\n' > .qemu-key-samples/kernel.img
printf 'kernel\t.qemu-key-samples/kernel.img\nbusybox\t%s\tBusyBox v1.36.1 (样本) multi-call binary.\nqemu-system-x86_64\t%s\tQEMU emulator version 8.2.2 (样本)\ntoolchain\tcargo 1.98.0 (样本)\ntoolchain\trustc 1.98.0 (样本)\n' \
  "$(printf 'a%.0s' $(seq 64))" "$(printf 'b%.0s' $(seq 64))" > .qemu-key-samples/environment.tsv
printf 'bin\td\t775\t-\nbin/busybox\tf\t755\t%s\nbench\tf\t775\t<被测二进制，见来宾程序单元>\ndevs\tf\t664\t%s\ninit\tf\t775\t%s\n' \
  "$(printf 'c%.0s' $(seq 64))" "$(printf 'd%.0s' $(seq 64))" "$(printf 'e%.0s' $(seq 64))" > .qemu-key-samples/initramfs-manifest.tsv
for mode in direct skip-new-pool-file-creation-barrier page-cache file-overwrite second-instance raise-rollback-floor; do
  printf '%s\tVM_DISKS=2 VM_DISK_MB=4096\n' "$mode"
done > .qemu-key-samples/mode-parameters.tsv
cat > .qemu-key-samples/change.sh <<'CHANGE'
#!/usr/bin/env bash
# 第二趟之前：改 vm-bench.sh 里 /init 的 here-doc 一行
set -euo pipefail
sed -i 's|^echo "SINGLEFS_EXIT=$rc"$|echo "SINGLEFS_EXIT=$rc" >/dev/console|' scripts/devices/vm-bench.sh
grep -q '>/dev/console' scripts/devices/vm-bench.sh
CHANGE
