//! 攻方腿模型（三方轮 m3-close-b19-e165-r2，形甲第二份样本）：判「两条 checkpoint_txg 相同而非 0、实例代号不同、都自证过的根
//! 同时进 `recovery::choose_root` 的入参」造不造得出。不走写者历史：直接把根环每一槽摆成任意内容（全零、乱码、任意 txg 与实例代号的根，
//! 摆在任意槽里，罩住错位写、错位读、旧内容读回这几种「槽里是什么」的来路），看 `choose_root` 逐槽调的读槽函数
//! `recovery::read_root_ring_slot` 给哪几槽交出自证过的根，再用产品的 `choose_root` 与本地四条择根序比择到哪条。
//!
//! 复跑时把这份拷进副本的 crates/singlefs-harness/tests/（harness 档测试目标；不调崩溃枚举函数：这里没有写者历史，也就没有崩溃状态，
//! 摆法的取样域写在报告「取样」一节）。盘是内存稀疏盘（SparseBlockDevice），每个几何只 mkfs 一次，之后从那一份克隆。
//!
//! 三条用例：
//!   e165_form_a_admission_per_slot             逐槽 × 一族根编码（txg 0..6S+3、实例 0..=3）：每槽写进一条、读一次，数每个 txg 有几槽交出
//!                                              自证过的根；对照臂把读槽换成只验自证、不核落点公式的一份，同一个计数器要报出形甲摆得出。
//!   e165_form_a_whole_ring_choices             整环随机摆，产品读槽建择根入参，数入参里的形甲、本地 today / v1823 / v1825 / v1826 各择哪条、
//!                                              产品 choose_root 与本地 today 是否一致；同一份摆法也过对照臂。
//!   e165_form_a_sliced_matches_single_thread   同一个小域单线程与切片多线程逐项计数相同。
//!
//! 对照与阳性对照（跑前写死，见 assert）：对照臂（不核落点公式）在两条用例上都要报出非 0 txg 的形甲；产品臂在「任意编码」那一族
//! （收写者写不出的 txg 0、实例非 0 的根）上要报出 txg 0 的形甲。两样任一报 0，计数器没判别力，这份模型作废。
//! 产品臂在两族上的非 0 txg 形甲计数是被判的读数，不 assert，照打。
#![allow(clippy::too_many_lines, reason = "模型一份文件，摆法、读法与计数写在一起")]

use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{
    genesis_root_ring_slots, make_filesystem, MakeFilesystemParameters,
    FIRST_VERSION_REGION_DEVICES, GENESIS_CHECKPOINT_TXG,
};
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, every_root_ring_slot, read_root_ring_slot,
    RootRingSlotReading,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot, RootRingSlotsPerRegion};
use singlefs_core::system_configuration::{SystemConfiguration, SystemImmutableSizes};
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_harness::heavy_threads::default_heavy_threads;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

/// 每块盘的字节数：稀疏盘，只存写过的扇区。
const DEVICE_BYTES: u64 = 4 << 30;
/// 根槽宽与扇区宽。
const ROOT_SLOT_BYTES: usize = 512;
const FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x45, 0x31, 0x36, 0x35, 0x2d, 0x72, 0x32, 0x2d, 0x66, 0x6f, 0x72, 0x6d, 0x2d, 0x61, 0x00, 0x01,
];
/// 取样的每区槽数 S：区间两条边与 mkfs 显式入口常取的 8。
const SLOTS_PER_REGION_SAMPLED: [u64; 3] = [4, 8, 16];
/// 根编码里取的实例代号：0..INSTANCES_SAMPLED。
const INSTANCES_SAMPLED: u32 = 4;
/// 整环那条用例每个 (S, 编码族) 摆几环。
const RINGS_PER_CASE: u64 = 15_000;
/// 单线程与切片多线程对拍的小域。
const RINGS_FOR_THE_CROSS_CHECK: u64 = 3_000;
/// 单项超过这么久每片报一行进度（`.claude/rules/test-script-discipline.md`「编译与算法都要快」）。
const PROGRESS_INTERVAL_SECONDS: u64 = 300;

