//! 攻方腿模型（三方轮 m3-close-b19-e165-r3，攻 C650 的新判据）：新判据说「承诺内的合法写者造不出让三条择根变体红的盘面，
//! 等价于『同一非 0 txg 至多一条根进择根』」。这份模型在真池的根环上判这句的两个方向、留档用例的 S 取样够不够、写者一侧那一半有没有用例看着。
//!
//! 复跑时拷进副本的 crates/singlefs-harness/tests/（harness 档测试目标）。盘是内存稀疏盘（SparseBlockDevice），每个几何只 mkfs 一次，之后从那一份克隆。
//! 不调崩溃枚举函数：四条用例都不造崩溃状态（写者那一条只读可写挂载做完、或挂载报错之后盘上留下的根环）。
//!
//! 四条用例：
//!   e165_r3_window_disks_split_the_equivalence        S ∈ {4, 16}：txg 0..=5 落的那六槽逐槽穷举（空、本槽 txg、本槽 txg + 3S、种根的两槽另摆 txg 0；
//!                                                      实例 0..=2），产品读槽 + 产品 choose_root + 本地 today / v1823 / v1825 / v1826；按「同一非 0 txg 至多一条」
//!                                                      「txg 0 只有一种实例代号」「进择根的根不倒挂」三个条件分组，数各变体与 today 择得不同的盘面；
//!                                                      三条变体各自「与 today 不同」的充要条件逐盘核。
//!   e165_r3_window_disks_sliced_matches_single_thread S = 4 的前 22400 个盘面单线程与切片多线程逐项相同。
//!   e165_r3_admission_over_every_slots_per_region     S 取格式区间 4..=16 每一档：每槽 × txg 0..6S+3 × 实例 0..=1 写进一条、调产品读槽，
//!                                                      核非 0 txg 恰好只在落点槽、txg 0 恰好只在种根的三槽、读得出与否不随实例代号变。
//!   e165_r3_writable_mounts_on_a_fresh_pool           mkfs 之后可写挂载、再可写挂载一次：每次之后读根环，数 txg 0 上有哪几种实例代号、
//!                                                      三条变体与 today 择不择同一条，池级 checker 红哪几条。
//!
//! 会红的对照（跑前写死，见 assert）：三条变体各有一条错的充要条件，逐盘对不上的盘数要大于 0（报 0 说明逐盘核没判别力，模型作废）；
//! 产品 choose_root 与本地 today 在每个盘面上择同一条（不同说明本地 today 抄错了）。另两条用例（逐档穷举、可写挂载）在没改过的树上该绿，
//! 在改坏读者或写者的副本上该红：哪一条改坏红哪一条写在报告「跑前写死的读法」一节。
#![allow(clippy::too_many_lines, reason = "模型一份文件，摆法、读法与计数写在一起")]

use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{
    genesis_root_ring_slots, make_filesystem, MakeFilesystemParameters,
    FIRST_VERSION_REGION_DEVICES, GENESIS_CHECKPOINT_TXG,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, every_root_ring_slot, read_root_ring_slot,
    RootRingSlotReading,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot, RootRingSlotsPerRegion};
use singlefs_core::system_configuration::{SystemConfiguration, SystemImmutableSizes};
use singlefs_format::{
    JOURNAL_RING_DEFAULT_BYTES, ROOT_RING_SLOTS_PER_REGION_MAXIMUM, ROOT_RING_SLOTS_PER_REGION_MINIMUM,
};
use singlefs_harness::heavy_threads::default_heavy_threads;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

/// 每块盘的字节数：稀疏盘，只存写过的扇区。
const DEVICE_BYTES: u64 = 4 << 30;
/// 根槽宽与扇区宽。
const ROOT_SLOT_BYTES: usize = 512;
const FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x45, 0x31, 0x36, 0x35, 0x2d, 0x72, 0x33, 0x2d, 0x61, 0x74, 0x74, 0x61, 0x63, 0x6b, 0x00, 0x01,
];
/// 逐槽穷举那条用例取的两档 S：格式区间的下沿与上沿。
const WINDOW_SLOTS_PER_REGION_SAMPLED: [u64; 2] = [4, 16];
/// 逐槽穷举摆的槽：txg 0..WINDOW_TXGS 各自的落点槽（S ≥ 2 时是三个区域各自的槽 0 与槽 1）。
const WINDOW_TXGS: u64 = 6;
/// 逐槽穷举根编码里取的实例代号：0..WINDOW_INSTANCES。
const WINDOW_INSTANCES: u32 = 3;
/// 单线程与切片多线程对拍的盘面数（S = 4 的前这么多个）。
const CROSS_CHECK_DISKS: u64 = 22_400;
/// 逐档穷举根编码里取的实例代号：0..ADMISSION_INSTANCES（落点公式不读实例代号，两种够判「随不随实例代号变」）。
const ADMISSION_INSTANCES: u32 = 2;
/// 可写挂载那条用例的 S：mkfs 显式入口常取的 8。
const MOUNT_SLOTS_PER_REGION: u64 = 8;
/// 可写挂载那条用例挂几次（每次之后读一遍根环）。
const WRITABLE_MOUNTS: u32 = 2;
/// 单项超过这么久每片报一行进度（`.claude/rules/test-script-discipline.md`「编译与算法都要快」）。
const PROGRESS_INTERVAL_SECONDS: u64 = 300;

