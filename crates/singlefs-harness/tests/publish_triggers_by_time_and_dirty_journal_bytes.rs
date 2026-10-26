//! 串行下时间与脏量两种发布触发（判决 `research/prompts/m4-publish-trigger-r1-main-verification.md` 第二节「时间那一支」「脏字节口径」「只供测试的开关」
//! 「串行下不立的」四格；D16（发布语义） 已定项 2、5、10；用户 2026-10-26 定射程「串行下时间与脏量两种触发，时钟由调用方给、测试里能换，阈值先用只供测试的开关」）：
//! 攒批的会话（`mounted_session::MountedSessionWithPendingChanges`）收下改动不当场发布，触发 ⟺ 待发非空 ∧（now − 上次发布的读数 ≥ `T_time` ∨
//! 待发改动要写的 journal 字节 ≥ 有效 `T_dirty`），只在 `accept` 收下一个完整改动之后与 `publish_when_due` 里判。
//!
//! 每条用例在两块内存稀疏盘上 mkfs、取号、暖机、第一个文件（`tests/common_admission_pool/mod.rs`），会话接上待发；阈值经默认关的特性
//! `test-only-publish-thresholds` 后面那个只供测试的来源压到测试的尺度（这个测试包的 dev 依赖打开它），钟是用例交的读数。
//! 时间那一支、脏量那一支各自先到、都不到、空闲时不推空发布、读数倒退不触发、脏量按 journal 字节算的三形（只改元数据的改动也计入、与环长 ÷ F 那一夹同量纲、
//! 批内覆盖写 K 次照 K 次计）、产品路径的有效值、冻结之后 `accept` 在任何读写之前拒，各一条。
mod common_admission_pool;

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use common_admission_pool::{
    content_of, device_bytes_of_the_384_slot_small_devices,
    parameters_of_the_384_slot_small_devices, plain_device, DeviceOverASparseImage, PoolUnderTest,
    FIRST_WRITE_TIME_SECONDS, OVERWRITE_BYTES,
};
use singlefs_core::address::{DeviceOffsetInBytes, InodeNumber};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::compaction_plan::CallerMonotonicClockReading;
use singlefs_core::journal::JournalBytesToWrite;
use singlefs_core::mounted_session::{
    publish_trigger_cause, AcceptRefused, ChangeAccepted, HowTheChangeWasTaken,
    MountedSessionWithPendingChanges, PublishTriggerCause, ReasonOfThePublish, UserChange,
};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::system_configuration::{
    journal_in_flight_record_limit, PublishTriggerThresholds,
};
use singlefs_core::transaction::FirstFile;
use singlefs_format::{JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES};
use singlefs_harness::memory_pool::{SparseBlockDevice, SparseDevice};

/// 时间那一支不该先到的用例给的 `T_time`：一小时，用例里的钟走不到。
const TIME_THRESHOLD_NEVER_REACHED: Duration = Duration::from_secs(3600);

/// 脏量那一支不该先到的用例给的（夹之前的）`T_dirty`：夹成环长 ÷ F（6 MiB 的环是 2 MiB），用例里收下的几次改动远到不了。
const DIRTY_THRESHOLD_NEVER_REACHED_BEFORE_THE_CLAMP: u64 = 1 << 40;

/// 时间那一支的用例给的 `T_time`：50 毫秒。
const TIME_THRESHOLD_OF_FIFTY_MILLISECONDS: Duration = Duration::from_millis(50);

fn reading(milliseconds: u64) -> CallerMonotonicClockReading {
    CallerMonotonicClockReading(Duration::from_millis(milliseconds))
}

/// 两块小盘上起池（第一个文件之后），会话接上待发，两个阈值取只供测试的值（脏量照环长夹）。
fn batching_pool_on<Device: DeviceOverASparseImage>(
    time_threshold: Duration,
    dirty_threshold_before_the_clamp: u64,
    wrap: impl Fn(singlefs_core::address::DeviceIdentity, SparseBlockDevice) -> Device,
) -> (PoolUnderTest<Device>, MountedSessionWithPendingChanges) {
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        wrap,
    );
    let session = pool.session.take().expect("起池之后有会话");
    let thresholds = PublishTriggerThresholds::of_test_only_values_clamped_by_the_journal_ring(
        time_threshold,
        dirty_threshold_before_the_clamp,
        pool.parameters.geometry.journal_ring_bytes,
    );
    (
        pool,
        MountedSessionWithPendingChanges::of_the_session_with_thresholds(session, thresholds),
    )
}

