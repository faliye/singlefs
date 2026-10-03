//! 准入那几条用例的池：两块内存稀疏盘上 mkfs、取号、暖机、第一个文件，之后经挂着的会话（`singlefs_core::mounted_session`）发布、
//! 崩了再挂，记下每条根带的文件内容（层 0 的 oracle 按它判读回什么）。
//! 这一份不引随机历史脚手架（`singlefs_harness::history`）与它带的故障注入、对拍：登记的崩溃枚举用例
//! `crash_enumeration_floor_raise_pushed_by_admission.rs` 直接经它搭池，引了那几份，改随机历史就让那一格整格重判
//! （`crates/singlefs-checker-tier/src/crash_identity.rs` 的 `RANDOM_HISTORY_SCAFFOLDING_FILE`）。
//! harness 档的准入用例经 `tests/common_admission/mod.rs` 用同一份（它把这里全部转出去，再补上按 `HistoryDeviceWidth` 起池与故障注入那一层）。
#![allow(dead_code, reason = "几个测试文件各自 mod 进来，只用到其中一部分")]

use std::collections::BTreeMap;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::UnitAreaStart;
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes};
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::mount::{
    mount_writable_with_space_admission, MountError, MountOutput,
    ParametersAndDeviceTableOfTheMount, ShadowLedger,
};
use singlefs_core::mounted_session::{
    MountedSession, UserChange, UserChangePublished, UserChangeRefused,
};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PoolVersion, PoolWriter,
    TransactionOutput,
};
use singlefs_format::SLOT_BYTES;
use singlefs_harness::memory_pool::{
    MemoryPool, PublishedVersion, SparseBlockDevice, SparseDevice,
};
use singlefs_harness::scenario::e142_parameters;
use singlefs_harness::segments::FixedGeometry;
use singlefs_harness::RecordingBlockDevice;

/// 第一个文件与之后几次覆盖写的写入时刻起点（与 `tests/common` 的 `FIXED_WRITE_TIME_SECONDS` 同一个数）。
pub const FIRST_WRITE_TIME_SECONDS: u64 = 1_788_000_000;

/// 会话里每次覆盖写的内容长度：一个数据单元装得下（攻方那几段脚本的 2999 字节）。
pub const OVERWRITE_BYTES: usize = 2999;

/// 单元区 384 槽那一档小盘的单元区槽数：六个 64 槽的聚簇段。
/// 抄自 `crates/singlefs-harness/src/history.rs` 的 `SMALL_DEVICE_UNIT_AREA_SLOTS`（`HistoryDeviceWidth::UnitAreaOf384Slots`）：
/// 那边是私有常量，引 `HistoryDeviceWidth` 就把随机历史脚手架带进上面那条登记用例的判法闭包，history.rs 又不在这一件能改的范围里，
/// 所以各留一份。`tests/common_admission/mod.rs` 的 `PoolUnderTest::start_after_the_first_file` 按 384 槽那一档起池时
/// 核这里三样（参数、盘宽、几何）与 history.rs 那一档逐项相同，两边改了一边就红。
const UNIT_AREA_SLOTS_OF_THE_384_SLOT_SMALL_DEVICES: u64 = 384;

/// 单元区 384 槽那一档小盘为环窗口补足的槽数：815（D28（挂载期承诺量） 已定项 1 末项环窗口每块盘常驻扣，用户 2026-10-02 定「甲」：小盘放大到扣得起）。
/// 抄自 `crates/singlefs-harness/src/history.rs` 的 `SMALL_DEVICE_RING_WINDOW_ALLOWANCE_SLOTS`（式子 750 + 5 + 60 的来历写在那边），为什么各留一份见上一条；
/// 名字里的 384 槽是补足之外的那一段。
const RING_WINDOW_ALLOWANCE_SLOTS_OF_THE_384_SLOT_SMALL_DEVICES: u64 = 815;

/// 单元区 384 槽那一档小盘的 journal 环字节数：6 MiB。
/// 抄自 `crates/singlefs-harness/src/history.rs` 的 `SMALL_DEVICE_JOURNAL_RING_BYTES`，为什么各留一份见上一条。
const JOURNAL_RING_BYTES_OF_THE_384_SLOT_SMALL_DEVICES: u64 = 6 << 20;

