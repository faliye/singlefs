#!/usr/bin/env python3
"""把乙甲代码轮第一轮攻方腿的探针拼进冻结副本的一份拷贝（只动拷贝，不动冻结副本与仓）。

用法：patch_the_copy.py <拷贝的仓根> <模型目录里的 probe 目录>
做的事（每一处原文都要在拷贝里恰好命中一次，命中不是一次就退 1、一个文件都不写）：
  1. crates/singlefs-core/src/mounted_session.rs：
     推串之前合取 7 那一判（empty_cluster_segments_below_the_floor_before_any_series）算出结果之后记一笔，开关关着时交 None；
     放不下的盘那一判（devices_short_even_with_everything_a_floor_raise_could_release）开关关着时交空表；
     推一串抬 F 之前记每块盘的全空聚簇段数，推成之后再记一次；
     refusal_is_short_of_space 开头加一道线程开关：开着时合取 7 自己的被拒当成空间不够（强推那一臂用）；
     槽对那一判的上界读数加一道线程开关：开着时照「每个放得开的槽各补一对」数（老上界那一臂用）；
     文件末尾拼上 probe/core_probe_module.rs。
  2. Cargo.toml：工作区成员加 crates/refusal-precheck-oracle-probe。
  3. 把 probe/Cargo.toml 与 probe/src/ 拷成 crates/refusal-precheck-oracle-probe/。
退出码：0 拼好；1 原文命中不是一次或拷贝不像冻结副本；2 用法错。
"""
import pathlib
import shutil
import sys

CONJUNCT_ORIGINAL = (
    "    admit_the_empty_cluster_segments_floor_on_every_device(\n"
    "        &empty_cluster_segments_on_each_device,\n"
    "        allocator\n"
    "            .compaction()\n"
    "            .switches\n"
    "            .empty_cluster_segments_floor\n"
    "            .segments(),\n"
    "    )\n"
    "    .err()\n"
    "}\n"
)
CONJUNCT_PATCHED = (
    "    let below_the_floor_before_any_series = admit_the_empty_cluster_segments_floor_on_every_device(\n"
    "        &empty_cluster_segments_on_each_device,\n"
    "        allocator\n"
    "            .compaction()\n"
    "            .switches\n"
    "            .empty_cluster_segments_floor\n"
    "            .segments(),\n"
    "    )\n"
    "    .err();\n"
    "    refusal_precheck_oracle_probe::note_conjunct_before_any_series(\n"
    "        below_the_floor_before_any_series.is_some(),\n"
    "    );\n"
    "    if refusal_precheck_oracle_probe::precheck_is_switched_off() {\n"
    "        return None;\n"
    "    }\n"
    "    below_the_floor_before_any_series\n"
    "}\n"
)
DEVICES_ORIGINAL = (
    ") -> Vec<DeviceShortEvenWithEverythingAFloorRaiseCouldRelease> {\n"
    "    let PublishError::SpaceAdmissionRefused(refused) = refusal else {\n"
    "        return Vec::new();\n"
    "    };\n"
)
DEVICES_PATCHED = (
    ") -> Vec<DeviceShortEvenWithEverythingAFloorRaiseCouldRelease> {\n"
    "    if refusal_precheck_oracle_probe::precheck_is_switched_off() {\n"
    "        return Vec::new();\n"
    "    }\n"
    "    let PublishError::SpaceAdmissionRefused(refused) = refusal else {\n"
    "        return Vec::new();\n"
    "    };\n"
)
SERIES_ORIGINAL = (
    "    match push_one_floor_raise_within_the_admission_budget(\n"
    "        parameters,\n"
    "        devices,\n"
    "        allocator,\n"
    "        Arc::make_mut(current),\n"
    "        shadow_ledger,\n"
    "        publishes_pushed_in_this_admission,\n"
    "    ) {\n"
    "        Ok(FloorRaisePushedWithinTheAdmissionBudget::Raised(raised)) => {\n"
)
SERIES_PATCHED = (
    "    let empty_cluster_segments_before_the_series =\n"
    "        refusal_precheck_oracle_probe::empty_cluster_segments_of(allocator);\n"
    "    match push_one_floor_raise_within_the_admission_budget(\n"
    "        parameters,\n"
    "        devices,\n"
    "        allocator,\n"
    "        Arc::make_mut(current),\n"
    "        shadow_ledger,\n"
    "        publishes_pushed_in_this_admission,\n"
    "    ) {\n"
    "        Ok(FloorRaisePushedWithinTheAdmissionBudget::Raised(raised)) => {\n"
    "            refusal_precheck_oracle_probe::note_series(\n"
    "                &empty_cluster_segments_before_the_series,\n"
    "                allocator,\n"
    "            );\n"
)
REFUSAL_ORIGINAL = (
    "pub fn refusal_is_short_of_space(refusal: &PublishError) -> bool {\n"
    "    match refusal {\n"
)
REFUSAL_PATCHED = (
    "pub fn refusal_is_short_of_space(refusal: &PublishError) -> bool {\n"
    "    if refusal_precheck_oracle_probe::conjunct_refusal_is_pushed_for()\n"
    "        && matches!(refusal, PublishError::EmptyClusterSegmentsBelowTheFloor(_))\n"
    "    {\n"
    "        return true;\n"
    "    }\n"
    "    match refusal {\n"
)
OLD_BOUND_ORIGINAL = (
    "            slot_pairs_once_everything_is_released: device_map\n"
    "                .slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(\n"
    "                    cluster_segments,\n"
    "                ),\n"
)
OLD_BOUND_PATCHED = (
    "            slot_pairs_once_everything_is_released: if refusal_precheck_oracle_probe::the_old_slot_pair_bound_is_used() {\n"
    "                SlotPairCount(\n"
    "                    device_map.unblocked_slot_pairs_outside_the_cluster_segments(cluster_segments)\n"
    "                        + device_map.deferred_slots()\n"
    "                        + device_map.isolated_slots()\n"
    "                        + (device_map.free_slots() - device_map.unblocked_slots()),\n"
    "                )\n"
    "            } else {\n"
    "                device_map.slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(\n"
    "                    cluster_segments,\n"
    "                )\n"
    "            },\n"
)
MEMBER_ORIGINAL = '    "crates/singlefs-mount-runtime",\n'
MEMBER_PATCHED = MEMBER_ORIGINAL + '    "crates/refusal-precheck-oracle-probe",\n'
CRATE_NAME = "refusal-precheck-oracle-probe"


