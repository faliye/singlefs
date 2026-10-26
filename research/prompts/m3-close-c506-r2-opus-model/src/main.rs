//! m3-close-c506-r2 攻方原型（只在这一腿的副本上编、跑；被攻过零轮）。四段，各打一类行：
//! - `name=sweep` / `name=sweep_example`：格式允许的每个槽距（4096..=1 MiB − 4096，按 pbs 步进）× S ∈ 4..=16，
//!   入库 mkfs 收还是拒，被判的那一判（S × 槽距 ≤ P × chunk，P × chunk 取第一版常量 3 MiB）过不过，
//!   0 / 1 / 0 归属下每块盘上真落的字节（两个系统配置槽、本盘区域的每个根槽、journal 环）有没有相叠。
//! - `name=formula`：mkfs 写 min(8, ⌊P × chunk ÷ 槽距⌋) 在每个槽距上写几，落在两条边上的槽距段，入库 mkfs 在那个 S 上收不收。
//! - `name=p_chunk` / `name=p_chunk_history`：系统配置里素数步长 P（偏移 363）与 chunk（偏移 367）两个字段改写之后，
//!   core 与池级 checker 收不收、checker 按盘上 P、chunk 算出的区域起点，那一判按常量与按盘上字段各算一次；只暖机可写挂载 24 次。
//! - `name=legit_writer*`：冻结副本打上补丁（设 C506_R2_CLAUSE_ONLY_WRITER 时 mkfs 不判两两不重叠，只判盘长）当「只照条款判盘长的写者」，
//!   S = 16、槽距 512 KiB 直接写出来，与第一轮的改字节盘逐扇区比、挂载历史逐行比；另在用过的盘上比一次。
//! 用户动作（挂几次）放开扫到 24 次；历史不含崩溃点，不调崩溃枚举。
use singlefs_checker::check_system_configuration_slot;
use singlefs_checker::image::{geometry_of, InvariantVerdict};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{choose_root, choose_system_configuration};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot, RootRingSlotsPerRegion};
use singlefs_core::system_configuration::{SystemImmutableSizes, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET};
use singlefs_format::{
    JOURNAL_RING_DEFAULT_BYTES, JOURNAL_RING_START_SLOT, ROOT_RING_BASE_SLOT, ROOT_RING_CHUNK_BYTES,
    ROOT_RING_PRIME_STEP, SLOT_BYTES,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

const IMAGE_BYTES: u64 = 4 << 30;
const SLOTS_PER_REGION_OFFSET: usize = 362;
const PRIME_STEP_OFFSET: usize = 363;
const CHUNK_OFFSET: usize = 367;
const SYSTEM_CONFIGURATION_SLOT_BYTES: usize = 4096;
const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"c506-r2-opus-prb";
/// 区域间距 P × chunk 取第一版常量（core 的 `region_start` 用的就是这两个常量）。
const REGION_GAP_FIRST_VERSION: u64 = ROOT_RING_PRIME_STEP * ROOT_RING_CHUNK_BYTES;
const BASE_BYTES: u64 = ROOT_RING_BASE_SLOT * SLOT_BYTES;
const JOURNAL_START_BYTES: u64 = JOURNAL_RING_START_SLOT * SLOT_BYTES;
const LARGEST_SPACING: u32 = 1_044_480;
const REGION_DEVICES: [u32; 3] = [0, 1, 0];
const CLAUSE_ONLY_WRITER: &str = "C506_R2_CLAUSE_ONLY_WRITER";

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

fn parameters_with(physical_block_size: u32, io_min: u32, spacing: u32, slots_per_region: u64) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size,
            minimum_input_output_bytes: io_min,
            fixed_structure_slot_spacing: spacing,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(slots_per_region)
                .expect("这一原型给的 S 都在 4..16 里"),
        },
    }
}

/// io_min 照 D2 已定项 19 算槽距（`slot_spacing_for`）的那一种参数，pbs 512。
fn parameters(slots_per_region: u64, io_min: u32) -> MakeFilesystemParameters {
    parameters_with(512, io_min, SystemImmutableSizes::slot_spacing_for(io_min), slots_per_region)
}

fn empty_devices(physical_block_size: u32) -> Devices {
    (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(physical_block_size)),
            )
        })
        .collect()
}

