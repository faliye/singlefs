//! `accept` 被空间拒时当场推抬 F 再判（D16（发布语义） 已定项 1「准入」那一行）的几格，核心层的攒批会话上判：
//! 待发里已收下的改动不丢、不重复发（推成、抬 F 写盘报错冻结、排空时抬 F 写盘报错冻结）；推到不再推了还放不下、真满没有可放的报拒不崩，拒绝里带推了几串，盘上状态挂得上、读得出；推抬 F 全放出来也放不下的不推（短在式子那一判、短在段外槽对那一判而三样全放开之后真成的对数也不够、推串之前全空聚簇段已少于 `N₀` 三形；全空聚簇段那一形经 `accept` 与经会话单发各判一次），拒绝里带这次调用已落盘的；
//! 走空那一步被空间拒时合取 7 免，全空聚簇段少于 `N₀` 也照推。
//! 真写满的池上的那一族与运行库一层在 `crates/singlefs-mount-runtime/tests/mount_runtime_accept_raises_the_floor_for_space.rs`。
mod common_admission_pool;

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use common_admission_pool::{
    content_of, parameters_of_the_384_slot_small_devices, DeviceOverASparseImage, PoolUnderTest,
    FIRST_WRITE_TIME_SECONDS, OVERWRITE_BYTES,
};
use singlefs_core::address::{
    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, InodeNumber,
    SlotNumber,
};
use singlefs_core::admission::{
    AvailableBytesOnOneDevice, BytesOnOneDevice, DeviceWithEmptyClusterSegmentsBelowTheFloor,
    EmptyClusterSegmentsBelowTheFloorOnSomeDevices, JudgementThatFoundTheDeviceShort,
};
use singlefs_core::allocator::{DeviceFreeMap, PoolAllocator, UnitAreaStart};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::compaction_plan::{
    CallerMonotonicClockReading, CompactionSwitches, EmptyClusterSegmentsFloor,
    EMPTY_CLUSTER_SEGMENTS_FLOOR_OF_THE_FIRST_VERSION,
};
use singlefs_core::mount::{FloorRaiseStop, PUBLISHES_PER_ADMISSION_AT_MOST};
use singlefs_core::mounted_session::{
    refusal_is_short_of_space, AcceptRefused, BatchJudgedAtTheAccept,
    DeviceShortEvenWithEverythingAFloorRaiseCouldRelease, HowTheChangeWasTaken, MountedSession,
    MountedSessionWithPendingChanges, NoSpaceEvenWithEverythingAFloorRaiseCouldRelease,
    ReasonOfThePublish, UserChange, UserChangeRefused,
};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::system_configuration::PublishTriggerThresholds;
use singlefs_core::transaction::{
    journal_bytes_of_data_changes_published_together_judged_before_any_read_or_write,
    DataChangesPublishedTogether, FirstFile, KindOfTheFrozenPublish, NewInodesOfOneChange,
    PoolWriter, PublishError, TransactionUnit, WholeWriteOfOneInode, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::{CLUSTER_SEGMENT_SLOTS, INODE_LEAF_RECORDS, SLOT_BYTES};
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};

/// 时间那一支不该先到：一小时。
const TIME_THRESHOLD_NEVER_REACHED: Duration = Duration::from_secs(3600);
/// 脏量那一支不该先到（夹之前的 `T_dirty`，夹成环长 ÷ F）。
const DIRTY_THRESHOLD_NEVER_REACHED_BEFORE_THE_CLAMP: u64 = 1 << 40;
/// 压余量的用例单元区的槽数：落点离单元区最高处隔离的那几个槽远。
const UNIT_AREA_SLOTS_OF_THE_LARGER_DEVICES: u64 = 8192;
/// 差额压在隔离上的用例留给抬 F 那一串的没挡的槽：够一次准入的发布落点，又远不够压过环窗口。
const UNBLOCKED_SLOTS_LEFT_FOR_THE_FLOOR_RAISE: u64 = 400;
/// 摆满、写不下的用例单元区的槽数：摆满要逐槽走，写不下的内容按它算。
const UNIT_AREA_SLOTS_OF_THE_SMALLER_DEVICES: u64 = 2048;
/// 放不下而不撞一次发布的日志记录条数上限（512）的那一格：500 个数据单元要 1000 槽，比这里的单元区大。
const UNIT_AREA_SLOTS_SMALLER_THAN_A_BATCH_WITHIN_THE_RECORD_LIMIT: u64 = 768;
const UNITS_OF_A_BATCH_WITHIN_THE_RECORD_LIMIT: usize = 500;
/// 小盘那一档为环窗口补足的槽数（同 `batching_across_requests_and_barriers.rs`）。
const RING_WINDOW_ALLOWANCE_SLOTS: u64 = 1007;
/// 接在第一个文件后面覆盖写几次：比要留的 4 个状态多得多，第 4 新的非空根之前放掉的槽推一串抬 F 就回收得到，够装下那一串自己占的；又不到一圈根环，那些槽还没被环的门槛放开。
const OVERWRITES_BEFORE_THE_ACCEPT: usize = 14;
/// 抬 F 那一串先写系统配置那一步的写数（每块盘一次）：放过它们，下一个写落在第一次空发布上。
const SYSTEM_CONFIGURATION_WRITES_OF_A_FLOOR_RAISE: u64 = 2;
/// 建满一片 inode 叶容器那么多个 inode：比覆盖写多分裂一片，两样单独各自过得了时合起来还差几个槽。
const NEW_INODES_SPLITTING_THE_FIRST_LEAF_CONTAINER: u64 = INODE_LEAF_RECORDS;

/// 用例摆的写故障，两块盘共用一份。
#[derive(Debug, Default)]
struct WriteFaultOfTheTest {
    is_armed: AtomicBool,
    writes_let_through: AtomicU64,
}

impl WriteFaultOfTheTest {
    /// 放过 `writes_let_through` 个写，下一个写报错一次。
    fn arm_after(&self, writes_let_through: u64) {
        self.writes_let_through
            .store(writes_let_through, Ordering::SeqCst);
        self.is_armed.store(true, Ordering::SeqCst);
    }

    fn this_write_fails(&self) -> bool {
        if !self.is_armed.load(Ordering::SeqCst) {
            return false;
        }
        let is_let_through = self
            .writes_let_through
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |left| {
                left.checked_sub(1)
            })
            .is_ok();
        if is_let_through {
            return false;
        }
        self.is_armed.store(false, Ordering::SeqCst);
        true
    }

    fn has_fired(&self) -> bool {
        !self.is_armed.load(Ordering::SeqCst)
    }
}

/// 稀疏盘外面包一层用例摆的写故障：报错时什么都不写。
struct DeviceFailingAWriteOnRequest {
    device: SparseBlockDevice,
    fault: Arc<WriteFaultOfTheTest>,
}

impl BlockDevice for DeviceFailingAWriteOnRequest {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.device.read_at(offset, buffer)
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        if self.fault.this_write_fails() {
            return Err(BlockDeviceError::InputOutput(std::io::Error::other(
                "用例摆的写错",
            )));
        }
        self.device.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.device.write_zeroes_at(offset, length)
    }

    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        self.device.discard_at(offset, length)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.device.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.device.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.device.size_in_bytes()
    }
}

impl DeviceOverASparseImage for DeviceFailingAWriteOnRequest {
    fn sparse_image(&self) -> &SparseDevice {
        &self.device.image
    }
}

/// 一条用例的池：摘下来的会话另放，盘还在池里。
struct PoolWithTheFloorBelowItsCeiling {
    pool: PoolUnderTest<DeviceFailingAWriteOnRequest>,
    session: MountedSession,
    fault: Arc<WriteFaultOfTheTest>,
}

/// 6 MiB 的环之后单元区 `unit_area_slots` 槽的盘的字节数。
fn device_bytes_with_a_unit_area_of(unit_area_slots: u64) -> u64 {
    let unit_area_start_slot = UnitAreaStart::following_the_journal_ring(
        parameters_of_the_384_slot_small_devices()
            .geometry
            .journal_ring_bytes,
    )
    .expect("6 MiB 的环末尾的下一个槽落在聚簇段边界上")
    .slot()
    .0;
    (unit_area_start_slot + RING_WINDOW_ALLOWANCE_SLOTS + unit_area_slots) * SLOT_BYTES
}

