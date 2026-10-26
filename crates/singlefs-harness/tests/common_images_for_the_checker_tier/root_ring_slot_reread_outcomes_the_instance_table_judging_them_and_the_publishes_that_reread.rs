//! harness 测试文件 `crates/singlefs-harness/tests/root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool,
    overwrite_in_process_as_instance_one_and_advance_the_pool,
    BYTE_INSIDE_THE_ROOT_RECORDS_TREE_TABLE_POINTER, FIXED_WRITE_TIME_SECONDS,
};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use crate::common_root_ring_holes::{
    content_of_the_version, devices_of, fixed_geometry_of_the_ring_hole_pools,
    parameters_of_the_ring_hole_pools, pool_of, pool_overwritten_through, recording_of,
    RingHoleDevices, WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::allocator::{PoolAllocator, TxgsARingSlotBadAtMountCanHold};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::mount_writable;
use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};
use singlefs_harness::memory_pool::{writes_and_segments, MemoryPool, SparseBlockDevice};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};
use std::cell::RefCell;
use std::rc::Rc;

/// 判的几条：记账与遍历（I-3.1、I-3.11）、被引用单元读回来对不对（I-2.1、I-7.4、I-4.8）。
pub const INVARIANTS_THE_REREAD_KEEPS_GREEN: [&str; 5] =
    ["I-3.1", "I-3.11", "I-2.1", "I-7.4", "I-4.8"];

/// 攻方那一形：实例 1 覆盖写到 txg 20，有效根 h = 18 所在的槽（区域 0、槽 2）挂载那一刻读错。
pub const LAST_TXG_OF_INSTANCE_ONE: u64 = 20;

pub const TXG_OF_THE_VALID_ROOT_UNREADABLE_AT_MOUNT: u64 = 18;

/// h 那一槽（区域 0、槽 2）可能住过的最旧 txg：2 × 3 + 0（S = 4）。
pub const OLDEST_TXG_THE_SLOT_OF_THE_VALID_ROOT_CAN_HAVE_HELD: u64 = 6;

/// 前洞那一形：实例 2 的 txg 9 那次覆盖写崩在根槽写之前（环还没转过来，那一槽从没写过）。
pub const TXG_OF_THE_FRONT_HOLE: u64 = 9;

/// 前洞那一形里那一槽成了根环表里最小的那个的那一版：9 + 3S − 1（txg 8 那一槽被盖掉）。
pub const TXG_AT_WHICH_THE_FRONT_HOLE_IS_REREAD: u64 = 20;

/// 被抛弃根那一形判的几条：同上去掉 I-7.4。被抛弃的根所在的槽挂载那一刻读不出，影子账罩不到它引用的单元（挂载那一刻就读不到那条根，
/// 与重读无关：攻方 `research/prompts/m3-root-ring-code-r2-opus-output.md` K1 第 3 小节「被抛弃」那一条），之后那几个单元被合法复用，
/// I-7.4 在这一形上红是今天已知的那一格，不归这条用例判。
pub const INVARIANTS_JUDGED_ON_THE_ABANDONED_ROOT: [&str; 4] =
    ["I-3.1", "I-3.11", "I-2.1", "I-4.8"];

/// 交 checker 档：重读不越过的那几条（[`INVARIANTS_THE_REREAD_KEEPS_GREEN`]）一条都不红。
pub fn hand_out_expecting_the_reread_keeps_its_invariants_green(
    images: &mut PoolCheckerImages,
    label: &str,
    build_the_image: impl FnOnce() -> MemoryPool,
) {
    images.hand_out(
        label,
        build_the_image,
        PoolCheckerExpectation::only_the_named_invariants()
            .and_none_of_these_violated(&INVARIANTS_THE_REREAD_KEEPS_GREEN),
    );
}

/// 一个根环槽的读这会儿怎么坏。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadsOfTheRootRingSlot {
    PassedThrough,
    DeviceErrorOnEveryRead,
    /// 任何一块盘上发出第一个写（取号写系统配置）之前每次读都报设备错，之后照读：只在可写挂载读阶段读错。
    DeviceErrorUntilTheFirstWriteOfAnyDevice,
    /// 只有下一次读坏：读得出，那一槽里一个字节取反（自证不过，盘上字节不变）；之后照读。
    OneByteFlippedOnTheNextReadOnly,
}

pub struct FaultOfOneRootRingSlotState {
    pub device: DeviceIdentity,
    pub offset_in_bytes: u64,
    pub length_in_bytes: u64,
    pub reads: ReadsOfTheRootRingSlot,
    pub a_write_was_sent: bool,
    pub failed_reads: u64,
    pub flipped_reads: u64,
}

