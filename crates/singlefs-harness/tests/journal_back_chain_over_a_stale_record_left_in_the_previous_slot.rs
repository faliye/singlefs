//! C569（前一槽躺着残留记录时反向链两边判法不同）：新实例接着前缀末 + 1 写、其中一条没持久、那一槽还躺着上一条时间线的残留记录。
//! I-8.6（反向链算法） 射程（`crates/singlefs-checker/invariants/03-journal.md`，用户 2026-10-19 定「不收，射程写明」）：
//! 池级 checker 把下一条认成本实例第一条、按链值应为 0 判违例，这一格判违例是有意的；恢复只在同实例前一条在盘上时比链，这一格不比。
//!
//! 镜像的造法：同一个 fsid 的池各走一段历史（`build_pool` 的 fsid 与写入时间是固定的，两段历史在 jsn 4 之前逐字节相同）——
//! 旧时间线上实例 1 写出 jsn 5（txg 5）；新时间线上同一段前缀之后重开、可写挂载取实例 2，写行发布写 jsn 5、暖机写 jsn 6、jsn 7。
//! 把旧时间线的 jsn 5 两份镜像放回新时间线 jsn 5 那一槽：就是「实例 2 的 jsn 5 没持久、那一槽躺着实例 1 的 jsn 5」那张盘面
//! （挂载那一刻 jsn 5 读不出、之后实例 2 那一写没持久，只在读故障加子集持久的历史里造得出）。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/writable_mount_takes_a_second_instance.rs"]
mod scenarios_of_writable_mount_takes_a_second_instance;

use common::{build_pool, E142_FILESYSTEM_IDENTIFIER};
use scenarios_of_writable_mount_takes_a_second_instance::{
    build_publish_second_version_and_remount, overwrite_in_process, second_content, third_content,
    RollbackFloorHandedToTheWriter,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::journal::{record_offset, JournalRecord};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::unit::unit_filesystem_identifier;
use singlefs_format::{JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES};
use singlefs_harness::memory_pool::MemoryPool;

/// 新实例写行发布那条记录的计数器：前缀末（实例 1 的 jsn 4，第二版那次发布）+ 1。
const COUNTER_OF_THE_ROW_PUBLISH: u64 = 5;

/// 新实例紧接在写行之后的那一条（第一次暖机）：它的计数器 − 1 那一槽就是躺着残留记录的那一槽。
const COUNTER_OF_THE_FIRST_WARM_UP: u64 = COUNTER_OF_THE_ROW_PUBLISH + 1;

fn journal_record_bytes() -> usize {
    usize::try_from(JOURNAL_RECORD_BYTES).expect("4096")
}

fn journal_record_on(image: &MemoryPool, device: DeviceIdentity, counter: u64) -> JournalRecord {
    let bytes = image.devices.get(&device).expect("池里两块盘").read(
        record_offset(counter, JOURNAL_RING_DEFAULT_BYTES),
        journal_record_bytes(),
    );
    JournalRecord::parse(
        &bytes,
        unit_filesystem_identifier(&E142_FILESYSTEM_IDENTIFIER),
    )
    .expect("那一槽的记录自证过")
}

fn verdict_of_the_back_chain_invariant(image: &MemoryPool) -> InvariantVerdict {
    check_pool_image(image)
        .into_iter()
        .find(|(invariant, _)| *invariant == "I-8.6")
        .expect("I-8.6 在池级 checker 的清单里")
        .1
}

#[test]
fn the_pool_checker_reddens_the_back_chain_of_the_record_after_a_stale_record_of_the_previous_timeline_while_the_recovery_reads_the_new_instance(
) {
    // 旧时间线：实例 1 在第二版之后再发一版，写出 jsn 5（txg 5）。
    let mut old_timeline = build_pool("c569-old-timeline");
    old_timeline.output = overwrite_in_process(
        &mut old_timeline,
        &second_content(),
        InstanceGeneration(1),
        RollbackFloorHandedToTheWriter::AsInThePreviousVersion,
    );
    let stray = overwrite_in_process(
        &mut old_timeline,
        &third_content(),
        InstanceGeneration(1),
        RollbackFloorHandedToTheWriter::AsInThePreviousVersion,
    );
    assert_eq!(
        (
            stray.record.instance,
            stray.record.counter,
            stray.root.checkpoint_txg
        ),
        (
            InstanceGeneration(1),
            COUNTER_OF_THE_ROW_PUBLISH,
            CheckpointTxg(5)
        ),
        "取样点：旧时间线上实例 1 写出的那一条坐在 jsn 5"
    );
    let old_timeline_image = old_timeline.memory_pool();

    // 新时间线：同一段前缀（第二版）之后重开、可写挂载取实例 2，写行 jsn 5、暖机 jsn 6、jsn 7。
    let (new_timeline, _second, mounted) =
        build_publish_second_version_and_remount("c569-new-timeline");
    assert_eq!(
        (
            mounted.output.instance,
            mounted.output.row_publish.record().counter,
            mounted.output.row_publish.record().back_chain
        ),
        (InstanceGeneration(2), COUNTER_OF_THE_ROW_PUBLISH, 0),
        "取样点：实例 2 接着前缀末 + 1 写，写行那一条是本实例第一条、链 0"
    );
    let mut image = new_timeline.memory_pool();
    let first_warm_up = journal_record_on(&image, DeviceIdentity(0), COUNTER_OF_THE_FIRST_WARM_UP);
    assert_eq!(
        first_warm_up.instance,
        InstanceGeneration(2),
        "取样点：jsn 6 是实例 2 的暖机"
    );
    assert_ne!(
        first_warm_up.back_chain, 0,
        "取样点：jsn 6 的链接实例 2 的 jsn 5，不是 0"
    );
    assert_eq!(
        verdict_of_the_back_chain_invariant(&image),
        InvariantVerdict::Holds,
        "对照：实例 2 的 jsn 5 持久时 I-8.6 成立"
    );

    // 实例 2 的 jsn 5 没持久：那一槽两份镜像都还是旧时间线实例 1 的 jsn 5。
    let offset = record_offset(COUNTER_OF_THE_ROW_PUBLISH, JOURNAL_RING_DEFAULT_BYTES);
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let stale = old_timeline_image
            .devices
            .get(&device)
            .expect("池里两块盘")
            .read(offset, journal_record_bytes());
        image
            .devices
            .get_mut(&device)
            .expect("池里两块盘")
            .write(offset, &stale);
        assert_eq!(
            journal_record_on(&image, device, COUNTER_OF_THE_ROW_PUBLISH).instance,
            InstanceGeneration(1),
            "盘 {device:?} 的 jsn 5 那一槽躺着实例 1 的残留记录、自证过"
        );
    }

    match verdict_of_the_back_chain_invariant(&image) {
        InvariantVerdict::Violated(detail) => assert!(
            detail.contains("实例 2、计数器 6"),
            "I-8.6 红在 jsn 6 上（计数器 − 1 那一条是别的实例的，按本实例第一条链应为 0）：{detail}"
        ),
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
            panic!("I-8.6 射程定的判法：残留记录坐在前一槽时池级 checker 判违例；实际 {other:?}")
        }
    }

    // 恢复这一格不比链：择实例 2 最新的根、读回第二版的内容。
    let report = recover(&image, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(2), mounted.current.root().checkpoint_txg),
            content: second_content()
        },
        "恢复照常择实例 2 的根：{:?}",
        report.journal
    );
}
