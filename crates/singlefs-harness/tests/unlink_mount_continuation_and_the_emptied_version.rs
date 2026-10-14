//! unlink 之后的那几种版本与续做（D8（核心索引结构） 已定项 16 第 5、6、7 条；D23（journal 的角色与格式） 已定项 14）：
//! 删空的一版（有树、文件集合为空）是 `PoolVersion` / `RebuiltVersion` / `PreviousVersion` 的第三个成员；可写挂载在写行与暖机之后
//! 把队列走到空；删除那两个只供测试的开关经可写挂载的开关链进会话；回退新根的 `deleted_inodes` 取 R_old 的、下一次挂载续做走空它。
//! 三个单元的第一个文件、只供测试的批大小 1：意图 → 留 2 → 留 1 → 最后一批。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/unlink_mount_continuation_and_the_emptied_version.rs"]
mod scenarios_of_unlink_mount_continuation_and_the_emptied_version;

use common::{
    build_pool, crash_state_devices, disk_snapshot, memory_pool_of_sparse_devices, parameters,
    FIXED_WRITE_TIME_SECONDS,
};
use common_images_for_the_checker_tier::PoolCheckerImages;
use scenarios_of_unlink_mount_continuation_and_the_emptied_version::{
    batches_of_one_data_unit, deletion_switches_with_batches_of_one, member_name,
    pool_with_a_three_unit_file, publish_the_intent_in_process, publish_the_intent_in_process_at,
    publish_the_next_step_in_process, remount_with_batches_of_one,
    txg_of_the_last_row_or_warm_up_publish, write_time_seconds_of_the_first_file, FIRST_FILE,
    INSTANCE_OF_THE_PROCESS,
};
use singlefs_core::address::{DeviceOffsetInBytes, InodeNumber};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, roll_back_by_a_forward_publish,
    RollbackTarget, ShadowLedger,
};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::mounted_session::{MountedSession, UserChange, UserChangeRefused};
use singlefs_core::records::STATISTIC_COMMITTED_RESERVATION_BYTES;
use singlefs_core::recovery::{rebuild_version, RebuiltVersion};
use singlefs_core::transaction::{
    publish_new_inodes, publish_overwrite, publish_unlink_intent, publish_unlink_step, FirstFile,
    PoolVersion, PoolWriter, TransactionOutput, TransactionUnit, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::{build_index_node, parse_index_node};
use singlefs_format::{INODE_RECORD_BYTES, PACKED_UNIT_RECORDS_OFFSET, SLOT_BYTES};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::SharedStream;

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

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_unlink_mount_continuation_and_the_emptied_version::a_writable_mount_after_the_intent_continues_the_queue_to_empty_after_its_warm_up`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_writable_mount_after_the_intent_continues_the_queue_to_empty_after_its_warm_up() {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_unlink_mount_continuation_and_the_emptied_version::a_writable_mount_after_the_intent_continues_the_queue_to_empty_after_its_warm_up,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_unlink_mount_continuation_and_the_emptied_version::the_intent_and_a_middle_batch_each_set_the_change_count_to_the_txg_of_their_own_publish`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn the_intent_and_a_middle_batch_each_set_the_change_count_to_the_txg_of_their_own_publish() {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_unlink_mount_continuation_and_the_emptied_version::the_intent_and_a_middle_batch_each_set_the_change_count_to_the_txg_of_their_own_publish,
    );
}

/// 第一个文件那条记录在 `version` 这一版里、两块盘上各读出来的三个时间秒（[`time_seconds_of_an_inode_on_each_device`]）。
fn time_seconds_of_the_first_file_on_each_device(
    image: &MemoryPool,
    version: &TransactionOutput,
) -> Vec<[u64; 3]> {
    time_seconds_of_an_inode_on_each_device(image, version, FIRST_FILE)
}