fn member_name(debug: &str) -> String {
    debug
        .split(|character: char| character == ' ' || character == '{' || character == '(')
        .next()
        .unwrap_or(debug)
        .to_string()
}

fn mkfs_verdict(parameters: &MakeFilesystemParameters) -> String {
    let mut devices = empty_devices(parameters.geometry.physical_block_size);
    match make_filesystem(parameters, &mut devices) {
        Ok(_) => "accepted".to_string(),
        Err(error) => format!("refused:{}", member_name(&format!("{error:?}"))),
    }
}

/// 0 / 1 / 0 归属下每块盘上真落字节的几段：两个系统配置槽、归这块盘的区域的每个根槽（宽 pbs）、journal 环。
/// 交回第一对相叠的（按设备、按排布次序），没有就 None。
fn first_overlap_of_bytes_actually_used(physical_block_size: u32, spacing: u32, slots_per_region: u64) -> Option<String> {
    let spacing = u64::from(spacing);
    for device in 0..2u32 {
        let mut extents: Vec<(String, u64, u64)> = vec![
            ("sc0".to_string(), 0, SYSTEM_CONFIGURATION_SLOT_BYTES as u64),
            ("sc1".to_string(), spacing, SYSTEM_CONFIGURATION_SLOT_BYTES as u64),
        ];
        for (region, owner) in REGION_DEVICES.iter().enumerate() {
            if *owner != device {
                continue;
            }
            let region = region as u64;
            for slot in 0..slots_per_region {
                let start = BASE_BYTES + region * REGION_GAP_FIRST_VERSION + slot * spacing;
                extents.push((format!("r{region}s{slot}"), start, u64::from(physical_block_size)));
            }
        }
        extents.push(("journal".to_string(), JOURNAL_START_BYTES, JOURNAL_RING_DEFAULT_BYTES));
        for (index, (first, first_start, first_length)) in extents.iter().enumerate() {
            for (second, second_start, second_length) in &extents[index + 1..] {
                if *first_start < second_start + second_length && *second_start < first_start + first_length {
                    return Some(format!("d{device}:{first}/{second}"));
                }
            }
        }
    }
    None
}

/// 被判的那一判，P × chunk 取第一版常量。
fn judgment_with_first_version_gap(spacing: u32, slots_per_region: u64) -> bool {
    slots_per_region * u64::from(spacing) <= REGION_GAP_FIRST_VERSION
}

/// mkfs 照被判形态写的 S：min(8, ⌊P × chunk ÷ 槽距⌋)。
fn formula_slots_per_region(spacing: u32) -> u64 {
    8u64.min(REGION_GAP_FIRST_VERSION / u64::from(spacing))
}

fn sweep(sizes: &[u32]) {
    for &physical_block_size in sizes {
        let io_min = physical_block_size;
        let (mut geometries, mut spacings, mut mkfs_accepted, mut judgment_passes) = (0u64, 0u64, 0u64, 0u64);
        let (mut accepted_but_refused_by_judgment, mut judgment_passes_but_bytes_overlap) = (0u64, 0u64);
        let (mut judgment_passes_region2_past_journal, mut judgment_refuses_without_bytes_overlap) = (0u64, 0u64);
        let mut judgment_passes_but_mkfs_refuses: std::collections::BTreeMap<String, u64> = Default::default();
        let mut mkfs_by_geometry: std::collections::BTreeMap<(u32, u64), bool> = Default::default();
        let mut examples: Vec<String> = Vec::new();
        let mut spacing = 4096u32;
        while spacing <= LARGEST_SPACING {
            spacings += 1;
            for slots_per_region in 4..=16u64 {
                geometries += 1;
                let verdict = mkfs_verdict(&parameters_with(physical_block_size, io_min, spacing, slots_per_region));
                let accepted = verdict == "accepted";
                mkfs_by_geometry.insert((spacing, slots_per_region), accepted);
                let judgment = judgment_with_first_version_gap(spacing, slots_per_region);
                let overlap = first_overlap_of_bytes_actually_used(physical_block_size, spacing, slots_per_region);
                let region2_end = BASE_BYTES + 2 * REGION_GAP_FIRST_VERSION + slots_per_region * u64::from(spacing);
                mkfs_accepted += u64::from(accepted);
                judgment_passes += u64::from(judgment);
                if accepted && !judgment {
                    accepted_but_refused_by_judgment += 1;
                    examples.push(format!("mkfs_accepted_judgment_refuses:S={slots_per_region},spacing={spacing}"));
                }
                if judgment && overlap.is_some() {
                    judgment_passes_but_bytes_overlap += 1;
                    examples.push(format!("judgment_passes_bytes_overlap:S={slots_per_region},spacing={spacing},{}", overlap.clone().unwrap_or_default()));
                }
                if judgment && region2_end > JOURNAL_START_BYTES {
                    judgment_passes_region2_past_journal += 1;
                }
                if !judgment && overlap.is_none() {
                    judgment_refuses_without_bytes_overlap += 1;
                }
                if judgment && !accepted {
                    *judgment_passes_but_mkfs_refuses.entry(verdict).or_default() += 1;
                }
            }
            spacing += physical_block_size;
        }
        println!(
            "name=sweep pbs={physical_block_size} io_min={io_min} spacings={spacings} geometries={geometries} mkfs_accepted={mkfs_accepted} \
judgment_passes={judgment_passes} mkfs_accepted_and_judgment_refuses={accepted_but_refused_by_judgment} \
judgment_passes_and_bytes_overlap_on_a_device={judgment_passes_but_bytes_overlap} judgment_passes_and_region2_end_past_journal_16MiB={judgment_passes_region2_past_journal} \
judgment_refuses_and_no_bytes_overlap={judgment_refuses_without_bytes_overlap} judgment_passes_and_mkfs_refuses={judgment_passes_but_mkfs_refuses:?}"
        );
        for example in examples.iter().take(6) {
            println!("name=sweep_example pbs={physical_block_size} {example}");
        }
        formula(physical_block_size, &mkfs_by_geometry);
    }
}

