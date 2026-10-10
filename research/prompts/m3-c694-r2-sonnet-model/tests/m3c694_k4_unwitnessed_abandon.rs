//! m3-c694-r2 辩方腿的探针之三（harness 档）：C694 那一形在「树表 0 条的一版被抛弃」上的结局。
//! 造法（与 `tests/common/mod.rs` 的 `abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before` 同一个机理，
//! 但藏的是最新那条根的根槽与它那次发布写的全部单元，不只数据单元；两种起点：
//!   no_file  ：mkfs → 取号 → 暖机（两条根），被抛弃的是暖机的第二条根（树表 0 条、中央映射根是空指针）
//!   with_file：mkfs → 取号 → 暖机 → 第一个文件，被抛弃的是第一个文件那条根）
//! 1. 崩溃镜像 X：那条根（R）持久、它之后轮换的系统配置槽没落（系统配置没见证到 R）；
//! 2. 第二次挂载：R 的根槽与它那次发布的每个单元整次挂载读回全 0（写到哪一段上那一段不再藏）；
//! 3. 第三次挂载：什么都不藏。R 读得出、按实例表被抛弃；影子账读它的账。
//! 四臂（今天 / C / W1 / W2）各自从头造一遍：臂开关在第二、三次挂载里都开着。
//! 环境变量：K4_ARMS（缺省 none,c,w1,w2）。

#[allow(dead_code)]
mod common_admission_pool;

use std::sync::atomic::Ordering;

use common_admission_pool::{content_of, device_bytes_of_the_384_slot_small_devices,
    fixed_geometry_of_the_384_slot_small_devices, parameters_of_the_384_slot_small_devices,
    plain_devices_on, FIRST_WRITE_TIME_SECONDS};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::mount::{
    candidate_switches, mount_writable_with_space_admission, probe_counters,
};
use singlefs_core::recovery::tree_table_entry_count;
use singlefs_core::root_record::RootRecord;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter,
};
use singlefs_harness::memory_pool::{
    writes_and_segments, MemoryPool, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

struct HidingDevice {
    inner: SparseBlockDevice,
    hidden: Vec<(u64, u64)>,
}

impl BlockDevice for HidingDevice {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)?;
        let length = buffer.len() as u64;
        for (start, hidden_length) in &self.hidden {
            if *start < offset.0 + length && offset.0 < *start + *hidden_length {
                let from = start.saturating_sub(offset.0) as usize;
                let to = ((*start + *hidden_length).min(offset.0 + length) - offset.0) as usize;
                buffer[from..to].fill(0);
            }
        }
        Ok(())
    }
    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        let length = bytes.len() as u64;
        self.hidden
            .retain(|(start, hidden_length)| !(*start < offset.0 + length && offset.0 < *start + *hidden_length));
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
        self.hidden
            .retain(|(start, hidden_length)| !(*start < offset.0 + length && offset.0 < *start + *hidden_length));
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

fn set_arm(arm: &str) {
    candidate_switches::W1.store(arm == "w1", Ordering::SeqCst);
    candidate_switches::W2.store(arm == "w2", Ordering::SeqCst);
    candidate_switches::W4.store(arm == "c", Ordering::SeqCst);
}

fn error_kind<E: std::fmt::Debug>(error: &E) -> String {
    let text = format!("{error:?}");
    let head: String = text.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
    // NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot { .. }) 这一类带内层名字的，把内层名字带上
    let inner: String = text
        .split_once('(')
        .map(|(_, rest)| rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect())
        .unwrap_or_default();
    format!("{head}({inner})")
}

