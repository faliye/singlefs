//! harness 测试文件 `crates/singlefs-harness/tests/row_publish_record_without_its_root.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{
    crash_state_devices, format_pool, geometry, memory_pool_of_sparse_devices, parameters,
    IMAGE_BYTES,
};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::journal::{record_offset, JournalRecord};
use singlefs_core::mount::{mount_writable, InstanceRow};
use singlefs_core::pointer::NodePointer;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, recover, replay_journal, scan_journal, JournalPolicy,
    NamedUnitVerification, PoolReader, RecoveryOutcome,
};
use singlefs_core::transaction::PoolVersion;
use singlefs_core::unit::{unit_filesystem_identifier, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED};
use singlefs_format::{
    DATA_UNIT_BYTES, JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES, NODE_BYTES,
};
use singlefs_harness::memory_pool::{writes_and_segments, MemoryPool};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

pub const BOTH_DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

pub fn unit_bytes_of_class(unit_class: u8) -> usize {
    let bytes = match unit_class {
        UNIT_CLASS_PACKED => DATA_UNIT_BYTES,
        UNIT_CLASS_INDEX_NODE => NODE_BYTES,
        other => panic!("写行那次发布只点名实例表（码 3）与分配记录节点（码 2）：{other}"),
    };
    usize::try_from(bytes).expect("单元宽装得进 usize")
}

/// 交 checker 档：这张镜像上池级 checker 一条都不红。
pub fn hand_out_expecting_no_invariant_violated(
    images: &mut PoolCheckerImages,
    step: &str,
    build_the_image: impl FnOnce() -> MemoryPool,
) {
    images.hand_out(
        step,
        build_the_image,
        PoolCheckerExpectation::no_invariant_violated(),
    );
}

