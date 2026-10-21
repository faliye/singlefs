//! 释放判定的两臂各能被测试强制进入（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3 末句「只供测试的开关：强制
//! 「立即释放 / 进 deadlist」」；`research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F4；`.claude/rules/fs-design.md` 五条硬要求第 2 条）：
//! 开关装在写入口上（`PoolWriter::set_snapshot_lifetime_judgement`），新开的写入口照判定。照判定时快照下覆盖写把换下的四个用户可见单元放进
//! deadlist、比快照新的单元释放（`snapshot_release_and_destroy_through_the_deadlist.rs`）；这里各取判定走不到的那一边强制进入：快照下强制
//! 立即释放、比快照新的单元强制进 deadlist。活头没打过快照时强制进 deadlist 会写出纪元 0 的条目，发布在任何写之前断言停下
//! （判决 `research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J4）。期望值从覆盖写之前那一版现算（它的映射 key 与落点）。
mod common;

use std::collections::BTreeSet;

use common::{build_pool, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::records::{birth_txg_of_a_mapping_key, DeadlistEntry};
use singlefs_core::transaction::{
    asks_the_snapshot_lifetime_judgement, publish_create_snapshot, publish_overwrite, FirstFile,
    PoolWriter, SnapshotLifetimeJudgement, TransactionOutput, TransactionUnit,
};

const INSTANCE: InstanceGeneration = InstanceGeneration(1);

/// 覆盖写的内容：一个数据单元装得下、与第一个文件不同的一份。
fn overwrite_content() -> Vec<u8> {
    (0..2700_u32)
        .map(|index| u8::try_from((index * 13 + 7) % 239).expect("小于 256"))
        .collect()
}

/// 同一个进程里接着 `pool.output` 建一个快照，池接到它那一版上。
fn create_snapshot_and_advance(pool: &mut BuiltPool) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let created = publish_create_snapshot(&mut writer, &mut pool.allocator, &previous, INSTANCE)
        .expect("建快照");
    pool.output = created.clone();
    created
}

/// 写入口装上 `judgement` 之后覆盖写一次，池接到它那一版上；交回写入口报的那一臂（运行时看得出走了哪一条）与发成的那一版。
fn overwrite_with_the_judgement_and_advance(
    pool: &mut BuiltPool,
    judgement: SnapshotLifetimeJudgement,
) -> (SnapshotLifetimeJudgement, TransactionOutput) {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    writer.set_snapshot_lifetime_judgement(judgement);
    let content = overwrite_content();
    let overwritten = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        INSTANCE,
    )
    .expect("覆盖写");
    pool.output = overwritten.clone();
    (writer.snapshot_lifetime_judgement(), overwritten)
}

/// 一版里六类用户可见角色各自的映射 key。
fn user_visible_mapping_keys(version: &TransactionOutput) -> Vec<(TransactionUnit, Vec<u8>)> {
    version
        .mapped_units
        .iter()
        .filter(|(identity, _)| asks_the_snapshot_lifetime_judgement(*identity))
        .cloned()
        .collect()
}

fn released_slots(version: &TransactionOutput) -> BTreeSet<SlotNumber> {
    version
        .released
        .iter()
        .map(|placement| placement.slot)
        .collect()
}

fn allocated_on_every_device(pool: &BuiltPool, slot: SlotNumber) -> bool {
    [DeviceIdentity(0), DeviceIdentity(1)].iter().all(|device| {
        pool.allocator
            .record_for(*device, slot)
            .is_some_and(|record| !record.is_released)
    })
}

/// 新开的写入口照判定（产品路径不认开关）。
#[test]
fn a_new_pool_writer_follows_the_snapshot_lifetime_judgement() {
    let mut pool = build_pool("snapshot-lifetime-judgement-of-a-new-writer");
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    assert_eq!(
        writer.snapshot_lifetime_judgement(),
        SnapshotLifetimeJudgement::FollowTheJudgement
    );
}

/// 快照下强制立即释放：换下的四个用户可见单元都生在建快照之前（判定会让它们进 deadlist），强制那一臂照样把它们放进这次的释放，
/// deadlist 一条不写、没有留存的映射条目。
#[test]
fn forcing_the_release_arm_releases_every_replaced_user_visible_unit_under_a_snapshot() {
    let mut pool = build_pool("snapshot-lifetime-judgement-forced-release");
    let created = create_snapshot_and_advance(&mut pool);
    let (judgement_reported, overwritten) = overwrite_with_the_judgement_and_advance(
        &mut pool,
        SnapshotLifetimeJudgement::ForcedToReleaseIntoTheDeferQueueForTests,
    );
    assert_eq!(
        judgement_reported,
        SnapshotLifetimeJudgement::ForcedToReleaseIntoTheDeferQueueForTests,
        "写入口报出这次走的是强制释放那一臂"
    );
    let referenced_by_the_snapshot = user_visible_mapping_keys(&created);
    assert_eq!(
        referenced_by_the_snapshot.len(),
        4,
        "第一个文件一个数据单元：数据单元、extent 根、inode 叶容器、inode 根"
    );
    let released = released_slots(&overwritten);
    for (identity, _) in &referenced_by_the_snapshot {
        assert!(
            released.contains(&created.unit(*identity).slot),
            "{identity:?}：被快照引用，强制释放那一臂照样放进这次的释放"
        );
    }
    assert!(
        overwritten.deadlist_entries.is_empty(),
        "deadlist 一条不写：{:?}",
        overwritten.deadlist_entries
    );
    assert!(
        overwritten
            .mapping_entries_kept_for_the_snapshots
            .is_empty(),
        "没有留存的映射条目"
    );
}