fn parameters_with(slots_per_region: u64) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: FIRST_VERSION_REGION_DEVICES,
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(
                slots_per_region,
            )
            .expect("取样的 S 在格式区间里"),
        },
    }
}

/// 两块内存盘上 mkfs 一次，交回那两块盘（可写挂载要拿它们挂）。
fn formatted_devices(parameters: &MakeFilesystemParameters) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            (
                identity,
                SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    make_filesystem(parameters, &mut devices).expect("两块内存盘上 mkfs");
    devices
}

/// 盘此刻的字节照抄一份成读者用的池。
fn memory_pool_of(devices: &[(DeviceIdentity, SparseBlockDevice)]) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: DEVICE_BYTES,
    }
}

/// mkfs 之后的池、择到的系统配置与 mkfs 写的第 0 代根（根编码都从它改 txg 与实例代号得来）。
#[derive(Clone)]
struct FormattedRing {
    pool: MemoryPool,
    system_configuration: SystemConfiguration,
    genesis_root: RootRecord,
    ring_slots: Vec<RootRingSlot>,
}

fn formatted_ring(slots_per_region: u64) -> FormattedRing {
    let pool = memory_pool_of(&formatted_devices(&parameters_with(slots_per_region)));
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

    fn device_and_offset(&self, ring_slot: RootRingSlot) -> (DeviceIdentity, DeviceOffsetInBytes) {
        let immutable = &self.system_configuration.immutable;
        (
            immutable.region_devices[usize::try_from(ring_slot.region).expect("区域号")],
            slot_offset(ring_slot, immutable.sizes.fixed_structure_slot_spacing),
        )
    }

    /// 往一槽里摆字节：写进这一槽的区域归属那块盘（读根环的读者只从那块盘读它）。
    fn put_bytes(&self, pool: &mut MemoryPool, ring_slot: RootRingSlot, bytes: &[u8]) {
        let (device, offset) = self.device_and_offset(ring_slot);
        pool.devices
            .get_mut(&device)
            .expect("池里有这块盘")
            .write(offset, bytes);
    }

    fn raw_bytes(&self, pool: &MemoryPool, ring_slot: RootRingSlot) -> Vec<u8> {
        let (device, offset) = self.device_and_offset(ring_slot);
        pool.devices
            .get(&device)
            .expect("池里有这块盘")
            .read(offset, ROOT_SLOT_BYTES)
    }

    /// 产品读槽（`recovery::read_root_ring_slot`：验自证、fsid，再核落点公式）交出的根。
    fn product_reading(&self, pool: &MemoryPool, ring_slot: RootRingSlot) -> Option<RootRecord> {
        let immutable = &self.system_configuration.immutable;
        match read_root_ring_slot(
            pool,
            &immutable.region_devices,
            &immutable.sizes,
            &immutable.filesystem_identifier,
            ring_slot,
        ) {
            RootRingSlotReading::SelfVerified(root) => Some(root),
            RootRingSlotReading::Bad(_) => None,
        }
    }
}

/// 本地四条择根序（Verus 必红世界表 C650 那几行与「比较不看实例代号」那一行的替换文各写一份）：读根环次序里往后走，后来的严格「更新」才换。
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

    fn label(self) -> &'static str {
        match self {
            Self::Today => "today",
            Self::IgnoresTheInstance => "v1823",
            Self::TieTakesTheLowerInstance => "v1825",
            Self::InstanceFirstThenTxg => "v1826",
        }
    }

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
            if chosen.is_none_or(|already| self.replaces(key_of(root), key_of(&already))) {
                chosen = Some(*root);
            }
        }
        chosen
    }
}

fn key_of(root: &RootRecord) -> (u64, u32) {
    (root.checkpoint_txg.0, root.instance.0)
}

/// txg 为 `window_txg` 的落点槽里摆什么：空，或一条根（txg、实例代号）。本槽 txg 与本槽 txg + 3S 都落这一槽；
/// 种根的区域 1、区域 2 槽 0（txg 1、2 的落点槽）另可摆 txg 0 的根（第 0 代根照 mkfs 种根的槽另认）；区域 0 槽 0 本来就是 txg 0 的落点。
fn window_options(window_txg: u64, slots_per_region: u64) -> Vec<Option<(u64, u32)>> {
    let ring_length = 3 * slots_per_region;
    let mut options: Vec<Option<(u64, u32)>> = vec![None];
    for instance in 0..WINDOW_INSTANCES {
        options.push(Some((window_txg, instance)));
    }
    if window_txg < 3 {
        for instance in 0..WINDOW_INSTANCES {
            options.push(Some((window_txg + ring_length, instance)));
        }
    }
    if window_txg == 1 || window_txg == 2 {
        for instance in 0..WINDOW_INSTANCES {
            options.push(Some((0, instance)));
        }
    }
    options
}