/// mkfs 写 min(8, ⌊3 MiB ÷ 槽距⌋) 在每个槽距上写几：按写出的 S 分段，报每段的槽距区间、个数，与入库 mkfs 在那个 S 上收不收。
fn formula(physical_block_size: u32, mkfs_by_geometry: &std::collections::BTreeMap<(u32, u64), bool>) {
    let mut by_value: std::collections::BTreeMap<u64, (u32, u32, u64, u64)> = Default::default();
    let mut spacing = 4096u32;
    while spacing <= LARGEST_SPACING {
        let written = formula_slots_per_region(spacing);
        let entry = by_value.entry(written).or_insert((spacing, spacing, 0, 0));
        entry.1 = spacing;
        entry.2 += 1;
        if mkfs_by_geometry.get(&(spacing, written)).copied().unwrap_or(false) {
            entry.3 += 1;
        }
        spacing += physical_block_size;
    }
    for (written, (low, high, count, accepted)) in by_value {
        let edge = if written == 4 || written == 16 { "on_an_edge" } else if written < 4 { "below_lower_edge_refuse_mkfs" } else { "inside" };
        println!(
            "name=formula pbs={physical_block_size} written_S={written} {edge} spacing_from={low} spacing_to={high} spacings={count} mkfs_accepts_that_S={accepted}"
        );
    }
    if physical_block_size == 512 {
        let powers: Vec<String> = (12..=19u32)
            .map(|exponent| {
                let io_min = 1u32 << exponent;
                let spacing = SystemImmutableSizes::slot_spacing_for(io_min);
                let written = formula_slots_per_region(spacing);
                format!("io_min={io_min}:spacing={spacing}:S={written}:mkfs={}", mkfs_verdict(&parameters(written, io_min)))
            })
            .collect();
        println!("name=formula_spacing_from_io_min_power_of_two {}", powers.join(" "));
        let odd: Vec<String> = [3072u32, 6144, 196_608, 393_216, 655_360, 786_432]
            .iter()
            .map(|io_min| {
                let spacing = SystemImmutableSizes::slot_spacing_for(*io_min);
                let written = formula_slots_per_region(spacing);
                let verdict = if written >= 4 { mkfs_verdict(&parameters(written, *io_min)) } else { "formula_refuses".to_string() };
                format!("io_min={io_min}:spacing={spacing}:S={written}:mkfs={verdict}")
            })
            .collect();
        println!("name=formula_spacing_from_io_min_not_power_of_two {}", odd.join(" "));
    }
}