/// mkfs 之后的池、择到的系统配置与 mkfs 写的第 0 代根（根编码都从它改 txg 与实例代号得来）。
#[derive(Clone)]
struct FormattedRing {
    pool: MemoryPool,
    system_configuration: SystemConfiguration,
    genesis_root: RootRecord,
    ring_slots: Vec<RootRingSlot>,
}

fn formatted_ring(slots_per_region: u64) -> FormattedRing {
    let geometry = SystemImmutableSizes {
        physical_block_size: 512,
        minimum_input_output_bytes: 512,
        fixed_structure_slot_spacing: 4096,
        journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
        root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(
            slots_per_region,
        )
        .expect("取样的 S 在格式区间 4..=16 里"),
    };
    let parameters = MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: FIRST_VERSION_REGION_DEVICES,
        geometry,
    };
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            (
                identity,
                SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    make_filesystem(&parameters, &mut devices).expect("两块内存盘上 mkfs");
    let pool = MemoryPool {
        devices: devices
            .into_iter()
            .map(|(identity, device)| (identity, device.image))
            .collect(),
        device_size_in_bytes: DEVICE_BYTES,
    };
    let system_configuration = choose_system_configuration(&pool).expect("mkfs 之后择得出系统配置");
    let genesis_root = choose_root(&pool, &system_configuration).expect("mkfs 种了第 0 代根");
    assert_eq!(genesis_root.checkpoint_txg, GENESIS_CHECKPOINT_TXG, "前提：mkfs 那条根是第 0 代");
    let ring_slots = every_root_ring_slot(&system_configuration.immutable.sizes);
    assert_eq!(
        u64::try_from(ring_slots.len()).expect("槽数"),
        3 * slots_per_region,
        "前提：读根环走 3S 个槽"
    );
    FormattedRing {
        pool,
        system_configuration,
        genesis_root,
        ring_slots,
    }
}

impl FormattedRing {
    fn slots_per_region(&self) -> RootRingSlotsPerRegion {
        self.system_configuration.immutable.sizes.root_ring_slots_per_region
    }

    /// 第 0 代根只改 txg 与实例代号的那一条编码（择根只看自证、fsid、落点与这两项）。
    fn root_with(&self, checkpoint_txg: u64, instance: u32) -> RootRecord {
        let mut root = self.genesis_root;
        root.checkpoint_txg = CheckpointTxg(checkpoint_txg);
        root.instance = InstanceGeneration(instance);
        root
    }

    fn device_and_offset(&self, ring_slot: RootRingSlot) -> (DeviceIdentity, singlefs_core::address::DeviceOffsetInBytes) {
        let immutable = &self.system_configuration.immutable;
        (
            immutable.region_devices[usize::try_from(ring_slot.region).expect("区域号")],
            slot_offset(ring_slot, immutable.sizes.fixed_structure_slot_spacing),
        )
    }
}

/// 往一槽里摆字节：写进这一槽的区域归属那块盘（读根环的读者只从那块盘读它）。
fn put_bytes_into_ring_slot(ring: &FormattedRing, pool: &mut MemoryPool, ring_slot: RootRingSlot, bytes: &[u8]) {
    let (device, offset) = ring.device_and_offset(ring_slot);
    pool.devices
        .get_mut(&device)
        .expect("池里有这块盘")
        .write(offset, bytes);
}

/// 读这一槽的原样字节（对照臂自己验自证用）。
fn raw_bytes_of_ring_slot(ring: &FormattedRing, pool: &MemoryPool, ring_slot: RootRingSlot) -> Vec<u8> {
    let (device, offset) = ring.device_and_offset(ring_slot);
    pool.devices
        .get(&device)
        .expect("池里有这块盘")
        .read(offset, ROOT_SLOT_BYTES)
}

/// 建择根入参用哪一份读槽：产品那一份（`recovery::read_root_ring_slot`，验自证、fsid 再核落点公式），
/// 或对照臂（只验自证与 fsid，`RootRecord::parse_slot`，不核落点公式——C743 之前的读法）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SlotReaderArm {
    ProductWithTheFormulaCheck,
    ControlWithoutTheFormulaCheck,
}

impl SlotReaderArm {
    const BOTH: [Self; 2] = [Self::ProductWithTheFormulaCheck, Self::ControlWithoutTheFormulaCheck];

