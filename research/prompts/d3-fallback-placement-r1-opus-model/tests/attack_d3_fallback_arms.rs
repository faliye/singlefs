//! d3-fallback-placement-r1 攻方腿模型（副本用，要先打 `tools/apply_fallback_arms.py`）：回落时优先已在重写集里的叶，几条臂在同一批形状上比。
//! 跑前写下的预期（结果反过来就撤回）：
//! ① 改空闲图造的盘上，「先重写集、不够退最低空槽」的臂与今天逐探相同（`did_not_settle` 那几探照撞），因为重写集里的空槽就是最低空槽串的前缀；只有改了「退到哪」的臂修得了。
//! ② 重写集的叶自己空槽多、最低空槽在别的叶时，「先重写集」的臂轮数与角色数都比今天少（这是它真赚的那一格，不是稻草人）。
//! ③ 两块盘差不到一段时，「每块盘各答」的臂在今天不拒的地方拒（`AnswersDiffer`）；池级的臂不拒。
//! ④ 只用用户操作老化到没有可开的段，再删光一段里的文件：「先重写集」的臂把提交内生块放回正在腾的那一段，全空段数比今天少、前台写比今天先被合取 7 拒。
//! 量到 ④ 不成立（全空段数不比今天少、前台写不比今天差）就撤回「变差」那一格。
//! 第一段（forged）与第二段（rich_release_leaf）照 `attack_fixes_r3_settling_fallback.rs` 的造法改空闲图，不写分配记录，不跑池级 checker。第四段（aged）只用用户操作，每段跑完判池级 checker。

mod common_admission_pool;

use std::collections::BTreeMap;
use std::time::Instant;

