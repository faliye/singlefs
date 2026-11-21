//! 会话发布用户改动（`MountedSession::publish_user_change`）进循环之前逐盘比这一刻探测到的 `physical_block_size` 与会话持着的那份参数里
//! 记的 mkfs 时的值：D12（目标介质） 已定项 4 比对清单第一行「拒绝挂载（可写与只读都拒）」，会话每次用户改动都比（与可写挂载、
//! 挂着之后收盘表的入口同一判，`mount::physical_block_sizes_probed_on_the_devices_differing_from_the_system_configuration`），C1040。
//! 两块盘同构的池挂着、发过几版之后，把盘 1 换成字节逐个相同、而探测值改一档的盘，会话发一次覆盖写：拒成点名那块盘、探测值与记录值的
//! `PhysicalBlockSizesProbedOnTheDevicesDifferFromTheSystemConfiguration`，交进来的两块盘一个读、写、写零、discard、屏障都没收到，
//! 逐字节不变，会话的现行那一版不动。对照：同一个会话、同一份字节交回探测值都是记录值的两块盘，同一个覆盖写照常发成、两块盘都被写。
//! 两条用例各管一个方向：记录 4096、盘 1 探测 512（不比时每个写都对得齐、照常发成）；记录 512、盘 1 探测 4096（不比时读 512 宽的根槽
//! 报不对齐，写照样落到那块盘上）。
//! 池：6 MiB 环、单元区 2048 槽的两块小盘（`tests/common_admission_pool/mod.rs` 的建池写法，物理块与 io_min 换成记录值）。

mod common_admission_pool;

use std::cell::RefCell;
use std::rc::Rc;

use common_admission_pool::{
    content_of, parameters_of_the_384_slot_small_devices, PoolUnderTest, OVERWRITE_BYTES,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::allocator::UnitAreaStart;
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::PhysicalBlockSizeProbedOnADeviceDifferingFromTheSystemConfiguration;
use singlefs_core::mounted_session::{
    MountedSession, UserChange, UserChangePublished, UserChangeRefused,
};
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::FirstFile;
use singlefs_format::SLOT_BYTES;
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};

/// 单元区槽数（与 `accept_raises_the_floor_for_space.rs` 的 `UNIT_AREA_SLOTS_OF_THE_SMALLER_DEVICES` 同一个数）。
const UNIT_AREA_SLOTS: u64 = 2048;
/// 小盘为环窗口补足的槽数（同 `accept_raises_the_floor_for_space.rs` 的 `RING_WINDOW_ALLOWANCE_SLOTS`）。
const RING_WINDOW_ALLOWANCE_SLOTS: u64 = 1007;
/// 换盘之前第一个文件之后覆盖写的次数（同 `entries_after_a_writable_mount_refuse_a_device_without_a_self_verified_system_configuration.rs`）。
const OVERWRITES_BEFORE_THE_SWAP: usize = 2;
/// 打印结局时最多留几个字符。
const DEBUG_CHARACTERS_KEPT: usize = 1500;
/// 盘表里的两块盘。
const DEVICE_IDENTITIES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

/// 交到包装这一层的一次调用的种类（不论底下那块盘收没收）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CallKind {
    Read,
    Write,
    WriteZeroes,
    Discard,
    Barrier,
}

impl CallKind {
    fn changes_the_medium(self) -> bool {
        match self {
            CallKind::Write | CallKind::WriteZeroes | CallKind::Discard => true,
            CallKind::Read | CallKind::Barrier => false,
        }
    }
}

/// 交到包装这一层的一次调用：盘与种类。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Call {
    device: DeviceIdentity,
    kind: CallKind,
}

type CallLog = Rc<RefCell<Vec<Call>>>;

/// 稀疏盘外面包一层，逐次记每一个读、写、写零、discard、屏障（含底下那块盘拒掉的），按发出次序记进两块盘共用的一份；探测不记。
struct DeviceCountingEveryCall {
    device: DeviceIdentity,
    inner: SparseBlockDevice,
    log: CallLog,
}

impl DeviceCountingEveryCall {
    fn note(&self, kind: CallKind) {
        self.log.borrow_mut().push(Call {
            device: self.device,
            kind,
        });
    }
}

impl BlockDevice for DeviceCountingEveryCall {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.note(CallKind::Read);
        self.inner.read_at(offset, buffer)
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.note(CallKind::Write);
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.note(CallKind::WriteZeroes);
        self.inner.write_zeroes_at(offset, length)
    }

    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        self.note(CallKind::Discard);
        self.inner.discard_at(offset, length)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.note(CallKind::Barrier);
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

