//! C322（取号那一步的屏障怎么放没有条款） 第一轮反推腿的模型 A：用仓里现成的 mkfs / 取号 / 暖机 / 第一个事务
//! （crates/singlefs-core），在内存盘（singlefs-harness 的 SparseBlockDevice）上重录 mkfs 之后那条流，
//! 按 crash.rs 的 writes_and_segments 同一条切段规则逐个枚举崩溃状态，逐状态看：两盘择到的超级块
//! （checker 的 chosen_superblocks）、每盘两槽、根环（valid_roots）、journal（recovery 的 scan_journal）、
//! 持久单元头里的写序实例代号。然后把「暖机第一次空发布开头那道屏障」摘掉再枚举一遍（甲把它当持久点）。
//! 只读仓里的 crate，不改它们。

use std::collections::BTreeSet;

use singlefs_checker::check_superblock_slot;
use singlefs_checker::image::{chosen_superblocks, valid_roots, ImageReader, InvariantVerdict};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::DeviceIdentity;
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::journal::JournalRecord;
use singlefs_core::make_filesystem::{
    make_filesystem, MakeFilesystemParameters, INSTANCE_TABLE_SLOT, MKFS_INSTANCE_GENERATION,
    TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::recovery::{choose_superblock, recover, scan_journal, JournalPolicy};
use singlefs_core::root_record::RootRecord;
use singlefs_core::superblock::FormatTimeGeometry;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter,
};
use singlefs_core::unit::{
    parse_data_unit, parse_index_node, parse_packed_unit, unit_filesystem_identifier,
};
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_harness::crash::{
    writes_and_segments, CrashImage, MemoryPool, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{
    RecordedOperationKind, RecordingBlockDevice, RetainedOperation, SharedStream,
};

/// 与 crates/singlefs-harness/tests/common/mod.rs 同参数：4 GiB 两盘、E142 的 fsid 与写入时间、3000 字节的第一个文件。
const IMAGE_BYTES: u64 = 4 << 30;
const FILE_BYTES: usize = 3000;
const FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x34, 0x32, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00,
];
const FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_000;

fn parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: FormatTimeGeometry {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
        },
    }
}

fn fixed_geometry() -> FixedGeometry {
    FixedGeometry {
        fixed_structure_slot_spacing: 4096,
        journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
    }
}

fn file_content() -> Vec<u8> {
    (0..FILE_BYTES)
        .map(|byte_position| u8::try_from(byte_position % 251).expect("小于 256"))
        .collect()
}

