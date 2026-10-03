//! 全环扫描建记录表时，同一个 (实例代号, 计数器) 在盘上有两份内容不同、各自自证得过的记录：表里留先扫到的那一份
//! （`recovery::scan_journal` 按 `device_identities()` 的次序逐盘扫，先到先得）。
//! 取前缀的核要的「键唯一」在真路径上就靠这一判（形式化题面第三轮判决 `research/prompts/m3-formal-r3-main-verification.md`
//! 第二节 S3、第四节欠账第 10 条）；改成后到覆盖，表里留的就换成后一块盘上的那一份。
//!
//! 造法：A（txg 3、jsn 3）的根落了，B（txg 4、jsn 4）一次单条发布只剩记录与单元、根槽坏了。1 号盘上 jsn 4 那一份的提交标记改成 0、重封
//! （0 号盘那一份原样）：两份都自证得过、内容不同。留 0 号盘那一份 ⇒ B 施加；留 1 号盘那一份 ⇒ B 的末条不带提交标记，整次不施加。

mod common;

use common::{
    build_pool, overwrite_in_process_as_instance_one_and_advance_the_pool, parameters,
    tear_the_root_slot_written_for_txg, CONTENT_BYTES_OF_PUBLISH_B,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::journal::{record_offset, JournalRecord};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::{choose_system_configuration, scan_journal, PoolReader};
use singlefs_core::unit::unit_filesystem_identifier;
use singlefs_format::JOURNAL_RECORD_BYTES;
use singlefs_harness::memory_pool::MemoryPool;

/// B 那一版文件的填充字节。
const CONTENT_FILL_BYTE_OF_PUBLISH_B: u8 = 0xB0;
/// 扫环先扫的盘（`MemoryPool::device_identities` 按盘号升序）与后扫的盘。
const DEVICE_SCANNED_FIRST: DeviceIdentity = DeviceIdentity(0);
const DEVICE_SCANNED_SECOND: DeviceIdentity = DeviceIdentity(1);

/// `device` 上 jsn `counter` 那一格解出来的记录。
fn record_on_device(
    image: &MemoryPool,
    device: DeviceIdentity,
    ring_bytes: u64,
    counter: u64,
) -> Option<JournalRecord> {
    let bytes = PoolReader::read(
        image,
        device,
        record_offset(counter, ring_bytes),
        usize::try_from(JOURNAL_RECORD_BYTES).expect("4096"),
    )
    .expect("环里那一格读得到");
    JournalRecord::parse(
        &bytes,
        unit_filesystem_identifier(&parameters().filesystem_identifier),
    )
}

/// 只改 `device` 上 jsn `counter` 那一份：解出来、改一处、`JournalRecord::to_bytes` 重封载荷与头部校验和、写回；另一块盘那一份不动。
fn rewrite_the_copy_on_one_device(
    image: &mut MemoryPool,
    device: DeviceIdentity,
    ring_bytes: u64,
    counter: u64,
    change: impl Fn(&mut JournalRecord),
) {
    let mut record =
        record_on_device(image, device, ring_bytes, counter).expect("改之前这一份自证过");
    change(&mut record);
    image
        .devices
        .get_mut(&device)
        .expect("有这块盘")
        .write(record_offset(counter, ring_bytes), &record.to_bytes());
}

/// 同一个键两份内容不同的记录：表里是 0 号盘（先扫）那一份；只读挂载照它施加 B。
#[test]
fn of_two_differing_copies_of_one_record_key_the_scan_keeps_the_copy_on_the_device_scanned_first() {
    let mut pool = build_pool("journal-scan-first-copy-of-a-duplicated-key");
    let first_version = (pool.output.root.instance, pool.output.root.checkpoint_txg);
    let second = overwrite_in_process_as_instance_one_and_advance_the_pool(
        &mut pool,
        &vec![CONTENT_FILL_BYTE_OF_PUBLISH_B; CONTENT_BYTES_OF_PUBLISH_B],
    );
    assert_eq!(
        (
            first_version,
            (second.root.checkpoint_txg, second.record.counter)
        ),
        (
            (InstanceGeneration(1), CheckpointTxg(3)),
            (CheckpointTxg(4), 4)
        ),
        "A 的根在 txg 3、jsn 3；B 一条记录 jsn 4"
    );
    let mut image = pool.memory_pool();
    tear_the_root_slot_written_for_txg(&mut image, 4);
    let system_configuration = choose_system_configuration(&image).expect("系统配置择得出");
    let ring_bytes = system_configuration.immutable.sizes.journal_ring_bytes;
    rewrite_the_copy_on_one_device(&mut image, DEVICE_SCANNED_SECOND, ring_bytes, 4, |record| {
        record.is_commit = false;
    });
    let copy_scanned_first = record_on_device(&image, DEVICE_SCANNED_FIRST, ring_bytes, 4)
        .expect("0 号盘那一份自证得过");
    let copy_scanned_second = record_on_device(&image, DEVICE_SCANNED_SECOND, ring_bytes, 4)
        .expect("1 号盘那一份重封过，自证得过");
    assert_eq!(
        (
            copy_scanned_first.is_commit,
            copy_scanned_second.is_commit,
            copy_scanned_first == copy_scanned_second
        ),
        (true, false, false),
        "同一个键 (1, 4) 两份内容不同：0 号盘的带提交标记，1 号盘的不带"
    );
    assert_eq!(
        image.device_identities(),
        [DEVICE_SCANNED_FIRST, DEVICE_SCANNED_SECOND],
        "扫环按盘号升序逐盘扫"
    );

    let records = scan_journal(&image, &system_configuration);
    assert_eq!(
        records.get(&(InstanceGeneration(1), 4)),
        Some(&copy_scanned_first),
        "表里 (1, 4) 是先扫到的 0 号盘那一份，不被 1 号盘那一份盖掉"
    );

    let mounted = mount_read_only(&image).expect("只读挂载");
    assert_eq!(
        (
            (
                mounted.chosen_root.instance,
                mounted.chosen_root.checkpoint_txg
            ),
            (
                mounted.effective_root.instance,
                mounted.effective_root.checkpoint_txg
            ),
            mounted.journal.above_water,
            mounted.journal.prefix_applied
        ),
        (
            first_version,
            (InstanceGeneration(1), CheckpointTxg(4)),
            1,
            1
        ),
        "所选根是 A 的；照 0 号盘那一份（带提交标记）施加 B，挂载到 B"
    );
}
