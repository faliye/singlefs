//! 进程内记根时重读挂载那一刻读坏的根环槽（D16（发布语义） 已定项 1「环里最旧有效根」那一行的 E2 句与「根槽这一次读坏」那一行）：
//! 重读那一次的几种结局各记成什么、判「按实例表有效」用哪张表、产品路径上记根的几处各自走到重读。
//!
//! 读坏的槽在回收门槛里钉在它可能住过的最旧 txg，重读读出有效根或被抛弃的根之前门槛不越过它；重读时机照 E2（根环表里这一项成了最小的那个时）；
//! 读回全零的根槽在回收门槛里照「自证不过」那一类钉住，只是不并进准入抬 F 的上限（用户 2026-10-04 定全零不并进上限，2026-10-05 定乙；
//! 条文还没写回，定案在 `records/2026-09-30-里程碑三开工第一轮.md`）。钉在最旧 txg 之后那一槽挂载那一刻起多半就是根环表里最小的那个，
//! 挂载之后第一次记根就重读。
//!
//! - 读得出而自证不过，与仍读不出同一处置，照旧钉着（用户 2026-10-04 定改法 A，代码轮第二轮判决
//!   `research/prompts/m3-root-ring-code-r2-main-verification.md` 第六节 RS1；攻方 `research/prompts/m3-root-ring-code-r2-opus-model/proto/src/bin/e2_reread_flip.rs`
//!   那一形：一条有效根所在的槽挂载那一刻读错、重读那一次读坏，改之前回收门槛越过它、I-3.1 记账少）。
//! - 读回全零：不算确认为空，照旧钉着（用户 2026-10-05 定乙）。同一目录 `front_hole_eio.rs` 那一形（从没写过的槽加挂载那一刻读错）上，
//!   改法 A 认下的「重读那一版按记的 txg 多记账、I-3.1 红在记账多的一边」因此回来了，到这个进程写掉那一槽为止（一版）。
//! - 读得出、自证过、按实例表有效的照读出来的 txg 算，那一槽从此算「这个进程知道住着根的槽」（用户 2026-10-04 定）；
//!   读得出、自证过而按实例表被抛弃的当没有根；判有效用的表写行之后是写行那一张（同一份判决第二节 RS2，正推腿探针 p1、p2）。
//! - 记根的产品路径：带单元的发布（`transaction::publish_admitted`，已有变异行管它）、树表 0 条那一版上要写行的写行发布
//!   （`transaction::prepare_the_row_publish_on_a_version_without_file`）走到重读。第一次可写挂载的零单元写行发布
//!   （`mount::establish_instance` 里 `rows_written` 为空那一支）走不到重读——那一刻根环里只有 mkfs 写的 txg 0，挂载时读坏的槽只可能是它，
//!   而它读不出时择不到根、挂载不成。
mod common;
mod common_root_ring_holes;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use common::{
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool,
    overwrite_in_process_as_instance_one_and_advance_the_pool,
    BYTE_INSIDE_THE_ROOT_RECORDS_TREE_TABLE_POINTER, FIXED_WRITE_TIME_SECONDS,
};
use common_root_ring_holes::{
    content_of_the_version, devices_of, fixed_geometry_of_the_ring_hole_pools,
    parameters_of_the_ring_hole_pools, pool_of, pool_overwritten_through, recording_of,
    RingHoleDevices, DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
    WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::allocator::{PoolAllocator, TxgsARingSlotBadAtMountCanHold};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor_to_the_admission_ceiling, MountError, ShadowLedger,
};
use singlefs_core::recovery::instance_table_chain_of_root;
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};
use singlefs_harness::memory_pool::{writes_and_segments, MemoryPool, SparseBlockDevice};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 判的几条：记账与遍历（I-3.1、I-3.11）、被引用单元读回来对不对（I-2.1、I-7.4、I-4.8）。
const INVARIANTS_THE_REREAD_KEEPS_GREEN: [&str; 5] = ["I-3.1", "I-3.11", "I-2.1", "I-7.4", "I-4.8"];
/// 攻方那一形：实例 1 覆盖写到 txg 20，有效根 h = 18 所在的槽（区域 0、槽 2）挂载那一刻读错。
const LAST_TXG_OF_INSTANCE_ONE: u64 = 20;
const TXG_OF_THE_VALID_ROOT_UNREADABLE_AT_MOUNT: u64 = 18;
/// h 那一槽（区域 0、槽 2）可能住过的最旧 txg：2 × 3 + 0（S = 4）。
const OLDEST_TXG_THE_SLOT_OF_THE_VALID_ROOT_CAN_HAVE_HELD: u64 = 6;
/// 前洞那一形：实例 2 的 txg 9 那次覆盖写崩在根槽写之前（环还没转过来，那一槽从没写过）。
const TXG_OF_THE_FRONT_HOLE: u64 = 9;
/// 前洞那一形里那一槽成了根环表里最小的那个的那一版：9 + 3S − 1（txg 8 那一槽被盖掉）。
const TXG_AT_WHICH_THE_FRONT_HOLE_IS_REREAD: u64 = 20;