fn batching_pool(
    time_threshold: Duration,
    dirty_threshold_before_the_clamp: u64,
) -> (
    PoolUnderTest<SparseBlockDevice>,
    MountedSessionWithPendingChanges,
) {
    batching_pool_on(
        time_threshold,
        dirty_threshold_before_the_clamp,
        plain_device,
    )
}

/// 覆盖写第一个文件（一个数据单元以内）交给 `accept`，内容按 `seed` 定。
fn accept_an_overwrite_of_the_first_file<Device: DeviceOverASparseImage>(
    pool: &mut PoolUnderTest<Device>,
    session: &mut MountedSessionWithPendingChanges,
    seed: u64,
    now: CallerMonotonicClockReading,
) -> (Vec<u8>, Result<ChangeAccepted, AcceptRefused>) {
    let content = content_of(OVERWRITE_BYTES, seed);
    let accepted = session.accept(
        &mut pool.devices,
        UserChange::Overwrite(FirstFile {
            content: &content,
            write_time_seconds: FIRST_WRITE_TIME_SECONDS + seed,
        }),
        now,
    );
    (content, accepted)
}

/// 收下了、没触发：这一次报的 journal 字节。
fn kept_pending_without_a_publish(accepted: &ChangeAccepted) -> JournalBytesToWrite {
    let HowTheChangeWasTaken::KeptPending {
        journal_bytes_of_this_change,
    } = accepted.taken
    else {
        panic!("数据写类收进待发：{accepted:?}");
    };
    assert!(
        accepted.publishes.is_empty(),
        "收下之后两支都没到：一次都不发，{accepted:?}"
    );
    journal_bytes_of_this_change
}

/// 这一刻盘上恢复读出的第一个文件。
fn first_file_read_back<Device: DeviceOverASparseImage>(pool: &PoolUnderTest<Device>) -> Vec<u8> {
    match recover(&pool.image(), JournalPolicy::Consult).outcome {
        RecoveryOutcome::FileRead { content, .. } => content,
        other @ (RecoveryOutcome::NoFile { .. } | RecoveryOutcome::Failed { .. }) => {
            panic!("第一个文件之后恢复读得出它：{other:?}")
        }
    }
}

fn txg_of_the_current_version(session: &MountedSessionWithPendingChanges) -> u64 {
    session.session().current.root().checkpoint_txg.0
}

/// 时间那一支先到：脏量压在阈值下（夹成 2 MiB），收下一次覆盖写之后钟走到 49 毫秒不发、走到 50 毫秒（`≥`）在 `publish_when_due` 里发，
/// 理由是时间那一支；发成的那一版读出的是收下的内容，收下之前那一刻盘上没变。
#[test]
fn the_time_threshold_reached_first_publishes_the_pending_change_by_time_when_due() {
    let (mut pool, mut session) = batching_pool(
        TIME_THRESHOLD_OF_FIFTY_MILLISECONDS,
        DIRTY_THRESHOLD_NEVER_REACHED_BEFORE_THE_CLAMP,
    );
    let content_before = first_file_read_back(&pool);
    let image_before = pool.image();
    let (content, accepted) =
        accept_an_overwrite_of_the_first_file(&mut pool, &mut session, 1, reading(0));
    kept_pending_without_a_publish(&accepted.expect("收下覆盖写"));
    assert_eq!(pool.image(), image_before, "收下不写盘：盘上逐字节不变");
    assert_eq!(
        first_file_read_back(&pool),
        content_before,
        "收下之后读出的仍是上一次发布那一版"
    );
    let not_yet = session
        .publish_when_due(&mut pool.devices, reading(49))
        .expect("49 毫秒：判一次");
    assert!(
        not_yet.is_none(),
        "离上次发布 49 毫秒、脏量远在阈值下：不发"
    );
    let published = session
        .publish_when_due(&mut pool.devices, reading(50))
        .expect("50 毫秒：判一次")
        .expect("离上次发布 50 毫秒（≥ T_time）：发");
    assert_eq!(
        published.reason,
        ReasonOfThePublish::TriggeredWhenDue(PublishTriggerCause::TimeThreshold),
        "时间那一支先到"
    );
    assert_eq!(
        first_file_read_back(&pool),
        content,
        "发成之后读出收下的内容"
    );
    assert_eq!(session.pending_changes_accepted(), 0, "待发排空");
}