/// 起池（第一个文件）、覆盖写 [`OVERWRITES_BEFORE_THE_ACCEPT`] 次，F 还在 0：上限随第 4 新的非空根走，高于 F。
fn pool_with_the_floor_below_its_ceiling(unit_area_slots: u64) -> PoolWithTheFloorBelowItsCeiling {
    let fault = Arc::new(WriteFaultOfTheTest::default());
    let fault_of_the_devices = Arc::clone(&fault);
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_with_a_unit_area_of(unit_area_slots),
        move |_identity: DeviceIdentity, device| DeviceFailingAWriteOnRequest {
            device,
            fault: Arc::clone(&fault_of_the_devices),
        },
    );
    for _overwrite in 0..OVERWRITES_BEFORE_THE_ACCEPT {
        pool.overwrite(OVERWRITE_BYTES)
            .expect("单元区够大：覆盖写发得成");
    }
    let session = pool.session.take().expect("起池之后有会话");
    PoolWithTheFloorBelowItsCeiling {
        pool,
        session,
        fault,
    }
}

fn batching(session: MountedSession, journal_ring_bytes: u64) -> MountedSessionWithPendingChanges {
    MountedSessionWithPendingChanges::of_the_session_with_thresholds(
        session,
        PublishTriggerThresholds::of_test_only_values_clamped_by_the_journal_ring(
            TIME_THRESHOLD_NEVER_REACHED,
            DIRTY_THRESHOLD_NEVER_REACHED_BEFORE_THE_CLAMP,
            journal_ring_bytes,
        ),
    )
}

fn reading_zero() -> CallerMonotonicClockReading {
    CallerMonotonicClockReading(Duration::ZERO)
}

fn first_file_read_back(image: &MemoryPool) -> Vec<u8> {
    match recover(image, JournalPolicy::Consult).outcome {
        RecoveryOutcome::FileRead { content, .. } => content,
        other @ (RecoveryOutcome::NoFile { .. } | RecoveryOutcome::Failed { .. }) => {
            panic!("第一个文件之后恢复读得出它：{other:?}")
        }
    }
}

fn the_publish_judgement_admits(
    pool: &mut PoolUnderTest<DeviceFailingAWriteOnRequest>,
    session: &MountedSession,
    allocator: &PoolAllocator,
    batch: DataChangesPublishedTogether<'_>,
) -> bool {
    let parameters = session.parameters_and_device_table.parameters_on_disk();
    let writer = PoolWriter::new(parameters, pool.devices.as_mut_slice());
    journal_bytes_of_data_changes_published_together_judged_before_any_read_or_write(
        &writer,
        allocator,
        session.current.version_of_trees().expect("有树"),
        batch,
        session.instance,
    )
    .is_ok()
}

/// 分配器的一份拷贝，每块盘单元区最高处 `slots` 个槽记成隔离（只动隔离位，不写盘）。
fn allocator_with_the_highest_slots_isolated(
    allocator: &PoolAllocator,
    slots: u64,
) -> PoolAllocator {
    let mut isolated = allocator.clone();
    if slots == 0 {
        return isolated;
    }
    let ends: Vec<(DeviceIdentity, u64)> = isolated
        .devices
        .iter()
        .map(|device_map| {
            (
                device_map.device,
                device_map.unit_area_start().slot().0 + device_map.unit_area_slots(),
            )
        })
        .collect();
    for (device, unit_area_end) in ends {
        isolated.isolate_abandoned(device, SlotNumber(unit_area_end - slots), slots);
    }
    isolated
}

/// 覆盖写第一个文件的内容与要并进去的建 inode。
struct ChangesOfTheCase {
    content: Vec<u8>,
    new_inodes: [NewInodesOfOneChange; 1],
}

fn changes_of_the_case() -> ChangesOfTheCase {
    ChangesOfTheCase {
        content: content_of(OVERWRITE_BYTES, 1701),
        new_inodes: [NewInodesOfOneChange {
            count: NEW_INODES_SPLITTING_THE_FIRST_LEAF_CONTAINER,
            write_time_seconds: FIRST_WRITE_TIME_SECONDS + 1701,
        }],
    }
}

fn overwrite_of_the_case(changes: &ChangesOfTheCase) -> UserChange<'_> {
    UserChange::Overwrite(FirstFile {
        content: &changes.content,
        write_time_seconds: FIRST_WRITE_TIME_SECONDS + 1702,
    })
}

fn new_inodes_of_the_case(changes: &ChangesOfTheCase) -> UserChange<'_> {
    UserChange::NewInodes {
        count: changes.new_inodes[0].count,
        write_time_seconds: changes.new_inodes[0].write_time_seconds,
    }
}

/// 二分隔离的槽数，交回「覆盖写并上建 inode」这一批刚好被判拒、两样单独各自过得了的那一份分配器（做法同 `batching_across_requests_and_barriers.rs` 的
/// `a_change_that_fits_alone_but_not_together_with_the_pending_changes_is_refused_at_the_accept`）。
fn allocator_refusing_only_the_changes_together(
    case: &mut PoolWithTheFloorBelowItsCeiling,
    changes: &ChangesOfTheCase,
) -> PoolAllocator {
    let whole_write = WholeWriteOfOneInode {
        inode: InodeNumber(FIRST_INODE_NUMBER),
        content: &changes.content,
        write_time_seconds: FIRST_WRITE_TIME_SECONDS + 1702,
    };
    let overwrite_alone = DataChangesPublishedTogether {
        whole_write: Some(whole_write),
        new_inodes: &[],
    };
    let new_inodes_alone = DataChangesPublishedTogether {
        whole_write: None,
        new_inodes: &changes.new_inodes,
    };
    let together = DataChangesPublishedTogether {
        whole_write: Some(whole_write),
        new_inodes: &changes.new_inodes,
    };
    let allocator = case.session.allocator.clone();
    assert!(
        the_publish_judgement_admits(&mut case.pool, &case.session, &allocator, together),
        "不压余量时合起来那一批过得了"
    );
    let (mut admitted_with, mut refused_with) = (0_u64, allocator.devices[0].unit_area_slots());
    // 迭代上界是 log2(单元区槽数) 轮；跨轮携带的是一过一不过的两端（夹紧不放松）。可用随隔离单调减，判定随可用单调。
    while refused_with - admitted_with > 1 {
        let middle = admitted_with + (refused_with - admitted_with) / 2;
        let isolated = allocator_with_the_highest_slots_isolated(&allocator, middle);
        if the_publish_judgement_admits(&mut case.pool, &case.session, &isolated, together) {
            admitted_with = middle;
        } else {
            refused_with = middle;
        }
    }
    let isolated = allocator_with_the_highest_slots_isolated(&allocator, refused_with);
    assert!(
        the_publish_judgement_admits(&mut case.pool, &case.session, &isolated, overwrite_alone)
            && the_publish_judgement_admits(
                &mut case.pool,
                &case.session,
                &isolated,
                new_inodes_alone
            ),
        "隔离 {refused_with} 个槽：合起来刚好过不了，两样单独各自过得了"
    );
    isolated
}

/// 待发里的改动发得成、只发一次：排空之后第一个文件读出收下的内容、水位多建的那几个，再排空什么都不发。
fn the_pending_changes_are_published_once<Device: DeviceOverASparseImage>(
    pool: &mut PoolUnderTest<Device>,
    session: &mut MountedSessionWithPendingChanges,
    content_accepted: &[u8],
    inodes_expected_in_the_batch: u64,
    inode_watermark_before: u64,
) {
    let published = session
        .publish_pending_now(&mut pool.devices, reading_zero())
        .unwrap_or_else(|refused| panic!("收下的发得成：{refused:?}"))
        .expect("待发不空：发了一次");
    assert_eq!(
        published.reason,
        ReasonOfThePublish::PendingChangesPublishedOnRequest
    );
    assert_eq!(
        published.published.versions_published.len(),
        1,
        "待发并成一次发布"
    );
    assert_eq!(session.pending_changes_accepted(), 0, "排空之后待发空");
    assert_eq!(
        first_file_read_back(&pool.image()),
        content_accepted,
        "收下的覆盖写没丢"
    );
    assert_eq!(
        session
            .session()
            .current
            .version_of_trees()
            .expect("有树")
            .inode_number_watermark(),
        inode_watermark_before + inodes_expected_in_the_batch,
        "收下的建 inode 没丢、也没发两遍"
    );
    let image_after_the_publish = pool.image();
    assert!(
        session
            .publish_pending_now(&mut pool.devices, reading_zero())
            .unwrap_or_else(|refused| panic!("待发空：不发也不拒：{refused:?}"))
            .is_none(),
        "待发空：再排空什么都不发"
    );
    assert!(
        pool.image() == image_after_the_publish,
        "再排空一个写都不发：不重复发"
    );
}