/// 单元区 384 槽那一档小盘的 mkfs 与发布参数：E142 装置那一份（物理块 512、io_min 512），只把 journal 环缩到 6 MiB。
#[must_use]
pub fn parameters_of_the_384_slot_small_devices() -> MakeFilesystemParameters {
    let mut parameters = e142_parameters(512, 512);
    parameters.geometry.journal_ring_bytes = JOURNAL_RING_BYTES_OF_THE_384_SLOT_SMALL_DEVICES;
    parameters
}

/// 单元区 384 槽那一档小盘每块盘的字节数：单元区起点（6 MiB 的环末尾的下一个槽）加环窗口补足的 815 槽、再加 384 槽。
#[must_use]
pub fn device_bytes_of_the_384_slot_small_devices() -> u64 {
    let unit_area_start_slot =
        UnitAreaStart::following_the_journal_ring(JOURNAL_RING_BYTES_OF_THE_384_SLOT_SMALL_DEVICES)
            .expect("6 MiB 的环末尾的下一个槽 1408 = 22 × 64，落在聚簇段边界上")
            .slot()
            .0;
    (unit_area_start_slot
        + RING_WINDOW_ALLOWANCE_SLOTS_OF_THE_384_SLOT_SMALL_DEVICES
        + UNIT_AREA_SLOTS_OF_THE_384_SLOT_SMALL_DEVICES)
        * SLOT_BYTES
}

/// 单元区 384 槽那一档小盘按落点分写的种类要的那份几何（`singlefs_harness::segments::FixedGeometry`）。
#[must_use]
pub fn fixed_geometry_of_the_384_slot_small_devices() -> FixedGeometry {
    let parameters = parameters_of_the_384_slot_small_devices();
    FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    }
}

/// 一块内存盘外面包的那一层（稀疏盘本身、录制写流的；注入故障的那一层在 `tests/common_admission/mod.rs`）：用例要从它取回稀疏盘上的镜像。
pub trait DeviceOverASparseImage: BlockDevice {
    fn sparse_image(&self) -> &SparseDevice;
}

impl DeviceOverASparseImage for SparseBlockDevice {
    fn sparse_image(&self) -> &SparseDevice {
        &self.image
    }
}

impl DeviceOverASparseImage for RecordingBlockDevice<SparseBlockDevice> {
    fn sparse_image(&self) -> &SparseDevice {
        &self.wrapped_device().image
    }
}

/// 用例里的一个池：两块盘、这一刻的会话（没有会话 = 进程退出之后还没挂），与每条写过的根带的文件内容（层 0 的 oracle 按它判读回什么）。
pub struct PoolUnderTest<Device: DeviceOverASparseImage> {
    pub parameters: MakeFilesystemParameters,
    pub device_bytes: u64,
    pub devices: Vec<(DeviceIdentity, Device)>,
    pub session: Option<MountedSession>,
    pub write_time_seconds: u64,
    pub content_of_the_current_version: Vec<u8>,
    pub content_of_each_root: BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<u8>>,
}

pub fn content_of(length: usize, seed: u64) -> Vec<u8> {
    (0..length)
        .map(|index| {
            let index = u64::try_from(index).expect("长度装得进 u64");
            u8::try_from((index * 131 + seed * 7 + 1) % 251).expect("小于 256")
        })
        .collect()
}

pub fn root_key(
    root: &singlefs_core::root_record::RootRecord,
) -> (InstanceGeneration, CheckpointTxg) {
    (root.instance, root.checkpoint_txg)
}

