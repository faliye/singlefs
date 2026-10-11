#!/usr/bin/env bash
# 攻方腿模型（m3-run-compare-split-r2）：在草稿目录里建几棵探针树（副本的 Cargo.toml、Cargo.lock、crates/），各改一处：
#   base         原样
#   k-gpu        加一份「录入比对」内核宿主 crash_record_compare_gpu.rs（include_str! 读 crash_record_compare.wgsl），由 crash_judge_gpu.rs 引
#   k-gpu-edit   k-gpu 再把内核 WGSL 改一个字（== 改成 !=）
#   k-amp        同一份宿主只由 crash_amplification.rs 引（第一轮 C1 那种接法，自证的负对照）
#   t-readfault  读故障态那份测试文件里历史执行的 per_step_checker: PerStepChecker::Skipped 改成 Run（种子与规模那一行不变）
#   move         crash.rs 里 ROOT_RECORD_ROLLBACK_FLOOR_OFFSET 与 rollback_floor_written_by 原样挪进新文件 crash_rollback_floor.rs（纯搬家，未编译）
#   swap         crash.rs 那一处「later_start < sector_offset + SECTOR_BYTES」两边对调成「sector_offset < later_start + SECTOR_BYTES」（改了语义，未编译）
# 用法：build_trees.sh <副本根> <草稿目录>
set -euo pipefail
copy_root="$1"; work="$2"
trees=(base k-gpu k-gpu-edit k-amp t-readfault move swap)
for tree in "${trees[@]}"; do
  rm -rf "$work/trees/$tree"; mkdir -p "$work/trees/$tree"
  rsync -a "$copy_root/Cargo.toml" "$copy_root/Cargo.lock" "$work/trees/$tree/"
  rsync -a --exclude target --exclude 'target-*' "$copy_root/crates" "$work/trees/$tree/"
done
src() { printf '%s/trees/%s/crates/singlefs-checker-tier/src' "$work" "$1"; }
for tree in k-gpu k-gpu-edit k-amp; do
  printf '%s\n' '//! 探针：丙簇「录入比对」内核的宿主占位（只为看闭包收不收它）。' \
    'pub const RECORD_COMPARE_SOURCE: &str = include_str!("crash_record_compare.wgsl");' > "$(src $tree)/crash_record_compare_gpu.rs"
  printf 'fn compare(a: u32, b: u32) -> bool { return a == b; }\n' > "$(src $tree)/crash_record_compare.wgsl"
  printf '\npub mod crash_record_compare_gpu;\n' >> "$(src $tree)/lib.rs"
done
for tree in k-gpu k-gpu-edit; do
  printf '\n#[allow(unused_imports)]\nuse crate::crash_record_compare_gpu::RECORD_COMPARE_SOURCE as _PROBE_RECORD_COMPARE_SOURCE;\n' >> "$(src $tree)/crash_judge_gpu.rs"
done
printf 'fn compare(a: u32, b: u32) -> bool { return a != b; }\n' > "$(src k-gpu-edit)/crash_record_compare.wgsl"
printf '\n#[allow(unused_imports)]\nuse crate::crash_record_compare_gpu::RECORD_COMPARE_SOURCE as _PROBE_RECORD_COMPARE_SOURCE;\n' >> "$(src k-amp)/crash_amplification.rs"
python3 -I - "$work/trees" <<'PY'
import sys, pathlib
trees = pathlib.Path(sys.argv[1])
def replace_once(path, old, new):
    text = path.read_text()
    assert text.count(old) == 1, (path, old, text.count(old))
    path.write_text(text.replace(old, new))
readfault = trees / "t-readfault/crates/singlefs-checker-tier/tests/crash_injection_read_faulted_states_never_hold_the_witness_criterion.rs"
replace_once(readfault, "        per_step_checker: PerStepChecker::Skipped,\n        device_width: WIDTH,\n",
             "        per_step_checker: PerStepChecker::Run,\n        device_width: WIDTH,\n")
src = trees / "move/crates/singlefs-checker-tier/src"
crash = src / "crash.rs"
text = crash.read_text()
start = text.index("/// 根记录里回退下界 F 的偏移")
end = text.index("/// 写表下标 `later_index` 那次写盖掉")
moved = text[start:end]
text = text[:start] + text[end:]
text = text.replace("use singlefs_format::NODE_POINTER_BYTES;\n",
                    "use singlefs_format::NODE_POINTER_BYTES;\nuse crate::crash_rollback_floor::rollback_floor_written_by;\n", 1)
crash.write_text(text)
moved = moved.replace("fn rollback_floor_written_by(", "pub(crate) fn rollback_floor_written_by(", 1)
(src / "crash_rollback_floor.rs").write_text(
    "//! 探针：从 crash.rs 原样挪出来的回退下界读法（纯搬家）。\n"
    "use singlefs_core::address::CheckpointTxg;\nuse singlefs_format::NODE_POINTER_BYTES;\n\n" + moved)
with open(src / "lib.rs", "a") as handle:
    handle.write("\npub mod crash_rollback_floor;\n")
replace_once(trees / "swap/crates/singlefs-checker-tier/src/crash.rs",
             "&& later_start < sector_offset + SECTOR_BYTES\n",
             "&& sector_offset < later_start + SECTOR_BYTES\n")
PY
echo "TREES_BUILT ${trees[*]}"