/// 被抛弃根那一形判的几条：同上去掉 I-7.4。被抛弃的根所在的槽挂载那一刻读不出，影子账罩不到它引用的单元（挂载那一刻就读不到那条根，
/// 与重读无关：攻方 `research/prompts/m3-root-ring-code-r2-opus-output.md` K1 第 3 小节「被抛弃」那一条），之后那几个单元被合法复用，
/// I-7.4 在这一形上红是今天已知的那一格，不归这条用例判。
const INVARIANTS_JUDGED_ON_THE_ABANDONED_ROOT: [&str; 4] = ["I-3.1", "I-3.11", "I-2.1", "I-4.8"];

fn violations_of(image: &MemoryPool) -> Vec<String> {
    violations_among(image, &INVARIANTS_THE_REREAD_KEEPS_GREEN)
}

fn violations_among(image: &MemoryPool, invariants_judged: &[&str]) -> Vec<String> {
    check_pool_image(image)
        .into_iter()
        .filter(|(invariant, _)| invariants_judged.contains(invariant))
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some(format!("{invariant}：{detail}")),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

/// 一个根环槽的读这会儿怎么坏。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReadsOfTheRootRingSlot {
    PassedThrough,
    DeviceErrorOnEveryRead,
    /// 任何一块盘上发出第一个写（取号写系统配置）之前每次读都报设备错，之后照读：只在可写挂载读阶段读错。
    DeviceErrorUntilTheFirstWriteOfAnyDevice,
    /// 只有下一次读坏：读得出，那一槽里一个字节取反（自证不过，盘上字节不变）；之后照读。
    OneByteFlippedOnTheNextReadOnly,
}

struct FaultOfOneRootRingSlotState {
    device: DeviceIdentity,
    offset_in_bytes: u64,
    length_in_bytes: u64,
    reads: ReadsOfTheRootRingSlot,
    a_write_was_sent: bool,
    failed_reads: u64,
    flipped_reads: u64,
}

/// 几块盘共用的、一个根环槽的读故障。
#[derive(Clone)]
struct FaultOfOneRootRingSlot(Rc<RefCell<FaultOfOneRootRingSlotState>>);

enum ReadOutcome {
    Read,
    DeviceError,
    FlipTheByteAt(usize),
}

impl FaultOfOneRootRingSlot {
    fn of_the_root_slot_of_txg(
        parameters: &MakeFilesystemParameters,
        checkpoint_txg: CheckpointTxg,
        reads: ReadsOfTheRootRingSlot,
    ) -> Self {
        let ring_slot = target_for_publish(
            checkpoint_txg,
            parameters.geometry.root_ring_slots_per_region,
        );
        Self(Rc::new(RefCell::new(FaultOfOneRootRingSlotState {
            device: parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")],
            offset_in_bytes: slot_offset(
                ring_slot,
                parameters.geometry.fixed_structure_slot_spacing,
            )
            .0,
            length_in_bytes: u64::from(parameters.geometry.physical_block_size),
            reads,
            a_write_was_sent: false,
            failed_reads: 0,
            flipped_reads: 0,
        })))
    }

    fn set_reads(&self, reads: ReadsOfTheRootRingSlot) {
        self.0.borrow_mut().reads = reads;
    }

    fn failed_reads(&self) -> u64 {
        self.0.borrow().failed_reads
    }

    fn flipped_reads(&self) -> u64 {
        self.0.borrow().flipped_reads
    }

    fn note_a_write(&self) {
        self.0.borrow_mut().a_write_was_sent = true;
    }

    fn note_a_read(
        &self,
        device: DeviceIdentity,
        offset_in_bytes: u64,
        length: u64,
    ) -> ReadOutcome {
        let mut state = self.0.borrow_mut();
        let overlaps = state.device == device
            && offset_in_bytes < state.offset_in_bytes + state.length_in_bytes
            && state.offset_in_bytes < offset_in_bytes + length;
        if !overlaps {
            return ReadOutcome::Read;
        }
        match state.reads {
            ReadsOfTheRootRingSlot::PassedThrough => ReadOutcome::Read,
            ReadsOfTheRootRingSlot::DeviceErrorOnEveryRead => {
                state.failed_reads += 1;
                ReadOutcome::DeviceError
            }
            ReadsOfTheRootRingSlot::DeviceErrorUntilTheFirstWriteOfAnyDevice => {
                if state.a_write_was_sent {
                    ReadOutcome::Read
                } else {
                    state.failed_reads += 1;
                    ReadOutcome::DeviceError
                }
            }
            ReadsOfTheRootRingSlot::OneByteFlippedOnTheNextReadOnly => {
                let byte_in_the_slot =
                    state.offset_in_bytes + BYTE_INSIDE_THE_ROOT_RECORDS_TREE_TABLE_POINTER;
                if byte_in_the_slot < offset_in_bytes
                    || byte_in_the_slot >= offset_in_bytes + length
                {
                    return ReadOutcome::Read;
                }
                state.flipped_reads += 1;
                state.reads = ReadsOfTheRootRingSlot::PassedThrough;
                ReadOutcome::FlipTheByteAt(
                    usize::try_from(byte_in_the_slot - offset_in_bytes).expect("落在缓冲区里"),
                )
            }
        }
    }
}