/// 录两块盘上 mkfs 之后到「要抛弃的那条根」为止的写；with_file 为真时那条根是第一个文件的根，否则是暖机的第二条根。
fn record_until_the_root_to_abandon(with_file: bool) -> (MemoryPool, Vec<RetainedWrite>) {
    let parameters = parameters_of_the_384_slot_small_devices();
    let device_bytes = device_bytes_of_the_384_slot_small_devices();
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    RecordingBlockDevice::with_shared_stream(
                        identity,
                        SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512)),
                        stream.clone(),
                    ),
                )
            })
            .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let base = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: device_bytes,
    };
    let operations_after_mkfs = stream.operation_count();
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        if with_file {
            let content = content_of(3000, 0);
            publish_first_file(
                &mut writer,
                &mut allocator,
                warmed.roots.last().expect("暖机两条根"),
                FirstFile { content: &content, write_time_seconds: FIRST_WRITE_TIME_SECONDS },
                instance,
                &warmed.last_record_bytes,
            )
            .expect("第一个文件");
        }
    }
    let recorded = stream.retained_operations()[operations_after_mkfs..].to_vec();
    let (writes, _segments) = writes_and_segments(&recorded, &fixed_geometry_of_the_384_slot_small_devices());
    (base, writes)
}

/// 树表 0 条的池上第二次挂载（实例 2）的写行那次发布的根：mkfs → 第一次可写挂载（实例 1，取号、写行、暖机）整次做完 → 第二次可写挂载只到它的第一条根（写行那次发布）。
fn record_the_row_publish_of_the_second_mount_of_a_pool_without_a_file() -> (MemoryPool, Vec<RetainedWrite>) {
    let parameters = parameters_of_the_384_slot_small_devices();
    let device_bytes = device_bytes_of_the_384_slot_small_devices();
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    RecordingBlockDevice::with_shared_stream(
                        identity,
                        SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512)),
                        stream.clone(),
                    ),
                )
            })
            .collect();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    mount_writable_with_space_admission(&parameters, &mut devices, SpaceAdmission::JudgedByTheFormula)
        .expect("第一次可写挂载");
    let base = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: device_bytes,
    };
    let operations_before = stream.operation_count();
    mount_writable_with_space_admission(&parameters, &mut devices, SpaceAdmission::JudgedByTheFormula)
        .expect("第二次可写挂载");
    let recorded = stream.retained_operations()[operations_before..].to_vec();
    let (writes, _segments) = writes_and_segments(&recorded, &fixed_geometry_of_the_384_slot_small_devices());
    let first_root = writes
        .iter()
        .position(|write| write.kind == StepKind::RootRecordFua)
        .expect("第二次挂载至少一条根");
    let truncated: Vec<RetainedWrite> = writes[..=first_root].to_vec();
    (base, truncated)
}

