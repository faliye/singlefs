//! 建只读快照那次发布（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1、S2 与第七节岔路 1；
//! `transaction::publish_create_snapshot`、会话的 `UserChange::CreateSnapshot`）与快照条目随之后每一次发布续上
//! （判决 `research/prompts/m4-batch2-code-r1-main-verification.md` 第二节 S2）。
//!
//! 建快照是一次独立发布：S.txg = 这次的 checkpoint_txg；树号从上一版的树 ID 水位起连发两个（码 14、码 15，水位 + 2）；码 14 的根指针 =
//! 活头 inode 树当时的根、头 ID = 活头 inode 树的树号、诞生 txg = S.txg、`previous_snapshot_txg` = 捕获那一刻活头的值；码 15 的根指针 =
//! 活头 extent 树当时的根、头 ID = 配对那条码 14 的树号、`previous_snapshot_txg` 0；活头 inode 条目的 `previous_snapshot_txg` 换成 S.txg
//! （D5（快照 / 空间记账机制） 已定项 9、已定项 14；D8（核心索引结构） 已定项 8 ①②）。第 11 个快照在任何写之前拒。
//! 期望值按条款从建快照之前那一版现算（水位、txg、活头两棵树的根），不从被测的输出反推；读回一律从盘上重建（录制流施加到内存盘上）。
//!
//! 这里的用例只钉条目；被快照引用的块进 deadlist、不释放、映射条目留存，与删快照，钉在
//! `snapshot_release_and_destroy_through_the_deadlist.rs`。
mod common;

use common::{build_pool, disk_snapshot, parameters, publish_overwrite_in_process, BuiltPool};
use singlefs_core::address::{CheckpointTxg, InodeNumber, InstanceGeneration, TreeIdentifier};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::records::{
    TreeTableEntry, SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION, TREE_KIND_EXTENT, TREE_KIND_INODE,
    TREE_KIND_SNAPSHOT_EXTENT_ROOT, TREE_KIND_SNAPSHOT_INODE_ROOT,
};
use singlefs_core::recovery::{
    read_the_first_file_of_a_snapshot, rebuild_version_with_its_snapshot_entries, RebuiltVersion,
    SnapshotFileReading, SnapshotTreeTableEntries,
};
use singlefs_core::transaction::{
    publish_create_snapshot, FirstFile, PoolVersion, PoolWriter, PublishError,
    SnapshotCreationRefusal, TransactionOutput, FIRST_INODE_NUMBER,
};

const WRITE_TIME_SECONDS_OF_THE_OVERWRITE: u64 = 1_788_000_060;

/// 覆盖写的内容：与第一个文件（`common::file_content`）不同的一份。
fn overwrite_content(seed: u32) -> Vec<u8> {
    (0..2500_u32)
        .map(|index| u8::try_from((index * 11 + seed) % 241).expect("小于 256"))
        .collect()
}

/// 在同一个进程里接着 `pool.output` 建一个快照（实例 `instance`），错误原样交回。
fn create_snapshot_in_process(
    pool: &mut BuiltPool,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    publish_create_snapshot(&mut writer, &mut pool.allocator, &previous, instance)
}

/// 建一个快照、池接到它那一版上。
fn create_snapshot_and_advance(pool: &mut BuiltPool) {
    pool.output = create_snapshot_in_process(pool, InstanceGeneration(1)).expect("建快照");
}

fn entry_of_kind(entries: &[TreeTableEntry], kind: u16) -> TreeTableEntry {
    *entries
        .iter()
        .find(|entry| entry.kind == kind)
        .expect("这种树在树表里恰好一条")
}

/// 从盘上（录制流施加到内存盘上）按 `version` 的根重建那一版：交回那一版与它树表里配成对的快照条目。
fn rebuilt_from_disk(
    pool: &BuiltPool,
    version: &TransactionOutput,
) -> (TransactionOutput, Vec<SnapshotTreeTableEntries>) {
    let rebuilt = rebuild_version_with_its_snapshot_entries(
        &pool.memory_pool(),
        &version.root,
        Some(version.record.clone()),
    )
    .expect("盘上这一版重建得出");
    match rebuilt.version {
        RebuiltVersion::WithFile(output) => (output, rebuilt.snapshot_entries),
        RebuiltVersion::WithoutFile | RebuiltVersion::WithTreesAndNoFile(_) => {
            panic!("建快照与覆盖写不动文件：这一版带文件")
        }
    }
}