fn inode_watermark_of(session: &MountedSessionWithPendingChanges) -> u64 {
    session
        .session()
        .current
        .version_of_trees()
        .expect("有树")
        .inode_number_watermark()
}

/// 待发里有一次覆盖写，再交建 inode：单独过得了，并进待发合起来被空间拒，当场推抬 F 再判、收下。推的空发布不含待发（盘上第一个文件还是旧内容），待发两次都留着，
/// 排空发成一次，之后再排空什么都不发。
#[test]
fn an_accept_refused_for_space_together_with_pending_changes_raises_the_floor_and_keeps_them() {
    let mut case = pool_with_the_floor_below_its_ceiling(UNIT_AREA_SLOTS_OF_THE_LARGER_DEVICES);
    let changes = changes_of_the_case();
    let content_before = case.pool.content_of_the_current_version.clone();
    let floor_before = case.session.current.root().rollback_floor;
    let isolated = allocator_refusing_only_the_changes_together(&mut case, &changes);
    case.session.allocator = isolated;
    let PoolWithTheFloorBelowItsCeiling {
        mut pool,
        session,
        fault: _,
    } = case;
    let journal_ring_bytes = pool.parameters.geometry.journal_ring_bytes;
    let mut session = batching(session, journal_ring_bytes);
    let inode_watermark_before = inode_watermark_of(&session);
    let overwrite_accepted = session
        .accept(
            &mut pool.devices,
            overwrite_of_the_case(&changes),
            reading_zero(),
        )
        .expect("覆盖写单独过得了：收下");
    assert!(
        overwrite_accepted
            .floor_raises_pushed_for_space
            .floor_raises
            .is_empty(),
        "覆盖写过得了：不推"
    );
    let accepted = session
        .accept(
            &mut pool.devices,
            new_inodes_of_the_case(&changes),
            reading_zero(),
        )
        .unwrap_or_else(|refused| panic!("推了抬 F 回收得到：收下：{refused:?}"));
    assert!(
        matches!(accepted.taken, HowTheChangeWasTaken::KeptPending { .. }),
        "收进待发：{:?}",
        accepted.taken
    );
    assert!(accepted.publishes.is_empty(), "两支都没到：待发不发");
    let raised = &accepted.floor_raises_pushed_for_space;
    assert!(!raised.floor_raises.is_empty(), "被空间拒：推了抬 F");
    assert_eq!(
        raised.floor_raises.len(),
        raised.refusals_that_pushed_the_floor_raises.len()
    );
    assert!(
        raised
            .refusals_that_pushed_the_floor_raises
            .iter()
            .all(refusal_is_short_of_space),
        "推它的是空间不够：{:?}",
        raised.refusals_that_pushed_the_floor_raises
    );
    assert!(
        session.session().current.root().rollback_floor > floor_before,
        "F 抬上去了"
    );
    assert_eq!(
        session.pending_changes_accepted(),
        2,
        "收下的两次都在待发里"
    );
    assert_eq!(
        first_file_read_back(&pool.image()),
        content_before,
        "空发布不含待发：盘上还是旧内容"
    );
    the_pending_changes_are_published_once(
        &mut pool,
        &mut session,
        &changes.content,
        changes.new_inodes[0].count,
        inode_watermark_before,
    );
}

/// 同上一条，抬 F 的第一次空发布写盘报错：拒成抬 F 自己报错、那一次空发布冻结（不是用户改动那一种），待发里的覆盖写留着。冻结着排空被拒照旧留着；
/// 重发成了发得成，只发一次。
#[test]
fn a_write_failure_in_the_floor_raise_of_an_accept_keeps_the_pending_changes_until_the_resend() {
    let mut case = pool_with_the_floor_below_its_ceiling(UNIT_AREA_SLOTS_OF_THE_LARGER_DEVICES);
    let changes = changes_of_the_case();
    let isolated = allocator_refusing_only_the_changes_together(&mut case, &changes);
    case.session.allocator = isolated;
    let PoolWithTheFloorBelowItsCeiling {
        mut pool,
        session,
        fault,
    } = case;
    let journal_ring_bytes = pool.parameters.geometry.journal_ring_bytes;
    let mut session = batching(session, journal_ring_bytes);
    let inode_watermark_before = inode_watermark_of(&session);
    session
        .accept(
            &mut pool.devices,
            overwrite_of_the_case(&changes),
            reading_zero(),
        )
        .expect("覆盖写单独过得了：收下");
    fault.arm_after(SYSTEM_CONFIGURATION_WRITES_OF_A_FLOOR_RAISE);
    let refused = session.accept(
        &mut pool.devices,
        new_inodes_of_the_case(&changes),
        reading_zero(),
    );
    assert!(fault.has_fired(), "故障打在了这一串抬 F 上");
    match &refused {
        Err(AcceptRefused::RefusedAfterRaisingTheFloorForSpace {
            judgement: BatchJudgedAtTheAccept::PendingChangesTogetherWithThisChange,
            refusal,
        }) => assert!(
            matches!(
                **refusal,
                UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(_)
            ),
            "抬 F 自己报的错原样：{refusal:?}"
        ),
        other => panic!("为并进待发那一批推的抬 F 写盘报错：{other:?}"),
    }
    assert_eq!(
        session
            .session()
            .allocator
            .frozen_publish()
            .map(|frozen| frozen.kind()),
        Some(KindOfTheFrozenPublish::NeitherOfTheCountedKinds),
        "冻结的是那一次空发布，不是用户改动"
    );
    assert_eq!(session.pending_changes_accepted(), 1, "待发里的覆盖写留着");
    let refused_while_frozen = session.publish_user_change(
        &mut pool.devices,
        new_inodes_of_the_case(&changes),
        reading_zero(),
    );
    assert!(refused_while_frozen.is_err(), "冻结着：排空被拒");
    assert_eq!(
        session.pending_changes_accepted(),
        1,
        "冻结的那一次不含待发：排空被拒之后放回"
    );
    assert!(
        session
            .resend_the_frozen_publish(&mut pool.devices)
            .expect("故障只报一次：重发做得成"),
        "冻结着一次发布"
    );
    assert_eq!(
        session.pending_changes_accepted(),
        1,
        "重发的是空发布：待发还在"
    );
    the_pending_changes_are_published_once(
        &mut pool,
        &mut session,
        &changes.content,
        0,
        inode_watermark_before,
    );
}

/// 排空待发时被空间拒、推的抬 F 写盘报错：冻结的是那一次空发布，待发的两次都留着（这一批没进冻结的那一次）；重发成了发得成，只发一次。
#[test]
fn a_write_failure_in_the_floor_raise_of_a_drain_keeps_the_pending_changes_until_the_resend() {
    let mut case = pool_with_the_floor_below_its_ceiling(UNIT_AREA_SLOTS_OF_THE_LARGER_DEVICES);
    let changes = changes_of_the_case();
    let isolated = allocator_refusing_only_the_changes_together(&mut case, &changes);
    let PoolWithTheFloorBelowItsCeiling {
        mut pool,
        session,
        fault,
    } = case;
    let journal_ring_bytes = pool.parameters.geometry.journal_ring_bytes;
    let mut session = batching(session, journal_ring_bytes);
    let inode_watermark_before = inode_watermark_of(&session);
    for change in [
        overwrite_of_the_case(&changes),
        new_inodes_of_the_case(&changes),
    ] {
        session
            .accept(&mut pool.devices, change, reading_zero())
            .expect("没压余量：收下");
    }
    session.session_for_tests_only().allocator = isolated;
    fault.arm_after(SYSTEM_CONFIGURATION_WRITES_OF_A_FLOOR_RAISE);
    let refused = session
        .publish_pending_now(&mut pool.devices, reading_zero())
        .expect_err("排空被空间拒、推的抬 F 写盘报错");
    assert!(fault.has_fired(), "故障打在了这一串抬 F 上");
    assert!(
        matches!(
            *refused.refusal,
            UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(_)
        ),
        "抬 F 自己报的错原样：{:?}",
        refused.refusal
    );
    assert_eq!(
        session
            .session()
            .allocator
            .frozen_publish()
            .map(|frozen| frozen.kind()),
        Some(KindOfTheFrozenPublish::NeitherOfTheCountedKinds),
        "冻结的是那一次空发布，不是这一批"
    );
    assert_eq!(
        session.pending_changes_accepted(),
        2,
        "这一批没进冻结的那一次：放回"
    );
    assert!(
        session
            .resend_the_frozen_publish(&mut pool.devices)
            .expect("故障只报一次：重发做得成"),
        "冻结着一次发布"
    );
    the_pending_changes_are_published_once(
        &mut pool,
        &mut session,
        &changes.content,
        changes.new_inodes[0].count,
        inode_watermark_before,
    );
}

