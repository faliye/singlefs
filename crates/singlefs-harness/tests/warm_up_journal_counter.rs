//! 里程碑「覆盖写、释放、回退与复用」增补 2 第 21 行（C366（暖机路径把 txg 写进计数器与 tail））：暖机的空记录按记录号接着数 jsn，系统配置的 tail 存的是
//! jsn 计数器（D23（journal 的角色与格式） 已定项 18），checkpoint_txg 与 jsn 各走各的。
//!
//! 三条用例：
//! - 函数这一层给不相等的起点：环里最后一条记录的 jsn 是 40 时，mkfs 之后那一档的暖机写 jsn 41、42，txg 仍是 1、2。
//! - 可写挂载这一层没有可达盘面：C554 乙判不出按真（用户 2026-09-27 定，C579（判据在单故障合法状态上也拒可写））。原来那一格（所选根自己那条记录两份都读不出，
//!   2026-09-23 harness 那一组查出，随机历史 3600 步里重挂时两个量处处相等，只有这一格分得开）在可写挂载的读阶段被乙拒，
//!   这里改钉「可写挂载拒、只读挂载照常择 (1, 4)」。
//! - 可写挂载这一层分得开两个量的可达盘面：顺序写两个数据单元的发布（一次两条记录，jsn 比 txg 多走一步）之后可写挂载，
//!   写行与暖机的 jsn、轮换的 tail 跟着 jsn 走。

mod common;

use common::{
    build_pool, disk_snapshot, geometry, parameters, publish_overwrite_in_process,
    FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::journal::{record_offset, JournalRecord};
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{
    mount_writable, MountError, Mounted, NewerPublishWitness, RollbackTarget,
    SelectedVersionAgainstTheWitness, StillUnreadableAfterOneReread, WitnessedCounterComparison,
};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::{choose_system_configuration, PoolReader};
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_core::transaction::{
    acquire_instance, publish_sequential_write, warm_up_after_journal_counter, FirstFile,
    PoolWriter,
};
use singlefs_core::unit::{data_unit_payload_capacity, unit_filesystem_identifier};
use singlefs_format::{JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES};
use singlefs_harness::memory_pool::SparseBlockDevice;
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

/// 所选根 (1, 4) 自己那条记录 jsn 4 两份都读不出时（C366（暖机路径把 txg 写进计数器与 tail） 在可写挂载这一层分得开 txg 与 jsn 的那一格）：
/// 历史：A（txg 3、jsn 3）→ B（txg 4、jsn 4）→ 进程退出 → 两块盘上 jsn 4 那一格各坏一个字节。
/// 系统配置见证过 jsn 4（B 那次发布轮换写的 tail），所选那一版 (1, 4) 自己的末条记录读不出、计数器等于见证值的记录也是这一条——
/// C554 乙判不出，按真处置，重读一次仍判不出：可写挂载拒成 `NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion
/// { 见证 4、判不出 })`，拒在取号之前、任何写之前：录制流一步都没多、两块盘逐字节不变。只读挂载照常，择的根是 (1, 4)、环里读得出的
/// 只有 jsn 1–3，一条都不施加（C579（判据在单故障合法状态上也拒可写） 用户 2026-09-27 定：维持乙字面，判不出按真、
/// 拒可写、只读挂载照常）。
/// 原来钉的是可写挂载做成之后写行 txg 5 / jsn 4、暖机 txg 6、7 / jsn 5、6、tail 跟着 jsn 走，再干净重开一次落后 1 的差照样留着——
/// 可写挂载被拒之后那一段不可达，删掉；函数这一层 jsn 与 txg 各走各的由这个文件第一条用例钉着。
/// 判别力自证：乙把「判不出」按假处置时这一条红（`crates/mutations.tsv`）。
#[test]
fn c366_when_the_chosen_root_own_record_is_unreadable_the_writable_mount_is_refused_by_the_undecidable_witness_and_the_read_only_mount_opens_the_chosen_root(
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
    pool.devices = Some(devices);
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let reopened_devices = pool.devices.as_mut().expect("镜像还开着");
    let refused = mount_writable(&parameters(), reopened_devices)
        .expect_err("所选根自己那条记录读不出、见证判不出：可写挂载拒");
    let reading = SelectedVersionAgainstTheWitness {
        selected_version: RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(4),
        },
        witness: NewerPublishWitness {
            witnessed_journal_counter: 4,
            comparison: WitnessedCounterComparison::Undecidable,
        },
    };
    assert!(
        matches!(
            &refused,
            MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)
                if **still_unreadable
                    == StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
                        first_read: reading,
                        reread: reading,
                    }
        ),
        "C554 乙判不出按真、重读仍判不出：{refused:?}"
    );
    assert_eq!(
        pool.stream.operation_count(),
        operations_before_the_mount,
        "拒在取号之前：录制流一个写、一道屏障都没多"
    );
    let read_only = mount_read_only(&*reopened_devices).expect("只读挂载照常");
    assert_eq!(
        (
            read_only.chosen_root.instance,
            read_only.chosen_root.checkpoint_txg
        ),
        (InstanceGeneration(1), CheckpointTxg(4)),
        "B 的根持久了：只读挂载择的根是 (1, 4)"
    );
    assert_eq!(
        (
            read_only.journal.valid_records,
            read_only.journal.prefix_applied
        ),
        (3, 0),
        "环里读得出的只有 jsn 1–3（jsn 4 两份都坏了），一条都不施加"
    );
    assert_eq!(
        read_only.effective_root, read_only.chosen_root,
        "没有记录要施加"
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "两块盘逐字节不变"
    );
}

