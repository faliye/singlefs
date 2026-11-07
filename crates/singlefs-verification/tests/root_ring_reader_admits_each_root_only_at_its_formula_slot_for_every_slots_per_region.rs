//! 验证包模块：无（只用验证包的内存池与每台默认重活线程数，不碰崩溃引擎模块）
//! 根环读者的落点穷举（C650（择根与重放没有合判） 的读者侧留档用例；判据在判决 `research/prompts/m3-close-b19-e165-r3-main-verification.md`
//! 第二节「C650 剩下那一句的判据（定）」那一格）：择根的两条改坏（比较不看实例代号、txg 平局取实例代号低的）与今天的择根序择得不同，
//! 当且仅当同一 txg 上两种实例代号同时进择根；承诺内挡住它的读者那一样是落点核——产品读槽 `recovery::read_root_ring_slot`
//! 只在落点公式算出的那一槽收 txg 不是 0 的根，只在 mkfs 种根的三槽（三个区域各自的槽 0）收 txg 0 的根
//! （D22（单元原子性怎么合成） 已定项 7 实例代号那一行、已定项 16）。写者那一半（txg 0 的根只有 mkfs 写）在
//! `root_ring_writer_txgs_across_writable_mounts_and_publishes_on_a_fresh_pool.rs`。
//!
//! 每区槽数 S 取格式区间 4..=16 的每一档，不只 2 的幂：落点公式里的取模写成按位与 `& (S − 1)` 在 2 的幂上与取模同值，
//! 只有别的那几档分得出。每档 mkfs 一次，之后对根环的每一槽，逐个写进一条根编码（mkfs 那条第 0 代根只改 txg 与实例代号：
//! txg 取 0..=9S，即每一槽三圈的落点，实例代号取 0..=3），调产品读槽看收不收，那一槽判完还原。槽里摆的是任意一条自证得过的根，
//! 罩错位写、错位读、旧内容读回这几种「槽里是什么」的来路。不造崩溃状态，不调崩溃枚举函数。
//!
//! 盘面数（写一条根编码、读一次算一个）：每档 3S 槽 × (9S + 1) 个 txg × 4 种实例代号，S 取 4..=16 合计 161616 个；
//! 按槽切片、每台默认重活线程数个线程并行，合并按片的次序（`admissions_judged_on_slices`）；单线程与切片的对拍是
//! `admissions_judged_on_slices_equal_admissions_judged_on_one_thread`。

use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{
    genesis_root_ring_slots, make_filesystem, MakeFilesystemParameters,
    FIRST_VERSION_REGION_DEVICES, GENESIS_CHECKPOINT_TXG,
};
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, every_root_ring_slot, read_root_ring_slot,
    BadRootRingSlotReading, RootRingSlotReading,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{
    slot_offset, target_for_publish, RootRingSlot, RootRingSlotsPerRegion,
};
use singlefs_core::system_configuration::{SystemConfiguration, SystemImmutableSizes};
use singlefs_format::{
    JOURNAL_RING_DEFAULT_BYTES, ROOT_RING_REGIONS, ROOT_RING_SLOTS_PER_REGION_MAXIMUM,
    ROOT_RING_SLOTS_PER_REGION_MINIMUM,
};
use singlefs_verification::heavy_threads::default_heavy_threads;
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice};

