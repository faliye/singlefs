//! 攻方原型（m3-batch52-54-code-r1 opus）：K2（I-9.36、I-9.37）与 K4（叶形状参照的形状覆盖）。只用内存稀疏盘，不建镜像文件。

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::{
    check_pool_image, check_pool_image_given_the_container_capacity_knobs, ContainerCapacityKnobs,
    LineageContainerCapacityKnob, RetirementContainerCapacityKnob, TombstoneContainerCapacityKnob,
};
use singlefs_core::address::{DeviceIdentity, InodeNumber, InstanceGeneration};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::container_tree::{
    ContainerTreeCapacities, ContainerTreeNodeEntries, ContainerTreeNodeEntriesCappedForTests,
};
use singlefs_core::lineage_tree::{LineageContainerCapacity, LineageRecordsPerContainerCappedForTests};
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::mount::{roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::retirement_tree::{
    RetirementContainerCapacity, RetirementRecordsPerContainerCappedForTests,
};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::tombstone_tree::{
    TombstoneContainerCapacity, TombstoneRecordsPerContainerCappedForTests,
};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_new_inodes, publish_unlink_intent,
    publish_unlink_steps_until_the_queue_is_empty, warm_up, FirstFile, PoolWriter,
    TransactionOutput,
};
use singlefs_core::unlink::{DeletionSwitches, UnlinkBatchSize};
use singlefs_format::{INODE_LEAF_RECORDS, JOURNAL_RING_DEFAULT_BYTES};
use singlefs_harness::history::SeededRandomSource;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;
const DEVICE_BYTES: u64 = 4 << 30;
const WRITE_TIME_SECONDS: u64 = 1_788_300_000;
const INSTANCE: InstanceGeneration = InstanceGeneration(1);

fn parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: *b"singlefs-opus-k2",
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(512),
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

/// 内存稀疏盘上的池：mkfs 与第一个文件只做一次，之后每段历史从它 `clone`。
struct Pool {
    devices: Devices,
    allocator: PoolAllocator,
    output: TransactionOutput,
}

impl Clone for Pool {
    fn clone(&self) -> Self {
        Self {
            devices: self
                .devices
                .iter()
                .map(|(identity, device)| {
                    let mut copy = SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512));
                    copy.image = device.image.clone();
                    (*identity, copy)
                })
                .collect(),
            allocator: self.allocator.clone(),
            output: self.output.clone(),
        }
    }
}