/// `inode` 那条记录在 `version` 这一版里、两块盘上各读出来的三个时间秒（偏移 64 起 atime、mtime、ctime 各 8 字节，D8（核心索引结构）
/// 已定项 6 字段表）：按盘读这一版每片 inode 叶容器的单元字节，在码 3 记录区（`PACKED_UNIT_RECORDS_OFFSET` 起、每条 `INODE_RECORD_BYTES`）里
/// 找 inode 号是它的那一条。读的是盘，不是内存里那份记录。
fn time_seconds_of_an_inode_on_each_device(
    image: &MemoryPool,
    version: &TransactionOutput,
    inode: InodeNumber,
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
                        .find(|record| field_at(record, 0) == inode.0)
                        .map(|record| {
                            [
                                field_at(record, 64),
                                field_at(record, 72),
                                field_at(record, 80),
                            ]
                        })
                })
                .unwrap_or_else(|| {
                    panic!("这一版的 inode 叶容器里有 inode {} 那条记录（意图与中间各批之后 nlink = 0、还在树里）", inode.0)
                })
        })
        .collect()
}

/// 意图那次带调用方给的写入时间（用户 2026-10-03 定 Q-F「乙 意图带时间、三格同改」）：三个单元的文件发意图，给一个与这条记录上一版不同的时间，
/// 再发一批（留 2）。意图那一版两块盘上第一个文件那条记录的三个时间秒都等于给的时间；中间那一批照抄意图那次写进去的值、不另取。
/// 判别力：意图照旧带上一版的时间时第一项红；那一批不照抄上一版这条记录的时间（另取一个）时第二项红。
#[test]
fn the_intent_writes_the_given_time_into_the_three_time_fields_and_a_middle_batch_copies_it() {
    let observed = time_fields_on_disk_after_an_intent_and_a_middle_batch_in_process(
        "write-time-of-the-deletion-steps",
        GivenTimeAgainstThePreviousVersion::Later,
    );
    assert_eq!(
        observed.on_disk_in_every_version_with_the_record[0],
        vec![[observed.given_time; 3]; 2],
        "意图那一版两块盘上那条记录的 atime、mtime、ctime 秒都是给的时间（上一版是 {}）",
        observed.time_of_the_previous_version
    );
    assert_eq!(
        observed.on_disk_in_every_version_with_the_record[1],
        vec![[observed.given_time; 3]; 2],
        "中间那一批照抄意图那次写进去的时间"
    );
}

/// 进程内发意图（给一个与这条记录上一版不同的时间，在上一版哪一边由 `given_time_against_the_previous_version` 定），再发一批（留 2）；
/// 读意图那一版与那一批各自两块盘上第一个文件那条记录的三个时间秒（按先后，一版一项）。
fn time_fields_on_disk_after_an_intent_and_a_middle_batch_in_process(
    pool_tag: &str,
    given_time_against_the_previous_version: GivenTimeAgainstThePreviousVersion,
) -> TimeFieldsOnDiskAfterAnUnlink {
    let mut pool = pool_with_a_three_unit_file(pool_tag);
    let time_of_the_previous_version = write_time_seconds_of_the_first_file(&pool);
    let given_time =
        given_time_against_the_previous_version.given_time(time_of_the_previous_version);
    let intent = publish_the_intent_in_process_at(&mut pool, given_time);
    let on_disk_after_the_intent =
        time_seconds_of_the_first_file_on_each_device(&pool.memory_pool(), &intent);
    let middle_batch = publish_the_next_step_in_process(&mut pool);
    let on_disk_after_the_middle_batch =
        time_seconds_of_the_first_file_on_each_device(&pool.memory_pool(), &middle_batch);
    TimeFieldsOnDiskAfterAnUnlink {
        time_of_the_previous_version,
        given_time,
        on_disk_in_every_version_with_the_record: vec![
            on_disk_after_the_intent,
            on_disk_after_the_middle_batch,
        ],
    }
}