    fn label(self) -> &'static str {
        match self {
            Self::ProductWithTheFormulaCheck => "product",
            Self::ControlWithoutTheFormulaCheck => "control_without_the_formula_check",
        }
    }

    fn reading(self, ring: &FormattedRing, pool: &MemoryPool, ring_slot: RootRingSlot) -> Option<RootRecord> {
        let immutable = &ring.system_configuration.immutable;
        match self {
            Self::ProductWithTheFormulaCheck => match read_root_ring_slot(
                pool,
                &immutable.region_devices,
                &immutable.sizes,
                &immutable.filesystem_identifier,
                ring_slot,
            ) {
                RootRingSlotReading::SelfVerified(root) => Some(root),
                RootRingSlotReading::Bad(_) => None,
            },
            Self::ControlWithoutTheFormulaCheck => RootRecord::parse_slot(
                &raw_bytes_of_ring_slot(ring, pool, ring_slot),
                &immutable.filesystem_identifier,
            ),
        }
    }
}

/// 本地四条择根序（E165 登记第七节「本地择根序」那几列的比较式各写一份）：读根环次序里往后走，后来的严格「更新」才换。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChoiceOrder {
    /// 今天的：txg 为主、平局实例代号高者赢。
    Today,
    /// v1823：比较不看实例代号。
    IgnoresTheInstance,
    /// v1825：txg 平局取实例代号低的。
    TieTakesTheLowerInstance,
    /// v1826：实例代号为主、txg 为次。
    InstanceFirstThenTxg,
}

impl ChoiceOrder {
    const COMPARED_WITH_TODAY: [Self; 3] = [
        Self::IgnoresTheInstance,
        Self::TieTakesTheLowerInstance,
        Self::InstanceFirstThenTxg,
    ];

    fn replaces(self, candidate: (u64, u32), chosen: (u64, u32)) -> bool {
        let (candidate_txg, candidate_instance) = candidate;
        let (chosen_txg, chosen_instance) = chosen;
        match self {
            Self::Today => {
                candidate_txg > chosen_txg
                    || (candidate_txg == chosen_txg && candidate_instance > chosen_instance)
            }
            Self::IgnoresTheInstance => candidate_txg > chosen_txg,
            Self::TieTakesTheLowerInstance => {
                candidate_txg > chosen_txg
                    || (candidate_txg == chosen_txg && candidate_instance < chosen_instance)
            }
            Self::InstanceFirstThenTxg => {
                candidate_instance > chosen_instance
                    || (candidate_instance == chosen_instance && candidate_txg > chosen_txg)
            }
        }
    }

    fn chosen_root(self, choice_input: &[Option<RootRecord>]) -> Option<RootRecord> {
        let mut chosen: Option<RootRecord> = None;
        for root in choice_input.iter().flatten() {
            let key = (root.checkpoint_txg.0, root.instance.0);
            if chosen.is_none_or(|already| {
                self.replaces(key, (already.checkpoint_txg.0, already.instance.0))
            }) {
                chosen = Some(*root);
            }
        }
        chosen
    }
}

/// 择根入参里有没有形甲：两条 txg 相同、实例代号不同的根。`only_at_the_newest_txg` 只看入参里最大的那个 txg。
/// 交回形甲所在的 txg（有几处取最大的那个）。
fn form_a_txg(choice_input: &[Option<RootRecord>], only_at_the_newest_txg: bool) -> Option<u64> {
    let mut instances_by_txg: BTreeMap<u64, BTreeSet<u32>> = BTreeMap::new();
    for root in choice_input.iter().flatten() {
        instances_by_txg
            .entry(root.checkpoint_txg.0)
            .or_default()
            .insert(root.instance.0);
    }
    if only_at_the_newest_txg {
        return instances_by_txg
            .iter()
            .next_back()
            .filter(|(_, instances)| instances.len() >= 2)
            .map(|(txg, _)| *txg);
    }
    instances_by_txg
        .iter()
        .rev()
        .find(|(txg, instances)| **txg != 0 && instances.len() >= 2)
        .map(|(txg, _)| *txg)
}