/// 每块内存盘的字节数：稀疏盘，只存写过的扇区。
const DEVICE_SIZE_IN_BYTES: u64 = 4 << 30;
/// 物理块宽，也是 io_min：根槽宽就是物理块宽（`read_root_ring_slot` 读这么多字节）。
const PHYSICAL_BLOCK_SIZE_IN_BYTES: u32 = 512;
const FIXED_STRUCTURE_SLOT_SPACING_IN_BYTES: u32 = 4096;
const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"reader-admission";
/// 写进每一槽的 txg 跨几圈落点：txg 取 0..=圈数 × 3S，每一槽在每一圈各是一个 txg 的落点。
const REVOLUTIONS_OF_THE_TXGS_WRITTEN: u64 = 3;
/// 写进每一槽的实例代号取 0..这个数：读者不看实例代号，每一种都要同样收或同样不收。
const INSTANCE_GENERATIONS_WRITTEN: u32 = 4;
/// S 取 4..=16 每一档时写进去的根编码总数（每档 3S × (9S + 1) × 4 之和）：钉住穷举的域没有缩。
const ENCODINGS_WRITTEN_OVER_EVERY_SLOTS_PER_REGION: u64 = 161_616;
/// 单线程与切片对拍取的那一档 S：不是 2 的幂。
const SLOTS_PER_REGION_OF_THE_SLICED_CROSS_CHECK: u64 = 5;
/// 单项超过这么久每片报一行进度（`.claude/rules/implementation-workflow.md`「测试与崩溃检测优先多线程」）。
const PROGRESS_INTERVAL_SECONDS: u64 = 300;
/// 报红时每一类最多列几条。
const MISMATCHES_LISTED_IN_A_FAILURE: usize = 8;

fn parameters_with_slots_per_region(slots_per_region: u64) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: FIRST_VERSION_REGION_DEVICES,
        geometry: SystemImmutableSizes {
            physical_block_size: PHYSICAL_BLOCK_SIZE_IN_BYTES,
            minimum_input_output_bytes: PHYSICAL_BLOCK_SIZE_IN_BYTES,
            fixed_structure_slot_spacing: FIXED_STRUCTURE_SLOT_SPACING_IN_BYTES,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(
                slots_per_region,
            )
            .expect("取的 S 都在格式区间 4..=16 里"),
        },
    }
}

/// 落点公式（D22（单元原子性怎么合成） 已定项 2：第 n 次发布写区域 n mod R 的槽 (n div R) mod S）照条款另写一份，
/// 不调产品的 `target_for_publish`：判的是读者对不对得上条款，产品那一份改坏时这里照样给出条款的落点。
fn ring_slot_given_by_the_publish_formula(
    checkpoint_txg: u64,
    slots_per_region: u64,
) -> RootRingSlot {
    RootRingSlot {
        region: checkpoint_txg % ROOT_RING_REGIONS,
        slot: (checkpoint_txg / ROOT_RING_REGIONS) % slots_per_region,
    }
}

/// mkfs 种第 0 代根的槽：三个区域各自的槽 0（D22（单元原子性怎么合成） 已定项 7 实例代号那一行），同样照条款另写一份。
fn ring_slots_where_make_filesystem_seeds_the_genesis_root() -> BTreeSet<RootRingSlot> {
    (0..ROOT_RING_REGIONS)
        .map(|region| RootRingSlot { region, slot: 0 })
        .collect()
}

/// mkfs 之后的一档几何：读者用的池、择到的系统配置、mkfs 写的第 0 代根（每条根编码从它只改 txg 与实例代号得来）、读根环次序的全部槽。
#[derive(Clone)]
struct FormattedRootRing {
    slots_per_region: u64,
    pool: MemoryPool,
    system_configuration: SystemConfiguration,
    genesis_root: RootRecord,
    ring_slots: Vec<RootRingSlot>,
}

/// 两块内存盘上照这一档 S mkfs 一次，盘此刻的字节照抄成读者用的池。
fn formatted_root_ring(slots_per_region: u64) -> FormattedRootRing {
    let parameters = parameters_with_slots_per_region(slots_per_region);
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    SparseBlockDevice::new(
                        DEVICE_SIZE_IN_BYTES,
                        PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_SIZE_IN_BYTES),
                    ),
                )
            })
            .collect();
    make_filesystem(&parameters, &mut devices).expect("两块内存盘上 mkfs");
    let pool = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: DEVICE_SIZE_IN_BYTES,
    };
    let system_configuration = choose_system_configuration(&pool).expect("mkfs 之后择得出系统配置");
    let genesis_root = choose_root(&pool, &system_configuration).expect("mkfs 种了第 0 代根");
    assert_eq!(
        (genesis_root.checkpoint_txg, genesis_root.instance),
        (GENESIS_CHECKPOINT_TXG, InstanceGeneration(0)),
        "前提：mkfs 写的根是第 0 代、实例代号 0"
    );
    let ring_slots = every_root_ring_slot(&system_configuration.immutable.sizes);
    assert_eq!(
        u64::try_from(ring_slots.len()).expect("槽数装得进 u64"),
        ROOT_RING_REGIONS * slots_per_region,
        "前提：读根环走 3S 个槽"
    );
    FormattedRootRing {
        slots_per_region,
        pool,
        system_configuration,
        genesis_root,
        ring_slots,
    }
}