/// 脏量那一支先到：`T_time` 一小时、钟一直停在 0，脏量压到三次覆盖写的 journal 字节；前两次收下不发，第三次收下之后在 `accept` 里发，
/// 理由是脏量那一支，待发三次改动的 journal 字节之和到了阈值。
#[test]
fn the_dirty_journal_bytes_reached_first_publish_the_pending_changes_in_the_accept_by_dirty() {
    let (mut probe_pool, mut probe_session) = batching_pool(
        TIME_THRESHOLD_NEVER_REACHED,
        DIRTY_THRESHOLD_NEVER_REACHED_BEFORE_THE_CLAMP,
    );
    let (_, probed) =
        accept_an_overwrite_of_the_first_file(&mut probe_pool, &mut probe_session, 1, reading(0));
    let journal_bytes_of_one_overwrite = kept_pending_without_a_publish(&probed.expect("收下"));
    let (mut pool, mut session) = batching_pool(
        TIME_THRESHOLD_NEVER_REACHED,
        3 * journal_bytes_of_one_overwrite.0,
    );
    for seed in 1..=2 {
        let (_, accepted) =
            accept_an_overwrite_of_the_first_file(&mut pool, &mut session, seed, reading(0));
        kept_pending_without_a_publish(&accepted.expect("收下"));
    }
    assert_eq!(
        session.journal_bytes_of_the_pending_changes(),
        JournalBytesToWrite(2 * journal_bytes_of_one_overwrite.0),
        "两次收下各算各的"
    );
    let txg_before = txg_of_the_current_version(&session);
    let (content, accepted) =
        accept_an_overwrite_of_the_first_file(&mut pool, &mut session, 3, reading(0));
    let accepted = accepted.expect("第三次收下");
    assert_eq!(
        accepted.publishes.len(),
        1,
        "第三次收下之后到了脏量阈值：这次调用里发一次"
    );
    assert_eq!(
        accepted.publishes[0].reason,
        ReasonOfThePublish::TriggeredAfterTheAccept(PublishTriggerCause::DirtyThreshold),
        "脏量那一支先到（钟没动）"
    );
    assert_eq!(
        txg_of_the_current_version(&session),
        txg_before + 1,
        "三次覆盖写并成一次发布"
    );
    assert_eq!(
        first_file_read_back(&pool),
        content,
        "发出去的是最后那一次的内容"
    );
}

/// 两支都不到：`T_time` 一小时、脏量夹成 2 MiB，收下一次之后钟走了几步，`publish_when_due` 一次都不发、盘上逐字节不变，待发还在。
#[test]
fn neither_threshold_reached_publishes_nothing_and_leaves_the_disk_unchanged() {
    let (mut pool, mut session) = batching_pool(
        TIME_THRESHOLD_NEVER_REACHED,
        DIRTY_THRESHOLD_NEVER_REACHED_BEFORE_THE_CLAMP,
    );
    let image_before = pool.image();
    let (_, accepted) =
        accept_an_overwrite_of_the_first_file(&mut pool, &mut session, 1, reading(0));
    kept_pending_without_a_publish(&accepted.expect("收下"));
    for milliseconds in [1, 1000, 60_000, 3_599_999] {
        assert!(
            session
                .publish_when_due(&mut pool.devices, reading(milliseconds))
                .expect("判一次")
                .is_none(),
            "{milliseconds} 毫秒：两支都没到，不发"
        );
    }
    assert_eq!(pool.image(), image_before, "一次都没发：盘上逐字节不变");
    assert_eq!(session.pending_changes_accepted(), 1, "待发还在");
}

