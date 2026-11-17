//! 探针的池：两块内存盘上 mkfs、取号、暖机、第一个文件（与验证包准入用例同一条起法，自己写一份），之后经挂着的会话发布；
//! 拷一份池（盘上写过的扇区与会话的每个字段），发布边界上崩了再挂、正常卸载再挂。

use std::collections::BTreeMap;
use std::sync::Arc;

use singlefs_core::address::DeviceIdentity;
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::UnitAreaStart;
use singlefs_core::compaction_plan::CompactionOfThisMount;
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::mount::{
    mount_writable_with_space_admission, unmount, ParametersAndDeviceTableOfTheMount, ShadowLedger,
};
use singlefs_core::mounted_session::MountedSession;
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PoolVersion, PoolWriter,
    FIRST_INODE_NUMBER,
};
use singlefs_format::SLOT_BYTES;

use crate::memory_device::MemoryDevice;

/// journal 环 6 MiB：单元区起点落在 1408 = 22 × 64 槽的段边界上（验证包 384 槽小盘那一档同一个数）。
pub const JOURNAL_RING_BYTES: u64 = 6 << 20;
/// 单元区里为环窗口补足的槽数与之外那一段（验证包 384 槽小盘那一档：1007 + 384）。
pub const RING_WINDOW_ALLOWANCE_SLOTS: u64 = 1007;
pub const UNIT_AREA_SLOTS_BEYOND_THE_ALLOWANCE: u64 = 384;
pub const FIRST_WRITE_TIME_SECONDS: u64 = 1_788_000_000;
const FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x34, 0x32, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00,
];

pub fn parameters_of_the_probe() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(512),
            journal_ring_bytes: JOURNAL_RING_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

pub fn content_of(length: usize, seed: u64) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index as u64 * 131 + seed * 7 + 1) % 251).expect("小于 256"))
        .collect()
}

pub struct Pool {
    pub parameters: MakeFilesystemParameters,
    pub devices: Vec<(DeviceIdentity, MemoryDevice)>,
    pub session: Option<MountedSession>,
    /// 这一刻活着的文件：inode 号 → 字节数（按交回的结局记，拒绝不改；unlink 被拒而意图已发成的照删）。
    pub files: BTreeMap<u64, u64>,
    pub write_time_seconds: u64,
    pub snapshots_created: u64,
}

fn copy_of_the_session(session: &MountedSession) -> MountedSession {
    MountedSession {
        allocator: session.allocator.clone(),
        current: session.current.clone(),
        instance: session.instance,
        shadow_ledger: session.shadow_ledger,
        parameters_and_device_table: session.parameters_and_device_table.clone(),
        compaction: session.compaction.clone(),
    }
}

impl Pool {
    /// 两块盘各在 1007 + 384 槽之外再多 `extra_slots` 槽（两块可以不等大）。
    pub fn start(extra_slots: [u64; 2]) -> Result<Pool, String> {
        let parameters = parameters_of_the_probe();
        let unit_area_start_slot = UnitAreaStart::following_the_journal_ring(JOURNAL_RING_BYTES)
            .map_err(|error| format!("unit-area-start {error:?}"))?
            .slot()
            .0;
        let mut devices: Vec<(DeviceIdentity, MemoryDevice)> = [0_u32, 1]
            .into_iter()
            .map(|index| {
                let slots = unit_area_start_slot
                    + RING_WINDOW_ALLOWANCE_SLOTS
                    + UNIT_AREA_SLOTS_BEYOND_THE_ALLOWANCE
                    + extra_slots[index as usize];
                (DeviceIdentity(index), MemoryDevice::new(slots * SLOT_BYTES))
            })
            .collect();
        let genesis = make_filesystem(&parameters, &mut devices)
            .map_err(|error| format!("mkfs {error:?}"))?;
        let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
        let first_content = content_of(3000, 0);
        let (instance, first_version) = {
            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
            let instance =
                acquire_instance(&mut writer).map_err(|error| format!("acquire {error:?}"))?;
            let warmed = warm_up(&mut writer, &genesis.root, instance)
                .map_err(|error| format!("warm-up {error:?}"))?;
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
            .map_err(|error| format!("first-file {error:?}"))?;
            (instance, first_version)
        };
        let parameters_and_device_table =
            ParametersAndDeviceTableOfTheMount::of_a_pool_on_disk(&devices)
                .map_err(|error| format!("device-table {error:?}"))?;
        let mut files = BTreeMap::new();
        files.insert(FIRST_INODE_NUMBER, 3000);
        Ok(Pool {
            parameters,
            devices,
            session: Some(MountedSession {
                allocator,
                current: PoolVersion::WithFile(Arc::new(first_version)),
                instance,
                shadow_ledger: ShadowLedger::On,
                parameters_and_device_table,
                compaction: CompactionOfThisMount::of_a_new_mount(),
            }),
            files,
            write_time_seconds: FIRST_WRITE_TIME_SECONDS,
            snapshots_created: 0,
        })
    }

    /// 一份拷贝：盘上写过的扇区、会话的每个字段（分配器整份拷，现行那一版共享，推的时候 `Arc::make_mut` 各拷各的）。
    pub fn copy(&self) -> Pool {
        Pool {
            parameters: self.parameters.clone(),
            devices: self.devices.clone(),
            session: self.session.as_ref().map(copy_of_the_session),
            files: self.files.clone(),
            write_time_seconds: self.write_time_seconds,
            snapshots_created: self.snapshots_created,
        }
    }

    pub fn session(&mut self) -> &mut MountedSession {
        self.session.as_mut().expect("这一刻有可写会话")
    }

    /// 现行那一版的 inode 号水位（建 inode 那次从它起编号）；现行那一版不带文件时 `None`。
    pub fn inode_number_watermark(&self) -> Option<u64> {
        self.session
            .as_ref()
            .and_then(|session| session.current.file_version())
            .map(|version| version.inode_number_watermark())
    }

    /// 进程退出（会话丢掉，盘停在最后一次落盘之后），再可写挂载一次（空间准入判着）。挂不上交回错误的头一段。
    pub fn crash_then_mount(&mut self) -> Result<(), String> {
        self.session = None;
        let mounted = mount_writable_with_space_admission(
            &self.parameters,
            &mut self.devices,
            SpaceAdmission::JudgedByTheFormula,
        )
        .map_err(|error| leading_text(&format!("{error:?}")))?;
        let (_output, session) = MountedSession::of_the_mount(mounted);
        self.session = Some(session);
        Ok(())
    }

    /// 正常卸载（抬 F 到现行那一版），再可写挂载一次。卸载报错照样去挂（与崩了再挂同一处置），错误的头一段带回。
    pub fn unmount_then_mount(&mut self) -> Result<Option<String>, String> {
        let mut session = self.session.take().expect("这一刻有可写会话");
        let unmounted = unmount(
            &self.parameters,
            &mut self.devices,
            &mut session.allocator,
            &mut session.current,
            session.shadow_ledger,
        );
        let unmount_refusal = unmounted
            .err()
            .map(|error| leading_text(&format!("{error:?}")));
        self.crash_then_mount()?;
        Ok(unmount_refusal)
    }
}

/// 错误文字的头一段（到第一个括号或空白为止）：认成员，不把整份结构印出来。
pub fn leading_text(text: &str) -> String {
    text.split(|character: char| character == '(' || character == '{' || character.is_whitespace())
        .next()
        .unwrap_or("")
        .to_string()
}