/// 比快照新的单元强制进 deadlist：建快照、照判定覆盖写一次（第一个文件的四个用户可见单元生在快照之前，照判定进 deadlist），
/// 再装上强制那一臂覆盖写一次：这一次换下的四个用户可见单元生在快照之后（判定会释放它们），强制那一臂让它们进 deadlist——
/// 头 = 活头 inode 树、纪元 = 活头此刻的 `previous_snapshot_txg`（建快照那次的 txg，非 0，判决
/// `research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J4）、映射 key 照上一版；不进这次的释放、两块盘上的记录仍在册未释放；
/// deadlist 与留存的映射条目在上一版那四条之外各多四条。
#[test]
fn forcing_the_deadlist_arm_keeps_every_replaced_user_visible_unit_born_after_the_snapshot() {
    let mut pool = build_pool("snapshot-lifetime-judgement-forced-deadlist");
    let created = create_snapshot_and_advance(&mut pool);
    let (_, first) = overwrite_with_the_judgement_and_advance(
        &mut pool,
        SnapshotLifetimeJudgement::FollowTheJudgement,
    );
    let epoch_of_the_snapshot = first.previous_snapshot_txg_of_the_live_head();
    assert_eq!(
        epoch_of_the_snapshot, created.root.checkpoint_txg,
        "活头的 previous_snapshot_txg 是建快照那次发布的 txg"
    );
    assert_ne!(
        epoch_of_the_snapshot,
        CheckpointTxg(0),
        "打过快照：纪元非 0"
    );
    assert_eq!(
        first.deadlist_entries.len(),
        4,
        "照判定那次覆盖写：第一个文件的四个用户可见单元生在快照之前，进 deadlist"
    );
    let (judgement_reported, overwritten) = overwrite_with_the_judgement_and_advance(
        &mut pool,
        SnapshotLifetimeJudgement::ForcedOntoTheDeadlistForTests,
    );
    assert_eq!(
        judgement_reported,
        SnapshotLifetimeJudgement::ForcedOntoTheDeadlistForTests,
        "写入口报出这次走的是强制进 deadlist 那一臂"
    );
    let replaced = user_visible_mapping_keys(&first);
    assert_eq!(replaced.len(), 4, "上一次覆盖写写下的四个用户可见单元");
    for (identity, key) in &replaced {
        assert!(
            birth_txg_of_a_mapping_key(key).is_some_and(|birth| birth > epoch_of_the_snapshot),
            "{identity:?}：生在快照之后（判定会释放它）"
        );
    }
    let mut expected_entries: BTreeSet<Vec<u8>> = first
        .deadlist_entries
        .iter()
        .map(DeadlistEntry::key_bytes)
        .collect();
    expected_entries.extend(replaced.iter().map(|(_, key)| {
        DeadlistEntry {
            head: first.tree_identifiers.inode,
            epoch: epoch_of_the_snapshot,
            mapping_key: key.clone(),
        }
        .key_bytes()
    }));
    let written_entries: BTreeSet<Vec<u8>> = overwritten
        .deadlist_entries
        .iter()
        .map(DeadlistEntry::key_bytes)
        .collect();
    assert_eq!(
        written_entries, expected_entries,
        "上一版那四条照留，四个换下的单元进 deadlist：头 = 活头 inode 树、纪元 = 建快照那次的 txg、映射 key 照上一版"
    );
    let released = released_slots(&overwritten);
    for (identity, _) in &replaced {
        let slot = first.unit(*identity).slot;
        assert!(
            !released.contains(&slot),
            "{identity:?}：强制进 deadlist，不进这次的释放"
        );
        assert!(
            allocated_on_every_device(&pool, slot),
            "{identity:?}：两块盘上的记录仍在册未释放"
        );
    }
    assert_eq!(
        overwritten.mapping_entries_kept_for_the_snapshots.len(),
        first.mapping_entries_kept_for_the_snapshots.len() + 4,
        "映射条目一条 deadlist 条目留存一条"
    );
}

/// 活头没打过快照时强制进 deadlist（判决 `research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J4）：换下的四个用户可见单元的
/// 纪元会是 0，纪元 0 的条目哪个快照的销毁都放不掉，发布断言停下，消息点名纪元；停在任何写之前，录制流一步都没多。
#[test]
fn forcing_the_deadlist_arm_without_any_snapshot_stops_before_any_write_instead_of_an_entry_of_epoch_zero(
) {
    let mut pool = build_pool("snapshot-lifetime-judgement-forced-deadlist-without-a-snapshot");
    let operations_before = pool.stream.operations().len();
    let deadlist_entries_before = pool.output.deadlist_entries.clone();
    let stopped = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        overwrite_with_the_judgement_and_advance(
            &mut pool,
            SnapshotLifetimeJudgement::ForcedOntoTheDeadlistForTests,
        )
    }));
    let payload = stopped.expect_err("没打过快照时强制进 deadlist：发布停下，不写纪元 0 的条目");
    let message = payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| {
            payload
                .downcast_ref::<&str>()
                .map(|text| (*text).to_string())
        })
        .unwrap_or_default();
    assert!(
        message.contains("进 deadlist 的条目纪元非 0"),
        "停下的是纪元那一条断言：{message}"
    );
    assert_eq!(
        pool.stream.operations().len(),
        operations_before,
        "停在任何写之前：录制流一步都没多"
    );
    assert_eq!(
        pool.output.deadlist_entries, deadlist_entries_before,
        "池停在覆盖写之前那一版"
    );
}
