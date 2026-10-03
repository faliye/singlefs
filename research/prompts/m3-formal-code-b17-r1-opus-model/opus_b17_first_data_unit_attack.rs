//! 攻方原型（三方轮 m3-formal-code-b17-r1，云端攻方）：I-5.5 在合法历史上会不会认出那一版、而 t1 ≠ U + 4。
//! 只在攻方自己的仓副本里跑（harness 档，内存稀疏盘 4 GiB × 2），不进库。每格打一行 `name=` 开头的结果。

mod common;

use common::{
    crash_state_devices, file_content, memory_pool_of_sparse_devices, parameters,
    FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
};
use singlefs_checker::image::{
    InvariantVerdict, FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK,
};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::journal::back_chain_of;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{mount_writable, Mounted};
use singlefs_core::transaction::{
    publish_first_file, publish_without_units, FirstFile, PoolVersion, PoolWriter,
    TransactionOutput, VersionWithoutFilePublishOutput, ZeroUnitPublishPlan,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

type MemoryDevice = RecordingBlockDevice<SparseBlockDevice>;

fn memory_devices(stream: &SharedStream) -> Vec<(DeviceIdentity, MemoryDevice)> {
    (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                RecordingBlockDevice::with_shared_stream(
                    DeviceIdentity(number),
                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect()
}

fn devices_of(image: &MemoryPool) -> Vec<(DeviceIdentity, MemoryDevice)> {
    crash_state_devices(image, &[], &[], &SharedStream::new())
}

/// 池级 checker 对一张镜像的判定：I-5.5 的判定文本，与红了的全部不变量。
fn judge(image: &MemoryPool) -> (String, Vec<String>) {
    let report = check_pool_image(image);
    let i55 = report
        .iter()
        .find(|(invariant, _)| {
            *invariant == FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK
        })
        .map(|(_, verdict)| match verdict {
            InvariantVerdict::Holds => "holds".to_string(),
            InvariantVerdict::NotApplicable(reason) => format!("not_applicable({reason})"),
            InvariantVerdict::Violated(detail) => format!("VIOLATED({detail})"),
        })
        .expect("I-5.5 在清单里");
    let red: Vec<String> = report
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(_) => Some(invariant.to_string()),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
        })
        .collect();
    (i55, red)
}

/// 在 `image` 的一份拷贝上接着 `version` 发第一个文件版本（内容 `content_bytes` 字节），交回发出来的那一版与发完的镜像。
fn first_file_on_a_copy(
    make_filesystem_parameters: &MakeFilesystemParameters,
    image: &MemoryPool,
    mounted: &Mounted,
    version: &VersionWithoutFilePublishOutput,
    content_bytes: usize,
) -> (TransactionOutput, MemoryPool) {
    let mut devices = devices_of(image);
    let mut allocator = mounted.allocator.clone();
    let output = {
        let mut writer = PoolWriter::new(make_filesystem_parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            &version.root,
            FirstFile {
                content: &file_content().repeat(40)[..content_bytes],
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 7,
            },
            mounted.output.instance,
            &version.record_bytes,
        )
        .expect("第一个文件版本")
    };
    (output, memory_pool_of_sparse_devices(&devices))
}

fn slots_of(output: &TransactionOutput) -> String {
    let mut slots: Vec<(u32, u64)> = output
        .data_pointers
        .iter()
        .flat_map(|pointer| pointer.locations)
        .map(|location| (location.device.0, location.slot.0))
        .collect();
    slots.sort_unstable();
    let mut smallest: Vec<(u32, u64)> = Vec::new();
    for (device, slot) in slots {
        if !smallest.iter().any(|(d, _)| *d == device) {
            smallest.push((device, slot));
        }
    }
    let low_records: Vec<String> = output
        .allocation_records
        .iter()
        .filter(|record| record.device.0 == 0 && record.slot.0 < smallest.first().map_or(0, |(_, slot)| *slot) + 8)
        .filter(|record| record.slot.0 % 64 < 8)
        .take(6)
        .map(|record| {
            format!(
                "{}+{}{}@{}",
                record.slot.0,
                record.span_slots,
                if record.is_released { "R" } else { "A" },
                record.generation.0
            )
        })
        .collect();
    format!("{smallest:?} low_allocation_records_dev0={low_records:?}")
}

/// 形 A：只做过 mkfs 的池反复可写挂载 k 次（每次只写行、暖机，不写文件），第 k 次挂载接着现行那一版发第一个文件版本。
/// 用户决定的那一步（挂几次）放开扫 1..=K。
#[test]
fn shape_a_remount_count_sweep_then_first_file() {
    let k_max: usize = std::env::var("OPUS_K_MAX")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(20);
    for ring in [None, Some(128u64 << 20)] {
        let mut make_filesystem_parameters = parameters();
        if let Some(bytes) = ring {
            make_filesystem_parameters.geometry.journal_ring_bytes = bytes;
        }
        let stream = SharedStream::new();
        let mut devices = memory_devices(&stream);
        let genesis = make_filesystem(&make_filesystem_parameters, &mut devices).expect("mkfs");
        let unit_area_start = genesis.unit_area_start.slot().0;
        for k in 1..=k_max {
            let mounted = mount_writable(&make_filesystem_parameters, &mut devices).expect("可写挂载");
            let PoolVersion::WithoutFile(version) = &mounted.current else {
                panic!("没写过文件的池上现行那一版树表 0 条")
            };
            let image = memory_pool_of_sparse_devices(&devices);
            let (output, after) =
                first_file_on_a_copy(&make_filesystem_parameters, &image, &mounted, version, 3000);
            let (i55, red) = judge(&after);
            println!(
                "name=shape_a ring={} k={k} instance={} first_file_txg={} inode_tree={} unit_area_start={unit_area_start} expected={} smallest_slot_per_device={} i55={i55} red={red:?}",
                ring.map_or("default".to_string(), |bytes| format!("{}MiB", bytes >> 20)),
                mounted.output.instance.0,
                output.root.checkpoint_txg.0,
                output.tree_identifiers.inode.0,
                unit_area_start + 4,
                slots_of(&output),
            );
            drop(mounted);
        }
    }
}

/// 形 B：mkfs 之后可写挂载两次（实例 2），在实例 2 里接着连推 z 次零单元发布，再发第一个文件版本。z 放开扫 0..=Z。
#[test]
fn shape_b_zero_unit_publish_count_sweep_then_first_file() {
    let z_max: u64 = std::env::var("OPUS_Z_MAX")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(40);
    let make_filesystem_parameters = parameters();
    let stream = SharedStream::new();
    let mut devices = memory_devices(&stream);
    let genesis = make_filesystem(&make_filesystem_parameters, &mut devices).expect("mkfs");
    let unit_area_start = genesis.unit_area_start.slot().0;
    drop(mount_writable(&make_filesystem_parameters, &mut devices).expect("实例 1"));
    let mounted = mount_writable(&make_filesystem_parameters, &mut devices).expect("实例 2");
    let PoolVersion::WithoutFile(version) = &mounted.current else {
        panic!("树表 0 条")
    };
    let mut current = version.clone();
    for z in 0..=z_max {
        let image = memory_pool_of_sparse_devices(&devices);
        let (output, after) =
            first_file_on_a_copy(&make_filesystem_parameters, &image, &mounted, &current, 3000);
        let (i55, red) = judge(&after);
        println!(
            "name=shape_b z={z} first_file_txg={} inode_tree={} unit_area_start={unit_area_start} expected={} smallest_slot_per_device={} i55={i55} red={red:?}",
            output.root.checkpoint_txg.0,
            output.tree_identifiers.inode.0,
            unit_area_start + 4,
            slots_of(&output),
        );
        let mut writer = PoolWriter::new(&make_filesystem_parameters, devices.as_mut_slice());
        current = publish_without_units(
            &mut writer,
            &current.root,
            ZeroUnitPublishPlan {
                txg: CheckpointTxg(current.root.checkpoint_txg.0 + 1),
                counter: current.record.counter + 1,
                instance: mounted.output.instance,
                back_chain: back_chain_of(&current.record_bytes),
                rollback_floor: current.root.rollback_floor,
                tree_identifier_watermark: current.root.tree_identifier_watermark,
            },
        )
        .expect("零单元发布");
    }
}

/// 形 C 用的读故障：几段落点，每段一个位掩码，位 i 置了 ⇒ 这一段第 i + 1 次被读时设备报错（第 64 次之后照读）。
struct MaskedRange {
    device: DeviceIdentity,
    offset: u64,
    length: u64,
    mask: u64,
}

#[derive(Clone)]
struct SharedMasks(std::rc::Rc<std::cell::RefCell<(Vec<MaskedRange>, Vec<u64>)>>);

impl SharedMasks {
    fn fails(&self, device: DeviceIdentity, offset: u64, length: u64) -> bool {
        let mut state = self.0.borrow_mut();
        let (ranges, reads) = &mut *state;
        let mut fails = false;
        for (range, count) in ranges.iter().zip(reads.iter_mut()) {
            if range.device == device
                && offset < range.offset + range.length
                && range.offset < offset + length
            {
                *count += 1;
                if *count <= 64 && (range.mask >> (*count - 1)) & 1 == 1 {
                    fails = true;
                }
            }
        }
        fails
    }
}

struct MaskedDevice {
    identity: DeviceIdentity,
    inner: MemoryDevice,
    masks: SharedMasks,
}

impl singlefs_core::block_device::BlockDevice for MaskedDevice {
    fn read_at(
        &self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        if self
            .masks
            .fails(self.identity, offset.0, u64::try_from(buffer.len()).expect("长度"))
        {
            return Err(singlefs_core::block_device::BlockDeviceError::InputOutput(
                std::io::Error::other("攻方原型点名的这一次读坏"),
            ));
        }
        self.inner.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        bytes: &[u8],
        durability: singlefs_core::block_device::WriteDurability,
    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
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
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// 形 C：实例 1 发第一个文件版本 (1, 3)（数据单元 U + 4），崩在 txg 3 的根槽 FUA 之后、系统配置轮换之前；重开可写挂载时
/// txg 3 的根槽按掩码 S 瞬时读坏、那次发布的 journal 记录各段按掩码 J 瞬时读坏（两样都是用户控制不了的盘的行为，放开扫），
/// 挂上之后故障撤掉、接着现行那一版（树表 0 条时）发第一个文件版本。看 I-5.5 有没有在只红它一条的镜像上红。
#[test]
fn shape_c_transient_read_masks_then_first_file() {
    use singlefs_core::root_ring::{slot_offset, target_for_publish};
    use singlefs_harness::memory_pool::writes_and_segments;
    use singlefs_harness::segments::StepKind;
    let s_bits: u32 = std::env::var("OPUS_S_BITS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(6);
    let make_filesystem_parameters = parameters();
    let geometry = common::geometry();
    let stream = SharedStream::retaining_contents();
    let mut devices = memory_devices(&stream);
    make_filesystem(&make_filesystem_parameters, &mut devices).expect("mkfs");
    let mounted = mount_writable(&make_filesystem_parameters, &mut devices).expect("实例 1");
    let PoolVersion::WithoutFile(version) = &mounted.current else {
        panic!("树表 0 条")
    };
    let before = memory_pool_of_sparse_devices(&devices);
    let start = stream.operation_count();
    let mut allocator = mounted.allocator.clone();
    let dropped = {
        let mut writer = PoolWriter::new(&make_filesystem_parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            &version.root,
            FirstFile {
                content: &file_content(),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            mounted.output.instance,
            &version.record_bytes,
        )
        .expect("实例 1 第一个文件版本")
    };
    let dropped_txg = dropped.root.checkpoint_txg;
    let operations = stream.retained_operations();
    let (writes, _segments) = writes_and_segments(&operations[start..], &geometry);
    let target = target_for_publish(dropped_txg, geometry.root_ring_slots_per_region);
    let region_device = make_filesystem_parameters.region_devices
        [usize::try_from(target.region).expect("区域号")];
    let root_offset = slot_offset(target, geometry.fixed_structure_slot_spacing);
    let root_write = writes
        .iter()
        .position(|write| {
            write.is_force_unit_access && write.device == region_device && write.offset == root_offset
        })
        .expect("根槽 FUA 写");
    let persisted: Vec<bool> = (0..writes.len()).map(|index| index <= root_write).collect();
    let crashed = memory_pool_of_sparse_devices(&crash_state_devices(
        &before,
        &writes,
        &persisted,
        &SharedStream::new(),
    ));
    let record_ranges: Vec<(DeviceIdentity, u64, u64)> = writes[..=root_write]
        .iter()
        .filter(|write| write.kind == StepKind::JournalRecord)
        .map(|write| (write.device, write.offset.0, write.length_in_bytes()))
        .collect();
    println!(
        "name=shape_c_base dropped_txg={} dropped_data={} record_ranges={record_ranges:?} root_device={} root_offset={}",
        dropped_txg.0,
        slots_of(&dropped),
        region_device.0,
        root_offset.0
    );
    let record_masks: Vec<u64> = std::env::var("OPUS_RECORD_MASKS").map_or_else(
        |_| vec![0u64, 1, 3, u64::MAX],
        |value| {
            value
                .split(',')
                .map(|mask| u64::from_str_radix(mask.trim_start_matches("0x"), 16).expect("掩码"))
                .collect()
        },
    );
    // OPUS_ROOT_MASKS=singles：前 16 次读里恰好坏一次的 16 种、恰好好一次的 16 种，加全好、全坏；否则前 s_bits 次读的全部组合。
    let root_masks: Vec<u64> = if std::env::var("OPUS_ROOT_MASKS").as_deref() == Ok("singles") {
        let mut masks = vec![0u64, u64::MAX];
        for bit in 0..16 {
            masks.push(1u64 << bit);
            masks.push(u64::MAX & !(1u64 << bit));
        }
        masks
    } else {
        (0..(1u64 << s_bits)).collect()
    };
    for record_mask in record_masks {
        for root_mask in root_masks.iter().copied() {
            let mut ranges = vec![MaskedRange {
                device: region_device,
                offset: root_offset.0,
                length: u64::from(make_filesystem_parameters.geometry.physical_block_size),
                mask: root_mask,
            }];
            for (device, offset, length) in &record_ranges {
                ranges.push(MaskedRange {
                    device: *device,
                    offset: *offset,
                    length: *length,
                    mask: record_mask,
                });
            }
            let count = ranges.len();
            let masks = SharedMasks(std::rc::Rc::new(std::cell::RefCell::new((
                ranges,
                vec![0; count],
            ))));
            let mut faulted: Vec<(DeviceIdentity, MaskedDevice)> = devices_of(&crashed)
                .into_iter()
                .map(|(identity, inner)| {
                    (
                        identity,
                        MaskedDevice {
                            identity,
                            inner,
                            masks: masks.clone(),
                        },
                    )
                })
                .collect();
            let outcome = mount_writable(&make_filesystem_parameters, &mut faulted);
            let root_reads = masks.0.borrow().1[0];
            let unfaulted: Vec<(DeviceIdentity, MemoryDevice)> = faulted
                .into_iter()
                .map(|(identity, device)| (identity, device.inner))
                .collect();
            let image = memory_pool_of_sparse_devices(&unfaulted);
            let line = match outcome {
                Err(error) => format!("mount=refused({error:?})"),
                Ok(second) => match &second.current {
                    PoolVersion::WithoutFile(version) => {
                        let (output, after) = first_file_on_a_copy(
                            &make_filesystem_parameters,
                            &image,
                            &second,
                            version,
                            3000,
                        );
                        let (i55, red) = judge(&after);
                        format!(
                            "mount=ok effective_txg={} watermark={} isolated={:?} first_file_txg={} inode_tree={} slots={} i55={i55} red={red:?}",
                            second.output.effective_root.checkpoint_txg.0,
                            version.root.tree_identifier_watermark,
                            second.output.isolated_slots_per_device,
                            output.root.checkpoint_txg.0,
                            output.tree_identifiers.inode.0,
                            slots_of(&output)
                        )
                    }
                    PoolVersion::WithFile(_) | PoolVersion::WithTreesAndNoFile(_) => {
                        format!(
                            "mount=ok effective_txg={} current_has_trees",
                            second.output.effective_root.checkpoint_txg.0
                        )
                    }
                },
            };
            println!(
                "name=shape_c record_mask={record_mask:#x} root_mask={root_mask:#b} root_reads={root_reads} {line}"
            );
        }
    }
}

/// 形 D 挂载之后用户接着做的事（放开扫）：什么都不做、直接发第一个文件版本、先推 1 / 2 次零单元发布再发、先重挂一次再发。
#[derive(Clone, Copy, Debug)]
enum AfterTheMount {
    Nothing,
    FirstFile,
    ZeroUnitThenFirstFile(u64),
    RemountThenFirstFile,
}

/// 形 D 的一格：崩溃状态 → 可写挂载 → 用户动作 → 池级 checker。交回 (I-5.5 的判定, 红了的条, 第一个文件版本的落点或 "-")。
fn shape_d_cell(crashed: &MemoryPool, action: AfterTheMount) -> (String, Vec<String>, String) {
    let make_filesystem_parameters = parameters();
    let mut devices = devices_of(crashed);
    let mounted = mount_writable(&make_filesystem_parameters, &mut devices).expect("崩溃状态要挂得上");
    let (mounted, mut current) = match (action, &mounted.current) {
        (AfterTheMount::Nothing, _) | (_, PoolVersion::WithFile(_) | PoolVersion::WithTreesAndNoFile(_)) => {
            let image = memory_pool_of_sparse_devices(&devices);
            let (i55, red) = judge(&image);
            return (i55, red, "-".to_string());
        }
        (AfterTheMount::RemountThenFirstFile, PoolVersion::WithoutFile(_)) => {
            drop(mounted);
            let again = mount_writable(&make_filesystem_parameters, &mut devices).expect("重挂");
            let PoolVersion::WithoutFile(version) = &again.current else {
                panic!("重挂之后仍是树表 0 条")
            };
            let version = version.clone();
            (again, version)
        }
        (_, PoolVersion::WithoutFile(version)) => {
            let version = version.clone();
            (mounted, version)
        }
    };
    if let AfterTheMount::ZeroUnitThenFirstFile(count) = action {
        for _ in 0..count {
            let mut writer = PoolWriter::new(&make_filesystem_parameters, devices.as_mut_slice());
            current = publish_without_units(
                &mut writer,
                &current.root,
                ZeroUnitPublishPlan {
                    txg: CheckpointTxg(current.root.checkpoint_txg.0 + 1),
                    counter: current.record.counter + 1,
                    instance: mounted.output.instance,
                    back_chain: back_chain_of(&current.record_bytes),
                    rollback_floor: current.root.rollback_floor,
                    tree_identifier_watermark: current.root.tree_identifier_watermark,
                },
            )
            .expect("零单元发布");
        }
    }
    let image = memory_pool_of_sparse_devices(&devices);
    let (output, after) =
        first_file_on_a_copy(&make_filesystem_parameters, &image, &mounted, &current, 2000);
    let (i55, red) = judge(&after);
    (
        i55,
        red,
        format!("txg={} inode_tree={} slots={}", output.root.checkpoint_txg.0, output.tree_identifiers.inode.0, slots_of(&output)),
    )
}

/// 形 D：mkfs → 取号 → 暖机两代 → 第一个文件版本，崩在那次发布录制流的每一个崩溃状态上（逐段展开，段内持久写的每个子集，
/// 前面的段全持久：1 + Σ(2^|段| − 1)），挂载之后五种动作。原地覆写的撕裂第三态不在里面。
#[test]
fn shape_d_crash_inside_the_first_file_version_then_build_again() {
    use singlefs_core::make_filesystem::allocator_after_make_filesystem;
    use singlefs_core::transaction::{acquire_instance, warm_up};
    use singlefs_harness::memory_pool::{closed_form_state_count, writes_and_segments};
    let count_only = std::env::var("OPUS_D_COUNT_ONLY").is_ok();
    let stream = SharedStream::retaining_contents();
    let mut devices = memory_devices(&stream);
    let make_filesystem_parameters = parameters();
    let genesis = make_filesystem(&make_filesystem_parameters, &mut devices).expect("mkfs");
    let mut allocator = allocator_after_make_filesystem(&make_filesystem_parameters, &devices, &genesis);
    let (before, start) = {
        let mut writer = PoolWriter::new(&make_filesystem_parameters, &mut devices);
        let instance = acquire_instance(&mut writer).expect("取号");
        let warm = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        drop(writer);
        let before = memory_pool_of_sparse_devices(&devices);
        let start = stream.operation_count();
        let mut writer = PoolWriter::new(&make_filesystem_parameters, &mut devices);
        publish_first_file(
            &mut writer,
            &mut allocator,
            warm.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &file_content(),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warm.last_record_bytes,
        )
        .expect("第一个文件版本");
        (before, start)
    };
    let operations = stream.retained_operations();
    let (writes, segments) = writes_and_segments(&operations[start..], &common::geometry());
    let sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
    println!(
        "name=shape_d_count writes={} segment_sizes={sizes:?} closed_form_state_count={}",
        writes.len(),
        closed_form_state_count(&segments)
    );
    // 派发约束：一段历史自己的全量超过约 10⁶ 个崩溃状态就不跑（这段 1 + Σ(2^|段| − 1) = 16777223），只报数。
    if count_only || closed_form_state_count(&segments) > 1_000_000 {
        println!("name=shape_d_skipped reason=closed_form_state_count_over_1e6_or_count_only");
        return;
    }
    let actions = [
        AfterTheMount::Nothing,
        AfterTheMount::FirstFile,
        AfterTheMount::ZeroUnitThenFirstFile(1),
        AfterTheMount::ZeroUnitThenFirstFile(2),
        AfterTheMount::RemountThenFirstFile,
    ];
    let mut persisted_before = vec![false; writes.len()];
    let mut states = 0u64;
    let mut cells = 0u64;
    let mut i55_red_cells = 0u64;
    let mut other_red_cells = 0u64;
    let mut holds_cells = 0u64;
    let mut judge_state = |persisted: &[bool], label: String| {
        let crashed = memory_pool_of_sparse_devices(&crash_state_devices(
            &before,
            &writes,
            persisted,
            &SharedStream::new(),
        ));
        for action in actions {
            let (i55, red, first) = shape_d_cell(&crashed, action);
            cells += 1;
            if i55 == "holds" {
                holds_cells += 1;
            }
            if red.iter().any(|invariant| invariant == "I-5.5") {
                i55_red_cells += 1;
                println!("name=shape_d_i55_red state={label} action={action:?} first={first} i55={i55} red={red:?}");
            } else if !red.is_empty() {
                other_red_cells += 1;
                println!("name=shape_d_other_red state={label} action={action:?} first={first} red={red:?}");
            }
        }
    };
    for (segment_index, segment) in segments.iter().enumerate() {
        for subset in 0..(1u64 << segment.len()) - 1 {
            let mut persisted = persisted_before.clone();
            for (bit, write_index) in segment.iter().enumerate() {
                if subset & (1 << bit) != 0 {
                    persisted[*write_index] = true;
                }
            }
            states += 1;
            judge_state(&persisted, format!("seg{segment_index}/sub{subset:#b}"));
        }
        for write_index in segment {
            persisted_before[*write_index] = true;
        }
    }
    states += 1;
    judge_state(&persisted_before, "all".to_string());
    println!(
        "name=shape_d_summary states={states} cells={cells} i55_holds_cells={holds_cells} i55_red_cells={i55_red_cells} other_red_cells={other_red_cells}"
    );
}

/// 形 E：长度放开扫（零长、1 字节、正好一个数据单元、多一字节、多个单元），挂 k ∈ {1, 2, 10} 次之后发第一个文件版本。
#[test]
fn shape_e_lengths_after_k_mounts() {
    let make_filesystem_parameters = parameters();
    for length in [0usize, 1, 32768, 32769, 100_000] {
        let stream = SharedStream::new();
        let mut devices = memory_devices(&stream);
        let genesis = make_filesystem(&make_filesystem_parameters, &mut devices).expect("mkfs");
        let unit_area_start = genesis.unit_area_start.slot().0;
        for k in 1..=10usize {
            let mounted = mount_writable(&make_filesystem_parameters, &mut devices).expect("可写挂载");
            if k == 1 || k == 2 || k == 10 {
                let PoolVersion::WithoutFile(version) = &mounted.current else {
                    panic!("树表 0 条")
                };
                let image = memory_pool_of_sparse_devices(&devices);
                let mut copy = devices_of(&image);
                let mut allocator = mounted.allocator.clone();
                let result = {
                    let mut writer = PoolWriter::new(&make_filesystem_parameters, copy.as_mut_slice());
                    publish_first_file(
                        &mut writer,
                        &mut allocator,
                        &version.root,
                        FirstFile {
                            content: &file_content().repeat(40)[..length],
                            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 7,
                        },
                        mounted.output.instance,
                        &version.record_bytes,
                    )
                };
                match result {
                    Ok(output) => {
                        let (i55, red) = judge(&memory_pool_of_sparse_devices(&copy));
                        println!(
                            "name=shape_e length={length} k={k} data_units={} expected={} smallest_slot_per_device={} i55={i55} red={red:?}",
                            output.data_pointers.len(),
                            unit_area_start + 4,
                            slots_of(&output)
                        );
                    }
                    Err(error) => println!("name=shape_e length={length} k={k} publish_refused={error:?}"),
                }
            }
            drop(mounted);
        }
    }
}