impl FormattedRootRing {
    fn root_slot_size_in_bytes(&self) -> usize {
        usize::try_from(
            self.system_configuration
                .immutable
                .sizes
                .physical_block_size,
        )
        .expect("根槽宽装得进 usize")
    }

    /// 写进每一槽的 txg：0..=圈数 × 3S。
    fn txgs_written(&self) -> std::ops::RangeInclusive<u64> {
        0..=REVOLUTIONS_OF_THE_TXGS_WRITTEN * ROOT_RING_REGIONS * self.slots_per_region
    }

    /// 第 0 代根只改 txg 与实例代号的那一条根编码（读者只验自证、fsid 与落点，择根只看这两项）。
    fn root_encoding(&self, checkpoint_txg: u64, instance: u32) -> Vec<u8> {
        let mut root = self.genesis_root;
        root.checkpoint_txg = CheckpointTxg(checkpoint_txg);
        root.instance = InstanceGeneration(instance);
        root.to_slot(self.root_slot_size_in_bytes())
    }

    /// 一槽住在哪块盘的哪个偏移：区域归属表给盘，槽距给偏移（读根环的读者只从那块盘读这一槽）。
    fn device_and_offset_of(
        &self,
        ring_slot: RootRingSlot,
    ) -> (DeviceIdentity, DeviceOffsetInBytes) {
        let immutable = &self.system_configuration.immutable;
        (
            immutable.region_devices
                [usize::try_from(ring_slot.region).expect("区域号装得进 usize")],
            slot_offset(ring_slot, immutable.sizes.fixed_structure_slot_spacing),
        )
    }

    fn bytes_in_the_slot(&self, pool: &MemoryPool, ring_slot: RootRingSlot) -> Vec<u8> {
        let (device, offset) = self.device_and_offset_of(ring_slot);
        pool.devices
            .get(&device)
            .expect("池里有这块盘")
            .read(offset, self.root_slot_size_in_bytes())
    }

    fn put_bytes_into_the_slot(
        &self,
        pool: &mut MemoryPool,
        ring_slot: RootRingSlot,
        bytes: &[u8],
    ) {
        let (device, offset) = self.device_and_offset_of(ring_slot);
        pool.devices
            .get_mut(&device)
            .expect("池里有这块盘")
            .write(offset, bytes);
    }
}

/// 一槽里逐个写进的根编码，产品读槽各收没收。
#[derive(Clone, Debug, PartialEq, Eq)]
struct AdmissionsInOneSlot {
    slots_per_region: u64,
    ring_slot: RootRingSlot,
    encodings_written: u64,
    /// 读不出（不是自证不过）的次数：内存池里读盘内的一槽不该读不出，不为 0 说明读没读到写进去的字节。
    unreadable_readings: u64,
    /// 收下的根与写进去的那条 txg 或实例代号对不上的次数：不为 0 说明读者交回的不是这一槽里的根。
    admitted_roots_differing_from_the_encoding_written: u64,
    /// 收下的根编码 (txg, 实例代号)，按写的次序。
    admitted_encodings: Vec<(u64, u32)>,
}