fn window_disk_count(slots_per_region: u64) -> u64 {
    (0..WINDOW_TXGS)
        .map(|window_txg| u64::try_from(window_options(window_txg, slots_per_region).len()).expect("选项数"))
        .product()
}

/// 盘面号按混合进制拆成六槽各摆什么（次序同 txg 0..WINDOW_TXGS）。
fn window_placement(disk: u64, slots_per_region: u64) -> Vec<Option<(u64, u32)>> {
    let mut rest = disk;
    (0..WINDOW_TXGS)
        .map(|window_txg| {
            let options = window_options(window_txg, slots_per_region);
            let radix = u64::try_from(options.len()).expect("选项数");
            let choice = usize::try_from(rest % radix).expect("下标");
            rest /= radix;
            options[choice]
        })
        .collect()
}

fn window_slots(ring: &FormattedRing) -> Vec<RootRingSlot> {
    (0..WINDOW_TXGS)
        .map(|window_txg| target_for_publish(CheckpointTxg(window_txg), ring.slots_per_region()))
        .collect()
}

/// 进择根的根（读根环次序）满不满足三个条件。
fn nonzero_txg_at_most_one(admitted: &[(u64, u32)]) -> bool {
    let mut seen: BTreeSet<u64> = BTreeSet::new();
    admitted.iter().filter(|(txg, _)| *txg != 0).all(|(txg, _)| seen.insert(*txg))
}

fn txg_zero_one_instance(admitted: &[(u64, u32)]) -> bool {
    admitted
        .iter()
        .filter(|(txg, _)| *txg == 0)
        .map(|(_, instance)| *instance)
        .collect::<BTreeSet<u32>>()
        .len()
        <= 1
}

/// 不倒挂：任两条，txg 小的那条实例代号不比 txg 大的那条高。
fn no_inversion(admitted: &[(u64, u32)]) -> bool {
    admitted.iter().all(|(lower_txg, lower_instance)| {
        admitted
            .iter()
            .all(|(higher_txg, higher_instance)| !(lower_txg < higher_txg && lower_instance > higher_instance))
    })
}

/// 三条变体（次序同 `ChoiceOrder::COMPARED_WITH_TODAY`）与 today 择得不同的充要条件，`correct` 取真那一组，否则取三条错的（阳性对照）。
/// 真的：v1823 ⟺ 最大 txg 上读根环次序最前那条不是那个 txg 上实例代号最高的；v1825 ⟺ 最大 txg 上有两种实例代号；
/// v1826 ⟺ 有一条根的实例代号高于 today 所选那条。错的：v1823 取 v1825 那一句；v1825 取「最大 txg 上有两条根」（不论实例）；v1826 取「有倒挂」。
fn characterized_differences(admitted: &[(u64, u32)], today: Option<(u64, u32)>, correct: bool) -> [bool; 3] {
    let Some(newest_txg) = admitted.iter().map(|(txg, _)| *txg).max() else {
        return [false; 3];
    };
    let at_newest: Vec<u32> = admitted
        .iter()
        .filter(|(txg, _)| *txg == newest_txg)
        .map(|(_, instance)| *instance)
        .collect();
    let highest_at_newest = at_newest.iter().copied().max().expect("最大 txg 上至少一条");
    let distinct_at_newest = at_newest.iter().copied().collect::<BTreeSet<u32>>().len();
    let today_instance = today.expect("有根就择得出").1;
    if correct {
        [
            at_newest[0] < highest_at_newest,
            distinct_at_newest >= 2,
            admitted.iter().any(|(_, instance)| *instance > today_instance),
        ]
    } else {
        [distinct_at_newest >= 2, at_newest.len() >= 2, !no_inversion(admitted)]
    }
}

/// 一段盘面号上的计数。合并按片的次序相加，「最小盘面号」取各片里最小的。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct WindowTally {
    disks: u64,
    placed_but_not_admitted: u64,
    admitted_outside_the_window: u64,
    product_choose_root_differs_from_today: u64,
    nonzero_txg_at_most_one: u64,
    txg_zero_one_instance: u64,
    no_inversion: u64,
    txg_zero_one_instance_and_no_inversion: u64,
    /// 下面每一项三格，次序同 `ChoiceOrder::COMPARED_WITH_TODAY`。
    differs: [u64; 3],
    differs_given_nonzero_txg_at_most_one: [u64; 3],
    differs_given_txg_zero_one_instance: [u64; 3],
    differs_given_no_inversion: [u64; 3],
    differs_given_txg_zero_one_instance_and_no_inversion: [u64; 3],
    characterization_mismatches: [u64; 3],
    wrong_characterization_mismatches: [u64; 3],
    smallest_disk_differing: [Option<u64>; 3],
    smallest_disk_differing_given_txg_zero_one_instance: [Option<u64>; 3],
}