/// 读的时候按 [`FaultOfOneRootRingSlot`] 读坏一个根环槽的盘；写、写零、屏障原样交给里面那块盘（写记一笔：取号之后不再报读错的那一档要它）。
struct DeviceWithOneRootRingSlotFaulted<Inner: BlockDevice> {
    identity: DeviceIdentity,
    inner: Inner,
    fault: FaultOfOneRootRingSlot,
}

impl<Inner: BlockDevice> BlockDevice for DeviceWithOneRootRingSlotFaulted<Inner> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length = u64::try_from(buffer.len()).expect("一次读的长度装得进 u64");
        match self.fault.note_a_read(self.identity, offset.0, length) {
            ReadOutcome::Read => self.inner.read_at(offset, buffer),
            ReadOutcome::DeviceError => Err(BlockDeviceError::InputOutput(std::io::Error::other(
                "用例点名的根环槽读坏",
            ))),
            ReadOutcome::FlipTheByteAt(index) => {
                self.inner.read_at(offset, buffer)?;
                buffer[index] = !buffer[index];
                Ok(())
            }
        }
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.fault.note_a_write();
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.fault.note_a_write();
        self.inner.write_zeroes_at(offset, length)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

fn with_the_fault<Inner: BlockDevice>(
    devices: Vec<(DeviceIdentity, Inner)>,
    fault: &FaultOfOneRootRingSlot,
) -> Vec<(DeviceIdentity, DeviceWithOneRootRingSlotFaulted<Inner>)> {
    devices
        .into_iter()
        .map(|(identity, inner)| {
            (
                identity,
                DeviceWithOneRootRingSlotFaulted {
                    identity,
                    inner,
                    fault: fault.clone(),
                },
            )
        })
        .collect()
}

type RingHoleDevicesWithTheFault = Vec<(
    DeviceIdentity,
    DeviceWithOneRootRingSlotFaulted<RecordingBlockDevice<SparseBlockDevice>>,
)>;

/// S = 4 的小池上一次可写挂载之后的会话：盘（一个根环槽带读故障）、录下的流与它的起点盘面、分配器、现行那一版。
struct SessionOnARingHolePool {
    image_before_the_mount: MemoryPool,
    stream: SharedStream,
    devices: RingHoleDevicesWithTheFault,
    fault: FaultOfOneRootRingSlot,
    ring_slot: RootRingSlot,
    allocator: PoolAllocator,
    current: TransactionOutput,
    instance: InstanceGeneration,
}

impl SessionOnARingHolePool {
    /// `image` 上可写挂载，txg 为 `txg_of_the_slot` 的那一槽挂载那一刻每次读都报设备错，挂载之后照读。
    fn mounted_with_the_slot_of_txg_unreadable_at_mount(
        image: MemoryPool,
        txg_of_the_slot: u64,
    ) -> Self {
        let parameters = parameters_of_the_ring_hole_pools();
        let fault = FaultOfOneRootRingSlot::of_the_root_slot_of_txg(
            &parameters,
            CheckpointTxg(txg_of_the_slot),
            ReadsOfTheRootRingSlot::DeviceErrorOnEveryRead,
        );
        let stream = SharedStream::retaining_contents();
        let mut devices = with_the_fault(recording_of(&image, &stream), &fault);
        let mounted = mount_writable(&parameters, &mut devices).expect("可写挂载");
        fault.set_reads(ReadsOfTheRootRingSlot::PassedThrough);
        assert!(fault.failed_reads() > 0, "挂载那一次读那一槽读错了");
        let ring_slot = target_for_publish(
            CheckpointTxg(txg_of_the_slot),
            parameters.geometry.root_ring_slots_per_region,
        );
        Self {
            image_before_the_mount: image,
            stream,
            devices,
            fault,
            ring_slot,
            instance: mounted.output.instance,
            allocator: mounted.allocator,
            current: mounted.current.into_file_version().expect("这一版带文件"),
        }
    }

    fn image_now(&self) -> MemoryPool {
        let mut image = self.image_before_the_mount.clone();
        image.apply(&self.stream.retained_operations());
        image
    }

    fn overwrite_once(&mut self) {
        let parameters = parameters_of_the_ring_hole_pools();
        let salt = self.current.root.checkpoint_txg.0 + 1;
        let mut writer = PoolWriter::new(&parameters, self.devices.as_mut_slice());
        self.current = publish_overwrite(
            &mut writer,
            &mut self.allocator,
            &self.current,
            FirstFile {
                content: &content_of_the_version(salt),
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + salt,
            },
            self.instance,
        )
        .expect("覆盖写");
    }

    /// 根环表里这一槽按挂载时读坏记着的 txg（不再这样记着时 `None`）。
    fn txgs_recorded_as_bad_at_mount(&self) -> Option<TxgsARingSlotBadAtMountCanHold> {
        self.allocator
            .root_ring_occupancy()
            .expect("可写挂载装了根环表")
            .ring_slots_bad_at_mount()
            .get(&self.ring_slot)
            .copied()
    }

    fn slot_is_known_to_hold_a_root(&self) -> bool {
        self.allocator
            .root_ring_occupancy()
            .expect("可写挂载装了根环表")
            .ring_slots_known_to_hold_a_root()
            .contains(&self.ring_slot)
    }
}