impl<Device: DeviceOverASparseImage> PoolUnderTest<Device> {
    /// mkfs、取号 1、暖机、第一个文件（3000 字节），都在同一个进程里；会话接着第一个文件那一版（mkfs 同一个进程那条会话，
    /// 分配器装着 mkfs 刚写下的根环表）。两块盘各 `device_bytes` 字节，`wrap` 给每块稀疏盘包上用例要的那一层。
    pub fn start_after_the_first_file_on_devices_of(
        parameters: MakeFilesystemParameters,
        device_bytes: u64,
        wrap: impl Fn(DeviceIdentity, SparseBlockDevice) -> Device,
    ) -> Self {
        let mut devices: Vec<(DeviceIdentity, Device)> = [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    wrap(
                        identity,
                        SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512)),
                    ),
                )
            })
            .collect();
        let genesis = make_filesystem(&parameters, &mut devices).expect("小盘上 mkfs 做得成");
        let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
        let first_content = content_of(3000, 0);
        let (instance, first_version) = {
            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
            let instance = acquire_instance(&mut writer).expect("取号 1");
            let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
            let first_version = publish_first_file(
                &mut writer,
                &mut allocator,
                warmed.roots.last().expect("暖机写了两条根"),
                FirstFile {
                    content: &first_content,
                    write_time_seconds: FIRST_WRITE_TIME_SECONDS,
                },
                instance,
                &warmed.last_record_bytes,
            )
            .expect("第一个文件");
            (instance, first_version)
        };
        let mut content_of_each_root = BTreeMap::new();
        content_of_each_root.insert(root_key(&first_version.root), first_content.clone());
        // mkfs 同一个进程这条会话没经过可写挂载：它认下的池从盘上读（与可写挂载同一套核），不拿用例手里的参数。
        let parameters_and_device_table = ParametersAndDeviceTableOfTheMount::of_a_pool_on_disk(
            &devices,
        )
        .expect(
            "mkfs 同一个进程刚写成的两块盘：盘表就是 mkfs 用的那一份，系统配置读得出、与盘表相符",
        );
        PoolUnderTest {
            parameters,
            device_bytes,
            devices,
            session: Some(MountedSession {
                allocator,
                current: PoolVersion::WithFile(first_version),
                instance,
                shadow_ledger: ShadowLedger::On,
                parameters_and_device_table,
            }),
            write_time_seconds: FIRST_WRITE_TIME_SECONDS,
            content_of_the_current_version: first_content,
            content_of_each_root,
        }
    }

    pub fn session(&self) -> &MountedSession {
        self.session.as_ref().expect("这一刻有可写会话")
    }

    pub fn current_file_version(&self) -> &TransactionOutput {
        self.session()
            .current
            .file_version()
            .expect("第一个文件之后现行那一版都带文件")
    }

    pub fn rollback_floor(&self) -> CheckpointTxg {
        self.session().current.root().rollback_floor
    }

    pub fn image(&self) -> MemoryPool {
        MemoryPool {
            devices: self
                .devices
                .iter()
                .map(|(identity, device)| (*identity, device.sparse_image().clone()))
                .collect(),
            device_size_in_bytes: self.device_bytes,
        }
    }

    pub fn note_root_of_the_current_content(
        &mut self,
        root: &singlefs_core::root_record::RootRecord,
    ) {
        self.content_of_each_root
            .insert(root_key(root), self.content_of_the_current_version.clone());
    }

    pub fn note_floor_raises(&mut self, floor_raises: &[singlefs_core::mount::RaisedFloor]) {
        let roots: Vec<singlefs_core::root_record::RootRecord> = floor_raises
            .iter()
            .flat_map(|raised| raised.publishes.iter().map(|publish| publish.root))
            .collect();
        for root in &roots {
            self.note_root_of_the_current_content(root);
        }
    }

    /// 进程退出（会话丢掉，盘停在最后一次落盘之后），再可写挂载一次（空间准入判着）。做成就换上新会话，并记下这次挂载写的每条根。
    pub fn crash_and_mount_writable(&mut self) -> Result<MountOutput, MountError> {
        self.session = None;
        let mounted = mount_writable_with_space_admission(
            &self.parameters,
            &mut self.devices,
            SpaceAdmission::JudgedByTheFormula,
        )?;
        let (output, session) = MountedSession::of_the_mount(mounted);
        if let Some(content) = self
            .content_of_each_root
            .get(&root_key(&output.effective_root))
        {
            self.content_of_the_current_version = content.clone();
        }
        let roots: Vec<singlefs_core::root_record::RootRecord> =
            std::iter::once(&output.row_publish)
                .chain(&output.warm_up_publishes)
                .map(|version| *version.root())
                .collect();
        for root in &roots {
            self.note_root_of_the_current_content(root);
        }
        self.note_floor_raises(output.space_admission.floor_raises());
        self.session = Some(session);
        Ok(output)
    }

    /// 经会话发布一次用户改动（准入不够、或落点取不到时先推抬 F 再判），记下推的根与新的一版的根。
    pub fn publish(
        &mut self,
        change: UserChange<'_>,
        content_after: Option<&[u8]>,
    ) -> Result<UserChangePublished, UserChangeRefused> {
        let mut session = self.session.take().expect("这一刻有可写会话");
        let outcome = session.publish_user_change(&mut self.devices, change);
        self.session = Some(session);
        match &outcome {
            Ok(published) => {
                self.note_floor_raises(&published.floor_raises);
                if let Some(content) = content_after {
                    self.content_of_the_current_version = content.to_vec();
                }
                let root = *self.session().current.root();
                self.note_root_of_the_current_content(&root);
            }
            Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space)) => {
                self.note_floor_raises(&no_space.floor_raises);
            }
            Err(UserChangeRefused::Publish { floor_raises, .. }) => {
                self.note_floor_raises(floor_raises);
            }
            Err(UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(failed)) => {
                self.note_floor_raises(&failed.floor_raises);
            }
            Err(UserChangeRefused::NoFileVersionToChange { floor_raises, .. }) => {
                self.note_floor_raises(floor_raises);
            }
            Err(
                UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount { .. }
                | UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. }
                | UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable { .. }
                | UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
                    ..
                },
            ) => {}
        }
        outcome
    }

    pub fn overwrite(&mut self, length: usize) -> Result<UserChangePublished, UserChangeRefused> {
        self.write_time_seconds += 1;
        let content = content_of(length, self.write_time_seconds);
        let write_time_seconds = self.write_time_seconds;
        self.publish(
            UserChange::Overwrite(FirstFile {
                content: &content,
                write_time_seconds,
            }),
            Some(&content),
        )
    }

    pub fn sequential_write(
        &mut self,
        length: usize,
    ) -> Result<UserChangePublished, UserChangeRefused> {
        self.write_time_seconds += 1;
        let content = content_of(length, self.write_time_seconds);
        let write_time_seconds = self.write_time_seconds;
        self.publish(
            UserChange::SequentialWrite(FirstFile {
                content: &content,
                write_time_seconds,
            }),
            Some(&content),
        )
    }

    pub fn create_one_inode(&mut self) -> Result<UserChangePublished, UserChangeRefused> {
        self.write_time_seconds += 1;
        let write_time_seconds = self.write_time_seconds;
        self.publish(
            UserChange::NewInodes {
                count: 1,
                write_time_seconds,
            },
            None,
        )
    }

    /// 已写过的每一版（(实例, txg) → 文件内容）：层 0 的 oracle 按实际走的根判读回的是哪一版。
    pub fn published_versions(&self) -> Vec<PublishedVersion> {
        self.content_of_each_root
            .iter()
            .map(|((instance, checkpoint_txg), content)| PublishedVersion {
                instance: *instance,
                checkpoint_txg: *checkpoint_txg,
                content: content.clone(),
            })
            .collect()
    }
}

/// 一份镜像上池级 checker 判违例的那几条（名字加第一处）。
pub fn checker_violations_on(image: &MemoryPool) -> Vec<String> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some(format!("{invariant}: {detail}")),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
        })
        .collect()
}

pub fn plain_device(_identity: DeviceIdentity, device: SparseBlockDevice) -> SparseBlockDevice {
    device
}

/// 一份镜像交给两块新的稀疏盘（拷贝，改它不动原来那份）。
pub fn plain_devices_on(image: &MemoryPool) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device =
                SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            (*identity, device)
        })
        .collect()
}