use common_admission_pool::{content_of, plain_device, PoolUnderTest, OVERWRITE_BYTES};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, FileOffsetInBytes, InodeNumber, SlotNumber,
};
use singlefs_core::allocation_record_tree::{
    AllocationRecordTreeNode, AllocationRecordTreeNodePosition,
};
use singlefs_core::allocator::fallback_arm_model::{
    prefer_the_nodes, reset_the_tally, set_the_arm, the_tally, FallbackArm,
};
use singlefs_core::allocator::{DeviceFreeMap, PoolAllocator, UnitAreaStart, UnitFootprint};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mounted_session::UserChange;
use singlefs_core::transaction::{
    publish_overwrite_of_an_inode, PartialWriteBytes, PoolWriter, PublishError, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::{ALLOCATION_RECORD_TREE_LEAF_SLOTS, CLUSTER_SEGMENT_SLOTS, SLOT_BYTES};
use singlefs_verification::memory_pool::SparseBlockDevice;
use singlefs_verification::scenario::e142_parameters;

type Pool = PoolUnderTest<SparseBlockDevice>;
type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

const JOURNAL_RING_BYTES: u64 = 6 << 20;
const FORGED_UNIT_AREA_SLOTS: u64 = 1 << 18;
const FIRST_LEAF_OF_THE_FREE_LIST: u64 = 12;

fn parameters() -> MakeFilesystemParameters {
    let mut parameters = e142_parameters(512, 512);
    parameters.geometry.journal_ring_bytes = JOURNAL_RING_BYTES;
    parameters
}

fn unit_area_start_slot() -> u64 {
    UnitAreaStart::following_the_journal_ring(JOURNAL_RING_BYTES)
        .expect("6 MiB 的环末尾落在聚簇段边界上")
        .slot()
        .0
}

fn device_bytes(unit_area_slots: u64) -> u64 {
    (unit_area_start_slot() + unit_area_slots) * SLOT_BYTES
}

fn copy_of_the_devices(devices: &Devices, bytes: u64) -> Devices {
    devices
        .iter()
        .map(|(identity, device)| {
            let mut copy = SparseBlockDevice::new(bytes, PhysicalBlockSizeInBytes(512));
            copy.image = device.image.clone();
            (*identity, copy)
        })
        .collect()
}

#[derive(Clone, Copy)]
struct FreeShape {
    free_slots_per_leaf: u64,
    leaves: u64,
    slots_of_the_dense_prefix: u64,
}

/// 同 `attack_fixes_r3_settling_fallback.rs`：稠密前缀、之后每片叶 F 个空槽、再往后每段留段首之外的槽。
fn forge_the_free_map(allocator: &mut PoolAllocator, shape: FreeShape) {
    let start = unit_area_start_slot();
    let leaf_slots = ALLOCATION_RECORD_TREE_LEAF_SLOTS;
    let is_left_free = |slot: u64| {
        let leaf = slot / leaf_slots;
        let offset = slot % leaf_slots;
        if leaf < FIRST_LEAF_OF_THE_FREE_LIST {
            false
        } else if leaf == FIRST_LEAF_OF_THE_FREE_LIST {
            offset < shape.slots_of_the_dense_prefix
        } else if leaf <= FIRST_LEAF_OF_THE_FREE_LIST + shape.leaves {
            (400..400 + shape.free_slots_per_leaf).contains(&offset)
        } else {
            (slot - start) % CLUSTER_SEGMENT_SLOTS != 0
        }
    };
    for device_map in &mut allocator.devices {
        for slot in start..start + device_map.unit_area_slots() {
            if !is_left_free(slot) && device_map.is_free(SlotNumber(slot)) {
                device_map.mark_allocated(SlotNumber(slot), 1);
            }
        }
    }
}

struct Probe {
    outcome: String,
    rounds: usize,
    roles: usize,
    did_not_settle: bool,
}

/// 在 `allocator`（调用方的拷贝）与盘的拷贝上发一次覆盖写第一个文件；角色数取固定点定下来那一刻的（合取 7 拒的也有）。
fn probe(pool: &Pool, allocator: &mut PoolAllocator, bytes: u64, write_time_seconds: u64) -> Probe {
    let parameters = parameters();
    let mut devices = copy_of_the_devices(&pool.devices, bytes);
    let session = pool.session();
    let previous = session.current.file_version().expect("带文件").clone();
    let content = content_of(OVERWRITE_BYTES, write_time_seconds);
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    reset_the_tally();
    let published = publish_overwrite_of_an_inode(
        &mut writer,
        allocator,
        &previous,
        InodeNumber(FIRST_INODE_NUMBER),
        &content,
        write_time_seconds,
        session.instance,
    );
    let rounds = writer.rounds_of_the_last_allocation_record_tree_settling();
    let roles = the_tally().last_roles;
    match published {
        Ok(_) => Probe { outcome: "published".to_string(), rounds, roles, did_not_settle: false },
        Err(PublishError::AllocationRecordTreeRewriteSetDidNotSettle { rounds: limit }) => Probe {
            outcome: format!("did_not_settle_limit_{limit}"),
            rounds: limit,
            roles,
            did_not_settle: true,
        },
        Err(PublishError::EmptyClusterSegmentsBelowTheFloor(_)) => Probe {
            outcome: "settled_then_refused_by_the_empty_segments_floor".to_string(),
            rounds,
            roles,
            did_not_settle: false,
        },
        Err(other) => Probe {
            outcome: format!("{other:?}").chars().take(100).collect(),
            rounds,
            roles,
            did_not_settle: false,
        },
    }
}

#[derive(Default)]
struct SweepTotal {
    probes: u64,
    did_not_settle: u64,
    most_rounds: usize,
    roles_sum: u64,
    worst_prefix: u64,
}

fn one_forged_probe(pool: &Pool, arm: FallbackArm, shape: FreeShape, sweep: &str, bytes: u64, total: &mut SweepTotal) {
    let began = Instant::now();
    let mut allocator = pool.session().allocator.clone();
    forge_the_free_map(&mut allocator, shape);
    let result = probe(pool, &mut allocator, bytes, pool.write_time_seconds + 100);
    println!(
        "d3-fallback: forged arm={} sweep={sweep} free_slots_per_leaf={} leaves={} prefix={} outcome={} rounds={} roles={} seconds={:.1}",
        arm.name(),
        shape.free_slots_per_leaf,
        shape.leaves,
        shape.slots_of_the_dense_prefix,
        result.outcome,
        result.rounds,
        result.roles,
        began.elapsed().as_secs_f64()
    );
    total.probes += 1;
    total.did_not_settle += u64::from(result.did_not_settle);
    total.roles_sum += u64::try_from(result.roles).expect("角色数");
    if result.rounds > total.most_rounds {
        total.most_rounds = result.rounds;
        total.worst_prefix = shape.slots_of_the_dense_prefix;
    }
}

/// 第一段：r3 模型的 `fallback` 同一造法，六条臂同一批形状。扫前缀 0..=40（每片叶 2 空槽、120 片），再在 14 与这条臂最差的前缀上扫 F × 片数。
#[test]
fn forged_free_maps_under_each_arm() {
    let bytes = device_bytes(FORGED_UNIT_AREA_SLOTS);
    let mut pool: Pool =
        PoolUnderTest::start_after_the_first_file_on_devices_of(parameters(), bytes, plain_device);
    for _ in 0..4 {
        pool.overwrite(OVERWRITE_BYTES).expect("空盘上覆盖写做得成");
    }
    let mut did_not_settle_of_today = 0;
    for arm in FallbackArm::ALL {
        set_the_arm(arm);
        let started = Instant::now();
        let mut total = SweepTotal::default();
        for prefix in 0..=40u64 {
            let shape = FreeShape { free_slots_per_leaf: 2, leaves: 120, slots_of_the_dense_prefix: prefix };
            one_forged_probe(&pool, arm, shape, "prefix", bytes, &mut total);
        }
        let worst_prefix = total.worst_prefix;
        for prefix in [14, worst_prefix] {
            for free_slots_per_leaf in [1u64, 2, 3, 4, 8, 32] {
                for leaves in [32u64, 64, 120, 200] {
                    let shape = FreeShape { free_slots_per_leaf, leaves, slots_of_the_dense_prefix: prefix };
                    one_forged_probe(&pool, arm, shape, "shape", bytes, &mut total);
                }
            }
        }
        println!(
            "d3-fallback: forged_summary arm={} probes={} did_not_settle={} most_rounds={} worst_prefix={worst_prefix} roles_sum={} seconds={:.0}",
            arm.name(),
            total.probes,
            total.did_not_settle,
            total.most_rounds,
            total.roles_sum,
            started.elapsed().as_secs_f64()
        );
        if arm == FallbackArm::Today {
            did_not_settle_of_today = total.did_not_settle;
        }
    }
    set_the_arm(FallbackArm::Today);
    assert!(did_not_settle_of_today > 0, "装置看门：今天的实现在这批形状上要撞到上界（r3 模型 prefix=14、15 那几探）");
}

/// 第二段：重写集的叶自己空槽多（上一版的提交内生块都在第 `home_leaf` 片叶里），最低空槽却在更低的稀疏叶里：今天往下走稀疏叶，先重写集的臂留在自己叶里。
#[test]
fn rich_release_leaf_under_each_arm() {
    let bytes = device_bytes(FORGED_UNIT_AREA_SLOTS);
    let start = unit_area_start_slot();
    for home_leaf in [20u64, 40, 80, 160] {
        let mut pool: Pool =
            PoolUnderTest::start_after_the_first_file_on_devices_of(parameters(), bytes, plain_device);
        {
            let allocator = &mut pool.session.as_mut().expect("有会话").allocator;
            for device_map in &mut allocator.devices {
                for slot in start..home_leaf * ALLOCATION_RECORD_TREE_LEAF_SLOTS {
                    if device_map.is_free(SlotNumber(slot)) {
                        device_map.mark_allocated(SlotNumber(slot), 1);
                    }
                }
            }
        }
        for _ in 0..4 {
            pool.overwrite(OVERWRITE_BYTES).expect("开段落到 home_leaf 之后覆盖写做得成");
        }
        for free_slots_per_leaf in [1u64, 2, 3, 8] {
            let mut forged = pool.session().allocator.clone();
            for device_map in &mut forged.devices {
                for slot in start..start + device_map.unit_area_slots() {
                    let leaf = slot / ALLOCATION_RECORD_TREE_LEAF_SLOTS;
                    let offset = slot % ALLOCATION_RECORD_TREE_LEAF_SLOTS;
                    let left_free = if leaf < FIRST_LEAF_OF_THE_FREE_LIST {
                        false
                    } else if leaf < home_leaf {
                        (400..400 + free_slots_per_leaf).contains(&offset)
                    } else {
                        (slot - start) % CLUSTER_SEGMENT_SLOTS != 0
                    };
                    if !left_free && device_map.is_free(SlotNumber(slot)) {
                        device_map.mark_allocated(SlotNumber(slot), 1);
                    }
                }
            }
            for arm in FallbackArm::ALL {
                set_the_arm(arm);
                let mut allocator = forged.clone();
                let result = probe(&pool, &mut allocator, bytes, pool.write_time_seconds + 100);
                println!(
                    "d3-fallback: rich_release_leaf arm={} home_leaf={home_leaf} free_slots_per_leaf={free_slots_per_leaf} outcome={} rounds={} roles={}",
                    arm.name(),
                    result.outcome,
                    result.rounds,
                    result.roles
                );
            }
        }
    }
    set_the_arm(FallbackArm::Today);
}

/// 第三段：两块盘差不到一段（大盘多出来的几槽凑不成段）。全空段都没有，低处留一个两盘都空的槽；大盘末尾多出来的槽落在小盘最后一片叶里，那片叶在重写集里。
#[test]
fn devices_differing_by_less_than_a_segment_under_each_arm() {
    let start = UnitAreaStart::following_the_journal_ring(JOURNAL_RING_BYTES).expect("落在段边界");
    let small_area = 64 * CLUSTER_SEGMENT_SLOTS;
    for extra_slots in [1u64, 33, 63] {
        let small_end = start.slot().0 + small_area;
        let lowest_common_free = start.slot().0 + 2 * CLUSTER_SEGMENT_SLOTS + 5;
        let last_leaf_of_the_small_device = (small_end - 1) / ALLOCATION_RECORD_TREE_LEAF_SLOTS;
        let mut base = PoolAllocator::new(vec![
            DeviceFreeMap::with_unit_area_start(DeviceIdentity(0), (small_end + extra_slots) * SLOT_BYTES, start),
            DeviceFreeMap::with_unit_area_start(DeviceIdentity(1), small_end * SLOT_BYTES, start),
        ]);
        for device_map in &mut base.devices {
            for slot in start.slot().0..small_end {
                if slot != lowest_common_free {
                    device_map.mark_allocated(SlotNumber(slot), 1);
                }
            }
        }
        let leaf_node = |device: u32| {
            AllocationRecordTreeNode::BelowTheRoot(AllocationRecordTreeNodePosition {
                level: 0,
                device: DeviceIdentity(device),
                index_in_device: last_leaf_of_the_small_device,
            })
        };
        for arm in FallbackArm::ALL {
            set_the_arm(arm);
            prefer_the_nodes([leaf_node(0), leaf_node(1)].into_iter());
            let mut allocator = base.clone();
            let answer = allocator.try_allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(1));
            println!(
                "d3-fallback: unequal arm={} extra_slots={extra_slots} lowest_common_free={lowest_common_free} answer={}",
                arm.name(),
                format!("{answer:?}").chars().take(120).collect::<String>()
            );
        }
    }
    set_the_arm(FallbackArm::Today);
}