impl Pool {
    fn after_the_first_file() -> Self {
        let parameters = parameters();
        let mut devices: Devices = [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| (identity, SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512))))
            .collect();
        let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
        let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
        let content: Vec<u8> = (0..3000u32).map(|index| u8::try_from(index % 251).expect("小于 256")).collect();
        let output = {
            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
            let instance = acquire_instance(&mut writer).expect("取号");
            let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
            publish_first_file(
                &mut writer,
                &mut allocator,
                warmed.roots.last().expect("暖机两条根"),
                FirstFile { content: &content, write_time_seconds: WRITE_TIME_SECONDS },
                instance,
                &warmed.last_record_bytes,
            )
            .expect("第一个文件")
        };
        Self { devices, allocator, output }
    }

    fn image(&self) -> MemoryPool {
        MemoryPool {
            devices: self.devices.iter().map(|(identity, device)| (*identity, device.image.clone())).collect(),
            device_size_in_bytes: DEVICE_BYTES,
        }
    }

    fn create(&mut self, count: u64) -> TransactionOutput {
        let parameters = parameters();
        let mut writer = PoolWriter::new(&parameters, self.devices.as_mut_slice());
        let output = publish_new_inodes(&mut writer, &mut self.allocator, &self.output, count, WRITE_TIME_SECONDS, INSTANCE)
            .expect("建空 inode");
        self.output = output.clone();
        output
    }

    fn unlink(&mut self, inode: u64, capacities: ContainerTreeCapacities) {
        let parameters = parameters();
        let mut writer = PoolWriter::new(&parameters, self.devices.as_mut_slice());
        writer.set_container_tree_capacities(capacities);
        let mut current = publish_unlink_intent(&mut writer, &mut self.allocator, &self.output, InodeNumber(inode), WRITE_TIME_SECONDS, INSTANCE)
            .unwrap_or_else(|refusal| panic!("unlink {inode} 意图：{refusal:?}"));
        publish_unlink_steps_until_the_queue_is_empty(&mut writer, &mut self.allocator, &mut current, UnlinkBatchSize::OneExtentLowerLeaf, INSTANCE)
            .unwrap_or_else(|refusal| panic!("unlink {inode} 各批：{refusal:?}"));
        self.output = current;
    }

    fn roll_back(&mut self, target: &TransactionOutput) -> Result<(), String> {
        let parameters = parameters();
        let before = self.clone();
        match roll_back_by_a_forward_publish(
            &parameters,
            &mut self.devices,
            &mut self.allocator,
            &mut self.output,
            RollbackTarget { instance: target.root.instance, checkpoint_txg: target.root.checkpoint_txg },
        ) {
            Ok(_) => Ok(()),
            Err(refusal) => {
                *self = before;
                Err(format!("{refusal:?}"))
            }
        }
    }

    fn live_inodes(&self) -> Vec<u64> {
        self.output
            .inode_leaf_containers
            .iter()
            .flat_map(|leaf| leaf.contents.records.iter().map(|record| record.inode))
            .collect()
    }

    fn tree_summary(&self) -> String {
        let numbers = |identities: Vec<(u64, usize)>| format!("{identities:?}");
        format!(
            "tombstone={} lineage={} retirement={} levels(t,l,r)=({},{},{}) leaves={:?}",
            numbers(self.output.tombstone_containers.iter().map(|c| (c.contents.identity.container, c.contents.records.len())).collect()),
            numbers(self.output.lineage_containers.iter().map(|c| (c.contents.identity.container, c.contents.records.len())).collect()),
            numbers(self.output.retirement_containers.iter().map(|c| (c.contents.identity.container, c.contents.records.len())).collect()),
            self.output.tombstone_tree_index_nodes.shape.root_level(),
            self.output.lineage_tree_index_nodes.shape.root_level(),
            self.output.retirement_tree_index_nodes.shape.root_level(),
            self.output.inode_leaf_containers.iter().map(|leaf| leaf.contents.records.len()).collect::<Vec<_>>(),
        )
    }
}

/// 一组旋钮：写入口与分配器开关同一份，池级 checker 的新入口给同一组值（0 = 产品容量、不给旋钮）。
#[derive(Clone, Copy, Debug)]
struct Knobs {
    tombstone: u64,
    lineage: u64,
    retirement: u64,
    node_entries: usize,
}

impl Knobs {
    fn capacities(self) -> ContainerTreeCapacities {
        ContainerTreeCapacities {
            tombstone_container: match self.tombstone {
                0 => TombstoneContainerCapacity::FromTheContainerFormat,
                n => TombstoneContainerCapacity::CappedForTests(TombstoneRecordsPerContainerCappedForTests::new(n).expect("1..=815")),
            },
            lineage_container: match self.lineage {
                0 => LineageContainerCapacity::FromTheContainerFormat,
                n => LineageContainerCapacity::CappedForTests(LineageRecordsPerContainerCappedForTests::new(n).expect("1..=1359")),
            },
            retirement_container: match self.retirement {
                0 => RetirementContainerCapacity::FromTheContainerFormat,
                n => RetirementContainerCapacity::CappedForTests(RetirementRecordsPerContainerCappedForTests::new(n).expect("1..=959")),
            },
            node_entries: match self.node_entries {
                0 => ContainerTreeNodeEntries::FromTheNodeFormat,
                n => ContainerTreeNodeEntries::CappedForTests(ContainerTreeNodeEntriesCappedForTests::new(n).expect("2..=135")),
            },
        }
    }

    fn switches(self) -> DeletionSwitches {
        let capacities = self.capacities();
        DeletionSwitches {
            batch_size: UnlinkBatchSize::OneExtentLowerLeaf,
            tombstone_container_capacity: capacities.tombstone_container,
            lineage_container_capacity: capacities.lineage_container,
            retirement_container_capacity: capacities.retirement_container,
            container_tree_node_entries: capacities.node_entries,
        }
    }

