//! 里程碑「第二个事务」增补 2 第 21 行（C366（暖机路径把 txg 写进计数器与 tail））：暖机的空记录按记录号接着数 jsn，系统配置的 tail 存的是
//! jsn 计数器（D23（journal 的角色与格式） 已定项 18），checkpoint_txg 与 jsn 各走各的。
//!
//! 两条用例：
//! - 函数这一层给不相等的起点：环里最后一条记录的 jsn 是 40 时，mkfs 之后那一档的暖机写 jsn 41、42，txg 仍是 1、2。
//! - 可写挂载这一层唯一分得开两个量的可达盘面：所选根自己那条记录两份都读不出（2026-09-23 harness 那一组查出，
//!   随机历史 3600 步里重挂时两个量处处相等，只有这一格分得开）。D23（journal 的角色与格式） 已定项 14 注 3 定前缀末的计数器
//!   取环里读得出的最大那一个（用户 2026-09-23 定），于是写行那次发布 txg 5、jsn 4，之后每次发布的 jsn 与 tail 都比 txg 小 1。

mod common;

use common::{
    build_pool, geometry, parameters, publish_overwrite_in_process, FIXED_WRITE_TIME_SECONDS,
    IMAGE_BYTES,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::journal::{record_offset, JournalRecord};
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{mount_writable, Mounted};
use singlefs_core::recovery::{choose_system_configuration, PoolReader};
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_core::transaction::{acquire_instance, warm_up_after_journal_counter, PoolWriter};
use singlefs_core::unit::unit_filesystem_identifier;
use singlefs_format::{JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES};
use singlefs_harness::crash::SparseBlockDevice;
use singlefs_harness::segments::StepKind;
use singlefs_harness::RetainedOperation;

/// 环里最后一条记录的 jsn：与暖机的 txg 1、2 都不相等，计数器写成 txg 就分得出来。
const LAST_JOURNAL_COUNTER_BEFORE_WARM_UP: u64 = 40;

#[test]
fn warm_up_after_a_journal_counter_other_than_zero_counts_records_on_from_it_while_txg_stays_one_and_two(
) {
    let parameters = parameters();
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = (0..2u32)
        .map(|device_number| {
            (
                DeviceIdentity(device_number),
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let warmed = {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut pool).expect("取号");
        assert_eq!(instance, InstanceGeneration(1));
        warm_up_after_journal_counter(
            &mut pool,
            &genesis.root,
            instance,
            LAST_JOURNAL_COUNTER_BEFORE_WARM_UP,
        )
        .expect("暖机")
    };

    let txg_and_counter: Vec<(CheckpointTxg, u64)> = warmed
        .records
        .iter()
        .map(|record| (record.checkpoint_txg, record.counter))
        .collect();
    assert_eq!(
        txg_and_counter,
        vec![(CheckpointTxg(1), 41), (CheckpointTxg(2), 42)],
        "jsn 接着前缀末 40 数（41、42），txg 照格式常量走 1、2"
    );
    let root_txgs: Vec<CheckpointTxg> = warmed
        .roots
        .iter()
        .map(|root| root.checkpoint_txg)
        .collect();
    assert_eq!(root_txgs, vec![CheckpointTxg(1), CheckpointTxg(2)]);

    let system_configuration =
        choose_system_configuration(&devices).expect("暖机之后系统配置自证得过");
    assert_eq!(
        system_configuration.quantities.journal_tail, 42,
        "系统配置的 tail 存 jsn 计数器（最后一条空记录的 42），不是 txg 2"
    );

    let filesystem_identifier = unit_filesystem_identifier(&parameters.filesystem_identifier);
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        for (expected_txg, counter) in [(CheckpointTxg(1), 41), (CheckpointTxg(2), 42)] {
            let bytes = PoolReader::read(
                &devices,
                device,
                record_offset(counter, JOURNAL_RING_DEFAULT_BYTES),
                record_bytes,
            )
            .expect("环里那一格读得到");
            let record = JournalRecord::parse(&bytes, filesystem_identifier)
                .expect("jsn 对应的那一格里是一条自证过的记录");
            assert_eq!(
                (record.checkpoint_txg, record.counter),
                (expected_txg, counter),
                "盘 {} 上 jsn {counter} 那一格",
                device.0
            );
        }
        for counter_equal_to_txg in [1, 2] {
            let bytes = PoolReader::read(
                &devices,
                device,
                record_offset(counter_equal_to_txg, JOURNAL_RING_DEFAULT_BYTES),
                record_bytes,
            )
            .expect("环里那一格读得到");
            assert!(
                bytes.iter().all(|byte| *byte == 0),
                "盘 {} 上 jsn {counter_equal_to_txg} 那一格没写过：记录不按 txg 落格",
                device.0
            );
        }
    }
}

/// 发布 B 的内容：与第一个文件不同，读回时分得出是哪一版。
fn second_version_content() -> Vec<u8> {
    (0..4100usize)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect()
}

/// 一次可写挂载写出的发布（写行那次在前、暖机在后），各自的 (txg, jsn)。
fn txg_and_counter_of_each_publish(mounted: &Mounted) -> Vec<(CheckpointTxg, u64)> {
    std::iter::once(&mounted.output.row_publish)
        .chain(mounted.output.warm_up_publishes.iter())
        .map(|publish| (publish.root().checkpoint_txg, publish.record().counter))
        .collect()
}

/// 一段录制流里写出去的系统配置槽（盘、实例代号、tail）与 journal 记录（盘、落点、txg、jsn），按写出的次序。
struct WrittenJournalPositions {
    system_configuration_tails: Vec<(DeviceIdentity, InstanceGeneration, u64)>,
    journal_records: Vec<(DeviceIdentity, DeviceOffsetInBytes, CheckpointTxg, u64)>,
}

fn written_journal_positions(operations: &[RetainedOperation]) -> WrittenJournalPositions {
    let filesystem_identifier = unit_filesystem_identifier(&parameters().filesystem_identifier);
    let mut positions = WrittenJournalPositions {
        system_configuration_tails: Vec::new(),
        journal_records: Vec::new(),
    };
    for retained in operations {
        match geometry().classify(&retained.operation) {
            StepKind::SystemConfigurationSlot => {
                let slot = SystemConfiguration::parse_slot(
                    retained.contents.as_deref().expect("录制流开了内容保留"),
                )
                .expect("写出去的系统配置槽自证得过");
                positions.system_configuration_tails.push((
                    retained.operation.device,
                    slot.quantities.journal_instance,
                    slot.quantities.journal_tail,
                ));
            }
            StepKind::JournalRecord => {
                let record = JournalRecord::parse(
                    retained.contents.as_deref().expect("录制流开了内容保留"),
                    filesystem_identifier,
                )
                .expect("写出去的记录自证得过");
                positions.journal_records.push((
                    retained.operation.device,
                    retained.operation.offset,
                    record.checkpoint_txg,
                    record.counter,
                ));
            }
            StepKind::ZeroFill
            | StepKind::UnitWrite
            | StepKind::RootRecordFua
            | StepKind::Barrier => {}
        }
    }
    positions
}

/// 所选根 (1, 4) 自己那条记录 jsn 4 两份都读不出时，可写挂载写的 jsn 与 tail 跟着记录号走（D23（journal 的角色与格式） 已定项 14 注 3：
/// 前缀末的计数器取环里读得出的最大那一个，新实例的第一条编在它 + 1、落回那条读不出的记录原来的槽；已定项 18：tail 存 jsn 计数器）。
/// 历史：A（txg 3、jsn 3）→ B（txg 4、jsn 4）→ 进程退出 → 两块盘上 jsn 4 那一格各坏一个字节 → 可写挂载。
/// 读得出的最大号是 jsn 3 ⇒ 写行那次发布 jsn 4、写进 jsn 4 那一格；txg = max(根环 4, 记录 3) + 1 = 5；暖机 txg 6、7 的 jsn 5、6；
/// 三次发布各自轮换的系统配置 tail 是那次的 jsn（4、5、6），不是 txg。
/// 再干净重开一次，落后 1 的差照样留着：前缀末 jsn 6 ⇒ 写行 txg 8、jsn 7，暖机 txg 9、10 的 jsn 8、9，tail 9。
/// 判别力自证：写行那次的 jsn 取 txg、暖机的 jsn 取上一版的 txg + 1、tail 取 txg，三处各改一处都红（`crates/mutations.tsv`）。
#[test]
fn c366_when_the_chosen_root_own_record_is_unreadable_the_new_instance_numbers_records_and_tail_from_the_highest_readable_record(
) {
    let mut pool = build_pool("c366-chosen-root-record-unreadable");
    let first_version = pool.output.clone();
    let second_version = publish_overwrite_in_process(
        &mut pool,
        &first_version,
        &second_version_content(),
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("发布 B");
    assert_eq!(
        (
            second_version.root.checkpoint_txg,
            second_version.record.counter
        ),
        (CheckpointTxg(4), 4),
        "到 B 为止 txg 与 jsn 还相等"
    );
    pool.output = second_version;
    let mut devices = pool.reopen_recorded();
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let chosen_root_own_record_offset = record_offset(4, JOURNAL_RING_DEFAULT_BYTES);
    for (_, device) in &mut devices {
        let mut bytes = vec![0u8; record_bytes];
        device
            .read_at(chosen_root_own_record_offset, &mut bytes)
            .expect("读 jsn 4 那一格");
        bytes[300] ^= 0xff;
        device
            .write_at(
                chosen_root_own_record_offset,
                &bytes,
                WriteDurability::Plain,
            )
            .expect("改坏 jsn 4 那一格");
    }
    let operations_before_the_mount = pool.stream.operation_count();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    assert_eq!(
        (
            mounted.output.chosen_root.instance,
            mounted.output.chosen_root.checkpoint_txg
        ),
        (InstanceGeneration(1), CheckpointTxg(4)),
        "B 的根持久了：所选根是 (1, 4)"
    );
    assert_eq!(
        (
            mounted.output.journal.valid_records,
            mounted.output.journal.prefix_applied
        ),
        (3, 0),
        "环里读得出的只有 jsn 1–3（jsn 4 两份都坏了），一条都不施加"
    );
    assert_eq!(
        mounted.output.effective_root, mounted.output.chosen_root,
        "没有记录要施加"
    );
    assert_eq!(
        txg_and_counter_of_each_publish(&mounted),
        vec![
            (CheckpointTxg(5), 4),
            (CheckpointTxg(6), 5),
            (CheckpointTxg(7), 6)
        ],
        "jsn 从读得出的最大号 3 接着数（4、5、6），txg 从 max(根环 4, 记录 3) + 1 = 5 起：两个量各走各的"
    );
    let written = written_journal_positions(
        &pool.stream.retained_operations()[operations_before_the_mount..],
    );
    let record_written_to = |counter: u64, txg: u64| {
        [DeviceIdentity(0), DeviceIdentity(1)].map(|device| {
            (
                device,
                record_offset(counter, JOURNAL_RING_DEFAULT_BYTES),
                CheckpointTxg(txg),
                counter,
            )
        })
    };
    assert_eq!(
        written.journal_records,
        [
            record_written_to(4, 5),
            record_written_to(5, 6),
            record_written_to(6, 7)
        ]
        .concat(),
        "三条记录按 jsn 落格：写行那条落回那条读不出的 jsn 4 原来的槽"
    );
    let tail_written = |tail: u64| {
        [DeviceIdentity(0), DeviceIdentity(1)].map(|device| (device, InstanceGeneration(2), tail))
    };
    assert_eq!(
        written.system_configuration_tails.len(),
        8,
        "取号两份 + 三次发布各两份"
    );
    assert_eq!(
        written.system_configuration_tails[2..],
        [tail_written(4), tail_written(5), tail_written(6)].concat(),
        "每次发布轮换的系统配置 tail 是那次的 jsn（4、5、6），不是 txg（5、6、7）"
    );

    pool.devices = Some(devices);
    let mut reopened = pool.reopen_recorded();
    let remounted = mount_writable(&parameters(), &mut reopened).expect("干净重开再可写挂载");
    assert_eq!(
        (
            remounted.output.chosen_root.instance,
            remounted.output.chosen_root.checkpoint_txg
        ),
        (InstanceGeneration(2), CheckpointTxg(7))
    );
    assert_eq!(
        txg_and_counter_of_each_publish(&remounted),
        vec![
            (CheckpointTxg(8), 7),
            (CheckpointTxg(9), 8),
            (CheckpointTxg(10), 9)
        ],
        "落后 1 的差一直留着：前缀末 jsn 6 ⇒ jsn 7、8、9；txg 8 ≡ 2 (mod 3) 落盘 0，暖机推两次"
    );
    assert_eq!(
        choose_system_configuration(&reopened)
            .expect("挂载之后系统配置自证得过")
            .quantities
            .journal_tail,
        9,
        "盘上的 tail 是最后一条记录的 jsn 9，不是 txg 10"
    );
    pool.devices = Some(reopened);
}
