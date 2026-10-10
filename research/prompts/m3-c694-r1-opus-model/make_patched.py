#!/usr/bin/env python3
"""m3-c694-r1 攻方原型：把冻结副本的一份拷贝改成「臂由环境变量选」的样子（只在拷贝上改，不碰冻结副本）。

用法：make_patched.py <拷贝根>
- crates/singlefs-core/src/mount.rs：
  * 影子账读一条被抛弃根的账与映射现算重读仍不成时（今天在那里拒），按 OPUS_C694_ARM 选：
    空或 today：照今天拒；A：一律跳过（计数照记）；B：只在那条根的树表落点在现行账里有一条覆盖它、代 > 那条根的 txg 的记录时跳过，否则照拒。
  * 可写挂载重建分配器之后，OPUS_C694_ARM 含 C：把环里读得出、txg 高于施加之后那一版、没施加的记录点名的单元与四条新根指针的落点在内存里隔离
    （现行账里仍分配着的不隔；隔离位这次挂载内不清）。
- crates/singlefs-core/src/mounted_read.rs：OPUS_C695_ARM 选 open_pool_for_read 的次序：空或 today 照今天；R：先读树表、0 条就报
  TreeTableHasNoEntries，再读映射；S：映射根是空指针就当映射 0 条、不读，往下照今天。
- 加一个工作区成员 crates/opus-c694-probe（原型二进制，源码从模型目录拷）。
每一处替换都要求旧串恰好命中一次，不中就退 3。
"""
import pathlib
import shutil
import sys

root = pathlib.Path(sys.argv[1])
here = pathlib.Path(__file__).resolve().parent


def replace_once(path, old, new):
    text = path.read_text()
    count = text.count(old)
    if count != 1:
        print(f"{path}: 旧串命中 {count} 次（要恰好 1 次）：{old[:80]!r}", file=sys.stderr)
        sys.exit(3)
    path.write_text(text.replace(old, new))


