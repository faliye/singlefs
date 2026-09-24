//! 里程碑「第二个事务」增补 2 收口表第 8 行（C495（第一个文件版本核上一版记录没有条款））：D23（journal 的角色与格式） 已定项 14
//! 「第一个文件版本接在哪条记录之后」（用户 2026-09-24 照今天的实现写成条款）——写第一个文件版本之前，要接的那一版之后的上一条记录
//! 必须解得出、且它的 `checkpoint_txg` 等于那一版的 txg；不满足就在任何写之前拒绝（`publish_first_file` 返回
//! `FirstFileVersionDoesNotFollowTheVersionItBuildsOn`），一个字节不写。
//!
//! 场景：只做过 mkfs 的池可写挂载（取号 1、零单元发布 txg 1 jsn 1、暖机 txg 2 jsn 2），要在 txg 2 那一版上写第一个文件版本，
//! 上一条记录是环里 jsn 2 那一格。故障注入落在那一格上（两块盘两份都动）：
//! - 两份都坏（翻一个字节）、两份都缺（整格写 0）——解不出，拒绝，交回的 `previous_record` 是 `None`；
//! - 两份都换成 jsn 1 那条（解得出，但 txg 1 不等于要接的 txg 2）——拒绝，交回 `Some((txg 1, jsn 1))`。
//!
//! 每一格都比拒绝前后的系统配置槽、根环、录制流步数（`DiskSnapshot`）与分配器的记录，逐项不变；
//! 最后把那一格还原成原来的 jsn 2，同一次调用照常写出第一个文件版本（txg 3、jsn 3）——拒绝确实是那一格引起的。

mod common;

use common::{file_content, format_pool, parameters, DiskSnapshot, FormattedPool, Recorded};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::journal::{record_offset, JournalRecord};
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{choose_system_configuration, readable_roots, PoolReader};
use singlefs_core::root_record::RootRecord;
use singlefs_core::transaction::{
    publish_first_file, FirstFile, PoolWriter, PublishError, TransactionOutput,
};
use singlefs_core::unit::unit_filesystem_identifier;
use singlefs_format::{
    JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES,
};

/// 要接的那一版（txg 2 的暖机根）之后的上一条记录：jsn 2。
const PREVIOUS_RECORD_COUNTER: u64 = 2;

/// 要接的那一版之后的上一条记录那一格上注入的故障，两块盘两份一起动。
#[derive(Clone, Copy, Debug)]
enum FaultOnBothCopiesOfThePreviousRecord {
    /// 两份各翻一个字节：读得出字节、自证不过。
    Corrupted,
    /// 两份都整格写 0：那一格上没有记录。
    Missing,
    /// 两份都换成 jsn 1 那条记录的字节：解得出，但它的 txg 是 1、不是要接的那一版的 2。
    HoldTheRecordBeforeIt,
}

/// 可写挂载之后的只做过 mkfs 的池：要接的那一版的根、挂载重建的分配器、jsn 2 那一格原来的字节。
struct VersionToBuildOn {
    formatted: FormattedPool,
    devices: Vec<(DeviceIdentity, Recorded)>,
    root: RootRecord,
    allocator: PoolAllocator,
    original_previous_record: Vec<u8>,
}

fn record_bytes() -> usize {
    usize::try_from(JOURNAL_RECORD_BYTES).expect("4096")
}

fn read_ring_slot(device: &Recorded, counter: u64) -> Vec<u8> {
    let mut bytes = vec![0u8; record_bytes()];
    device
        .read_at(
            record_offset(counter, JOURNAL_RING_DEFAULT_BYTES),
            &mut bytes,
        )
        .expect("环里那一格读得到");
    bytes
}

fn mount_a_formatted_pool(tag: &str) -> VersionToBuildOn {
    let mut formatted = format_pool(tag);
    let mut devices = formatted.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("只做过 mkfs 的池可写挂载");
    let root = *mounted.current.root();
    assert_eq!(
        (
            root.instance,
            root.checkpoint_txg,
            mounted.current.record().counter
        ),
        (
            InstanceGeneration(1),
            CheckpointTxg(2),
            PREVIOUS_RECORD_COUNTER
        ),
        "要接的那一版是暖机 txg 2，它之后的上一条记录是 jsn 2"
    );
    let original_previous_record = read_ring_slot(&devices[0].1, PREVIOUS_RECORD_COUNTER);
    assert_eq!(
        original_previous_record,
        mounted.current.record_bytes(),
        "盘上 jsn 2 那一格就是挂载写出的那条"
    );
    VersionToBuildOn {
        formatted,
        devices,
        root,
        allocator: mounted.allocator,
        original_previous_record,
    }
}