    fn checker_knobs(self) -> ContainerCapacityKnobs {
        ContainerCapacityKnobs {
            tombstone: (self.tombstone > 0).then(|| TombstoneContainerCapacityKnob::new(self.tombstone).expect("1..=815")),
            lineage: (self.lineage > 0).then(|| LineageContainerCapacityKnob::new(self.lineage).expect("1..=1359")),
            retirement: (self.retirement > 0).then(|| RetirementContainerCapacityKnob::new(self.retirement).expect("1..=959")),
        }
    }
}

fn violations(verdicts: &[(&'static str, InvariantVerdict)]) -> Vec<String> {
    verdicts
        .iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some(format!("{invariant}: {detail}")),
            _ => None,
        })
        .collect()
}

fn verdict_name(verdicts: &[(&'static str, InvariantVerdict)], invariant: &str) -> String {
    let verdict = &verdicts.iter().find(|(name, _)| *name == invariant).expect("清单里有").1;
    match verdict {
        InvariantVerdict::Holds => "Holds".to_string(),
        InvariantVerdict::Violated(_) => "Violated".to_string(),
        InvariantVerdict::NotApplicable(reason) => format!("NotApplicable({reason})"),
        InvariantVerdict::PartiallyJudged(reason) => format!("PartiallyJudged({reason})"),
    }
}

/// K2.2：三棵容器树各写两片（旋钮一片 1 条），打印容器号与两个入口的违例。打变异的副本上容器号不连 / 不从 1 起 / 多一片空容器，
/// 看池级 checker 红不红。
#[test]
fn opus_probe_k2_container_numbers_under_the_knobs() {
    let knobs = Knobs { tombstone: 1, lineage: 1, retirement: 1, node_entries: 0 };
    let mut pool = Pool::after_the_first_file();
    let first_file_version = pool.output.clone();
    pool.allocator.set_deletion_switches(knobs.switches());
    pool.create(u64::from(INODE_LEAF_RECORDS)); // 2..=234：叶 [1..=233]、[234]
    pool.unlink(234, knobs.capacities()); // 最右那片删空：退役一条、墓碑一条
    pool.create(1); // 235 开新叶
    pool.unlink(235, knobs.capacities()); // 又退役一条、墓碑一条
    pool.roll_back(&first_file_version).expect("回退一次");
    pool.roll_back(&first_file_version).expect("回退两次");
    let image = pool.image();
    let with_knobs = check_pool_image_given_the_container_capacity_knobs(&image, knobs.checker_knobs());
    let product = check_pool_image(&image);
    eprintln!("OPUS_K2_NUMBERS trees {}", pool.tree_summary());
    eprintln!("OPUS_K2_NUMBERS knob_entry_violations={:?}", violations(&with_knobs));
    eprintln!("OPUS_K2_NUMBERS knob_entry I-9.37={} I-9.22={}", verdict_name(&with_knobs, "I-9.37"), verdict_name(&with_knobs, "I-9.22"));
    eprintln!("OPUS_K2_NUMBERS product_entry_violated={:?}", violations(&product).iter().map(|line| line.split(':').next().unwrap_or("").to_string()).collect::<Vec<_>>());
}

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|value| value.parse().ok()).unwrap_or(default)
}