/// 逐槽 × 一族根编码的读数：每个 txg 在哪几槽读得出自证过的根（槽按读根环次序的下标），以及读得出与否随不随实例代号变。
#[derive(Debug, Default, PartialEq, Eq)]
struct AdmissionTally {
    pairs_judged: u64,
    pairs_admitted: u64,
    slots_admitting_txg: BTreeMap<u64, BTreeSet<usize>>,
    /// 同一槽、同一 txg，几个实例代号里有的读得出、有的读不出的 (槽, txg) 个数。
    instance_dependent_admissions: u64,
}

fn admission_per_slot(ring: &FormattedRing, arm: SlotReaderArm, txg_bound: u64) -> AdmissionTally {
    let mut tally = AdmissionTally::default();
    let mut pool = ring.pool.clone();
    for (slot_index, ring_slot) in ring.ring_slots.iter().copied().enumerate() {
        let original_bytes = raw_bytes_of_ring_slot(ring, &pool, ring_slot);
        for checkpoint_txg in 0..txg_bound {
            let mut admitted_instances = 0u32;
            for instance in 0..INSTANCES_SAMPLED {
                let bytes = ring.root_with(checkpoint_txg, instance).to_slot(ROOT_SLOT_BYTES);
                put_bytes_into_ring_slot(ring, &mut pool, ring_slot, &bytes);
                tally.pairs_judged += 1;
                if arm.reading(ring, &pool, ring_slot).is_some() {
                    tally.pairs_admitted += 1;
                    admitted_instances += 1;
                    tally
                        .slots_admitting_txg
                        .entry(checkpoint_txg)
                        .or_default()
                        .insert(slot_index);
                }
            }
            if admitted_instances != 0 && admitted_instances != INSTANCES_SAMPLED {
                tally.instance_dependent_admissions += 1;
            }
        }
        put_bytes_into_ring_slot(ring, &mut pool, ring_slot, &original_bytes);
    }
    tally
}

/// 一条非 0 txg 的根写进它落点槽的那个偏移、却写在另一块盘上：产品读槽只读区域归属那块盘，读不到它。交回读到了的个数。
fn admitted_from_the_device_the_region_does_not_live_on(ring: &FormattedRing, txg_bound: u64) -> (u64, u64) {
    let mut judged = 0u64;
    let mut admitted = 0u64;
    for checkpoint_txg in 1..txg_bound {
        let mut pool = ring.pool.clone();
        let ring_slot = target_for_publish(CheckpointTxg(checkpoint_txg), ring.slots_per_region());
        let (region_device, offset) = ring.device_and_offset(ring_slot);
        put_bytes_into_ring_slot(ring, &mut pool, ring_slot, &[0u8; ROOT_SLOT_BYTES]);
        let other_device = pool
            .devices
            .keys()
            .copied()
            .find(|identity| *identity != region_device)
            .expect("两块盘");
        pool.devices
            .get_mut(&other_device)
            .expect("池里有这块盘")
            .write(offset, &ring.root_with(checkpoint_txg, 1).to_slot(ROOT_SLOT_BYTES));
        judged += 1;
        if SlotReaderArm::ProductWithTheFormulaCheck
            .reading(ring, &pool, ring_slot)
            .is_some()
        {
            admitted += 1;
        }
    }
    (judged, admitted)
}