// ── 第四段（aged）：只用用户操作与重挂，不改空闲图 ──

const AGED_UNIT_AREA_SLOTS: u64 = 1 << 14;
const AGED_REMOUNTS_DURING_THE_FILL: usize = 3;
const AGED_WHOLE_RUN_SECONDS_AT_MOST: f64 = 40.0 * 60.0;

fn create_inodes(pool: &mut Pool, count: u64) -> Option<Vec<InodeNumber>> {
    let watermark = pool.current_file_version().inode_number_watermark();
    pool.write_time_seconds += 1;
    let write_time_seconds = pool.write_time_seconds;
    pool.publish(UserChange::NewInodes { count, write_time_seconds }, None).ok()?;
    Some((watermark..watermark + count).map(InodeNumber).collect())
}

fn write_one_data_unit(pool: &mut Pool, inode: InodeNumber) -> Result<(), String> {
    pool.write_time_seconds += 1;
    let write_time_seconds = pool.write_time_seconds;
    let bytes = content_of(data_unit_payload_capacity(), write_time_seconds);
    pool.publish(
        UserChange::WriteAtOffset {
            inode,
            offset: FileOffsetInBytes(0),
            bytes: PartialWriteBytes::new(&bytes).expect("不空"),
            write_time_seconds,
        },
        None,
    )
    .map(|_| ())
    .map_err(|refused| format!("{refused:?}").chars().take(110).collect())
}

