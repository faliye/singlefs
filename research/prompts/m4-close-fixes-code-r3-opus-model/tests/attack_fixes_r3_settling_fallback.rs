//! m4-close-fixes-code-r3 攻方腿模型（副本用）：P2a 之后固定点还剩「取落点碰到新叶」那一半轮数，这一半在提交内生块回落到最低空槽时（D3（空间分配） 已定项 8 ②）长不长。
//! 一片叶罩 812 槽，一个单元的两份落两块盘同一槽：取落点每碰到一片新叶，下一轮多重写两片叶（两块盘各一），又多要两个落点。
//! 空闲槽每片叶 F 个时，一轮多出来的落点数 × 2 / F 就是下一轮多出来的落点数：F = 2 不衰减，轮数 ≈ 2 × 片数 / 每轮的差，走到稠密的那一段才定。
//! 第一段（forged）：空闲图直接改（`commit_generated_units_fall_back_when_the_segment_runs_out.rs` 同一造法，不写分配记录，只改落点去处），扫 F、片数、前缀长度，量产品的固定点走几轮、撞不撞 64。
//! 第二段（natural）：不改空闲图，只用用户操作把小盘写到被拒、隔几个删一个、再随机覆盖写，隔一段探一次，量可达的历史上轮数到几、进没进回落。
//! 探一次：分配器拷贝、盘拷贝上发一次覆盖写，读 `PoolWriter::rounds_of_the_last_allocation_record_tree_settling`；没有全空段时前台写在固定点定下之后被合取 7 拒，轮数已经记下。

mod common_admission_pool;

use std::cell::Cell;
use std::time::Instant;

