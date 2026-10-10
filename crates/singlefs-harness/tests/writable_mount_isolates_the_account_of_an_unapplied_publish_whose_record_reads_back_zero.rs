//! C 的账那一份（判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节「K2 T1 C 的盲区」那一行与第三节第 1 条，攻方 `C_ACCOUNT`）：
//! 可写挂载读阶段、重建分配器之后，水位之上每一次没施加的发布，用它任一条读得出的记录的新根段拼一条根、按影子账同一份读法读它的账，
//! 账里的落点并进 C 的集合（账读不出就不加，交 F1 乙），解开时机照 C（`ring`）。
//!
//! 造法（`tests/common_unwitnessed_publish/mod.rs`，攻方报告第三节第 1 小节那一形）：384 槽小盘、第一个文件之后覆盖写 1 次，再录一次三条记录的发布 R
//! （顺序写三个数据单元），崩在 R 的根槽落盘之后、系统配置轮换之前；取新实例的那次可写挂载 M1 里 R 的根槽与 R 的某一条记录（末条 `rL`、中间一条 `rM`、
//! 第一条 `rF`）读回全零，R 的单元读得出。F1 乙不拒（读得出的那两条点名的单元验得过），缺的那条记录单独点名的单元不在别的记录里，
//! R 的根槽读不出、影子账也不知道 R 是一条根；读得出的记录的新根段说得出 R 的树表在哪，R 的账（单元都好）读得出。
//! M1 之后照常覆盖写一次、再正常卸载：R 的单元一份都没被写掉，卸载之后池级 checker 不红 I-7.4（被抛弃根引用的单元被重新分配）。
mod common_unwitnessed_publish;

use common_unwitnessed_publish::{
    checker_violations_on, devices_reading_back_all_zero_on,
    image_of_devices_reading_back_all_zero, plain_devices_on, pool_after_the_mount,
    record_one_publish_after_overwrites, units_of_r_overwritten_on, PublishRecordedAsR,
    RangesReadingBackAllZero, OVERWRITE_BYTES,
};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::journal::JournalRecord;
use singlefs_core::mount::{mount_writable, unmount, ShadowLedger};
use singlefs_core::unit::unit_filesystem_identifier;
use singlefs_harness::memory_pool::MemoryPool;

/// 覆盖写几次之后录 R：攻方那一形量过 k = 1 与 4（两者逐格同数），取 1。
const OVERWRITES_BEFORE_R: u64 = 1;

/// M1 里 R 的根槽与 R 的第 `hidden_record` 条记录读回全零；M1 之后覆盖写一次、正常卸载，逐步核 R 的单元没被写掉、卸载之后 I-7.4 不红。
/// 交回 M1 之后每块盘上分配器隔离的槽数（报告里写的「M1 隔离槽数」）。
fn assert_the_units_of_the_publish_stay_unwritten_through_an_overwrite_and_the_unmount_when_record_reads_back_zero(
    hidden_record: usize,
) -> Vec<(DeviceIdentity, u64)> {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_R,
        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
    );
    assert_eq!(recorded.record_count(), 3, "顺序写三个数据单元切成三条记录");
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&[recorded.root_slot_write()])
        .with_the_writes(&recorded.record_writes_of_the_record(hidden_record));
    let mut devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
    let mounted = mount_writable(&recorded.pool.parameters, &mut devices)
        .expect("R 缺一条记录、读得出的记录点名的单元验得过：F1 乙不拒，可写挂载做得成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "R 的根槽读回全零、缺一条记录：R 整次不施加，所选那一版是 P"
    );
    let isolated_slots_after_the_mount: Vec<(DeviceIdentity, u64)> = mounted
        .allocator
        .devices
        .iter()
        .map(|device_map| (device_map.device, device_map.isolated_slots()))
        .collect();
    let image_after_the_mount =
        image_of_devices_reading_back_all_zero(&devices, recorded.pool.device_bytes);
    let mut pool = pool_after_the_mount(&recorded, &image_after_the_mount, mounted);
    pool.overwrite(OVERWRITE_BYTES)
        .expect("384 槽小盘上 M1 之后覆盖写一次落得下");
    assert_eq!(
        units_of_r_overwritten_on(&recorded, &pool.image()),
        Vec::<(DeviceIdentity, u64)>::new(),
        "M1 之后第一次覆盖写没取到 R 的单元（C 隔离了 R 的账里的落点）"
    );
    let session = pool.session();
    let mut devices_of_the_unmount = plain_devices_on(&pool.image());
    let mut allocator_of_the_unmount = session.allocator.clone();
    let mut current_of_the_unmount = session.current.clone();
    unmount(
        &recorded.pool.parameters,
        &mut devices_of_the_unmount,
        &mut allocator_of_the_unmount,
        &mut current_of_the_unmount,
        ShadowLedger::On,
    )
    .expect("M1 之后覆盖写一次的池正常卸载做得成");
    let image_after_the_unmount = MemoryPool {
        devices: devices_of_the_unmount
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: recorded.pool.device_bytes,
    };
    assert_eq!(
        units_of_r_overwritten_on(&recorded, &image_after_the_unmount),
        Vec::<(DeviceIdentity, u64)>::new(),
        "卸载那一串空发布也没取到 R 的单元"
    );
    let reallocated_units_of_abandoned_roots: Vec<String> =
        checker_violations_on(&image_after_the_unmount)
            .into_iter()
            .filter(|violation| violation.starts_with("I-7.4"))
            .collect();
    assert_eq!(
        reallocated_units_of_abandoned_roots,
        Vec::<String>::new(),
        "卸载之后池级 checker 不红 I-7.4：被抛弃的 R 引用的单元没被重新分配"
    );
    isolated_slots_after_the_mount
}