#[test]
fn e165_form_a_admission_per_slot() {
    for slots_per_region in SLOTS_PER_REGION_SAMPLED {
        let started = Instant::now();
        let ring = formatted_ring(slots_per_region);
        let ring_length = 3 * slots_per_region;
        let txg_bound = 2 * ring_length + 3;
        let genesis_slot_indexes: BTreeSet<usize> = genesis_root_ring_slots()
            .map(|genesis_slot| {
                ring.ring_slots
                    .iter()
                    .position(|ring_slot| *ring_slot == genesis_slot)
                    .expect("种根的槽在根环里")
            })
            .collect();
        for arm in SlotReaderArm::BOTH {
            let tally = admission_per_slot(&ring, arm, txg_bound);
            let nonzero: Vec<(u64, &BTreeSet<usize>)> = tally
                .slots_admitting_txg
                .iter()
                .filter(|(txg, _)| **txg != 0)
                .map(|(txg, slots)| (*txg, slots))
                .collect();
            let nonzero_txgs_in_two_or_more_slots =
                nonzero.iter().filter(|(_, slots)| slots.len() >= 2).count();
            let largest_slot_count_of_a_nonzero_txg =
                nonzero.iter().map(|(_, slots)| slots.len()).max().unwrap_or(0);
            let formula_mismatches = (1..txg_bound)
                .filter(|txg| {
                    let formula_slot = target_for_publish(CheckpointTxg(*txg), ring.slots_per_region());
                    let formula_index = ring
                        .ring_slots
                        .iter()
                        .position(|ring_slot| *ring_slot == formula_slot)
                        .expect("落点槽在根环里");
                    tally.slots_admitting_txg.get(txg) != Some(&BTreeSet::from([formula_index]))
                })
                .count();
            let slots_admitting_txg_zero = tally
                .slots_admitting_txg
                .get(&0)
                .cloned()
                .unwrap_or_default();
            println!(
                "OPUSMODEL name=admission arm={} slots_per_region={slots_per_region} ring_slots={ring_length} txg_range=0..{txg_bound} instances=0..{INSTANCES_SAMPLED} pairs_judged={} pairs_admitted={} nonzero_txgs_admitted={} nonzero_txgs_admitted_in_two_or_more_slots={nonzero_txgs_in_two_or_more_slots} largest_slot_count_of_a_nonzero_txg={largest_slot_count_of_a_nonzero_txg} nonzero_txgs_not_exactly_at_their_formula_slot={formula_mismatches} slots_admitting_txg_zero={:?} genesis_slots={:?} instance_dependent_admissions={} threads=1",
                arm.label(),
                tally.pairs_judged,
                tally.pairs_admitted,
                nonzero.len(),
                slots_admitting_txg_zero,
                genesis_slot_indexes,
                tally.instance_dependent_admissions,
            );
            if arm == SlotReaderArm::ControlWithoutTheFormulaCheck {
                assert!(
                    nonzero_txgs_in_two_or_more_slots > 0,
                    "对照臂（不核落点公式）要报出非 0 txg 摆得进两槽；报 0 说明这个计数器没判别力，模型作废"
                );
            }
        }
        let (judged, admitted) = admitted_from_the_device_the_region_does_not_live_on(&ring, txg_bound);
        println!(
            "OPUSMODEL name=other_device slots_per_region={slots_per_region} roots_written_on_the_other_device={judged} admitted_by_the_product_reader={admitted} seconds={:.2}",
            started.elapsed().as_secs_f64()
        );
    }
}

/// 根编码取哪一族：写者写得出的（txg 0 只有 mkfs 写、实例 0；txg ≥ 1 的实例 ≥ 1），或任意编码（另收 txg 0、实例非 0 的根，写者写不出）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EncodingFamily {
    WriterReachable,
    AnyEncoding,
}

impl EncodingFamily {
    const BOTH: [Self; 2] = [Self::WriterReachable, Self::AnyEncoding];

    fn label(self) -> &'static str {
        match self {
            Self::WriterReachable => "writer_reachable",
            Self::AnyEncoding => "any_encoding",
        }
    }

    fn instance_for(self, checkpoint_txg: u64, random: u64) -> u32 {
        let spread = u32::try_from(random % u64::from(INSTANCES_SAMPLED)).expect("小于 4");
        match self {
            Self::WriterReachable if checkpoint_txg == 0 => 0,
            Self::WriterReachable => 1 + spread % (INSTANCES_SAMPLED - 1),
            Self::AnyEncoding => spread,
        }
    }

    fn seed_word(self) -> u64 {
        match self {
            Self::WriterReachable => 0x5752_4954_4552,
            Self::AnyEncoding => 0x414e_5945_4e43,
        }
    }
}

/// 摆法的随机源（splitmix64）：每环一个种子，由 (S, 编码族, 环号) 定，单独复现一环只要这三样。
struct SplitMix(u64);

impl SplitMix {
    fn next_word(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut mixed = self.0;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        mixed ^ (mixed >> 31)
    }
}

fn ring_seed(slots_per_region: u64, family: EncodingFamily, ring_index: u64) -> u64 {
    family.seed_word() ^ (slots_per_region << 48) ^ ring_index.wrapping_mul(0x2545_f491_4f6c_dd1d)
}

