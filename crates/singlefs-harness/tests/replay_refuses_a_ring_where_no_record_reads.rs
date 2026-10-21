//! 可写挂载在「所选根有文件而环里一条记录都读不出」时那道拒绝（`MountError::FileVersionWithoutAnyJournalRecord`，
//! `.claude/kb/checks-owed.md` C496（环里一条记录都读不出的那道拒绝的前提不承重）；判决 `research/prompts/m2-refusals-presumed-r1-main-verification.md`
//! 第三节 R3：登记成第一版不支持）。拒的时候手里已经有一条能把整版重建出来的根，只读恢复在记录全没了时照样把文件读回来；
//! 这道拒绝挡的只是「从盘上重建上一版要一条记录顶着，环里一条都拿不出来」（`mount.rs` 的 `record_standing_for_the_version`：
//! 所选根覆盖的最后一条读得出就是它，读不出就拿环里计数器最大的那一条顶着）。
//!
//! 两份盘面都从新池新建文件那一版出发（暖机 txg 1、2 各一条记录，第一个文件 txg 3 一条记录），根槽与单元都不动，
//! 系统配置的见证停在环里还读得出的记录上（[`witness_the_journal_only_up_to`]：不然判据 N-配置 先拒，走不到这一道）：
//! - 环里一条记录都读不出（两块盘上每一条都清零）：可写挂载在任何写之前拒成 `FileVersionWithoutAnyJournalRecord`，盘上逐字节不变；
//!   而所选根 (1, 3) 自己就把整版重建得出来（交给它一条记录顶着时），只读恢复读回第一个文件；
//! - 环里只剩与这一版无关的记录（第一个文件那一条清零，暖机那两条留着）：这道拒绝不挡，照常重建、可写挂载做成、现行那一版带着第一个文件。
//!
//! 判别力：顶着的记录只认所选根自己那一条（读不出不拿环里别的顶）时，两份盘面判得一样——都拒，后一条红。
//! 判据今天就是「环里读得出的记录不少于一条」：拿它当变异不改任何判定，所以变异行取的是只认自己那一条。

mod common;

use common::{build_pool, disk_snapshot, file_content, parameters, BuiltPool, Recorded};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::journal::record_offset;
use singlefs_core::mount::{mount_writable, MountError};
use singlefs_core::recovery::{
    rebuild_version, recover, JournalPolicy, RebuiltVersion, RecoveryOutcome,
};
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_format::{
    JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES,
};

/// 新池新建文件那一版的盘上写过记录的计数器：暖机那两次发布各一条，第一个文件那次一条。
struct RecordCountersOfTheBuiltPool {
    warm_up: Vec<u64>,
    first_file: u64,
}

fn record_counters_of(pool: &BuiltPool) -> RecordCountersOfTheBuiltPool {
    assert!(
        pool.output.earlier_records_of_this_publish.is_empty(),
        "一个数据单元的第一个文件那次发布只写一条记录"
    );
    RecordCountersOfTheBuiltPool {
        warm_up: pool
            .warm_up
            .records
            .iter()
            .map(|record| record.counter)
            .collect(),
        first_file: pool.output.record.counter,
    }
}

/// 两块盘 journal 默认环里计数器 `counter` 那一格清零（两份都读不出）。
fn zero_the_record_on_every_device(devices: &mut [(DeviceIdentity, Recorded)], counter: u64) {
    let zeros = vec![0u8; usize::try_from(JOURNAL_RECORD_BYTES).expect("4096")];
    for (_, device) in devices.iter_mut() {
        device
            .write_at(
                record_offset(counter, JOURNAL_RING_DEFAULT_BYTES),
                &zeros,
                WriteDurability::Plain,
            )
            .expect("清零一条记录");
    }
}

/// 两块盘四个系统配置槽见证的 journal tail 都改成 `witnessed_counter`（经读者解开、改字段、写者重写，整槽校验和随之重封）。
/// 系统配置见证到的那一条记录读不出、所选那一版的末条也读不出时，判据 N-配置 判不出、按真处置（C554 乙，`mount.rs` 的
/// `WitnessedCounterComparison::Undecidable`），可写挂载在这一道之前就拒成「较新的状态重读一次仍读不出」，走不到要看的那一道：
/// 见证停在环里还读得出的那几条（一条都不剩时停在 0，同取号写的那一档），盘面上只剩「记录读不出」这一样。
fn witness_the_journal_only_up_to(
    devices: &mut [(DeviceIdentity, Recorded)],
    witnessed_counter: u64,
) {
    let slot_spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    for (_, device) in devices.iter_mut() {
        for offset in [DeviceOffsetInBytes(0), DeviceOffsetInBytes(slot_spacing)] {
            let mut slot = vec![0u8; slot_bytes];
            device.read_at(offset, &mut slot).expect("读系统配置槽");
            let mut system_configuration =
                SystemConfiguration::parse_slot(&slot).expect("新池新建文件之后每一槽都自证得过");
            system_configuration.quantities.journal_tail = witnessed_counter;
            device
                .write_at(
                    offset,
                    &system_configuration.to_slot(),
                    WriteDurability::Plain,
                )
                .expect("写回系统配置槽");
        }
    }
}