fn unlink(pool: &mut Pool, inode: InodeNumber) -> Result<(), String> {
    pool.write_time_seconds += 1;
    let write_time_seconds = pool.write_time_seconds;
    pool.publish(UserChange::Unlink { inode, write_time_seconds }, None)
        .map(|_| ())
        .map_err(|refused| format!("{refused:?}").chars().take(110).collect())
}

fn shape_of(pool: &Pool) -> String {
    let allocator = &pool.session().allocator;
    let start = unit_area_start_slot();
    let device_map = &allocator.devices[0];
    let mut free_of_each_leaf: Vec<u64> = Vec::new();
    let mut runs = 0u64;
    let mut previous_free = false;
    for slot in start..start + device_map.unit_area_slots() {
        let leaf = usize::try_from(slot / ALLOCATION_RECORD_TREE_LEAF_SLOTS).expect("片号");
        if free_of_each_leaf.len() <= leaf {
            free_of_each_leaf.resize(leaf + 1, 0);
        }
        let free = device_map.is_free(SlotNumber(slot));
        free_of_each_leaf[leaf] += u64::from(free);
        runs += u64::from(free && !previous_free);
        previous_free = free;
    }
    format!(
        "empty_segments={} openable={} free_slots={} runs={runs} fell_back={} free_of_each_leaf={:?}",
        device_map.empty_segments(),
        device_map.lowest_empty_segment().is_some(),
        device_map.free_slots(),
        allocator.commit_generated_placements_that_fell_back_to_the_lowest_free_slot(),
        free_of_each_leaf
    )
}