mount = root / "crates/singlefs-core/src/mount.rs"
replace_once(
    mount,
    """                                Some(placements) => placements,
                                None => return Err(
                                    MountError::NewerStateStillUnreadableAfterOneReread(Box::new(""",
    """                                Some(placements) => placements,
                                None if opus_c694_skip_the_unreadable_abandoned_root(root, current_records) => continue,
                                None => return Err(
                                    MountError::NewerStateStillUnreadableAfterOneReread(Box::new(""",
)
replace_once(
    mount,
    """/// C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算（用户 2026-09-28 定，被攻过零轮）：被抛弃根的树表或分配记录树读不出时，""",
    """/// m3-c694-r1 攻方原型：臂由环境变量 OPUS_C694_ARM 选（`+` 连几个臂）。
fn opus_c694_arm_has(arm: &str) -> bool {
    std::env::var("OPUS_C694_ARM").unwrap_or_default().split('+').any(|name| name == arm)
}

/// 臂 A：一律跳过；臂 B：被抛弃根的树表落点（每条位置条目）在现行账里各有一条覆盖它、代 > 那条根的 txg 的记录才跳过。
fn opus_c694_skip_the_unreadable_abandoned_root(
    root: &RootRecord,
    current_records: &[AllocationRecord],
) -> bool {
    let skip = if opus_c694_arm_has("A") {
        true
    } else if opus_c694_arm_has("B") {
        root.tree_table.locations.iter().all(|location| {
            current_records.iter().any(|record| {
                record.device == location.device
                    && record.slot.0 <= location.slot.0
                    && location.slot.0 < record.slot.0 + u64::from(record.span_slots)
                    && record.generation > root.checkpoint_txg
            })
        })
    } else {
        false
    };
    if std::env::var("OPUS_C694_TRACE").is_ok() {
        eprintln!(
            "OPUS_C694_UNREADABLE_ABANDONED root=({},{}) skip={skip}",
            root.instance.0, root.checkpoint_txg.0
        );
    }
    skip
}

/// 臂 C：可写挂载时，环里读得出、txg 高于施加之后那一版的记录（没施加的）点名的单元与四条新根指针的落点在内存里隔离（现行账里仍分配着的不隔）。
#[allow(clippy::ptr_arg, reason = "原型")]
fn opus_c694_isolate_units_named_by_unapplied_records<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    unit_area_start: UnitAreaStart,
    allocator: &mut PoolAllocator,
    records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
    effective_root: &RootRecord,
) -> u64 {
    let allocated: BTreeSet<(u32, u64)> = allocator
        .records()
        .iter()
        .filter(|record| !record.is_released)
        .map(|record| (record.device.0, record.slot.0))
        .collect();
    let mut isolated = 0;
    for record in records.values() {
        if record.checkpoint_txg <= effective_root.checkpoint_txg {
            continue;
        }
        let mut located: Vec<([LocationEntry; 2], u64)> = vec![
            (record.new_tree_table.locations, 1),
            (record.new_mapping_root.locations, 1),
            (record.new_instance_table.locations, 1),
            (record.new_allocation_record_tree_root.locations, 1),
        ];
        for named in &record.named {
            let span = match named.unit_class {
                crate::unit::UNIT_CLASS_DATA | crate::unit::UNIT_CLASS_PACKED => 2,
                _ => 1,
            };
            located.push((named.locations, span));
        }
        for (locations, span) in located {
            for location in locations {
                if location.slot.0 == 0 {
                    continue;
                }
                if placement_lies_in_the_unit_area_of_its_device(
                    devices,
                    unit_area_start,
                    location.device,
                    location.slot,
                    span,
                )
                .is_err()
                {
                    continue;
                }
                if allocated.contains(&(location.device.0, location.slot.0)) {
                    continue;
                }
                allocator.isolate_abandoned(location.device, location.slot, span);
                isolated += 1;
            }
        }
    }
    if std::env::var("OPUS_C694_TRACE").is_ok() {
        eprintln!("OPUS_C694_C_ISOLATED placements={isolated}");
    }
    isolated
}

/// C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算（用户 2026-09-28 定，被攻过零轮）：被抛弃根的树表或分配记录树读不出时，""",
)
replace_once(
    mount,
    """    } = rebuilt?;
    allocator.set_deletion_switches(deletion_switches);""",
    """    } = rebuilt?;
    if opus_c694_arm_has("C") {
        opus_c694_isolate_units_named_by_unapplied_records(
            devices,
            unit_area_start,
            &mut allocator,
            &records,
            &effective_root,
        );
    }
    allocator.set_deletion_switches(deletion_switches);""",
)

mounted_read = root / "crates/singlefs-core/src/mounted_read.rs"
replace_once(
    mounted_read,
    """    let mapping_tree = read_code_two_tree(
        &root.mapping_root,""",
    """    // m3-c694-r1 攻方原型：OPUS_C695_ARM=R 先判树表 0 条再读映射；=S 只在映射根是空指针时先判树表 0 条。
    let opus_c695_arm = std::env::var("OPUS_C695_ARM").unwrap_or_default();
    if opus_c695_arm == "R"
        || (opus_c695_arm == "S" && root.mapping_root == NodePointer::empty_root())
    {
        let tree_table_bytes =
            read_unit_via_locations(reader, &root.tree_table.locations, node_bytes)?;
        let tree_table = parse_index_node(&tree_table_bytes).map_err(|error| {
            OpenPoolForReadFailure::UnitMalformed {
                what: "树表单元",
                error,
            }
        })?;
        if tree_table.entries.is_empty() {
            return Err(OpenPoolForReadFailure::TreeTableHasNoEntries);
        }
    }
    let mapping_tree = read_code_two_tree(
        &root.mapping_root,""",
)

# 原型二进制：工作区加一个成员，源码从模型目录拷。
probe = root / "crates/opus-c694-probe"
(probe / "src").mkdir(parents=True, exist_ok=True)
shutil.copy(here / "probe" / "Cargo.toml", probe / "Cargo.toml")
shutil.copy(here / "probe" / "main.rs", probe / "src" / "main.rs")
replace_once(
    root / "Cargo.toml",
    """    "crates/singlefs-reference-filesystem",
]""",
    """    "crates/singlefs-reference-filesystem",
    "crates/opus-c694-probe",
]""",
)
print("patched", root)
