//! m3-delete-r1 攻方腿（副本专用，被攻过零轮）：原型「删除一批」（`transaction::publish_unlink_batch_for_tests`，从尾部截 extent、
//! inode 1 记录 nlink = 0、size 跟着截）上量 U4（待删占用扣两次）与 U5（删掉 inode 1 之后恢复拒不拒），外加 U3（续做同一批两次盘上字节）。
//! 全部在内存稀疏盘上跑，不建镜像文件；只打印 `E name=...` 行，断言钉住看到的事实。

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::admission::{
    admit_on_every_device, AdmissionReading, BytesOnOneDevice, BytesSummedOverAllReplicas,
    DemandOnDevice, PoolWideCommitments,
};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{
    make_filesystem, MakeFilesystemParameters, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_sequential_write,
    publish_unlink_batch_for_tests, warm_up, FirstFile, PoolVersion, PoolWriter, PublishError,
    TransactionOutput, TransactionUnit,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, SLOT_BYTES};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

const IMAGE_BYTES: u64 = 4 << 30;
const WRITE_TIME: u64 = 1_788_000_000;
const FILE_UNITS: usize = 6;

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

fn parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: *b"_SFS-M3DEL-ATK1\0",
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

fn content_needing(units: usize, seed: usize) -> Vec<u8> {
    let payload = data_unit_payload_capacity();
    ((0..(units - 1) * payload + payload / 2)
        .map(|index| u8::try_from((index * 31 + seed * 7 + 1) % 251).expect("小于 256")))
    .collect()
}

fn memory_pool_of(devices: &Devices) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: IMAGE_BYTES,
    }
}

fn devices_of(image: &MemoryPool) -> Devices {
    image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            (*identity, device)
        })
        .collect()
}

/// mkfs → 取号 → 暖机 → 第一个文件（3000 字节）→ 顺序写成 `FILE_UNITS` 个数据单元，同一个实例、同一个进程。
fn pool_with_a_file_of_six_units() -> (Devices, PoolAllocator, TransactionOutput, InstanceGeneration) {
    let parameters = parameters();
    let mut devices: Devices = (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
        Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
    );
    let first_content = content_needing(1, 1);
    let (instance, output) = {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut pool).expect("取号");
        let warm = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
        let first = publish_first_file(
            &mut pool,
            &mut allocator,
            warm.roots.last().expect("暖机根"),
            FirstFile { content: &first_content, write_time_seconds: WRITE_TIME },
            instance,
            &warm.last_record_bytes,
        )
        .expect("第一个文件");
        let content = content_needing(FILE_UNITS, 2);
        let six = publish_sequential_write(
            &mut pool,
            &mut allocator,
            &first,
            FirstFile { content: &content, write_time_seconds: WRITE_TIME + 60 },
            instance,
        )
        .expect("顺序写六个单元");
        (instance, six)
    };
    (devices, allocator, output, instance)
}

fn batch(
    devices: &mut Devices,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    keep_units: u64,
    size_after: u64,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    let parameters = parameters();
    let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
    publish_unlink_batch_for_tests(&mut pool, allocator, previous, keep_units, size_after, instance)
}

