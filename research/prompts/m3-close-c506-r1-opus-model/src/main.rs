//! m3-close-c506-r1 攻方原型（只在这一腿的副本上编、跑；被攻过零轮）。三段，各打一类行：
//! - `name=mkfs_geometry`：入库 mkfs（`make_filesystem`）在 S × io_min 每一格上收还是拒，拒的报哪一个成员；
//!   连同纯算术：区域长 S × 槽距、同一块盘上区域 0 与区域 2 相距 6 MiB、区域 2 末端对 journal 环起点 16 MiB。
//! - `name=same_bytes` / `name=reader_accepts`：mkfs 收的一份池把系统配置里 S 那一字节改写、整槽校验和重封，
//!   先证这一改在空盘上与「一个直接写那个 S 的 mkfs」逐扇区相同（在 mkfs 两边都收的几何上比），
//!   再看 core 择系统配置、checker `geometry_of`、可写挂载收不收 S = 16、槽距 512 KiB 那一份。
//! - `name=history`：在三份池上接连只暖机可写挂载，逐次打出最新根、根环里自证过而不住落点公式那一槽的根、
//!   池级 checker 判红的条目。用户动作（挂几次）放开扫到 24 次；不含崩溃点，不调崩溃枚举。
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
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

const IMAGE_BYTES: u64 = 4 << 30;
const PHYSICAL_BLOCK_SIZE: u32 = 512;
/// 系统配置字段表里「每区槽数 S」那一字节的偏移（`.claude/kb/layout/01-first-txn.md` 一，偏移按行累加那一段）。
const SLOTS_PER_REGION_OFFSET: usize = 362;
const SYSTEM_CONFIGURATION_SLOT_BYTES: usize = 4096;
const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"c506-opus-probe!";
const MEBIBYTE: u64 = 1 << 20;

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

fn parameters(slots_per_region: u64, io_min: u32) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: PHYSICAL_BLOCK_SIZE,
            minimum_input_output_bytes: io_min,
            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(io_min),
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(
                slots_per_region,
            )
            .expect("这一原型给的 S 都在 4..16 里"),
        },
    }
}

fn empty_devices() -> Devices {
    (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_SIZE)),
            )
        })
        .collect()
}

/// `Debug` 打出来的错误只留成员名（第一个空格、花括号或圆括号之前），一行装得下。
fn member_name(debug: &str) -> String {
    debug
        .split(|character: char| character == ' ' || character == '{' || character == '(')
        .next()
        .unwrap_or(debug)
        .to_string()
}

/// mkfs 拒绝重叠时点名的那一对（`FixedStructuresOverlap` 带的两段），其余成员只报名字。
fn mkfs_verdict(parameters: &MakeFilesystemParameters) -> String {
    let mut devices = empty_devices();
    match make_filesystem(parameters, &mut devices) {
        Ok(_) => "accepted".to_string(),
        Err(error) => {
            let debug = format!("{error:?}");
            let name = member_name(&debug);
            if name == "FixedStructuresOverlap" {
                let first = debug.split("structure: ").nth(1).map(member_name).unwrap_or_default();
                let second = debug.split("structure: ").nth(2).map(member_name).unwrap_or_default();
                let first_region = debug.split("region: ").nth(1).map(|rest| member_name(rest).trim_end_matches(',').to_string()).unwrap_or_default();
                let second_region = debug.split("region: ").nth(2).map(|rest| member_name(rest).trim_end_matches(',').to_string()).unwrap_or_default();
                format!("refused:{name}[{first}{first_region}/{second}{second_region}]")
            } else {
                format!("refused:{name}")
            }
        }
    }
}

fn mkfs_geometry_table() {
    for io_min in [4096u32, 65_536, 131_072, 262_144, 524_288] {
        for slots_per_region in [4u64, 5, 6, 8, 12, 16] {
            let parameters = parameters(slots_per_region, io_min);
            let spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
            let region_bytes = slots_per_region * spacing;
            println!(
                "name=mkfs_geometry S={slots_per_region} io_min={io_min} spacing={spacing} region_bytes={region_bytes} \
region_bytes_over_region_gap_3MiB={} region0_meets_region2_on_device0={} region2_end_past_journal_start_16MiB={} mkfs={}",
                region_bytes > 3 * MEBIBYTE,
                region_bytes > 6 * MEBIBYTE,
                7 * MEBIBYTE + region_bytes > 16 * MEBIBYTE,
                mkfs_verdict(&parameters)
            );
        }
    }
}