/// 一环摆什么：每槽全零、乱码或一条根的字节。四分之一的环只在第 0 代根那三槽摆根（别的槽全零或乱码），让最大 txg 落在 0 上；
/// 其余的环每槽八分之三摆一条住在它落点公式里的根（txg = 该槽最小 txg + k·3S，k ∈ 0..3），八分之三摆一条 txg 取自这一环
/// 撞在一起的三个值 {0, 低, 低 + 3S}（低与低 + 3S 落在同一个落点槽）的根——摆在哪槽不看公式，罩错位写与错位读。
fn placement_of_the_ring(ring: &FormattedRing, family: EncodingFamily, ring_index: u64) -> Vec<Vec<u8>> {
    let slots_per_region = ring.slots_per_region().count();
    let ring_length = 3 * slots_per_region;
    let mut random = SplitMix(ring_seed(slots_per_region, family, ring_index));
    let genesis_only = random.next_word() % 4 == 0;
    let colliding_low = 1 + random.next_word() % (2 * ring_length);
    let colliding_txgs = [0, colliding_low, colliding_low + ring_length];
    let genesis_slots: Vec<RootRingSlot> = genesis_root_ring_slots().collect();
    let root_bytes = |checkpoint_txg: u64, random: &mut SplitMix| {
        let instance = family.instance_for(checkpoint_txg, random.next_word());
        ring.root_with(checkpoint_txg, instance).to_slot(ROOT_SLOT_BYTES)
    };
    ring.ring_slots
        .iter()
        .map(|ring_slot| {
            let draw = random.next_word() % 8;
            let garbage = |random: &mut SplitMix| -> Vec<u8> {
                (0..ROOT_SLOT_BYTES / 8)
                    .flat_map(|_| random.next_word().to_le_bytes())
                    .collect()
            };
            if genesis_only {
                if genesis_slots.contains(ring_slot) && draw >= 2 {
                    return root_bytes(0, &mut random);
                }
                return if draw % 2 == 0 { vec![0u8; ROOT_SLOT_BYTES] } else { garbage(&mut random) };
            }
            match draw {
                0 => vec![0u8; ROOT_SLOT_BYTES],
                1 => garbage(&mut random),
                2..=4 => {
                    let smallest_txg_of_the_slot = ring_slot.slot * 3 + ring_slot.region;
                    let turns = random.next_word() % 3;
                    root_bytes(smallest_txg_of_the_slot + turns * ring_length, &mut random)
                }
                _ => {
                    let checkpoint_txg = colliding_txgs[usize::try_from(random.next_word() % 3).expect("小于 3")];
                    root_bytes(checkpoint_txg, &mut random)
                }
            }
        })
        .collect()
}

/// 一段环号上一个读槽臂的计数。合并按片的次序相加，「最小环号」取各片里最小的。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct RingTally {
    rings: u64,
    /// 入参里有非 0 txg 的形甲（两条同 txg、实例代号不同）的环数。
    form_a_at_a_nonzero_txg: u64,
    /// 入参最大的那个 txg 上有形甲、且那个 txg 不是 0 的环数。
    form_a_at_the_newest_nonzero_txg: u64,
    /// 入参最大的那个 txg 是 0、且它上面有形甲的环数。
    form_a_at_the_newest_txg_zero: u64,
    /// 本地 v1823、v1825、v1826 各自择到的根与本地 today 不同的环数（次序同 `ChoiceOrder::COMPARED_WITH_TODAY`）。
    differs_from_today: [u64; 3],
    /// 只在产品臂上数：产品 `choose_root` 择到的根与本地 today 不同的环数。
    product_choose_root_differs_from_today: u64,
    smallest_ring_with_form_a_at_a_nonzero_txg: Option<u64>,
    smallest_ring_where_v1823_or_v1825_differs: Option<u64>,
}

impl RingTally {
    fn merge(&mut self, slice: &Self) {
        self.rings += slice.rings;
        self.form_a_at_a_nonzero_txg += slice.form_a_at_a_nonzero_txg;
        self.form_a_at_the_newest_nonzero_txg += slice.form_a_at_the_newest_nonzero_txg;
        self.form_a_at_the_newest_txg_zero += slice.form_a_at_the_newest_txg_zero;
        for (total, part) in self.differs_from_today.iter_mut().zip(slice.differs_from_today) {
            *total += part;
        }
        self.product_choose_root_differs_from_today += slice.product_choose_root_differs_from_today;
        self.smallest_ring_with_form_a_at_a_nonzero_txg = smaller_ring(
            self.smallest_ring_with_form_a_at_a_nonzero_txg,
            slice.smallest_ring_with_form_a_at_a_nonzero_txg,
        );
        self.smallest_ring_where_v1823_or_v1825_differs = smaller_ring(
            self.smallest_ring_where_v1823_or_v1825_differs,
            slice.smallest_ring_where_v1823_or_v1825_differs,
        );
    }
}

