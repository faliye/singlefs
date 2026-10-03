//! unlink 之后的那几种版本与续做（D8（核心索引结构） 已定项 16 第 5、6、7 条；D23（journal 的角色与格式） 已定项 14）：
//! 删空的一版（有树、文件集合为空）是 `PoolVersion` / `RebuiltVersion` / `PreviousVersion` 的第三个成员；可写挂载在写行与暖机之后
//! 把队列走到空；删除那两个只供测试的开关经可写挂载的开关链进会话；回退新根的 `deleted_inodes` 取 R_old 的、下一次挂载续做走空它。
//! 三个单元的第一个文件、只供测试的批大小 1：意图 → 留 2 → 留 1 → 最后一批。

mod common;

use common::{
    build_pool, crash_state_devices, disk_snapshot, memory_pool_of_sparse_devices, parameters,
    sequential_write_in_process_as_instance_one_and_advance_the_pool, BuiltPool,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceOffsetInBytes, InodeNumber, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, roll_back_by_a_forward_publish,
    Mounted, RollbackTarget, ShadowLedger,
};
use singlefs_core::mounted_read::{mount_read_only, OpenFileFailure};
use singlefs_core::mounted_session::{MountedSession, UserChange, UserChangeRefused};
use singlefs_core::records::{
    InodeRecord, STATISTIC_COMMITTED_RESERVATION_BYTES, TREE_KIND_DELETED_INODES,
};
use singlefs_core::recovery::{rebuild_version, RebuiltVersion};
use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
use singlefs_core::transaction::{
    publish_new_inodes, publish_overwrite, publish_unlink_intent, publish_unlink_step, FirstFile,
    PoolVersion, PoolWriter, TransactionOutput, TransactionUnit, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::{build_index_node, data_unit_payload_capacity, parse_index_node};
use singlefs_core::unlink::{
    DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize,
};
use singlefs_format::{INODE_RECORD_BYTES, PACKED_UNIT_RECORDS_OFFSET, SLOT_BYTES};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::SharedStream;

const FIRST_FILE: InodeNumber = InodeNumber(FIRST_INODE_NUMBER);

/// 进程内那几次删除发布都在新池新建文件那个实例里发。
const INSTANCE_OF_THE_PROCESS: InstanceGeneration = InstanceGeneration(1);

fn batches_of_one_data_unit() -> UnlinkBatchSize {
    UnlinkBatchSize::CappedForTests(
        UnlinkBatchDataUnitsCappedForTests::new(1).expect("1 在 1..=144 里"),
    )
}

fn deletion_switches_with_batches_of_one() -> DeletionSwitches {
    DeletionSwitches {
        batch_size: batches_of_one_data_unit(),
        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

/// 新池新建文件之后把第一个文件顺序写成三个数据单元（最后一个单元装半个净荷）。
fn pool_with_a_three_unit_file(tag: &str) -> BuiltPool {
    let payload_capacity = data_unit_payload_capacity();
    let content: Vec<u8> = (0..2 * payload_capacity + payload_capacity / 2)
        .map(|index| u8::try_from((index * 13 + 5) % 251).expect("小于 256"))
        .collect();
    let mut pool = build_pool(tag);
    let grown =
        sequential_write_in_process_as_instance_one_and_advance_the_pool(&mut pool, &content);
    assert_eq!(grown.data_pointers.len(), 3, "第一个文件三个数据单元");
    pool
}

/// 第一个文件那条记录现有的写入时间（池的当前版是进程内最后发的那一版）：unlink 的意图照传它，盘上字节与 Q-F 乙之前相同。
fn write_time_seconds_of_the_first_file(pool: &BuiltPool) -> u64 {
    pool.output
        .inode_record
        .expect("这一刻第一个文件那条记录在")
        .write_time_seconds
}

/// 在这个进程里发 unlink 的意图，带第一个文件那条记录现有的写入时间。
fn publish_the_intent_in_process(pool: &mut BuiltPool) -> TransactionOutput {
    let write_time_seconds = write_time_seconds_of_the_first_file(pool);
    publish_the_intent_in_process_at(pool, write_time_seconds)
}

/// 在这个进程里发 unlink 的意图，带调用方给的写入时间（用户 2026-10-03 定 Q-F 乙：意图带时间、三格同改）。
fn publish_the_intent_in_process_at(
    pool: &mut BuiltPool,
    write_time_seconds: u64,
) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let output = publish_unlink_intent(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FIRST_FILE,
        write_time_seconds,
        INSTANCE_OF_THE_PROCESS,
    )
    .expect("意图");
    pool.output = output.clone();
    output
}

fn publish_the_next_step_in_process(pool: &mut BuiltPool) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let output = publish_unlink_step(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FIRST_FILE,
        batches_of_one_data_unit(),
        INSTANCE_OF_THE_PROCESS,
    )
    .expect("删除的下一步");
    pool.output = output.clone();
    output
}

/// 重开镜像、按批大小 1 的删除开关可写挂载一次（空间准入照判、影子账开）。
fn remount_with_batches_of_one(
    pool: &mut BuiltPool,
) -> (
    Mounted,
    Vec<(singlefs_core::address::DeviceIdentity, common::Recorded)>,
) {
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
        &parameters(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        deletion_switches_with_batches_of_one(),
    )
    .expect("重开可写挂载");
    (mounted, devices)
}

/// 这次挂载自己发的最后一次（写行或暖机）那条根的 txg：续做的发布排在它后面。
fn txg_of_the_last_row_or_warm_up_publish(mounted: &Mounted) -> u64 {
    mounted
        .output
        .warm_up_publishes
        .last()
        .unwrap_or(&mounted.output.row_publish)
        .root()
        .checkpoint_txg
        .0
}

fn violations(image: &MemoryPool) -> Vec<String> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some(format!("{invariant}: {detail}")),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
        })
        .collect()
}