/// 空闲时不推空发布（判决第二节「时间那一支」：触发式带「待发非空」）：`T_time` 10 毫秒，一次都没收下，钟走到十亿毫秒，`publish_when_due` 与
/// `publish_pending_now` 都不发，现行那一版的 txg 不动、盘上逐字节不变；纯函数那一判待发空时两支都到了也交回不触发。
#[test]
fn an_idle_session_with_nothing_pending_pushes_no_empty_publish_however_far_the_clock_goes() {
    let (mut pool, mut session) = batching_pool(
        Duration::from_millis(10),
        DIRTY_THRESHOLD_NEVER_REACHED_BEFORE_THE_CLAMP,
    );
    let image_before = pool.image();
    let txg_before = txg_of_the_current_version(&session);
    for milliseconds in [0, 10, 1_000_000, 1_000_000_000] {
        assert!(
            session
                .publish_when_due(&mut pool.devices, reading(milliseconds))
                .expect("判一次")
                .is_none(),
            "{milliseconds} 毫秒：待发空，不推空发布"
        );
    }
    assert!(
        session
            .publish_pending_now(&mut pool.devices, reading(1_000_000_001))
            .expect("立即排空")
            .is_none(),
        "立即排空：待发空，不推空发布"
    );
    assert_eq!(
        txg_of_the_current_version(&session),
        txg_before,
        "一次都没发"
    );
    assert_eq!(pool.image(), image_before, "盘上逐字节不变");
    assert_eq!(
        publish_trigger_cause(
            true,
            Duration::from_secs(1_000_000),
            JournalBytesToWrite(u64::MAX),
            session.thresholds(),
        ),
        None,
        "待发空时时间与脏量都到了也不触发"
    );
}

/// 读数倒退当作没过时间、不回拨（判决第二节「时间那一支」）：基线 1000 毫秒，收下之后交一个倒退的读数 500 不发；1099 不发、1100 发（时间那一支）。
/// 发成之后再交倒退的读数 900 收下一次：上次发布的读数停在 1100、不回拨，1199 不发、1200 发。
#[test]
fn a_clock_reading_going_backwards_counts_as_no_time_elapsed_and_does_not_move_the_last_publish_back(
) {
    let (mut pool, mut session) = batching_pool(
        Duration::from_millis(100),
        DIRTY_THRESHOLD_NEVER_REACHED_BEFORE_THE_CLAMP,
    );
    assert!(session
        .publish_when_due(&mut pool.devices, reading(1000))
        .expect("基线")
        .is_none());
    let (_, accepted) =
        accept_an_overwrite_of_the_first_file(&mut pool, &mut session, 1, reading(1050));
    kept_pending_without_a_publish(&accepted.expect("收下"));
    for backwards_or_short in [500, 0, 1099] {
        assert!(
            session
                .publish_when_due(&mut pool.devices, reading(backwards_or_short))
                .expect("判一次")
                .is_none(),
            "{backwards_or_short} 毫秒：倒退或离基线不到 100 毫秒，不发"
        );
    }
    let first = session
        .publish_when_due(&mut pool.devices, reading(1100))
        .expect("判一次")
        .expect("1100 毫秒：离基线 100 毫秒，发");
    assert_eq!(
        first.reason,
        ReasonOfThePublish::TriggeredWhenDue(PublishTriggerCause::TimeThreshold)
    );
    assert_eq!(session.reading_of_the_last_publish(), Some(reading(1100)));
    let (_, accepted) =
        accept_an_overwrite_of_the_first_file(&mut pool, &mut session, 2, reading(900));
    kept_pending_without_a_publish(&accepted.expect("倒退的读数上收下"));
    assert_eq!(
        session.reading_of_the_last_publish(),
        Some(reading(1100)),
        "倒退的读数不回拨上次发布的读数"
    );
    assert!(session
        .publish_when_due(&mut pool.devices, reading(1199))
        .expect("判一次")
        .is_none());
    let second = session
        .publish_when_due(&mut pool.devices, reading(1200))
        .expect("判一次")
        .expect("1200 毫秒：离上次发布 100 毫秒，发");
    assert_eq!(
        second.reason,
        ReasonOfThePublish::TriggeredWhenDue(PublishTriggerCause::TimeThreshold)
    );
}