/// 两块盘四个系统配置槽各改几个字节、整槽校验和按改完的字节重封（第一轮 `declare_slots_per_region` 的同一个改法，改的字段由 `edit` 给）。
fn rewrite_system_configuration(devices: &mut Devices, spacing: u64, edit: &dyn Fn(&mut [u8])) {
    for (_, device) in devices.iter_mut() {
        for offset in [0, spacing] {
            let mut slot = vec![0u8; SYSTEM_CONFIGURATION_SLOT_BYTES];
            device.read_at(DeviceOffsetInBytes(offset), &mut slot).expect("系统配置槽在盘内");
            edit(&mut slot);
            let digest = wide_checksum_with_field_zeroed(&slot, SYSTEM_CONFIGURATION_SLOT_BYTES, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET);
            slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
            device.write_at(DeviceOffsetInBytes(offset), &slot, WriteDurability::Plain).expect("改完的槽写得回去");
        }
    }
}

fn nonzero_sectors(devices: &Devices) -> Vec<(u32, u64, Vec<u8>)> {
    let mut sectors = Vec::new();
    for (identity, device) in devices {
        for (sector, bytes) in device.image.written_sectors() {
            if bytes.iter().any(|byte| *byte != 0) {
                sectors.push((identity.0, *sector, bytes.clone()));
            }
        }
    }
    sectors
}

fn differing_sectors(left: &Devices, right: &Devices) -> (usize, usize, usize, Vec<String>) {
    let left = nonzero_sectors(left);
    let right = nonzero_sectors(right);
    let only_left: Vec<&(u32, u64, Vec<u8>)> = left.iter().filter(|sector| !right.contains(sector)).collect();
    let only_right: Vec<&(u32, u64, Vec<u8>)> = right.iter().filter(|sector| !left.contains(sector)).collect();
    let mut where_: Vec<String> = only_left.iter().map(|(device, sector, _)| format!("left:d{device}@{}", sector * 512)).collect();
    where_.extend(only_right.iter().map(|(device, sector, _)| format!("right:d{device}@{}", sector * 512)));
    where_.truncate(8);
    (left.len(), right.len(), only_left.len() + only_right.len(), where_)
}

fn memory_pool(devices: &Devices) -> MemoryPool {
    MemoryPool {
        devices: devices.iter().map(|(identity, device)| (*identity, device.image.clone())).collect(),
        device_size_in_bytes: IMAGE_BYTES,
    }
}