fn member_name(version: &PoolVersion) -> &'static str {
    match version {
        PoolVersion::WithoutFile(_) => "WithoutFile",
        PoolVersion::WithFile(_) => "WithFile",
        PoolVersion::WithTreesAndNoFile(_) => "WithTreesAndNoFile",
    }
}

fn rebuilt_member_name(version: &RebuiltVersion) -> &'static str {
    match version {
        RebuiltVersion::WithoutFile => "WithoutFile",
        RebuiltVersion::WithFile(_) => "WithFile",
        RebuiltVersion::WithTreesAndNoFile(_) => "WithTreesAndNoFile",
    }
}

#[test]
fn only_the_version_after_the_final_batch_rebuilds_as_with_trees_and_no_file() {
    let mut pool = pool_with_a_three_unit_file("emptied-version-rebuilt-member");
    let mut published = vec![
        pool.output.clone(),
        publish_the_intent_in_process(&mut pool),
    ];
    for _ in 0..3 {
        published.push(publish_the_next_step_in_process(&mut pool));
    }
    let image = pool.memory_pool();
    let members: Vec<&str> = published
        .iter()
        .map(|version| {
            let rebuilt = rebuild_version(&image, &version.root, Some(version.record.clone()))
                .unwrap_or_else(|failure| panic!("从盘上重建：{failure:?}"));
            rebuilt_member_name(&rebuilt)
        })
        .collect();
    assert_eq!(
        members,
        vec!["WithFile", "WithFile", "WithFile", "WithFile", "WithTreesAndNoFile"],
        "删除之前、意图、两批之后都带文件（inode 树有那条 nlink = 0 的记录、队列有 key）；最后一批之后 inode 树与 extent 树零条目根、\
         deleted_inodes 根指针置零，是有树而文件集合为空的一版"
    );
}

#[test]
fn a_session_mounted_with_batches_of_one_unlinks_a_three_unit_file_in_an_intent_and_three_steps() {
    let mut pool = pool_with_a_three_unit_file("session-takes-the-mount-deletion-switches");
    let (mounted, mut devices) = remount_with_batches_of_one(&mut pool);
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let txg_before = session.current.root().checkpoint_txg.0;
    session
        .publish_user_change(
            &mut devices,
            UserChange::Unlink {
                inode: FIRST_FILE,
                write_time_seconds: write_time_seconds_of_the_first_file(&pool),
            },
        )
        .expect("会话 unlink");
    assert_eq!(
        (
            session.allocator.deletion_switches(),
            session.current.root().checkpoint_txg.0 - txg_before,
        ),
        (deletion_switches_with_batches_of_one(), 4),
        "删除开关从可写挂载的开关链装在分配器上、会话照它：意图一次 + 留 2、留 1 两批 + 最后一批"
    );
}

/// 会话交回 unlink 那一次调用发成的每一版（`UserChangePublished::versions_published`，模型对拍逐次比它们）：意图那次与三步，按 txg 连着、
/// 最后一个就是会话的现行那一版；意图那一版号在 `deleted_inodes` 里、三个数据单元都在，之后各步剩 2、1、0 个，最后一步删了记录。
/// 判别力：意图那次不记进去时交回的只有三步（模型拿第一步当意图比，随机历史那一路判对不上）。
#[test]
fn the_session_hands_in_the_intent_and_every_step_of_an_unlink_in_publish_order() {
    let mut pool = pool_with_a_three_unit_file("session-hands-in-every-version-of-the-unlink");
    let (mounted, mut devices) = remount_with_batches_of_one(&mut pool);
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let txg_before = session.current.root().checkpoint_txg.0;
    let published = session
        .publish_user_change(
            &mut devices,
            UserChange::Unlink {
                inode: FIRST_FILE,
                write_time_seconds: write_time_seconds_of_the_first_file(&pool),
            },
        )
        .expect("会话 unlink");
    assert_eq!(
        published
            .versions_published
            .iter()
            .map(|version| (
                version.root.checkpoint_txg.0 - txg_before,
                version.deleted_inodes.clone(),
                version.data_pointers.len(),
                version.inode_record.is_some(),
            ))
            .collect::<Vec<_>>(),
        vec![
            (1, vec![FIRST_FILE], 3, true),
            (2, vec![FIRST_FILE], 2, true),
            (3, vec![FIRST_FILE], 1, true),
            (4, Vec::new(), 0, false),
        ],
        "意图那次、留 2、留 1、最后一批，按 txg 连着"
    );
    assert_eq!(
        published
            .versions_published
            .last()
            .map(|version| version.root),
        Some(*session.current.root()),
        "最后交回的那一版就是会话的现行那一版"
    );
}