/// 脏量按待发改动要写的 journal 字节算，只改元数据的改动也计入（攻方 P2-甲：只数数据单元字节时，只建 inode 的那一批脏量那一支永远不触发）：
/// 钟不动、`T_time` 一小时，脏量压到两次建 inode 的 journal 字节；每次建 inode 报的 journal 字节至少一条记录，第二次收下之后在 `accept` 里按脏量发，
/// 发成的那一版建出了两次的 inode。
#[test]
fn changes_creating_inodes_only_count_toward_the_dirty_journal_bytes() {
    let accept_new_inodes = |pool: &mut PoolUnderTest<SparseBlockDevice>,
                             session: &mut MountedSessionWithPendingChanges,
                             seed: u64| {
        session.accept(
            &mut pool.devices,
            UserChange::NewInodes {
                count: 2,
                write_time_seconds: FIRST_WRITE_TIME_SECONDS + seed,
            },
            reading(0),
        )
    };
    let (mut probe_pool, mut probe_session) = batching_pool(
        TIME_THRESHOLD_NEVER_REACHED,
        DIRTY_THRESHOLD_NEVER_REACHED_BEFORE_THE_CLAMP,
    );
    let journal_bytes_of_one_creation = kept_pending_without_a_publish(
        &accept_new_inodes(&mut probe_pool, &mut probe_session, 1).expect("收下"),
    );
    assert!(
        journal_bytes_of_one_creation.0 >= JOURNAL_RECORD_BYTES,
        "建 inode 一个数据单元都不写，照样要写至少一条记录：{journal_bytes_of_one_creation:?}"
    );
    let (mut pool, mut session) = batching_pool(
        TIME_THRESHOLD_NEVER_REACHED,
        2 * journal_bytes_of_one_creation.0,
    );
    let watermark_before = session
        .session()
        .current
        .version_of_trees()
        .expect("有树")
        .inode_number_watermark();
    kept_pending_without_a_publish(&accept_new_inodes(&mut pool, &mut session, 1).expect("收下"));
    let accepted = accept_new_inodes(&mut pool, &mut session, 2).expect("第二次收下");
    assert_eq!(
        accepted.publishes.len(),
        1,
        "两次建 inode 的 journal 字节到了阈值：发"
    );
    assert_eq!(
        accepted.publishes[0].reason,
        ReasonOfThePublish::TriggeredAfterTheAccept(PublishTriggerCause::DirtyThreshold)
    );
    let version = session.session().current.version_of_trees().expect("有树");
    assert_eq!(
        version.inode_number_watermark(),
        watermark_before + 4,
        "两次各建两个，并在一次发布里"
    );
    for inode in watermark_before..watermark_before + 4 {
        assert!(
            version.inode_record_of(InodeNumber(inode)).is_some(),
            "inode {inode} 建出来了"
        );
    }
}

