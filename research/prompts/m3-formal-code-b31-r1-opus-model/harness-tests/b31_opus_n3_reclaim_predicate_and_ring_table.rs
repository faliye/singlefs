//! 第 31 批代码轮第一轮攻方原型（只在攻方副本里编跑，不进仓）：mkfs 同一个进程那条会话（照 `common::build_pool` 搭，盘换成内存稀疏盘）
//! 里每一步之后核四样：
//! ① 分配器装着的根环表里的有效根（槽 → txg）与盘上根环读得出的根逐槽相同；
//! ② `overwrite_in_one_instance.rs` 新判法自己模拟的「环里最旧的根」与盘上根环最小的 txg 相同；
//! ③ 新判法的「已回收」谓词（已释放 ∧ 释放代 ≤ max(这一版根上的 F, 模拟的最旧根)）与 core 的真回收（已释放记录的槽在空闲图上空着）逐条相同；
//! ④ 每盘「已分配」= 不是已回收的记录跨度之和（新判法本身）；
//! 每隔几步（与最后一步）跑池级 checker，违例记下。场景甲照抄那条用例的会话（200 次 4100 字节覆盖写）；场景乙固定前缀（build_pool 会话），
//! 用户动作放开扫：覆盖写（几种长度）、抬 F（随机目标）、挂着时回退（随机环里的根）。
//! 环境变量 B31_SEEDS（场景乙的种子数，默认 8）、B31_STEPS（每段步数，默认 60）、B31_CHECKER_EVERY（每几步跑一次 checker，默认 1）、
//! B31_SEED_BASE（种子起点，默认 0）。

mod common;

use std::collections::BTreeMap;

use common::{
    memory_pool_of_sparse_devices, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT,
    TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{
    raise_rollback_floor, roll_back_by_a_forward_publish, RollbackTarget, ShadowLedger,
};
use singlefs_core::recovery::{choose_system_configuration, readable_roots};
use singlefs_core::root_ring::{target_for_publish, RootRingSlot};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolWriter,
    TransactionOutput,
};
use singlefs_format::{ROOT_RING_REGIONS, SLOT_BYTES};
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

type Device = RecordingBlockDevice<SparseBlockDevice>;

struct Session {
    devices: Vec<(DeviceIdentity, Device)>,
    allocator: PoolAllocator,
    output: TransactionOutput,
    genesis_txg: CheckpointTxg,
}

/// 照 `common::build_pool`（第 31 批之后那一份）搭，只把文件镜像换成内存稀疏盘。
fn sparse_build_pool() -> Session {
    sparse_build_pool_on(SharedStream::new())
}