/// 挂载交回续做发成的每一步（`MountOutput::deletion_queue_continuation`，随机历史接着拿模型的「走空的一步」逐次比）：
/// 意图落盘之后重开，续做三步接在写行与暖机之后、按 txg 连着，最后一步就是挂载交回的现行那一版。
/// 判别力：续做照走而交回空表时这一条红（随机历史那一路的模型少比三条根、下一步对不上）。
#[test]
fn the_mount_hands_in_every_continuation_step_after_its_warm_up_in_publish_order() {
    let mut pool = pool_with_a_three_unit_file("mount-hands-in-every-continuation-step");
    publish_the_intent_in_process(&mut pool);
    let (mounted, devices) = remount_with_batches_of_one(&mut pool);
    let last_row_or_warm_up_txg = txg_of_the_last_row_or_warm_up_publish(&mounted);
    assert_eq!(
        mounted
            .output
            .deletion_queue_continuation
            .iter()
            .map(|step| (
                step.root.checkpoint_txg.0 - last_row_or_warm_up_txg,
                step.data_pointers.len(),
                step.inode_record.is_some(),
            ))
            .collect::<Vec<_>>(),
        vec![(1, 2, true), (2, 1, true), (3, 0, false)],
        "留 2、留 1、最后一批，接在暖机之后、按 txg 连着"
    );
    assert_eq!(
        mounted
            .output
            .deletion_queue_continuation
            .last()
            .map(|step| step.root),
        Some(*mounted.current.root()),
        "最后一步就是挂载交回的现行那一版"
    );
    drop(devices);
}

#[test]
fn after_the_unlink_the_session_holds_the_emptied_version_and_refuses_an_overwrite_before_any_write(
) {
    let mut pool = pool_with_a_three_unit_file("session-emptied-version-refuses-overwrite");
    let (mounted, mut devices) = remount_with_batches_of_one(&mut pool);
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    session
        .publish_user_change(
            &mut devices,
            UserChange::Unlink {
                inode: FIRST_FILE,
                write_time_seconds: write_time_seconds_of_the_first_file(&pool),
            },
        )
        .expect("会话 unlink");
    assert_eq!(
        member_name(&session.current),
        "WithTreesAndNoFile",
        "删完唯一的文件之后会话的现行那一版是第三个成员"
    );
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let content = vec![7_u8; 100];
    let refusal = session
        .publish_user_change(
            &mut devices,
            UserChange::Overwrite(FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            }),
        )
        .expect_err("删空的一版上只放建 inode");
    assert!(
        matches!(
            &refusal,
            UserChangeRefused::NoFileVersionToChange {
                floor_raises,
                refusals_that_pushed_the_floor_raises,
                versions_published,
            } if floor_raises.is_empty()
                && refusals_that_pushed_the_floor_raises.is_empty()
                && versions_published.is_empty()
        ),
        "拒成没有可改的文件，队列本来就空、没有走空：发成的几版与推的抬 F 都是空的：{refusal:?}"
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "拒在任何写之前：系统配置槽、根环、录制流步数都不变"
    );
}

#[test]
fn new_inodes_on_the_emptied_version_are_published_and_the_version_holds_files_again() {
    let mut pool = pool_with_a_three_unit_file("session-emptied-version-takes-new-inodes");
    let (mounted, mut devices) = remount_with_batches_of_one(&mut pool);
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    session
        .publish_user_change(
            &mut devices,
            UserChange::Unlink {
                inode: FIRST_FILE,
                write_time_seconds: write_time_seconds_of_the_first_file(&pool),
            },
        )
        .expect("会话 unlink");
    session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 2,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
        )
        .expect("删空的一版上建两个 size 0 的 inode");
    let PoolVersion::WithFile(current) = &session.current else {
        panic!(
            "建了 inode 之后 inode 树有条目，归带文件的一版：{}",
            member_name(&session.current)
        );
    };
    assert_eq!(
        (
            current.inode_record,
            current
                .inode_leaf_containers
                .iter()
                .map(|container| container.contents.records.len())
                .sum::<usize>(),
            current.data_pointers.len(),
        ),
        (None, 2, 0),
        "第一个文件那条记录不在、两条 size 0 的记录、没有数据单元"
    );
}