/// 一槽里逐个写进每一条根编码、每写一条调一次产品读槽，判完把这一槽还原成判之前的字节。
fn admissions_in_one_slot(
    ring: &FormattedRootRing,
    pool: &mut MemoryPool,
    ring_slot: RootRingSlot,
) -> AdmissionsInOneSlot {
    let immutable = &ring.system_configuration.immutable;
    let bytes_before_the_judging = ring.bytes_in_the_slot(pool, ring_slot);
    let mut admissions = AdmissionsInOneSlot {
        slots_per_region: ring.slots_per_region,
        ring_slot,
        encodings_written: 0,
        unreadable_readings: 0,
        admitted_roots_differing_from_the_encoding_written: 0,
        admitted_encodings: Vec::new(),
    };
    // 迭代上界：(9S + 1) 个 txg × 4 种实例代号；跨轮不带状态（每一轮整槽覆写），没有提前出口。
    for checkpoint_txg in ring.txgs_written() {
        for instance in 0..INSTANCE_GENERATIONS_WRITTEN {
            ring.put_bytes_into_the_slot(
                pool,
                ring_slot,
                &ring.root_encoding(checkpoint_txg, instance),
            );
            admissions.encodings_written += 1;
            match read_root_ring_slot(
                &*pool,
                &immutable.region_devices,
                &immutable.sizes,
                &immutable.filesystem_identifier,
                ring_slot,
            ) {
                RootRingSlotReading::SelfVerified(root) => {
                    if (root.checkpoint_txg, root.instance)
                        != (CheckpointTxg(checkpoint_txg), InstanceGeneration(instance))
                    {
                        admissions.admitted_roots_differing_from_the_encoding_written += 1;
                    }
                    admissions
                        .admitted_encodings
                        .push((checkpoint_txg, instance));
                }
                RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable) => {
                    admissions.unreadable_readings += 1;
                }
                RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified) => {}
            }
        }
    }
    ring.put_bytes_into_the_slot(pool, ring_slot, &bytes_before_the_judging);
    admissions
}

/// 一片里的槽逐个判：每档 S 在这一片头一次用到时从 mkfs 之后的池克隆一份，这一片自己改、自己还原。
fn admissions_in_one_slice(
    rings: &[FormattedRootRing],
    slots_of_the_slice: &[(usize, RootRingSlot)],
    slice_label: usize,
) -> Vec<AdmissionsInOneSlot> {
    let mut pools_of_this_slice: BTreeMap<usize, MemoryPool> = BTreeMap::new();
    let mut admissions_of_the_slice = Vec::with_capacity(slots_of_the_slice.len());
    let mut last_progress = Instant::now();
    for (ring_index, ring_slot) in slots_of_the_slice {
        let ring = &rings[*ring_index];
        let pool = pools_of_this_slice
            .entry(*ring_index)
            .or_insert_with(|| ring.pool.clone());
        admissions_of_the_slice.push(admissions_in_one_slot(ring, pool, *ring_slot));
        if last_progress.elapsed().as_secs() >= PROGRESS_INTERVAL_SECONDS {
            println!(
                "ROOT_RING_READER_ADMISSION_PROGRESS slice={slice_label} slots_in_the_slice={} slots_judged={}",
                slots_of_the_slice.len(),
                admissions_of_the_slice.len()
            );
            last_progress = Instant::now();
        }
    }
    admissions_of_the_slice
}

/// 每档 S 的每一槽（档的次序、读根环的次序）按区间切成 `thread_count` 片并行判（`std::thread::scope`，每片自己一份池）；
/// 合并按片的次序，单线程与切片多线程交回的逐项相同。
fn admissions_judged_on_slices(
    rings: &[FormattedRootRing],
    thread_count: usize,
) -> Vec<AdmissionsInOneSlot> {
    let slots_to_judge: Vec<(usize, RootRingSlot)> = rings
        .iter()
        .enumerate()
        .flat_map(|(ring_index, ring)| {
            ring.ring_slots
                .iter()
                .map(move |ring_slot| (ring_index, *ring_slot))
        })
        .collect();
    let slice_length = slots_to_judge.len().div_ceil(thread_count.max(1)).max(1);
    let admissions_per_slice: Vec<Vec<AdmissionsInOneSlot>> = std::thread::scope(|scope| {
        let slice_threads: Vec<_> = slots_to_judge
            .chunks(slice_length)
            .enumerate()
            .map(|(slice_label, slots_of_the_slice)| {
                scope.spawn(move || admissions_in_one_slice(rings, slots_of_the_slice, slice_label))
            })
            .collect();
        slice_threads
            .into_iter()
            .map(|slice_thread| slice_thread.join().expect("切片线程没有 panic"))
            .collect()
    });
    admissions_per_slice.into_iter().flatten().collect()
}