/// 恰好要 `data_units` 个数据单元的内容：最后一个单元装一半。
fn content_needing(data_units: usize, seed: usize) -> Vec<u8> {
    let payload_capacity = data_unit_payload_capacity();
    (0..(data_units - 1) * payload_capacity + payload_capacity / 2)
        .map(|index| u8::try_from((index * 13 + seed * 5 + 3) % 251).expect("小于 256"))
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

/// 可写挂载这一层分得开 txg 与 jsn 的可达盘面：一次发布的记录条数等于它的数据单元数（D23（journal 的角色与格式） 已定项 4），
/// 顺序写两个数据单元的发布让 jsn 比 txg 多走一步，之后一直差着。历史：A（txg 3、jsn 3）→ B 顺序写两个数据单元（txg 4、jsn 4、5）
/// → 进程退出 → 干净重开、可写挂载。所选根 (1, 4) 自己那次发布的末条 jsn 5 读得出、等于系统配置见证的 tail 5，C554 乙不拒。
/// 前缀末 jsn 5 ⇒ 写行那次 txg 5、jsn 6，暖机的 jsn 接着 7、8；三次发布轮换的系统配置 tail 是那次的 jsn，不是 txg
/// （已定项 18）；三条记录按 jsn 落格。
/// C579（判据在单故障合法状态上也拒可写） 定案之后第二条用例那一格被乙拒，挂载层三处 jsn 的取法（写行、暖机、轮换的 tail）由这一条钉：写行那次的 jsn 取 txg、
/// 暖机的 jsn 取上一版的 txg + 1，各改一处都红（`crates/mutations.tsv`）。
#[test]
fn a_writable_mount_after_a_two_record_publish_numbers_its_records_and_tail_from_the_jsn_one_ahead_of_the_txg(
) {
    let mut pool = build_pool("warm-up-counter-after-a-two-record-publish");
    let second_version = {
        let publish_parameters = parameters();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        let previous = pool.output.clone();
        publish_sequential_write(
            &mut writer,
            &mut pool.allocator,
            &previous,
            FirstFile {
                content: &content_needing(2, 1),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            InstanceGeneration(1),
        )
        .expect("顺序写 B")
    };
    assert_eq!(
        (
            second_version.root.checkpoint_txg,
            second_version
                .earlier_records_of_this_publish
                .iter()
                .map(|written| written.record.counter)
                .chain(std::iter::once(second_version.record.counter))
                .collect::<Vec<u64>>()
        ),
        (CheckpointTxg(4), vec![4, 5]),
        "B 一次发布两条记录：txg 4、jsn 4 与 5"
    );
    pool.output = second_version;
    let mut devices = pool.reopen_recorded();
    let operations_before_the_mount = pool.stream.operation_count();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    assert_eq!(
        (
            mounted.output.chosen_root.instance,
            mounted.output.chosen_root.checkpoint_txg,
            mounted.output.instance
        ),
        (
            InstanceGeneration(1),
            CheckpointTxg(4),
            InstanceGeneration(2)
        ),
        "所选根是 B 的 (1, 4)，取号 2"
    );
    assert_eq!(
        txg_and_counter_of_each_publish(&mounted),
        vec![
            (CheckpointTxg(5), 6),
            (CheckpointTxg(6), 7),
            (CheckpointTxg(7), 8)
        ],
        "jsn 从前缀末 5 接着数（6、7、8），txg 从所选根 4 + 1 = 5 起：两个量各走各的"
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
            record_written_to(6, 5),
            record_written_to(7, 6),
            record_written_to(8, 7)
        ]
        .concat(),
        "三条记录按 jsn 落格，不压 B 的 jsn 5 那一格"
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
        [tail_written(6), tail_written(7), tail_written(8)].concat(),
        "每次发布轮换的系统配置 tail 是那次的 jsn（6、7、8），不是 txg（5、6、7）"
    );
    pool.devices = Some(devices);
}