#[test]
fn a_writable_mount_after_the_intent_continues_the_queue_to_empty_after_its_warm_up() {
    let mut pool = pool_with_a_three_unit_file("mount-continues-after-the-intent");
    publish_the_intent_in_process(&mut pool);
    let (mounted, devices) = remount_with_batches_of_one(&mut pool);
    let continuation_publishes =
        mounted.current.root().checkpoint_txg.0 - txg_of_the_last_row_or_warm_up_publish(&mounted);
    assert_eq!(
        (
            member_name(&mounted.current),
            continuation_publishes,
            mounted
                .current
                .version_of_trees()
                .expect("有树")
                .deleted_inodes
                .clone(),
        ),
        ("WithTreesAndNoFile", 3, Vec::new()),
        "写行与暖机之后续做：留 2、留 1 两批 + 最后一批，交回时队列空、是删空的一版"
    );
    drop(devices);
    let image = pool.memory_pool();
    assert_eq!(
        violations(&image),
        Vec::<String>::new(),
        "续做之后池级 checker 全绿"
    );
    let read_only = mount_read_only(&image).expect("只读挂载");
    assert!(
        matches!(
            read_only.mounted.open_file(&image, FIRST_FILE),
            Err(OpenFileFailure::NoSuchInode { inode }) if inode == FIRST_FILE
        ),
        "续做之后只读挂载打开第一个文件答「没有文件」"
    );
}

/// 第一个文件那条 inode 记录在这一版里的样子。
fn record_of_the_first_file(version: &TransactionOutput) -> InodeRecord {
    version
        .inode_leaf_containers
        .iter()
        .flat_map(|container| container.contents.records.iter())
        .find(|record| record.inode == FIRST_FILE.0)
        .copied()
        .expect("意图与中间各批之后第一个文件的记录还在（nlink = 0）")
}

/// 意图那次与中间一批都算一次改动（用户 2026-10-03 定 Q-F「算改动，改实现」；D8（核心索引结构） 已定项 6 偏移 88 改动计数 =
/// 最后一次改动所在发布的 checkpoint_txg）：三个单元的文件发意图、再发一批（留 2），两次之后第一个文件那条记录的改动计数各等于那一次发布的 txg；
/// 写入时间另由 [`the_intent_writes_the_given_time_into_the_three_time_fields_and_a_middle_batch_copies_it`] 判（偏移 64 起 atime、mtime、ctime
/// 三个秒字段共写一个 `write_time_seconds`，`records::InodeRecord::to_bytes`；D8（核心索引结构） 已定项 6 字段表）。
/// 两次之后池级 checker 全绿（I-9.26（inode 改动计数落在出生代与容器诞生代号之间））。
/// 判别力：意图那次或那一批照旧带上一版的改动计数时那一项红。
#[test]
fn the_intent_and_a_middle_batch_each_set_the_change_count_to_the_txg_of_their_own_publish() {
    let mut pool = pool_with_a_three_unit_file("change-count-of-the-deletion-steps");
    let before_the_unlink = record_of_the_first_file(&pool.output);
    let intent = publish_the_intent_in_process(&mut pool);
    let middle_batch = publish_the_next_step_in_process(&mut pool);
    let after_the_intent = record_of_the_first_file(&intent);
    let after_the_middle_batch = record_of_the_first_file(&middle_batch);
    assert!(
        before_the_unlink.change_count < intent.root.checkpoint_txg.0,
        "改动计数本来低于意图那次的 txg：{} 对 {}",
        before_the_unlink.change_count,
        intent.root.checkpoint_txg.0
    );
    assert_eq!(
        (
            after_the_intent.change_count,
            after_the_middle_batch.change_count,
        ),
        (
            intent.root.checkpoint_txg.0,
            middle_batch.root.checkpoint_txg.0,
        ),
        "意图那次与中间那一批各把改动计数设成自己那次发布的 txg"
    );
    assert_eq!(
        violations(&pool.memory_pool()),
        Vec::<String>::new(),
        "两次之后池级 checker 全绿"
    );
}