/// 每块盘单元区里还空闲的槽都记成已分配（盘上不动）。
fn mark_every_free_slot_allocated(session: &mut MountedSession) {
    for device_map in &mut session.allocator.devices {
        let start = device_map.unit_area_start().slot().0;
        let slots = device_map.unit_area_slots();
        // 迭代上界是单元区槽数，跨轮只改这一槽的分配位。
        for slot in start..start + slots {
            if device_map.is_free(SlotNumber(slot)) {
                device_map.mark_allocated(SlotNumber(slot), 1);
            }
        }
    }
}

/// 真满没有可放的（`mark_allocated` 摆满，F 还抬得动而抬 F 那一串自己放不下）：报拒不崩，一串都没推成、盘上逐字节不变、待发空，崩了再挂载读得出旧内容。
#[test]
fn an_accept_on_a_pool_with_no_slot_left_is_refused_without_a_panic_and_the_disk_untouched() {
    let PoolWithTheFloorBelowItsCeiling {
        mut pool,
        mut session,
        fault: _,
    } = pool_with_the_floor_below_its_ceiling(UNIT_AREA_SLOTS_OF_THE_SMALLER_DEVICES);
    mark_every_free_slot_allocated(&mut session);
    let journal_ring_bytes = pool.parameters.geometry.journal_ring_bytes;
    let mut session = batching(session, journal_ring_bytes);
    let changes = changes_of_the_case();
    let image_before = pool.image();
    let refused = session
        .accept(
            &mut pool.devices,
            overwrite_of_the_case(&changes),
            reading_zero(),
        )
        .expect_err("一个槽都没有：收不下");
    let AcceptRefused::RefusedAfterRaisingTheFloorForSpace {
        judgement: BatchJudgedAtTheAccept::ThisChangeAlone,
        refusal,
    } = &refused
    else {
        panic!("判的是这一次单独那一批：{refused:?}");
    };
    assert!(
        matches!(**refusal, UserChangeRefused::NoSpaceAfterRaisingTheFloor(_)),
        "推到不再推了：报空间不够，实际 {refusal:?}"
    );
    assert!(
        refused.floor_raises_pushed_by_this_accept().is_empty(),
        "抬 F 那一串自己放不下：一串都没推成"
    );
    assert!(pool.image() == image_before, "盘上逐字节不变");
    assert_eq!(session.pending_changes_accepted(), 0, "待发空");
    drop(session);
    let content_before = pool.content_of_the_current_version.clone();
    pool.crash_and_mount_writable()
        .expect("崩了再可写挂载做得成：摆满只在内存里");
    assert_eq!(
        first_file_read_back(&pool.image()),
        content_before,
        "再挂载读得出旧内容"
    );
}

/// 内容比单元区还大：推抬 F 全放出来也放不下，一串都不推就拒，上界照可用、defer 与隔离的字节算，盘上逐字节不变、待发空，崩了再挂载读得出旧内容。
#[test]
fn an_accept_larger_than_everything_a_floor_raise_could_release_is_refused_without_a_series() {
    let PoolWithTheFloorBelowItsCeiling {
        mut pool,
        session,
        fault: _,
    } = pool_with_the_floor_below_its_ceiling(
        UNIT_AREA_SLOTS_SMALLER_THAN_A_BATCH_WITHIN_THE_RECORD_LIMIT,
    );
    let floor_before = session.current.root().rollback_floor;
    let journal_ring_bytes = pool.parameters.geometry.journal_ring_bytes;
    let mut session = batching(session, journal_ring_bytes);
    let content =
        vec![0x5A_u8; UNITS_OF_A_BATCH_WITHIN_THE_RECORD_LIMIT * data_unit_payload_capacity()];
    let image_before = pool.image();
    let refused = session
        .accept(
            &mut pool.devices,
            UserChange::SequentialWrite(FirstFile {
                content: &content,
                write_time_seconds: FIRST_WRITE_TIME_SECONDS + 1703,
            }),
            reading_zero(),
        )
        .expect_err("内容比单元区还大：收不下");
    let AcceptRefused::RefusedAfterRaisingTheFloorForSpace {
        judgement: BatchJudgedAtTheAccept::ThisChangeAlone,
        refusal,
    } = &refused
    else {
        panic!("判的是这一次单独那一批：{refused:?}");
    };
    let UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(no_space) = &**refusal
    else {
        panic!("全放出来也放不下：不推，实际 {refusal:?}");
    };
    let PublishError::SpaceAdmissionRefused(admission_refusal) = &no_space.last_refusal else {
        panic!("判出这几块盘的是准入被拒：{:?}", no_space.last_refusal);
    };
    assert_eq!(
        no_space.devices_short.len(),
        pool.devices.len(),
        "两块盘都放不下"
    );
    for device_short in &no_space.devices_short {
        let short = admission_refusal
            .short_devices
            .iter()
            .find(|short| short.device == device_short.device)
            .expect("点名的盘在拒绝里");
        assert_eq!(
            short.judged_short_by,
            JudgementThatFoundTheDeviceShort::FormulaAvailableAgainstDemand,
            "段外的槽对装得下 500 个单元，短在式子那一判：{short:?}"
        );
        let device_map = session
            .session()
            .allocator
            .devices
            .iter()
            .find(|device_map| device_map.device == device_short.device)
            .expect("点名的盘在池里");
        assert_eq!(
            device_short.most_a_floor_raise_could_make_available.0,
            short.available.0
                + i128::from(device_map.deferred_slots() + device_map.isolated_slots())
                    * i128::from(SLOT_BYTES),
            "上界是这块盘的可用加 defer 与隔离的字节"
        );
        assert!(
            device_map.deferred_slots() > 0,
            "覆盖写过：defer 里有槽，上面那一格才判得出它算没算"
        );
        assert_eq!(device_short.demand, short.demand, "需求原样");
    }
    assert!(
        refused.floor_raises_pushed_by_this_accept().is_empty()
            && no_space.versions_published.is_empty()
            && no_space.refusals_that_pushed_the_floor_raises.is_empty(),
        "头一次被拒就判到这里：一串都没推、一版都没发"
    );
    assert_eq!(
        session.session().current.root().rollback_floor,
        floor_before,
        "F 没动"
    );
    assert!(pool.image() == image_before, "盘上逐字节不变");
    assert_eq!(session.pending_changes_accepted(), 0, "待发空");
    drop(session);
    let content_before = pool.content_of_the_current_version.clone();
    pool.crash_and_mount_writable()
        .expect("崩了再可写挂载做得成：这一次没动盘");
    assert_eq!(
        first_file_read_back(&pool.image()),
        content_before,
        "再挂载读得出旧内容"
    );
}

/// 每块盘从单元区最高处往下把没挡的槽记成隔离（只动隔离位，不写盘），留下 `unblocked_slots_left` 个没挡的槽给抬 F 那一串落点。
fn isolate_the_highest_unblocked_slots_leaving(
    session: &mut MountedSession,
    unblocked_slots_left: u64,
) {
    for device_map in &mut session.allocator.devices {
        let start = device_map.unit_area_start().slot().0;
        let slots = device_map.unit_area_slots();
        // 迭代上界是单元区槽数；提前出口：没挡的槽只剩这么多。
        for slot in (start..start + slots).rev() {
            if device_map.unblocked_slots() <= unblocked_slots_left {
                break;
            }
            if !device_map.is_blocked_for_commit_generated(SlotNumber(slot)) {
                device_map.isolate(SlotNumber(slot), 1);
            }
        }
    }
}