/// K2.1：随机历史（建、unlink、挂着回退）在三棵容器树都装了旋钮的池上，每一步之后带同一组旋钮值交池级 checker 的新入口，
/// 合法镜像上一条都不许红。用户动作（建几个、删哪个、退到哪一版）全放开，种子 `OPUS_K2_SEED_FIRST..=OPUS_K2_SEED_LAST`，每个种子 `OPUS_K2_OPS` 步。
#[test]
fn opus_probe_k2_random_knob_histories() {
    let first_seed = env_u64("OPUS_K2_SEED_FIRST", 1);
    let last_seed = env_u64("OPUS_K2_SEED_LAST", 2);
    let operations = env_u64("OPUS_K2_OPS", 20);
    let base = Pool::after_the_first_file();
    let mut violating_checks_overall = 0u64;
    // 迭代上界：种子个数。
    for seed in first_seed..=last_seed {
        let started = std::time::Instant::now();
        let mut random = SeededRandomSource::from_seed(seed);
        let knobs = Knobs {
            tombstone: 1 + random.below(3),
            lineage: 1 + random.below(2),
            retirement: 1 + random.below(2),
            node_entries: [0usize, 2, 3][usize::try_from(random.below(3)).expect("小")],
        };
        let mut pool = base.clone();
        pool.allocator.set_deletion_switches(knobs.switches());
        pool.create(u64::from(INODE_LEAF_RECORDS) + 1 + random.below(8));
        let mut versions: Vec<TransactionOutput> = vec![pool.output.clone()];
        let (mut creates, mut unlinks, mut rollbacks_ok, mut rollbacks_refused, mut checks, mut violating_checks) = (0, 0, 0, 0, 0, 0);
        let mut first_violation = String::from("none");
        let mut i937 = std::collections::BTreeMap::<String, u64>::new();
        let mut i936 = std::collections::BTreeMap::<String, u64>::new();
        // 迭代上界：OPUS_K2_OPS。
        for operation in 0..operations {
            let choice = random.below(100);
            let live: Vec<u64> = pool.live_inodes().into_iter().filter(|inode| *inode != 1).collect();
            if choice < 25 || live.is_empty() {
                pool.create(1 + random.below(3));
                creates += 1;
            } else if choice < 85 {
                // 25 从记录最少的那片叶里删、25 删最大的活号（最右那片叶删空就退役）、10 随便删一个活着的号：逼出删空与吸收。
                let inode = if choice < 50 {
                    let smallest = pool
                        .output
                        .inode_leaf_containers
                        .iter()
                        .filter(|leaf| leaf.contents.records.iter().any(|record| record.inode != 1))
                        .min_by_key(|leaf| leaf.contents.records.len())
                        .expect("有活着的号就有这样一片");
                    let candidates: Vec<u64> = smallest.contents.records.iter().map(|record| record.inode).filter(|inode| *inode != 1).collect();
                    candidates[usize::try_from(random.below(u64::try_from(candidates.len()).expect("小"))).expect("小")]
                } else if choice < 75 {
                    *live.iter().max().expect("非空")
                } else {
                    live[usize::try_from(random.below(u64::try_from(live.len()).expect("小"))).expect("小")]
                };
                pool.unlink(inode, knobs.capacities());
                unlinks += 1;
            } else {
                let candidates: Vec<&TransactionOutput> = versions
                    .iter()
                    .filter(|version| version.root.checkpoint_txg != pool.output.root.checkpoint_txg)
                    .collect();
                if candidates.is_empty() {
                    continue;
                }
                let target = candidates[usize::try_from(random.below(u64::try_from(candidates.len()).expect("小"))).expect("小")].clone();
                match pool.roll_back(&target) {
                    Ok(()) => rollbacks_ok += 1,
                    Err(_) => rollbacks_refused += 1,
                }
            }
            versions.push(pool.output.clone());
            if versions.len() > 12 {
                versions.remove(0);
            }
            let verdicts = check_pool_image_given_the_container_capacity_knobs(&pool.image(), knobs.checker_knobs());
            checks += 1;
            *i937.entry(verdict_name(&verdicts, "I-9.37")).or_default() += 1;
            *i936.entry(verdict_name(&verdicts, "I-9.36")).or_default() += 1;
            let found = violations(&verdicts);
            if !found.is_empty() {
                violating_checks += 1;
                if first_violation == "none" {
                    first_violation = format!("op={operation} {found:?} trees {}", pool.tree_summary());
                }
            }
        }
        violating_checks_overall += violating_checks;
        eprintln!(
            "OPUS_K2_RANDOM seed={seed} knobs={knobs:?} ops={operations} creates={creates} unlinks={unlinks} rollbacks_ok={rollbacks_ok} rollbacks_refused={rollbacks_refused} checks={checks} violating_checks={violating_checks} i937={i937:?} i936={i936:?} final {} first_violation={first_violation} elapsed_seconds={:.1}",
            pool.tree_summary(),
            started.elapsed().as_secs_f64()
        );
    }
    assert_eq!(violating_checks_overall, 0, "旋钮下的合法镜像带同一组旋钮值交新入口，一条都不许红");
}