fn per_device_counts(allocator: &PoolAllocator) -> String {
    allocator
        .devices
        .iter()
        .map(|map| {
            format!(
                "dev{}_allocated={} dev{}_free={} dev{}_deferred={}",
                map.device.0,
                map.allocated_slots(),
                map.device.0,
                map.free_slots(),
                map.device.0,
                map.deferred_slots()
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// 这次发布新占的槽（逐盘）：分配代 = 这次 txg、未释放的记录的跨度之和。
fn slots_taken_by(output: &TransactionOutput, device: DeviceIdentity) -> u64 {
    output
        .allocation_records
        .iter()
        .filter(|record| {
            record.device == device
                && !record.is_released
                && record.generation == output.root.checkpoint_txg
        })
        .map(|record| u64::from(record.span_slots))
        .sum()
}

/// 被删的数据单元里有几个槽还没回到空闲（逐盘）：还被 extent 树引用的 + 已释放还在 defer 里的。
fn deleted_data_slots_not_free(
    allocator: &PoolAllocator,
    deleted: &[singlefs_core::pointer::DataPointer],
    device: DeviceIdentity,
) -> u64 {
    let map = allocator
        .devices
        .iter()
        .find(|map| map.device == device)
        .expect("盘");
    deleted
        .iter()
        .flat_map(|pointer| pointer.locations.iter())
        .filter(|location| location.device == device)
        .map(|location| {
            (0..TransactionUnit::Data(singlefs_core::address::DataUnitIndexInFile(0)).span_slots())
                .filter(|offset| !map.is_free(SlotNumber(location.slot.0 + offset)))
                .count() as u64
        })
        .sum()
}

fn available_under(allocator: &PoolAllocator, pending: u64) -> Vec<i128> {
    AdmissionReading::of_allocator(
        allocator,
        BytesOnOneDevice::ZERO,
        PoolWideCommitments {
            pending_delete: BytesSummedOverAllReplicas(pending),
            committed_reservation: BytesSummedOverAllReplicas::ZERO,
            checkpoint_reserve_pool: BytesSummedOverAllReplicas::ZERO,
        },
    )
    .available_on_each_device()
    .into_iter()
    .map(|(_, available)| available.0)
    .collect()
}

fn recovery_outcome(image: &MemoryPool) -> String {
    match recover(image, JournalPolicy::Consult).outcome {
        RecoveryOutcome::NoFile { root } => format!("NoFile root={root:?}"),
        RecoveryOutcome::FileRead { root, content } => {
            format!("FileRead root={root:?} bytes={}", content.len())
        }
        RecoveryOutcome::Failed { root, failure } => format!("Failed root={root:?} failure={failure:?}"),
    }
}

fn read_only_mount(image: &MemoryPool) -> String {
    match mount_read_only(image) {
        Ok(_) => "Ok".to_owned(),
        Err(failure) => format!("Err({failure:?})"),
    }
}

fn writable_mount(image: &MemoryPool) -> String {
    let mut devices = devices_of(image);
    match mount_writable(&parameters(), &mut devices) {
        Ok(mounted) => match mounted.current {
            PoolVersion::WithFile(version) => format!(
                "Ok(WithFile units={} size={})",
                version.data_pointers.len(),
                version.inode_record.size
            ),
            PoolVersion::WithoutFile(_) => "Ok(WithoutFile)".to_owned(),
        },
        Err(error) => format!("Err({error:?})"),
    }
}

fn checker_red(image: &MemoryPool) -> Vec<String> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some(format!("{invariant}:{detail}")),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
        })
        .collect()
}

fn set_nlink_zero_for_inode_one() {
    singlefs_core::records::NLINK_ZERO_INODE_FOR_TESTS.with(|cell| cell.set(Some(1)));
}

/// U4 / U5：意图那次（原型里是 keep = 6、size 不变、nlink = 0 的一次发布，代替写意图）之后三批（留 4、2、0 个单元），
/// 每一步之后逐盘的计数、四种「待删占用」口径下式子的可用、冷恢复、只读挂载、可写挂载与池级 checker。
#[test]
fn three_batches_after_the_intent_space_recovery_and_checker() {
    let (mut devices, mut allocator, six, instance) = pool_with_a_file_of_six_units();
    let payload = u64::try_from(data_unit_payload_capacity()).expect("净荷");
    let data_unit_span = TransactionUnit::Data(singlefs_core::address::DataUnitIndexInFile(0)).span_slots();
    let all_replicas_of_one_unit = data_unit_span * SLOT_BYTES * 2;
    let deleted = six.data_pointers.clone();
    let mut steps: Vec<(String, u64, TransactionOutput, PoolAllocator, MemoryPool)> = vec![(
        "before_unlink".to_owned(),
        6,
        six.clone(),
        allocator.clone(),
        memory_pool_of(&devices),
    )];
    set_nlink_zero_for_inode_one();
    let mut previous = batch(&mut devices, &mut allocator, &six, 6, six.inode_record.size, instance)
        .expect("意图那次（留 6）");
    steps.push(("intent".to_owned(), 6, previous.clone(), allocator.clone(), memory_pool_of(&devices)));
    for keep in [4u64, 2, 0] {
        let next = batch(&mut devices, &mut allocator, &previous, keep, keep * payload, instance)
            .unwrap_or_else(|error| panic!("留 {keep} 那一批：{error:?}"));
        steps.push((format!("batch_keep_{keep}"), keep, next.clone(), allocator.clone(), memory_pool_of(&devices)));
        previous = next;
    }
    // 每一次发布新占的槽（两盘之和，字节）：「剩下几批还要多少」那一口径由它倒着加。
    let demand_bytes: Vec<u64> = steps
        .iter()
        .map(|(_, _, output, _, _)| {
            (0..2u32).map(|device| slots_taken_by(output, DeviceIdentity(device))).sum::<u64>() * SLOT_BYTES
        })
        .collect();
    for (index, (name, keep, output, allocator_after, image)) in steps.iter().enumerate() {
        let has_intent = index >= 1;
        let unreleased = if has_intent { keep * all_replicas_of_one_unit } else { 0 };
        let unreclaimed = if has_intent {
            (0..2u32)
                .map(|device| deleted_data_slots_not_free(allocator_after, &deleted, DeviceIdentity(device)))
                .sum::<u64>()
                * SLOT_BYTES
        } else {
            0
        };
        let remaining_batches: u64 = if has_intent && index + 1 < steps.len() {
            demand_bytes[index + 1..].iter().sum()
        } else {
            0
        };
        println!(
            "E name=state step={name} txg={} keep={keep} size={} demand_bytes_both_devices={} {}",
            output.root.checkpoint_txg.0,
            output.inode_record.size,
            demand_bytes[index],
            per_device_counts(allocator_after)
        );
        for (kind, pending) in [
            ("A_unreleased_extents", unreleased),
            ("A2_unreclaimed_extents", unreclaimed),
            ("B_zero", 0),
            ("C_remaining_batches_metadata", remaining_batches),
        ] {
            println!(
                "E name=available step={name} pending_kind={kind} pending_bytes_all_replicas={pending} available_bytes_per_device={:?}",
                available_under(allocator_after, pending)
            );
        }
        println!("E name=recover step={name} outcome={}", recovery_outcome(image));
        println!("E name=read_only_mount step={name} result={}", read_only_mount(image));
        println!("E name=writable_mount step={name} result={}", writable_mount(image));
        println!("E name=checker step={name} red={:?}", checker_red(image));
        assert!(checker_red(image).is_empty(), "池级 checker 在 {name} 上不红");
        let recovered = recovery_outcome(image);
        if *keep == 0 {
            assert!(recovered.contains("extent 树里第一个文件一个数据单元都没有"), "删到 0 个单元之后冷恢复拒：{recovered}");
            assert!(writable_mount(image).starts_with("Err(Recovery(UnitMalformed"), "可写挂载同样拒");
        } else {
            assert!(recovered.starts_with("FileRead"), "中间态冷恢复读回一个（截短的）文件：{recovered}");
        }
    }

    // 扣两次那一格：意图刚提交、一个单元都还没释放。需求取「每块盘此刻的空闲字节」——分配器实际发得出这么多——，
    // 口径 B 放行、口径 A 拒，差的正好是 6 个单元一份副本的字节。
    let (_, _, _, allocator_at_the_intent, _) = &steps[1];
    let demand: Vec<DemandOnDevice> = allocator_at_the_intent
        .devices
        .iter()
        .map(|map| DemandOnDevice { device: map.device, bytes: BytesOnOneDevice::of_slots(map.free_slots()) })
        .collect();
    for (kind, pending) in [("A_unreleased_extents", 6 * all_replicas_of_one_unit), ("B_zero", 0)] {
        let reading = AdmissionReading::of_allocator(
            allocator_at_the_intent,
            BytesOnOneDevice::ZERO,
            PoolWideCommitments {
                pending_delete: BytesSummedOverAllReplicas(pending),
                committed_reservation: BytesSummedOverAllReplicas::ZERO,
                checkpoint_reserve_pool: BytesSummedOverAllReplicas::ZERO,
            },
        );
        let verdict = admit_on_every_device(&reading, &demand);
        println!(
            "E name=double_count step=intent pending_kind={kind} demand_equals_free_slots=true admitted={} detail={:?}",
            verdict.is_ok(),
            verdict.clone().err().map(|refusal| refusal.short_devices)
        );
        assert_eq!(verdict.is_ok(), kind == "B_zero", "需求等于空闲槽：口径 B 放行、口径 A 拒");
    }
}

/// U3：同一批（意图之后留 4）续做两次。同一个实例、同一个分配器拷贝上重跑：盘上逐字节相同；
/// 崩在这一批落盘之前、重开可写挂载（新实例、写行、暖机）之后再做同一批：目标态相同，盘上字节不同。
#[test]
fn the_same_batch_redone_after_a_crash_writes_other_bytes_for_the_same_target() {
    let (mut devices, mut allocator, six, instance) = pool_with_a_file_of_six_units();
    let payload = u64::try_from(data_unit_payload_capacity()).expect("净荷");
    set_nlink_zero_for_inode_one();
    let intent = batch(&mut devices, &mut allocator, &six, 6, six.inode_record.size, instance)
        .expect("意图那次");
    let image_at_the_intent = memory_pool_of(&devices);

    let mut devices_one = devices_of(&image_at_the_intent);
    let mut allocator_one = allocator.clone();
    let first = batch(&mut devices_one, &mut allocator_one, &intent, 4, 4 * payload, instance).expect("第一次");
    let mut devices_two = devices_of(&image_at_the_intent);
    let mut allocator_two = allocator.clone();
    let second = batch(&mut devices_two, &mut allocator_two, &intent, 4, 4 * payload, instance).expect("第二次");
    let same_instance_equal = memory_pool_of(&devices_one) == memory_pool_of(&devices_two);
    println!("E name=redo_same_instance images_byte_equal={same_instance_equal} outputs_equal={}", first == second);

    let mut devices_three = devices_of(&image_at_the_intent);
    let mounted = mount_writable(&parameters(), &mut devices_three).expect("崩溃之后可写挂载");
    let PoolVersion::WithFile(current) = mounted.current else { panic!("带文件的一版") };
    let mut allocator_three = mounted.allocator;
    let third = batch(&mut devices_three, &mut allocator_three, &current, 4, 4 * payload, mounted.output.instance)
        .expect("新实例里再做同一批");
    let rewritten_bytes = |output: &TransactionOutput| -> Vec<(String, Vec<u8>)> {
        output
            .units
            .iter()
            .filter(|unit| output.rewritten.contains(&unit.identity))
            .map(|unit| (format!("{:?}", unit.identity), unit.bytes.clone()))
            .collect()
    };
    let first_units = rewritten_bytes(&first);
    let third_units = rewritten_bytes(&third);
    let equal_units = first_units
        .iter()
        .filter(|(role, bytes)| third_units.iter().any(|(other_role, other)| other_role == role && other == bytes))
        .count();
    let inode_record_without_change_count = |output: &TransactionOutput| {
        (output.inode_record.inode, output.inode_record.object_birth, output.inode_record.size)
    };
    let recovered_one = recovery_outcome(&memory_pool_of(&devices_one));
    let recovered_three = recovery_outcome(&memory_pool_of(&devices_three));
    println!(
        "E name=redo_after_crash instance_first={} instance_redo={} txg_first={} txg_redo={} rewritten_units_first={} rewritten_units_redo={} rewritten_units_byte_equal={} data_pointers_equal={} inode_record_equal_but_change_count={} change_count_first={} change_count_redo={}",
        instance.0,
        mounted.output.instance.0,
        first.root.checkpoint_txg.0,
        third.root.checkpoint_txg.0,
        first_units.len(),
        third_units.len(),
        equal_units,
        first.data_pointers == third.data_pointers,
        inode_record_without_change_count(&first) == inode_record_without_change_count(&third),
        first.inode_record.change_count,
        third.inode_record.change_count
    );
    println!("E name=redo_after_crash_recover first={recovered_one} redo={recovered_three}");
    assert!(same_instance_equal, "同一个实例、同一个分配器拷贝上重跑同一批：盘上逐字节相同");
    assert_eq!(equal_units, 0, "新实例里重做同一批：重写的单元没有一个逐字节相同");
    assert_eq!(first.data_pointers, third.data_pointers, "留下的数据指针相同");
    assert_eq!(recovered_one.split(" bytes=").nth(1), recovered_three.split(" bytes=").nth(1), "读回长度相同");
}

/// U3 / U4：空间满到一次覆盖写被准入拒时，删除的一批同样被拒（这一批要重写 extent 节点、inode 叶与根、分配记录树、记账树、映射树，
/// 是有普通分配的发布、准入判它）。满是在分配器的拷贝上把空闲槽标成已分配造的，盘上不写。
#[test]
fn a_delete_batch_is_refused_by_admission_on_a_pool_too_full_for_one_overwrite() {
    let (mut devices, mut allocator, six, instance) = pool_with_a_file_of_six_units();
    let payload = u64::try_from(data_unit_payload_capacity()).expect("净荷");
    set_nlink_zero_for_inode_one();
    let intent = batch(&mut devices, &mut allocator, &six, 6, six.inode_record.size, instance)
        .expect("意图那次");
    // 这一批要新占多少槽：在拷贝上先跑一次。
    let image = memory_pool_of(&devices);
    let mut probe_devices = devices_of(&image);
    let mut probe_allocator = allocator.clone();
    let probe = batch(&mut probe_devices, &mut probe_allocator, &intent, 4, 4 * payload, instance).expect("探一次");
    let demand_slots: Vec<u64> = (0..2u32).map(|device| slots_taken_by(&probe, DeviceIdentity(device))).collect();
    // 把每块盘的空闲槽标成已分配，只留 demand − 1 个。
    for map in &mut allocator.devices {
        let device = map.device;
        let keep_free = demand_slots[usize::try_from(device.0).expect("盘号")] - 1;
        let start = map.unit_area_start().slot().0;
        let end = start + map.unit_area_slots();
        let mut slot = start;
        while map.free_slots() > keep_free && slot < end {
            if map.is_free(SlotNumber(slot)) {
                map.mark_allocated(SlotNumber(slot), 1);
            }
            slot += 1;
        }
    }
    println!("E name=full_pool demand_slots_of_the_batch={demand_slots:?} {}", per_device_counts(&allocator));
    let mut full_devices = devices_of(&image);
    let mut full_allocator = allocator.clone();
    let refused = batch(&mut full_devices, &mut full_allocator, &intent, 4, 4 * payload, instance);
    println!("E name=full_pool_delete_batch result={:?}", refused.as_ref().map(|output| output.root.checkpoint_txg.0));
    let parameters = parameters();
    let mut overwrite_devices = devices_of(&image);
    let mut overwrite_allocator = allocator.clone();
    let one_unit = content_needing(1, 9);
    let overwrite = {
        let mut pool = PoolWriter::new(&parameters, overwrite_devices.as_mut_slice());
        singlefs_core::transaction::publish_overwrite(
            &mut pool,
            &mut overwrite_allocator,
            &intent,
            FirstFile { content: &one_unit, write_time_seconds: WRITE_TIME + 120 },
            instance,
        )
    };
    println!("E name=full_pool_overwrite result={:?}", overwrite.as_ref().map(|output| output.root.checkpoint_txg.0));
    assert!(matches!(refused, Err(PublishError::SpaceAdmissionRefused(_))), "删除的一批被准入拒");
}

/// U5 / I-4.6：一批删掉尾部两个单元、inode size 不跟着截（「只删 extent、记录等回收完成再动」那一种中间态）。
#[test]
fn a_batch_that_leaves_the_size_is_refused_by_recovery() {
    let (mut devices, mut allocator, six, instance) = pool_with_a_file_of_six_units();
    set_nlink_zero_for_inode_one();
    let intent = batch(&mut devices, &mut allocator, &six, 6, six.inode_record.size, instance)
        .expect("意图那次");
    let kept_size = batch(&mut devices, &mut allocator, &intent, 4, six.inode_record.size, instance)
        .expect("留 4、size 不变");
    let image = memory_pool_of(&devices);
    println!(
        "E name=size_kept step=batch_keep_4 size={} units={} recover={} writable_mount={} checker_red={:?}",
        kept_size.inode_record.size,
        kept_size.data_pointers.len(),
        recovery_outcome(&image),
        writable_mount(&image),
        checker_red(&image)
    );
    assert!(recovery_outcome(&image).contains("E142 走读同款"), "size 不跟着截的中间态冷恢复拒");
    assert!(checker_red(&image).is_empty(), "池级 checker 不红");
}

/// U6：每次发布的写表按屏障 / FUA 切段之后的两态闭式（1 + Σ(2^|段| − 1)，不含原地覆写的第三态）。
#[test]
fn two_state_closed_form_of_each_unlink_publish() {
    use singlefs_harness::memory_pool::{closed_form_state_count, writes_and_segments};
    use singlefs_harness::segments::FixedGeometry;
    use singlefs_harness::{RecordingBlockDevice, SharedStream};
    let (devices, mut allocator, six, instance) = pool_with_a_file_of_six_units();
    let payload = u64::try_from(data_unit_payload_capacity()).expect("净荷");
    let stream = SharedStream::retaining_contents();
    let mut recorded: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = devices
        .into_iter()
        .map(|(identity, device)| (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())))
        .collect();
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: 4096,
        journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
        root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
    };
    let parameters = parameters();
    set_nlink_zero_for_inode_one();
    let mut previous = six;
    let mut total = 0u64;
    for (name, keep, size) in [
        ("intent_keep_6", 6u64, None),
        ("batch_keep_4", 4, Some(4 * payload)),
        ("batch_keep_2", 2, Some(2 * payload)),
        ("batch_keep_0", 0, Some(0)),
    ] {
        let before = stream.retained_operations().len();
        let next = {
            let mut pool = PoolWriter::new(&parameters, recorded.as_mut_slice());
            publish_unlink_batch_for_tests(
                &mut pool,
                &mut allocator,
                &previous,
                keep,
                size.unwrap_or(previous.inode_record.size),
                instance,
            )
            .expect("一批")
        };
        let operations = stream.retained_operations();
        let (writes, segments) = writes_and_segments(&operations[before..], &geometry);
        let states = closed_form_state_count(&segments);
        total += states - 1;
        println!(
            "E name=closed_form publish={name} txg={} writes={} segments={} segment_sizes={:?} two_state_closed_form={states}",
            next.root.checkpoint_txg.0,
            writes.len(),
            segments.len(),
            segments.iter().map(Vec::len).collect::<Vec<_>>()
        );
        previous = next;
    }
    println!("E name=closed_form_sum publishes=4 sum_of_states_minus_one_each={total}");
    // 对照：同一个池上一次覆写（一个单元）。
    let before = stream.retained_operations().len();
    let one_unit = content_needing(1, 5);
    let overwrite_result = {
        let mut pool = PoolWriter::new(&parameters, recorded.as_mut_slice());
        singlefs_core::transaction::publish_sequential_write(
            &mut pool,
            &mut allocator,
            &previous,
            FirstFile { content: &one_unit, write_time_seconds: WRITE_TIME + 300 },
            instance,
        )
    };
    let operations = stream.retained_operations();
    let (writes, segments) = writes_and_segments(&operations[before..], &geometry);
    println!(
        "E name=closed_form publish=sequential_write_one_unit_after_the_delete result_ok={} writes={} segments={} segment_sizes={:?} two_state_closed_form={}",
        overwrite_result.is_ok(),
        writes.len(),
        segments.len(),
        segments.iter().map(Vec::len).collect::<Vec<_>>(),
        closed_form_state_count(&segments)
    );
    if let Err(error) = &overwrite_result {
        println!("E name=write_after_delete error={error:?}");
    }
}