fn smaller_disk(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (left, None) => left,
        (None, right) => right,
    }
}

impl WindowTally {
    fn merge(&mut self, slice: &Self) {
        self.disks += slice.disks;
        self.placed_but_not_admitted += slice.placed_but_not_admitted;
        self.admitted_outside_the_window += slice.admitted_outside_the_window;
        self.product_choose_root_differs_from_today += slice.product_choose_root_differs_from_today;
        self.nonzero_txg_at_most_one += slice.nonzero_txg_at_most_one;
        self.txg_zero_one_instance += slice.txg_zero_one_instance;
        self.no_inversion += slice.no_inversion;
        self.txg_zero_one_instance_and_no_inversion += slice.txg_zero_one_instance_and_no_inversion;
        for variant in 0..3 {
            self.differs[variant] += slice.differs[variant];
            self.differs_given_nonzero_txg_at_most_one[variant] += slice.differs_given_nonzero_txg_at_most_one[variant];
            self.differs_given_txg_zero_one_instance[variant] += slice.differs_given_txg_zero_one_instance[variant];
            self.differs_given_no_inversion[variant] += slice.differs_given_no_inversion[variant];
            self.differs_given_txg_zero_one_instance_and_no_inversion[variant] +=
                slice.differs_given_txg_zero_one_instance_and_no_inversion[variant];
            self.characterization_mismatches[variant] += slice.characterization_mismatches[variant];
            self.wrong_characterization_mismatches[variant] += slice.wrong_characterization_mismatches[variant];
            self.smallest_disk_differing[variant] =
                smaller_disk(self.smallest_disk_differing[variant], slice.smallest_disk_differing[variant]);
            self.smallest_disk_differing_given_txg_zero_one_instance[variant] = smaller_disk(
                self.smallest_disk_differing_given_txg_zero_one_instance[variant],
                slice.smallest_disk_differing_given_txg_zero_one_instance[variant],
            );
        }
    }
}

/// 一个盘面摆进池（六槽整份覆写，窗口外的槽保持 mkfs 之后的全零），交回产品读槽建的择根入参。
fn place_window_disk(ring: &FormattedRing, pool: &mut MemoryPool, window: &[RootRingSlot], disk: u64) -> Vec<Option<(u64, u32)>> {
    let placement = window_placement(disk, ring.slots_per_region().count());
    for (ring_slot, option) in window.iter().zip(&placement) {
        let bytes = match option {
            None => vec![0u8; ROOT_SLOT_BYTES],
            Some((checkpoint_txg, instance)) => ring.root_with(*checkpoint_txg, *instance).to_slot(ROOT_SLOT_BYTES),
        };
        ring.put_bytes(pool, *ring_slot, &bytes);
    }
    placement
}

fn judge_window_disks(ring: &FormattedRing, disks: std::ops::Range<u64>, slice_label: usize) -> WindowTally {
    let mut pool = ring.pool.clone();
    let window = window_slots(ring);
    let mut tally = WindowTally::default();
    let mut last_progress = Instant::now();
    for disk in disks.clone() {
        let placement = place_window_disk(ring, &mut pool, &window, disk);
        let choice_input: Vec<Option<RootRecord>> = ring
            .ring_slots
            .iter()
            .map(|ring_slot| ring.product_reading(&pool, *ring_slot))
            .collect();
        for (ring_slot, option) in window.iter().zip(&placement) {
            let index = ring.ring_slots.iter().position(|slot| slot == ring_slot).expect("窗口的槽在根环里");
            if option.is_some() && choice_input[index].is_none() {
                tally.placed_but_not_admitted += 1;
            }
        }
        for (ring_slot, reading) in ring.ring_slots.iter().zip(&choice_input) {
            if reading.is_some() && !window.contains(ring_slot) {
                tally.admitted_outside_the_window += 1;
            }
        }
        let admitted: Vec<(u64, u32)> = choice_input.iter().flatten().map(key_of).collect();
        tally.disks += 1;
        let today = ChoiceOrder::Today.chosen_root(&choice_input);
        if choose_root(&pool, &ring.system_configuration) != today {
            tally.product_choose_root_differs_from_today += 1;
        }
        let at_most_one = nonzero_txg_at_most_one(&admitted);
        let one_instance = txg_zero_one_instance(&admitted);
        let not_inverted = no_inversion(&admitted);
        tally.nonzero_txg_at_most_one += u64::from(at_most_one);
        tally.txg_zero_one_instance += u64::from(one_instance);
        tally.no_inversion += u64::from(not_inverted);
        tally.txg_zero_one_instance_and_no_inversion += u64::from(one_instance && not_inverted);
        let today_key = today.as_ref().map(key_of);
        let characterized = characterized_differences(&admitted, today_key, true);
        let wrongly_characterized = characterized_differences(&admitted, today_key, false);
        for (variant, order) in ChoiceOrder::COMPARED_WITH_TODAY.into_iter().enumerate() {
            let differs = order.chosen_root(&choice_input) != today;
            if characterized[variant] != differs {
                tally.characterization_mismatches[variant] += 1;
            }
            if wrongly_characterized[variant] != differs {
                tally.wrong_characterization_mismatches[variant] += 1;
            }
            if !differs {
                continue;
            }
            tally.differs[variant] += 1;
            tally.smallest_disk_differing[variant] = smaller_disk(tally.smallest_disk_differing[variant], Some(disk));
            tally.differs_given_nonzero_txg_at_most_one[variant] += u64::from(at_most_one);
            tally.differs_given_no_inversion[variant] += u64::from(not_inverted);
            tally.differs_given_txg_zero_one_instance_and_no_inversion[variant] += u64::from(one_instance && not_inverted);
            if one_instance {
                tally.differs_given_txg_zero_one_instance[variant] += 1;
                tally.smallest_disk_differing_given_txg_zero_one_instance[variant] = smaller_disk(
                    tally.smallest_disk_differing_given_txg_zero_one_instance[variant],
                    Some(disk),
                );
            }
        }
        if last_progress.elapsed().as_secs() >= PROGRESS_INTERVAL_SECONDS {
            println!(
                "OPUSMODEL name=progress slice={slice_label} range={}..{} judged={}",
                disks.start,
                disks.end,
                disk + 1 - disks.start
            );
            last_progress = Instant::now();
        }
    }
    tally
}