/// 第 `position` 个快照（从 0 数）按条款该有的那一对条目：它建在 `before` 那一版之后，`before` 那一版的活头两棵树、水位与 txg 是输入。
fn snapshot_pair_expected_after(
    before: &TransactionOutput,
    previous_snapshot_txg_of_the_live_head_before: CheckpointTxg,
) -> SnapshotTreeTableEntries {
    let snapshot_txg = CheckpointTxg(before.root.checkpoint_txg.0 + 1);
    let watermark = before.root.tree_identifier_watermark;
    let live_inode_tree = entry_of_kind(&before.tree_table_entries, TREE_KIND_INODE);
    let live_extent_tree = entry_of_kind(&before.tree_table_entries, TREE_KIND_EXTENT);
    SnapshotTreeTableEntries {
        inode_root: TreeTableEntry {
            kind: TREE_KIND_SNAPSHOT_INODE_ROOT,
            tree: TreeIdentifier(watermark),
            root: live_inode_tree.root,
            previous_snapshot_txg: previous_snapshot_txg_of_the_live_head_before,
            birth_txg: snapshot_txg,
            head_identifier: live_inode_tree.tree.0,
        },
        extent_root: TreeTableEntry {
            kind: TREE_KIND_SNAPSHOT_EXTENT_ROOT,
            tree: TreeIdentifier(watermark + 1),
            root: live_extent_tree.root,
            previous_snapshot_txg: CheckpointTxg(0),
            birth_txg: snapshot_txg,
            head_identifier: watermark,
        },
    }
}

/// 第一个文件之后建一个快照：盘上那一版的树表在十二条之后恰好多一对，两条逐字段是条款给的值（S.txg = 这次的 txg、树号是上一版的水位与它加一、
/// 根指针抄活头两棵树、头 ID 与前驱照判决 S1）；活头 inode 条目的 `previous_snapshot_txg` 换成 S.txg，其余十一条仍 0；根记录的树 ID 水位加二；
/// 活头 inode 树、extent 树的根不动（文件内容不动）。
#[test]
fn creating_a_snapshot_writes_one_pair_of_entries_whose_fields_follow_the_judgement() {
    let mut pool = build_pool("snapshot-creation-one-pair");
    let before = pool.output.clone();
    create_snapshot_and_advance(&mut pool);
    let created = pool.output.clone();
    let snapshot_txg = CheckpointTxg(before.root.checkpoint_txg.0 + 1);
    assert_eq!(
        created.root.checkpoint_txg, snapshot_txg,
        "建快照是接在上一版后面的一次发布"
    );
    let (on_disk, snapshot_entries) = rebuilt_from_disk(&pool, &created);
    assert_eq!(
        snapshot_entries,
        vec![snapshot_pair_expected_after(&before, CheckpointTxg(0))],
        "树表里恰好一对快照条目，逐字段是条款给的值"
    );
    assert_eq!(
        on_disk.tree_table_entries.len(),
        before.tree_table_entries.len() + 2,
        "十二条之后多一对"
    );
    let live_inode_tree = entry_of_kind(&on_disk.tree_table_entries, TREE_KIND_INODE);
    assert_eq!(
        live_inode_tree.previous_snapshot_txg, snapshot_txg,
        "活头 inode 条目的 previous_snapshot_txg 换成 S.txg"
    );
    for entry in &on_disk.tree_table_entries {
        if entry.kind != TREE_KIND_INODE && entry.kind != TREE_KIND_SNAPSHOT_INODE_ROOT {
            assert_eq!(
                entry.previous_snapshot_txg,
                CheckpointTxg(0),
                "树 {:?}（种类 {}）不是活头 inode 条目、也不是码 14：previous_snapshot_txg 写 0",
                entry.tree,
                entry.kind
            );
        }
    }
    assert_eq!(
        on_disk.root.tree_identifier_watermark,
        before.root.tree_identifier_watermark + 2,
        "从水位连发两个号：水位加二"
    );
    for kind in [TREE_KIND_INODE, TREE_KIND_EXTENT] {
        assert_eq!(
            entry_of_kind(&on_disk.tree_table_entries, kind).root,
            entry_of_kind(&before.tree_table_entries, kind).root,
            "种类 {kind}：建快照不动活头的树"
        );
    }
    assert_eq!(
        created.snapshot_entries(),
        snapshot_entries,
        "发布交回的这一版与盘上读回的快照条目相同"
    );
}