/// U5：删到 0 个单元之后，今天的写入口还能做什么——再建第一个文件（`publish_first_file`）、给 inode 1 顺序写（同一个 (inode 号, 对象出生代)）、
/// 建新 inode（`publish_new_inodes`），各自的结局与之后冷恢复读到什么。
#[test]
fn what_the_writer_does_after_the_file_is_deleted() {
    let (mut devices, mut allocator, six, instance) = pool_with_a_file_of_six_units();
    let payload = u64::try_from(data_unit_payload_capacity()).expect("净荷");
    set_nlink_zero_for_inode_one();
    let mut previous = batch(&mut devices, &mut allocator, &six, 6, six.inode_record.size, instance).expect("意图");
    for keep in [4u64, 2, 0] {
        previous = batch(&mut devices, &mut allocator, &previous, keep, keep * payload, instance).expect("一批");
    }
    let deleted = previous;
    let image = memory_pool_of(&devices);
    let parameters = parameters();
    let content = content_needing(1, 11);

    let mut first_devices = devices_of(&image);
    let mut first_allocator = allocator.clone();
    let first = {
        let mut pool = PoolWriter::new(&parameters, first_devices.as_mut_slice());
        publish_first_file(
            &mut pool,
            &mut first_allocator,
            &deleted.root,
            FirstFile { content: &content, write_time_seconds: WRITE_TIME + 400 },
            instance,
            &deleted.record_bytes,
        )
    };
    println!("E name=after_delete op=publish_first_file result={:?}", first.as_ref().map(|output| output.root.checkpoint_txg.0));
    assert!(matches!(first, Err(PublishError::FirstFileVersionOnAVersionThatAlreadyHasAFile { tree_table_entries: 7 })));

    singlefs_core::records::NLINK_ZERO_INODE_FOR_TESTS.with(|cell| cell.set(None));
    let mut write_devices = devices_of(&image);
    let mut write_allocator = allocator.clone();
    let rewritten = {
        let mut pool = PoolWriter::new(&parameters, write_devices.as_mut_slice());
        publish_sequential_write(
            &mut pool,
            &mut write_allocator,
            &deleted,
            FirstFile { content: &content, write_time_seconds: WRITE_TIME + 400 },
            instance,
        )
    };
    match &rewritten {
        Ok(output) => println!(
            "E name=after_delete op=sequential_write_to_inode_1 result=Ok txg={} object_birth_deleted={} object_birth_written={} recover={}",
            output.root.checkpoint_txg.0,
            deleted.inode_record.object_birth.0,
            output.inode_record.object_birth.0,
            recovery_outcome(&memory_pool_of(&write_devices))
        ),
        Err(error) => println!("E name=after_delete op=sequential_write_to_inode_1 result=Err({error:?})"),
    }
    assert_eq!(
        rewritten.as_ref().ok().map(|output| output.inode_record.object_birth),
        Some(deleted.inode_record.object_birth),
        "删完之后给 inode 1 顺序写照收、对象出生代不变"
    );

    let mut inode_devices = devices_of(&image);
    let mut inode_allocator = allocator.clone();
    let created = {
        let mut pool = PoolWriter::new(&parameters, inode_devices.as_mut_slice());
        singlefs_core::transaction::publish_new_inodes(&mut pool, &mut inode_allocator, &deleted, 1, WRITE_TIME + 400, instance)
    };
    match &created {
        Ok(output) => println!(
            "E name=after_delete op=publish_new_inodes_1 result=Ok watermark={} recover={}",
            output.inode_number_watermark(),
            recovery_outcome(&memory_pool_of(&inode_devices))
        ),
        Err(error) => println!("E name=after_delete op=publish_new_inodes_1 result=Err({error:?})"),
    }
}