/// 盘面号区间按线程数切片并行（`std::thread::scope`），每片自己一份池；合并按片的次序。
fn judge_window_disks_sliced(ring: &FormattedRing, disks: std::ops::Range<u64>, threads: usize) -> WindowTally {
    let thread_count = u64::try_from(threads).expect("线程数").max(1);
    let length = disks.end - disks.start;
    let slice_length = length.div_ceil(thread_count);
    let slices: Vec<std::ops::Range<u64>> = (0..thread_count)
        .map(|slice| {
            (disks.start + slice * slice_length).min(disks.end)..(disks.start + (slice + 1) * slice_length).min(disks.end)
        })
        .filter(|range| !range.is_empty())
        .collect();
    let per_slice: Vec<WindowTally> = std::thread::scope(|scope| {
        let handles: Vec<_> = slices
            .iter()
            .cloned()
            .enumerate()
            .map(|(slice_label, range)| scope.spawn(move || judge_window_disks(ring, range, slice_label)))
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().expect("切片线程没有 panic"))
            .collect()
    });
    let mut merged = WindowTally::default();
    for slice in &per_slice {
        merged.merge(slice);
    }
    merged
}

fn key_text(key: Option<(u64, u32)>) -> String {
    key.map_or_else(|| "none".to_owned(), |(txg, instance)| format!("({txg},{instance})"))
}

/// 一个盘面的摆法与四条择根序各择哪条，打成一行（复现那一盘只要 S 与盘面号）。
fn window_example_line(ring: &FormattedRing, disk: u64, why: &str) -> String {
    let mut pool = ring.pool.clone();
    let window = window_slots(ring);
    let placement = place_window_disk(ring, &mut pool, &window, disk);
    let roots: Vec<String> = window
        .iter()
        .zip(&placement)
        .filter_map(|(ring_slot, option)| {
            option.map(|key| format!("r{}s{}:{}", ring_slot.region, ring_slot.slot, key_text(Some(key))))
        })
        .collect();
    let choice_input: Vec<Option<RootRecord>> = ring
        .ring_slots
        .iter()
        .map(|ring_slot| ring.product_reading(&pool, *ring_slot))
        .collect();
    let choices: Vec<String> = [ChoiceOrder::Today]
        .into_iter()
        .chain(ChoiceOrder::COMPARED_WITH_TODAY)
        .map(|order| format!("{}={}", order.label(), key_text(order.chosen_root(&choice_input).as_ref().map(key_of))))
        .collect();
    format!(
        "OPUSMODEL name=window_example slots_per_region={} why={why} disk={disk} roots=[{}] {} product_choose_root={}",
        ring.slots_per_region().count(),
        roots.join(" "),
        choices.join(" "),
        key_text(choose_root(&pool, &ring.system_configuration).as_ref().map(key_of)),
    )
}