/// 连建十个快照都做成：盘上最新那一版十对条目，第 k 个的树号是建它之前那一版的水位（第一个快照之前的水位加 2k），诞生 txg 一个比一个大一，
/// 码 14 的前驱是前一个快照的诞生 txg（第一个是 0），活头 inode 条目的前驱是第十个的诞生 txg。第十一个拒成
/// `SnapshotCreationRefused(MoreSnapshotsThanTheFirstVersionAllows { snapshots: 10 })`，在任何写之前：两盘的系统配置槽、根环里的根与录制流步数逐项不变。
#[test]
fn ten_snapshots_are_created_and_the_eleventh_is_refused_before_any_write() {
    let mut pool = build_pool("snapshot-creation-eleventh-refused");
    let mut expected_pairs = Vec::new();
    for _snapshot in 0..SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION {
        let before = pool.output.clone();
        let previous_snapshot_txg_of_the_live_head =
            entry_of_kind(&before.tree_table_entries, TREE_KIND_INODE).previous_snapshot_txg;
        expected_pairs.push(snapshot_pair_expected_after(
            &before,
            previous_snapshot_txg_of_the_live_head,
        ));
        create_snapshot_and_advance(&mut pool);
    }
    let tenth = pool.output.clone();
    let (on_disk, snapshot_entries) = rebuilt_from_disk(&pool, &tenth);
    assert_eq!(
        snapshot_entries, expected_pairs,
        "十对条目逐字段是条款给的值（树号、诞生 txg、前驱链）"
    );
    assert_eq!(
        entry_of_kind(&on_disk.tree_table_entries, TREE_KIND_INODE).previous_snapshot_txg,
        tenth.root.checkpoint_txg,
        "活头 inode 条目的前驱是最后一个快照的诞生 txg"
    );

    let disk_before_the_eleventh = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let eleventh = create_snapshot_in_process(&mut pool, InstanceGeneration(1));
    assert!(
        matches!(
            eleventh,
            Err(PublishError::SnapshotCreationRefused(
                SnapshotCreationRefusal::MoreSnapshotsThanTheFirstVersionAllows { snapshots: 10 }
            ))
        ),
        "第十一个拒：{eleventh:?}"
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        disk_before_the_eleventh,
        "第十一个在任何写之前拒：盘上逐项不变"
    );
}

/// 建快照之后覆盖写：盘上最新那一版活头 inode 树、extent 树的根换了，快照那一对条目逐字段不变（根指针仍是建快照那一刻活头的根），
/// 活头 inode 条目的前驱仍是 S.txg。被快照钉住的块进 deadlist 那一半钉在 `snapshot_release_and_destroy_through_the_deadlist.rs`。
#[test]
fn an_overwrite_after_a_snapshot_moves_the_live_roots_and_keeps_the_snapshot_entries() {
    let mut pool = build_pool("snapshot-creation-overwrite-after");
    create_snapshot_and_advance(&mut pool);
    let created = pool.output.clone();
    let (_, entries_at_creation) = rebuilt_from_disk(&pool, &created);
    let content = overwrite_content(5);
    pool.output = publish_overwrite_in_process(
        &mut pool,
        &created,
        &content,
        WRITE_TIME_SECONDS_OF_THE_OVERWRITE,
        InstanceGeneration(1),
    )
    .expect("快照之后覆盖写");
    let overwritten = pool.output.clone();
    let (on_disk, snapshot_entries) = rebuilt_from_disk(&pool, &overwritten);
    assert_eq!(
        snapshot_entries, entries_at_creation,
        "快照条目随覆盖写那次发布原样续上"
    );
    let snapshot = entries_at_creation[0];
    assert_ne!(
        entry_of_kind(&on_disk.tree_table_entries, TREE_KIND_INODE).root,
        snapshot.inode_root.root,
        "活头 inode 树的根换了"
    );
    assert_ne!(
        entry_of_kind(&on_disk.tree_table_entries, TREE_KIND_EXTENT).root,
        snapshot.extent_root.root,
        "活头 extent 树的根换了"
    );
    assert_eq!(
        entry_of_kind(&on_disk.tree_table_entries, TREE_KIND_INODE).previous_snapshot_txg,
        snapshot.inode_root.birth_txg,
        "活头 inode 条目的前驱仍是那个快照的 S.txg"
    );
}