/// 差额压在隔离的槽上：可用加 defer 与隔离够得上需求，所以照推；隔离这一串解不开，推到不再推了还放不下，拒绝里带推成的串，待发空；
/// 推的数在一次准入的预算之内，崩了再挂载读得出旧内容。判别力：上界不算隔离的字节时这一条判成一串都不推。
#[test]
fn an_accept_short_only_behind_isolated_slots_pushes_a_series_and_reports_it_in_the_refusal() {
    let PoolWithTheFloorBelowItsCeiling {
        mut pool,
        mut session,
        fault: _,
    } = pool_with_the_floor_below_its_ceiling(UNIT_AREA_SLOTS_OF_THE_SMALLER_DEVICES);
    isolate_the_highest_unblocked_slots_leaving(
        &mut session,
        UNBLOCKED_SLOTS_LEFT_FOR_THE_FLOOR_RAISE,
    );
    let floor_before = session.current.root().rollback_floor;
    let journal_ring_bytes = pool.parameters.geometry.journal_ring_bytes;
    let mut session = batching(session, journal_ring_bytes);
    let changes = changes_of_the_case();
    let refused = session
        .accept(
            &mut pool.devices,
            overwrite_of_the_case(&changes),
            reading_zero(),
        )
        .expect_err("隔离这一串解不开：收不下");
    let AcceptRefused::RefusedAfterRaisingTheFloorForSpace {
        judgement: BatchJudgedAtTheAccept::ThisChangeAlone,
        refusal,
    } = &refused
    else {
        panic!("判的是这一次单独那一批：{refused:?}");
    };
    let UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space) = &**refusal else {
        panic!("推到不再推了：报空间不够，实际 {refusal:?}");
    };
    assert!(
        matches!(
            no_space.last_refusal,
            PublishError::SpaceAdmissionRefused(_) | PublishError::PlacementRefused { .. }
        ),
        "最后一次被拒是空间不够：{:?}",
        no_space.last_refusal
    );
    assert!(
        matches!(
            no_space.stop,
            FloorRaiseStop::FloorAlreadyAtTheCeiling { .. }
                | FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded { .. }
        ),
        "停在上限或预算上：{:?}",
        no_space.stop
    );
    let series = refused.floor_raises_pushed_by_this_accept();
    assert!(!series.is_empty(), "F 抬得动：推过");
    assert_eq!(
        series.len(),
        no_space.floor_raises.len(),
        "这个入口报的串数就是拒绝里带的那几串"
    );
    let empty_publishes: usize = series.iter().map(|raised| raised.publishes.len()).sum();
    assert!(
        empty_publishes < PUBLISHES_PER_ADMISSION_AT_MOST,
        "推的空发布不超过一次准入的预算 B − 1：{empty_publishes}"
    );
    assert!(
        session.session().current.root().rollback_floor > floor_before,
        "F 抬上去了"
    );
    assert_eq!(session.pending_changes_accepted(), 0, "待发空");
    drop(session);
    let content_before = pool.content_of_the_current_version.clone();
    pool.crash_and_mount_writable()
        .expect("崩了再可写挂载做得成");
    assert_eq!(
        first_file_read_back(&pool.image()),
        content_before,
        "再挂载读得出旧内容"
    );
}

/// 删除队列留一个没走完的 unlink：建一个 inode、给它写一个单元，unlink 意图写盘报错冻结，重发成了之后意图落盘、它的批还没走。
/// 三次改动的写时间从 `write_time_seconds` 起逐次加一秒。
fn leave_an_unlink_whose_batches_have_not_walked(
    pool: &mut PoolUnderTest<DeviceFailingAWriteOnRequest>,
    session: &mut MountedSessionWithPendingChanges,
    fault: &WriteFaultOfTheTest,
    write_time_seconds: u64,
) {
    let inode_to_unlink = InodeNumber(inode_watermark_of(session));
    let content_of_the_unlinked_file = vec![0x3C_u8; data_unit_payload_capacity()];
    for change in [
        UserChange::NewInodes {
            count: 1,
            write_time_seconds,
        },
        UserChange::SequentialWriteOfAnInode {
            inode: inode_to_unlink,
            content: &content_of_the_unlinked_file,
            write_time_seconds: write_time_seconds + 1,
        },
    ] {
        session
            .publish_user_change(&mut pool.devices, change, reading_zero())
            .unwrap_or_else(|refused| panic!("单元区还放得下一个单元：{refused:?}"));
    }
    fault.arm_after(0);
    let intent_refused = session.publish_user_change(
        &mut pool.devices,
        UserChange::Unlink {
            inode: inode_to_unlink,
            write_time_seconds: write_time_seconds + 2,
        },
        reading_zero(),
    );
    assert!(fault.has_fired(), "故障打在了 unlink 意图上");
    assert!(intent_refused.is_err(), "意图写盘报错：冻结");
    assert!(
        session
            .resend_the_frozen_publish(&mut pool.devices)
            .expect("故障只报一次：重发做得成"),
        "冻结着一次发布"
    );
    assert!(
        !session
            .session()
            .current
            .version_of_trees()
            .expect("有树")
            .queue_of_committed_intents_is_empty(),
        "阳性对照：意图落盘了，它的批还没走"
    );
}

/// 删除队列没走空时交一次放不下的写：走空那几步先发成，这次写才判到全放出来也放不下，拒绝里带着走空发成的那几版，不丢账。
/// 队列照 [`leave_an_unlink_whose_batches_have_not_walked`] 留下。
#[test]
fn a_write_refused_outright_after_walking_the_queue_keeps_the_versions_of_the_walk() {
    let PoolWithTheFloorBelowItsCeiling {
        mut pool,
        session,
        fault,
    } = pool_with_the_floor_below_its_ceiling(
        UNIT_AREA_SLOTS_SMALLER_THAN_A_BATCH_WITHIN_THE_RECORD_LIMIT,
    );
    let journal_ring_bytes = pool.parameters.geometry.journal_ring_bytes;
    let mut session = batching(session, journal_ring_bytes);
    leave_an_unlink_whose_batches_have_not_walked(
        &mut pool,
        &mut session,
        &fault,
        FIRST_WRITE_TIME_SECONDS + 1705,
    );
    let content =
        vec![0x5A_u8; UNITS_OF_A_BATCH_WITHIN_THE_RECORD_LIMIT * data_unit_payload_capacity()];
    let Err(refused) = session.publish_user_change(
        &mut pool.devices,
        UserChange::SequentialWrite(FirstFile {
            content: &content,
            write_time_seconds: FIRST_WRITE_TIME_SECONDS + 1708,
        }),
        reading_zero(),
    ) else {
        panic!("内容比单元区还大：发不成");
    };
    let UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(no_space) =
        &*refused.refusal
    else {
        panic!("全放出来也放不下：不推，实际 {:?}", refused.refusal);
    };
    assert!(
        !no_space.versions_published.is_empty(),
        "走空发成的那几版带在拒绝里"
    );
    assert_eq!(
        no_space.floor_raises.len(),
        no_space.refusals_that_pushed_the_floor_raises.len(),
        "推成的串与推它的被拒一一对应"
    );
}

/// 段外的槽对短，差额压在抬 F 扣住的槽上（上一次抬 F 半路报错之后留下的形：扣住的槽要等 F 在每块盘上生效才放开）：
/// 现有的槽对加 defer 与隔离比需求少一对，只有算上扣住的才够得上，所以不许判成全放出来也放不下。
#[test]
fn a_write_short_on_slot_pairs_only_behind_slots_held_for_the_floor_is_not_refused_outright() {
    let PoolWithTheFloorBelowItsCeiling {
        mut pool,
        mut session,
        fault: _,
    } = pool_with_the_floor_below_its_ceiling(UNIT_AREA_SLOTS_OF_THE_SMALLER_DEVICES);
    let units_of_the_write =
        u64::try_from(UNITS_OF_A_BATCH_WITHIN_THE_RECORD_LIMIT).expect("单元数装得进 u64");
    let cluster_segments = session.allocator.cluster_segments().clone();
    for device_map in &mut session.allocator.devices {
        let slot_pairs_left = units_of_the_write
            .checked_sub(device_map.deferred_slots() + device_map.isolated_slots() + 1)
            .expect("覆盖写十几次的 defer 少于这次写的单元数");
        let start = device_map.unit_area_start().slot().0;
        let slots = device_map.unit_area_slots();
        // 迭代上界是单元区槽数；提前出口：段外的槽对只剩这么多。
        for slot in (start..start + slots).rev() {
            if device_map
                .unblocked_slot_pairs_outside_the_cluster_segments(&cluster_segments)
                .0
                <= slot_pairs_left
            {
                break;
            }
            if device_map.is_free(SlotNumber(slot)) {
                device_map.hold_until_floor_takes_effect(SlotNumber(slot), 1);
            }
        }
        assert_eq!(
            device_map
                .unblocked_slot_pairs_outside_the_cluster_segments(&cluster_segments)
                .0,
            slot_pairs_left,
            "阳性对照：扣到现有的槽对加 defer 与隔离比需求少一对"
        );
    }
    let journal_ring_bytes = pool.parameters.geometry.journal_ring_bytes;
    let mut session = batching(session, journal_ring_bytes);
    let content =
        vec![0x5A_u8; UNITS_OF_A_BATCH_WITHIN_THE_RECORD_LIMIT * data_unit_payload_capacity()];
    let outcome = session.accept(
        &mut pool.devices,
        UserChange::SequentialWrite(FirstFile {
            content: &content,
            write_time_seconds: FIRST_WRITE_TIME_SECONDS + 1704,
        }),
        reading_zero(),
    );
    if let Err(AcceptRefused::RefusedAfterRaisingTheFloorForSpace { refusal, .. }) = &outcome {
        assert!(
            !matches!(
                **refusal,
                UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(_)
            ),
            "扣住的槽放得开：上界够得上需求，照推，实际 {refusal:?}"
        );
    }
}