/// 进程内发意图给的时间早于这条记录上一版（上一版减 600 秒）：意图那一版与中间那一批两块盘上那条记录的三个时间秒都等于给的那个较早的时间。
/// 上面那一条只有晚于上一版一形，判不出「意图取给的时间与上一版里较大的那个」；会话与续做两条早于上一版的用例不经直接调
/// `publish_unlink_intent` 这一路。
/// 判别力：意图取 `write_time_seconds.max(record.write_time_seconds)`（`research/prompts/m3-batch16-code-r1-main-verification.md`
/// 第二节 N7 攻方那一刀）时意图写下的是上一版的时间，这一条红、晚于上一版那一条照旧绿。
#[test]
fn the_intent_given_a_time_earlier_than_the_previous_version_writes_that_earlier_time_into_the_three_time_fields_and_a_middle_batch_copies_it(
) {
    let observed = time_fields_on_disk_after_an_intent_and_a_middle_batch_in_process(
        "earlier-write-time-of-the-deletion-steps",
        GivenTimeAgainstThePreviousVersion::Earlier,
    );
    assert_eq!(
        observed.on_disk_in_every_version_with_the_record,
        vec![vec![[observed.given_time; 3]; 2]; 2],
        "意图那一版与中间那一批，两块盘上那条记录的 atime、mtime、ctime 秒都是给的那个较早的时间（上一版是 {}）",
        observed.time_of_the_previous_version
    );
}

/// 调用方给 unlink 的时间与这条记录上一版的写入时间差几秒。
const SECONDS_BETWEEN_THE_GIVEN_TIME_AND_THE_PREVIOUS_VERSION: u64 = 600;

/// 给 unlink 的时间比这条记录上一版晚几秒、不是 60 的倍数（上一版是测试池写的固定时间，是 60 的倍数）：取整到分钟的实现写下的不是它。
const SECONDS_AFTER_THE_PREVIOUS_VERSION_THAT_ARE_NOT_A_WHOLE_MINUTE: u64 = 7;

/// 给 unlink 的时间超过 32 位（2^32 + 7 秒，晚于测试池写的固定时间）：按 32 位存的实现写下的是 7。
const GIVEN_TIME_PAST_THIRTY_TWO_BITS_IN_SECONDS: u64 = (1 << 32) + 7;

/// 会话里 unlink 之前先建的那个 inode 的时间比这条记录上一版晚几秒：晚于 unlink 给的时间（上一版加 600 秒）。
const SECONDS_AFTER_THE_PREVIOUS_VERSION_OF_THE_NEW_INODE_BEFORE_THE_UNLINK: u64 = 1000;

/// 调用方给 unlink 的时间怎么取：落在这条记录上一版写入时间的哪一边、取哪个值。Q-F 乙只说意图写调用方给的时间、各批照抄，
/// 没说时间只进不退、没说存几位、没说精度到哪；每一形判一种错实现（`research/prompts/m3-batch17-code-r1-opus-output.md` 第三节）：
/// 早于上一版那一形判得出「意图取给的时间与上一版里较大的那个」（`research/prompts/m3-batch16-code-r1-main-verification.md` 第二节 N7），
/// 超过 32 位那一形判得出「按 32 位存」，不是 60 的倍数那一形判得出「取整到分钟」；晚于上一版 600 秒那一形这三种都判不出。
#[derive(Clone, Copy)]
enum GivenTimeAgainstThePreviousVersion {
    Later,
    Earlier,
    LaterBySecondsThatAreNotAWholeMinute,
    PastThirtyTwoBits,
}

impl GivenTimeAgainstThePreviousVersion {
    fn given_time(self, time_of_the_previous_version: u64) -> u64 {
        match self {
            Self::Later => time_of_the_previous_version
                .checked_add(SECONDS_BETWEEN_THE_GIVEN_TIME_AND_THE_PREVIOUS_VERSION)
                .expect("上一版的时间是测试池写的固定时间，加 600 秒不溢出"),
            Self::Earlier => time_of_the_previous_version
                .checked_sub(SECONDS_BETWEEN_THE_GIVEN_TIME_AND_THE_PREVIOUS_VERSION)
                .expect("上一版的时间是测试池写的固定时间，大于 600 秒"),
            Self::LaterBySecondsThatAreNotAWholeMinute => time_of_the_previous_version
                .checked_add(SECONDS_AFTER_THE_PREVIOUS_VERSION_THAT_ARE_NOT_A_WHOLE_MINUTE)
                .expect("上一版的时间是测试池写的固定时间，加 7 秒不溢出"),
            Self::PastThirtyTwoBits => GIVEN_TIME_PAST_THIRTY_TWO_BITS_IN_SECONDS,
        }
    }
}