fn red_invariants(devices: &Devices) -> Vec<String> {
    check_pool_image(&memory_pool(devices))
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some(format!("{invariant}<{}>", detail.chars().take(70).collect::<String>())),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

/// 根环每一槽按 core 的几何（常量 P、chunk）读一次：自证过、而 txg 按落点公式不该住这一槽的根（第 0 代根照 mkfs 种的槽 0 另认）。
fn misplaced_roots(devices: &Devices, slots_per_region: u64, spacing: u32) -> Vec<String> {
    let s = RootRingSlotsPerRegion::from_system_configuration_field(slots_per_region).expect("S");
    let mut misplaced = Vec::new();
    for region in 0..3u64 {
        for slot in 0..slots_per_region {
            let ring_slot = RootRingSlot { region, slot };
            let device = &devices[REGION_DEVICES[region as usize] as usize].1;
            let mut bytes = vec![0u8; 512];
            device.read_at(slot_offset(ring_slot, spacing), &mut bytes).expect("根槽在盘内");
            if let Some(root) = RootRecord::parse_slot(&bytes, &FILESYSTEM_IDENTIFIER) {
                let fits = if root.checkpoint_txg.0 == 0 { slot == 0 } else { target_for_publish(root.checkpoint_txg, s) == ring_slot };
                if !fits {
                    misplaced.push(format!("r{region}s{slot}:txg{}", root.checkpoint_txg.0));
                }
            }
        }
    }
    misplaced
}

fn chosen_root_txg(devices: &Devices) -> String {
    choose_system_configuration(devices)
        .ok()
        .and_then(|chosen| choose_root(devices, &chosen))
        .map_or_else(|| "none".to_string(), |root| root.checkpoint_txg.0.to_string())
}

/// 同一份池接连只暖机可写挂载 `mounts` 次；挂载 0 是一次都没挂的那一刻。逐次打一行，交回这些行（不带臂名，给两条臂逐行比）。
fn history(arm: &str, mut devices: Devices, parameters: &MakeFilesystemParameters, mounts: u64, with_misplaced: bool) -> Vec<String> {
    let spacing = parameters.geometry.fixed_structure_slot_spacing;
    let slots_per_region = parameters.geometry.root_ring_slots_per_region.count();
    let mut lines = Vec::new();
    for mount in 0..=mounts {
        let result = if mount == 0 {
            "not_mounted".to_string()
        } else {
            match mount_writable(parameters, &mut devices) {
                Ok(_) => "ok".to_string(),
                Err(error) => format!("refused:{}", member_name(&format!("{error:?}"))),
            }
        };
        let misplaced = if with_misplaced { format!(" misplaced_roots={:?}", misplaced_roots(&devices, slots_per_region, spacing)) } else { String::new() };
        let line = format!(
            "mount={mount} mount_result={result} chosen_root_txg={}{misplaced} red={:?}",
            chosen_root_txg(&devices),
            red_invariants(&devices)
        );
        println!("name=history arm={arm} {line}");
        lines.push(line);
    }
    lines
}

/// 同一份池拷一份（`SparseBlockDevice` 不实现 `Clone`，它的 `image` 实现）：盘长与 pbs 取这一原型建盘时的那两个。
fn copy_devices(devices: &Devices) -> Devices {
    devices
        .iter()
        .map(|(identity, device)| {
            let mut copy = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
            copy.image = device.image.clone();
            (*identity, copy)
        })
        .collect()
}

fn formatted(parameters: &MakeFilesystemParameters) -> Devices {
    let mut devices = empty_devices(parameters.geometry.physical_block_size);
    make_filesystem(parameters, &mut devices).expect("mkfs 收这一格");
    devices
}

/// 系统配置里 P、chunk 两个字段改写之后，两个读者各怎么判、checker 把区域放在哪；被判的那一判按常量与按盘上字段各算一次。
fn p_chunk_arm(arm: &str, base: &MakeFilesystemParameters, mounts_before_rewrite: u64, declared_slots_per_region: u8, prime_step: u32, chunk_bytes: u32) {
    let spacing = base.geometry.fixed_structure_slot_spacing;
    let mut devices = formatted(base);
    for _ in 0..mounts_before_rewrite {
        mount_writable(base, &mut devices).expect("改字段之前的暖机挂得上");
    }
    rewrite_system_configuration(&mut devices, u64::from(spacing), &|slot: &mut [u8]| {
        slot[SLOTS_PER_REGION_OFFSET] = declared_slots_per_region;
        slot[PRIME_STEP_OFFSET..PRIME_STEP_OFFSET + 4].copy_from_slice(&prime_step.to_le_bytes());
        slot[CHUNK_OFFSET..CHUNK_OFFSET + 4].copy_from_slice(&chunk_bytes.to_le_bytes());
    });
    let core = match choose_system_configuration(&devices) {
        Ok(chosen) => format!("accepted(S={})", chosen.immutable.sizes.root_ring_slots_per_region.count()),
        Err(failure) => format!("refused:{}", member_name(&format!("{failure:?}"))),
    };
    let mut slot = vec![0u8; SYSTEM_CONFIGURATION_SLOT_BYTES];
    devices[0].1.read_at(DeviceOffsetInBytes(0), &mut slot).expect("槽 0");
    let view = check_system_configuration_slot(&slot).expect("改完的槽自证得过");
    let (checker, checker_region_starts) = match geometry_of(&slot, &view) {
        Ok(geometry) => (
            format!("accepted(S={},P={},chunk={})", geometry.slots_per_region, geometry.prime_step, geometry.chunk_bytes),
            (0..3u64).map(|region| geometry.base_slot * SLOT_BYTES + region * geometry.prime_step * geometry.chunk_bytes).collect::<Vec<u64>>(),
        ),
        Err(verdict) => (format!("refused:{verdict:?}"), Vec::new()),
    };
    let core_region_starts: Vec<u64> = (0..3u64).map(|region| BASE_BYTES + region * REGION_GAP_FIRST_VERSION).collect();
    let region_bytes = u64::from(declared_slots_per_region) * u64::from(spacing);
    let on_disk_gap = u64::from(prime_step) * u64::from(chunk_bytes);
    let checker_region2_inside_journal = checker_region_starts.get(2).is_some_and(|start| *start >= JOURNAL_START_BYTES);
    println!(
        "name=p_chunk arm={arm} S={declared_slots_per_region} spacing={spacing} P={prime_step} chunk={chunk_bytes} region_bytes={region_bytes} \
core_choose_system_configuration={core} checker_geometry_of={checker} core_region_starts={core_region_starts:?} checker_region_starts={checker_region_starts:?} \
checker_region2_start_inside_journal_ring={checker_region2_inside_journal} judgment_with_first_version_gap={} judgment_with_on_disk_gap={} p_and_chunk_equal_first_version={}",
        region_bytes <= REGION_GAP_FIRST_VERSION,
        region_bytes <= on_disk_gap,
        u64::from(prime_step) == ROOT_RING_PRIME_STEP && u64::from(chunk_bytes) == ROOT_RING_CHUNK_BYTES
    );
    let parameters = parameters_with(
        base.geometry.physical_block_size,
        base.geometry.minimum_input_output_bytes,
        spacing,
        u64::from(declared_slots_per_region),
    );
    let mut devices = devices;
    for mount in 0..=4u64 {
        if mount > 0 {
            let result = mount_writable(&parameters, &mut devices).map_or_else(|error| format!("refused:{}", member_name(&format!("{error:?}"))), |_| "ok".to_string());
            print!("name=p_chunk_history arm={arm} mount_after_rewrite={mount} mount_result={result} ");
        } else {
            print!("name=p_chunk_history arm={arm} mount_after_rewrite=0 mount_result=not_mounted ");
        }
        let (newest_prime, newest_chunk) = newest_slot_prime_and_chunk(&devices[0].1);
        println!(
            "disk0_newest_slot_P={newest_prime} disk0_newest_slot_chunk={newest_chunk} judgment_with_on_disk_gap={} core_chosen_root_txg={} red={:?}",
            region_bytes <= u64::from(newest_prime) * u64::from(newest_chunk),
            chosen_root_txg(&devices),
            red_invariants(&devices)
        );
    }
}

/// 盘 0 两个系统配置槽里世代号（偏移 147）大的那一槽自述的 P 与 chunk。
fn newest_slot_prime_and_chunk(device: &SparseBlockDevice) -> (u32, u32) {
    let mut best: Option<(u64, u32, u32)> = None;
    let mut slot_one_spacing = 0u64;
    for slot_index in 0..2u64 {
        let mut slot = vec![0u8; SYSTEM_CONFIGURATION_SLOT_BYTES];
        let offset = if slot_index == 0 { 0 } else { slot_one_spacing };
        device.read_at(DeviceOffsetInBytes(offset), &mut slot).expect("系统配置槽在盘内");
        if slot_index == 0 {
            slot_one_spacing = u64::from(u32::from_le_bytes(slot[429..433].try_into().expect("4")));
        }
        if check_system_configuration_slot(&slot).is_err() {
            continue;
        }
        let generation = u64::from_le_bytes(slot[147..155].try_into().expect("8"));
        let prime = u32::from_le_bytes(slot[PRIME_STEP_OFFSET..PRIME_STEP_OFFSET + 4].try_into().expect("4"));
        let chunk = u32::from_le_bytes(slot[CHUNK_OFFSET..CHUNK_OFFSET + 4].try_into().expect("4"));
        if best.is_none_or(|(known, _, _)| generation > known) {
            best = Some((generation, prime, chunk));
        }
    }
    best.map_or((0, 0), |(_, prime, chunk)| (prime, chunk))
}

fn p_chunk() {
    let first_version_prime = u32::try_from(ROOT_RING_PRIME_STEP).expect("3");
    let first_version_chunk = u32::try_from(ROOT_RING_CHUNK_BYTES).expect("1 MiB");
    p_chunk_arm("control_S8_4KiB_P3_chunk1MiB_after_24_mounts", &parameters(8, 512), 24, 8, first_version_prime, first_version_chunk);
    p_chunk_arm("S8_4KiB_P0_after_24_mounts", &parameters(8, 512), 24, 8, 0, first_version_chunk);
    p_chunk_arm("S8_4KiB_chunk4MiB_after_24_mounts", &parameters(8, 512), 24, 8, first_version_prime, 4 << 20);
    p_chunk_arm("S8_4KiB_P5_after_24_mounts", &parameters(8, 512), 24, 8, 5, first_version_chunk);
    p_chunk_arm("S16_512KiB_chunk4MiB_from_S4_fresh", &parameters(4, 524_288), 0, 16, first_version_prime, 4 << 20);
}

/// 只照条款判盘长的写者：设 C506_R2_CLAUSE_ONLY_WRITER 之后调入库 mkfs（补丁让它跳过两两不重叠那一判），调完就撤。
fn legit_writer_mkfs(parameters: &MakeFilesystemParameters, devices: &mut Devices) -> String {
    std::env::set_var(CLAUSE_ONLY_WRITER, "1");
    let result = make_filesystem(parameters, devices);
    std::env::remove_var(CLAUSE_ONLY_WRITER);
    match result {
        Ok(_) => "accepted".to_string(),
        Err(error) => format!("refused:{}", member_name(&format!("{error:?}"))),
    }
}

fn rewritten_from(from: &MakeFilesystemParameters, devices: &mut Devices, to: u8) {
    make_filesystem(from, devices).expect("mkfs 收改字节的起点那一格");
    let spacing = u64::from(from.geometry.fixed_structure_slot_spacing);
    rewrite_system_configuration(devices, spacing, &|slot: &mut [u8]| slot[SLOTS_PER_REGION_OFFSET] = to);
}

fn legit_writer() {
    let sixteen = parameters(16, 524_288);
    println!("name=legit_writer_patch_off mkfs_S16_512KiB={}", mkfs_verdict(&sixteen));
    let mut legit = empty_devices(512);
    println!("name=legit_writer_patch_on mkfs_S16_512KiB={}", legit_writer_mkfs(&sixteen, &mut legit));
    let mut rewritten = empty_devices(512);
    rewritten_from(&parameters(4, 524_288), &mut rewritten, 16);
    let (left, right, differing, where_) = differing_sectors(&rewritten, &legit);
    println!("name=legit_writer_fresh_sectors nonzero_rewritten={left} nonzero_legit={right} differing_sectors={differing} where={where_:?}");
    let legit_lines = history("legit_writer_S16_512KiB_fresh", legit, &sixteen, 24, true);
    let rewritten_lines = history("rewritten_S4_to_S16_512KiB_fresh", rewritten, &sixteen, 24, true);
    let same = legit_lines.iter().zip(&rewritten_lines).filter(|(left, right)| left == right).count();
    let red_i717 = legit_lines.iter().filter(|line| line.contains("red=[\"I-7.17")).count();
    println!("name=legit_writer_fresh_history lines={} identical_lines={same} legit_states_with_i717_red_first={red_i717}", legit_lines.len());

    // 用过的盘：同一 fsid 先做一个 S = 6、512 KiB 的池（mkfs 收：6 × 512 KiB = 3 MiB），挂 6 次；再在这两块盘上各做一次。
    let six = parameters(6, 524_288);
    let mut used = formatted(&six);
    for _ in 0..6 {
        mount_writable(&six, &mut used).expect("S = 6 的旧池挂得上");
    }
    println!("name=used_disk_old_pool S=6 spacing=524288 chosen_root_txg={} roots_beyond_slot3_of_region0={:?}", chosen_root_txg(&used), misplaced_roots(&used, 16, 524_288));
    let mut legit_used = copy_devices(&used);
    let legit_verdict = legit_writer_mkfs(&sixteen, &mut legit_used);
    let mut rewritten_used = copy_devices(&used);
    rewritten_from(&parameters(4, 524_288), &mut rewritten_used, 16);
    let (left, right, differing, where_) = differing_sectors(&rewritten_used, &legit_used);
    println!("name=legit_writer_used_sectors legit_mkfs={legit_verdict} nonzero_rewritten={left} nonzero_legit={right} differing_sectors={differing} where={where_:?}");
    history("legit_writer_S16_512KiB_used", legit_used, &sixteen, 2, true);
    history("rewritten_S4_to_S16_512KiB_used", rewritten_used, &sixteen, 2, true);
}

fn main() {
    let started = std::time::Instant::now();
    let section = std::env::args().nth(1).unwrap_or_else(|| "all".to_string());
    if section == "sweep_4096" {
        sweep(&[4096]);
    }
    if section == "all" || section == "sweep" {
        sweep(&[512, 4096]);
        eprintln!("sweep done at {:?}", started.elapsed());
    }
    if section == "all" || section == "p_chunk" {
        p_chunk();
        eprintln!("p_chunk done at {:?}", started.elapsed());
    }
    if section == "all" || section == "legit" {
        legit_writer();
        eprintln!("legit done at {:?}", started.elapsed());
    }
}