fn print_window_tally(slots_per_region: u64, threads: usize, tally: &WindowTally, seconds: f64) {
    println!(
        "OPUSMODEL name=window_disks slots_per_region={slots_per_region} disks={} threads={threads} placed_but_not_admitted={} admitted_outside_the_window={} product_choose_root_differs_from_today={} nonzero_txg_at_most_one={} txg_zero_one_instance={} no_inversion={} txg_zero_one_instance_and_no_inversion={} differs[v1823,v1825,v1826]={:?} differs_given_nonzero_txg_at_most_one={:?} differs_given_txg_zero_one_instance={:?} differs_given_no_inversion={:?} differs_given_txg_zero_one_instance_and_no_inversion={:?} characterization_mismatches={:?} wrong_characterization_mismatches={:?} seconds={seconds:.2}",
        tally.disks,
        tally.placed_but_not_admitted,
        tally.admitted_outside_the_window,
        tally.product_choose_root_differs_from_today,
        tally.nonzero_txg_at_most_one,
        tally.txg_zero_one_instance,
        tally.no_inversion,
        tally.txg_zero_one_instance_and_no_inversion,
        tally.differs,
        tally.differs_given_nonzero_txg_at_most_one,
        tally.differs_given_txg_zero_one_instance,
        tally.differs_given_no_inversion,
        tally.differs_given_txg_zero_one_instance_and_no_inversion,
        tally.characterization_mismatches,
        tally.wrong_characterization_mismatches,
    );
}

#[test]
fn e165_r3_window_disks_split_the_equivalence() {
    let threads = default_heavy_threads().get();
    for slots_per_region in WINDOW_SLOTS_PER_REGION_SAMPLED {
        let ring = formatted_ring(slots_per_region);
        let disks = window_disk_count(slots_per_region);
        let started = Instant::now();
        let tally = judge_window_disks_sliced(&ring, 0..disks, threads);
        print_window_tally(slots_per_region, threads, &tally, started.elapsed().as_secs_f64());
        for (variant, order) in ChoiceOrder::COMPARED_WITH_TODAY.into_iter().enumerate() {
            if let Some(disk) = tally.smallest_disk_differing[variant] {
                println!("{}", window_example_line(&ring, disk, &format!("smallest_disk_where_{}_differs", order.label())));
            }
            if let Some(disk) = tally.smallest_disk_differing_given_txg_zero_one_instance[variant] {
                println!(
                    "{}",
                    window_example_line(&ring, disk, &format!("smallest_disk_where_{}_differs_with_one_txg_zero_instance", order.label()))
                );
            }
        }
        assert_eq!(tally.disks, disks, "盘面数要等于窗口的全域");
        assert_eq!(tally.placed_but_not_admitted, 0, "摆在落点槽（txg 0 在种根槽）的根产品读槽都要收；不收说明摆法错了");
        assert_eq!(tally.admitted_outside_the_window, 0, "窗口外的槽是全零，读不出根");
        assert_eq!(
            tally.product_choose_root_differs_from_today, 0,
            "产品 choose_root 与本地 today 要择同一条；不同说明本地 today 抄错了，变体的比较作废"
        );
        assert_eq!(tally.characterization_mismatches, [0, 0, 0], "三条变体各自的充要条件要逐盘对上");
        assert!(
            tally.wrong_characterization_mismatches.iter().all(|mismatches| *mismatches > 0),
            "阳性对照：三条错的条件都要有对不上的盘；报 0 说明这个窗口分不出对错，逐盘核没判别力"
        );
    }
}

#[test]
fn e165_r3_window_disks_sliced_matches_single_thread() {
    let threads = default_heavy_threads().get();
    let ring = formatted_ring(4);
    let single = judge_window_disks_sliced(&ring, 0..CROSS_CHECK_DISKS, 1);
    let sliced = judge_window_disks_sliced(&ring, 0..CROSS_CHECK_DISKS, threads);
    println!(
        "OPUSMODEL name=window_cross_check slots_per_region=4 disks={CROSS_CHECK_DISKS} threads={threads} single_thread_equals_sliced={} single_differs={:?} sliced_differs={:?}",
        single == sliced,
        single.differs,
        sliced.differs,
    );
    assert_eq!(single, sliced, "单线程与切片多线程的计数要逐项相同");
}