/// U1 / C118：删到一半（意图之后留 4）时管理员回退到删之前那一版（txg 4）。回退那次发布取 R_old 的 extent 树与 inode 树、
/// 复活被这次删除释放的两个数据单元，文件整个回来；记账行、树表里别的条目照现行那一版（`mount::version_to_publish_the_rollback_after`）。
#[test]
fn an_administrator_rollback_in_the_middle_of_a_delete_brings_the_whole_file_back() {
    use singlefs_core::address::CheckpointTxg;
    use singlefs_core::mount::{roll_back_by_a_forward_publish, RollbackTarget};
    let (mut devices, mut allocator, six, instance) = pool_with_a_file_of_six_units();
    let payload = u64::try_from(data_unit_payload_capacity()).expect("净荷");
    set_nlink_zero_for_inode_one();
    let intent = batch(&mut devices, &mut allocator, &six, 6, six.inode_record.size, instance).expect("意图");
    let mut current = batch(&mut devices, &mut allocator, &intent, 4, 4 * payload, instance).expect("留 4");
    singlefs_core::records::NLINK_ZERO_INODE_FOR_TESTS.with(|cell| cell.set(None));
    let before = recovery_outcome(&memory_pool_of(&devices));
    let rolled = roll_back_by_a_forward_publish(
        &parameters(),
        &mut devices,
        &mut allocator,
        &mut current,
        RollbackTarget { instance, checkpoint_txg: CheckpointTxg(six.root.checkpoint_txg.0) },
    );
    let image = memory_pool_of(&devices);
    println!(
        "E name=rollback_mid_delete before={before} rollback_ok={} txg_after={} units_after={} size_after={} recover_after={} checker_red={:?} accounting_rows_equal_to_the_deleting_version_except_watermark={}",
        rolled.is_ok(),
        current.root.checkpoint_txg.0,
        current.data_pointers.len(),
        current.inode_record.size,
        recovery_outcome(&image),
        checker_red(&image),
        current
            .accounting_entries
            .iter()
            .filter(|entry| entry.statistic == singlefs_core::records::STATISTIC_PENDING_DELETE_BYTES)
            .map(|entry| entry.value)
            .collect::<Vec<_>>()
            == vec![0]
    );
    assert!(rolled.is_ok());
    assert_eq!(current.data_pointers.len(), FILE_UNITS, "回退之后六个数据单元都回来了");
}