struct FilledFiles {
    /// 每个还在的文件的数据单元落在哪个槽（发布之前在分配器拷贝上取同一个落点）。
    slot_of_each_file: Vec<(InodeNumber, u64)>,
}

/// 写到前台被拒。交回拒绝的原文。
fn fill_until_refused(pool: &mut Pool, files: &mut FilledFiles, started: Instant, estimate_at: &mut Option<f64>) -> String {
    loop {
        let Some(batch) = create_inodes(pool, 100) else {
            return "建 inode 被拒".to_string();
        };
        for inode in batch {
            let slot = pool
                .session()
                .allocator
                .clone()
                .allocate_user_data(CheckpointTxg(1))
                .map(|placement| placement.slot.0);
            match (write_one_data_unit(pool, inode), slot) {
                (Ok(()), Some(slot)) => files.slot_of_each_file.push((inode, slot)),
                (Ok(()), None) => {}
                (Err(refused), _) => return refused,
            }
            if files.slot_of_each_file.len() == 500 && estimate_at.is_none() {
                *estimate_at = Some(started.elapsed().as_secs_f64());
            }
        }
    }
}

/// 从同一份盘面起一个新池，重挂一次（重挂写的根照这条臂走）。
fn pool_from_the_image(image: &Devices, bytes: u64, write_time_seconds: u64) -> Result<Pool, String> {
    let mut pool: Pool = PoolUnderTest {
        parameters: parameters(),
        device_bytes: bytes,
        devices: copy_of_the_devices(image, bytes),
        session: None,
        write_time_seconds,
        content_of_the_current_version: Vec::new(),
        content_of_each_root: BTreeMap::new(),
    };
    pool.crash_and_mount_writable()
        .map(|_| ())
        .map_err(|refused| format!("{refused:?}").chars().take(160).collect::<String>())?;
    Ok(pool)
}

fn checker_verdict(pool: &Pool) -> String {
    let verdicts = check_pool_image(&pool.image());
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect();
    format!("checker_verdicts={} checker_violated={} names={violated:?}", verdicts.len(), violated.len())
}

fn tally_text() -> String {
    let tally = the_tally();
    format!(
        "settled={} most_rounds={} rounds_sum={} roles_sum={} did_not_settle={}",
        tally.settled, tally.most_rounds, tally.rounds_sum, tally.roles_sum, tally.did_not_settle
    )
}

#[derive(Clone, Copy, Debug)]
enum WhichSegments {
    Lowest,
    Middle,
    Highest,
}