/// 一档 S 的逐槽读数：每个 txg 在哪几槽（读根环次序的下标）读得出自证过的根，以及读得出与否随不随实例代号变。
fn admission_line_of_one_geometry(slots_per_region: u64) -> (String, bool) {
    let ring = formatted_ring(slots_per_region);
    let ring_length = 3 * slots_per_region;
    let txg_bound = 2 * ring_length + 3;
    let started = Instant::now();
    let genesis_indexes: BTreeSet<usize> = genesis_root_ring_slots()
        .map(|genesis_slot| {
            ring.ring_slots
                .iter()
                .position(|ring_slot| *ring_slot == genesis_slot)
                .expect("种根的槽在根环里")
        })
        .collect();
    let mut pool = ring.pool.clone();
    let mut slots_admitting_txg: BTreeMap<u64, BTreeSet<usize>> = BTreeMap::new();
    let mut pairs_judged = 0u64;
    let mut instance_dependent_admissions = 0u64;
    for (slot_index, ring_slot) in ring.ring_slots.iter().copied().enumerate() {
        let original_bytes = ring.raw_bytes(&pool, ring_slot);
        for checkpoint_txg in 0..txg_bound {
            let mut admitted_instances = 0u32;
            for instance in 0..ADMISSION_INSTANCES {
                let bytes = ring.root_with(checkpoint_txg, instance).to_slot(ROOT_SLOT_BYTES);
                ring.put_bytes(&mut pool, ring_slot, &bytes);
                pairs_judged += 1;
                if ring.product_reading(&pool, ring_slot).is_some() {
                    admitted_instances += 1;
                    slots_admitting_txg.entry(checkpoint_txg).or_default().insert(slot_index);
                }
            }
            if admitted_instances != 0 && admitted_instances != ADMISSION_INSTANCES {
                instance_dependent_admissions += 1;
            }
        }
        ring.put_bytes(&mut pool, ring_slot, &original_bytes);
    }
    let nonzero_txgs_in_two_or_more_slots = (1..txg_bound)
        .filter(|txg| slots_admitting_txg.get(txg).map_or(0, BTreeSet::len) >= 2)
        .count();
    let nonzero_txgs_not_exactly_at_their_formula_slot = (1..txg_bound)
        .filter(|txg| {
            let formula_slot = target_for_publish(CheckpointTxg(*txg), ring.slots_per_region());
            let formula_index = ring
                .ring_slots
                .iter()
                .position(|ring_slot| *ring_slot == formula_slot)
                .expect("落点槽在根环里");
            slots_admitting_txg.get(txg) != Some(&BTreeSet::from([formula_index]))
        })
        .count();
    let slots_admitting_txg_zero = slots_admitting_txg.get(&0).cloned().unwrap_or_default();
    let txg_zero_exactly_at_the_genesis_slots = slots_admitting_txg_zero == genesis_indexes;
    let holds = nonzero_txgs_in_two_or_more_slots == 0
        && nonzero_txgs_not_exactly_at_their_formula_slot == 0
        && txg_zero_exactly_at_the_genesis_slots
        && instance_dependent_admissions == 0;
    let line = format!(
        "OPUSMODEL name=admission_every_geometry slots_per_region={slots_per_region} txg_range=0..{txg_bound} instances=0..{ADMISSION_INSTANCES} pairs_judged={pairs_judged} nonzero_txgs_admitted_in_two_or_more_slots={nonzero_txgs_in_two_or_more_slots} nonzero_txgs_not_exactly_at_their_formula_slot={nonzero_txgs_not_exactly_at_their_formula_slot} txg_zero_slots_admitted={} txg_zero_exactly_at_the_genesis_slots={txg_zero_exactly_at_the_genesis_slots} instance_dependent_admissions={instance_dependent_admissions} holds={holds} seconds={:.2}",
        slots_admitting_txg_zero.len(),
        started.elapsed().as_secs_f64(),
    );
    (line, holds)
}