/// 一个数据单元的槽数换成 `pairs` 个槽对的字节（落得下那一判与抬 F 之前先算的槽对那一判都按它换）。
fn bytes_of_slot_pairs(pairs: u64) -> u64 {
    BytesOnOneDevice::of_slots(
        pairs * TransactionUnit::Data(DataUnitIndexInFile::FIRST).span_slots(),
    )
    .0
}

/// 槽对那一判的上界照「每个放得开的槽各补一对」数：段外没挡的槽对，加 defer、隔离与空闲而挡着（隔离或抬 F 扣住）的槽各算一对。
/// 一个数据单元两槽进 defer 算两对、聚簇段里的 defer 槽也算，比三样全放开之后段外真成的对数多；用例拿它核需求落在两者之间（阳性对照）。
fn slot_pairs_counting_one_pair_for_every_slot_a_floor_raise_could_release(
    device_map: &DeviceFreeMap,
    cluster_segments: &BTreeSet<SlotNumber>,
) -> u64 {
    device_map
        .unblocked_slot_pairs_outside_the_cluster_segments(cluster_segments)
        .0
        + device_map.deferred_slots()
        + device_map.isolated_slots()
        + (device_map.free_slots() - device_map.unblocked_slots())
}

/// 这块盘单元区里最高的、整段 64 槽都没分配过（`segment_is_empty`）、又不是这次挂载开过的聚簇段的那一段的起点槽号。
/// 第一个文件与覆盖写都落在低处，单元区大半空着。
fn highest_unused_segment_outside_the_cluster_segments(
    device_map: &DeviceFreeMap,
    cluster_segments: &BTreeSet<SlotNumber>,
) -> u64 {
    let start = device_map.unit_area_start().slot().0;
    let full_segments = device_map.unit_area_slots() / CLUSTER_SEGMENT_SLOTS;
    // 迭代上界是单元区的整段数；提前出口：找到最高那一段。
    (0..full_segments)
        .rev()
        .find(|segment| {
            let segment_start = start + segment * CLUSTER_SEGMENT_SLOTS;
            device_map.segment_is_empty(usize::try_from(*segment).expect("段号装得进 usize"))
                && !cluster_segments.contains(&SlotNumber(segment_start))
        })
        .map(|segment| start + segment * CLUSTER_SEGMENT_SLOTS)
        .expect("单元区大半空着：有没分配过的整段")
}

/// 每块盘上段外两槽都空的槽对，除了最高那一整段没分配过的（[`highest_unused_segment_outside_the_cluster_segments`]），都把上面那一槽记成已分配
/// （盘上不动）：那几对罩进未释放的记录、放开什么都不再成对；下面那一槽照旧没挡，留给抬 F 那一串的提交内生块。最高那一段原样留着：
/// 它的 32 对是段外仅剩没挡的槽对，它全空让合取 7（`N₀` 取第一版的值）照旧放行，推不推只由槽对那一判的上界定。
fn take_the_upper_slot_of_every_free_slot_pair_outside_the_cluster_segments_but_the_highest_unused_segment(
    session: &mut MountedSession,
) {
    let cluster_segments = session.allocator.cluster_segments().clone();
    for device_map in &mut session.allocator.devices {
        let start = device_map.unit_area_start().slot().0;
        let end = start + device_map.unit_area_slots();
        let kept_segment_start =
            highest_unused_segment_outside_the_cluster_segments(device_map, &cluster_segments);
        // 迭代上界是单元区槽数的一半：每一轮往后两槽，跨轮只改这一对上面那一槽的分配位。
        for first_slot_of_the_pair in (start..end - 1).step_by(2) {
            let segment_start = start
                + (first_slot_of_the_pair - start) / CLUSTER_SEGMENT_SLOTS * CLUSTER_SEGMENT_SLOTS;
            let is_kept_or_inside_a_cluster_segment = segment_start == kept_segment_start
                || cluster_segments.contains(&SlotNumber(segment_start));
            if !is_kept_or_inside_a_cluster_segment
                && device_map.is_free(SlotNumber(first_slot_of_the_pair))
                && device_map.is_free(SlotNumber(first_slot_of_the_pair + 1))
            {
                device_map.mark_allocated(SlotNumber(first_slot_of_the_pair + 1), 1);
            }
        }
    }
}

/// 这一次单独那一批被判全放出来也放不下：交回那个成员。
fn refused_outright_for_this_change_alone(
    refused: &AcceptRefused,
) -> &NoSpaceEvenWithEverythingAFloorRaiseCouldRelease {
    let AcceptRefused::RefusedAfterRaisingTheFloorForSpace {
        judgement: BatchJudgedAtTheAccept::ThisChangeAlone,
        refusal,
    } = refused
    else {
        panic!("判的是这一次单独那一批：{refused:?}");
    };
    let UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(no_space) = &**refusal
    else {
        panic!("全放出来也放不下：不推，实际 {refusal:?}");
    };
    no_space
}

/// 被拒的这一次一串都没推、一版都没发：F 没动、盘上逐字节不变、待发空；进程退出再可写挂载读得出旧内容。
fn nothing_was_pushed_and_the_old_content_reads_back_after_a_crash(
    pool: &mut PoolUnderTest<DeviceFailingAWriteOnRequest>,
    session: MountedSessionWithPendingChanges,
    refused: &AcceptRefused,
    image_before: &MemoryPool,
    floor_before: CheckpointTxg,
) {
    let no_space = refused_outright_for_this_change_alone(refused);
    assert!(
        refused.floor_raises_pushed_by_this_accept().is_empty()
            && no_space.versions_published.is_empty()
            && no_space.refusals_that_pushed_the_floor_raises.is_empty(),
        "头一次被拒就判到这里：一串都没推、一版都没发"
    );
    assert_eq!(
        session.session().current.root().rollback_floor,
        floor_before,
        "F 没动"
    );
    assert!(pool.image() == *image_before, "盘上逐字节不变");
    assert_eq!(session.pending_changes_accepted(), 0, "待发空");
    drop(session);
    the_old_content_reads_back_after_a_crash(pool);
}

/// 会话放下之后进程退出：再可写挂载做得成，读得出这一次之前的内容。
fn the_old_content_reads_back_after_a_crash(
    pool: &mut PoolUnderTest<DeviceFailingAWriteOnRequest>,
) {
    let content_before = pool.content_of_the_current_version.clone();
    pool.crash_and_mount_writable()
        .expect("崩了再可写挂载做得成：这一次没动盘");
    assert_eq!(
        first_file_read_back(&pool.image()),
        content_before,
        "再挂载读得出旧内容"
    );
}