/// 两块盘四个系统配置槽里 S 那一字节改成 `declared`，整槽校验和按改完的字节重封（与
/// `crates/singlefs-harness/tests/system_configuration_slots_per_region.rs` 的 `declare_slots_per_region_on_disk` 同一个改法）。
fn declare_slots_per_region(devices: &mut Devices, spacing: u64, declared: u8) {
    for (_, device) in devices.iter_mut() {
        for offset in [0, spacing] {
            let mut slot = vec![0u8; SYSTEM_CONFIGURATION_SLOT_BYTES];
            device.read_at(DeviceOffsetInBytes(offset), &mut slot).expect("系统配置槽在盘内");
            slot[SLOTS_PER_REGION_OFFSET] = declared;
            let digest = wide_checksum_with_field_zeroed(
                &slot,
                SYSTEM_CONFIGURATION_SLOT_BYTES,
                SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
            );
            slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
                .copy_from_slice(&digest);
            device
                .write_at(DeviceOffsetInBytes(offset), &slot, WriteDurability::Plain)
                .expect("改完的槽写得回去");
        }
    }
}

/// 两块盘上非全零的扇区：写零与没写过在盘上是同一个字节，比的是盘上内容，不比写过哪些扇区。
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

fn formatted(slots_per_region: u64, io_min: u32) -> Devices {
    let mut devices = empty_devices();
    make_filesystem(&parameters(slots_per_region, io_min), &mut devices).expect("mkfs 收这一格");
    devices
}

/// 改字节造出来的池与直接 mkfs 的池在空盘上逐扇区比：两边都收的几何上才比得了。
fn same_bytes(from: u64, to: u64, io_min: u32) {
    let spacing = u64::from(SystemImmutableSizes::slot_spacing_for(io_min));
    let mut rewritten = formatted(from, io_min);
    declare_slots_per_region(&mut rewritten, spacing, u8::try_from(to).expect("S 装得进一字节"));
    let direct = formatted(to, io_min);
    let left = nonzero_sectors(&rewritten);
    let right = nonzero_sectors(&direct);
    let differing = left.iter().filter(|sector| !right.contains(sector)).count()
        + right.iter().filter(|sector| !left.contains(sector)).count();
    println!(
        "name=same_bytes rewritten_from_S={from} to_S={to} io_min={io_min} nonzero_sectors_rewritten={} nonzero_sectors_direct={} differing_sectors={differing}",
        left.len(),
        right.len()
    );
}

fn memory_pool(devices: &Devices) -> MemoryPool {
    MemoryPool {
        devices: devices.iter().map(|(identity, device)| (*identity, device.image.clone())).collect(),
        device_size_in_bytes: IMAGE_BYTES,
    }
}

/// S = 4、槽距 512 KiB 的池（mkfs 收）改成自述 S = 16：core、checker、可写挂载各判一次。交回改完的盘。
fn sixteen_slots_at_512_kibibytes() -> Devices {
    let io_min = 524_288u32;
    let spacing = u64::from(SystemImmutableSizes::slot_spacing_for(io_min));
    let mut devices = formatted(4, io_min);
    declare_slots_per_region(&mut devices, spacing, 16);
    let core = match choose_system_configuration(&devices) {
        Ok(chosen) => format!(
            "accepted(S={},spacing={})",
            chosen.immutable.sizes.root_ring_slots_per_region.count(),
            chosen.immutable.sizes.fixed_structure_slot_spacing
        ),
        Err(failure) => format!("refused:{}", member_name(&format!("{failure:?}"))),
    };
    let mut slot = vec![0u8; SYSTEM_CONFIGURATION_SLOT_BYTES];
    devices[0].1.read_at(DeviceOffsetInBytes(0), &mut slot).expect("槽 0");
    let view = check_system_configuration_slot(&slot).expect("改完的槽自证得过");
    let checker = match geometry_of(&slot, &view) {
        Ok(geometry) => format!("accepted(S={},spacing={})", geometry.slots_per_region, geometry.slot_spacing),
        Err(verdict) => format!("refused:{verdict:?}"),
    };
    println!("name=reader_accepts S=16 io_min={io_min} spacing={spacing} core_choose_system_configuration={core} checker_geometry_of={checker}");
    devices
}

const REGION_DEVICES: [u32; 3] = [0, 1, 0];