fn smaller_ring(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (left, None) => left,
        (None, right) => right,
    }
}

/// 一环摆进池、两个读槽臂各建一份择根入参、各数一遍。两臂读的是同一份摆法。
fn judge_rings(ring: &FormattedRing, family: EncodingFamily, ring_indexes: std::ops::Range<u64>, slice_label: usize) -> [RingTally; 2] {
    let mut pool = ring.pool.clone();
    let mut tallies = [RingTally::default(), RingTally::default()];
    let mut last_progress = Instant::now();
    let slice_start = ring_indexes.start;
    for ring_index in ring_indexes.clone() {
        let placement = placement_of_the_ring(ring, family, ring_index);
        for (ring_slot, bytes) in ring.ring_slots.iter().zip(&placement) {
            put_bytes_into_ring_slot(ring, &mut pool, *ring_slot, bytes);
        }
        for (arm, tally) in SlotReaderArm::BOTH.into_iter().zip(tallies.iter_mut()) {
            let choice_input: Vec<Option<RootRecord>> = ring
                .ring_slots
                .iter()
                .map(|ring_slot| arm.reading(ring, &pool, *ring_slot))
                .collect();
            tally.rings += 1;
            if form_a_txg(&choice_input, false).is_some() {
                tally.form_a_at_a_nonzero_txg += 1;
                tally.smallest_ring_with_form_a_at_a_nonzero_txg =
                    smaller_ring(tally.smallest_ring_with_form_a_at_a_nonzero_txg, Some(ring_index));
            }
            match form_a_txg(&choice_input, true) {
                Some(0) => tally.form_a_at_the_newest_txg_zero += 1,
                Some(_) => tally.form_a_at_the_newest_nonzero_txg += 1,
                None => {}
            }
            let today = ChoiceOrder::Today.chosen_root(&choice_input);
            for (order, count) in ChoiceOrder::COMPARED_WITH_TODAY
                .into_iter()
                .zip(tally.differs_from_today.iter_mut())
            {
                if order.chosen_root(&choice_input) != today {
                    *count += 1;
                    if order != ChoiceOrder::InstanceFirstThenTxg {
                        tally.smallest_ring_where_v1823_or_v1825_differs = smaller_ring(
                            tally.smallest_ring_where_v1823_or_v1825_differs,
                            Some(ring_index),
                        );
                    }
                }
            }
            if arm == SlotReaderArm::ProductWithTheFormulaCheck
                && choose_root(&pool, &ring.system_configuration) != today
            {
                tally.product_choose_root_differs_from_today += 1;
            }
        }
        if last_progress.elapsed().as_secs() >= PROGRESS_INTERVAL_SECONDS {
            println!(
                "OPUSMODEL name=progress slice={slice_label} range={slice_start}..{} judged={}",
                ring_indexes.end,
                ring_index + 1 - slice_start
            );
            last_progress = Instant::now();
        }
    }
    tallies
}

/// 环号区间按线程数切片并行（`std::thread::scope`），每片自己一份池；合并按片的次序。
fn judge_rings_sliced(ring: &FormattedRing, family: EncodingFamily, rings: u64, threads: usize) -> [RingTally; 2] {
    let thread_count = u64::try_from(threads).expect("线程数").max(1);
    let slice_length = rings.div_ceil(thread_count);
    let slices: Vec<std::ops::Range<u64>> = (0..thread_count)
        .map(|slice| (slice * slice_length).min(rings)..((slice + 1) * slice_length).min(rings))
        .filter(|range| !range.is_empty())
        .collect();
    let per_slice: Vec<[RingTally; 2]> = std::thread::scope(|scope| {
        let handles: Vec<_> = slices
            .iter()
            .cloned()
            .enumerate()
            .map(|(slice_label, range)| scope.spawn(move || judge_rings(ring, family, range, slice_label)))
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().expect("切片线程没有 panic"))
            .collect()
    });
    let mut merged = [RingTally::default(), RingTally::default()];
    for slice in &per_slice {
        for (total, part) in merged.iter_mut().zip(slice) {
            total.merge(part);
        }
    }
    merged
}