/// 会话里 unlink 之前这次挂载还发不发别的改动。
#[derive(Clone, Copy)]
enum ChangeBeforeTheUnlinkInTheSession {
    NoOtherChange,
    /// 先建一个 inode，时间是上一版加 1000 秒（晚于 unlink 给的时间）：会话里时间只进不退的实现把 unlink 的时间抬到它。
    ANewInodeLaterThanTheGivenTime,
}

/// 一次 unlink 给的时间、这条记录上一版的写入时间，与之后还带这条记录的几版两块盘上读回的三个时间秒（一版一项、一块盘一组）。
struct TimeFieldsOnDiskAfterAnUnlink {
    time_of_the_previous_version: u64,
    given_time: u64,
    on_disk_in_every_version_with_the_record: Vec<Vec<[u64; 3]>>,
}

/// 经会话发 unlink（`UserChange::Unlink`，会话把调用方给的时间交给意图那次）：给一个与这条记录上一版不同的时间（在上一版哪一边由
/// `given_time_against_the_previous_version` 定），会话发成意图与三步；读意图那一版与还带这条记录的两批（留 2、留 1）各自两块盘上
/// 第一个文件那条记录的三个时间秒。`change_before_the_unlink` 说这次挂载在 unlink 之前还发不发别的改动。
fn time_fields_on_disk_after_a_session_unlink(
    pool_tag: &str,
    given_time_against_the_previous_version: GivenTimeAgainstThePreviousVersion,
    change_before_the_unlink: ChangeBeforeTheUnlinkInTheSession,
) -> TimeFieldsOnDiskAfterAnUnlink {
    let mut pool = pool_with_a_three_unit_file(pool_tag);
    let time_of_the_previous_version = write_time_seconds_of_the_first_file(&pool);
    let given_time =
        given_time_against_the_previous_version.given_time(time_of_the_previous_version);
    let (mounted, mut devices) = remount_with_batches_of_one(&mut pool);
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    match change_before_the_unlink {
        ChangeBeforeTheUnlinkInTheSession::NoOtherChange => {}
        ChangeBeforeTheUnlinkInTheSession::ANewInodeLaterThanTheGivenTime => {
            let time_of_the_new_inode = time_of_the_previous_version
                .checked_add(SECONDS_AFTER_THE_PREVIOUS_VERSION_OF_THE_NEW_INODE_BEFORE_THE_UNLINK)
                .expect("上一版的时间是测试池写的固定时间，加 1000 秒不溢出");
            assert!(
                time_of_the_new_inode > given_time,
                "先建的那个 inode 的时间 {time_of_the_new_inode} 晚于 unlink 给的时间 {given_time}"
            );
            session
                .publish_user_change(
                    &mut devices,
                    UserChange::NewInodes {
                        count: 1,
                        write_time_seconds: time_of_the_new_inode,
                    },
                )
                .expect("同一次挂载里 unlink 之前先建一个 inode");
        }
    }
    let published = session
        .publish_user_change(
            &mut devices,
            UserChange::Unlink {
                inode: FIRST_FILE,
                write_time_seconds: given_time,
            },
        )
        .expect("会话 unlink");
    drop(devices);
    let image = pool.memory_pool();
    let on_disk_in_every_version_with_the_record = published
        .versions_published
        .iter()
        .filter(|version| version.inode_record.is_some())
        .map(|version| time_seconds_of_the_first_file_on_each_device(&image, version))
        .collect();
    TimeFieldsOnDiskAfterAnUnlink {
        time_of_the_previous_version,
        given_time,
        on_disk_in_every_version_with_the_record,
    }
}