/// 同一块盘、同一段字节上住着的两个 (区域, 槽)：落点公式当它们是两个槽，盘上是一个。
fn aliased_slot_pairs(slots_per_region: u64, spacing: u32) -> Vec<String> {
    let mut seen: Vec<(u32, u64, RootRingSlot)> = Vec::new();
    let mut pairs = Vec::new();
    for region in 0..3u64 {
        for slot in 0..slots_per_region {
            let ring_slot = RootRingSlot { region, slot };
            let device = REGION_DEVICES[usize::try_from(region).expect("区域号")];
            let offset = slot_offset(ring_slot, spacing).0;
            for (other_device, other_offset, other) in &seen {
                if *other_device == device && *other_offset == offset {
                    pairs.push(format!("d{device}@{offset}:r{}s{}=r{}s{}", other.region, other.slot, region, slot));
                }
            }
            seen.push((device, offset, ring_slot));
        }
    }
    pairs
}

/// 根环每一槽读一次：自证过、而 txg 按落点公式不该住这一槽的根（第 0 代根照 mkfs 种的槽 0 另认）。
fn misplaced_roots(devices: &Devices, slots_per_region: u64, spacing: u32) -> Vec<String> {
    let s = RootRingSlotsPerRegion::from_system_configuration_field(slots_per_region).expect("S");
    let mut misplaced = Vec::new();
    for region in 0..3u64 {
        for slot in 0..slots_per_region {
            let ring_slot = RootRingSlot { region, slot };
            let device = &devices[usize::try_from(REGION_DEVICES[usize::try_from(region).expect("区域号")]).expect("盘号")].1;
            let mut bytes = vec![0u8; usize::try_from(PHYSICAL_BLOCK_SIZE).expect("根槽宽")];
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

fn red_invariants(devices: &Devices) -> Vec<String> {
    check_pool_image(&memory_pool(devices))
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some(format!("{invariant}<{}>", detail.chars().take(90).collect::<String>())),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

fn history(arm: &str, mut devices: Devices, slots_per_region: u64, io_min: u32, mounts: u64) {
    let parameters = parameters(slots_per_region, io_min);
    let spacing = parameters.geometry.fixed_structure_slot_spacing;
    println!(
        "name=history_geometry arm={arm} S={slots_per_region} spacing={spacing} aliased_slot_pairs={:?}",
        aliased_slot_pairs(slots_per_region, spacing)
    );
    for mount in 1..=mounts {
        let result = match mount_writable(&parameters, &mut devices) {
            Ok(_) => "ok".to_string(),
            Err(error) => format!("refused:{}", member_name(&format!("{error:?}"))),
        };
        let newest = choose_system_configuration(&devices)
            .ok()
            .and_then(|chosen| choose_root(&devices, &chosen))
            .map_or_else(|| "none".to_string(), |root| root.checkpoint_txg.0.to_string());
        println!(
            "name=history arm={arm} mount={mount} mount_result={result} chosen_root_txg={newest} misplaced_roots={:?} red={:?}",
            misplaced_roots(&devices, slots_per_region, spacing),
            red_invariants(&devices)
        );
    }
}

fn main() {
    mkfs_geometry_table();
    same_bytes(4, 5, 524_288);
    same_bytes(4, 16, 4096);
    same_bytes(4, 12, 131_072);
    let aliased = sixteen_slots_at_512_kibibytes();
    let mounts = 24;
    history("S16_spacing512KiB_rewritten", aliased, 16, 524_288, mounts);
    history("S16_spacing4KiB_direct", formatted(16, 4096), 16, 4096, mounts);
    history("S4_spacing512KiB_direct", formatted(4, 524_288), 4, 524_288, mounts);
    // mkfs 拒、而三个区域在各自那块盘上并不相叠的两格：区域 0 与 2 同在盘 0、相距 6 MiB，区域 1 独在盘 1。
    history("S16_spacing256KiB_rewritten", rewritten(4, 16, 262_144), 16, 262_144, mounts);
    history("S8_spacing512KiB_rewritten", rewritten(4, 8, 524_288), 8, 524_288, mounts);
}

/// mkfs 收的 S = `from` 那一份池改成自述 S = `to`（`same_bytes` 那三格证过：空盘上与直接写 `to` 的 mkfs 逐扇区相同）。
fn rewritten(from: u64, to: u64, io_min: u32) -> Devices {
    let spacing = u64::from(SystemImmutableSizes::slot_spacing_for(io_min));
    let mut devices = formatted(from, io_min);
    declare_slots_per_region(&mut devices, spacing, u8::try_from(to).expect("S 装得进一字节"));
    devices
}