/// 这一版 inode 1 那条记录盘上的 nlink（叶容器单元里第 0 条记录偏移 36 的 4 字节小端）。
fn raw_nlink_of_inode_one(output: &TransactionOutput) -> u32 {
    let leaf = output
        .units
        .iter()
        .find(|unit| matches!(unit.identity, TransactionUnit::InodeLeafContainer(_)))
        .expect("inode 叶容器");
    let at = usize::try_from(singlefs_format::PACKED_UNIT_RECORDS_OFFSET).expect("偏移") + 36;
    u32::from_le_bytes(leaf.bytes[at..at + 4].try_into().expect("4 字节"))
}

/// U3：只读挂载在删除的每一步读到什么——挂载态里有几条 inode 记录、inode 1 在不在、打开它得到几个单元与多大、
/// 盘上那条记录的 nlink 是几（读者的 `InodeRecord` 里没有 nlink，读者分不出「删除中」）。
#[test]
fn what_a_read_only_mount_sees_at_each_step_of_the_delete() {
    use singlefs_core::address::InodeNumber;
    let (mut devices, mut allocator, six, instance) = pool_with_a_file_of_six_units();
    let payload = u64::try_from(data_unit_payload_capacity()).expect("净荷");
    let mut steps: Vec<(String, TransactionOutput, MemoryPool)> =
        vec![("before_unlink".to_owned(), six.clone(), memory_pool_of(&devices))];
    set_nlink_zero_for_inode_one();
    let mut previous = batch(&mut devices, &mut allocator, &six, 6, six.inode_record.size, instance).expect("意图");
    steps.push(("intent".to_owned(), previous.clone(), memory_pool_of(&devices)));
    for keep in [4u64, 2, 0] {
        previous = batch(&mut devices, &mut allocator, &previous, keep, keep * payload, instance).expect("一批");
        steps.push((format!("batch_keep_{keep}"), previous.clone(), memory_pool_of(&devices)));
    }
    for (name, output, image) in &steps {
        let mounted = mount_read_only(image).expect("只读挂载");
        let records = mounted.mounted.inode_records();
        let opened = match mounted.mounted.open_file(image, InodeNumber(1)) {
            Ok(file) => format!("Ok(units={} size={})", file.data_unit_count(), file.inode_record().size),
            Err(failure) => format!("Err({failure:?})"),
        };
        println!(
            "E name=reader step={name} inode_records={} inode_1_present={} raw_nlink_on_disk={} open_file_inode_1={opened}",
            records.len(),
            records.iter().any(|record| record.inode == 1),
            raw_nlink_of_inode_one(output)
        );
        assert!(records.iter().any(|record| record.inode == 1), "读者在每一步都看得到 inode 1 那条记录");
        assert!(opened.starts_with("Ok("), "读者在每一步都打得开 inode 1");
    }
}