/// 短在段外槽对那一判，需求落在「三样全放开之后段外真成的槽对」与「每个放得开的槽各补一对」之间：段外没挡的槽对只剩最高那一段的 32 对，
/// defer 里是覆盖写换下的数据单元（两槽放开只成一对）与聚簇段里的提交内生块（放开不在段外成对）。真成的对数不够，一串都不推就拒，
/// 点名两块盘、上界是真成的对数；盘上逐字节不变、待发空，崩了再挂载读得出旧内容。
/// 判别力：上界照每个放得开的槽各补一对算时够得上需求，推一串抬 F，推完照旧放不下。
#[test]
fn an_accept_short_on_slot_pairs_that_releasing_everything_would_not_complete_is_refused_without_a_series(
) {
    let PoolWithTheFloorBelowItsCeiling {
        mut pool,
        mut session,
        fault: _,
    } = pool_with_the_floor_below_its_ceiling(UNIT_AREA_SLOTS_OF_THE_SMALLER_DEVICES);
    take_the_upper_slot_of_every_free_slot_pair_outside_the_cluster_segments_but_the_highest_unused_segment(
        &mut session,
    );
    let cluster_segments = session.allocator.cluster_segments().clone();
    let units_of_the_write = session
        .allocator
        .devices
        .iter()
        .map(|device_map| {
            device_map
                .slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(
                    &cluster_segments,
                )
                .0
        })
        .max()
        .expect("池里有盘")
        + 1;
    let mut devices_short_expected = Vec::new();
    for device_map in &session.allocator.devices {
        assert_eq!(
            device_map
                .unblocked_slot_pairs_outside_the_cluster_segments(&cluster_segments)
                .0,
            CLUSTER_SEGMENT_SLOTS / 2,
            "阳性对照：段外没挡的槽对只剩最高那一段的 32 对"
        );
        assert!(
            device_map.empty_segments() >= EMPTY_CLUSTER_SEGMENTS_FLOOR_OF_THE_FIRST_VERSION,
            "阳性对照：最高那一段全空，合取 7 那一判放行"
        );
        assert!(
            device_map.deferred_slots() > 0,
            "阳性对照：覆盖写过，defer 里有槽"
        );
        let slot_pairs_once_everything_is_released = device_map
            .slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(&cluster_segments)
            .0;
        let slot_pairs_counting_one_pair_for_every_slot =
            slot_pairs_counting_one_pair_for_every_slot_a_floor_raise_could_release(
                device_map,
                &cluster_segments,
            );
        assert!(
            slot_pairs_once_everything_is_released < units_of_the_write
                && units_of_the_write <= slot_pairs_counting_one_pair_for_every_slot,
            "阳性对照：需求 {units_of_the_write} 对落在全放开之后真成的 {slot_pairs_once_everything_is_released} 对\
             与每个放得开的槽各补一对的 {slot_pairs_counting_one_pair_for_every_slot} 对之间"
        );
        devices_short_expected.push(DeviceShortEvenWithEverythingAFloorRaiseCouldRelease {
            device: device_map.device,
            judged_short_by:
                JudgementThatFoundTheDeviceShort::SlotPairsOutsideTheClusterSegmentsForDataUnits,
            most_a_floor_raise_could_make_available: AvailableBytesOnOneDevice(i128::from(
                bytes_of_slot_pairs(slot_pairs_once_everything_is_released),
            )),
            demand: BytesOnOneDevice(bytes_of_slot_pairs(units_of_the_write)),
        });
    }
    let floor_before = session.current.root().rollback_floor;
    let journal_ring_bytes = pool.parameters.geometry.journal_ring_bytes;
    let mut session = batching(session, journal_ring_bytes);
    let content = vec![
        0x5A_u8;
        usize::try_from(units_of_the_write).expect("单元数装得进 usize")
            * data_unit_payload_capacity()
    ];
    let image_before = pool.image();
    let refused = session
        .accept(
            &mut pool.devices,
            UserChange::SequentialWrite(FirstFile {
                content: &content,
                write_time_seconds: FIRST_WRITE_TIME_SECONDS + 1709,
            }),
            reading_zero(),
        )
        .expect_err("全放开之后段外真成的槽对也比需求少：收不下");
    let no_space = refused_outright_for_this_change_alone(&refused);
    assert!(
        no_space.empty_cluster_segments_below_the_floor.is_none(),
        "判出它的是槽对那一判，不是合取 7：{:?}",
        no_space.empty_cluster_segments_below_the_floor
    );
    assert_eq!(
        no_space.devices_short, devices_short_expected,
        "两块盘都点名：判它短的是槽对那一判，上界是三样全放开之后段外真成的槽对，需求是这次的数据单元"
    );
    nothing_was_pushed_and_the_old_content_reads_back_after_a_crash(
        &mut pool,
        session,
        &refused,
        &image_before,
        floor_before,
    );
}

/// 每块盘从单元区最高处往下把空闲的槽记成抬 F 扣住（只动扣住位，不写盘），到段外没挡的槽对只剩 `slot_pairs_left` 对为止：
/// 扣住的槽不罩在未释放的记录下，三样全放开之后照旧成对。
fn hold_the_highest_free_slots_leaving_slot_pairs(
    session: &mut MountedSession,
    slot_pairs_left: u64,
) {
    let cluster_segments = session.allocator.cluster_segments().clone();
    for device_map in &mut session.allocator.devices {
        let start = device_map.unit_area_start().slot().0;
        let slots = device_map.unit_area_slots();
        // 迭代上界是单元区槽数；提前出口：段外没挡的槽对只剩这么多。
        for slot in (start..start + slots).rev() {
            if device_map
                .unblocked_slot_pairs_outside_the_cluster_segments(&cluster_segments)
                .0
                <= slot_pairs_left
            {
                break;
            }
            if device_map.is_free(SlotNumber(slot)) {
                device_map.hold_until_floor_takes_effect(SlotNumber(slot), 1);
            }
        }
    }
}

/// 会话装上只供测试的 `N₀`（`EmptyClusterSegmentsFloor::ForcedForTests`），整理的别的开关照产品值。
fn force_the_empty_cluster_segments_floor(session: &mut MountedSession, floor_in_segments: u64) {
    let CompactionSwitches {
        batch_size,
        forced_region,
        run_forcing,
        region_exclusion,
        destination_policy,
        release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor: _,
        relocation_publish_order,
    } = CompactionSwitches::PRODUCT;
    session
        .allocator
        .set_compaction_switches(CompactionSwitches {
            batch_size,
            forced_region,
            run_forcing,
            region_exclusion,
            destination_policy,
            release_checksum_mismatch_after_issue,
            empty_cluster_segments_floor: EmptyClusterSegmentsFloor::ForcedForTests(
                floor_in_segments,
            ),
            relocation_publish_order,
        });
}

/// 会话摆成落得下那一判会先拒一次 [`UNITS_OF_A_BATCH_WITHIN_THE_RECORD_LIMIT`] 个数据单元的写（段外没挡的槽对比需求少一对，差额压在抬 F 扣住的槽上）、
/// 槽对那一判的上界够得上需求，再经只供测试的开关把 `N₀` 抬到比每块盘的全空段数都多一段（这个池单元区大半空着，全空段多）。
/// 交回合取 7 那一判该报的那一份（`N₀` 与每块盘的全空段数）。
fn hold_slots_and_raise_the_empty_cluster_segments_floor_above_every_device(
    session: &mut MountedSession,
) -> EmptyClusterSegmentsBelowTheFloorOnSomeDevices {
    let units_of_the_write =
        u64::try_from(UNITS_OF_A_BATCH_WITHIN_THE_RECORD_LIMIT).expect("单元数装得进 u64");
    hold_the_highest_free_slots_leaving_slot_pairs(session, units_of_the_write - 1);
    let cluster_segments = session.allocator.cluster_segments().clone();
    for device_map in &session.allocator.devices {
        assert_eq!(
            device_map
                .unblocked_slot_pairs_outside_the_cluster_segments(&cluster_segments)
                .0,
            units_of_the_write - 1,
            "阳性对照：段外没挡的槽对比需求少一对，落得下那一判拒"
        );
        assert!(
            device_map
                .slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(
                    &cluster_segments
                )
                .0
                >= units_of_the_write,
            "阳性对照：扣住的槽放得开，槽对那一判的上界够得上需求"
        );
    }
    let empty_cluster_segments_on_each_device: Vec<DeviceWithEmptyClusterSegmentsBelowTheFloor> =
        session
            .allocator
            .devices
            .iter()
            .map(|device_map| DeviceWithEmptyClusterSegmentsBelowTheFloor {
                device: device_map.device,
                empty_cluster_segments: device_map.empty_segments(),
            })
            .collect();
    let floor_in_segments = empty_cluster_segments_on_each_device
        .iter()
        .map(|device| device.empty_cluster_segments)
        .max()
        .expect("池里有盘")
        + 1;
    force_the_empty_cluster_segments_floor(session, floor_in_segments);
    EmptyClusterSegmentsBelowTheFloorOnSomeDevices {
        floor_in_segments,
        short_devices: empty_cluster_segments_on_each_device,
    }
}