/// 给的时间晚于这条记录上一版：意图那一版与还带这条记录的两批（留 2、留 1）各自两块盘上第一个文件那条记录的三个时间秒都等于给的时间。
/// 上面那一条直接调 `publish_unlink_intent`，判不出会话这一层（`research/prompts/m3-batch14-code-r1-main-verification.md` 第二节 N3）。
/// 判别力：会话那一层丢掉调用方的时间、意图写 0 时（攻方那一刀 MQ1）这一条红。
#[test]
fn a_session_unlink_writes_the_given_time_into_the_three_time_fields_of_the_intent_and_of_every_batch_on_disk(
) {
    let observed = time_fields_on_disk_after_a_session_unlink(
        "write-time-of-a-session-unlink",
        GivenTimeAgainstThePreviousVersion::Later,
        ChangeBeforeTheUnlinkInTheSession::NoOtherChange,
    );
    assert_eq!(
        observed.on_disk_in_every_version_with_the_record,
        vec![vec![[observed.given_time; 3]; 2]; 3],
        "意图那一版与留 2、留 1 两批，两块盘上那条记录的 atime、mtime、ctime 秒都是调用方给会话的时间（上一版是 {}）",
        observed.time_of_the_previous_version
    );
}

/// 给的时间早于这条记录上一版（上一版减 600 秒）：意图那一版与留 2、留 1 两批各自两块盘上那条记录的三个时间秒都等于给的那个较早的时间，
/// 不取两者里较大的那个。
/// 判别力：意图取 `write_time_seconds.max(record.write_time_seconds)`（`research/prompts/m3-batch16-code-r1-main-verification.md`
/// 第二节 N7 攻方那一刀）时这一条红、晚于上一版那一条照旧绿。
#[test]
fn a_session_unlink_given_a_time_earlier_than_the_previous_version_writes_that_earlier_time_into_the_three_time_fields_of_the_intent_and_of_every_batch_on_disk(
) {
    let observed = time_fields_on_disk_after_a_session_unlink(
        "earlier-write-time-of-a-session-unlink",
        GivenTimeAgainstThePreviousVersion::Earlier,
        ChangeBeforeTheUnlinkInTheSession::NoOtherChange,
    );
    assert_eq!(
        observed.on_disk_in_every_version_with_the_record,
        vec![vec![[observed.given_time; 3]; 2]; 3],
        "意图那一版与留 2、留 1 两批，两块盘上那条记录的 atime、mtime、ctime 秒都是调用方给会话的那个较早的时间（上一版是 {}）",
        observed.time_of_the_previous_version
    );
}

/// 同一次挂载里先建一个 inode（时间是上一版加 1000 秒），再 unlink 第一个文件、给上一版加 600 秒（晚于上一版、早于会话里上一次给的）：
/// 意图那一版与留 2、留 1 两批两块盘上那条记录的三个时间秒都等于这次给的时间。上面几条的 unlink 都是这次挂载的第一次改动。
/// 判别力：会话那一层 unlink 的时间取给的与会话里给过的更晚时间里较大的那个（时间只进不退，攻方 `research/prompts/m3-batch17-code-r1-opus-output.md`
/// 第三节 qfm）时写下的是上一版加 1000 秒，这一条红；会话里只有 unlink 那几条照旧绿。
#[test]
fn a_session_unlink_after_a_new_inode_with_a_later_time_in_the_same_mount_writes_the_given_time_into_the_three_time_fields_of_the_intent_and_of_every_batch_on_disk(
) {
    let observed = time_fields_on_disk_after_a_session_unlink(
        "write-time-of-a-session-unlink-after-a-later-new-inode",
        GivenTimeAgainstThePreviousVersion::Later,
        ChangeBeforeTheUnlinkInTheSession::ANewInodeLaterThanTheGivenTime,
    );
    assert_eq!(
        observed.on_disk_in_every_version_with_the_record,
        vec![vec![[observed.given_time; 3]; 2]; 3],
        "意图那一版与留 2、留 1 两批，两块盘上那条记录的 atime、mtime、ctime 秒都是这次给的时间，不是之前那个 inode 的更晚时间（上一版是 {}）",
        observed.time_of_the_previous_version
    );
}

/// 同一次挂载里后一次 unlink 给的时间比这条记录上一版晚几秒：早于前一次给的（上一版加 600 秒）、不早于中间建的那个 inode 的时间（上一版）。
const SECONDS_AFTER_THE_PREVIOUS_VERSION_OF_THE_SECOND_UNLINK_IN_THE_SAME_MOUNT: u64 = 300;