/// U1 / C118 的另一个方向：删到留 2（txg 7）时管理员回退到意图那一版（txg 5：六个单元都在、nlink = 0）。
/// 回退那次发布取 R_old 的 inode 树与 extent 树：删除中的 inode（nlink = 0）带着全部 extent 回来；
/// 树表里别的条目、记账行照现行那一版取（`mount.rs` 回退那一段只换 extent 与 inode 两种）。
#[test]
fn an_administrator_rollback_into_a_version_in_the_middle_of_a_delete() {
    use singlefs_core::address::CheckpointTxg;
    use singlefs_core::mount::{roll_back_by_a_forward_publish, RollbackTarget};
    let (mut devices, mut allocator, six, instance) = pool_with_a_file_of_six_units();
    let payload = u64::try_from(data_unit_payload_capacity()).expect("净荷");
    set_nlink_zero_for_inode_one();
    let intent = batch(&mut devices, &mut allocator, &six, 6, six.inode_record.size, instance).expect("意图");
    let four = batch(&mut devices, &mut allocator, &intent, 4, 4 * payload, instance).expect("留 4");
    let mut current = batch(&mut devices, &mut allocator, &four, 2, 2 * payload, instance).expect("留 2");
    singlefs_core::records::NLINK_ZERO_INODE_FOR_TESTS.with(|cell| cell.set(None));
    let nlink_before = raw_nlink_of_inode_one(&current);
    let rolled = roll_back_by_a_forward_publish(
        &parameters(),
        &mut devices,
        &mut allocator,
        &mut current,
        RollbackTarget { instance, checkpoint_txg: CheckpointTxg(intent.root.checkpoint_txg.0) },
    );
    let image = memory_pool_of(&devices);
    println!(
        "E name=rollback_into_mid_delete target_txg={} rollback={} txg_after={} units_after={} size_after={} raw_nlink_before={nlink_before} raw_nlink_after={} inode_leaf_rewritten_by_the_rollback={} recover_after={} checker_red={:?}",
        intent.root.checkpoint_txg.0,
        match &rolled { Ok(_) => "Ok".to_owned(), Err(error) => format!("Err({error:?})") },
        current.root.checkpoint_txg.0,
        current.data_pointers.len(),
        current.inode_record.size,
        raw_nlink_of_inode_one(&current),
        current.rewritten.iter().any(|unit| matches!(unit, TransactionUnit::InodeLeafContainer(_))),
        recovery_outcome(&image),
        checker_red(&image)
    );
}