/// 第一个文件那条记录在 `version` 这一版里、两块盘上各读出来的三个时间秒（偏移 64 起 atime、mtime、ctime 各 8 字节，D8（核心索引结构）
/// 已定项 6 字段表）：按盘读这一版每片 inode 叶容器的单元字节，在码 3 记录区（`PACKED_UNIT_RECORDS_OFFSET` 起、每条 `INODE_RECORD_BYTES`）里
/// 找 inode 号是第一个文件的那一条。读的是盘，不是内存里那份记录。
fn time_seconds_of_the_first_file_on_each_device(
    image: &MemoryPool,
    version: &TransactionOutput,
) -> Vec<[u64; 3]> {
    let record_bytes = usize::try_from(INODE_RECORD_BYTES).expect("140");
    let records_offset = usize::try_from(PACKED_UNIT_RECORDS_OFFSET).expect("136");
    let field_at = |record: &[u8], offset: usize| {
        u64::from_le_bytes(
            record[offset..offset + 8]
                .try_into()
                .expect("切出来的正好 8 字节"),
        )
    };
    image
        .devices
        .values()
        .map(|device| {
            version
                .units
                .iter()
                .filter(|unit| matches!(unit.identity, TransactionUnit::InodeLeafContainer(_)))
                .find_map(|unit| {
                    let unit_bytes = device.read(
                        DeviceOffsetInBytes(unit.slot.0 * SLOT_BYTES),
                        usize::try_from(unit.identity.span_slots() * SLOT_BYTES)
                            .expect("单元字节数"),
                    );
                    unit_bytes[records_offset..]
                        .chunks_exact(record_bytes)
                        .find(|record| field_at(record, 0) == FIRST_FILE.0)
                        .map(|record| {
                            [
                                field_at(record, 64),
                                field_at(record, 72),
                                field_at(record, 80),
                            ]
                        })
                })
                .expect("这一版的 inode 叶容器里有第一个文件那条记录（意图与中间各批之后 nlink = 0、还在树里）")
        })
        .collect()
}

/// 意图那次带调用方给的写入时间（用户 2026-10-03 定 Q-F「乙 意图带时间、三格同改」）：三个单元的文件发意图，给一个与这条记录上一版不同的时间，
/// 再发一批（留 2）。意图那一版两块盘上第一个文件那条记录的三个时间秒都等于给的时间；中间那一批照抄意图那次写进去的值、不另取。
/// 判别力：意图照旧带上一版的时间时第一项红；那一批不照抄上一版这条记录的时间（另取一个）时第二项红。
#[test]
fn the_intent_writes_the_given_time_into_the_three_time_fields_and_a_middle_batch_copies_it() {
    let mut pool = pool_with_a_three_unit_file("write-time-of-the-deletion-steps");
    let time_before_the_unlink = write_time_seconds_of_the_first_file(&pool);
    let time_of_the_intent = time_before_the_unlink + 600;
    let intent = publish_the_intent_in_process_at(&mut pool, time_of_the_intent);
    let on_disk_after_the_intent =
        time_seconds_of_the_first_file_on_each_device(&pool.memory_pool(), &intent);
    let middle_batch = publish_the_next_step_in_process(&mut pool);
    let on_disk_after_the_middle_batch =
        time_seconds_of_the_first_file_on_each_device(&pool.memory_pool(), &middle_batch);
    assert_eq!(
        on_disk_after_the_intent,
        vec![[time_of_the_intent; 3]; 2],
        "意图那一版两块盘上那条记录的 atime、mtime、ctime 秒都是给的时间（上一版是 {time_before_the_unlink}）"
    );
    assert_eq!(
        on_disk_after_the_middle_batch,
        vec![[time_of_the_intent; 3]; 2],
        "中间那一批照抄意图那次写进去的时间"
    );
}

/// 两块盘的池：标签 6 是全部副本之和，每份一个副本。
const REPLICAS_OF_THE_TWO_DEVICE_POOL: u64 = 2;