/// `version` 这一版的 inode 树里有没有 `inode` 那条记录。
fn version_holds_the_record_of(version: &TransactionOutput, inode: InodeNumber) -> bool {
    version
        .inode_leaf_container_contents()
        .iter()
        .flat_map(|container| container.records.iter())
        .any(|record| record.inode == inode.0)
}

/// 同一次挂载里 unlink 两次、后一次给的时间更早（判决 `research/prompts/m3-batch19-code-r1-main-verification.md` 第二节「K2 N1 错实现漏红」那一行，
/// 攻方 `research/prompts/m3-batch19-code-r1-opus-output.md`「N1」一节）：先 unlink 第一个文件、给上一版加 600 秒（会话走完意图与三步，这一版删空），
/// 再建一个 inode（时间就是上一版的，不晚于后一次给的），再 unlink 它、给上一版加 300 秒。后一次那个 inode 那条记录只在意图那一版里
/// （它没有数据单元，意图之后一批就删完），意图那一版两块盘上那条记录的三个时间秒都等于后一次给的时间。上面几条每次挂载只 unlink 一次。
/// 判别力：会话那一层只对 unlink 只进不退（unlink 的时间取给的与这次挂载里上一次 unlink 用过的时间里较大的那个，攻方那一刀 q-unlink-mono）时
/// 写下的是前一次的上一版加 600 秒，这一条红；只看这条记录上一版的时间、只看上一版各条记录最晚的时间、只记建 inode 给过的时间的几种写法
/// 写下的都是后一次给的时间，照旧绿。
#[test]
fn two_session_unlinks_in_the_same_mount_the_second_given_an_earlier_time_write_the_second_given_time_into_the_three_time_fields_of_its_intent_on_disk(
) {
    let mut pool =
        pool_with_a_three_unit_file("write-time-of-a-second-session-unlink-earlier-than-the-first");
    let time_of_the_previous_version = write_time_seconds_of_the_first_file(&pool);
    let time_given_to_the_first_unlink =
        GivenTimeAgainstThePreviousVersion::Later.given_time(time_of_the_previous_version);
    let time_given_to_the_second_unlink = time_of_the_previous_version
        .checked_add(SECONDS_AFTER_THE_PREVIOUS_VERSION_OF_THE_SECOND_UNLINK_IN_THE_SAME_MOUNT)
        .expect("上一版的时间是测试池写的固定时间，加 300 秒不溢出");
    let time_of_the_new_inode = time_of_the_previous_version;
    assert!(
        time_of_the_new_inode <= time_given_to_the_second_unlink
            && time_given_to_the_second_unlink < time_given_to_the_first_unlink,
        "建的那个 inode 的时间 {time_of_the_new_inode} 不晚于后一次给的 {time_given_to_the_second_unlink}，后一次早于前一次给的 {time_given_to_the_first_unlink}"
    );
    let (mounted, mut devices) = remount_with_batches_of_one(&mut pool);
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    session
        .publish_user_change(
            &mut devices,
            UserChange::Unlink {
                inode: FIRST_FILE,
                write_time_seconds: time_given_to_the_first_unlink,
            },
        )
        .expect("同一次挂载里前一次 unlink 第一个文件");
    let created = session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: time_of_the_new_inode,
            },
        )
        .expect("删空的那一版上建一个 inode");
    let new_inodes: Vec<InodeNumber> = created
        .versions_published
        .last()
        .expect("建 inode 发成一版")
        .inode_leaf_container_contents()
        .iter()
        .flat_map(|container| container.records.iter())
        .map(|record| InodeNumber(record.inode))
        .collect();
    let [new_inode] = new_inodes[..] else {
        panic!("删空的那一版上建一个之后树里只有新建的那一条记录：{new_inodes:?}");
    };
    let published = session
        .publish_user_change(
            &mut devices,
            UserChange::Unlink {
                inode: new_inode,
                write_time_seconds: time_given_to_the_second_unlink,
            },
        )
        .expect("同一次挂载里后一次 unlink 新建的那个 inode");
    drop(devices);
    let image = pool.memory_pool();
    let on_disk_in_every_version_with_the_record: Vec<Vec<[u64; 3]>> = published
        .versions_published
        .iter()
        .filter(|version| version_holds_the_record_of(version, new_inode))
        .map(|version| time_seconds_of_an_inode_on_each_device(&image, version, new_inode))
        .collect();
    assert_eq!(
        on_disk_in_every_version_with_the_record,
        vec![vec![[time_given_to_the_second_unlink; 3]; 2]],
        "后一次 unlink 的意图那一版（还带那条记录的只有这一版），两块盘上 inode {} 那条记录的 atime、mtime、ctime 秒都是后一次给的时间，\
         不是前一次给的 {time_given_to_the_first_unlink}",
        new_inode.0
    );
}