/// 脏量与环长 ÷ F 那一夹同量纲（攻方 P2-乙）：有效 `T_dirty` 是 journal 字节，夹的上沿就是一次恢复重放前缀最多那么多条记录（环槽数 ÷ F）的 journal 字节；
/// 产品路径的有效值在小环（6 MiB）与默认环上都取到这一夹，只供测试的来源给的夹之前的值大于它时同样被夹到它；
/// 收下的每次改动报的都是整条记录的字节（4096 的倍数），与阈值逐字节比。
#[test]
fn the_dirty_threshold_is_clamped_by_the_ring_in_journal_bytes_like_the_bytes_of_the_pending_changes(
) {
    for journal_ring_bytes in [
        parameters_of_the_384_slot_small_devices()
            .geometry
            .journal_ring_bytes,
        JOURNAL_RING_DEFAULT_BYTES,
    ] {
        let clamp_in_journal_bytes =
            JournalBytesToWrite::of_records(journal_in_flight_record_limit(journal_ring_bytes));
        assert_eq!(
            PublishTriggerThresholds::of_the_mount(journal_ring_bytes).dirty_threshold(),
            clamp_in_journal_bytes,
            "环长 {journal_ring_bytes}：产品路径的 T_dirty 2 GiB 夹成在飞上限那么多条记录的 journal 字节"
        );
        assert_eq!(
            PublishTriggerThresholds::of_test_only_values_clamped_by_the_journal_ring(
                Duration::from_millis(1),
                u64::MAX,
                journal_ring_bytes,
            )
            .dirty_threshold(),
            clamp_in_journal_bytes,
            "只供测试的来源给夹之前的值，照样夹"
        );
    }
    let (mut pool, mut session) = batching_pool(
        TIME_THRESHOLD_NEVER_REACHED,
        DIRTY_THRESHOLD_NEVER_REACHED_BEFORE_THE_CLAMP,
    );
    let (_, accepted) =
        accept_an_overwrite_of_the_first_file(&mut pool, &mut session, 1, reading(0));
    let journal_bytes = kept_pending_without_a_publish(&accepted.expect("收下"));
    assert!(
        journal_bytes.0 > 0 && journal_bytes.0 % JOURNAL_RECORD_BYTES == 0,
        "收下的改动报的是整条记录的 journal 字节，不是数据字节：{journal_bytes:?}"
    );
}

/// 批内同一个文件覆盖写 K 次照 K 次的记录计（攻方 P2-丙；D16（发布语义） 已定项 5：不折叠）：脏量压到三次覆盖写的 journal 字节，同一个文件覆盖写三次，
/// 每次报的字节相同、合计三倍，第三次收下之后按脏量发；发出去的那一版只写最后那一次的内容（折叠的是发布的内容，不是脏量）。
#[test]
fn overwriting_the_same_file_three_times_in_one_batch_counts_three_overwrites_of_journal_bytes() {
    let (mut probe_pool, mut probe_session) = batching_pool(
        TIME_THRESHOLD_NEVER_REACHED,
        DIRTY_THRESHOLD_NEVER_REACHED_BEFORE_THE_CLAMP,
    );
    let mut reported = Vec::new();
    for seed in 1..=3 {
        let (_, accepted) = accept_an_overwrite_of_the_first_file(
            &mut probe_pool,
            &mut probe_session,
            seed,
            reading(0),
        );
        reported.push(kept_pending_without_a_publish(&accepted.expect("收下")));
    }
    assert!(
        reported.iter().all(|bytes| *bytes == reported[0]),
        "同一个文件三次覆盖写各报一样多：{reported:?}"
    );
    assert_eq!(
        probe_session.journal_bytes_of_the_pending_changes(),
        JournalBytesToWrite(3 * reported[0].0),
        "三次相加，不折叠成一次"
    );
    let (mut pool, mut session) = batching_pool(TIME_THRESHOLD_NEVER_REACHED, 3 * reported[0].0);
    for seed in 1..=2 {
        let (_, accepted) =
            accept_an_overwrite_of_the_first_file(&mut pool, &mut session, seed, reading(0));
        kept_pending_without_a_publish(&accepted.expect("收下"));
    }
    let (content, accepted) =
        accept_an_overwrite_of_the_first_file(&mut pool, &mut session, 3, reading(0));
    let accepted = accepted.expect("第三次收下");
    assert_eq!(
        accepted.publishes.len(),
        1,
        "三次覆盖写的 journal 字节到了阈值：发"
    );
    assert_eq!(
        accepted.publishes[0].reason,
        ReasonOfThePublish::TriggeredAfterTheAccept(PublishTriggerCause::DirtyThreshold)
    );
    assert_eq!(first_file_read_back(&pool), content, "读出最后那一次的内容");
}

/// 产品路径的有效值（D16（发布语义） 已定项 5）：不经只供测试的来源接上的会话，`T_time` 5000 毫秒，`T_dirty` = min(2 GiB, 环长 ÷ F)（6 MiB 的环是 2 MiB）。
#[test]
fn a_session_of_the_mount_takes_five_seconds_and_the_ring_clamp_as_its_thresholds() {
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        plain_device,
    );
    let journal_ring_bytes = pool.parameters.geometry.journal_ring_bytes;
    let session =
        MountedSessionWithPendingChanges::of_the_session(pool.session.take().expect("有会话"));
    assert_eq!(
        session.thresholds().time_threshold(),
        Duration::from_millis(5000),
        "T_time 默认 5 s"
    );
    assert_eq!(
        session.thresholds().dirty_threshold(),
        JournalBytesToWrite((2_u64 << 30).min(journal_ring_bytes / 3)),
        "T_dirty = min(2 GiB, 环长 ÷ 3)"
    );
}