/// 物理块与 io_min 都是 `recorded` 的小盘参数（别的照 `parameters_of_the_384_slot_small_devices`：6 MiB 环）。
fn parameters_recording(recorded: u32) -> MakeFilesystemParameters {
    let mut parameters = parameters_of_the_384_slot_small_devices();
    parameters.geometry.physical_block_size = recorded;
    parameters.geometry.minimum_input_output_bytes = recorded;
    parameters.geometry.fixed_structure_slot_spacing =
        SystemImmutableSizes::slot_spacing_for(recorded);
    parameters
}

fn device_bytes() -> u64 {
    let unit_area_start_slot = UnitAreaStart::following_the_journal_ring(
        parameters_of_the_384_slot_small_devices()
            .geometry
            .journal_ring_bytes,
    )
    .expect("6 MiB 的环末尾的下一个槽落在聚簇段边界上")
    .slot()
    .0;
    (unit_area_start_slot + RING_WINDOW_ALLOWANCE_SLOTS + UNIT_AREA_SLOTS) * SLOT_BYTES
}

/// 两块探测值都是 `recorded` 的盘上 mkfs、取号、暖机、第一个文件，再覆盖写 [`OVERWRITES_BEFORE_THE_SWAP`] 次；交回池与摘下来的会话。
fn pool_recording(recorded: u32) -> (PoolUnderTest<SparseBlockDevice>, MountedSession) {
    let bytes = device_bytes();
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_recording(recorded),
        bytes,
        move |_identity: DeviceIdentity, _device_probing_512: SparseBlockDevice| {
            SparseBlockDevice::new(bytes, PhysicalBlockSizeInBytes(recorded))
        },
    );
    for overwrite in 0..OVERWRITES_BEFORE_THE_SWAP {
        if let Err(refusal) = pool.overwrite(OVERWRITE_BYTES) {
            panic!(
                "起池第 {overwrite} 次覆盖写发得成，实际 {}",
                outcome_line(&Err(refusal))
            );
        }
    }
    let session = pool.session.take().expect("起池之后有会话");
    (pool, session)
}

/// 镜像里的两块盘逐块装进探测值各为 `probes`（按盘表次序）的稀疏盘，外面包上记调用的那一层。
fn counting_devices_on(
    image: &MemoryPool,
    probes: [u32; 2],
    log: &CallLog,
) -> Vec<(DeviceIdentity, DeviceCountingEveryCall)> {
    DEVICE_IDENTITIES
        .into_iter()
        .zip(probes)
        .map(|(identity, probe)| {
            let mut inner =
                SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(probe));
            inner.image = image
                .devices
                .get(&identity)
                .cloned()
                .expect("镜像里有这块盘");
            (
                identity,
                DeviceCountingEveryCall {
                    device: identity,
                    inner,
                    log: Rc::clone(log),
                },
            )
        })
        .collect()
}

/// 结局一行（Debug，截在 [`DEBUG_CHARACTERS_KEPT`] 个字符）：失败消息里用。
fn outcome_line(outcome: &Result<UserChangePublished, UserChangeRefused>) -> String {
    let text = format!("{outcome:?}");
    let total = text.chars().count();
    if total <= DEBUG_CHARACTERS_KEPT {
        text
    } else {
        let head: String = text.chars().take(DEBUG_CHARACTERS_KEPT).collect();
        format!("{head}…（共 {total} 字符，截在 {DEBUG_CHARACTERS_KEPT}）")
    }
}