/// 几档 S 各 mkfs 一次，按档切片并行；交回的次序同 `slots_per_region_of_the_rings`。
fn formatted_root_rings_of(
    slots_per_region_of_the_rings: &[u64],
    thread_count: usize,
) -> Vec<FormattedRootRing> {
    let slice_length = slots_per_region_of_the_rings
        .len()
        .div_ceil(thread_count.max(1))
        .max(1);
    std::thread::scope(|scope| {
        let slice_threads: Vec<_> = slots_per_region_of_the_rings
            .chunks(slice_length)
            .map(|slots_per_region_of_the_slice| {
                scope.spawn(move || {
                    slots_per_region_of_the_slice
                        .iter()
                        .map(|slots_per_region| formatted_root_ring(*slots_per_region))
                        .collect::<Vec<FormattedRootRing>>()
                })
            })
            .collect();
        slice_threads
            .into_iter()
            .flat_map(|slice_thread| slice_thread.join().expect("mkfs 的切片线程没有 panic"))
            .collect()
    })
}

/// 场景：S 取格式区间 4..=16 每一档，根环每一槽逐个写进每一条根编码（txg 0..=9S、实例代号 0..=3），调产品读槽。
/// 预期：txg 0 的根只在 mkfs 种根的三槽收、每一种实例代号都收；txg 不是 0 的根只在落点公式那一槽收，同一个 txg 恰好一槽收；
/// 收下的条数每档 4 × (9S + 3)。
/// 判别力：`crates/mutations.tsv`「C650 读者侧留档：第 0 代根的落点核放宽到每一槽」让 txg 0 那一条断言红；
/// 「C650 读者侧留档：落点核的槽号改成按位与 S − 1」让落点公式那一条断言红，红在 S 不是 2 的幂的那几档。
#[test]
fn every_slots_per_region_from_four_to_sixteen_admits_a_nonzero_txg_root_only_at_its_formula_slot_and_a_txg_zero_root_only_at_the_genesis_slots(
) {
    let thread_count = default_heavy_threads().get();
    let started = Instant::now();
    let slots_per_region_of_the_rings: Vec<u64> =
        (ROOT_RING_SLOTS_PER_REGION_MINIMUM..=ROOT_RING_SLOTS_PER_REGION_MAXIMUM).collect();
    let rings = formatted_root_rings_of(&slots_per_region_of_the_rings, thread_count);
    let genesis_slots = ring_slots_where_make_filesystem_seeds_the_genesis_root();
    assert_eq!(
        genesis_root_ring_slots().collect::<BTreeSet<RootRingSlot>>(),
        genesis_slots,
        "前提：产品的 mkfs 种根槽与条款那三槽相同"
    );
    for ring in &rings {
        for checkpoint_txg in ring.txgs_written() {
            assert_eq!(
                target_for_publish(
                    CheckpointTxg(checkpoint_txg),
                    ring.system_configuration
                        .immutable
                        .sizes
                        .root_ring_slots_per_region
                ),
                ring_slot_given_by_the_publish_formula(checkpoint_txg, ring.slots_per_region),
                "前提：产品写者的落点与条款的落点公式相同（S = {}，txg {checkpoint_txg}）",
                ring.slots_per_region
            );
        }
    }
    let admissions = admissions_judged_on_slices(&rings, thread_count);
    let mut slots_admitting_the_encoding: BTreeMap<(u64, u64, u32), BTreeSet<RootRingSlot>> =
        BTreeMap::new();
    for admissions_in_the_slot in &admissions {
        for (checkpoint_txg, instance) in &admissions_in_the_slot.admitted_encodings {
            slots_admitting_the_encoding
                .entry((
                    admissions_in_the_slot.slots_per_region,
                    *checkpoint_txg,
                    *instance,
                ))
                .or_default()
                .insert(admissions_in_the_slot.ring_slot);
        }
    }
    let encodings_written: u64 = admissions
        .iter()
        .map(|admissions_in_the_slot| admissions_in_the_slot.encodings_written)
        .sum();
    let unreadable_readings: u64 = admissions
        .iter()
        .map(|admissions_in_the_slot| admissions_in_the_slot.unreadable_readings)
        .sum();
    let admitted_roots_differing_from_the_encoding_written: u64 = admissions
        .iter()
        .map(|admissions_in_the_slot| {
            admissions_in_the_slot.admitted_roots_differing_from_the_encoding_written
        })
        .sum();
    let encodings_admitted: u64 = admissions
        .iter()
        .map(|admissions_in_the_slot| {
            u64::try_from(admissions_in_the_slot.admitted_encodings.len()).expect("条数装得进 u64")
        })
        .sum();
    let encodings_admitted_by_the_clause: u64 = rings
        .iter()
        .map(|ring| {
            u64::from(INSTANCE_GENERATIONS_WRITTEN)
                * (REVOLUTIONS_OF_THE_TXGS_WRITTEN * ROOT_RING_REGIONS * ring.slots_per_region
                    + ROOT_RING_REGIONS)
        })
        .sum();
    let mut txg_zero_encodings_off_the_genesis_slots: Vec<String> = Vec::new();
    let mut nonzero_txg_encodings_off_their_formula_slot: Vec<String> = Vec::new();
    let mut slots_per_region_admitting_off_the_formula_slot: BTreeSet<u64> = BTreeSet::new();
    let mut nonzero_txg_encodings_not_admitted_in_exactly_one_slot: Vec<String> = Vec::new();
    for ring in &rings {
        for checkpoint_txg in ring.txgs_written() {
            for instance in 0..INSTANCE_GENERATIONS_WRITTEN {
                let admitting_slots = slots_admitting_the_encoding
                    .get(&(ring.slots_per_region, checkpoint_txg, instance))
                    .cloned()
                    .unwrap_or_default();
                let description = format!(
                    "S={} txg={checkpoint_txg} instance={instance} admitted_at={admitting_slots:?}",
                    ring.slots_per_region
                );
                if checkpoint_txg == GENESIS_CHECKPOINT_TXG.0 {
                    if admitting_slots != genesis_slots {
                        txg_zero_encodings_off_the_genesis_slots.push(description);
                    }
                    continue;
                }
                if admitting_slots.len() != 1 {
                    nonzero_txg_encodings_not_admitted_in_exactly_one_slot
                        .push(description.clone());
                }
                if admitting_slots
                    != BTreeSet::from([ring_slot_given_by_the_publish_formula(
                        checkpoint_txg,
                        ring.slots_per_region,
                    )])
                {
                    slots_per_region_admitting_off_the_formula_slot.insert(ring.slots_per_region);
                    nonzero_txg_encodings_off_their_formula_slot.push(description);
                }
            }
        }
    }
    println!(
        "ROOT_RING_READER_ADMISSION slots_per_region={}..={} threads={thread_count} encodings_written={encodings_written} encodings_admitted={encodings_admitted} unreadable_readings={unreadable_readings} admitted_roots_differing_from_the_encoding_written={admitted_roots_differing_from_the_encoding_written} txg_zero_encodings_off_the_genesis_slots={} nonzero_txg_encodings_off_their_formula_slot={} slots_per_region_admitting_off_the_formula_slot={slots_per_region_admitting_off_the_formula_slot:?} nonzero_txg_encodings_not_admitted_in_exactly_one_slot={} seconds={:.2}",
        ROOT_RING_SLOTS_PER_REGION_MINIMUM,
        ROOT_RING_SLOTS_PER_REGION_MAXIMUM,
        txg_zero_encodings_off_the_genesis_slots.len(),
        nonzero_txg_encodings_off_their_formula_slot.len(),
        nonzero_txg_encodings_not_admitted_in_exactly_one_slot.len(),
        started.elapsed().as_secs_f64(),
    );
    assert_eq!(
        encodings_written, ENCODINGS_WRITTEN_OVER_EVERY_SLOTS_PER_REGION,
        "穷举的域：S 取 4..=16 每一档、每一槽、txg 0..=9S、实例代号 0..=3，一条都不少"
    );
    assert_eq!(
        unreadable_readings, 0,
        "内存池里读盘内的根槽不该读不出：读不出说明读者没读到写进去的字节，下面几条断言没有判别力"
    );
    assert_eq!(
        admitted_roots_differing_from_the_encoding_written, 0,
        "读者收下的根要是这一槽里刚写进去的那一条"
    );
    assert!(
        txg_zero_encodings_off_the_genesis_slots.is_empty(),
        "读者侧落点核：txg 0 的根只在 mkfs 种根的三槽（三个区域各自的槽 0）收、每一种实例代号都收；对不上的 {} 条，前几条：{:?}",
        txg_zero_encodings_off_the_genesis_slots.len(),
        &txg_zero_encodings_off_the_genesis_slots[..txg_zero_encodings_off_the_genesis_slots.len().min(MISMATCHES_LISTED_IN_A_FAILURE)]
    );
    assert!(
        nonzero_txg_encodings_off_their_formula_slot.is_empty(),
        "读者侧落点核：txg 不是 0 的根只在落点公式那一槽收；对不上的 {} 条，落在 S = {:?} 这几档，前几条：{:?}",
        nonzero_txg_encodings_off_their_formula_slot.len(),
        slots_per_region_admitting_off_the_formula_slot,
        &nonzero_txg_encodings_off_their_formula_slot[..nonzero_txg_encodings_off_their_formula_slot.len().min(MISMATCHES_LISTED_IN_A_FAILURE)]
    );
    assert!(
        nonzero_txg_encodings_not_admitted_in_exactly_one_slot.is_empty(),
        "同一个不是 0 的 txg 恰好一槽收；对不上的 {} 条，前几条：{:?}",
        nonzero_txg_encodings_not_admitted_in_exactly_one_slot.len(),
        &nonzero_txg_encodings_not_admitted_in_exactly_one_slot
            [..nonzero_txg_encodings_not_admitted_in_exactly_one_slot
                .len()
                .min(MISMATCHES_LISTED_IN_A_FAILURE)]
    );
    assert_eq!(
        encodings_admitted, encodings_admitted_by_the_clause,
        "收下的条数：每档 4 种实例代号 × (9S 个不是 0 的 txg 各一槽 + txg 0 的三槽)"
    );
}