pub fn c533_row_publish_record_persisted_without_its_root_on_a_formatted_pool_is_never_applied_and_its_two_units_stay_isolated_from_the_next_mount(
    images: &mut PoolCheckerImages,
) {
    let mut formatted = format_pool("c533-row-publish-record-without-root");
    let mut first = formatted.reopen_recorded();
    let first_mount = mount_writable(&parameters(), &mut first).expect("第一次可写挂载：不写行");
    formatted.devices = Some(first);
    assert_eq!(first_mount.current.root().checkpoint_txg, CheckpointTxg(2));
    let operations_before_the_second_mount = formatted.stream.operation_count();
    let mut second = formatted.reopen_recorded();
    let row_written = mount_writable(&parameters(), &mut second).expect("第二次可写挂载：写行");
    formatted.devices = Some(second);
    let PoolVersion::WithoutFile(row_publish) = &row_written.output.row_publish else {
        panic!("树表 0 条的一版上写行，写出来的仍是没有文件版本的一版")
    };
    assert_eq!(
        (
            row_publish.root.instance,
            row_publish.root.checkpoint_txg,
            row_publish.record.counter
        ),
        (InstanceGeneration(2), CheckpointTxg(3), 3)
    );
    // 分配记录树按位置寻址（D8（核心索引结构） 已定项 14）：4 GiB 两块盘上根在第 2 层，写行那次写出五个节点
    // （两块盘各自的叶 61、各自的第 1 层节点 0 与根），加实例表共六个单元。
    assert_eq!(
        row_publish.record.named.len(),
        6,
        "写行那次发布点名实例表与分配记录树五个节点，六个单元"
    );
    let unit_slots_of_the_row_publish = |named: &[singlefs_core::journal::NamedUnit]| {
        named
            .iter()
            .map(|named| {
                assert_eq!(named.locations[1].slot, named.locations[0].slot, "两盘同槽");
                named.locations[0].slot
            })
            .collect::<Vec<SlotNumber>>()
    };
    let slots_of_the_crashed_row_publish = unit_slots_of_the_row_publish(&row_publish.record.named);
    assert!(slots_of_the_crashed_row_publish
        .contains(&row_publish.root.instance_table.locations[0].slot));
    assert!(slots_of_the_crashed_row_publish
        .contains(&row_publish.root.allocation_record_tree_root.locations[0].slot));

    // 崩溃状态：第二次挂载写出去的写里，根槽写之前的全部持久（取号两轮四写、六个单元各两写、记录两写），根槽与之后的一个都没有。
    let operations = formatted.retained_operations();
    let mut base = MemoryPool::with_devices(&BOTH_DEVICES, IMAGE_BYTES);
    base.apply(&operations[..operations_before_the_second_mount]);
    let (writes, _segments) = writes_and_segments(
        &operations[operations_before_the_second_mount..],
        &geometry(),
    );
    let first_root_write = writes
        .iter()
        .position(|write| write.kind == StepKind::RootRecordFua)
        .expect("写行那次发布有根槽写");
    let persisted_kinds: Vec<StepKind> = writes[..first_root_write]
        .iter()
        .map(|write| write.kind)
        .collect();
    assert_eq!(
        persisted_kinds,
        [StepKind::SystemConfigurationSlot; 4]
            .into_iter()
            .chain([StepKind::UnitWrite; 12])
            .chain([StepKind::JournalRecord; 2])
            .collect::<Vec<_>>(),
        "根槽写之前是取号两轮四写、六个单元各两写、记录两写"
    );
    let persisted: Vec<bool> = (0..writes.len())
        .map(|write_index| write_index < first_root_write)
        .collect();
    let crash_stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(&base, &writes, &persisted, &crash_stream);
    let crash_image = memory_pool_of_sparse_devices(&devices);

    // 那两个单元与那条记录都在盘上。
    for named in &row_publish.record.named {
        for location in &named.locations {
            let bytes = crash_image
                .read(
                    location.device,
                    location.slot.to_device_offset(),
                    unit_bytes_of_class(named.unit_class),
                )
                .expect("单元区读得到");
            assert_eq!(
                crc32_castagnoli(&bytes),
                location.unit_checksum,
                "写行那次发布的单元（码 {}）在盘 {} 槽 {} 上完整",
                named.unit_class,
                location.device.0,
                location.slot.0
            );
        }
    }
    let filesystem_identifier = unit_filesystem_identifier(&parameters().filesystem_identifier);
    for device in BOTH_DEVICES {
        let bytes = crash_image
            .read(
                device,
                record_offset(3, JOURNAL_RING_DEFAULT_BYTES),
                usize::try_from(JOURNAL_RECORD_BYTES).expect("4096"),
            )
            .expect("环里读得到");
        let record = JournalRecord::parse(&bytes, filesystem_identifier)
            .expect("写行那条记录两份都自证得过");
        assert_eq!(
            (record.instance, record.checkpoint_txg, record.counter),
            (InstanceGeneration(2), CheckpointTxg(3), 3)
        );
    }

    // 恢复：所选根 (1, 2)，那条记录是实例 2 的，不在前缀候选里；重建出来的根就是 (1, 2)，拿不到那两个单元。
    let report = recover(&crash_image, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::NoFile {
            root: (InstanceGeneration(1), CheckpointTxg(2))
        }
    );
    assert_eq!(
        report.effective_root,
        Some((InstanceGeneration(1), CheckpointTxg(2)))
    );
    assert_eq!(
        (
            report.journal.valid_records,
            report.journal.above_water,
            report.journal.prefix_applied
        ),
        (3, 0, 0),
        "jsn 1–3 都读得出；jsn 3 是实例 2 的，前缀不跨实例边界，水位之上一条都没有"
    );
    let system_configuration =
        choose_system_configuration(&crash_image).expect("崩溃镜像上系统配置择得出");
    let chosen_root = choose_root(&crash_image, &system_configuration).expect("择得出根");
    let records = scan_journal(&crash_image, &system_configuration);
    let (_, rebuilt_root) = replay_journal(
        &crash_image,
        &chosen_root,
        system_configuration.immutable.sizes.journal_ring_bytes,
        &records,
        NamedUnitVerification::Verify,
    )
    .expect("所选根那次发布只有一条记录带末条标志：锚点认得出");
    assert_eq!(rebuilt_root, chosen_root, "重建出来的根就是所选根本身");
    assert_eq!(
        rebuilt_root.instance_table, formatted.genesis.root.instance_table,
        "重建出来的根指着 mkfs 那一片实例表，不是写行那次写的那一片"
    );
    assert_eq!(
        rebuilt_root.allocation_record_tree_root,
        NodePointer::empty_root(),
        "重建出来的根没有分配记录树，写行那次写的那片节点谁也不引用"
    );

    // checker：遍历从根走，那两个单元谁也不引用，一条违例都没有；把它们在两块盘上都改坏，遍历一侧的判决逐字不变——走读不读它们。
    // 读它们的只有 I-8.11（点名项与被指单元头相同）：jsn 3 那条记录点名了它们，改坏之前整单元校验和对得上、头与点名项相同，判成立；
    // 改坏之后每条位置条目的校验和都对不上，那一行「校验和不相符的位置条目不判」，环里再没有可比的，转成不适用。
    // 改坏之前 jsn 3 点名的单元校验和对得上、头与点名项相同：I-8.11 判成立
    images.hand_out(
        "崩溃镜像",
        || crash_image.clone(),
        PoolCheckerExpectation::no_invariant_violated().and_holds("I-8.11"),
    );
    // 两个孤单元两块盘上都改坏之后，I-8.11 没有可比的单元、报不适用（理由串「整单元校验和」对拍字里没有，checker 档只核没判到），
    // I-8.11 之外 checker 的判决逐条不变
    images.hand_out(
        "崩溃镜像上两个孤单元两块盘上都改坏",
        || {
            let mut orphan_units_damaged = crash_image.clone();
            for named in &row_publish.record.named {
                for location in &named.locations {
                    orphan_units_damaged.flip_byte(
                        location.device,
                        location.slot.to_device_offset(),
                        100,
                    );
                }
            }
            orphan_units_damaged
        },
        PoolCheckerExpectation::no_invariant_violated()
            .and_not_judged("I-8.11")
            .and_judged_like_the_image("崩溃镜像", &["I-8.11"]),
    );

    // 重开可写挂载：账从 mkfs 那一版重建，那两个落点不在账里；记录 jsn 3 在水位之上没施加，C 把它点名的六个落点隔离，写行那次发布取别的槽。
    let third_mount = mount_writable(&parameters(), &mut devices).expect("崩溃之后重开可写挂载");
    assert_eq!(
        third_mount.output.instance,
        InstanceGeneration(3),
        "号 2 取号时已写进系统配置，烧掉了"
    );
    assert_eq!(
        third_mount.output.rows_written,
        vec![
            InstanceRow {
                instance: InstanceGeneration(1),
                selected_root_txg: CheckpointTxg(2),
                applied_transaction_high_water: 0,
            },
            InstanceRow {
                instance: InstanceGeneration(2),
                selected_root_txg: CheckpointTxg(0),
                applied_transaction_high_water: 0,
            },
        ],
        "实例 1 那一行 (1, 2, 0)；烧掉的实例 2 按中间实例写 (2, 0, 0)"
    );
    let PoolVersion::WithoutFile(rewritten_row_publish) = &third_mount.output.row_publish else {
        panic!("仍是树表 0 条的一版")
    };
    assert_eq!(
        (
            rewritten_row_publish.root.checkpoint_txg,
            rewritten_row_publish.record.counter
        ),
        (CheckpointTxg(4), 4),
        "txg 与 jsn 都从环里那条孤记录（txg 3、jsn 3）之后接"
    );
    let slots_of_the_rewritten_row_publish =
        unit_slots_of_the_row_publish(&rewritten_row_publish.record.named);
    assert!(
        slots_of_the_rewritten_row_publish
            .iter()
            .all(|slot| !slots_of_the_crashed_row_publish.contains(slot)),
        "这次写行取的落点 {slots_of_the_rewritten_row_publish:?} 一个都不是崩掉那次写的 {slots_of_the_crashed_row_publish:?}（C 隔离了它们）"
    );
    assert!(
        third_mount
            .allocator
            .records()
            .iter()
            .all(|record| !slots_of_the_crashed_row_publish.contains(&record.slot)),
        "挂载之后没有一条分配记录罩着崩掉那次写的六个槽"
    );
    for device in BOTH_DEVICES {
        let device_map = third_mount
            .allocator
            .devices
            .iter()
            .find(|device_map| device_map.device == device)
            .expect("池里有这块盘");
        assert!(
            slots_of_the_crashed_row_publish
                .iter()
                .all(|slot| !device_map.is_free(*slot)),
            "盘 {device:?} 上崩掉那次写的六个槽都隔离着、不空闲"
        );
    }
    hand_out_expecting_no_invariant_violated(images, "崩溃之后重开可写挂载", || {
        memory_pool_of_sparse_devices(&devices)
    });
}