fn print_ring_tally(slots_per_region: u64, family: EncodingFamily, arm: SlotReaderArm, threads: usize, tally: &RingTally, seconds: f64) {
    let [v1823, v1825, v1826] = tally.differs_from_today;
    println!(
        "OPUSMODEL name=ring_choices slots_per_region={slots_per_region} family={} arm={} rings={} threads={threads} form_a_at_a_nonzero_txg={} form_a_at_the_newest_nonzero_txg={} form_a_at_the_newest_txg_zero={} v1823_differs_from_today={v1823} v1825_differs_from_today={v1825} v1826_differs_from_today={v1826} product_choose_root_differs_from_today={} smallest_ring_with_form_a_at_a_nonzero_txg={} smallest_ring_where_v1823_or_v1825_differs={} seconds={seconds:.2}",
        family.label(),
        arm.label(),
        tally.rings,
        tally.form_a_at_a_nonzero_txg,
        tally.form_a_at_the_newest_nonzero_txg,
        tally.form_a_at_the_newest_txg_zero,
        tally.product_choose_root_differs_from_today,
        tally
            .smallest_ring_with_form_a_at_a_nonzero_txg
            .map_or_else(|| "none".to_owned(), |ring_index| ring_index.to_string()),
        tally
            .smallest_ring_where_v1823_or_v1825_differs
            .map_or_else(|| "none".to_owned(), |ring_index| ring_index.to_string()),
    );
}

#[test]
fn e165_form_a_whole_ring_choices() {
    let threads = default_heavy_threads().get();
    for slots_per_region in SLOTS_PER_REGION_SAMPLED {
        let ring = formatted_ring(slots_per_region);
        for family in EncodingFamily::BOTH {
            let started = Instant::now();
            let tallies = judge_rings_sliced(&ring, family, RINGS_PER_CASE, threads);
            let seconds = started.elapsed().as_secs_f64();
            for (arm, tally) in SlotReaderArm::BOTH.into_iter().zip(&tallies) {
                print_ring_tally(slots_per_region, family, arm, threads, tally, seconds);
            }
            let [product, control] = &tallies;
            assert_eq!(
                product.product_choose_root_differs_from_today, 0,
                "产品 choose_root 与本地 today 在产品臂的入参上要择同一条；不同说明本地 today 抄错了，v1823 至 v1826 的比较作废"
            );
            assert!(
                control.form_a_at_a_nonzero_txg > 0,
                "对照臂（不核落点公式）要报出非 0 txg 的形甲；报 0 说明摆法撞不出同 txg，模型作废"
            );
            if family == EncodingFamily::AnyEncoding {
                assert!(
                    product.form_a_at_the_newest_txg_zero > 0,
                    "阳性对照：任意编码一族里产品臂要报出 txg 0 上的形甲（第 0 代根那三槽另认）；报 0 说明计数器在产品臂上看不见形甲，模型作废"
                );
            }
        }
    }
}

#[test]
fn e165_form_a_sliced_matches_single_thread() {
    let threads = default_heavy_threads().get();
    let ring = formatted_ring(8);
    let family = EncodingFamily::AnyEncoding;
    let single = judge_rings_sliced(&ring, family, RINGS_FOR_THE_CROSS_CHECK, 1);
    let sliced = judge_rings_sliced(&ring, family, RINGS_FOR_THE_CROSS_CHECK, threads);
    for (arm, (single_tally, sliced_tally)) in SlotReaderArm::BOTH.into_iter().zip(single.iter().zip(&sliced)) {
        println!(
            "OPUSMODEL name=cross_check arm={} rings={RINGS_FOR_THE_CROSS_CHECK} single_thread_equals_sliced={} threads={threads} single={single_tally:?} sliced={sliced_tally:?}",
            arm.label(),
            single_tally == sliced_tally,
        );
    }
    assert_eq!(single, sliced, "单线程与切片多线程的计数要逐项相同");
}