use common_admission_pool::{content_of, plain_device, PoolUnderTest, OVERWRITE_BYTES};
use singlefs_core::address::{DeviceIdentity, FileOffsetInBytes, InodeNumber, SlotNumber};
use singlefs_core::allocator::{PoolAllocator, UnitAreaStart};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mounted_session::UserChange;
use singlefs_core::transaction::{
    publish_overwrite_of_an_inode, PartialWriteBytes, PoolWriter, PublishError, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::{ALLOCATION_RECORD_TREE_LEAF_SLOTS, SLOT_BYTES};
use singlefs_verification::memory_pool::SparseBlockDevice;
use singlefs_verification::scenario::e142_parameters;

type Pool = PoolUnderTest<SparseBlockDevice>;
type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

const JOURNAL_RING_BYTES: u64 = 6 << 20;
/// 第一段挂多大的单元区：片数上限是它决定的（2¹⁸ 槽约 322 片叶）。
const FORGED_UNIT_AREA_SLOTS: u64 = 1 << 18;
/// 第一段稀疏那一段从第几片叶起：前面的（第一个文件、开放段都在里面）一个空槽都不留。
const FIRST_LEAF_OF_THE_FREE_LIST: u64 = 12;
const FORGED_SWEEP_SECONDS_AT_MOST: f64 = 20.0 * 60.0;
const NATURAL_RUN_SECONDS_AT_MOST: f64 = 15.0 * 60.0;
const NATURAL_UNIT_AREA_SLOTS: u64 = 1 << 14;

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

/// 只留：第 [`FIRST_LEAF_OF_THE_FREE_LIST`] 片叶开头一段稠密前缀；之后 `leaves` 片叶每片 `free_slots_per_leaf` 个连着的空槽（偶数起点，数据单元也落得下）；
/// 再往后每个 64 槽段留段首之外的槽（一个全空段都没有，放得下收尾）。
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
            (slot - start) % 64 != 0
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

/// 在 `allocator`（调用方的拷贝）与盘的拷贝上发一次覆盖写第一个文件。
fn probe(pool: &Pool, allocator: &mut PoolAllocator, bytes: u64, write_time_seconds: u64) -> Probe {
    let parameters = parameters();
    let mut devices = copy_of_the_devices(&pool.devices, bytes);
    let session = pool.session();
    let previous = session.current.file_version().expect("带文件").clone();
    let content = content_of(OVERWRITE_BYTES, write_time_seconds);
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
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
    match published {
        Ok(version) => Probe {
            outcome: "published".to_string(),
            rounds,
            roles: version.rewritten.len(),
            did_not_settle: false,
        },
        Err(PublishError::AllocationRecordTreeRewriteSetDidNotSettle { rounds: limit }) => Probe {
            outcome: format!("did_not_settle_limit_{limit}"),
            rounds: limit,
            roles: 0,
            did_not_settle: true,
        },
        Err(PublishError::EmptyClusterSegmentsBelowTheFloor(_)) => Probe {
            outcome: "settled_then_refused_by_the_empty_segments_floor".to_string(),
            rounds,
            roles: 0,
            did_not_settle: false,
        },
        Err(other) => Probe {
            outcome: format!("{other:?}").chars().take(160).collect(),
            rounds,
            roles: 0,
            did_not_settle: false,
        },
    }
}

#[test]
fn forged_free_maps_sweep_the_rounds_of_the_placement_half_of_the_fixed_point() {
    let bytes = device_bytes(FORGED_UNIT_AREA_SLOTS);
    let mut pool: Pool =
        PoolUnderTest::start_after_the_first_file_on_devices_of(parameters(), bytes, plain_device);
    for _ in 0..4 {
        pool.overwrite(OVERWRITE_BYTES).expect("空盘上覆盖写做得成");
    }
    let started = Instant::now();
    let probes = Cell::new(0u64);
    let did_not_settle = Cell::new(0u64);
    let most_rounds = Cell::new(0usize);
    let prefix_of_the_most_rounds = Cell::new(0u64);
    let run_one = |pool: &Pool, shape: FreeShape, sweep: &str| -> Probe {
        let began = Instant::now();
        let mut allocator = pool.session().allocator.clone();
        forge_the_free_map(&mut allocator, shape);
        let result = probe(pool, &mut allocator, bytes, pool.write_time_seconds + 100);
        println!(
            "attack-fixes-r3: fallback sweep={sweep} free_slots_per_leaf={} leaves={} prefix={} outcome={} rounds={} roles={} seconds={:.1}",
            shape.free_slots_per_leaf,
            shape.leaves,
            shape.slots_of_the_dense_prefix,
            result.outcome,
            result.rounds,
            result.roles,
            began.elapsed().as_secs_f64()
        );
        probes.set(probes.get() + 1);
        did_not_settle.set(did_not_settle.get() + u64::from(result.did_not_settle));
        if result.rounds > most_rounds.get() {
            most_rounds.set(result.rounds);
            prefix_of_the_most_rounds.set(shape.slots_of_the_dense_prefix);
        }
        result
    };
    // 先跑一个点打估时：前缀 0、每片两个空槽。
    let first = run_one(
        &pool,
        FreeShape { free_slots_per_leaf: 2, leaves: 120, slots_of_the_dense_prefix: 0 },
        "estimate",
    );
    println!("attack-fixes-r3: fallback estimate first_probe_rounds={} budget_seconds={FORGED_SWEEP_SECONDS_AT_MOST:.0}", first.rounds);
    // 每片两个空槽：扫前缀长度（前缀吃掉多少，头一次碰到稀疏那一段的差就是多少）。撞到上界三次就够，过了预算就停、打印停在哪。
    let mut hits = 0;
    for prefix in 1..=160u64 {
        if started.elapsed().as_secs_f64() > FORGED_SWEEP_SECONDS_AT_MOST / 2.0 || hits >= 3 {
            println!("attack-fixes-r3: fallback prefix_sweep_stopped_at={prefix} hits={hits} seconds={:.0}", started.elapsed().as_secs_f64());
            break;
        }
        let result = run_one(
            &pool,
            FreeShape { free_slots_per_leaf: 2, leaves: 120, slots_of_the_dense_prefix: prefix },
            "prefix",
        );
        hits += u32::from(result.did_not_settle);
    }
    let prefix = prefix_of_the_most_rounds.get();
    // 轮数最多的那个前缀上扫每片空槽数与片数：F 大于 2 的衰减，片数少的走不满。
    for free_slots_per_leaf in [1u64, 2, 3, 4, 8, 32] {
        for leaves in [32u64, 64, 120, 200] {
            if started.elapsed().as_secs_f64() > FORGED_SWEEP_SECONDS_AT_MOST {
                println!("attack-fixes-r3: fallback shape_sweep_stopped seconds={:.0}", started.elapsed().as_secs_f64());
                break;
            }
            run_one(
                &pool,
                FreeShape { free_slots_per_leaf, leaves, slots_of_the_dense_prefix: prefix },
                "shape",
            );
        }
    }
    println!(
        "attack-fixes-r3: fallback_summary probes={} did_not_settle={} most_rounds={} prefix_of_the_most_rounds={prefix} seconds={:.0}",
        probes.get(),
        did_not_settle.get(),
        most_rounds.get(),
        started.elapsed().as_secs_f64()
    );
    assert!(probes.get() > 0);
}

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
    .map_err(|refused| format!("{refused:?}").chars().take(120).collect())
}