/// 同 [`sparse_build_pool`]，录进调用方给的流（场景丙要留内容的流）。
fn sparse_build_pool_on(stream: SharedStream) -> Session {
    let parameters = parameters();
    let mut devices: Vec<(DeviceIdentity, Device)> = (0..2u32)
        .map(|number| {
            let identity = DeviceIdentity(number);
            (
                identity,
                RecordingBlockDevice::with_shared_stream(
                    identity,
                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    );
    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
        &parameters,
        &genesis,
    ));
    let content = common::file_content();
    let output = {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut pool).expect("取号");
        let warmed = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
        publish_first_file(
            &mut pool,
            &mut allocator,
            warmed.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("第一个文件")
    };
    Session {
        devices,
        allocator,
        output,
        genesis_txg: genesis.root.checkpoint_txg,
    }
}

/// 照抄 `overwrite_in_one_instance.rs` 新判法里的模拟（第 31 批 31-3 那一份），一字不改，只把几何取法换成本文件的 `parameters()`。
fn oldest_root_in_the_ring_after_this_process_wrote_every_txg_up_to(
    newest_txg: CheckpointTxg,
    genesis_txg: CheckpointTxg,
) -> CheckpointTxg {
    let slots_per_region = parameters().geometry.root_ring_slots_per_region;
    let mut txg_of_each_ring_slot: BTreeMap<RootRingSlot, CheckpointTxg> = (0..ROOT_RING_REGIONS)
        .map(|region| (RootRingSlot { region, slot: 0 }, genesis_txg))
        .collect();
    for written_txg in genesis_txg.0 + 1..=newest_txg.0 {
        txg_of_each_ring_slot.insert(
            target_for_publish(CheckpointTxg(written_txg), slots_per_region),
            CheckpointTxg(written_txg),
        );
    }
    *txg_of_each_ring_slot
        .values()
        .min()
        .expect("环里至少有 mkfs 写的那几槽")
}

/// 从 `RootRingOccupancy` 的 `{:?}` 里取出每一槽的占用：有效根交回 (槽 → txg)，别的种类只数名字。
fn table_of(allocator: &PoolAllocator) -> (BTreeMap<RootRingSlot, u64>, Vec<String>) {
    let text = format!(
        "{:?}",
        allocator.root_ring_occupancy().expect("build_pool 一族装着根环表")
    );
    let mut valid = BTreeMap::new();
    let mut others = Vec::new();
    let marker = "RootRingSlot { region: ";
    let mut rest = text.as_str();
    // 迭代上界：表里每一槽一次。
    while let Some(start) = rest.find(marker) {
        rest = &rest[start + marker.len()..];
        let region_end = rest.find(',').expect("区域号后面是逗号");
        let region: u64 = rest[..region_end].trim().parse().expect("区域号");
        rest = &rest[region_end..];
        let slot_start = rest.find("slot: ").expect("槽号") + "slot: ".len();
        rest = &rest[slot_start..];
        let slot_end = rest.find(' ').expect("槽号后面是空格");
        let slot: u64 = rest[..slot_end].parse().expect("槽号");
        rest = &rest[slot_end..];
        let kind_start = rest.find("}: ").expect("槽之后是占用") + "}: ".len();
        rest = &rest[kind_start..];
        let kind_end = rest
            .find(|character: char| character == ' ' || character == ',' || character == '}' || character == '(')
            .expect("占用种类");
        let kind = &rest[..kind_end];
        if kind == "ValidRoot" {
            let txg_start = rest.find("CheckpointTxg(").expect("txg") + "CheckpointTxg(".len();
            let txg_end = txg_start + rest[txg_start..].find(')').expect("txg 收尾");
            let txg: u64 = rest[txg_start..txg_end].parse().expect("txg");
            valid.insert(RootRingSlot { region, slot }, txg);
        } else {
            others.push(kind.to_string());
        }
    }
    (valid, others)
}

#[derive(Default, Debug)]
struct Tally {
    steps: u64,
    steps_with_reclaimed_records: u64,
    reclaimed_records_seen: u64,
    released_records_seen: u64,
    checker_runs: u64,
    highest_txg: u64,
    applied: BTreeMap<&'static str, u64>,
    refused: BTreeMap<String, u64>,
    findings: Vec<String>,
}

/// 一步之后的四样核对与（按 `run_checker`）池级 checker。
fn check_step(session: &Session, label: &str, run_checker: bool, tally: &mut Tally) {
    tally.steps += 1;
    let output = &session.output;
    tally.highest_txg = tally.highest_txg.max(output.root.checkpoint_txg.0);
    let image = memory_pool_of_sparse_devices(&session.devices);
    let system_configuration = choose_system_configuration(&image).expect("系统配置");
    let slots_per_region = system_configuration
        .immutable
        .sizes
        .root_ring_slots_per_region;
    let roots = readable_roots(
        &image,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );
    let mut on_disk: BTreeMap<RootRingSlot, u64> = BTreeMap::new();
    for root in &roots {
        let ring_slot = target_for_publish(root.checkpoint_txg, slots_per_region);
        let entry = on_disk.entry(ring_slot).or_insert(root.checkpoint_txg.0);
        *entry = (*entry).min(root.checkpoint_txg.0);
    }
    let (table, others) = table_of(&session.allocator);
    if !others.is_empty() {
        tally
            .findings
            .push(format!("{label}：根环表里有有效根以外的占用 {others:?}"));
    }
    if table != on_disk {
        tally.findings.push(format!(
            "{label}：根环表与盘上根环不同：表 {table:?}，盘 {on_disk:?}"
        ));
    }
    let disk_oldest = on_disk.values().min().copied().expect("盘上至少一条根");
    let simulated_oldest = oldest_root_in_the_ring_after_this_process_wrote_every_txg_up_to(
        output.root.checkpoint_txg,
        session.genesis_txg,
    );
    if simulated_oldest.0 != disk_oldest {
        tally.findings.push(format!(
            "{label}：新判法模拟的最旧根 {} ≠ 盘上根环最小 txg {disk_oldest}",
            simulated_oldest.0
        ));
    }
    let floor = output.root.rollback_floor.max(simulated_oldest);
    let mut any_reclaimed = false;
    for device_map in &session.allocator.devices {
        let mut unreclaimed_span: u64 = 0;
        for record in output
            .allocation_records
            .iter()
            .filter(|record| record.device == device_map.device)
        {
            let predicate_says_reclaimed = record.is_released && record.generation <= floor;
            if !predicate_says_reclaimed {
                unreclaimed_span += u64::from(record.span_slots);
            }
            if record.is_released {
                tally.released_records_seen += 1;
                let free_in_core = (0..u64::from(record.span_slots))
                    .all(|offset| device_map.is_free(SlotNumber(record.slot.0 + offset)));
                if free_in_core {
                    tally.reclaimed_records_seen += 1;
                    any_reclaimed = true;
                }
                if free_in_core != predicate_says_reclaimed {
                    tally.findings.push(format!(
                        "{label}：盘 {} 槽 {} 释放代 {}：新判法说已回收 = {predicate_says_reclaimed}，core 空闲图上空着 = {free_in_core}（门槛 {}）",
                        record.device.0, record.slot.0, record.generation.0, floor.0
                    ));
                }
            }
        }
        if device_map.allocated_slots() != unreclaimed_span {
            tally.findings.push(format!(
                "{label}：盘 {} 已分配 {} 槽 ≠ 不是已回收的记录跨度之和 {} 槽（{} 字节一槽）",
                device_map.device.0,
                device_map.allocated_slots(),
                unreclaimed_span,
                SLOT_BYTES
            ));
        }
    }
    if any_reclaimed {
        tally.steps_with_reclaimed_records += 1;
    }
    if run_checker {
        tally.checker_runs += 1;
        for (invariant, verdict) in check_pool_image(&image) {
            if matches!(verdict, InvariantVerdict::Violated(_)) {
                tally
                    .findings
                    .push(format!("{label}：checker {invariant} {verdict:?}"));
            }
        }
    }
}

fn content_of(length: usize, seed: u64) -> Vec<u8> {
    (0..length)
        .map(|index| {
            u8::try_from((index as u64 * 7 + 3 + seed * 13) % 253).expect("小于 256")
        })
        .collect()
}

fn environment_number(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn overwrite(session: &mut Session, content: &[u8]) -> Result<(), String> {
    let parameters = parameters();
    let previous = session.output.clone();
    let mut writer = PoolWriter::new(&parameters, session.devices.as_mut_slice());
    let output = publish_overwrite(
        &mut writer,
        &mut session.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        InstanceGeneration(1),
    )
    .map_err(|error| format!("{error:?}"))?;
    session.output = output;
    Ok(())
}

fn report(name: &str, tally: &Tally) {
    println!(
        "name={name} steps={} steps_with_reclaimed_records={} released_records_seen={} reclaimed_records_seen={} checker_runs={} highest_txg={} applied={:?} refused={:?} findings={}",
        tally.steps,
        tally.steps_with_reclaimed_records,
        tally.released_records_seen,
        tally.reclaimed_records_seen,
        tally.checker_runs,
        tally.highest_txg,
        tally.applied,
        tally.refused,
        tally.findings.len()
    );
    for finding in tally.findings.iter().take(40) {
        println!("finding {name}: {finding}");
    }
}

/// 场景甲：照抄 `allocated_statistic_equals_the_span_sum_of_the_unreclaimed_allocation_records_past_six_hundred_records` 的会话
/// （同一份 4100 字节内容覆盖写 200 次），每一步核四样。
#[test]
fn scenario_a_the_session_of_the_812_record_case() {
    let checker_every = environment_number("B31_CHECKER_EVERY", 1).max(1);
    let mut session = sparse_build_pool();
    let mut tally = Tally::default();
    check_step(&session, "甲 第一个文件", true, &mut tally);
    let second_content = content_of(4100, 0);
    for index in 1..=200u64 {
        overwrite(&mut session, &second_content).expect("覆盖写");
        *tally.applied.entry("overwrite").or_insert(0) += 1;
        let label = format!(
            "甲 第 {index} 次覆盖写（txg {}，{} 条记录）",
            session.output.root.checkpoint_txg.0,
            session.output.allocation_records.len()
        );
        check_step(&session, &label, index % checker_every == 0 || index == 200, &mut tally);
    }
    report("scenario_a", &tally);
    assert!(tally.findings.is_empty(), "场景甲有发现，见上");
}

struct SplitMix64(u64);

impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut mixed = self.0;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        mixed ^ (mixed >> 31)
    }
}