/// 一块内存盘：照转给里面那块稀疏盘，另记读写调用了几次；`fail_writes` 设着时写报 I/O 错、不落盘。
struct DeviceFailingWritesOnRequest {
    inner: SparseBlockDevice,
    fail_writes: Rc<Cell<bool>>,
    calls: Rc<Cell<u64>>,
}

impl DeviceFailingWritesOnRequest {
    fn note_a_call(&self) {
        self.calls.set(self.calls.get() + 1);
    }
}

impl BlockDevice for DeviceFailingWritesOnRequest {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.note_a_call();
        self.inner.read_at(offset, buffer)
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.note_a_call();
        if self.fail_writes.get() {
            return Err(BlockDeviceError::InputOutput(std::io::Error::other(
                "用例设的写失败",
            )));
        }
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.note_a_call();
        self.inner.write_zeroes_at(offset, length)
    }

    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        self.note_a_call();
        self.inner.discard_at(offset, length)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.note_a_call();
        self.inner.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }

    fn journal_record_offsets_hint(
        &self,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        self.inner
            .journal_record_offsets_hint(ring_start, ring_bytes)
    }
}

impl DeviceOverASparseImage for DeviceFailingWritesOnRequest {
    fn sparse_image(&self) -> &SparseDevice {
        &self.inner.image
    }
}

/// 发布写失败冻结之后 `accept` 在任何读写之前拒（判决第二节「串行下不立的」那一格；攻方 P7-乙：冻结期间照常收下，改动就攒在发不出去的那一批里）：
/// 收下一次覆盖写，让写报错，立即排空那一次没发成、冻结在分配器上，待发并进了冻结的那一次、清空；之后再交一次覆盖写，拒成「冻结着没重发」，
/// 盘一次读写都没调、待发没长。
#[test]
fn an_accept_after_a_publish_frozen_by_a_write_failure_is_refused_before_any_read_or_write() {
    let fail_writes = Rc::new(Cell::new(false));
    let calls = Rc::new(Cell::new(0_u64));
    let (mut pool, mut session) = batching_pool_on(
        TIME_THRESHOLD_NEVER_REACHED,
        DIRTY_THRESHOLD_NEVER_REACHED_BEFORE_THE_CLAMP,
        |_identity, inner| DeviceFailingWritesOnRequest {
            inner,
            fail_writes: Rc::clone(&fail_writes),
            calls: Rc::clone(&calls),
        },
    );
    let (_, accepted) =
        accept_an_overwrite_of_the_first_file(&mut pool, &mut session, 1, reading(0));
    kept_pending_without_a_publish(&accepted.expect("收下"));
    fail_writes.set(true);
    session
        .publish_pending_now(&mut pool.devices, reading(1))
        .expect_err("写报错：排空那一次没发成");
    assert!(
        session.session().allocator.frozen_publish().is_some(),
        "落盘途中写失败：那一次冻结在分配器上"
    );
    assert_eq!(
        session.pending_changes_accepted(),
        0,
        "待发并进了冻结的那一次（D16（发布语义） 已定项 17），不留第二份"
    );
    let calls_before_the_accept = calls.get();
    let (_, refused) =
        accept_an_overwrite_of_the_first_file(&mut pool, &mut session, 2, reading(2));
    assert!(
        matches!(
            refused,
            Err(AcceptRefused::PublishFrozenAfterAWriteFailureIsNotResentYet { .. })
        ),
        "冻结着：收下也拒，{refused:?}"
    );
    assert_eq!(
        calls.get(),
        calls_before_the_accept,
        "在任何读写之前拒：盘一次都没调"
    );
    assert_eq!(session.pending_changes_accepted(), 0, "冻结期间待发不长");
}
