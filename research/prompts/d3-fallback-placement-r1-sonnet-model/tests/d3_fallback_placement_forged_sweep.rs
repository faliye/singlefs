//! d3-fallback-placement-r1 正推腿模型（未打补丁的树与打了补丁的树各跑一遍，读数逐行对）：真发布路径上，提交内生块回落政策对固定点轮数的影响。
//! 拷自 `research/prompts/m4-close-fixes-code-r3-opus-model/tests/attack_fixes_r3_settling_fallback.rs` 的第一段（空闲图直接改的那一段），去掉第二段（用户操作写到盘满，
//! 那一段没走到这个形状）；改动只有三处：观测行前缀换成 `d3-fallback-placement:`，合取 7 的 `N₀` 压成 0 让发布走完（这样 `roles` 是真发布重写的角色数，
//! 写放大读它），不改政策。未打补丁的树这一遍是今天的政策（回落取该设备内槽号最小的空槽），打补丁的树这一遍是先取重写集的叶里的空槽。
//! 一片叶罩 812 槽，一个单元的两份落两块盘同一槽：取落点每碰到一片新叶，下一轮多重写两片叶（两块盘各一），又多要两个落点。
//! 探一次：分配器拷贝、盘拷贝上发一次覆盖写，读 `PoolWriter::rounds_of_the_last_allocation_record_tree_settling`。

mod common_admission_pool;

use std::cell::Cell;
use std::time::Instant;

use common_admission_pool::{content_of, plain_device, PoolUnderTest, OVERWRITE_BYTES};
use singlefs_core::address::{DeviceIdentity, FileOffsetInBytes, InodeNumber, SlotNumber};
use singlefs_core::allocator::{PoolAllocator, UnitAreaStart};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::compaction_plan::{CompactionSwitches, EmptyClusterSegmentsFloor};
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
    let product = CompactionSwitches::PRODUCT;
    allocator.set_compaction_switches(CompactionSwitches {
        batch_size: product.batch_size,
        forced_region: product.forced_region,
        run_forcing: product.run_forcing,
        region_exclusion: product.region_exclusion,
        destination_policy: product.destination_policy,
        release_checksum_mismatch_after_issue: product.release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor: EmptyClusterSegmentsFloor::ForcedForTests(0),
        relocation_publish_order: product.relocation_publish_order,
    });
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
            "d3-fallback-placement: fallback sweep={sweep} free_slots_per_leaf={} leaves={} prefix={} outcome={} rounds={} roles={} seconds={:.1}",
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
    println!("d3-fallback-placement: fallback estimate first_probe_rounds={} budget_seconds={FORGED_SWEEP_SECONDS_AT_MOST:.0}", first.rounds);
    // 每片两个空槽：扫前缀长度（前缀吃掉多少，头一次碰到稀疏那一段的差就是多少）。撞到上界三次就够，过了预算就停、打印停在哪。
    let mut hits = 0;
    for prefix in 1..=160u64 {
        if started.elapsed().as_secs_f64() > FORGED_SWEEP_SECONDS_AT_MOST / 2.0 || hits >= 3 {
            println!("d3-fallback-placement: fallback prefix_sweep_stopped_at={prefix} hits={hits} seconds={:.0}", started.elapsed().as_secs_f64());
            break;
        }
        let result = run_one(
            &pool,
            FreeShape { free_slots_per_leaf: 2, leaves: 120, slots_of_the_dense_prefix: prefix },
            "prefix",
        );
        hits += u32::from(result.did_not_settle);
    }
    // 两棵树逐点对读：形状扫的前缀固定取 14（攻方模型未打补丁时撞上界的前缀之一），不随各自的最多轮数走。
    let prefix = 14u64;
    // 轮数最多的那个前缀上扫每片空槽数与片数：F 大于 2 的衰减，片数少的走不满。
    for free_slots_per_leaf in [1u64, 2, 3, 4, 8, 32] {
        for leaves in [32u64, 64, 120, 200] {
            if started.elapsed().as_secs_f64() > FORGED_SWEEP_SECONDS_AT_MOST {
                println!("d3-fallback-placement: fallback shape_sweep_stopped seconds={:.0}", started.elapsed().as_secs_f64());
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
        "d3-fallback-placement: fallback_summary probes={} did_not_settle={} most_rounds={} prefix_of_the_most_rounds={} seconds={:.0}",
        probes.get(),
        did_not_settle.get(),
        most_rounds.get(),
        prefix_of_the_most_rounds.get(),
        started.elapsed().as_secs_f64()
    );
    assert!(probes.get() > 0);
}