const LENGTHS: [usize; 6] = [0, 1, 1000, 4100, 9000, 16000];

/// 场景乙：前缀固定在 build_pool 会话（第一个文件之后、一次挂载都不做），之后的用户动作按种子抽：覆盖写（六种长度）、抬 F（随机目标）、
/// 挂着时回退（随机挑盘上环里的一条根）。每一步之后核四样与 checker。
#[test]
fn scenario_b_user_actions_swept_after_the_build_pool_prefix() {
    let seeds = environment_number("B31_SEEDS", 8);
    let steps = environment_number("B31_STEPS", 60);
    let seed_base = environment_number("B31_SEED_BASE", 0);
    let checker_every = environment_number("B31_CHECKER_EVERY", 1).max(1);
    let mut total = Tally::default();
    for seed in seed_base..seed_base + seeds {
        let mut random = SplitMix64(seed);
        let mut session = sparse_build_pool();
        check_step(&session, &format!("乙 种子 {seed} 第一个文件"), true, &mut total);
        for step in 1..=steps {
            let choice = random.next() % 10;
            let current_txg = session.output.root.checkpoint_txg.0;
            let current_floor = session.output.root.rollback_floor.0;
            let (kind, outcome): (&'static str, Result<(), String>) = if choice < 6 {
                let length = LENGTHS[usize::try_from(random.next() % 6).expect("小于 6")];
                ("overwrite", overwrite(&mut session, &content_of(length, random.next() % 97)))
            } else if choice < 8 {
                let target = current_floor + random.next() % (current_txg - current_floor + 3);
                let mut current = session.output.clone();
                let raised = raise_rollback_floor(
                    &parameters(),
                    &mut session.devices,
                    &mut session.allocator,
                    &mut current,
                    CheckpointTxg(target),
                    ShadowLedger::On,
                );
                session.output = current;
                ("raise_floor", raised.map(|_| ()).map_err(|error| format!("{error:?}")))
            } else {
                let image = memory_pool_of_sparse_devices(&session.devices);
                let system_configuration = choose_system_configuration(&image).expect("系统配置");
                let mut ring_txgs: Vec<u64> = readable_roots(
                    &image,
                    &system_configuration.immutable.region_devices,
                    &system_configuration.immutable.sizes,
                    &system_configuration.immutable.filesystem_identifier,
                )
                .iter()
                .map(|root| root.checkpoint_txg.0)
                .collect();
                ring_txgs.sort_unstable();
                ring_txgs.dedup();
                let target = ring_txgs[usize::try_from(random.next() % ring_txgs.len() as u64).expect("下标")];
                let rolled = roll_back_by_a_forward_publish(
                    &parameters(),
                    &mut session.devices,
                    &mut session.allocator,
                    &mut session.output,
                    RollbackTarget {
                        instance: InstanceGeneration(1),
                        checkpoint_txg: CheckpointTxg(target),
                    },
                );
                ("roll_back", rolled.map(|_| ()).map_err(|error| format!("{error:?}")))
            };
            match outcome {
                Ok(()) => *total.applied.entry(kind).or_insert(0) += 1,
                Err(error) => {
                    let member: String = error.chars().take_while(|character| *character != '{' && *character != '(').collect();
                    *total.refused.entry(format!("{kind}:{}", member.trim())).or_insert(0) += 1;
                }
            }
            let label = format!("乙 种子 {seed} 第 {step} 步 {kind}（txg {}）", session.output.root.checkpoint_txg.0);
            check_step(&session, &label, step % checker_every == 0 || step == steps, &mut total);
        }
    }
    report("scenario_b", &total);
    assert!(total.findings.is_empty(), "场景乙有发现，见上");
}