/// U1 / C118：删到留 4 时回退到删之前那一版（txg 4），之后「意图还在」的续做——意图若住在回退不换的结构里
/// （记账行、树表里 extent 与 inode 之外的条目），回退之后它照现行那一版留着；续做照它再删一批，刚回退回来的文件又被截。
/// 原型没有意图记录，这里只量「回退之后那一批照样做得成、读回又短了」这一半；「意图留着」是按 `mount.rs` 那一段推的。
#[test]
fn a_batch_redone_after_the_rollback_truncates_the_file_the_administrator_brought_back() {
    use singlefs_core::address::CheckpointTxg;
    use singlefs_core::mount::{roll_back_by_a_forward_publish, RollbackTarget};
    let (mut devices, mut allocator, six, instance) = pool_with_a_file_of_six_units();
    let payload = u64::try_from(data_unit_payload_capacity()).expect("净荷");
    set_nlink_zero_for_inode_one();
    let intent = batch(&mut devices, &mut allocator, &six, 6, six.inode_record.size, instance).expect("意图");
    let mut current = batch(&mut devices, &mut allocator, &intent, 4, 4 * payload, instance).expect("留 4");
    singlefs_core::records::NLINK_ZERO_INODE_FOR_TESTS.with(|cell| cell.set(None));
    let rolled = roll_back_by_a_forward_publish(
        &parameters(),
        &mut devices,
        &mut allocator,
        &mut current,
        RollbackTarget { instance, checkpoint_txg: CheckpointTxg(six.root.checkpoint_txg.0) },
    );
    assert!(rolled.is_ok(), "回退");
    let after_rollback = recovery_outcome(&memory_pool_of(&devices));
    let nlink_after_rollback = raw_nlink_of_inode_one(&current);
    set_nlink_zero_for_inode_one();
    let redone = batch(&mut devices, &mut allocator, &current, 4, 4 * payload, instance);
    let image = memory_pool_of(&devices);
    println!(
        "E name=redo_after_rollback recover_after_rollback={after_rollback} raw_nlink_after_rollback={nlink_after_rollback} redo={} recover_after_redo={} checker_red={:?}",
        match &redone { Ok(output) => format!("Ok(txg={} units={})", output.root.checkpoint_txg.0, output.data_pointers.len()), Err(error) => format!("Err({error:?})") },
        recovery_outcome(&image),
        checker_red(&image)
    );
}