/// 几块盘共用的、一个根环槽的读故障。
#[derive(Clone)]
pub struct FaultOfOneRootRingSlot(Rc<RefCell<FaultOfOneRootRingSlotState>>);

pub enum ReadOutcome {
    Read,
    DeviceError,
    FlipTheByteAt(usize),
}

impl FaultOfOneRootRingSlot {
    pub fn of_the_root_slot_of_txg(
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

    pub fn set_reads(&self, reads: ReadsOfTheRootRingSlot) {
        self.0.borrow_mut().reads = reads;
    }

    pub fn failed_reads(&self) -> u64 {
        self.0.borrow().failed_reads
    }

    pub fn flipped_reads(&self) -> u64 {
        self.0.borrow().flipped_reads
    }

    pub fn note_a_write(&self) {
        self.0.borrow_mut().a_write_was_sent = true;
    }

    pub fn note_a_read(
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
pub struct DeviceWithOneRootRingSlotFaulted<Inner: BlockDevice> {
    pub identity: DeviceIdentity,
    pub inner: Inner,
    pub fault: FaultOfOneRootRingSlot,
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

pub fn with_the_fault<Inner: BlockDevice>(
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

pub type RingHoleDevicesWithTheFault = Vec<(
    DeviceIdentity,
    DeviceWithOneRootRingSlotFaulted<RecordingBlockDevice<SparseBlockDevice>>,
)>;

/// S = 4 的小池上一次可写挂载之后的会话：盘（一个根环槽带读故障）、录下的流与它的起点盘面、分配器、现行那一版。
pub struct SessionOnARingHolePool {
    pub image_before_the_mount: MemoryPool,
    pub stream: SharedStream,
    pub devices: RingHoleDevicesWithTheFault,
    pub fault: FaultOfOneRootRingSlot,
    pub ring_slot: RootRingSlot,
    pub allocator: PoolAllocator,
    pub current: TransactionOutput,
    pub instance: InstanceGeneration,
}

impl SessionOnARingHolePool {
    /// `image` 上可写挂载，txg 为 `txg_of_the_slot` 的那一槽挂载那一刻每次读都报设备错，挂载之后照读。
    pub fn mounted_with_the_slot_of_txg_unreadable_at_mount(
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

    pub fn image_now(&self) -> MemoryPool {
        let mut image = self.image_before_the_mount.clone();
        image.apply(&self.stream.retained_operations());
        image
    }

    pub fn overwrite_once(&mut self) {
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
    pub fn txgs_recorded_as_bad_at_mount(&self) -> Option<TxgsARingSlotBadAtMountCanHold> {
        self.allocator
            .root_ring_occupancy()
            .expect("可写挂载装了根环表")
            .ring_slots_bad_at_mount()
            .get(&self.ring_slot)
            .copied()
    }

    pub fn slot_is_known_to_hold_a_root(&self) -> bool {
        self.allocator
            .root_ring_occupancy()
            .expect("可写挂载装了根环表")
            .ring_slots_known_to_hold_a_root()
            .contains(&self.ring_slot)
    }
}

/// 攻方那一形挂载之后、重读之前：实例 1 覆盖写到 20；实例 2 可写挂载那一刻读 h = 18 那一槽报读错（挂载里写行、暖机记根时的重读也读错）。
/// 那一槽钉在它可能住过的最旧 txg 6（能住的最大 txg 18），是根环表里最小的那个：挂载之后第一次记根就重读。
pub fn session_right_after_the_mount_before_the_reread() -> SessionOnARingHolePool {
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
pub fn a_valid_root_read_bad_once_on_the_reread_keeps_the_slot_pinned_at_the_oldest_txg_it_can_have_held(
    images: &mut PoolCheckerImages,
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
    // 重读读坏那一版：门槛没越过盘上 txg 18 那条有效根，被引用单元那几条不红（I-3.1 红不红不管）
    images.hand_out(
        "重读读到自证不过那一版",
        || session.image_now(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_none_of_these_violated(&INVARIANTS_THE_REREAD_KEEPS_GREEN[1..]),
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
    hand_out_expecting_the_reread_keeps_its_invariants_green(
        images,
        "下一次记根再重读、读出 txg 18 之后那一版",
        || session.image_now(),
    );
}

/// 场景：同一盘面，挂载之后第一次覆盖写那一次重读读得出（读故障只在挂载那一刻）。
/// 预期：读出 txg 18 的根、按实例表有效：那一槽换成「重读读出的有效根 18」，算这个进程知道住着根的槽；那一版那几条不变量不红。
pub fn a_valid_root_read_on_the_reread_holds_the_reclaim_threshold_at_its_txg(
    images: &mut PoolCheckerImages,
) {
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
    // 门槛照读出来的 18 算
    hand_out_expecting_the_reread_keeps_its_invariants_green(
        images,
        "重读读出有效根 18 之后那一版",
        || session.image_now(),
    );
}

/// 场景（攻方 `front_hole_eio 4 9 mount`）：实例 1 写到 3，实例 2 覆盖写到 8，txg 9 那次覆盖写崩在根槽写之前
/// （记录已持久；环还没转过来，那一槽从没写过、盘上全零）；实例 3 可写挂载那一刻读那一槽报读错，之后照读；覆盖写到 9 + 2 × 3S。
/// 预期：那一槽钉在它可能住过的最旧 txg 9；环里还有比 9 旧的有效根时不重读，txg 20 那一版它成了根环表里最小的那个、重读读回全零：
/// 不算确认为空，照旧钉在 9（用户 2026-10-05 定乙），txg 21 这个进程写掉那一槽才解开。每一版那几条（[`INVARIANTS_THE_REREAD_KEEPS_GREEN`]）
/// 都不红，txg 20 那一版也一样：这是 C773（读坏与全零根槽的回收门槛钉只在内存里）定案「钉也落盘」之后的口径——根记录带着钉出来的
/// 回收门槛，池级 checker 把「钉带出来的那一截」不看引用并进遍历得到的和（`walk::released_slots_the_pin_kept_in_defer`），
/// 盘上环里最旧的有效根 10 与钉着的 9 之间那一截不再落在记账多的一边。C773 之前钉只在内存里，checker 按 10 算门槛，
/// txg 20 那一版 I-3.1 红在记账多的一边，是当时认下的代价。
pub fn a_front_hole_unreadable_at_mount_reread_as_all_zero_stays_pinned_until_the_process_overwrites_it(
    images: &mut PoolCheckerImages,
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
    // 迭代上界：覆盖写到 9 + 2 × 3S；跨轮携带的是现行那一版与红过的版本。
    while session.current.root.checkpoint_txg.0 < TXG_OF_THE_FRONT_HOLE + 2 * ring_length {
        session.overwrite_once();
        let txg = session.current.root.checkpoint_txg.0;
        // 每一版那几条都不红；重读读回全零、照旧钉着的 txg 20 那一版也一样（C773 钉落盘之后，钉带出来的那一截并进遍历），
        // 只在名字上标出来
        let label = if txg == TXG_AT_WHICH_THE_FRONT_HOLE_IS_REREAD {
            format!("覆盖写到 txg {txg}（重读读回全零、照旧钉着）")
        } else {
            format!("覆盖写到 txg {txg}")
        };
        hand_out_expecting_the_reread_keeps_its_invariants_green(images, &label, || {
            session.image_now()
        });
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
}

/// 场景：实例 1 覆盖写到 txg 5（最新那条根 a = 5）；崩溃恢复抛弃 a（它的根槽与数据单元一时读不出，择根落到 4，实例 2 写行 6、暖机 7），
/// 实例 2 再覆盖写两次（8、9）；实例 3 可写挂载那一刻读 a 那一槽报读错，之后照读；覆盖写到 a + 3S − 1 = 28（S = 8 的 24 槽环）。
/// 预期：记 txg 28 那条根之前 a 那一槽成了根环表里最小的那个（txg 0–4 已被盖掉），重读读出 txg 5 的根，按写行那一张实例表（实例 1 停在 4）被抛弃：当没有根，
/// 那一槽从根环表拿掉、不算这个进程知道住着根的槽；txg 28 那一版那几条不变量（不含 I-7.4，见 [`INVARIANTS_JUDGED_ON_THE_ABANDONED_ROOT`]）不红。
pub fn an_abandoned_root_read_on_the_reread_is_taken_as_no_root(images: &mut PoolCheckerImages) {
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
        pool.output = crate::common::publish_overwrite_in_process(
            &mut pool,
            &previous,
            &content_of_the_version(previous.root.checkpoint_txg.0 + 1),
            FIXED_WRITE_TIME_SECONDS + previous.root.checkpoint_txg.0 + 1,
            InstanceGeneration(2),
        )
        .expect("实例 2 覆盖写");
    }
    let parameters = crate::common::parameters();
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
    images.hand_out(
        &format!("txg {txg_at_which_the_slot_is_reread} 那一版"),
        || pool.memory_pool(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_none_of_these_violated(&INVARIANTS_JUDGED_ON_THE_ABANDONED_ROOT),
    );
}