/// 记录值 `recorded` 的池上把盘 1 换成探测值 `probed` 的盘，会话发一次覆盖写：拒成点名盘 1、`probed` 与 `recorded` 的那一个成员，
/// 两块盘一个调用都没收到、逐字节不变，会话的现行那一版不动；同一个会话、同一份字节交回探测值都是 `recorded` 的两块盘，照常发成、两块盘都被写。
fn assert_a_session_publish_with_device_one_probing_another_physical_block_size_is_refused_before_any_read_or_write(
    recorded: u32,
    probed: u32,
) {
    let swapped_device = DeviceIdentity(1);
    let (pool, mut session) = pool_recording(recorded);
    let image = pool.image();
    let write_time_seconds = pool.write_time_seconds + 1;
    let log_of_the_swapped_devices: CallLog = Rc::new(RefCell::new(Vec::new()));
    let mut swapped_devices =
        counting_devices_on(&image, [recorded, probed], &log_of_the_swapped_devices);
    let images_before: Vec<SparseDevice> = swapped_devices
        .iter()
        .map(|(_, device)| device.inner.image.clone())
        .collect();
    let content = content_of(OVERWRITE_BYTES, write_time_seconds);
    let current_version_before = (
        session.current.root().instance,
        session.current.root().checkpoint_txg,
    );
    let refused = session.publish_user_change(
        &mut swapped_devices,
        UserChange::Overwrite(FirstFile {
            content: &content,
            write_time_seconds,
        }),
    );
    let disagreements = match &refused {
        Err(UserChangeRefused::PhysicalBlockSizesProbedOnTheDevicesDifferFromTheSystemConfiguration {
            disagreements,
        }) => disagreements.clone(),
        other => panic!(
            "记录 {recorded}、盘 1 探测 {probed}：会话发布要在进循环之前拒成探测值那一判，实际 {}",
            outcome_line(other)
        ),
    };
    assert_eq!(
        disagreements,
        vec![
            PhysicalBlockSizeProbedOnADeviceDifferingFromTheSystemConfiguration {
                identity_handed_in: swapped_device,
                probed_physical_block_size: PhysicalBlockSizeInBytes(probed),
                physical_block_size_in_the_system_configuration: PhysicalBlockSizeInBytes(recorded),
            }
        ],
        "只点名换上去的盘 1：探测到 {probed}、池记着 {recorded}"
    );
    let calls_of_the_refused_publish = log_of_the_swapped_devices.borrow().clone();
    assert_eq!(
        calls_of_the_refused_publish
            .iter()
            .filter(|call| call.kind.changes_the_medium())
            .count(),
        0,
        "拒在任何写之前：一个写、写零、discard 都没交到盘上，实际 {calls_of_the_refused_publish:?}"
    );
    assert_eq!(
        calls_of_the_refused_publish,
        Vec::new(),
        "只调探测：一个读、屏障也没交到盘上"
    );
    for ((identity, device), before) in swapped_devices.iter().zip(&images_before) {
        assert!(
            device.inner.image == *before,
            "{identity:?} 逐字节不变：拒在任何写之前"
        );
    }
    assert_eq!(
        (
            session.current.root().instance,
            session.current.root().checkpoint_txg
        ),
        current_version_before,
        "会话的现行那一版不动（实例代号与 txg）"
    );
    let log_of_the_devices_probing_the_recorded_value: CallLog = Rc::new(RefCell::new(Vec::new()));
    let mut devices_probing_the_recorded_value = counting_devices_on(
        &image,
        [recorded, recorded],
        &log_of_the_devices_probing_the_recorded_value,
    );
    let published = session.publish_user_change(
        &mut devices_probing_the_recorded_value,
        UserChange::Overwrite(FirstFile {
            content: &content,
            write_time_seconds,
        }),
    );
    assert!(
        published.is_ok(),
        "对照：同一个会话、同一份字节、两块盘都探测 {recorded}，同一个覆盖写照常发成，实际 {}",
        outcome_line(&published)
    );
    for (identity, device) in &devices_probing_the_recorded_value {
        assert!(
            log_of_the_devices_probing_the_recorded_value
                .borrow()
                .iter()
                .any(|call| call.device == *identity && call.kind.changes_the_medium()),
            "对照：{identity:?} 收到了这次发布的写"
        );
        let before = image.devices.get(identity).expect("镜像里有这块盘");
        assert!(
            device.inner.image != *before,
            "对照：{identity:?} 的字节被这次发布改了"
        );
    }
}

/// 记录 4096 的池，盘 1 换成探测 512 的盘（探测值往小改一档）。
#[test]
fn a_session_publish_with_device_one_probing_512_on_a_pool_recording_4096_is_refused_before_any_read_or_write(
) {
    assert_a_session_publish_with_device_one_probing_another_physical_block_size_is_refused_before_any_read_or_write(
        4096, 512,
    );
}

/// 记录 512 的池，盘 1 换成探测 4096 的盘（探测值往大改一档）。
#[test]
fn a_session_publish_with_device_one_probing_4096_on_a_pool_recording_512_is_refused_before_any_read_or_write(
) {
    assert_a_session_publish_with_device_one_probing_another_physical_block_size_is_refused_before_any_read_or_write(
        512, 4096,
    );
}
