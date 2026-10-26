//! 恢复、只读挂载、可写挂载三处判退的同一套（D23（journal 的角色与格式） 已定项 14 F2 段，`recovery::fall_back_judged_on_the_three_readers`）：
//! - C755（恢复与两种挂载触发退回的失败不同）：判退不看这一处自己的读者，看三样读者（冷走读、打开、建上一版）读过的单元的并集；并集只用来判退，
//!   落到哪一版之后成不成各看各的读者，退不成留在所选的那一版（用户 2026-10-20 定「三句都补」）。
//! - C866（F2 退回不分字节丢了与内容拒收） 改法甲：F2 只为字节丢了退；并集里有内容拒收就不退（判决
//!   `research/prompts/m3-close-c693-r3-main-verification.md` 第二节「㊁ 实现改法甲」那一格与第三节「㊁ 实现」那一行）。
//!
//! 盘面都在 384 槽小盘上：第一个文件之后覆盖写 5 次，再录一次覆盖写 R，崩在 R 的根槽之后、系统配置轮换之前（R 没见证，P 已确认）。
//! 判别力：`crates/mutations.tsv` 里以「根环一簇 R3」「根环一簇 R4」开头的行，各让这里点名的用例红。
mod common;
mod common_pointer_head_corruption;
mod common_unwitnessed_publish;