/// 场景：S = 5 那一档（不是 2 的幂）的根环每一槽，单线程判一遍、切成至少两片多线程判一遍。
/// 预期：两遍交回的逐项相同（合并按片的次序，`.claude/rules/test-script-discipline.md`「编译与算法都要快」）。
/// 判别力：`crates/mutations.tsv`「C650 读者侧留档：切片合并倒着接」让这一条红。
#[test]
fn admissions_judged_on_slices_equal_admissions_judged_on_one_thread() {
    let rings = [formatted_root_ring(
        SLOTS_PER_REGION_OF_THE_SLICED_CROSS_CHECK,
    )];
    let thread_count_of_the_slices = default_heavy_threads().get().max(2);
    let on_one_thread = admissions_judged_on_slices(&rings, 1);
    let on_slices = admissions_judged_on_slices(&rings, thread_count_of_the_slices);
    println!(
        "ROOT_RING_READER_ADMISSION_CROSS_CHECK slots_per_region={SLOTS_PER_REGION_OF_THE_SLICED_CROSS_CHECK} threads={thread_count_of_the_slices} slots={} slices_equal_one_thread={}",
        on_one_thread.len(),
        on_slices == on_one_thread
    );
    assert_eq!(
        u64::try_from(on_one_thread.len()).expect("槽数装得进 u64"),
        ROOT_RING_REGIONS * SLOTS_PER_REGION_OF_THE_SLICED_CROSS_CHECK,
        "单线程那一遍判到这一档根环的每一槽"
    );
    assert_eq!(
        on_slices, on_one_thread,
        "切片多线程与单线程判出的要逐项相同、次序相同"
    );
}