/// 两个快照的池进程退出、可写挂载（挂载自己写行与暖机），再经会话覆盖写一次：盘上最新那一版仍是那两对条目，逐字段与挂载之前相同
/// （判决 `research/prompts/m4-batch2-code-r1-main-verification.md` 第二节 S2：写路径从重建出来的那一版把快照条目续上）。
#[test]
fn a_pool_with_two_snapshots_keeps_both_pairs_through_a_writable_mount_and_a_publish() {
    let mut pool = build_pool("snapshot-creation-two-pairs-across-a-mount");
    create_snapshot_and_advance(&mut pool);
    create_snapshot_and_advance(&mut pool);
    let before_the_mount = pool.output.clone();
    let (_, entries_before_the_mount) = rebuilt_from_disk(&pool, &before_the_mount);
    assert_eq!(entries_before_the_mount.len(), 2, "挂载之前两对");
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("两个快照的池可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let content = overwrite_content(9);
    session
        .publish_user_change(
            &mut devices,
            UserChange::Overwrite(FirstFile {
                content: &content,
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_OVERWRITE,
            }),
        )
        .expect("挂着之后覆盖写");
    pool.devices = Some(devices);
    let newest = match &session.current {
        PoolVersion::WithFile(version) => version.clone(),
        PoolVersion::WithoutFile(_) | PoolVersion::WithTreesAndNoFile(_) => {
            panic!("覆盖写之后现行那一版带文件")
        }
    };
    assert!(
        newest.root.checkpoint_txg > before_the_mount.root.checkpoint_txg,
        "挂载与覆盖写都接在两个快照之后发"
    );
    let (on_disk, snapshot_entries) = rebuilt_from_disk(&pool, &newest);
    assert_eq!(
        snapshot_entries, entries_before_the_mount,
        "挂载与之后的发布把两对条目逐字段原样续上"
    );
    assert_eq!(
        entry_of_kind(&on_disk.tree_table_entries, TREE_KIND_INODE).previous_snapshot_txg,
        entries_before_the_mount[1].inode_root.birth_txg,
        "活头 inode 条目的前驱仍是第二个快照的诞生 txg"
    );
}

/// 经挂着的会话建快照（`UserChange::CreateSnapshot`）：一次调用只发一版（独立发布，不与别的改动合在一起），那一版的快照条目是一对、
/// 诞生 txg 是那一版自己的 txg；文件内容不动（数据指针与挂载之后那一版相同）。
#[test]
fn creating_a_snapshot_through_the_mounted_session_publishes_one_version_of_its_own() {
    let mut pool = build_pool("snapshot-creation-through-the-session");
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let after_the_mount = session
        .current
        .version_of_trees()
        .expect("第一个文件之后挂载：有树的一版")
        .clone();
    let published = session
        .publish_user_change(&mut devices, UserChange::CreateSnapshot)
        .expect("经会话建快照");
    pool.devices = Some(devices);
    assert_eq!(
        published.versions_published.len(),
        1,
        "建快照一次调用只发一版"
    );
    let created = published.versions_published[0].clone();
    let (on_disk, snapshot_entries) = rebuilt_from_disk(&pool, &created);
    assert_eq!(
        snapshot_entries,
        vec![snapshot_pair_expected_after(
            &after_the_mount,
            CheckpointTxg(0)
        )],
        "会话发的那一版恰好一对，逐字段是条款给的值"
    );
    assert_eq!(
        on_disk.data_pointers, after_the_mount.data_pointers,
        "文件内容不动：数据指针与挂载之后那一版相同"
    );
}

/// 文件集合为空时建快照放行（用户 2026-10-16 定「放开」，判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F5）：
/// 经会话删掉第一个文件之后（有树而文件集合为空的一版）建快照照常发成一版、树表里多一对条目；之后再建一个 inode，读快照读到的是建它那一刻——
/// 没有文件；删快照照常删掉。
#[test]
fn a_snapshot_created_after_every_file_is_deleted_reads_no_file_and_is_destroyed() {
    let mut pool = build_pool("snapshot-creation-on-the-emptied-version");
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    session
        .publish_user_change(
            &mut devices,
            UserChange::Unlink {
                inode: InodeNumber(FIRST_INODE_NUMBER),
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_OVERWRITE,
            },
        )
        .expect("删第一个文件");
    assert!(
        matches!(session.current, PoolVersion::WithTreesAndNoFile(_)),
        "删完第一个文件，现行那一版有树而文件集合为空"
    );
    let created = session
        .publish_user_change(&mut devices, UserChange::CreateSnapshot)
        .expect("文件集合为空时建快照照常发布");
    assert_eq!(
        created.versions_published.len(),
        1,
        "建快照一次调用只发一版"
    );
    let snapshot = created.versions_published[0]
        .tree_table_entries
        .iter()
        .find(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .expect("建快照那一版树表里有码 14 那一条")
        .tree;
    session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_OVERWRITE + 60,
            },
        )
        .expect("建快照之后再建一个 inode");
    let root_after_the_new_inode = *session.current.root();
    pool.devices = Some(devices);
    assert_eq!(
        read_the_first_file_of_a_snapshot(&pool.memory_pool(), &root_after_the_new_inode, snapshot)
            .expect("读快照"),
        SnapshotFileReading::NoFile,
        "快照抄的是文件集合为空那一刻：读到没有文件"
    );
    let mut devices_after_reading_the_snapshot = pool.devices.take().expect("镜像还开着");
    session
        .publish_user_change(
            &mut devices_after_reading_the_snapshot,
            UserChange::DestroySnapshot { snapshot },
        )
        .expect("删快照");
    pool.devices = Some(devices_after_reading_the_snapshot);
    assert!(
        !session
            .current
            .version_of_trees()
            .expect("有树的一版")
            .tree_table_entries
            .iter()
            .any(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT),
        "删完之后树表里没有快照"
    );
}