/// 探一次并打印这一刻的空闲形状：全空段有没有、回落发过几次、每片叶的空槽数（前 40 片）。
fn natural_probe(pool: &Pool, label: &str, most_rounds: &mut usize) {
    let bytes = device_bytes(NATURAL_UNIT_AREA_SLOTS);
    let mut allocator = pool.session().allocator.clone();
    let start = unit_area_start_slot();
    let device_map = &allocator.devices[0];
    let mut free_of_each_leaf: Vec<u64> = Vec::new();
    for slot in start..start + device_map.unit_area_slots() {
        let leaf = usize::try_from(slot / ALLOCATION_RECORD_TREE_LEAF_SLOTS).expect("片号");
        if free_of_each_leaf.len() <= leaf {
            free_of_each_leaf.resize(leaf + 1, 0);
        }
        free_of_each_leaf[leaf] += u64::from(device_map.is_free(SlotNumber(slot)));
    }
    let shape = format!(
        "empty_segments={} lowest_empty_segment={:?} free_slots={} fell_back_so_far={} free_of_each_leaf={:?}",
        device_map.empty_segments(),
        device_map.lowest_empty_segment(),
        device_map.free_slots(),
        allocator.commit_generated_placements_that_fell_back_to_the_lowest_free_slot(),
        free_of_each_leaf.iter().take(40).collect::<Vec<_>>()
    );
    let result = probe(pool, &mut allocator, bytes, pool.write_time_seconds + 1);
    *most_rounds = (*most_rounds).max(result.rounds);
    println!(
        "attack-fixes-r3: natural {label} outcome={} rounds={} roles={} {shape}",
        result.outcome, result.rounds, result.roles
    );
}

/// 只用用户操作：一个单元的小文件写到被拒，隔 `unlink_every` 个删一个，再随机覆盖写。
fn natural_history(unlink_every: usize) {
    let started = Instant::now();
    let bytes = device_bytes(NATURAL_UNIT_AREA_SLOTS);
    let mut pool: Pool =
        PoolUnderTest::start_after_the_first_file_on_devices_of(parameters(), bytes, plain_device);
    let mut most_rounds = 0usize;
    let mut files: Vec<InodeNumber> = Vec::new();
    let mut fill_refusal = String::new();
    'fill: loop {
        let Some(batch) = create_inodes(&mut pool, 100) else {
            fill_refusal = "建 inode 被拒".to_string();
            break 'fill;
        };
        for inode in batch {
            match write_one_data_unit(&mut pool, inode) {
                Ok(()) => files.push(inode),
                Err(refused) => {
                    fill_refusal = refused;
                    break 'fill;
                }
            }
            if files.len() % 1000 == 0 {
                println!("attack-fixes-r3: natural unlink_every={unlink_every} files={} seconds={:.0}", files.len(), started.elapsed().as_secs_f64());
            }
            if started.elapsed().as_secs_f64() > NATURAL_RUN_SECONDS_AT_MOST {
                fill_refusal = "预算到了".to_string();
                break 'fill;
            }
        }
    }
    println!("attack-fixes-r3: natural unlink_every={unlink_every} filled files={} refusal={fill_refusal} seconds={:.0}", files.len(), started.elapsed().as_secs_f64());
    natural_probe(&pool, &format!("unlink_every={unlink_every} after_the_fill"), &mut most_rounds);
    let mut kept: Vec<InodeNumber> = Vec::new();
    for (index, inode) in files.iter().enumerate() {
        if index % unlink_every != 0 {
            kept.push(*inode);
            continue;
        }
        pool.write_time_seconds += 1;
        let write_time_seconds = pool.write_time_seconds;
        if pool.publish(UserChange::Unlink { inode: *inode, write_time_seconds }, None).is_err() {
            kept.push(*inode);
        }
    }
    natural_probe(&pool, &format!("unlink_every={unlink_every} after_the_unlinks"), &mut most_rounds);
    let mut state = 88_172_645_463_325_252u64;
    let mut refused_in_a_row = 0;
    for overwrite in 0..kept.len() {
        if started.elapsed().as_secs_f64() > NATURAL_RUN_SECONDS_AT_MOST || refused_in_a_row >= 20 {
            println!("attack-fixes-r3: natural unlink_every={unlink_every} churn_stopped_at={overwrite} refused_in_a_row={refused_in_a_row}");
            break;
        }
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let inode = kept[usize::try_from(state % kept.len() as u64).expect("下标")];
        refused_in_a_row = if write_one_data_unit(&mut pool, inode).is_ok() { 0 } else { refused_in_a_row + 1 };
        if overwrite % 200 == 0 {
            natural_probe(&pool, &format!("unlink_every={unlink_every} churn={overwrite}"), &mut most_rounds);
        }
    }
    println!("attack-fixes-r3: natural_summary unlink_every={unlink_every} files={} kept={} most_rounds={most_rounds} seconds={:.0}", files.len(), kept.len(), started.elapsed().as_secs_f64());
}

#[test]
fn a_nearly_full_pool_reached_by_user_operations_alone_reports_its_rounds() {
    let threads: Vec<_> = [2usize, 8, 64]
        .into_iter()
        .map(|unlink_every| std::thread::spawn(move || natural_history(unlink_every)))
        .collect();
    for thread in threads {
        thread.join().expect("这段历史跑完");
    }
}