/// 记账标签 6（墓碑预付）里的 H 从墓碑树根节点的码 2 头现读（代码轮 U8，`research/prompts/m3-unlink-code-r1-main-verification.md` 第三节；
/// D3（空间分配） 已定项 2 字面）：建一个空 inode 再 unlink 它（意图那次开出墓碑树的第一片容器、根层级 1，H = 0），在内存里那一版上把墓碑树根单元
/// 头里的层级改成 3（形状不动），直接在它上面发一次覆盖写（写入类发布，墓碑树照抄）：标签 6 比在原样那一版上发的多 2 × 16384 × 副本数——
/// 按头里的层级算，不按内存里那份形状。
/// 判别力：H 照旧读内存里的形状时两次一样多，那一项红。
#[test]
fn the_tombstone_prepay_of_an_overwrite_reads_the_tombstone_tree_height_from_the_root_node_header()
{
    let mut pool = build_pool("tombstone-prepay-reads-the-root-node-header");
    let second_inode = InodeNumber(FIRST_INODE_NUMBER + 1);
    let with_a_tombstone = {
        let publish_parameters = parameters();
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        let with_a_second_inode = publish_new_inodes(
            &mut writer,
            &mut pool.allocator,
            &previous,
            1,
            FIXED_WRITE_TIME_SECONDS,
            INSTANCE_OF_THE_PROCESS,
        )
        .expect("建一个空 inode");
        let intent = publish_unlink_intent(
            &mut writer,
            &mut pool.allocator,
            &with_a_second_inode,
            second_inode,
            FIXED_WRITE_TIME_SECONDS,
            INSTANCE_OF_THE_PROCESS,
        )
        .expect("unlink 那个空 inode 的意图");
        publish_unlink_step(
            &mut writer,
            &mut pool.allocator,
            &intent,
            second_inode,
            batches_of_one_data_unit(),
            INSTANCE_OF_THE_PROCESS,
        )
        .expect("它没有数据单元：意图之后直接是最后一批")
    };
    pool.output = with_a_tombstone.clone();
    let root_header =
        parse_index_node(&with_a_tombstone.unit(TransactionUnit::TombstoneRoot).bytes)
            .expect("墓碑树根解得开");
    assert_eq!(
        (
            root_header.level,
            with_a_tombstone.tombstone_containers.len()
        ),
        (1, 1),
        "一片容器、根层级 1：H = 0"
    );
    let mut filesystem_identifier = [0_u8; 16];
    filesystem_identifier[..8].copy_from_slice(&root_header.filesystem_identifier.to_le_bytes());
    let root_two_levels_higher = build_index_node(
        root_header.tree,
        root_header.level + 2,
        root_header.key_width,
        &root_header.smallest_key,
        &root_header.largest_key,
        root_header.birth_txg,
        &filesystem_identifier,
        root_header.instance,
        root_header.birth_sequence,
        u16::try_from(root_header.entry_width).expect("条目宽两字节"),
        &root_header.entries,
    );
    let mut with_the_root_header_two_levels_higher = with_a_tombstone.clone();
    with_the_root_header_two_levels_higher
        .units
        .iter_mut()
        .find(|unit| unit.identity == TransactionUnit::TombstoneRoot)
        .expect("这一版的单元里有墓碑树根")
        .bytes = root_two_levels_higher;
    let image = pool.memory_pool();
    let tombstone_prepay_after_an_overwrite_on = |previous: &TransactionOutput| {
        let mut allocator = pool.allocator.clone();
        let mut devices = crash_state_devices(&image, &[], &[], &SharedStream::new());
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        let content = vec![7_u8; 100];
        publish_overwrite(
            &mut writer,
            &mut allocator,
            previous,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
            },
            INSTANCE_OF_THE_PROCESS,
        )
        .expect("覆盖写做成")
        .accounting_entries
        .iter()
        .find(|entry| entry.statistic == STATISTIC_COMMITTED_RESERVATION_BYTES)
        .expect("记账里有标签 6 那一行")
        .value
    };
    assert_eq!(
        tombstone_prepay_after_an_overwrite_on(&with_the_root_header_two_levels_higher)
            - tombstone_prepay_after_an_overwrite_on(&with_a_tombstone),
        2 * SLOT_BYTES * REPLICAS_OF_THE_TWO_DEVICE_POOL,
        "标签 6 按墓碑树根节点头里的层级算：头里高两层，H 多 2"
    );
}

#[test]
fn two_copies_of_the_image_between_two_batches_continue_to_byte_identical_disks() {
    let mut pool = pool_with_a_three_unit_file("two-copies-continue-the-same-batches");
    publish_the_intent_in_process(&mut pool);
    publish_the_next_step_in_process(&mut pool);
    let image_between_two_batches = pool.memory_pool();
    let continue_on_a_copy = || {
        let stream = SharedStream::new();
        let mut devices = crash_state_devices(&image_between_two_batches, &[], &[], &stream);
        let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
            &parameters(),
            &mut devices,
            SpaceAdmission::JudgedByTheFormula,
            ShadowLedger::On,
            deletion_switches_with_batches_of_one(),
        )
        .expect("这一份镜像可写挂载、续做");
        assert_eq!(
            member_name(&mounted.current),
            "WithTreesAndNoFile",
            "续做到底"
        );
        memory_pool_of_sparse_devices(&devices)
    };
    let first_copy = continue_on_a_copy();
    let second_copy = continue_on_a_copy();
    assert!(
        first_copy == second_copy,
        "同一份崩溃镜像各自重挂载续做同一批，两份盘逐字节相同（D8 已定项 16 第 7 条 ①b）"
    );
    assert!(first_copy != image_between_two_batches, "续做真的写了盘");
}

#[test]
fn rolling_back_to_the_intent_takes_its_deleted_inodes_root_and_the_next_mount_continues_it_to_empty(
) {
    let mut pool = pool_with_a_three_unit_file("rollback-to-the-intent-then-continue");
    let intent = publish_the_intent_in_process(&mut pool);
    for _ in 0..3 {
        publish_the_next_step_in_process(&mut pool);
    }
    let deleted_inodes_root_of = |version: &TransactionOutput| {
        version
            .tree_table_entries
            .iter()
            .find(|entry| entry.kind == TREE_KIND_DELETED_INODES)
            .expect("树表里 deleted_inodes 一条")
            .root
    };
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    roll_back_by_a_forward_publish(
        &publish_parameters,
        devices,
        &mut pool.allocator,
        &mut pool.output,
        RollbackTarget {
            instance: intent.root.instance,
            checkpoint_txg: intent.root.checkpoint_txg,
        },
    )
    .expect("回退到意图那一版");
    assert_eq!(
        (
            pool.output.deleted_inodes.clone(),
            deleted_inodes_root_of(&pool.output),
        ),
        (vec![FIRST_FILE], deleted_inodes_root_of(&intent)),
        "回退新根的 deleted_inodes 取 R_old 的：队列带回那个号、根指针是意图那一版的（D23 已定项 14）"
    );
    assert_eq!(
        violations(&pool.memory_pool()),
        Vec::<String>::new(),
        "回退之后池级 checker 全绿"
    );
    let (mounted, remounted_devices) = remount_with_batches_of_one(&mut pool);
    assert_eq!(
        member_name(&mounted.current),
        "WithTreesAndNoFile",
        "下一次可写挂载把带回来的队列续做到空"
    );
    drop(remounted_devices);
    assert_eq!(
        violations(&pool.memory_pool()),
        Vec::<String>::new(),
        "续做之后池级 checker 全绿"
    );
}