use common::PoolImageGeometry;
use common_pointer_head_corruption::replace_the_unit_and_propagate_its_checksum_on;
use common_unwitnessed_publish::{
    devices_reading_back_all_zero_on, image_of_devices_reading_back_all_zero,
    record_one_publish_after_overwrites, PublishRecordedAsR, RangesReadingBackAllZero,
    RecordedPublish,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::{mount_read_only, OpenPoolForReadFailure};
use singlefs_core::recovery::{
    recover, FailureKindForTheFallBack, JournalPolicy, RecoveryFailure, RecoveryOutcome,
};
use singlefs_core::unit::{build_index_node, parse_data_unit, parse_index_node, UnitError};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::mount_transparency::{read_recovery_fields, ComparedVersion};

const OVERWRITES_BEFORE_R: u64 = 5;
/// 树表条目里 flags 那两字节的起点（树 ID 8 + 条目长度 2 + 树的种类 2，D22（单元原子性怎么合成） 已定项 7「树表单元」）。
const TREE_TABLE_ENTRY_FLAGS_OFFSET: usize = 12;
/// flags 里任何种类上都没登记的一位（位 0 只在快照 inode 根条目上登记为「销毁中」）。
const TREE_TABLE_ENTRY_FLAG_NOBODY_REGISTERED: u8 = 0b10;

/// 三处各落到哪一版：恢复（读回内容或没有文件的那一版，失败时 `None`）、只读挂载、可写挂载（做成时那一版，做不成时 `None`）。
#[derive(Debug, PartialEq, Eq)]
struct LandingOfTheThreePlaces {
    recovery: Option<(InstanceGeneration, CheckpointTxg)>,
    read_only_mount: Option<(InstanceGeneration, CheckpointTxg)>,
    writable_mount: Option<(InstanceGeneration, CheckpointTxg)>,
}

fn landing_of_the_three_places(
    recorded: &RecordedPublish,
    image: &MemoryPool,
    ranges: &RangesReadingBackAllZero,
) -> LandingOfTheThreePlaces {
    let devices = devices_reading_back_all_zero_on(image, ranges);
    let recovery = match recover(&devices, JournalPolicy::Consult).outcome {
        RecoveryOutcome::FileRead { root, content } => {
            if let Some(content_of_the_version) = recorded.pool.content_of_each_root.get(&root) {
                assert_eq!(
                    &content, content_of_the_version,
                    "恢复读回的是它落到的那一版的内容"
                );
            }
            Some(root)
        }
        RecoveryOutcome::NoFile { root } => Some(root),
        RecoveryOutcome::Failed { .. } => None,
    };
    let read_only_mount = mount_read_only(&devices).ok().map(|mounted| {
        (
            mounted.effective_root.instance,
            mounted.effective_root.checkpoint_txg,
        )
    });
    let mut devices_of_the_mount = devices_reading_back_all_zero_on(image, ranges);
    let writable_mount = mount_writable(&recorded.pool.parameters, &mut devices_of_the_mount)
        .ok()
        .map(|mounted| {
            (
                mounted.output.effective_root.instance,
                mounted.output.effective_root.checkpoint_txg,
            )
        });
    LandingOfTheThreePlaces {
        recovery,
        read_only_mount,
        writable_mount,
    }
}

/// 场景：C755 那一行「怎么拦」的盘面——R 只有一个数据单元，它的两份读回全零，R 的记录读得出。
/// 预期：冷走读读那个数据单元、读不出（字节丢了）；打开不读数据单元、建上一版照抄读不出的数据单元的位置项，这两样读得通。
/// 三处判退看并集，三处都退到见证到的 P、都落在 P（今天恢复落 P、两种挂载落 R）。
/// 判别力：「根环一簇 R3：判退的并集里不收冷走读那一样」那一行（两种挂载照旧落 R），这一条红。
#[test]
fn a_data_unit_of_the_unwitnessed_version_reading_back_zero_lands_the_three_places_on_the_same_previous_version(
) {
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    let data_unit_groups: Vec<_> = recorded
        .unit_writes_grouped_by_unit()
        .into_iter()
        .filter(|copies| parse_data_unit(copies[0].bytes().expect("单元写带字节")).is_ok())
        .collect();
    assert_eq!(
        data_unit_groups.len(),
        1,
        "前提：覆盖写 R 只写了一个数据单元"
    );
    assert_eq!(data_unit_groups[0].len(), 2, "前提：那个数据单元两份");
    let image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let ranges = RangesReadingBackAllZero::default().with_the_writes(&data_unit_groups[0]);
    assert_eq!(
        landing_of_the_three_places(&recorded, &image, &ranges),
        LandingOfTheThreePlaces {
            recovery: Some(recorded.previous_key),
            read_only_mount: Some(recorded.previous_key),
            writable_mount: Some(recorded.previous_key),
        },
        "R 的数据单元读不出：三处都落在见证到的 P（R 是 {:?}）",
        recorded.key
    );
}

/// R 的树表单元第一条条目的 flags 置上一个没人登记的位，按原来的头重建、重封，新的整单元校验和沿引用链补到 R 的根槽（与 R 的记录的新根段）。
fn image_whose_unwitnessed_version_carries_a_tree_table_entry_with_an_unknown_flag(
    recorded: &RecordedPublish,
) -> MemoryPool {
    let mut image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let tree_table_slot = recorded.root().tree_table.locations[0].slot;
    let node_bytes = usize::try_from(singlefs_format::NODE_BYTES).expect("16384");
    let before = image
        .devices
        .get(&DeviceIdentity(0))
        .expect("池里有盘 0")
        .read(tree_table_slot.to_device_offset(), node_bytes);
    let node = parse_index_node(&before).expect("R 的树表单元解得开");
    let mut entries = node.entries.clone();
    entries[0][TREE_TABLE_ENTRY_FLAGS_OFFSET] |= TREE_TABLE_ENTRY_FLAG_NOBODY_REGISTERED;
    let after = build_index_node(
        node.tree,
        node.level,
        node.key_width,
        &node.smallest_key,
        &node.largest_key,
        node.birth_txg,
        &recorded.pool.parameters.filesystem_identifier,
        node.instance,
        node.birth_sequence,
        u16::try_from(node.entry_width).expect("条目宽 200"),
        &entries,
    );
    replace_the_unit_and_propagate_its_checksum_on(
        &mut image,
        &PoolImageGeometry {
            parameters: recorded.pool.parameters.clone(),
            device_bytes: recorded.pool.device_bytes,
        },
        tree_table_slot,
        &after,
    );
    image
}

/// 场景：没见证的内容拒收盘面（C866 那一行「会红的形态」）：R 的树表条目带 flags 未知位（字节读得到、整单元校验和对得上，读者不收），
/// 读的那一刻没有一段读回全零。
/// 预期：恢复、只读挂载、可写挂载都不退回 P：恢复照报 R 的树表条目解不开，两种挂载都做不成，可写挂载在任何写之前拒、盘上逐字节不变；
/// 挂载透明判官读这份盘面（`read_recovery_fields`）不 panic、落到的同是 R（判决第二节 T2-乙）。今天三处都退回 P。
/// 判别力：「根环一簇 R4：解不开的单元算成字节丢了」那一行（三处退回 P），这一条红。
#[test]
fn an_unwitnessed_version_whose_tree_table_entry_carries_an_unknown_flag_is_not_fallen_back_from() {
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    let image =
        image_whose_unwitnessed_version_carries_a_tree_table_entry_with_an_unknown_flag(&recorded);
    let no_range = RangesReadingBackAllZero::default();
    let devices = devices_reading_back_all_zero_on(&image, &no_range);
    assert_eq!(
        recover(&devices, JournalPolicy::Consult).outcome,
        RecoveryOutcome::Failed {
            root: Some(recorded.key),
            failure: RecoveryFailure::UnitMalformed {
                what: "树表条目"
            },
        },
        "恢复：内容拒收不退，照报 R 的树表条目解不开"
    );
    assert!(
        mount_read_only(&devices).is_err(),
        "只读挂载：不退，打不开 R"
    );
    let mut devices_of_the_mount = devices_reading_back_all_zero_on(&image, &no_range);
    assert!(
        mount_writable(&recorded.pool.parameters, &mut devices_of_the_mount).is_err(),
        "可写挂载：不退，建不出 R"
    );
    assert!(
        image_of_devices_reading_back_all_zero(&devices_of_the_mount, recorded.pool.device_bytes)
            == image,
        "可写挂载在任何写之前拒：两块盘逐字节不变"
    );
    let fields = read_recovery_fields(&devices, ComparedVersion::TheRecoveredVersion);
    assert_eq!(
        fields.report.effective_root,
        Some(recorded.key),
        "挂载透明判官照恢复同一份判退，不 panic，落到 R"
    );
}

/// 场景：只读挂载打开那一样读者报的几种失败交给 F2 判退时归哪一类。
/// 预期：读回来的单元解不开（`UnitMalformed`：位置条目里的整单元校验和对得上，节点自述的载荷校验和不对）是内容拒收，不算字节丢了
/// （判决第二节 T2-甲 打中草稿的那一处）；沿根读单元读不出是字节丢了；树表 0 条不是读坏。
#[test]
fn the_opening_reader_counts_a_unit_it_cannot_parse_as_refused_content_and_not_as_lost_bytes() {
    assert_eq!(
        OpenPoolForReadFailure::UnitMalformed {
            what: "树表单元",
            error: UnitError::PayloadChecksumMismatch,
        }
        .failure_kind_for_the_fall_back(),
        Some(FailureKindForTheFallBack::ContentRefused),
        "解不开的单元：内容拒收"
    );
    assert_eq!(
        OpenPoolForReadFailure::Walk(RecoveryFailure::UnitUnreadable {
            slot: singlefs_core::address::SlotNumber(1),
        })
        .failure_kind_for_the_fall_back(),
        Some(FailureKindForTheFallBack::BytesLost),
        "读不出的单元：字节丢了"
    );
    assert_eq!(
        OpenPoolForReadFailure::TreeTableHasNoEntries.failure_kind_for_the_fall_back(),
        None,
        "树表 0 条：不是读坏"
    );
}