/// 给 unlink 的时间超过 32 位（2^32 + 7 秒）：意图那一版与留 2、留 1 两批两块盘上那条记录的三个时间秒都是整个 64 位值。
/// 判别力：意图写时间时按 32 位存（攻方第三节 qfu）时写下的是 7，这一条红；给的时间小于 2^32 的几条照旧绿。
#[test]
fn a_session_unlink_given_a_time_past_thirty_two_bits_writes_it_whole_into_the_three_time_fields_of_the_intent_and_of_every_batch_on_disk(
) {
    let observed = time_fields_on_disk_after_a_session_unlink(
        "write-time-of-a-session-unlink-past-thirty-two-bits",
        GivenTimeAgainstThePreviousVersion::PastThirtyTwoBits,
        ChangeBeforeTheUnlinkInTheSession::NoOtherChange,
    );
    assert!(
        observed.given_time > u64::from(u32::MAX)
            && observed.given_time > observed.time_of_the_previous_version,
        "给的时间 {} 超过 32 位、晚于上一版 {}",
        observed.given_time,
        observed.time_of_the_previous_version
    );
    assert_eq!(
        observed.on_disk_in_every_version_with_the_record,
        vec![vec![[observed.given_time; 3]; 2]; 3],
        "意图那一版与留 2、留 1 两批，两块盘上那条记录的 atime、mtime、ctime 秒都是给的整个 64 位时间"
    );
}

/// 给 unlink 的时间不是 60 的倍数（上一版加 7 秒）：意图那一版与留 2、留 1 两批两块盘上那条记录的三个时间秒精确到秒、等于给的时间。
/// 判别力：意图写时间时取整到分钟（攻方第三节 qfr）时写下的是上一版的时间，这一条红；给的时间是 60 的倍数的几条照旧绿。
#[test]
fn a_session_unlink_given_a_time_that_is_not_a_whole_minute_writes_it_to_the_second_into_the_three_time_fields_of_the_intent_and_of_every_batch_on_disk(
) {
    let observed = time_fields_on_disk_after_a_session_unlink(
        "write-time-of-a-session-unlink-not-a-whole-minute",
        GivenTimeAgainstThePreviousVersion::LaterBySecondsThatAreNotAWholeMinute,
        ChangeBeforeTheUnlinkInTheSession::NoOtherChange,
    );
    assert_ne!(
        observed.given_time % 60,
        0,
        "给的时间 {} 不是 60 的倍数",
        observed.given_time
    );
    assert_eq!(
        observed.on_disk_in_every_version_with_the_record,
        vec![vec![[observed.given_time; 3]; 2]; 3],
        "意图那一版与留 2、留 1 两批，两块盘上那条记录的 atime、mtime、ctime 秒都是给的时间、精确到秒（上一版是 {}）",
        observed.time_of_the_previous_version
    );
}