/// 攻方那一形挂载之后、重读之前：实例 1 覆盖写到 20；实例 2 可写挂载那一刻读 h = 18 那一槽报读错（挂载里写行、暖机记根时的重读也读错）。
/// 那一槽钉在它可能住过的最旧 txg 6（能住的最大 txg 18），是根环表里最小的那个：挂载之后第一次记根就重读。
fn session_right_after_the_mount_before_the_reread() -> SessionOnARingHolePool {
    let session = SessionOnARingHolePool::mounted_with_the_slot_of_txg_unreadable_at_mount(
        pool_overwritten_through(LAST_TXG_OF_INSTANCE_ONE),
        TXG_OF_THE_VALID_ROOT_UNREADABLE_AT_MOUNT,
    );
    assert_eq!(
        session.txgs_recorded_as_bad_at_mount(),
        Some(TxgsARingSlotBadAtMountCanHold::new(
            CheckpointTxg(OLDEST_TXG_THE_SLOT_OF_THE_VALID_ROOT_CAN_HAVE_HELD),
            CheckpointTxg(TXG_OF_THE_VALID_ROOT_UNREADABLE_AT_MOUNT)
        )),
        "前提：根环表把那一槽记成挂载时读坏：钉在 6，能住的最大 txg 18"
    );
    session
}

/// 场景（攻方 `e2_reread_flip 4 20 2 flip`）：挂载之后第一次覆盖写时只有一次读把那一槽读坏（一个字节取反，盘上不变），
/// 那一次就是记这条根之前的重读；再覆盖写一次（照读）。
/// 预期：读得出而自证不过与仍读不出同一处置，那一槽照旧钉在 6，回收门槛不越过盘上 txg 18 那条有效根：被引用单元那几条不红，
/// 只有 I-3.1 可能红在记账多的一边（钉着的 6 比盘上环里最旧的有效根旧，安全方向，定案的代价）；
/// 下一次记根再重读、读出 txg 18 的根，那几条不变量都不红。
#[test]
fn a_valid_root_read_bad_once_on_the_reread_keeps_the_slot_pinned_at_the_oldest_txg_it_can_have_held(
) {
    let mut session = session_right_after_the_mount_before_the_reread();
    session
        .fault
        .set_reads(ReadsOfTheRootRingSlot::OneByteFlippedOnTheNextReadOnly);
    session.overwrite_once();
    assert_eq!(
        session.fault.flipped_reads(),
        1,
        "挂载之后第一次覆盖写读那一槽、读坏了一次"
    );
    assert_eq!(
        session.txgs_recorded_as_bad_at_mount(),
        Some(TxgsARingSlotBadAtMountCanHold::new(
            CheckpointTxg(OLDEST_TXG_THE_SLOT_OF_THE_VALID_ROOT_CAN_HAVE_HELD),
            CheckpointTxg(TXG_OF_THE_VALID_ROOT_UNREADABLE_AT_MOUNT)
        )),
        "重读读到自证不过：照旧钉在 6"
    );
    let violated_after_the_bad_reread: Vec<String> = violations_of(&session.image_now())
        .iter()
        .map(|violation| violation.split('：').next().expect("不变量名").to_string())
        .collect();
    assert!(
        violated_after_the_bad_reread
            .iter()
            .all(|invariant| invariant == "I-3.1"),
        "重读读坏那一版：门槛没越过盘上 txg 18 那条有效根，被引用单元那几条不红：{violated_after_the_bad_reread:?}"
    );
    session.overwrite_once();
    assert_eq!(
        (
            session.txgs_recorded_as_bad_at_mount(),
            session.slot_is_known_to_hold_a_root()
        ),
        (None, true),
        "下一次记根再重读、读出 txg 18 的根"
    );
    assert_eq!(
        violations_of(&session.image_now()),
        Vec::<String>::new(),
        "读出 txg 18 之后那一版"
    );
}

/// 场景：同一盘面，挂载之后第一次覆盖写那一次重读读得出（读故障只在挂载那一刻）。
/// 预期：读出 txg 18 的根、按实例表有效：那一槽换成「重读读出的有效根 18」，算这个进程知道住着根的槽；那一版那几条不变量不红。
#[test]
fn a_valid_root_read_on_the_reread_holds_the_reclaim_threshold_at_its_txg() {
    let mut session = session_right_after_the_mount_before_the_reread();
    session.overwrite_once();
    assert_eq!(
        (
            session.txgs_recorded_as_bad_at_mount(),
            session.slot_is_known_to_hold_a_root()
        ),
        (None, true),
        "重读读出有效根：不再按挂载时读坏记，那一槽算这个进程知道住着根的槽"
    );
    assert_eq!(
        violations_of(&session.image_now()),
        Vec::<String>::new(),
        "门槛照读出来的 18 算"
    );
}