/// `rL`：末条读回全零，末条单独点名的第三个数据单元与那几个元数据单元只在 R 的账里。
/// 判别力：C 不并 R 的账（变异「C 不并没施加的发布的账（末条读回全零）」）时 M1 之后第一次覆盖写就写掉 R 的单元（攻方量到 2 份）、这一条红。
#[test]
fn a_publish_whose_root_slot_and_last_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount(
) {
    let isolated_slots =
        assert_the_units_of_the_publish_stay_unwritten_through_an_overwrite_and_the_unmount_when_record_reads_back_zero(3);
    println!("M1 隔离槽数（rL）：{isolated_slots:?}");
}

/// `rM`：中间一条读回全零，它单独点名的第二个数据单元只在 R 的账里。判别力：C 不并 R 的账（变异「C 不并没施加的发布的账（中间一条读回全零）」）时这一条红。
#[test]
fn a_publish_whose_root_slot_and_middle_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount(
) {
    let isolated_slots =
        assert_the_units_of_the_publish_stay_unwritten_through_an_overwrite_and_the_unmount_when_record_reads_back_zero(2);
    println!("M1 隔离槽数（rM）：{isolated_slots:?}");
}

/// `rF`：第一条读回全零，它单独点名的第一个数据单元只在 R 的账里。判别力：C 不并 R 的账（变异「C 不并没施加的发布的账（第一条读回全零）」）时这一条红。
#[test]
fn a_publish_whose_root_slot_and_first_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount(
) {
    let isolated_slots =
        assert_the_units_of_the_publish_stay_unwritten_through_an_overwrite_and_the_unmount_when_record_reads_back_zero(1);
    println!("M1 隔离槽数（rF）：{isolated_slots:?}");
}

/// C 选法 `all`（判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 1 条：不只认带末条标志的记录）在 `C_ACCOUNT` 之下分得开的取样点
/// （`.claude/rules/mutation-sampling.md`「没红的三类，判据不同」第三类：原来那一格的末条读不出时 `C_ACCOUNT` 读得出 R 的账、把前两条点名的落点
/// 一并隔离，选法退成只认末条也照样护住）：R 的三条记录都读得出，M1 里 R 的根槽与 R 的树表读回全零。末条点名的树表两份都验不过，R 整次不施加，
/// 所选那一版是 P；三条记录齐（序号 1..3、带末条），F1 乙不判，C 的账读不出也不从映射现算；记录拼根读 R 的账经树表、读不出，`C_ACCOUNT` 什么都不加
/// ——前两条点名的落点只有 C 记录那一份护着。
/// 判别力：选法退成只认带末条标志的记录（变异「第33批T C694 的 C 选法退成只认带末条标志的记录（last）」）时前两条点名的数据单元 M1 之后空闲、这一条红。
#[test]
fn the_units_named_by_the_first_two_records_of_a_publish_whose_root_slot_and_tree_table_read_back_zero_are_isolated(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_R,
        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
    );
    assert_eq!(recorded.record_count(), 3, "顺序写三个数据单元切成三条记录");
    let tree_table_writes = recorded.unit_writes_at(&recorded.root().tree_table);
    assert_eq!(tree_table_writes.len(), 2, "R 自己写了树表，两块盘各一份");
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&[recorded.root_slot_write()])
        .with_the_writes(&tree_table_writes);
    let mut devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
    let mounted = mount_writable(&recorded.pool.parameters, &mut devices)
        .expect("R 的三条记录齐、树表读不出：F1 乙不判，可写挂载做得成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "R 的根槽读回全零、末条点名的树表验不过：R 整次不施加，所选那一版是 P"
    );
    let filesystem_identifier =
        unit_filesystem_identifier(&recorded.pool.parameters.filesystem_identifier);
    for ordinal in [1, 2] {
        let record = JournalRecord::parse(
            recorded.record_writes_of_the_record(ordinal)[0]
                .bytes()
                .expect("记录写带字节"),
            filesystem_identifier,
        )
        .expect("R 的记录解得开");
        assert!(!record.named.is_empty(), "第 {ordinal} 条记录点名了单元");
        for named in &record.named {
            for location in &named.locations {
                let device_map = mounted
                    .allocator
                    .devices
                    .iter()
                    .find(|device_map| device_map.device == location.device)
                    .expect("点名项的盘在池里");
                assert!(
                    !device_map.is_free(location.slot),
                    "R 第 {ordinal} 条记录点名的落点（盘 {:?} 槽 {:?}）M1 之后不空闲（C 隔离了它）",
                    location.device,
                    location.slot
                );
            }
        }
    }
}
