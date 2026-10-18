//! I-2.5（位置条目按设备身份升序，`crates/singlefs-checker/invariants/04-校验和.md`）的射程里有「journal 点名项里的位置条目数组」：
//! 顶着所选那一版的那条 journal 记录，点名项的两条位置条目也在重建上一版那一判里按 I-2.5 判（代码轮
//! `research/prompts/m3-b44-code-r3-main-verification.md` 第二节 L4 与第三节第 2 条：恢复这一侧别处没有按 I-2.5 判点名项的位置条目）；
//! 坏的交 `Recovery(InvariantViolated { invariant: "I-2.5", .. })`，可写挂载在任何写之前拒、两块盘逐字节不变。
//! 点名项的写者 `NamedUnit::write_to` 不断言升序，坏镜像不 panic，这一判之前它照收、重新编码之后照抄写回。
//! 坏镜像照 `journal_record_whose_new_root_segment_pointer_does_not_ascend_is_refused_by_the_writable_mount.rs` 那一形：新池新建文件之后，
//! 两块盘的 journal 环里与最新那条根同实例、同 txg、带本次发布末条标志的那条记录，每个点名项的两条位置条目对调（设备 1 在前），
//! 重新编码（记录头的宽校验和随编码重算）；根槽与单元一字不改。

mod common;
mod common_pointer_head_corruption;

use common::{build_pool, parameters};
use common_pointer_head_corruption::{newest_root, writable_mount_refused_before_any_write};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::journal::{record_offset, JournalRecord, JournalRecordPlaceInPublish};
use singlefs_core::mount::MountError;
use singlefs_core::recovery::{PoolReader, RecoveryFailure};
use singlefs_core::unit::unit_filesystem_identifier;
use singlefs_format::JOURNAL_RECORD_BYTES;
use singlefs_harness::memory_pool::MemoryPool;

/// 两块盘上与最新那条根同实例、同 txg、带本次发布末条标志的那条记录，每个点名项的两条位置条目对调、重新编码；交回改了几条记录（两块盘各一份）。
fn swap_the_location_entries_of_every_named_unit_in_the_record_of_the_newest_root(
    image: &mut MemoryPool,
) -> usize {
    let parameters = parameters();
    let root = newest_root(image);
    let filesystem_identifier = unit_filesystem_identifier(&parameters.filesystem_identifier);
    let ring_bytes = parameters.geometry.journal_ring_bytes;
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let mut changed = 0;
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        // 迭代上界是环里的记录槽数；读不出一条自证过的记录就停（新池新建文件之后环里只有开头那几条）。
        for counter in 1..=ring_bytes / JOURNAL_RECORD_BYTES {
            let offset = record_offset(counter, ring_bytes);
            let bytes =
                PoolReader::read(&*image, device, offset, record_bytes).expect("环里读得到");
            let Some(record) = JournalRecord::parse(&bytes, filesystem_identifier) else {
                break;
            };
            if record.instance != root.instance
                || record.checkpoint_txg != root.checkpoint_txg
                || record.place_in_publish != JournalRecordPlaceInPublish::LastRecordOfThePublish
            {
                continue;
            }
            let mut swapped = record.clone();
            assert!(
                !swapped.named.is_empty(),
                "新池新建文件那次发布的末条记录点名了单元"
            );
            // 迭代上界是这条记录的点名项数；不跨轮带状态；没有提前出口。
            for named_unit in &mut swapped.named {
                assert!(
                    named_unit.locations[0].device < named_unit.locations[1].device,
                    "改之前每个点名项的两条位置条目按设备身份升序"
                );
                named_unit.locations.swap(0, 1);
            }
            let swapped_bytes = swapped.to_bytes();
            assert_eq!(
                swapped_bytes.len(),
                record_bytes,
                "重新编码的记录照样一条宽"
            );
            let reparsed = JournalRecord::parse(&swapped_bytes, filesystem_identifier)
                .expect("重新编码的记录头校验和自洽，照样解得开");
            assert!(
                reparsed.named.iter().all(|named_unit| {
                    named_unit.locations.map(|location| location.device)
                        == [DeviceIdentity(1), DeviceIdentity(0)]
                }),
                "每个点名项两条位置条目对调之后设备 1 在前"
            );
            image
                .devices
                .get_mut(&device)
                .expect("池里有这块盘")
                .write(offset, &swapped_bytes);
            changed += 1;
        }
    }
    changed
}

/// 场景：新池新建文件之后，顶着最新那一版的那条 journal 记录每个点名项的两条位置条目都逆序（两块盘上那一份都改）。
/// 预期：可写挂载在任何写之前交 I-2.5 那一拒（两块盘逐字节不变由 `writable_mount_refused_before_any_write` 核），不 panic。
/// 判别力：重建不判点名项的位置条目（`crates/mutations.tsv`「4-4 修红 W17 L4：重建不按 I-2.5 判所选那一版那条 journal 记录的点名项」）时
/// 可写挂载照常重建、取号、往下写，`writable_mount_refused_before_any_write` 在挂载做成那一处红。
#[test]
fn a_record_whose_named_unit_location_entries_do_not_ascend_is_refused_by_the_writable_mount_before_any_write(
) {
    let mut image = build_pool("record-named-unit-locations-not-ascending").memory_pool();
    let changed =
        swap_the_location_entries_of_every_named_unit_in_the_record_of_the_newest_root(&mut image);
    assert_eq!(changed, 2, "两块盘上各改到一份那条记录");
    let refusal = writable_mount_refused_before_any_write(&image);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::InvariantViolated {
                invariant: "I-2.5",
                ..
            })
        ),
        "点名项两条位置条目逆序的记录在重建上一版那一判里按 I-2.5 拒：{refusal:?}"
    );
}