fn scenario(label: &str, base: &MemoryPool, writes: &[RetainedWrite], arm: &str) {
    set_arm(arm);
    let parameters: MakeFilesystemParameters = parameters_of_the_384_slot_small_devices();
    let root_indices: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    let last_root = *root_indices.last().expect("至少一条根");
    let previous_root = root_indices.len().checked_sub(2).map(|index| root_indices[index]);
    let unit_writes: Vec<usize> = (previous_root.map_or(0, |index| index + 1)..last_root)
        .filter(|index| writes[*index].kind == StepKind::UnitWrite)
        .collect();
    // 崩溃镜像 X：除了最后那条根之后的系统配置槽轮换，别的写都落了。
    let mut image = base.clone();
    for (index, write) in writes.iter().enumerate() {
        if write.kind == StepKind::SystemConfigurationSlot && index > last_root {
            continue;
        }
        image.apply_writes(std::slice::from_ref(write));
    }
    // 第二次挂载：R 的根槽与它那次发布的每个单元读回全 0。
    let hidden_of = |identity: DeviceIdentity| -> Vec<(u64, u64)> {
        std::iter::once(last_root)
            .chain(unit_writes.iter().copied())
            .filter(|index| writes[*index].device == identity)
            .map(|index| (writes[index].offset.0, writes[index].length_in_bytes()))
            .collect()
    };
    let mut hiding: Vec<(DeviceIdentity, HidingDevice)> = image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut inner = SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
            inner.image = sparse.clone();
            (*identity, HidingDevice { inner, hidden: hidden_of(*identity) })
        })
        .collect();
    let before2 = probe_counters::snapshot();
    let second = mount_writable_with_space_admission(&parameters, &mut hiding, SpaceAdmission::JudgedByTheFormula);
    let after2 = probe_counters::snapshot();
    let (second_text, after_second) = match second {
        Ok(mounted) => {
            let isolated: u64 = mounted.output.isolated_slots_per_device.iter().map(|(_, slots)| *slots).sum();
            let image2 = MemoryPool {
                devices: hiding.iter().map(|(identity, device)| (*identity, device.inner.image.clone())).collect(),
                device_size_in_bytes: image.device_size_in_bytes,
            };
            (
                format!(
                    "Ok chose=({},{}) eff=({},{}) isolated={isolated} c_hook_placements={}",
                    mounted.output.chosen_root.instance.0,
                    mounted.output.chosen_root.checkpoint_txg.0,
                    mounted.output.effective_root.instance.0,
                    mounted.output.effective_root.checkpoint_txg.0,
                    after2[8] - before2[8]
                ),
                Some(image2),
            )
        }
        Err(error) => (format!("Err {}", error_kind(&error)), None),
    };
    let Some(image2) = after_second else {
        println!("K4E3 label={label} arm={arm} unit_writes={} second={second_text}", unit_writes.len());
        set_arm("none");
        return;
    };
    let clobbered = unit_writes
        .iter()
        .filter(|index| {
            let write = &writes[**index];
            let length = usize::try_from(write.length_in_bytes()).expect("单元写长");
            let read_back = image2.devices[&write.device].read(write.offset, length);
            !write.contents.still_on_disk(&read_back)
        })
        .count();
    // R 的树表单元这一刻读不读得出（树表 0 条的根：账就是它自己与根记录里的几条指针）。
    let root_record = RootRecord::parse_slot(
        writes[last_root].bytes().expect("根槽写带字节"),
        &parameters.filesystem_identifier,
    )
    .expect("被抛弃的那条根自证得过");
    let tree_table_text = |image: &MemoryPool| match tree_table_entry_count(image, &root_record) {
        Ok(entries) => format!("Ok({entries})"),
        Err(error) => format!("Err({})", error_kind(&error)),
    };
    let tree_table_after_second = tree_table_text(&image2);
    // 第三次挂载：什么都不藏。
    let mut plain = plain_devices_on(&image2);
    let before3 = probe_counters::snapshot();
    let third = mount_writable_with_space_admission(&parameters, &mut plain, SpaceAdmission::JudgedByTheFormula);
    let after3 = probe_counters::snapshot();
    let third_text = match third {
        Ok(mounted) => format!(
            "Ok eff=({},{}) abandoned_roots_unreadable={}",
            mounted.output.effective_root.instance.0,
            mounted.output.effective_root.checkpoint_txg.0,
            mounted.output.abandoned_roots_unreadable
        ),
        Err(error) => format!("Err {}", error_kind(&error)),
    };
    println!(
        "K4E3 label={label} arm={arm} unit_writes={} clobbered_by_the_second_mount={clobbered} r_tree_table_entries_after_the_second_mount={tree_table_after_second} second={second_text} third={third_text} third_abandoned_roots_seen={} third_account_unreadable={} third_mapping_recompute_failed={} third_refused={}",
        unit_writes.len(),
        after3[1] - before3[1],
        after3[2] - before3[2],
        after3[3] - before3[3],
        after3[4] - before3[4]
    );
    set_arm("none");
}

#[test]
#[ignore = "辩方腿的探针：按 rerun.sh 起"]
fn k4_an_unwitnessed_newest_root_abandoned_under_each_arm() {
    let arms_text = std::env::var("K4_ARMS").unwrap_or_else(|_| "none,c,w1,w2".to_string());
    for (label, with_file) in [
        ("no_file_warm_up_root", false),
        ("with_file_first_file_root", true),
        ("no_file_second_mount_row_publish_root", false),
    ] {
        let (base, writes) = if label.starts_with("no_file_second") {
            record_the_row_publish_of_the_second_mount_of_a_pool_without_a_file()
        } else {
            record_until_the_root_to_abandon(with_file)
        };
        let roots = writes.iter().filter(|write| write.kind == StepKind::RootRecordFua).count();
        println!("K4E3SETUP label={label} writes={} roots={roots}", writes.len());
        for arm in arms_text.split(',') {
            scenario(label, &base, &writes, arm);
        }
    }
}