/// 场景：同一盘面，挂载之后第一次覆盖写那一次重读读出 txg 18 的有效根；之后那一槽每次读都报设备错，接着准入抬 F 抬到上限。
/// 预期：算上限那一遍那一槽读不出、它是这个进程知道住着根的槽：重读一次仍读不出，在任何写之前拒
/// （`RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`），盘上逐字节不变、录制流不多一步。
#[test]
fn a_slot_whose_reread_read_a_valid_root_staying_unreadable_refuses_the_raise_before_any_write() {
    let mut session = session_right_after_the_mount_before_the_reread();
    session.overwrite_once();
    assert!(
        session.slot_is_known_to_hold_a_root(),
        "前提：重读读出了有效根"
    );
    session
        .fault
        .set_reads(ReadsOfTheRootRingSlot::DeviceErrorOnEveryRead);
    let image_before = session.image_now();
    let operations_before = session.stream.operations().len();
    let refused = raise_rollback_floor_to_the_admission_ceiling(
        &parameters_of_the_ring_hole_pools(),
        &mut session.devices,
        &mut session.allocator,
        &mut session.current,
        ShadowLedger::On,
    );
    assert!(
        matches!(
            refused,
            Err(MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread { ring_slot, .. })
                if ring_slot == session.ring_slot
        ),
        "知道住着根的槽重读仍读不出就拒：{refused:?}"
    );
    assert_eq!(
        session.stream.operations().len(),
        operations_before,
        "一个写、一道屏障都没发"
    );
    assert!(session.image_now() == image_before, "盘上逐字节不变");
}

/// 场景（攻方 `front_hole_eio 4 9 mount`）：实例 1 写到 3，实例 2 覆盖写到 8，txg 9 那次覆盖写崩在根槽写之前
/// （记录已持久；环还没转过来，那一槽从没写过、盘上全零）；实例 3 可写挂载那一刻读那一槽报读错，之后照读；覆盖写到 9 + 2 × 3S。
/// 预期：那一槽钉在它可能住过的最旧 txg 9；环里还有比 9 旧的有效根时不重读，txg 20 那一版它成了根环表里最小的那个、重读读回全零：
/// 不算确认为空，照旧钉在 9（用户 2026-10-05 定乙），txg 21 这个进程写掉那一槽才解开。被引用单元那几条每一版都不红；只有 txg 20 那一版
/// I-3.1 红、只许红在记账多的一边——钉着的 9 比盘上环里最旧的有效根 10 旧，池级 checker 读盘上全零的那一槽、按 10 算门槛，安全方向，
/// 定案的代价（与改法 A 当初认下的是同一版）。照「从没写过」拿掉时 txg 20 那一版也不红。
#[test]
fn a_front_hole_unreadable_at_mount_reread_as_all_zero_stays_pinned_until_the_process_overwrites_it(
) {
    let parameters = parameters_of_the_ring_hole_pools();
    let image_with_the_front_hole = {
        let mut devices: RingHoleDevices = devices_of(&pool_overwritten_through(3));
        let mounted = mount_writable(&parameters, &mut devices).expect("实例 2 的可写挂载");
        let instance = mounted.output.instance;
        let mut allocator = mounted.allocator.clone();
        let mut current = mounted.current.into_file_version().expect("这一版带文件");
        // 迭代上界：覆盖写到 txg 8。
        while current.root.checkpoint_txg.0 + 1 < TXG_OF_THE_FRONT_HOLE {
            let salt = current.root.checkpoint_txg.0 + 1;
            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
            current = publish_overwrite(
                &mut writer,
                &mut allocator,
                &current,
                FirstFile {
                    content: &content_of_the_version(salt),
                    write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + salt,
                },
                instance,
            )
            .expect("覆盖写");
        }
        let before = pool_of(&devices);
        let stream = SharedStream::retaining_contents();
        let mut recording = recording_of(&before, &stream);
        {
            let mut writer = PoolWriter::new(&parameters, recording.as_mut_slice());
            let published = publish_overwrite(
                &mut writer,
                &mut allocator.clone(),
                &current,
                FirstFile {
                    content: &content_of_the_version(TXG_OF_THE_FRONT_HOLE),
                    write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS
                        + TXG_OF_THE_FRONT_HOLE,
                },
                instance,
            )
            .expect("洞那一次覆盖写");
            assert_eq!(
                published.root.checkpoint_txg,
                CheckpointTxg(TXG_OF_THE_FRONT_HOLE)
            );
        }
        let operations = stream.retained_operations();
        let (writes, _segments) =
            writes_and_segments(&operations, &fixed_geometry_of_the_ring_hole_pools());
        let root_write = writes
            .iter()
            .position(|write| write.kind == StepKind::RootRecordFua)
            .expect("那一次的根槽写");
        let mut crashed = before;
        crashed.apply_writes(&writes[..root_write]);
        crashed
    };
    let mut session = SessionOnARingHolePool::mounted_with_the_slot_of_txg_unreadable_at_mount(
        image_with_the_front_hole,
        TXG_OF_THE_FRONT_HOLE,
    );
    assert_eq!(
        session.txgs_recorded_as_bad_at_mount(),
        Some(TxgsARingSlotBadAtMountCanHold::new(
            CheckpointTxg(TXG_OF_THE_FRONT_HOLE),
            CheckpointTxg(TXG_OF_THE_FRONT_HOLE)
        )),
        "前提：根环表把那一槽记成挂载时读坏，钉在 9（能住的最大 txg 也是 9）"
    );
    let ring_length = 3 * parameters.geometry.root_ring_slots_per_region.count();
    let mut violations_of_each_version: BTreeMap<u64, Vec<String>> = BTreeMap::new();
    // 迭代上界：覆盖写到 9 + 2 × 3S；跨轮携带的是现行那一版与红过的版本。
    while session.current.root.checkpoint_txg.0 < TXG_OF_THE_FRONT_HOLE + 2 * ring_length {
        session.overwrite_once();
        let violations = violations_of(&session.image_now());
        if !violations.is_empty() {
            violations_of_each_version.insert(session.current.root.checkpoint_txg.0, violations);
        }
        let txg = session.current.root.checkpoint_txg.0;
        if txg == TXG_AT_WHICH_THE_FRONT_HOLE_IS_REREAD - 1 {
            assert!(
                session.txgs_recorded_as_bad_at_mount().is_some(),
                "txg {txg}：环里还有比 9 旧的 txg 8，那一槽没重读、仍钉着"
            );
        }
        if txg == TXG_AT_WHICH_THE_FRONT_HOLE_IS_REREAD {
            assert_eq!(
                (
                    session.txgs_recorded_as_bad_at_mount(),
                    session.slot_is_known_to_hold_a_root()
                ),
                (
                    Some(TxgsARingSlotBadAtMountCanHold::new(
                        CheckpointTxg(TXG_OF_THE_FRONT_HOLE),
                        CheckpointTxg(TXG_OF_THE_FRONT_HOLE)
                    )),
                    false
                ),
                "txg 20：重读读回全零，不算确认为空，照旧钉在 9、不算知道住着根的槽"
            );
        }
        if txg == TXG_AT_WHICH_THE_FRONT_HOLE_IS_REREAD + 1 {
            assert_eq!(
                (
                    session.txgs_recorded_as_bad_at_mount(),
                    session.slot_is_known_to_hold_a_root()
                ),
                (None, true),
                "txg 21：这个进程写掉那一槽，不再钉着、算知道住着根的槽"
            );
        }
    }
    assert_eq!(
        violations_of_each_version
            .keys()
            .copied()
            .collect::<Vec<u64>>(),
        vec![TXG_AT_WHICH_THE_FRONT_HOLE_IS_REREAD],
        "只有重读读回全零、照旧钉着的 txg 20 那一版红：{violations_of_each_version:?}"
    );
    assert!(
        violations_of_each_version
            .values()
            .flatten()
            .all(|violation| violation.starts_with("I-3.1：")
                && allocated_statistic_counts_more_than_the_walk(violation)),
        "txg 20 那一版只许 I-3.1 红在记账多的一边：{violations_of_each_version:?}"
    );
}