/// 一块内存盘外面再包一层：整池数写调用（两块盘共用一个计数），数到 `failing_write` 那一次报块设备错、一个字节都不落盘，别的原样交给内层。
/// 造「续做里的一步在落盘途中失败」那一形（不是空间不够的错，代码轮 U9）。
struct DeviceFailingOneCountedWrite<Inner> {
    inner: Inner,
    writes_so_far: std::rc::Rc<std::cell::Cell<u64>>,
    failing_write: u64,
}

impl<Inner: singlefs_core::block_device::BlockDevice> singlefs_core::block_device::BlockDevice
    for DeviceFailingOneCountedWrite<Inner>
{
    fn read_at(
        &self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.inner.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        bytes: &[u8],
        durability: singlefs_core::block_device::WriteDurability,
    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        let this_write = self.writes_so_far.get() + 1;
        self.writes_so_far.set(this_write);
        if this_write == self.failing_write {
            return Err(singlefs_core::block_device::BlockDeviceError::InputOutput(
                std::io::Error::other("用例注入的写错"),
            ));
        }
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(
        &mut self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> singlefs_core::block_device::PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// 续做里的一步没发成、不是空间不够（块设备错）：挂载交回 `MountError::Publish`，错里的已落盘写账（`writes_of_persisted_publishes`）
/// 一次发布一份、按先后，每一份都有写（代码轮 U9，`PublishSequenceFailed::writes_of_persisted_publishes` 的契约）。份数等于这次挂载落在盘上的
/// 发布数（写行、暖机、续做第一步）只在这条用例挑的注错点上成立：注错点放在续做第二步的第一个写上。注错点落在一次发布的记录写之后
/// （根槽写、轮换的系统配置写）时，只读恢复靠那条记录多走一次，份数比只读挂载看到的少 1（攻方 `m3-unlink-code-r2` H4，代码轮 V4），这条用例不判那几点。
/// 造法：三个单元的文件发完意图就崩（续做三步），同一份镜像先在一份拷贝上挂一次数清写调用，再在另一份上让续做第二步的第一次写报错。
/// 判别力：续做那一处的错照旧交空的已落盘写账时，份数是 0、盘上落了 1 + 暖机 + 1 次，那一项红。
#[test]
fn a_mount_whose_continuation_step_fails_on_a_device_error_hands_back_the_writes_of_every_publish_it_landed(
) {
    let mut pool = pool_with_a_three_unit_file("continuation-device-error-hands-back-the-writes");
    let intent = publish_the_intent_in_process(&mut pool);
    let image = pool.memory_pool();
    let counting_stream = SharedStream::new();
    let mut counting_devices = crash_state_devices(&image, &[], &[], &counting_stream);
    let counted = mount_writable_with_test_only_switches_and_deletion_switches(
        &parameters(),
        &mut counting_devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        deletion_switches_with_batches_of_one(),
    )
    .expect("拷贝上的那一次挂载续做走完");
    let continuation_writes: Vec<u64> = counted
        .output
        .deletion_queue_continuation
        .iter()
        .map(|step| step.writes.total().write_calls)
        .collect();
    assert_eq!(continuation_writes.len(), 3, "续做三步");
    let writes_of_the_whole_mount = u64::try_from(
        counting_stream
            .operations()
            .iter()
            .filter(|operation| operation.kind != singlefs_harness::RecordedOperationKind::Barrier)
            .count(),
    )
    .expect("写数装得进 u64");
    let writes_before_the_second_step =
        writes_of_the_whole_mount - continuation_writes[1] - continuation_writes[2];
    let warm_up_publishes = counted.output.warm_up_publishes.len();
    drop(counted);
    let writes_so_far = std::rc::Rc::new(std::cell::Cell::new(0));
    let mut failing_devices: Vec<(
        singlefs_core::address::DeviceIdentity,
        DeviceFailingOneCountedWrite<_>,
    )> = crash_state_devices(&image, &[], &[], &SharedStream::new())
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                DeviceFailingOneCountedWrite {
                    inner: device,
                    writes_so_far: writes_so_far.clone(),
                    failing_write: writes_before_the_second_step + 1,
                },
            )
        })
        .collect();
    let failed = match mount_writable_with_test_only_switches_and_deletion_switches(
        &parameters(),
        &mut failing_devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        deletion_switches_with_batches_of_one(),
    ) {
        Err(singlefs_core::mount::MountError::Publish(failed)) => failed,
        Err(other) => panic!("续做那一步的块设备错交回 MountError::Publish：{other:?}"),
        Ok(_) => panic!("续做第二步的写报错，挂载却做成了"),
    };
    assert!(
        matches!(
            failed.cause,
            singlefs_core::transaction::PublishError::BlockDevice(_)
        ),
        "错是那一次写的块设备错：{:?}",
        failed.cause
    );
    let image_after = MemoryPool {
        devices: failing_devices
            .iter()
            .map(|(identity, device)| (*identity, device.inner.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: common::IMAGE_BYTES,
    };
    let publishes_landed = mount_read_only(&image_after)
        .expect("只读挂载")
        .effective_root
        .checkpoint_txg
        .0
        - intent.root.checkpoint_txg.0;
    assert_eq!(
        (
            failed.writes_of_persisted_publishes.len(),
            u64::try_from(failed.writes_of_persisted_publishes.len()).expect("份数"),
        ),
        (1 + warm_up_publishes + 1, publishes_landed,),
        "错里的已落盘写账一次发布一份：写行、暖机、续做第一步，份数等于盘上落了几次发布"
    );
    assert!(
        failed
            .writes_of_persisted_publishes
            .iter()
            .all(|writes| writes.total().write_calls > 0),
        "每一份都有写：{:?}",
        failed.writes_of_persisted_publishes
    );
}

/// 回退到意图那一版（队列带回第一个文件）之后经会话覆盖写（攻方 `m3-unlink-code-r1` A2 那一形）：会话先走空、落盘三步（留 2、留 1、最后一批），
/// 现行那一版成了有树而文件集合为空的一版，再拒成 `NoFileVersionToChange`——拒绝里带着走空发成的三版（按 txg 接在回退那一版之后），
/// 一串抬 F 都没推；盘上从回退那一版起正好落了三次（代码轮 U10）。
/// 判别力：这个成员照旧不带发成的几版时交回的是空表，那一项红。
#[test]
fn an_overwrite_after_rolling_back_to_the_intent_walks_the_queue_and_its_refusal_carries_every_version_it_published(
) {
    let mut pool = pool_with_a_three_unit_file("overwrite-after-rolling-back-to-the-intent");
    let (mounted, mut devices) = remount_with_batches_of_one(&mut pool);
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let unlinked = session
        .publish_user_change(
            &mut devices,
            UserChange::Unlink {
                inode: FIRST_FILE,
                write_time_seconds: write_time_seconds_of_the_first_file(&pool),
            },
        )
        .expect("会话 unlink");
    let intent_root = unlinked.versions_published[0].root;
    let mut current = session
        .current
        .version_of_trees()
        .expect("删完之后有树")
        .clone();
    roll_back_by_a_forward_publish(
        &parameters(),
        &mut devices,
        &mut session.allocator,
        &mut current,
        RollbackTarget {
            instance: intent_root.instance,
            checkpoint_txg: intent_root.checkpoint_txg,
        },
    )
    .expect("挂着时回退到意图那一版");
    let rollback_txg = current.root.checkpoint_txg.0;
    assert_eq!(current.deleted_inodes, vec![FIRST_FILE], "回退带回队列");
    session.current = PoolVersion::of_a_version_of_trees(current);
    let content = vec![7_u8; 100];
    let refusal = session
        .publish_user_change(
            &mut devices,
            UserChange::Overwrite(FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            }),
        )
        .expect_err("走空之后没有可改的文件");
    let UserChangeRefused::NoFileVersionToChange {
        floor_raises,
        refusals_that_pushed_the_floor_raises,
        versions_published,
    } = &refusal
    else {
        panic!("拒成没有可改的文件：{refusal:?}");
    };
    assert_eq!(
        (
            floor_raises.len(),
            refusals_that_pushed_the_floor_raises.len(),
            versions_published
                .iter()
                .map(|version| (
                    version.root.checkpoint_txg.0 - rollback_txg,
                    version.data_pointers.len(),
                    version.deleted_inodes.len(),
                ))
                .collect::<Vec<_>>(),
        ),
        (0, 0, vec![(1, 2, 1), (2, 1, 1), (3, 0, 0)]),
        "拒绝里带着走空发成的三版（留 2、留 1、最后一批），一串抬 F 都没推"
    );
    assert_eq!(
        member_name(&session.current),
        "WithTreesAndNoFile",
        "会话的现行那一版是走空之后的那一版"
    );
    drop(devices);
    assert_eq!(
        mount_read_only(&pool.memory_pool())
            .expect("只读挂载")
            .effective_root
            .checkpoint_txg
            .0
            - rollback_txg,
        3,
        "盘上从回退那一版起正好落了三次发布，与拒绝里带的版数相等"
    );
}