/// 进程内发意图（给一个与这条记录上一版不同的时间，在上一版哪一边由 `given_time_against_the_previous_version` 定），一批没发就重开镜像
/// （意图落盘之后崩溃），可写挂载续做；读续做里还带这条记录的那两批（留 2、留 1）各自两块盘上第一个文件那条记录的三个时间秒。
fn time_fields_on_disk_in_the_continuation_batches_after_an_intent(
    pool_tag: &str,
    given_time_against_the_previous_version: GivenTimeAgainstThePreviousVersion,
) -> TimeFieldsOnDiskAfterAnUnlink {
    let mut pool = pool_with_a_three_unit_file(pool_tag);
    let time_of_the_previous_version = write_time_seconds_of_the_first_file(&pool);
    let given_time =
        given_time_against_the_previous_version.given_time(time_of_the_previous_version);
    publish_the_intent_in_process_at(&mut pool, given_time);
    let (mounted, devices) = remount_with_batches_of_one(&mut pool);
    drop(devices);
    let image = pool.memory_pool();
    let on_disk_in_every_version_with_the_record = mounted
        .output
        .deletion_queue_continuation
        .iter()
        .filter(|step| step.inode_record.is_some())
        .map(|step| time_seconds_of_the_first_file_on_each_device(&image, step))
        .collect();
    TimeFieldsOnDiskAfterAnUnlink {
        time_of_the_previous_version,
        given_time,
        on_disk_in_every_version_with_the_record,
    }
}

/// 意图给的时间晚于这条记录上一版：续做里还带这条记录的那两批（留 2、留 1）各自两块盘上第一个文件那条记录的三个时间秒都等于意图那次给的时间。
/// 续做的几批没有调用方给时间，照抄从盘上读回的上一版这条记录的值（用户 2026-10-03 定 Q-F 乙「各批照抄」；D8（核心索引结构） 已定项 16 第 7 条 ①b）。
/// 判别力：续做那几批另取时间（取 0、取一个固定时间），或挂载从盘上读回这条记录时丢了时间，这一条红。
#[test]
fn the_batches_of_a_mount_continuation_after_the_intent_copy_the_time_the_intent_wrote_on_disk() {
    let observed = time_fields_on_disk_in_the_continuation_batches_after_an_intent(
        "write-time-of-the-continuation-batches",
        GivenTimeAgainstThePreviousVersion::Later,
    );
    assert_eq!(
        observed.on_disk_in_every_version_with_the_record,
        vec![vec![[observed.given_time; 3]; 2]; 2],
        "续做的留 2、留 1 两批，两块盘上那条记录的 atime、mtime、ctime 秒都照抄意图那次给的时间（上一版是 {}）",
        observed.time_of_the_previous_version
    );
}

/// 意图给的时间早于这条记录上一版（上一版减 600 秒）：续做的留 2、留 1 两批各自两块盘上那条记录的三个时间秒都照抄意图写下的那个较早的时间。
/// 判别力：意图取 `write_time_seconds.max(record.write_time_seconds)`（`research/prompts/m3-batch16-code-r1-main-verification.md`
/// 第二节 N7 攻方那一刀）时意图写下的是上一版的时间、续做照抄它，这一条红、晚于上一版那一条照旧绿。
#[test]
fn the_batches_of_a_mount_continuation_after_an_intent_given_a_time_earlier_than_the_previous_version_copy_that_earlier_time_on_disk(
) {
    let observed = time_fields_on_disk_in_the_continuation_batches_after_an_intent(
        "earlier-write-time-of-the-continuation-batches",
        GivenTimeAgainstThePreviousVersion::Earlier,
    );
    assert_eq!(
        observed.on_disk_in_every_version_with_the_record,
        vec![vec![[observed.given_time; 3]; 2]; 2],
        "续做的留 2、留 1 两批，两块盘上那条记录的 atime、mtime、ctime 秒都照抄意图写下的那个较早的时间（上一版是 {}）",
        observed.time_of_the_previous_version
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

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_unlink_mount_continuation_and_the_emptied_version::rolling_back_to_the_intent_takes_its_deleted_inodes_root_and_the_next_mount_continues_it_to_empty`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn rolling_back_to_the_intent_takes_its_deleted_inodes_root_and_the_next_mount_continues_it_to_empty(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_unlink_mount_continuation_and_the_emptied_version::rolling_back_to_the_intent_takes_its_deleted_inodes_root_and_the_next_mount_continues_it_to_empty,
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
