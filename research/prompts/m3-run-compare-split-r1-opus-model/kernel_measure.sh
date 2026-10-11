#!/usr/bin/env bash
# 攻方腿模型（m3-run-compare-split-r1）：量判器三个内核与两个 P14 探针入口展开之后的表达式数（kernel-size，数法照 crash_judge_dispatch.rs 的 kernel_inlined_size）。
# 先量三个现成内核、与仓里单测钉的值（334 042 / 395 566 / 352 481）逐个比，比上了探针的数才作数。
# 用法：kernel_measure.sh <副本根> <草稿目录>；只编一个只依赖 naga 的小程序（不编 checker 档、不碰显卡）。
set -euo pipefail
copy_root="$1"; work="$2"; model="$(cd "$(dirname "$0")" && pwd)"
source_directory="$copy_root/crates/singlefs-checker-tier/src"
(cd "$model/kernel-size" && CARGO_TARGET_DIR="$work/target-kernel-size" nice -n 19 cargo build --release --offline)
tool="$work/target-kernel-size/release/m3-run-compare-split-kernel-size"
mkdir -p "$work/ks"; cd "$work/ks"; rm -f stub-*.wgsl knobs.wgsl
printf 'const D_TOMBSTONE_CONTAINER_CAPACITY_KNOB: u32 = 0u;\nconst D_LINEAGE_CONTAINER_CAPACITY_KNOB: u32 = 0u;\nconst D_RETIREMENT_CONTAINER_CAPACITY_KNOB: u32 = 0u;\n' > knobs.wgsl
common="$source_directory/crash_judge_common.wgsl"; recovery="$source_directory/crash_judge_recovery.wgsl"
checker_common="$source_directory/crash_judge_checker_common.wgsl"
python3 -I "$model/stub_prelude.py" stub-recovery.wgsl "$common" "$recovery"
python3 -I "$model/stub_prelude.py" stub-scan.wgsl knobs.wgsl "$common" "$checker_common" "$source_directory/crash_judge_checker_scan.wgsl"
python3 -I "$model/stub_prelude.py" stub-walk.wgsl "$common" "$checker_common" "$source_directory/crash_judge_checker.wgsl"
python3 -I "$model/stub_prelude.py" stub-p14.wgsl "$common" "$recovery" "$model/p14_probe_entry.wgsl"
python3 -I "$model/stub_prelude.py" stub-merged.wgsl "$common" "$recovery" "$model/p14_merged_probe_entry.wgsl"
printf 'kernel=recovery '; nice -n 19 "$tool" main stub-recovery.wgsl "$common" "$recovery"
printf 'kernel=checker-scan '; nice -n 19 "$tool" main stub-scan.wgsl knobs.wgsl "$common" "$checker_common" "$source_directory/crash_judge_checker_scan.wgsl"
printf 'kernel=checker-walk '; nice -n 19 "$tool" main stub-walk.wgsl "$common" "$checker_common" "$source_directory/crash_judge_checker.wgsl"
printf 'probe=p14-alone '; nice -n 19 "$tool" p14_probe stub-p14.wgsl "$common" "$recovery" "$model/p14_probe_entry.wgsl"
printf 'probe=p14-merged-into-recovery '; nice -n 19 "$tool" merged_probe stub-merged.wgsl "$common" "$recovery" "$model/p14_merged_probe_entry.wgsl"
nice -n 19 "$tool" --each main stub-recovery.wgsl "$common" "$recovery" | grep -E 'name=(recover|walk_to_file|choose_system_configuration|choose_root|scan_journal|check_records|classify_oracle|begin_state|parse_system_configuration|load_rows_of_the_instance_table_chain|instance_table_chain_readable|newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to|read_unit_via_locations) '
nice -n 19 "$tool" --each main stub-scan.wgsl knobs.wgsl "$common" "$checker_common" "$source_directory/crash_judge_checker_scan.wgsl" | grep -E 'name=(begin_state|version_of_location|location_word|tab) ' | sed 's/^/scan /'