/// I-3.1 那一条红在记账比遍历多的一边：说明里「记账的已分配 Some(a)，遍历全部有效根得到 b」a 大于 b
/// （与 `raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window.rs` 同一个判法另写一份）。
fn allocated_statistic_counts_more_than_the_walk(violation: &str) -> bool {
    let after = |marker: &str| {
        violation.split(marker).nth(1).and_then(|rest| {
            rest.chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>()
                .parse::<u64>()
                .ok()
        })
    };
    matches!(
        (after("记账的已分配 Some("), after("遍历全部有效根得到 ")),
        (Some(counted), Some(walked)) if counted > walked
    )
}

/// 场景：实例 1 覆盖写到 txg 5（最新那条根 a = 5）；崩溃恢复抛弃 a（它的根槽与数据单元一时读不出，择根落到 4，实例 2 写行 6、暖机 7），
/// 实例 2 再覆盖写两次（8、9）；实例 3 可写挂载那一刻读 a 那一槽报读错，之后照读；覆盖写到 a + 3S − 1 = 28（S = 8 的 24 槽环）。
/// 预期：记 txg 28 那条根之前 a 那一槽成了根环表里最小的那个（txg 0–4 已被盖掉），重读读出 txg 5 的根，按写行那一张实例表（实例 1 停在 4）被抛弃：当没有根，
/// 那一槽从根环表拿掉、不算这个进程知道住着根的槽；txg 28 那一版那几条不变量（不含 I-7.4，见 [`INVARIANTS_JUDGED_ON_THE_ABANDONED_ROOT`]）不红。
#[test]
fn an_abandoned_root_read_on_the_reread_is_taken_as_no_root() {
    let mut pool = build_pool("abandoned-root-read-on-the-e2-reread");
    // 迭代上界：txg 4、5 两次覆盖写。
    for _ in 0..2 {
        let content = content_of_the_version(pool.output.root.checkpoint_txg.0 + 1);
        overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, &content);
    }
    let newest = pool.output.clone();
    let txg_of_the_abandoned_root = newest.root.checkpoint_txg.0;
    let mounted_as_instance_two =
        abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &newest);
    assert_eq!(
        mounted_as_instance_two.output.instance,
        InstanceGeneration(2),
        "前提：实例 2 抛弃了 a"
    );
    // 迭代上界：实例 2 两次覆盖写。
    for _ in 0..2 {
        let previous = pool.output.clone();
        pool.output = common::publish_overwrite_in_process(
            &mut pool,
            &previous,
            &content_of_the_version(previous.root.checkpoint_txg.0 + 1),
            FIXED_WRITE_TIME_SECONDS + previous.root.checkpoint_txg.0 + 1,
            InstanceGeneration(2),
        )
        .expect("实例 2 覆盖写");
    }
    let parameters = common::parameters();
    let fault = FaultOfOneRootRingSlot::of_the_root_slot_of_txg(
        &parameters,
        CheckpointTxg(txg_of_the_abandoned_root),
        ReadsOfTheRootRingSlot::DeviceErrorOnEveryRead,
    );
    let mut devices = with_the_fault(pool.reopen_recorded(), &fault);
    let mounted = mount_writable(&parameters, &mut devices).expect("实例 3 的可写挂载");
    fault.set_reads(ReadsOfTheRootRingSlot::PassedThrough);
    assert!(fault.failed_reads() > 0, "挂载那一次读 a 那一槽读错了");
    let ring_slot = target_for_publish(
        CheckpointTxg(txg_of_the_abandoned_root),
        parameters.geometry.root_ring_slots_per_region,
    );
    let mut allocator = mounted.allocator;
    let occupancy_of = |allocator_of_the_mount: &PoolAllocator| {
        let occupancy = allocator_of_the_mount
            .root_ring_occupancy()
            .expect("可写挂载装了根环表");
        (
            occupancy.ring_slots_bad_at_mount().get(&ring_slot).copied(),
            occupancy
                .ring_slots_known_to_hold_a_root()
                .contains(&ring_slot),
        )
    };
    assert_eq!(
        occupancy_of(&allocator),
        (
            Some(TxgsARingSlotBadAtMountCanHold::new(
                CheckpointTxg(txg_of_the_abandoned_root),
                CheckpointTxg(txg_of_the_abandoned_root)
            )),
            false
        ),
        "前提：根环表把 a 那一槽记成挂载时读坏，钉在 5（环还没转过来，能住的最大 txg 也是 5）"
    );
    let mut current = mounted.current.into_file_version().expect("这一版带文件");
    let ring_length = 3 * parameters.geometry.root_ring_slots_per_region.count();
    let txg_at_which_the_slot_is_reread = txg_of_the_abandoned_root + ring_length - 1;
    // 迭代上界：覆盖写到 a + 3S − 1。
    while current.root.checkpoint_txg.0 < txg_at_which_the_slot_is_reread {
        let checkpoint_txg = current.root.checkpoint_txg.0 + 1;
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        current = publish_overwrite(
            &mut writer,
            &mut allocator,
            &current,
            FirstFile {
                content: &content_of_the_version(checkpoint_txg),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + checkpoint_txg,
            },
            InstanceGeneration(3),
        )
        .expect("实例 3 覆盖写");
    }
    assert_eq!(
        occupancy_of(&allocator),
        (None, false),
        "重读读出被抛弃的根：当没有根，从根环表拿掉，不算知道住着根的槽"
    );
    assert_eq!(
        violations_among(
            &pool.memory_pool(),
            &INVARIANTS_JUDGED_ON_THE_ABANDONED_ROOT
        ),
        Vec::<String>::new(),
        "txg {txg_at_which_the_slot_is_reread} 那一版"
    );
}

