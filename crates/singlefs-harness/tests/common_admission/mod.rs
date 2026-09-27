//! 实五那几条用例共用的搭建（`second_transaction_admission_raises_the_floor_before_refusing.rs` 与
//! `second_transaction_crash_inside_the_floor_raise_pushed_by_the_session.rs`）：两块内存稀疏盘上 mkfs、取号、暖机、第一个文件，
//! 之后经挂着的会话（`singlefs_core::mounted_session`）发布、崩了再挂，记下每条根带的文件内容（层 0 的 oracle 按它判读回什么）。
#![allow(dead_code, reason = "两个测试文件各自只用到其中一部分")]

use std::collections::BTreeMap;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes};
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::mount::{
    mount_writable_with_space_admission, MountError, MountOutput, MountSpaceAdmission,
    ParametersAndDeviceTableOfTheMount, ShadowLedger,
};
use singlefs_core::mounted_session::{
    MountedSession, UserChange, UserChangePublished, UserChangeRefused,
};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PoolVersion, PoolWriter,
    TransactionOutput,
};
use singlefs_harness::fault_injection::FaultInjectingBlockDevice;
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{
    MemoryPool, PublishedVersion, SparseBlockDevice, SparseDevice,
};
use singlefs_harness::RecordingBlockDevice;

/// 第一个文件与之后几次覆盖写的写入时刻起点（与 `tests/common` 的 `FIXED_WRITE_TIME_SECONDS` 同一个数）。
pub const FIRST_WRITE_TIME_SECONDS: u64 = 1_788_000_000;

/// 会话里每次覆盖写的内容长度：一个数据单元装得下（攻方那几段脚本的 2999 字节）。
pub const OVERWRITE_BYTES: usize = 2999;

/// 一块内存盘外面包的那一层（稀疏盘本身、注入故障的、录制写流的）：用例要从它取回稀疏盘上的镜像。
pub trait DeviceOverASparseImage: BlockDevice {
    fn sparse_image(&self) -> &SparseDevice;
}

impl DeviceOverASparseImage for SparseBlockDevice {
    fn sparse_image(&self) -> &SparseDevice {
        &self.image
    }
}

impl DeviceOverASparseImage for FaultInjectingBlockDevice<SparseBlockDevice> {
    fn sparse_image(&self) -> &SparseDevice {
        &self.wrapped_device().image
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
    /// 分配器装着 mkfs 刚写下的根环表）。`wrap` 给每块稀疏盘包上用例要的那一层。
    pub fn start_after_the_first_file(
        device_width: HistoryDeviceWidth,
        wrap: impl Fn(DeviceIdentity, SparseBlockDevice) -> Device,
    ) -> Self {
        let parameters = device_width.parameters();
        let device_bytes = device_width.device_bytes();
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
            Err(
                UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount { .. }
                | UserChangeRefused::NoFileVersionToChange
                | UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. },
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

pub fn start_plain(device_width: HistoryDeviceWidth) -> PoolUnderTest<SparseBlockDevice> {
    PoolUnderTest::start_after_the_first_file(device_width, plain_device)
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

/// 这一刻的盘面拷一份、只崩了再挂那一次可写挂载（空间准入判着）交回的空间准入结局。
pub fn space_admission_of_a_crash_remount_on_a_copy(
    pool: &PoolUnderTest<SparseBlockDevice>,
) -> Result<MountSpaceAdmission, MountError> {
    let mut devices = plain_devices_on(&pool.image());
    mount_writable_with_space_admission(
        &pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .map(|mounted| mounted.output.space_admission)
}