/// 此刻盘上可比的一份快照（与 `common::disk_snapshot` 同三项），直接从开着的设备读，不从录制流重放整份镜像：
/// 两盘各两个系统配置槽的原样字节、根环里全部自证过的根、录制流里已有几步。
fn disk_snapshot_of_open_devices(version: &VersionToBuildOn) -> DiskSnapshot {
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let mut system_configuration_slots = Vec::new();
    for (identity, _) in &version.devices {
        for offset in [0, spacing] {
            system_configuration_slots.push(
                PoolReader::read(
                    &version.devices,
                    *identity,
                    DeviceOffsetInBytes(offset),
                    slot_bytes,
                )
                .expect("系统配置槽读得到"),
            );
        }
    }
    let system_configuration =
        choose_system_configuration(&version.devices).expect("系统配置择得出");
    DiskSnapshot {
        system_configuration_slots,
        readable_roots: readable_roots(
            &version.devices,
            &system_configuration.immutable.region_devices,
            &system_configuration.immutable.sizes,
            &system_configuration.immutable.filesystem_identifier,
        ),
        recorded_operations: version.formatted.stream.operation_count(),
    }
}

/// 两块盘上 jsn 2 那一格一起换成 `bytes`（写经录制器，录制流跟着多几步：拒绝前的快照在这之后拍）。
fn overwrite_the_previous_record_slot(version: &mut VersionToBuildOn, bytes: &[u8]) {
    for (_, device) in &mut version.devices {
        device
            .write_at(
                record_offset(PREVIOUS_RECORD_COUNTER, JOURNAL_RING_DEFAULT_BYTES),
                bytes,
                WriteDurability::Plain,
            )
            .expect("改写 jsn 2 那一格");
    }
}

fn inject(version: &mut VersionToBuildOn, fault: FaultOnBothCopiesOfThePreviousRecord) {
    let replacement = match fault {
        FaultOnBothCopiesOfThePreviousRecord::Corrupted => {
            let mut corrupted = version.original_previous_record.clone();
            corrupted[300] ^= 0xff;
            corrupted
        }
        FaultOnBothCopiesOfThePreviousRecord::Missing => vec![0u8; record_bytes()],
        FaultOnBothCopiesOfThePreviousRecord::HoldTheRecordBeforeIt => {
            read_ring_slot(&version.devices[0].1, PREVIOUS_RECORD_COUNTER - 1)
        }
    };
    overwrite_the_previous_record_slot(version, &replacement);
}

/// 调用方从环里读要接的那一版之后的上一条记录（两块盘各读一份；两份都动过，读哪份都一样），交给 `publish_first_file`。
fn publish_the_first_file_version_after_the_record_on_the_ring(
    version: &mut VersionToBuildOn,
) -> Result<TransactionOutput, PublishError> {
    let previous_record = read_ring_slot(&version.devices[0].1, PREVIOUS_RECORD_COUNTER);
    assert_eq!(
        previous_record,
        read_ring_slot(&version.devices[1].1, PREVIOUS_RECORD_COUNTER),
        "两份一起动过"
    );
    let publish_parameters = parameters();
    let mut writer = PoolWriter::new(&publish_parameters, version.devices.as_mut_slice());
    let content = file_content();
    publish_first_file(
        &mut writer,
        &mut version.allocator,
        &version.root,
        FirstFile {
            content: &content,
            write_time_seconds: common::FIXED_WRITE_TIME_SECONDS,
        },
        InstanceGeneration(1),
        &previous_record,
    )
}

