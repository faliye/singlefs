
// ───────── 以下是辩方腿 m4-validate-code-r2-sonnet 加的探针（上面是冻结副本里同名用例文件的辅助函数，逐字照搬）─────────
// 只打 PROBE 行，不断言产品行为。
mod common_pointer_head_corruption;

use common_pointer_head_corruption::{the_slot_of, tree_table_entry_of_kind};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::records::TREE_KIND_EXTENT;
use singlefs_core::mount::RollbackError;
use std::panic::{catch_unwind, AssertUnwindSafe};

fn third_content() -> Vec<u8> {
    (0..2100)
        .map(|index| u8::try_from((index * 7 + 11) % 239).expect("小于 256"))
        .collect()
}

fn checker_text(pool: &BuiltPool) -> String {
    let verdicts = check_pool_image(&pool.memory_pool());
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(invariant, _)| *invariant)
        .collect();
    let i317 = verdicts
        .iter()
        .find(|(invariant, _)| *invariant == "I-3.17")
        .map(|(_, verdict)| format!("{verdict:?}").chars().take(60).collect::<String>())
        .unwrap_or_else(|| "absent".to_string());
    format!("checker_checked={} checker_violated={violated:?} I-3.17={i317}", verdicts.len())
}

fn records_text(pool: &BuiltPool, label: &str, slot: SlotNumber) -> String {
    let image = pool.memory_pool();
    let system_configuration = choose_system_configuration(&image).expect("择系统配置");
    let newest = choose_root(&image, &system_configuration).expect("最新根");
    match allocation_records_under_root(&image, &newest) {
        Ok(records) => {
            let parts: Vec<String> = records
                .iter()
                .filter(|record| record.slot == slot)
                .map(|record| {
                    format!(
                        "dev{}:released={},gen={},span={},mark={:#06x}",
                        record.device.0,
                        record.is_released,
                        record.generation.0,
                        record.span_slots,
                        u16::from_le_bytes([record.to_bytes()[10], record.to_bytes()[11]]) & 0x4000
                    )
                })
                .collect();
            format!("{label}(slot {}): [{}]", slot.0, parts.join(" "))
        }
        Err(failure) => format!("{label}: tree_read_ERR({failure:?})").chars().take(200).collect(),
    }
}

fn corrupt_the_copy_on_device_one(pool: &mut BuiltPool, slot: SlotNumber) -> Vec<u8> {
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let intact = read_copy_at(devices, DeviceIdentity(1), slot);
    let mut corrupted = intact.clone();
    corrupted[4096] ^= 0xff;
    write_copy_at(devices, DeviceIdentity(1), slot, &corrupted);
    intact
}

fn restore_the_copy_on_device_one(pool: &mut BuiltPool, slot: SlotNumber, intact: &[u8]) {
    write_copy_at(pool.devices.as_mut().expect("镜像还开着"), DeviceIdentity(1), slot, intact);
}

fn rollback_text(pool: &mut BuiltPool, target_txg: CheckpointTxg) -> String {
    let publish_parameters = parameters();
    let result = catch_unwind(AssertUnwindSafe(|| {
        roll_back_by_a_forward_publish(
            &publish_parameters,
            pool.devices.as_mut().expect("镜像还开着"),
            &mut pool.allocator,
            &mut pool.output,
            RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: target_txg },
        )
    }));
    match result {
        Ok(Ok(_)) => "rollback=ok".to_string(),
        Ok(Err(refusal)) => format!("rollback=REFUSED {}", describe(&refusal)),
        Err(_) => "rollback=PANIC".to_string(),
    }
}

fn describe(refusal: &RollbackError) -> String {
    format!("{refusal:?}").chars().take(220).collect()
}

/// 一次发布隔离两个单元（数据单元与 extent 树根，盘 1 上那两份都改坏）：两个槽各自两块盘都置记号；回退到 A 清两个槽。
#[test]
fn probe_two_units_quarantined_in_one_publish_then_rolled_back() {
    let mut pool = build_pool("probe-two-quarantined");
    let first = pool.output.clone();
    let data_slot = first.data_pointers[0].locations[0].slot;
    let extent_root_slot = the_slot_of(&tree_table_entry_of_kind(&pool.memory_pool(), TREE_KIND_EXTENT).root);
    let intact_data = corrupt_the_copy_on_device_one(&mut pool, data_slot);
    let intact_node = corrupt_the_copy_on_device_one(&mut pool, extent_root_slot);
    let second = publish_overwrite_in_process(
        &mut pool,
        &first,
        &second_content(),
        FIXED_WRITE_TIME_SECONDS + SECONDS_BETWEEN_THE_TWO_WRITES,
        InstanceGeneration(1),
    )
    .expect("两份改坏时覆盖写照成");
    let quarantined_slots: Vec<(u64, u64)> = second
        .quarantined_after_release_checksum_mismatch
        .iter()
        .map(|copy| (copy.device.0.into(), copy.placement.slot.0))
        .collect();
    eprintln!("PROBE name=two_units quarantined_copies={} {:?}", quarantined_slots.len(), quarantined_slots);
    pool.output = second;
    eprintln!("PROBE name=two_units_after_quarantine {} {} {}", records_text(&pool, "data", data_slot), records_text(&pool, "extent_root", extent_root_slot), checker_text(&pool));
    restore_the_copy_on_device_one(&mut pool, data_slot, &intact_data);
    restore_the_copy_on_device_one(&mut pool, extent_root_slot, &intact_node);
    let text = rollback_text(&mut pool, first.root.checkpoint_txg);
    eprintln!("PROBE name=two_units_after_rollback_to_A {text} {} {} {}", records_text(&pool, "data", data_slot), records_text(&pool, "extent_root", extent_root_slot), checker_text(&pool));
}