/// 第四段的一条序列：删光 `count` 个段里的文件（用户定的位置与个数），再写：覆盖写别的文件 `writes` 次，数成了几次、第一次被拒的原因、全空段数的轨迹。
fn delete_whole_segments_then_write(
    arm: FallbackArm,
    image: &Devices,
    bytes: u64,
    write_time_seconds: u64,
    files: &FilledFiles,
    which: WhichSegments,
    count: usize,
    writes: usize,
) {
    set_the_arm(arm);
    reset_the_tally();
    let label = format!("arm={} which={which:?} count={count}", arm.name());
    let mut pool = match pool_from_the_image(image, bytes, write_time_seconds) {
        Ok(pool) => pool,
        Err(refused) => {
            println!("d3-fallback: aged_delete {label} mount_refused={refused}");
            return;
        }
    };
    let start = unit_area_start_slot();
    let mut files_per_segment: BTreeMap<u64, Vec<InodeNumber>> = BTreeMap::new();
    for (inode, slot) in &files.slot_of_each_file {
        files_per_segment.entry((slot - start) / CLUSTER_SEGMENT_SLOTS).or_default().push(*inode);
    }
    let full_of_data: Vec<u64> = files_per_segment
        .iter()
        .filter(|(_, inodes)| inodes.len() >= 28)
        .map(|(segment, _)| *segment)
        .collect();
    if full_of_data.len() < count {
        println!("d3-fallback: aged_delete {label} not_enough_segments_full_of_data={}", full_of_data.len());
        return;
    }
    let first = match which {
        WhichSegments::Lowest => 0,
        WhichSegments::Middle => (full_of_data.len() - count) / 2,
        WhichSegments::Highest => full_of_data.len() - count,
    };
    let chosen = &full_of_data[first..first + count];
    println!("d3-fallback: aged_delete {label} chosen_segments={chosen:?} {}", shape_of(&pool));
    let mut unlink_refusals = 0u64;
    for segment in chosen {
        for inode in &files_per_segment[segment] {
            if let Err(refused) = unlink(&mut pool, *inode) {
                unlink_refusals += 1;
                if unlink_refusals == 1 {
                    println!("d3-fallback: aged_delete {label} first_unlink_refusal={refused}");
                }
            }
        }
    }
    println!("d3-fallback: aged_delete {label} after_the_unlinks unlink_refusals={unlink_refusals} {} {}", shape_of(&pool), tally_text());
    let kept: Vec<InodeNumber> = files_per_segment
        .iter()
        .filter(|(segment, _)| !chosen.contains(segment))
        .flat_map(|(_, inodes)| inodes.iter().copied())
        .collect();
    let mut written = 0u64;
    let mut first_refusal = String::new();
    for write in 0..writes {
        match write_one_data_unit(&mut pool, kept[write * 7 % kept.len()]) {
            Ok(()) => written += 1,
            Err(refused) => {
                if first_refusal.is_empty() {
                    first_refusal = format!("at_write={write} {refused}");
                }
            }
        }
        if (write + 1) % 16 == 0 {
            let device_map = &pool.session().allocator.devices[0];
            println!(
                "d3-fallback: aged_delete_trace {label} write={} written={written} empty_segments={} openable={}",
                write + 1,
                device_map.empty_segments(),
                device_map.lowest_empty_segment().is_some()
            );
        }
    }
    println!(
        "d3-fallback: aged_delete_summary {label} unlink_refusals={unlink_refusals} writes={writes} written={written} first_refusal=[{first_refusal}] {} {} {}",
        shape_of(&pool),
        tally_text(),
        checker_verdict(&pool)
    );
}

/// 第四段的另一条序列：隔 `unlink_every` 个删一个，再随机覆盖写。
fn scattered_unlinks_then_churn(arm: FallbackArm, image: &Devices, bytes: u64, write_time_seconds: u64, files: &FilledFiles, unlink_every: usize, churn: usize) {
    set_the_arm(arm);
    reset_the_tally();
    let label = format!("arm={} unlink_every={unlink_every}", arm.name());
    let mut pool = match pool_from_the_image(image, bytes, write_time_seconds) {
        Ok(pool) => pool,
        Err(refused) => {
            println!("d3-fallback: aged_churn {label} mount_refused={refused}");
            return;
        }
    };
    let mut kept: Vec<InodeNumber> = Vec::new();
    let mut unlink_refusals = 0u64;
    for (index, (inode, _)) in files.slot_of_each_file.iter().enumerate() {
        if index % unlink_every != 0 {
            kept.push(*inode);
        } else if unlink(&mut pool, *inode).is_err() {
            unlink_refusals += 1;
            kept.push(*inode);
        }
    }
    println!("d3-fallback: aged_churn {label} after_the_unlinks unlink_refusals={unlink_refusals} {} {}", shape_of(&pool), tally_text());
    let mut state = 88_172_645_463_325_252u64;
    let (mut written, mut refused) = (0u64, 0u64);
    for step in 0..churn {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let inode = kept[usize::try_from(state % kept.len() as u64).expect("下标")];
        if write_one_data_unit(&mut pool, inode).is_ok() {
            written += 1;
        } else {
            refused += 1;
        }
        if (step + 1) % 100 == 0 {
            println!("d3-fallback: aged_churn_trace {label} step={} written={written} refused={refused} {}", step + 1, shape_of(&pool));
        }
    }
    println!("d3-fallback: aged_churn_summary {label} written={written} refused={refused} {} {} {}", shape_of(&pool), tally_text(), checker_verdict(&pool));
}