/// 场景：实例 1 覆盖写到 txg 3 之后可写挂载（实例 2 写行）。
/// 预期：挂载交回的分配器上，进程内重读判「按实例表有效」用的那张表是写行那次发布写下的那一张（带实例 1 的行），
/// 不是挂载时最新那条根指着的那一张——池级 checker 按最新那条根的实例表判有效，写行之后它带的就是写行那一张。
#[test]
fn the_instance_table_judging_the_rereads_after_the_mount_is_the_one_the_row_publish_wrote() {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut devices = devices_of(&pool_overwritten_through(3));
    let mounted = mount_writable(&parameters, &mut devices).expect("可写挂载");
    let table_of = |root: &RootRecord| {
        instance_table_chain_of_root(&pool_of(&devices), root)
            .expect("实例表读得出")
            .records
    };
    let table_of_the_row_publish = table_of(mounted.output.row_publish.root());
    assert_ne!(
        table_of_the_row_publish,
        table_of(&mounted.output.chosen_root),
        "前提：写行那一张比挂载时最新那条根的多实例 1 那一行"
    );
    assert_eq!(
        mounted
            .allocator
            .root_ring_occupancy()
            .expect("可写挂载装了根环表")
            .instance_table_judging_the_rereads(),
        &table_of_the_row_publish,
        "判有效用写行那一张"
    );
}