/// A（盘 1 数据单元那份改坏）→ B（隔离 A 的数据单元）→ C（再覆盖写）。回退到 B（B 不引用被隔离的那个单元）：记号该留着；
/// 再回退到 A（引用它，盘 1 那份先写回原样）：记号该清掉。
#[test]
fn probe_rollback_to_a_version_that_does_not_reference_the_quarantined_unit() {
    let mut pool = build_pool("probe-rollback-not-referencing");
    let first = pool.output.clone();
    let data_slot = first.data_pointers[0].locations[0].slot;
    let intact = corrupt_the_copy_on_device_one(&mut pool, data_slot);
    let second = publish_overwrite_in_process(&mut pool, &first, &second_content(), FIXED_WRITE_TIME_SECONDS + 60, InstanceGeneration(1))
        .expect("B");
    let third = publish_overwrite_in_process(&mut pool, &second, &third_content(), FIXED_WRITE_TIME_SECONDS + 120, InstanceGeneration(1))
        .expect("C");
    let (txg_a, txg_b) = (first.root.checkpoint_txg, second.root.checkpoint_txg);
    pool.output = third;
    eprintln!("PROBE name=abc_after_c {} {}", records_text(&pool, "A_data", data_slot), checker_text(&pool));
    let text = rollback_text(&mut pool, txg_b);
    eprintln!("PROBE name=abc_after_rollback_to_B {text} {} {}", records_text(&pool, "A_data", data_slot), checker_text(&pool));
    restore_the_copy_on_device_one(&mut pool, data_slot, &intact);
    let text = rollback_text(&mut pool, txg_a);
    eprintln!("PROBE name=abc_after_rollback_to_A {text} {} {}", records_text(&pool, "A_data", data_slot), checker_text(&pool));
}

/// 回退到引用被隔离单元的 A，而盘 1 那份没写回原样（仍是坏的）：逐盘验该拒；拒了之后分配器与盘上的记号都不该变。
#[test]
fn probe_rollback_refused_leaves_the_mark_in_place() {
    let mut pool = build_pool("probe-rollback-refused");
    let first = pool.output.clone();
    let data_slot = first.data_pointers[0].locations[0].slot;
    let _intact = corrupt_the_copy_on_device_one(&mut pool, data_slot);
    let second = publish_overwrite_in_process(&mut pool, &first, &second_content(), FIXED_WRITE_TIME_SECONDS + 60, InstanceGeneration(1))
        .expect("B");
    pool.output = second;
    let text = rollback_text(&mut pool, first.root.checkpoint_txg);
    let allocator_marks: Vec<String> = [DeviceIdentity(0), DeviceIdentity(1)]
        .iter()
        .map(|device| {
            let record = pool.allocator.record_for(*device, data_slot);
            format!("dev{}:{:?}", device.0, record.map(|record| (record.is_released, record.quarantine_mark)))
        })
        .collect();
    eprintln!("PROBE name=refused_rollback {text} allocator_after={allocator_marks:?} {} {}", records_text(&pool, "A_data", data_slot), checker_text(&pool));
}

/// 把隔离着的镜像（和一份没隔离的对照）按扇区倒成二进制文件，给「不认记号的旧读者」那一步（旧代码从 HEAD 导出，另编）读。
/// 格式：u64 小端盘字节数；之后每条 u32 小端设备号、u64 小端扇区号、512 字节。不设 M4R2_DUMP_DIR 时什么都不做。
fn dump_image(image: &singlefs_harness::memory_pool::MemoryPool, path: &std::path::Path) {
    let mut bytes = image.device_size_in_bytes.to_le_bytes().to_vec();
    for (identity, device) in &image.devices {
        for (sector, content) in device.written_sectors() {
            bytes.extend_from_slice(&u32::from(identity.0).to_le_bytes());
            bytes.extend_from_slice(&sector.to_le_bytes());
            bytes.extend_from_slice(content);
        }
    }
    std::fs::write(path, bytes).expect("倒镜像");
}

#[test]
fn probe_dump_the_marked_image_and_a_control_for_the_old_reader() {
    let Ok(directory) = std::env::var("M4R2_DUMP_DIR") else {
        return;
    };
    let directory = std::path::PathBuf::from(directory);
    let (marked, data_slot, _) = pool_with_the_data_unit_of_the_first_version_quarantined("probe-dump-marked");
    dump_image(&marked.memory_pool(), &directory.join("marked.bin"));
    let mut control = build_pool("probe-dump-control");
    let first = control.output.clone();
    let second = publish_overwrite_in_process(&mut control, &first, &second_content(), FIXED_WRITE_TIME_SECONDS + 60, InstanceGeneration(1))
        .expect("对照的覆盖写");
    control.output = second;
    dump_image(&control.memory_pool(), &directory.join("control.bin"));
    eprintln!("PROBE name=dump marked_data_slot={} {}", data_slot.0, records_text(&marked, "marked", data_slot));
}