/// 老化：今天的实现写到被拒、重挂、再写，几趟。盘面只建这一次，之后每条序列从它拷。
#[test]
fn aged_by_user_operations_under_each_arm() {
    let started = Instant::now();
    let bytes = device_bytes(AGED_UNIT_AREA_SLOTS);
    set_the_arm(FallbackArm::Today);
    let mut pool: Pool =
        PoolUnderTest::start_after_the_first_file_on_devices_of(parameters(), bytes, plain_device);
    let mut files = FilledFiles { slot_of_each_file: Vec::new() };
    let mut estimate_at: Option<f64> = None;
    for cycle in 0..=AGED_REMOUNTS_DURING_THE_FILL {
        let refusal = fill_until_refused(&mut pool, &mut files, started, &mut estimate_at);
        println!(
            "d3-fallback: aged_fill cycle={cycle} files={} refusal=[{refusal}] {} seconds={:.0}",
            files.slot_of_each_file.len(),
            shape_of(&pool),
            started.elapsed().as_secs_f64()
        );
        if let Some(seconds_for_500) = estimate_at {
            let expected_files = AGED_UNIT_AREA_SLOTS as f64 / 2.0;
            let publishes_of_the_sequences = 6.0 * (6.0 * 200.0 + 2.0 * 2000.0);
            let estimate = seconds_for_500 / 500.0 * (expected_files + publishes_of_the_sequences);
            println!("d3-fallback: aged_estimate seconds_for_500_files={seconds_for_500:.0} whole_run_estimate_seconds={estimate:.0} limit={AGED_WHOLE_RUN_SECONDS_AT_MOST:.0}");
            if estimate > AGED_WHOLE_RUN_SECONDS_AT_MOST {
                println!("d3-fallback: aged_stopped_by_the_estimate");
                return;
            }
        }
        if cycle < AGED_REMOUNTS_DURING_THE_FILL {
            if let Err(refused) = pool.crash_and_mount_writable() {
                println!("d3-fallback: aged_fill cycle={cycle} remount_refused={}", format!("{refused:?}").chars().take(200).collect::<String>());
                break;
            }
        }
    }
    println!("d3-fallback: aged_start {} {}", shape_of(&pool), checker_verdict(&pool));
    let image = copy_of_the_devices(&pool.devices, bytes);
    let write_time_seconds = pool.write_time_seconds;
    for arm in FallbackArm::ALL {
        set_the_arm(arm);
        let mut probing = pool_from_the_image(&image, bytes, write_time_seconds);
        if let Ok(probing) = probing.as_mut() {
            let mut allocator = probing.session().allocator.clone();
            let result = probe(probing, &mut allocator, bytes, write_time_seconds + 100);
            println!("d3-fallback: aged_probe arm={} outcome={} rounds={} roles={} {}", arm.name(), result.outcome, result.rounds, result.roles, shape_of(probing));
        }
        for which in [WhichSegments::Lowest, WhichSegments::Middle, WhichSegments::Highest] {
            for count in [1usize, 4] {
                delete_whole_segments_then_write(arm, &image, bytes, write_time_seconds, &files, which, count, 200);
            }
        }
        for unlink_every in [8usize, 64] {
            scattered_unlinks_then_churn(arm, &image, bytes, write_time_seconds, &files, unlink_every, 400);
        }
        println!("d3-fallback: aged_arm_done arm={} seconds={:.0}", arm.name(), started.elapsed().as_secs_f64());
    }
    set_the_arm(FallbackArm::Today);
}