/// 被拒的这一次是推串之前合取 7 那一判判出的：没有上界小于需求的盘，合取 7 那一份是 `below_the_floor_expected`，
/// 被拒的那一次发布是落得下那一判的段外槽对那一样拒的；一串都没推、一版都没发。
fn refused_by_the_empty_cluster_segments_judgement_before_any_series(
    no_space: &NoSpaceEvenWithEverythingAFloorRaiseCouldRelease,
    below_the_floor_expected: EmptyClusterSegmentsBelowTheFloorOnSomeDevices,
) {
    assert!(
        no_space.devices_short.is_empty(),
        "没有上界小于需求的盘：{:?}",
        no_space.devices_short
    );
    assert_eq!(
        no_space.empty_cluster_segments_below_the_floor,
        Some(below_the_floor_expected),
        "合取 7 那一份：N₀ 与每块盘的全空段数"
    );
    let PublishError::SpaceAdmissionRefused(admission_refusal) = &no_space.last_refusal else {
        panic!("被拒的是落得下那一判：{:?}", no_space.last_refusal);
    };
    assert!(
        !admission_refusal.short_devices.is_empty()
            && admission_refusal.short_devices.iter().all(|short| {
                short.judged_short_by
                    == JudgementThatFoundTheDeviceShort::SlotPairsOutsideTheClusterSegmentsForDataUnits
            }),
        "推串之前被拒的是段外槽对那一判：{admission_refusal:?}"
    );
    assert!(
        no_space.floor_raises.is_empty()
            && no_space.refusals_that_pushed_the_floor_raises.is_empty()
            && no_space.versions_published.is_empty(),
        "头一次被拒就判到这里：一串都没推、一版都没发"
    );
}

/// 落得下那一判先拒、槽对那一判的上界够得上需求，而每块盘的全空聚簇段在写之前都已少于 `N₀`
/// （[`hold_slots_and_raise_the_empty_cluster_segments_floor_above_every_device`]）：被拒的是写文件内容、归合取 7 管，
/// 推串之前那一判判出，一串都不推就拒，带上合取 7 那一份、不点名槽对短的盘；盘上逐字节不变、待发空，崩了再挂载读得出旧内容。
/// 判别力：推串之前不判合取 7 时推一串抬 F。
#[test]
fn an_accept_while_empty_cluster_segments_are_below_the_floor_is_refused_without_a_series() {
    let PoolWithTheFloorBelowItsCeiling {
        mut pool,
        mut session,
        fault: _,
    } = pool_with_the_floor_below_its_ceiling(UNIT_AREA_SLOTS_OF_THE_SMALLER_DEVICES);
    let below_the_floor_expected =
        hold_slots_and_raise_the_empty_cluster_segments_floor_above_every_device(&mut session);
    let floor_before = session.current.root().rollback_floor;
    let journal_ring_bytes = pool.parameters.geometry.journal_ring_bytes;
    let mut session = batching(session, journal_ring_bytes);
    let content =
        vec![0x5A_u8; UNITS_OF_A_BATCH_WITHIN_THE_RECORD_LIMIT * data_unit_payload_capacity()];
    let image_before = pool.image();
    let refused = session
        .accept(
            &mut pool.devices,
            UserChange::SequentialWrite(FirstFile {
                content: &content,
                write_time_seconds: FIRST_WRITE_TIME_SECONDS + 1710,
            }),
            reading_zero(),
        )
        .expect_err("全空聚簇段少于 N₀：收不下");
    refused_by_the_empty_cluster_segments_judgement_before_any_series(
        refused_outright_for_this_change_alone(&refused),
        below_the_floor_expected,
    );
    nothing_was_pushed_and_the_old_content_reads_back_after_a_crash(
        &mut pool,
        session,
        &refused,
        &image_before,
        floor_before,
    );
}

/// 同一形的池与同一次写，不经 `accept`、经会话的 [`MountedSession::publish_user_change`] 单发：被拒的是这次改动自己那一次发布
/// （删除队列是空的，没有先走空），写文件内容归合取 7 管，推串之前那一判判出，一串都不推就拒，带上合取 7 那一份；
/// F 没动、盘上逐字节不变，崩了再挂载读得出旧内容。
/// 判别力：`publish_user_change` 那一处把被拒的改动当成合取 7 免的那一种交下去时照推一串抬 F。
#[test]
fn a_publish_of_a_write_while_empty_cluster_segments_are_below_the_floor_is_refused_without_a_series(
) {
    let PoolWithTheFloorBelowItsCeiling {
        mut pool,
        mut session,
        fault: _,
    } = pool_with_the_floor_below_its_ceiling(UNIT_AREA_SLOTS_OF_THE_SMALLER_DEVICES);
    let below_the_floor_expected =
        hold_slots_and_raise_the_empty_cluster_segments_floor_above_every_device(&mut session);
    assert!(
        session
            .current
            .version_of_trees()
            .expect("有树")
            .queue_of_committed_intents_is_empty(),
        "阳性对照：删除队列空，被拒的是这次改动自己那一次发布"
    );
    let floor_before = session.current.root().rollback_floor;
    let content =
        vec![0x5A_u8; UNITS_OF_A_BATCH_WITHIN_THE_RECORD_LIMIT * data_unit_payload_capacity()];
    let image_before = pool.image();
    let Err(refused) = session.publish_user_change(
        &mut pool.devices,
        UserChange::SequentialWrite(FirstFile {
            content: &content,
            write_time_seconds: FIRST_WRITE_TIME_SECONDS + 1711,
        }),
    ) else {
        panic!("全空聚簇段少于 N₀：发不成");
    };
    let UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(no_space) = &refused
    else {
        panic!("写文件内容归合取 7 管：推串之前判出、不推，实际 {refused:?}");
    };
    refused_by_the_empty_cluster_segments_judgement_before_any_series(
        no_space,
        below_the_floor_expected,
    );
    assert_eq!(
        session.current.root().rollback_floor,
        floor_before,
        "F 没动"
    );
    assert!(pool.image() == image_before, "盘上逐字节不变");
    drop(session);
    the_old_content_reads_back_after_a_crash(&mut pool);
}

/// 删除队列没走空、走空那一步被空间拒，而每块盘的全空聚簇段在写之前都已少于 `N₀`：走空的每一步是删除类、合取 7 免，
/// 推串之前不判合取 7，照推一串抬 F。单元区里空闲的槽都记成已分配，抬 F 那一串自己也放不下，推到不再推了报空间不够，
/// 报里被拒的是走空那一步、空间不够的那两种之一，不再推是因为抬 F 那一串自己放不下；走空的头一步就被拒、抬 F 那一串没推成，
/// 报里推成的串与发成的版都是空的。
/// 判别力：走空那一处把被拒的那一步当成合取 7 判的前台写入交下去时，推串之前那一判判出、一串都不推就拒（全放出来也放不下）。
#[test]
fn a_walk_step_refused_for_space_while_empty_cluster_segments_are_below_the_floor_is_pushed_for() {
    let PoolWithTheFloorBelowItsCeiling {
        mut pool,
        session,
        fault,
    } = pool_with_the_floor_below_its_ceiling(UNIT_AREA_SLOTS_OF_THE_SMALLER_DEVICES);
    let journal_ring_bytes = pool.parameters.geometry.journal_ring_bytes;
    let mut session = batching(session, journal_ring_bytes);
    leave_an_unlink_whose_batches_have_not_walked(
        &mut pool,
        &mut session,
        &fault,
        FIRST_WRITE_TIME_SECONDS + 1712,
    );
    mark_every_free_slot_allocated(session.session_for_tests_only());
    let floor_in_segments = session
        .session()
        .allocator
        .devices
        .iter()
        .map(DeviceFreeMap::empty_segments)
        .max()
        .expect("池里有盘")
        + 1;
    force_the_empty_cluster_segments_floor(session.session_for_tests_only(), floor_in_segments);
    let Err(refused) = session.publish_user_change(
        &mut pool.devices,
        UserChange::NewInodes {
            count: 1,
            write_time_seconds: FIRST_WRITE_TIME_SECONDS + 1715,
        },
        reading_zero(),
    ) else {
        panic!("单元区里一个空闲的槽都没有：走空那一步发不成");
    };
    let UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space) = &*refused.refusal else {
        panic!(
            "走空那一步免合取 7：照推，推到不再推了报空间不够，实际 {:?}",
            refused.refusal
        );
    };
    assert!(
        refusal_is_short_of_space(&no_space.last_refusal),
        "被拒的是走空那一步、空间不够：{:?}",
        no_space.last_refusal
    );
    assert!(
        matches!(
            no_space.stop,
            FloorRaiseStop::FloorRaiseRefusedForSpace { .. }
        ),
        "不再推是因为抬 F 那一串自己也放不下：{:?}",
        no_space.stop
    );
    assert!(
        no_space.floor_raises.is_empty()
            && no_space.refusals_that_pushed_the_floor_raises.is_empty(),
        "抬 F 那一串自己放不下：这次调用一串都没推成，实际推成 {} 串",
        no_space.floor_raises.len()
    );
    assert!(
        no_space.versions_published.is_empty(),
        "单元区里一个空闲的槽都没有：走空的头一步就被拒，这次调用一版都没发成，实际发成 {} 版",
        no_space.versions_published.len()
    );
}