/// 树表 0 条的池上一次可写挂载：录下来，崩在 txg 落在 `crash_before_the_root_of` 里的第一个根槽写之前（那次发布的单元与记录已持久、根槽没持久；
/// 这次挂载没写那几个 txg 的根就整段落盘）。交回之后的盘面与这次挂载落盘的最后一个 txg（崩了的那一次是它的 txg）。
fn mount_a_pool_without_file(
    image: &MemoryPool,
    crash_before_the_root_of: &[u64],
) -> (MemoryPool, u64) {
    let parameters = parameters_of_the_ring_hole_pools();
    let stream = SharedStream::retaining_contents();
    let mut recording = recording_of(image, &stream);
    mount_writable(&parameters, &mut recording).expect("树表 0 条的池可写挂载");
    let (writes, _segments) = writes_and_segments(
        &stream.retained_operations(),
        &fixed_geometry_of_the_ring_hole_pools(),
    );
    let txg_of_the_root_written_by = |write: &singlefs_harness::memory_pool::RetainedWrite| {
        write
            .bytes()
            .and_then(|bytes| RootRecord::parse_slot(bytes, &parameters.filesystem_identifier))
            .map(|root| root.checkpoint_txg.0)
    };
    let cut = writes.iter().position(|write| {
        write.kind == StepKind::RootRecordFua
            && txg_of_the_root_written_by(write)
                .is_some_and(|txg| crash_before_the_root_of.contains(&txg))
    });
    let mut after = image.clone();
    match cut {
        Some(cut) => {
            let txg = txg_of_the_root_written_by(&writes[cut]).expect("根槽写带着一条根");
            after.apply_writes(&writes[..cut]);
            (after, txg)
        }
        None => {
            after.apply_writes(&writes);
            let last = writes
                .iter()
                .filter(|write| write.kind == StepKind::RootRecordFua)
                .filter_map(txg_of_the_root_written_by)
                .max()
                .expect("可写挂载至少写一条根");
            (after, last)
        }
    }
}

/// mkfs 之后一直不写文件，接连可写挂载、崩掉 `crash_before_the_root_of` 里那几个 txg 的根，到下一次挂载的写行是 `first_txg_of_the_next_mount` 为止。
fn pool_without_file_before_the_mount_writing_its_row_at(
    crash_before_the_root_of: &[u64],
    first_txg_of_the_next_mount: u64,
) -> MemoryPool {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut devices: RingHoleDevices = (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(
                    DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
                    PhysicalBlockSizeInBytes(512),
                ),
            )
        })
        .collect();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut image = pool_of(&devices);
    let mut last_txg = 0u64;
    // 迭代上界：每次挂载至少前进一个 txg，到下一次挂载的写行 txg 为止；跨轮携带的是盘面与落盘的最后一个 txg。
    while last_txg + 1 < first_txg_of_the_next_mount {
        let (after, last) = mount_a_pool_without_file(&image, crash_before_the_root_of);
        assert!(last > last_txg, "每次挂载都往前推");
        image = after;
        last_txg = last;
    }
    assert_eq!(
        last_txg + 1,
        first_txg_of_the_next_mount,
        "前提：下一次挂载的写行落在 {first_txg_of_the_next_mount}"
    );
    image
}

/// 场景：树表 0 条的池（S = 4），mkfs 之后接连可写挂载、从不写文件；txg 14 那次写行崩在根槽写之前（那一槽里留着上一圈 txg 2 的根），
/// txg 23 那次写行同样崩在根槽写之前；下一次可写挂载写行 24、暖机 25（两块盘各一条）。挂载那一刻（取号之前）txg 14 那一槽读不出，之后照读。
/// 预期：那一槽钉在它可能住过的最旧 txg 2，环里没有比它更旧的有效根（txg 23 那个洞里留着的是 11）：写行 24 那次记根之前它是根环表里最小的那个，
/// 重读读出 txg 2 的根（按写行那一张实例表有效）；挂载交回的根环表里那一槽不再按读坏记、算这个进程知道住着根的槽。
/// 这一条钉的是树表 0 条那一版上的写行发布（`transaction::prepare_the_row_publish_on_a_version_without_file`）走到 E2 重读。
#[test]
fn the_row_publish_on_a_version_without_file_rereads_the_slot_unreadable_at_mount_pinned_at_the_oldest_txg_it_can_have_held(
) {
    let image = pool_without_file_before_the_mount_writing_its_row_at(&[14, 23], 24);
    let parameters = parameters_of_the_ring_hole_pools();
    let fault = FaultOfOneRootRingSlot::of_the_root_slot_of_txg(
        &parameters,
        CheckpointTxg(14),
        ReadsOfTheRootRingSlot::DeviceErrorUntilTheFirstWriteOfAnyDevice,
    );
    let stream = SharedStream::retaining_contents();
    let mut devices = with_the_fault(recording_of(&image, &stream), &fault);
    let mounted = mount_writable(&parameters, &mut devices).expect("可写挂载");
    assert_eq!(
        (
            mounted.output.row_publish.root().checkpoint_txg,
            mounted
                .output
                .warm_up_publishes
                .iter()
                .map(|version| version.root().checkpoint_txg)
                .collect::<Vec<_>>()
        ),
        (CheckpointTxg(24), vec![CheckpointTxg(25)]),
        "前提：写行 24、暖机一次 25"
    );
    assert!(fault.failed_reads() > 0, "前提：挂载读阶段读那一槽读错");
    let ring_slot = target_for_publish(
        CheckpointTxg(14),
        parameters.geometry.root_ring_slots_per_region,
    );
    let occupancy = mounted
        .allocator
        .root_ring_occupancy()
        .expect("可写挂载装了根环表");
    assert_eq!(
        (
            occupancy.ring_slots_bad_at_mount().get(&ring_slot).copied(),
            occupancy
                .ring_slots_known_to_hold_a_root()
                .contains(&ring_slot)
        ),
        (None, true),
        "写行那次记根之前重读那一槽、读出 txg 2 的根"
    );
}