#[test]
fn e165_r3_admission_over_every_slots_per_region() {
    let geometries: Vec<u64> = (ROOT_RING_SLOTS_PER_REGION_MINIMUM..=ROOT_RING_SLOTS_PER_REGION_MAXIMUM).collect();
    let threads = default_heavy_threads().get().min(geometries.len()).max(1);
    let chunk = geometries.len().div_ceil(threads);
    let lines: Vec<(String, bool)> = std::thread::scope(|scope| {
        let handles: Vec<_> = geometries
            .chunks(chunk)
            .map(|part| {
                scope.spawn(move || {
                    part.iter()
                        .map(|slots_per_region| admission_line_of_one_geometry(*slots_per_region))
                        .collect::<Vec<(String, bool)>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| handle.join().expect("切片线程没有 panic"))
            .collect()
    });
    for (line, _) in &lines {
        println!("{line}");
    }
    let geometries_that_fail: Vec<u64> = geometries
        .iter()
        .zip(&lines)
        .filter(|(_, (_, holds))| !holds)
        .map(|(slots_per_region, _)| *slots_per_region)
        .collect();
    println!(
        "OPUSMODEL name=admission_every_geometry_summary geometries={} threads={threads} geometries_that_fail={geometries_that_fail:?}",
        geometries.len()
    );
    assert!(
        geometries_that_fail.is_empty(),
        "读者侧落点核：每一档 S 上非 0 txg 恰好只在落点槽、txg 0 恰好只在种根的三槽、读得出与否不随实例代号变；红在哪几档见上一行"
    );
}

/// 一次可写挂载之后的根环：进择根的根（读根环次序）、txg 0 上的实例代号、四条择根序与产品 choose_root 各择哪条、池级 checker 红哪几条。
struct RingReadout {
    line: String,
    txg_zero_instances: BTreeSet<u32>,
    variants_differing: Vec<&'static str>,
    mount_refused: bool,
}

fn ring_readout(devices: &[(DeviceIdentity, SparseBlockDevice)], mounts_done: u32, outcome: &str) -> RingReadout {
    let pool = memory_pool_of(devices);
    let system_configuration = choose_system_configuration(&pool).expect("挂载之后择得出系统配置");
    let immutable = &system_configuration.immutable;
    let ring_slots = every_root_ring_slot(&immutable.sizes);
    let choice_input: Vec<Option<RootRecord>> = ring_slots
        .iter()
        .map(|ring_slot| {
            match read_root_ring_slot(
                &pool,
                &immutable.region_devices,
                &immutable.sizes,
                &immutable.filesystem_identifier,
                *ring_slot,
            ) {
                RootRingSlotReading::SelfVerified(root) => Some(root),
                RootRingSlotReading::Bad(_) => None,
            }
        })
        .collect();
    let admitted: Vec<String> = ring_slots
        .iter()
        .zip(&choice_input)
        .filter_map(|(ring_slot, reading)| {
            reading
                .as_ref()
                .map(|root| format!("r{}s{}:{}", ring_slot.region, ring_slot.slot, key_text(Some(key_of(root)))))
        })
        .collect();
    let txg_zero_instances: BTreeSet<u32> = choice_input
        .iter()
        .flatten()
        .filter(|root| root.checkpoint_txg.0 == 0)
        .map(|root| root.instance.0)
        .collect();
    let today = ChoiceOrder::Today.chosen_root(&choice_input);
    let variants_differing: Vec<&'static str> = ChoiceOrder::COMPARED_WITH_TODAY
        .into_iter()
        .filter(|order| order.chosen_root(&choice_input) != today)
        .map(ChoiceOrder::label)
        .collect();
    let choices: Vec<String> = [ChoiceOrder::Today]
        .into_iter()
        .chain(ChoiceOrder::COMPARED_WITH_TODAY)
        .map(|order| format!("{}={}", order.label(), key_text(order.chosen_root(&choice_input).as_ref().map(key_of))))
        .collect();
    // 池级 checker 之前先打一行：checker 在改坏的树写出的盘上若 panic，根环读数照样留在输出里。
    println!(
        "OPUSMODEL name=writable_mount_ring slots_per_region={MOUNT_SLOTS_PER_REGION} mounts_done={mounts_done} outcome={outcome} admitted=[{}] txg_zero_instances={txg_zero_instances:?} variants_differing={variants_differing:?}",
        admitted.join(" ")
    );
    let violated: Vec<&'static str> = check_pool_image(&pool)
        .into_iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(invariant, _)| invariant)
        .collect();
    let line = format!(
        "OPUSMODEL name=writable_mount slots_per_region={MOUNT_SLOTS_PER_REGION} mounts_done={mounts_done} outcome={outcome} admitted=[{}] txg_zero_instances={txg_zero_instances:?} {} product_choose_root={} variants_differing={variants_differing:?} pool_checker_violated={violated:?}",
        admitted.join(" "),
        choices.join(" "),
        key_text(choose_root(&pool, &system_configuration).as_ref().map(key_of)),
    );
    RingReadout {
        line,
        txg_zero_instances,
        variants_differing,
        mount_refused: outcome != "ok",
    }
}

/// 用户在 mkfs 之后做的只有可写挂载（不写文件、不卸载：现行那一版树表 0 条时卸载一个字节都不写，与不卸载同一张盘）：挂一次、再挂一次，
/// 两次之后各读一遍根环。没改过的树上 txg 0 只有 mkfs 那一种实例代号（0），三条变体与 today 择同一条，两次挂载都做成。
#[test]
fn e165_r3_writable_mounts_on_a_fresh_pool() {
    let parameters = parameters_with(MOUNT_SLOTS_PER_REGION);
    let mut devices = formatted_devices(&parameters);
    let mut readouts: Vec<RingReadout> = Vec::new();
    for mounts_done in 1..=WRITABLE_MOUNTS {
        // 挂载中途 panic 也照样读盘：panic 之前落下的写都在稀疏盘上，读到的就是停在那一刻的盘。
        let mounted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| mount_writable(&parameters, &mut devices)));
        let outcome = match mounted {
            Err(_) => "panicked".to_owned(),
            Ok(Ok(_mounted)) => "ok".to_owned(),
            Ok(Err(error)) => {
                let text: String = format!("{error:?}").chars().filter(|character| !character.is_whitespace()).take(200).collect();
                format!("refused:{text}")
            }
        };
        let readout = ring_readout(&devices, mounts_done, &outcome);
        println!("{}", readout.line);
        readouts.push(readout);
    }
    for readout in &readouts {
        assert!(!readout.mount_refused, "没改过的树上 mkfs 之后的可写挂载要做成：{}", readout.line);
        assert!(
            readout.txg_zero_instances.iter().all(|instance| *instance == 0),
            "写者侧：txg 0 的根只有 mkfs 写、实例代号 0；别的实例代号出现在 txg 0 上说明有写者写了 txg 0：{}",
            readout.line
        );
        assert!(
            readout.variants_differing.is_empty(),
            "合法写者写出的盘上三条变体与 today 择同一条：{}",
            readout.line
        );
    }
}