/// 只读恢复读回第一个文件、走的是第一个文件那条根 (1, 3)。
fn assert_the_read_only_recovery_reads_the_first_file_back(pool: &BuiltPool, what: &str) {
    let report = recover(&pool.memory_pool(), JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(3)),
            content: file_content(),
        },
        "{what}：只读恢复照样从根 (1, 3) 读回第一个文件"
    );
}

#[test]
fn a_writable_mount_is_refused_before_any_write_when_no_record_in_the_ring_reads_although_the_root_rebuilds_the_whole_version(
) {
    let mut pool = build_pool("replay-refuses-a-ring-where-no-record-reads");
    let counters = record_counters_of(&pool);
    let record_of_the_first_file = pool.output.record.clone();
    let root_of_the_first_file = pool.output.root;
    let mut devices = pool.reopen_recorded();
    for counter in counters
        .warm_up
        .iter()
        .copied()
        .chain(std::iter::once(counters.first_file))
    {
        zero_the_record_on_every_device(&mut devices, counter);
    }
    witness_the_journal_only_up_to(&mut devices, 0);
    pool.devices = Some(devices);
    let image = pool.memory_pool();
    assert!(
        matches!(
            rebuild_version(
                &image,
                &root_of_the_first_file,
                Some(record_of_the_first_file)
            ),
            Ok(RebuiltVersion::WithFile(_))
        ),
        "所选根 (1, 3) 自己把整版重建得出来：交给它一条记录顶着就建得出带文件的一版"
    );
    assert_the_read_only_recovery_reads_the_first_file_back(&pool, "环里一条记录都读不出");
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let mut devices_of_the_mount = pool.devices.take().expect("镜像还开着");
    let refusal = mount_writable(&parameters(), &mut devices_of_the_mount).err();
    pool.devices = Some(devices_of_the_mount);
    assert!(
        matches!(
            refusal,
            Some(MountError::FileVersionWithoutAnyJournalRecord)
        ),
        "环里一条记录都读不出：可写挂载拒成 FileVersionWithoutAnyJournalRecord，实际 {refusal:?}"
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "拒之前一个写、一道屏障都没发"
    );
}

#[test]
fn a_writable_mount_rebuilds_the_version_as_usual_when_only_records_unrelated_to_it_are_left_in_the_ring(
) {
    let mut pool = build_pool("replay-with-only-unrelated-records-in-the-ring");
    let counters = record_counters_of(&pool);
    assert!(
        !counters.warm_up.is_empty(),
        "暖机那几次发布的记录留在环里，与第一个文件那一版无关"
    );
    let mut devices = pool.reopen_recorded();
    zero_the_record_on_every_device(&mut devices, counters.first_file);
    witness_the_journal_only_up_to(
        &mut devices,
        *counters
            .warm_up
            .iter()
            .max()
            .expect("上面判过暖机那几条不空"),
    );
    pool.devices = Some(devices);
    assert_the_read_only_recovery_reads_the_first_file_back(&pool, "环里只剩与这一版无关的记录");
    let mut devices_of_the_mount = pool.devices.take().expect("镜像还开着");
    let mounted = mount_writable(&parameters(), &mut devices_of_the_mount).unwrap_or_else(|refusal| {
        panic!("环里还有一条（与这一版无关的）记录：这道拒绝不挡，照常重建、挂载做成，实际 {refusal:?}")
    });
    pool.devices = Some(devices_of_the_mount);
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        (InstanceGeneration(1), CheckpointTxg(3)),
        "择根落在第一个文件那条根上"
    );
    let current = mounted
        .current
        .file_version()
        .expect("照常重建出来的那一版带着第一个文件");
    assert_eq!(
        current
            .inode_record
            .expect("第一个文件那条 inode 记录在")
            .size,
        u64::try_from(file_content().len()).expect("文件长装得进 u64"),
        "现行那一版就是第一个文件那一版接着写行与暖机"
    );
}