def replace_once(text, original, patched, where):
    count = text.count(original)
    if count != 1:
        print(f"  ✗ {where}：原文命中 {count} 次，要恰好 1 次", file=sys.stderr)
        sys.exit(1)
    return text.replace(original, patched)


def main():
    if len(sys.argv) != 3:
        print(__doc__, file=sys.stderr)
        sys.exit(2)
    copy = pathlib.Path(sys.argv[1])
    probe = pathlib.Path(sys.argv[2])
    session_path = copy / "crates/singlefs-core/src/mounted_session.rs"
    manifest_path = copy / "Cargo.toml"
    if not session_path.is_file() or not manifest_path.is_file():
        print(f"  ✗ {copy} 不像冻结副本的拷贝（缺 mounted_session.rs 或 Cargo.toml）", file=sys.stderr)
        sys.exit(1)
    session = session_path.read_text(encoding="utf-8")
    session = replace_once(session, CONJUNCT_ORIGINAL, CONJUNCT_PATCHED, "推串之前合取 7 那一判")
    session = replace_once(session, DEVICES_ORIGINAL, DEVICES_PATCHED, "放不下的盘那一判")
    session = replace_once(session, SERIES_ORIGINAL, SERIES_PATCHED, "推一串抬 F")
    session = replace_once(session, REFUSAL_ORIGINAL, REFUSAL_PATCHED, "合取 7 的被拒是不是空间不够")
    session = replace_once(session, OLD_BOUND_ORIGINAL, OLD_BOUND_PATCHED, "槽对那一判的上界")
    session += (probe / "core_probe_module.rs").read_text(encoding="utf-8")
    manifest = replace_once(
        manifest_path.read_text(encoding="utf-8"), MEMBER_ORIGINAL, MEMBER_PATCHED, "工作区成员"
    )
    crate_directory = copy / "crates" / CRATE_NAME
    if crate_directory.exists():
        print(f"  ✗ {crate_directory} 已在：给一份新拷贝", file=sys.stderr)
        sys.exit(1)
    session_path.write_text(session, encoding="utf-8")
    manifest_path.write_text(manifest, encoding="utf-8")
    (crate_directory / "src").mkdir(parents=True)
    shutil.copyfile(probe / "Cargo.toml", crate_directory / "Cargo.toml")
    sources = sorted((probe / "src").glob("*.rs"))
    for source in sources:
        shutil.copyfile(source, crate_directory / "src" / source.name)
    print(f"PATCH_THE_COPY copy={copy} probe_sources={len(sources)}")


if __name__ == "__main__":
    main()