/// mkfs → 取号 → 暖机 → 第一个事务（与 tests/common 的 build_pool 同一串调用，盘换成内存盘）；
/// 返回 mkfs 之后的基线镜像与 mkfs 之后的全部步（带内容）。
fn record_first_mount_stream() -> (MemoryPool, Vec<RetainedOperation>) {
    let parameters = parameters();
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = (0..2u32)
        .map(|device_number| {
            (
                DeviceIdentity(device_number),
                RecordingBlockDevice::with_shared_stream(
                    DeviceIdentity(device_number),
                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mkfs_operation_count = stream.operations().len();
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(&[
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    ]);
    let content = file_content();
    {
        let mut pool = PoolWriter {
            parameters: &parameters,
            devices: &mut devices,
        };
        let instance = acquire_instance(&mut pool, MKFS_INSTANCE_GENERATION).expect("取号");
        let warm_up_output = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
        publish_first_file(
            &mut pool,
            &mut allocator,
            &genesis.root,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warm_up_output.last_record_bytes,
        )
        .expect("第一个事务");
    }
    let operations = stream.retained_operations();
    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    base.apply(&operations[..mkfs_operation_count]);
    (base, operations[mkfs_operation_count..].to_vec())
}

/// 写表里每一条的名字：first-txn-layout.md 零那张写清单的步号（a1 / w1..w6 / t1..t11），带盘号。
fn layout_labels(writes: &[RetainedWrite]) -> Vec<String> {
    let superblock_tags = ["a1", "a1", "w3", "w3", "w6", "w6", "t11", "t11"];
    let journal_tags = ["w1", "w1", "w4", "w4", "t9", "t9"];
    let root_tags = ["w2", "w5", "t10"];
    let unit_tags = [
        "t1", "t1", "t2", "t2", "t3", "t3", "t4", "t4", "t5", "t5", "t6", "t6", "t7", "t7", "t8", "t8",
    ];
    let mut seen = [0usize; 4];
    writes
        .iter()
        .map(|write| {
            let tag = match write.kind {
                StepKind::SuperblockSlot => {
                    seen[0] += 1;
                    superblock_tags[seen[0] - 1]
                }
                StepKind::JournalRecord => {
                    seen[1] += 1;
                    journal_tags[seen[1] - 1]
                }
                StepKind::RootRecordFua => {
                    seen[2] += 1;
                    root_tags[seen[2] - 1]
                }
                StepKind::UnitWrite => {
                    seen[3] += 1;
                    unit_tags[seen[3] - 1]
                }
                StepKind::Barrier => "barrier",
            };
            format!("{tag}@disk{}", write.device.0)
        })
        .collect()
}

/// 一次写落盘之后在盘上带着的实例代号，从写的内容直接解（不经恢复代码）。
fn instance_carried_by(write: &RetainedWrite) -> u32 {
    match write.kind {
        StepKind::SuperblockSlot => {
            check_superblock_slot(&write.bytes)
                .expect("超级块槽自证")
                .journal_instance
        }
        StepKind::RootRecordFua => {
            RootRecord::parse_slot(&write.bytes, &FILESYSTEM_IDENTIFIER)
                .expect("根记录自证")
                .instance
                .0
        }
        StepKind::JournalRecord => {
            JournalRecord::parse(&write.bytes, unit_filesystem_identifier(&FILESYSTEM_IDENTIFIER))
                .expect("journal 记录自证")
                .instance
                .0
        }
        StepKind::UnitWrite => match write.bytes[6] {
            1 => parse_data_unit(&write.bytes).expect("码 1").write_order.instance.0,
            2 => parse_index_node(&write.bytes).expect("码 2").instance.0,
            3 => parse_packed_unit(&write.bytes).expect("码 3").write_order.instance.0,
            unit_class => panic!("没登记的单元类 {unit_class}"),
        },
        StepKind::Barrier => panic!("写表里没有屏障"),
    }
}

/// 枚举：前面的段全持久 + 当前段任意真子集，最后再加全部持久那一个（crash.rs 的 enumerate_layer0_selecting 同形）。
fn for_each_crash_state(write_total: usize, segments: &[Vec<usize>], visit: &mut dyn FnMut(&[bool])) {
    let mut persisted_before = vec![false; write_total];
    for segment in segments {
        let full_mask: u64 = (1u64 << segment.len()) - 1;
        for mask in 0..full_mask {
            let mut persisted = persisted_before.clone();
            for (bit, write_index) in segment.iter().enumerate() {
                if mask & (1u64 << bit) != 0 {
                    persisted[*write_index] = true;
                }
            }
            visit(&persisted);
        }
        for write_index in segment {
            persisted_before[*write_index] = true;
        }
    }
    visit(&persisted_before);
}

/// 各盘择到的那一份超级块：(槽世代号, 实例代号)，用 checker 的 chosen_superblocks（每盘两槽里校验和过且世代号大的那个）。
fn chosen_generation_and_instance(image: &CrashImage<'_>) -> [(u64, u32); 2] {
    let chosen = chosen_superblocks(image);
    assert_eq!(chosen.len(), 2, "两块盘");
    let on_disk = |position: usize| {
        let (_, choice) = &chosen[position];
        let (view, _) = choice.as_ref().expect("两盘都有有效的超级块槽");
        (view.slot_generation, view.journal_instance)
    };
    [on_disk(0), on_disk(1)]
}

/// 每盘两槽里全部自证过的超级块带着的实例代号（不只是择到的那一份）。
fn every_valid_superblock_instance(image: &CrashImage<'_>) -> BTreeSet<u32> {
    let mut instances = BTreeSet::new();
    for device in ImageReader::devices(image) {
        for offset in [0u64, 4096] {
            if let Some(bytes) = ImageReader::read(image, device, offset, 4096) {
                if let Ok(view) = check_superblock_slot(&bytes) {
                    instances.insert(view.journal_instance);
                }
            }
        }
    }
    instances
}

fn persisted_labels(labels: &[String], persisted: &[bool]) -> String {
    labels
        .iter()
        .zip(persisted)
        .filter(|(_, is_persisted)| **is_persisted)
        .map(|(label, _)| label.as_str())
        .collect::<Vec<_>>()
        .join(",")
}

/// 原样的那条流：逐状态看两盘择到的超级块（世代号、实例代号）、J1 的原句状态、各种「下一次取号」取法撞不撞号。
#[allow(clippy::too_many_lines, reason = "一次枚举里各计数就地累加，拆开会把同一个状态读好几遍")]
fn report_original_stream(base: &MemoryPool, operations: &[RetainedOperation], geometry: &FixedGeometry) {
    let (writes, segments) = writes_and_segments(operations, geometry);
    let labels = layout_labels(&writes);
    let carried: Vec<u32> = writes.iter().map(instance_carried_by).collect();
    println!(
        "ORIGINAL segments={:?} writes={} carried_instances={carried:?}",
        segments.iter().map(Vec::len).collect::<Vec<_>>(),
        writes.len()
    );
    assert!(labels[0] == "a1@disk0" && labels[1] == "a1@disk1", "写表开头是取号那两次超级块写");
    let mut states = 0u64;
    let mut instances_unequal = 0u64;
    let mut generations_unequal = 0u64;
    let mut order_conflicts = 0u64;
    let mut generation_only_choice_differs = 0u64;
    let mut exactly_one_acquisition_write = 0u64;
    let mut exactly_one_acquisition_write_and_any_later_write = 0u64;
    let mut replacement_check_violations = 0u64;
    let mut highest_rule_collides = 0u64;
    let mut first_disk_rule_collides = 0u64;
    let mut generation_choice_rule_collides = 0u64;
    let mut first_disk_collision_example: Option<String> = None;
    let mut unequal_details: Vec<String> = Vec::new();
    for_each_crash_state(writes.len(), &segments, &mut |persisted| {
        states += 1;
        let image = CrashImage { base, writes: &writes, persisted: persisted.to_vec() };
        let pair = chosen_generation_and_instance(&image);
        let (generation_zero, instance_zero) = pair[0];
        let (generation_one, instance_one) = pair[1];
        if instance_zero != instance_one {
            instances_unequal += 1;
        }
        if generation_zero != generation_one {
            generations_unequal += 1;
        }
        if (generation_zero > generation_one && instance_zero < instance_one)
            || (generation_zero < generation_one && instance_zero > instance_one)
            || (generation_zero == generation_one && instance_zero != instance_one)
        {
            order_conflicts += 1;
        }
        let generation_choice = if generation_one > generation_zero { instance_one } else { instance_zero };
        let highest_superblock = instance_zero.max(instance_one);
        if generation_choice != highest_superblock {
            generation_only_choice_differs += 1;
        }
        let acquisition_persisted = usize::from(persisted[0]) + usize::from(persisted[1]);
        if acquisition_persisted == 1 {
            exactly_one_acquisition_write += 1;
            if persisted[2..].iter().any(|is_persisted| *is_persisted) {
                exactly_one_acquisition_write_and_any_later_write += 1;
            }
        }
        // mkfs 种下的根与单元带实例 0（它们在基线镜像里，不在写表里）。
        let mut evidence: BTreeSet<u32> = BTreeSet::from([0]);
        let mut highest_root = 0u32;
        for (write_index, is_persisted) in persisted.iter().enumerate() {
            if !*is_persisted || writes[write_index].kind == StepKind::SuperblockSlot {
                continue;
            }
            evidence.insert(carried[write_index]);
            if writes[write_index].kind == StepKind::RootRecordFua {
                highest_root = highest_root.max(carried[write_index]);
            }
        }
        if evidence.iter().any(|instance| *instance > highest_superblock) {
            replacement_check_violations += 1;
        }
        let mut present_anywhere = evidence.clone();
        present_anywhere.extend(every_valid_superblock_instance(&image));
        if present_anywhere.contains(&(highest_superblock.max(highest_root) + 1)) {
            highest_rule_collides += 1;
        }
        if present_anywhere.contains(&(instance_zero.max(highest_root) + 1)) {
            first_disk_rule_collides += 1;
            first_disk_collision_example.get_or_insert_with(|| persisted_labels(&labels, persisted));
        }
        if present_anywhere.contains(&(generation_choice.max(highest_root) + 1)) {
            generation_choice_rule_collides += 1;
        }
        if instance_zero != instance_one {
            let chosen = chosen_superblocks(&image);
            let checker_geometry = chosen[0].1.as_ref().expect("盘 0 有有效槽").1;
            let ring: BTreeSet<u32> = valid_roots(&image, &checker_geometry)
                .iter()
                .map(|(_, _, view)| view.instance)
                .collect();
            let core_superblock = choose_superblock(&image).expect("择超级块");
            let journal: BTreeSet<u32> = scan_journal(&image, &core_superblock)
                .values()
                .map(|record| record.instance.0)
                .collect();
            let outcome = recover(&image, JournalPolicy::Consult).outcome;
            let instance_verdict = check_pool_image(&image)
                .into_iter()
                .find(|(invariant, _)| *invariant == "I-7.7")
                .map(|(_, verdict)| verdict);
            unequal_details.push(format!(
                "persisted=[{}] chosen(generation,instance)={pair:?} root_ring_instances={ring:?} journal_instances={journal:?} recovery={outcome:?} checker_I-7.7={instance_verdict:?}",
                persisted_labels(&labels, persisted)
            ));
        }
    });
    println!("ORIGINAL states={states} instances_unequal={instances_unequal} generations_unequal={generations_unequal} generation_instance_order_conflicts={order_conflicts} generation_only_choice_differs={generation_only_choice_differs}");
    println!("ORIGINAL exactly_one_acquisition_write={exactly_one_acquisition_write} exactly_one_acquisition_write_and_any_later_write={exactly_one_acquisition_write_and_any_later_write} replacement_check_violations={replacement_check_violations}");
    println!("ORIGINAL next_acquisition_collides_with_instance_on_disk: max_over_all_superblocks_rule={highest_rule_collides} first_disk_chosen_rule={first_disk_rule_collides} generation_only_choice_rule={generation_choice_rule_collides} first_disk_example=[{}]", first_disk_collision_example.unwrap_or_default());
    for detail in &unequal_details {
        println!("UNEQUAL {detail}");
    }
}

/// 摘掉「暖机第一次空发布开头那道屏障」（甲把它当取号的持久点）再枚举一遍：取号两写与暖机第一条记录两写并成一段。
/// 数今天 checker 的 I-7.7、甲改过的 I-7.7、本腿的替代检查各在几个状态上判红，以及下一次取号撞号的状态。
#[allow(clippy::too_many_lines, reason = "一次枚举里各计数就地累加")]
fn report_without_warm_up_leading_barrier(
    base: &MemoryPool,
    operations: &[RetainedOperation],
    geometry: &FixedGeometry,
) {
    let first_barrier = operations
        .iter()
        .position(|retained| retained.operation.kind == RecordedOperationKind::Barrier)
        .expect("mkfs 之后第一道屏障就是暖机第一次空发布开头那道");
    let mut mutated = operations.to_vec();
    mutated.remove(first_barrier);
    let (writes, segments) = writes_and_segments(&mutated, geometry);
    let labels = layout_labels(&writes);
    let carried: Vec<u32> = writes.iter().map(instance_carried_by).collect();
    println!(
        "NO_WARM_UP_LEADING_BARRIER removed_step={first_barrier} segments={:?} writes={}",
        segments.iter().map(Vec::len).collect::<Vec<_>>(),
        writes.len()
    );
    let first_segment = segments[0].clone();
    let mut states = 0u64;
    let mut equality_violations = 0u64;
    let mut jia_violations = 0u64;
    let mut replacement_check_violations = 0u64;
    let mut reuse = 0u64;
    let mut reuse_while_jia_holds = 0u64;
    let mut checker_states_in_first_segment = 0u64;
    let mut checker_violations_in_first_segment = 0u64;
    let mut examples: Vec<String> = Vec::new();
    for_each_crash_state(writes.len(), &segments, &mut |persisted| {
        states += 1;
        let image = CrashImage { base, writes: &writes, persisted: persisted.to_vec() };
        let pair = chosen_generation_and_instance(&image);
        let highest_superblock = pair[0].1.max(pair[1].1);
        let equal = pair[0].1 == pair[1].1;
        let mut roots = BTreeSet::from([0u32]);
        let mut records = BTreeSet::new();
        let mut units = BTreeSet::from([0u32]);
        for (write_index, is_persisted) in persisted.iter().enumerate() {
            if !*is_persisted {
                continue;
            }
            match writes[write_index].kind {
                StepKind::RootRecordFua => {
                    roots.insert(carried[write_index]);
                }
                StepKind::JournalRecord => {
                    records.insert(carried[write_index]);
                }
                StepKind::UnitWrite => {
                    units.insert(carried[write_index]);
                }
                StepKind::SuperblockSlot | StepKind::Barrier => {}
            }
        }
        let highest_root = roots.iter().copied().max().unwrap_or(0);
        if !equal {
            equality_violations += 1;
        }
        let at_least_root = pair.iter().all(|(_, instance)| *instance >= highest_root);
        let jia_holds = (equal
            || (!roots.contains(&highest_superblock) && !records.contains(&highest_superblock)))
            && at_least_root;
        if !jia_holds {
            jia_violations += 1;
        }
        let replacement_holds = roots
            .iter()
            .chain(&records)
            .chain(&units)
            .all(|instance| *instance <= highest_superblock);
        if !replacement_holds {
            replacement_check_violations += 1;
        }
        let next = highest_superblock.max(highest_root) + 1;
        let reused = roots.contains(&next) || records.contains(&next) || units.contains(&next);
        if reused {
            reuse += 1;
            if jia_holds {
                reuse_while_jia_holds += 1;
            }
        }
        let only_first_segment = persisted
            .iter()
            .enumerate()
            .all(|(write_index, is_persisted)| !*is_persisted || first_segment.contains(&write_index));
        if only_first_segment {
            checker_states_in_first_segment += 1;
            let checker_says_violated = check_pool_image(&image).into_iter().any(|(invariant, verdict)| {
                invariant == "I-7.7" && matches!(verdict, InvariantVerdict::Violated(_))
            });
            if checker_says_violated {
                checker_violations_in_first_segment += 1;
            }
            if reused || !replacement_holds {
                examples.push(format!(
                    "persisted=[{}] superblock_instances=({}, {}) record_instances={records:?} checker_I-7.7_violated={checker_says_violated} jia_I-7.7_holds={jia_holds} next_acquisition={next} reused={reused}",
                    persisted_labels(&labels, persisted),
                    pair[0].1,
                    pair[1].1
                ));
            }
        }
    });
    println!("NO_WARM_UP_LEADING_BARRIER states={states} equality_violations={equality_violations} jia_I-7.7_violations={jia_violations} replacement_check_violations={replacement_check_violations} reuse={reuse} reuse_while_jia_I-7.7_holds={reuse_while_jia_holds} checker_states_in_first_segment={checker_states_in_first_segment} checker_I-7.7_violations_in_first_segment={checker_violations_in_first_segment}");
    for example in &examples {
        println!("NO_BARRIER_STATE {example}");
    }
}

/// 超级块 481 字节住 4096 字节槽的第一个 512 字节扇区里，其余七个扇区新旧都是 0：按扇区撕开只能留下整份旧的或整份新的。
fn report_superblock_padding(operations: &[RetainedOperation], geometry: &FixedGeometry) {
    let (writes, _) = writes_and_segments(operations, geometry);
    let superblock_writes: Vec<&RetainedWrite> =
        writes.iter().filter(|write| write.kind == StepKind::SuperblockSlot).collect();
    let padded = superblock_writes
        .iter()
        .filter(|write| write.bytes.len() == 4096 && write.bytes[481..].iter().all(|byte| *byte == 0))
        .count();
    println!("SUPERBLOCK_PADDING superblock_writes={} bytes_481_to_4096_all_zero={padded}", superblock_writes.len());
}

fn main() {
    let (base, operations) = record_first_mount_stream();
    let geometry = fixed_geometry();
    report_superblock_padding(&operations, &geometry);
    report_original_stream(&base, &operations, &geometry);
    report_without_warm_up_leading_barrier(&base, &operations, &geometry);
}