/// 注入 `fault` 之后调用，断言在任何写之前拒绝、交回的 `previous_record` 是 `expected_previous_record`，盘上与分配器逐项不变。
fn assert_refused_before_any_write(
    version: &mut VersionToBuildOn,
    fault: FaultOnBothCopiesOfThePreviousRecord,
    expected_previous_record: Option<(CheckpointTxg, u64)>,
) {
    inject(version, fault);
    let before = disk_snapshot_of_open_devices(version);
    let allocation_records_before = version.allocator.records().to_vec();
    let refused = publish_the_first_file_version_after_the_record_on_the_ring(version);
    match refused {
        Err(PublishError::FirstFileVersionDoesNotFollowTheVersionItBuildsOn {
            version_to_build_on,
            previous_record,
        }) => assert_eq!(
            (version_to_build_on, previous_record),
            (CheckpointTxg(2), expected_previous_record),
            "{fault:?}：拒绝的成员点名要接的那一版与读到的上一条记录"
        ),
        other => panic!(
            "{fault:?}：要在任何写之前返回 FirstFileVersionDoesNotFollowTheVersionItBuildsOn，实际 {:?}",
            other.map(|output| (output.root.checkpoint_txg, output.record.counter))
        ),
    }
    let after = disk_snapshot_of_open_devices(version);
    assert_eq!(
        after.system_configuration_slots, before.system_configuration_slots,
        "{fault:?}：两盘系统配置槽逐字节不变"
    );
    assert_eq!(
        after.readable_roots, before.readable_roots,
        "{fault:?}：根环没有新根"
    );
    assert_eq!(
        after.recorded_operations, before.recorded_operations,
        "{fault:?}：一个写、一道屏障都没发"
    );
    assert_eq!(
        version.allocator.records(),
        allocation_records_before.as_slice(),
        "{fault:?}：分配器没动"
    );
}

/// 把 jsn 2 那一格还原，同一次调用照常写出第一个文件版本：拒绝是那一格引起的，不是别的。
fn assert_the_restored_record_lets_the_first_file_version_through(version: &mut VersionToBuildOn) {
    let original = version.original_previous_record.clone();
    overwrite_the_previous_record_slot(version, &original);
    let published = publish_the_first_file_version_after_the_record_on_the_ring(version)
        .expect("jsn 2 那一格还原之后照常写出第一个文件版本");
    assert_eq!(
        (published.root.checkpoint_txg, published.record.counter),
        (CheckpointTxg(3), 3),
        "txg 与 jsn 都接在 txg 2、jsn 2 之后"
    );
}

/// C495：要接的那一版之后的上一条记录两份都坏、两份都缺，`publish_first_file` 在任何写之前拒绝。
/// 判别力自证：让「上一条记录读不出」被当成接得上（jsn 从 1 起），这条用例红（`crates/mutations.tsv`）。
#[test]
fn c495_first_file_version_whose_previous_record_is_unreadable_on_both_copies_is_refused_before_any_write(
) {
    let mut version = mount_a_formatted_pool("c495-previous-record-unreadable");
    let filesystem_identifier = unit_filesystem_identifier(&parameters().filesystem_identifier);
    for fault in [
        FaultOnBothCopiesOfThePreviousRecord::Corrupted,
        FaultOnBothCopiesOfThePreviousRecord::Missing,
    ] {
        assert_refused_before_any_write(&mut version, fault, None);
        for (_, device) in &version.devices {
            assert!(
                JournalRecord::parse(
                    &read_ring_slot(device, PREVIOUS_RECORD_COUNTER),
                    filesystem_identifier
                )
                .is_none(),
                "{fault:?}：那一格两份都解不出"
            );
        }
    }
    assert_the_restored_record_lets_the_first_file_version_through(&mut version);
}

/// C495：要接的那一版之后的上一条记录解得出、但它的 txg 不等于那一版的（两份都是 jsn 1 那条，txg 1 ≠ 2），
/// `publish_first_file` 在任何写之前拒绝。
/// 判别力自证：核验只看解不解得出、不比 txg，这条用例红（`crates/mutations.tsv`）。
#[test]
fn c495_first_file_version_whose_previous_record_belongs_to_another_version_is_refused_before_any_write(
) {
    let mut version = mount_a_formatted_pool("c495-previous-record-of-another-version");
    assert_refused_before_any_write(
        &mut version,
        FaultOnBothCopiesOfThePreviousRecord::HoldTheRecordBeforeIt,
        Some((CheckpointTxg(1), 1)),
    );
    assert_the_restored_record_lets_the_first_file_version_through(&mut version);
}