/// K2.1 产品容量的边：产品容量下退役 `OPUS_K2_RETIREMENTS` 片叶（默认 960，退役树第二片容器开出来），产品入口判 I-9.36。
#[test]
#[ignore = "原型：挂钟长，单独点名跑"]
fn opus_probe_k2_product_boundary_retirement() {
    let retirements = env_u64("OPUS_K2_RETIREMENTS", 960);
    let started = std::time::Instant::now();
    let mut pool = Pool::after_the_first_file();
    pool.create(u64::from(INODE_LEAF_RECORDS) - 1); // 2..=233：叶 [1..=233] 满
    // 迭代上界：OPUS_K2_RETIREMENTS。每一轮建一个号开新叶、再删到底，那片叶删空退役一条。
    for _ in 0..retirements {
        let created = pool.create(1);
        let inode = created.inode_number_watermark() - 1;
        pool.unlink(inode, ContainerTreeCapacities::FROM_THE_FORMAT);
    }
    let verdicts = check_pool_image(&pool.image());
    eprintln!(
        "OPUS_K2_PRODUCT_RETIREMENT retirements={retirements} retirement_counts={:?} I-9.36={} violations={:?} elapsed_seconds={:.1}",
        pool.output.retirement_containers.iter().map(|c| c.contents.records.len()).collect::<Vec<_>>(),
        verdict_name(&verdicts, "I-9.36"),
        violations(&verdicts),
        started.elapsed().as_secs_f64()
    );
}

/// K2.1 产品容量的边：产品容量下挂着回退 `OPUS_K2_ROLLBACKS` 次（默认 1360，谱系树第二片容器开出来），产品入口判 I-9.36。
#[test]
#[ignore = "原型：挂钟长，单独点名跑"]
fn opus_probe_k2_product_boundary_lineage() {
    let rollbacks = env_u64("OPUS_K2_ROLLBACKS", 1360);
    let started = std::time::Instant::now();
    let mut pool = Pool::after_the_first_file();
    let mut previous = pool.output.clone();
    pool.create(1);
    // 迭代上界：OPUS_K2_ROLLBACKS。每一次退到紧前那一版（根环转过去之后第一个文件那一版退不到）。
    for round in 0..rollbacks {
        let target = previous.clone();
        previous = pool.output.clone();
        pool.roll_back(&target).unwrap_or_else(|refusal| panic!("第 {round} 次回退：{refusal}"));
    }
    let verdicts = check_pool_image(&pool.image());
    eprintln!(
        "OPUS_K2_PRODUCT_LINEAGE rollbacks={rollbacks} lineage_counts={:?} I-9.36={} violations={:?} elapsed_seconds={:.1}",
        pool.output.lineage_containers.iter().map(|c| c.contents.records.len()).collect::<Vec<_>>(),
        verdict_name(&verdicts, "I-9.36"),
        violations(&verdicts),
        started.elapsed().as_secs_f64()
    );
}