/// 场景丙（回收时机提前那一类错法，坏实现 R4 用）：build_pool 会话里逐次覆盖写，每一次取「这次发布第一道屏障之前的写全落、之后一个不落」
/// 那一个崩溃态（发布前的镜像 + 这次发布第一道屏障之前的写），跑池级 checker、数违例；同时按层 0 的切段数这次发布的闭式状态数，
/// 只报数、不全量枚举（每次都远超 10⁶，见报告）。不断言，只打数：基线与坏实现各跑一遍比。
#[test]
fn scenario_c_unit_writes_persisted_before_the_first_barrier_of_each_overwrite() {
    use singlefs_harness::segments::{closed_form_state_count, split_into_segments};
    use singlefs_harness::{RecordedOperation, RecordedOperationKind};
    let steps = environment_number("B31_C_STEPS", 60);
    let stream = SharedStream::retaining_contents();
    let mut session = sparse_build_pool_on(stream.clone());
    let geometry = common::geometry();
    let mut violated_states = 0u64;
    let mut by_invariant: BTreeMap<&'static str, u64> = BTreeMap::new();
    let mut first_violation: Option<String> = None;
    let mut smallest_closed_form = u64::MAX;
    for index in 1..=steps {
        let before_image = memory_pool_of_sparse_devices(&session.devices);
        let before = stream.operation_count();
        overwrite(&mut session, &content_of(4100, index)).expect("覆盖写");
        let operations = stream.retained_operations();
        let this_publish = &operations[before..];
        let recorded: Vec<RecordedOperation> =
            this_publish.iter().map(|retained| retained.operation.clone()).collect();
        smallest_closed_form =
            smallest_closed_form.min(closed_form_state_count(&split_into_segments(&recorded, &geometry)));
        let cut = this_publish
            .iter()
            .position(|retained| retained.operation.kind == RecordedOperationKind::Barrier)
            .unwrap_or(this_publish.len());
        let mut crashed = before_image.clone();
        crashed.apply(&this_publish[..cut]);
        let mut this_state_violated = false;
        for (invariant, verdict) in check_pool_image(&crashed) {
            if let InvariantVerdict::Violated(reason) = verdict {
                this_state_violated = true;
                *by_invariant.entry(invariant).or_insert(0) += 1;
                if first_violation.is_none() {
                    first_violation = Some(format!(
                        "txg {}（第 {index} 次覆盖写，切在第 {cut} 步）{invariant}：{reason}",
                        session.output.root.checkpoint_txg.0
                    ));
                }
            }
        }
        if this_state_violated {
            violated_states += 1;
        }
    }
    println!(
        "name=scenario_c steps={steps} last_txg={} violated_states={violated_states} by_invariant={by_invariant:?} smallest_closed_form_state_count_of_one_publish={smallest_closed_form}",
        session.output.root.checkpoint_txg.0
    );
    println!("first_violation={}", first_violation.unwrap_or_else(|| "none".to_string()));
}