// ─── K4：叶形状参照（照 `inode_leaf_merge_and_borrow_against_an_independent_reference.rs` 的参照另抄一份，加形状计数）───

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ReferenceLeafIdentity {
    birth_tree: u64,
    record_type: u16,
    container: u64,
    container_birth: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ReferenceLeaf {
    identity: ReferenceLeafIdentity,
    inode_numbers: Vec<u64>,
}

#[derive(Default, Debug)]
struct ShapeCounts {
    removes: u64,
    drops: u64,
    drop_then_absorb: u64,
    right_absorbs: u64,
    left_absorbs: u64,
    removes_with_two_or_more_absorbs: u64,
    both_pairs_fit: u64,
    both_pairs_fit_outside_the_first_step: u64,
    borrows: u64,
    borrow_then_absorb: u64,
    borrow_skipped_rightmost: u64,
    leaves_left_at_116: u64,
}

struct ReferenceLeafShape {
    inode_tree: u64,
    leaves: Vec<ReferenceLeaf>,
    retired_in_this_change: Vec<ReferenceLeafIdentity>,
    counts: ShapeCounts,
    absorbs_in_this_remove: u64,
}

fn leaf_capacity() -> usize {
    usize::try_from(INODE_LEAF_RECORDS).expect("233")
}

impl ReferenceLeafShape {
    fn of_the_version(version: &TransactionOutput) -> Self {
        let leaves = leaves_of_the_version(version);
        let inode_tree = leaves.first().expect("至少一片叶").identity.birth_tree;
        Self { inode_tree, leaves, retired_in_this_change: Vec::new(), counts: ShapeCounts::default(), absorbs_in_this_remove: 0 }
    }
    fn append(&mut self, inode: u64, txg: u64) {
        if self.leaves.last().is_none_or(|leaf| leaf.inode_numbers.len() == leaf_capacity()) {
            self.leaves.push(ReferenceLeaf {
                identity: ReferenceLeafIdentity { birth_tree: self.inode_tree, record_type: 2, container: inode, container_birth: txg },
                inode_numbers: Vec::new(),
            });
        }
        self.leaves.last_mut().expect("至少一片").inode_numbers.push(inode);
    }
    fn records_at(&self, position: usize) -> usize {
        self.leaves[position].inode_numbers.len()
    }
    fn drop_leaf(&mut self, position: usize) -> ReferenceLeaf {
        let dropped = self.leaves.remove(position);
        self.retired_in_this_change.push(dropped.identity);
        dropped
    }
    fn left_absorbs_its_right_neighbour(&mut self, left_position: usize) {
        let absorbed = self.drop_leaf(left_position + 1);
        self.leaves[left_position].inode_numbers.extend(absorbed.inode_numbers);
        self.absorbs_in_this_remove += 1;
    }
    fn merge_from(&mut self, start_position: usize) -> bool {
        let mut position = start_position;
        let mut merged = false;
        loop {
            let right_pair_fits = position + 1 < self.leaves.len() && self.records_at(position) + self.records_at(position + 1) <= leaf_capacity();
            let left_pair_fits = position > 0 && self.records_at(position - 1) + self.records_at(position) <= leaf_capacity();
            if right_pair_fits && left_pair_fits {
                self.counts.both_pairs_fit += 1;
                if merged {
                    self.counts.both_pairs_fit_outside_the_first_step += 1;
                }
            }
            if right_pair_fits {
                self.left_absorbs_its_right_neighbour(position);
                self.counts.right_absorbs += 1;
            } else if left_pair_fits {
                self.left_absorbs_its_right_neighbour(position - 1);
                self.counts.left_absorbs += 1;
                position -= 1;
            } else {
                return merged;
            }
            merged = true;
        }
    }
    fn borrow_from_the_right_if_below_the_floor(&mut self, position: usize) {
        if self.records_at(position) >= leaf_capacity().div_ceil(2) {
            return;
        }
        if position + 1 >= self.leaves.len() {
            self.counts.borrow_skipped_rightmost += 1;
            return;
        }
        let moved_count = (self.records_at(position + 1) - self.records_at(position)) / 2;
        let moved: Vec<u64> = self.leaves[position + 1].inode_numbers.drain(..moved_count).collect();
        self.leaves[position].inode_numbers.extend(moved);
        self.counts.borrows += 1;
        if self.merge_from(position + 1) {
            self.counts.borrow_then_absorb += 1;
        }
    }
    fn remove(&mut self, inode: u64) {
        self.counts.removes += 1;
        self.absorbs_in_this_remove = 0;
        let position = self.leaves.iter().position(|leaf| leaf.inode_numbers.contains(&inode)).expect("在参照里");
        self.leaves[position].inode_numbers.retain(|kept| *kept != inode);
        if self.leaves[position].inode_numbers.is_empty() {
            self.drop_leaf(position);
            self.counts.drops += 1;
            if position > 0 && position < self.leaves.len() && self.merge_from(position - 1) {
                self.counts.drop_then_absorb += 1;
            }
        } else if !self.merge_from(position) {
            self.borrow_from_the_right_if_below_the_floor(position);
        }
        if self.absorbs_in_this_remove >= 2 {
            self.counts.removes_with_two_or_more_absorbs += 1;
        }
        self.counts.leaves_left_at_116 += u64::try_from(self.leaves.iter().filter(|leaf| leaf.inode_numbers.len() == 116).count()).expect("小");
    }
}

fn identity_of(identity: &singlefs_core::unit::PackedIdentity) -> ReferenceLeafIdentity {
    ReferenceLeafIdentity { birth_tree: identity.birth_tree.0, record_type: identity.record_type, container: identity.container, container_birth: identity.container_birth.0 }
}

fn leaves_of_the_version(version: &TransactionOutput) -> Vec<ReferenceLeaf> {
    version
        .inode_leaf_containers
        .iter()
        .map(|container| ReferenceLeaf {
            identity: identity_of(&container.contents.identity),
            inode_numbers: container.contents.records.iter().map(|record| record.inode).collect(),
        })
        .collect()
}

fn retirement_records_of(version: &TransactionOutput) -> Vec<(ReferenceLeafIdentity, u64)> {
    version
        .retirement_containers
        .iter()
        .flat_map(|container| container.contents.records.iter())
        .map(|record| (identity_of(&record.retired), record.death_txg.0))
        .collect()
}

fn compare(what: &str, before: &[(ReferenceLeafIdentity, u64)], version: &TransactionOutput, reference: &mut ReferenceLeafShape) {
    assert_eq!(leaves_of_the_version(version), reference.leaves, "{what}：叶与参照不同");
    let after = retirement_records_of(version);
    assert!(after.starts_with(before), "{what}：退役树只追加");
    let mut by_publish: Vec<ReferenceLeafIdentity> = after[before.len()..].iter().map(|(identity, _)| *identity).collect();
    let mut by_reference = std::mem::take(&mut reference.retired_in_this_change);
    by_publish.sort_unstable();
    by_reference.sort_unstable();
    assert_eq!(by_publish, by_reference, "{what}：退役集合与参照不同");
}

/// 照参照用例那一段历史（种子、常量、随机数的取用次序都照抄）在内存盘上走一遍，逐次比，并数参照走到过哪几形。
fn run_the_reference_history(seed: u64) -> ShapeCounts {
    let mut pool = Pool::after_the_first_file();
    let mut random = SeededRandomSource::from_seed(seed);
    let mut reference = ReferenceLeafShape::of_the_version(&pool.output);
    let mut left_to_create = 582 + random.below(17);
    // 迭代上界：每批至少建一个。
    while left_to_create > 0 {
        let count = (1 + random.below(260)).min(left_to_create);
        let before = retirement_records_of(&pool.output);
        let first_new = pool.output.inode_number_watermark();
        let version = pool.create(count);
        for inode in first_new..first_new + count {
            reference.append(inode, version.root.checkpoint_txg.0);
        }
        compare("建", &before, &version, &mut reference);
        left_to_create -= count;
    }
    let mut unlinks = 0;
    // 迭代上界：260 次。
    while reference.counts.both_pairs_fit == 0 {
        assert!(unlinks < 260, "种子 {seed}：删了 {unlinks} 次还没走到两对都装得下");
        let removable_of = |leaf: &ReferenceLeaf| leaf.inode_numbers.iter().filter(|inode| **inode != 1).count();
        let most = reference.leaves.iter().map(removable_of).max().expect("至少一片");
        let position = reference.leaves.iter().position(|leaf| removable_of(leaf) == most).expect("有");
        let removable: Vec<u64> = reference.leaves[position].inode_numbers.iter().copied().filter(|inode| *inode != 1).collect();
        let pick = usize::try_from(random.below(u64::try_from(removable.len()).expect("小"))).expect("小");
        let before = retirement_records_of(&pool.output);
        pool.unlink(removable[pick], ContainerTreeCapacities::FROM_THE_FORMAT);
        reference.remove(removable[pick]);
        let version = pool.output.clone();
        compare("删", &before, &version, &mut reference);
        unlinks += 1;
    }
    // 迭代上界：常量 6。
    for _ in 0..6 {
        let live: Vec<u64> = reference.leaves.iter().flat_map(|leaf| leaf.inode_numbers.iter().copied()).filter(|inode| *inode != 1).collect();
        let pick = usize::try_from(random.below(u64::try_from(live.len()).expect("小"))).expect("小");
        let before = retirement_records_of(&pool.output);
        pool.unlink(live[pick], ContainerTreeCapacities::FROM_THE_FORMAT);
        reference.remove(live[pick]);
        let version = pool.output.clone();
        compare("随机删", &before, &version, &mut reference);
    }
    reference.counts
}

/// K4.2：参照用例那一段历史（种子 53）走到过哪几形；`OPUS_K4_SEED_FIRST..=OPUS_K4_SEED_LAST` 另看同一形状的历史换种子时走到哪几形。
#[test]
fn opus_probe_k4_shapes_reached_by_the_reference_history() {
    let started = std::time::Instant::now();
    eprintln!("OPUS_K4 seed=53 {:?} elapsed_seconds={:.1}", run_the_reference_history(53), started.elapsed().as_secs_f64());
    let first_seed = env_u64("OPUS_K4_SEED_FIRST", 1);
    let last_seed = env_u64("OPUS_K4_SEED_LAST", 0);
    // 迭代上界：种子个数。
    for seed in first_seed..=last_seed {
        let started = std::time::Instant::now();
        eprintln!("OPUS_K4 seed={seed} {:?} elapsed_seconds={:.1}", run_the_reference_history(seed), started.elapsed().as_secs_f64());
    }
}

// ─── K3：回退取 inode 号水位那一段抽成函数前后逐字节相同（同一段历史上比每一步的根记录与水位）───

/// 产品容量下的随机历史（建、删、挂着回退），每一步之后把这一版的根记录与 inode 号水位记进摘要。第 53 批前后的 `mount.rs` 各跑一遍比摘要。
#[test]
fn opus_probe_k3_rollback_history_digest() {
    let first_seed = env_u64("OPUS_K3_SEED_FIRST", 1);
    let last_seed = env_u64("OPUS_K3_SEED_LAST", 4);
    let operations = env_u64("OPUS_K3_OPS", 80);
    let base = Pool::after_the_first_file();
    // 迭代上界：种子个数。
    for seed in first_seed..=last_seed {
        let mut random = SeededRandomSource::from_seed(seed);
        let mut pool = base.clone();
        pool.create(u64::from(INODE_LEAF_RECORDS) + 1 + random.below(8));
        let mut versions: Vec<TransactionOutput> = vec![pool.output.clone()];
        let mut trail = String::new();
        let (mut rollbacks_ok, mut rollbacks_refused) = (0u64, 0u64);
        // 迭代上界：OPUS_K3_OPS。
        for _ in 0..operations {
            let choice = random.below(100);
            let live: Vec<u64> = pool.live_inodes().into_iter().filter(|inode| *inode != 1).collect();
            if choice < 25 || live.is_empty() {
                pool.create(1 + random.below(3));
            } else if choice < 80 {
                let inode = if choice < 55 { *live.iter().max().expect("非空") } else {
                    live[usize::try_from(random.below(u64::try_from(live.len()).expect("小"))).expect("小")]
                };
                pool.unlink(inode, ContainerTreeCapacities::FROM_THE_FORMAT);
            } else {
                let candidates: Vec<&TransactionOutput> = versions.iter().filter(|version| version.root.checkpoint_txg != pool.output.root.checkpoint_txg).collect();
                if candidates.is_empty() {
                    continue;
                }
                let target = candidates[usize::try_from(random.below(u64::try_from(candidates.len()).expect("小"))).expect("小")].clone();
                match pool.roll_back(&target) {
                    Ok(()) => rollbacks_ok += 1,
                    Err(_) => rollbacks_refused += 1,
                }
            }
            versions.push(pool.output.clone());
            if versions.len() > 12 {
                versions.remove(0);
            }
            trail.push_str(&format!("{:?}|{}|{:?}\n", pool.output.root, pool.output.inode_number_watermark(), pool.output.lineage_containers.iter().map(|c| c.contents.records.clone()).collect::<Vec<_>>()));
        }
        eprintln!(
            "OPUS_K3_DIGEST seed={seed} ops={operations} rollbacks_ok={rollbacks_ok} rollbacks_refused={rollbacks_refused} final_txg={} final_watermark={} digest={}",
            pool.output.root.checkpoint_txg.0,
            pool.output.inode_number_watermark(),
            singlefs_harness::sha256::sha256_hexadecimal(trail.as_bytes())
        );
    }
}
