//! 七·3 CPU 侧两处快写法（用户 2026-10-26 定「两处都换，并进七·3」；七·2 设计轮第三轮判决
//! `research/prompts/m4-gpugc-design-r3-main-verification.md` 第二、四节；岔路单 `research/prompts/m4-gpugc-design-r1-forks.md`
//! 区域扫描按身份查那一行与「补 2」CRC 换指令那一行）：
//! - 区域扫描两处线性找换成按身份索引（`singlefs_core::region_scan::CentralMappingIdentityIndex`）：随机条目与记录上，
//!   索引查出来的与改前在条目上线性找的逐项相同（改前的写法留在这里当参照）；条目数翻倍时扫描看过的条目数不涨；
//! - CRC-32C 在 x86_64 上查到 SSE4.2 走 `crc32` 指令（`singlefs_core::checksum`）：硬件那条、强制走的软件那条，
//!   各自在随机长度（含 0、1、7、8、9、4095、4096 与三路一块的边界）、页对齐与不对齐、跨页的起点与已发布的校验值上，
//!   与池级 checker 的按位实现逐个相同；走哪条路看得出来，判据就是「查到 SSE4.2」。

use std::collections::BTreeSet;

use singlefs_checker::crc32_castagnoli_bitwise;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber, TreeIdentifier,
};
use singlefs_core::allocator::{AllocationRecord, AllocationRecordQuarantineMark};
use singlefs_core::checksum::{
    castagnoli_route_of_this_processor, crc32_castagnoli, crc32_castagnoli_on_route,
    CastagnoliRoute,
};
use singlefs_core::logged_operations::CompactionRegion;
use singlefs_core::pointer::{LocationEntry, PointerHead};
use singlefs_core::records::mapping_key_for_data;
use singlefs_core::region_scan::{
    scan_region, scan_region_over_the_identity_index, AllocationInRegion, CentralMappingEntry,
    CentralMappingIdentityIndex, MappedUnitClass, MovableUnit,
};
use singlefs_core::unit::{build_data_unit, DataUnitIdentity, WriteOrder};
use singlefs_format::{TREE_IDENTIFIER_EXTENT, UNIT_AREA_START_SLOT};

/// splitmix64：写死种子，每一趟抽到的条目、记录与字节都相同。
struct SplitMixSequence {
    state: u64,
}

impl SplitMixSequence {
    fn seeded(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_value(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut mixed = self.state;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        mixed ^ (mixed >> 31)
    }

    /// `[0, bound)` 里的一个数。
    fn below(&mut self, bound: u64) -> u64 {
        self.next_value() % bound
    }
}

// ───────────────────────────── 区域扫描按身份查 ─────────────────────────────

/// 改前 `classify_allocation` 找指着这一槽的映射条目的写法（`CentralMappingEntry::points_at` 在条目上逐条 filter），原样留作参照。
fn entries_pointing_at_by_the_linear_lookup(
    mapping_entries: &[CentralMappingEntry],
    device: DeviceIdentity,
    slot: SlotNumber,
) -> Vec<&CentralMappingEntry> {
    mapping_entries
        .iter()
        .filter(|entry| {
            entry
                .locations
                .iter()
                .any(|location| location.device == device && location.slot == slot)
        })
        .collect()
}

/// 改前 `classify_allocation` 判「头里的 key 在不在映射里」的写法（在条目上逐条 `any`），原样留作参照。
fn mapping_key_is_mapped_by_the_linear_lookup(
    mapping_entries: &[CentralMappingEntry],
    key: &[u8],
) -> bool {
    mapping_entries.iter().any(|entry| entry.key == key)
}

/// 随机映射条目：key 从一小把里抽（同一把 key 出现几次）、两条位置项的盘从四块里抽、槽从八个里抽，
/// 有的条目两条位置项指同一落点。
const RANDOM_MAPPING_ROUNDS: usize = 300;
const RANDOM_MAPPING_ENTRIES_AT_MOST: u64 = 40;
const RANDOM_RECORDS_PER_ROUND: usize = 24;
const RANDOM_KEY_ALPHABET: u64 = 12;
const RANDOM_DEVICES: u64 = 4;
const RANDOM_SLOTS: u64 = 8;
const RANDOM_FIRST_SLOT: u64 = UNIT_AREA_START_SLOT;
/// 映射 key 的字节数（D19（块指针的结构与宽度预算） 已定项 10：27 字节）。
const MAPPING_KEY_BYTES: usize = 27;

/// 那一小把 key 里的第 `key_number` 把：打头的类标签取 1 到 3，其余字节都是 `key_number`。
fn mapping_key_numbered(key_number: u8) -> Vec<u8> {
    let mut key = vec![key_number; MAPPING_KEY_BYTES];
    key[0] = 1 + key_number % 3;
    key
}

fn random_mapping_key(sequence: &mut SplitMixSequence) -> Vec<u8> {
    mapping_key_numbered(u8::try_from(sequence.below(RANDOM_KEY_ALPHABET)).expect("小于 12"))
}

fn random_location(sequence: &mut SplitMixSequence) -> LocationEntry {
    LocationEntry {
        device: DeviceIdentity(u32::try_from(sequence.below(RANDOM_DEVICES)).expect("小于 4")),
        slot: SlotNumber(RANDOM_FIRST_SLOT + sequence.below(RANDOM_SLOTS)),
        unit_checksum: u32::try_from(sequence.below(1 << 16)).expect("小于 2^16"),
    }
}

fn random_mapping_entries(sequence: &mut SplitMixSequence) -> Vec<CentralMappingEntry> {
    let entries = sequence.below(RANDOM_MAPPING_ENTRIES_AT_MOST + 1);
    (0..entries)
        .map(|_entry| {
            let key = random_mapping_key(sequence);
            let first_location = random_location(sequence);
            let second_location = if sequence.below(4) == 0 {
                first_location
            } else {
                random_location(sequence)
            };
            CentralMappingEntry {
                key,
                locations: [first_location, second_location],
            }
        })
        .collect()
}

fn random_allocation_records(sequence: &mut SplitMixSequence) -> Vec<AllocationRecord> {
    (0..RANDOM_RECORDS_PER_ROUND)
        .map(|_record| AllocationRecord {
            device: DeviceIdentity(u32::try_from(sequence.below(RANDOM_DEVICES)).expect("小于 4")),
            slot: SlotNumber(RANDOM_FIRST_SLOT + sequence.below(RANDOM_SLOTS)),
            span_slots: 2,
            generation: CheckpointTxg(7),
            is_released: false,
            quarantine_mark: AllocationRecordQuarantineMark::NotQuarantined,
        })
        .collect()
}

/// 两份条目清单逐项是同一条（同一个地址）：次序与条数都要相同，值相同而不是同一条的也不算。
fn same_entries_in_the_same_order(
    by_the_index: &[&CentralMappingEntry],
    by_the_linear_lookup: &[&CentralMappingEntry],
) -> bool {
    by_the_index.len() == by_the_linear_lookup.len()
        && by_the_index
            .iter()
            .zip(by_the_linear_lookup)
            .all(|(indexed, linear)| std::ptr::eq(*indexed, *linear))
}

/// 随机映射条目与随机记录上，按落点查（每条记录的盘与槽，连同八个槽与四块盘的每一种）与按 key 查（一小把 key 的每一把，
/// 连同一把映射里没有的），索引交回的与改前在条目上线性找的逐项相同：同一批条目、同一个次序。
#[test]
fn identity_index_lookups_match_the_linear_lookups_on_random_mapping_entries_and_records() {
    let mut sequence = SplitMixSequence::seeded(0x0007_0003_0000_0001);
    let mut location_lookups_with_an_entry = 0_usize;
    let mut location_lookups_with_two_or_more_entries = 0_usize;
    let mut entries_whose_only_matching_location_is_the_second = 0_usize;
    // 迭代上界是轮数；每轮的条目与记录只在这一轮里用。
    for round in 0..RANDOM_MAPPING_ROUNDS {
        let mapping_entries = random_mapping_entries(&mut sequence);
        let records = random_allocation_records(&mut sequence);
        let index = CentralMappingIdentityIndex::of_the_mapping_entries(&mapping_entries);
        let every_location = (0..RANDOM_DEVICES).flat_map(|device| {
            (0..RANDOM_SLOTS).map(move |slot_offset| {
                (
                    DeviceIdentity(u32::try_from(device).expect("小于 4")),
                    SlotNumber(RANDOM_FIRST_SLOT + slot_offset),
                )
            })
        });
        for (device, slot) in records
            .iter()
            .map(|record| (record.device, record.slot))
            .chain(every_location)
        {
            let by_the_index = index.entries_pointing_at(device, slot);
            let by_the_linear_lookup =
                entries_pointing_at_by_the_linear_lookup(&mapping_entries, device, slot);
            assert!(
                same_entries_in_the_same_order(&by_the_index, &by_the_linear_lookup),
                "第 {round} 轮，盘 {device:?} 槽 {slot:?}：索引交回 {by_the_index:?}，线性找交回 {by_the_linear_lookup:?}"
            );
            location_lookups_with_an_entry += usize::from(!by_the_linear_lookup.is_empty());
            location_lookups_with_two_or_more_entries +=
                usize::from(by_the_linear_lookup.len() >= 2);
            entries_whose_only_matching_location_is_the_second += by_the_linear_lookup
                .iter()
                .filter(|entry| {
                    let [first_location, second_location] = entry.locations;
                    !(first_location.device == device && first_location.slot == slot)
                        && second_location.device == device
                        && second_location.slot == slot
                })
                .count();
        }
        let mut absent_key = vec![0xEE_u8; MAPPING_KEY_BYTES];
        absent_key[0] = 1;
        for key in (0..RANDOM_KEY_ALPHABET)
            .map(|key_number| mapping_key_numbered(u8::try_from(key_number).expect("小于 12")))
            .chain([absent_key])
        {
            assert_eq!(
                index.holds_mapping_key(&key),
                mapping_key_is_mapped_by_the_linear_lookup(&mapping_entries, &key),
                "第 {round} 轮，key {key:?}：按 key 查与线性找不同"
            );
        }
    }
    assert!(
        location_lookups_with_an_entry > 0
            && location_lookups_with_two_or_more_entries > 0
            && entries_whose_only_matching_location_is_the_second > 0,
        "随机条目要撞到有条目、有两条以上条目、只有第二条位置项指着的落点：{location_lookups_with_an_entry} / \
         {location_lookups_with_two_or_more_entries} / {entries_whose_only_matching_location_is_the_second}"
    );
}

const FILESYSTEM_IDENTIFIER: [u8; 16] = [7u8; 16];
const BIRTH_TXG: CheckpointTxg = CheckpointTxg(7);
const WRITING_INSTANCE: InstanceGeneration = InstanceGeneration(2);
const REGION_DEVICE: DeviceIdentity = DeviceIdentity(0);
const MIRROR_DEVICE: DeviceIdentity = DeviceIdentity(1);
/// R 里放几个可搬的码 1 数据单元（每个跨两槽）。
const MOVABLE_UNITS_IN_THE_REGION: u64 = 8;
/// 映射条目的两档：R 里那几条之外，另放这么多条指着 R 外的条目；第二档是第一档的两倍。
const MAPPING_ENTRIES_OF_THE_SMALLER_MAPPING: u64 = 1024;

/// 单元区里起点 50240 的那个聚簇段 [50240, 50304)。
fn region_under_test() -> CompactionRegion {
    CompactionRegion::of_one_cluster_segment(REGION_DEVICE, SlotNumber(UNIT_AREA_START_SLOT + 64))
        .expect("50240 = 785 × 64")
}

fn write_order_of(transaction: u64) -> WriteOrder {
    WriteOrder {
        instance: WRITING_INSTANCE,
        transaction,
    }
}

/// 写者给事务号 `transaction` 的那个码 1 数据单元的映射 key（出生树 = extent 树、出生 txg、写序）。
fn data_unit_mapping_key(transaction: u64) -> Vec<u8> {
    mapping_key_for_data(
        PointerHead {
            birth_tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
            birth_txg: BIRTH_TXG,
        },
        write_order_of(transaction),
    )
}

fn data_unit_bytes(transaction: u64) -> Vec<u8> {
    build_data_unit(
        DataUnitIdentity {
            tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
            object: 1,
            object_birth: CheckpointTxg(3),
            anchor_offset: 0,
        },
        BIRTH_TXG,
        &FILESYSTEM_IDENTIFIER,
        write_order_of(transaction),
        &[0x5A; 100],
    )
}

fn both_devices_at(slot: SlotNumber) -> [LocationEntry; 2] {
    [REGION_DEVICE, MIRROR_DEVICE].map(|device| LocationEntry {
        device,
        slot,
        unit_checksum: 0,
    })
}

/// 一个池：R 里 [`MOVABLE_UNITS_IN_THE_REGION`] 个可搬的数据单元（两块盘同槽各一条记录、映射指着它），
/// 加 `entries_outside_the_region` 条指着 R 外的映射条目（没有记录、不进扫描），映射条目表里 R 外的在前。
struct PoolWithAGrowingMapping {
    records: Vec<AllocationRecord>,
    mapping_entries: Vec<CentralMappingEntry>,
    unit_bytes_by_slot: Vec<(SlotNumber, Vec<u8>)>,
}

impl PoolWithAGrowingMapping {
    fn with_entries_outside_the_region(entries_outside_the_region: u64) -> Self {
        let mut records = Vec::new();
        let mut mapping_entries = Vec::new();
        let mut unit_bytes_by_slot = Vec::new();
        // R 外的条目排在前面、R 里的排在后面：在条目上线性找要先走过全部 R 外的条目才找得到 R 里的，看过的条目数随它们涨。
        let first_slot_outside_the_region = region_under_test().end_slot().0;
        for outside_number in 0..entries_outside_the_region {
            mapping_entries.push(CentralMappingEntry {
                key: data_unit_mapping_key(MOVABLE_UNITS_IN_THE_REGION + outside_number),
                locations: both_devices_at(SlotNumber(
                    first_slot_outside_the_region + 2 * outside_number,
                )),
            });
        }
        for unit_number in 0..MOVABLE_UNITS_IN_THE_REGION {
            let slot = SlotNumber(region_under_test().start_slot().0 + 2 * unit_number);
            for device in [REGION_DEVICE, MIRROR_DEVICE] {
                records.push(AllocationRecord {
                    device,
                    slot,
                    span_slots: 2,
                    generation: BIRTH_TXG,
                    is_released: false,
                    quarantine_mark: AllocationRecordQuarantineMark::NotQuarantined,
                });
            }
            mapping_entries.push(CentralMappingEntry {
                key: data_unit_mapping_key(unit_number),
                locations: both_devices_at(slot),
            });
            unit_bytes_by_slot.push((slot, data_unit_bytes(unit_number)));
        }
        Self {
            records,
            mapping_entries,
            unit_bytes_by_slot,
        }
    }

    fn read_unit_bytes(
        &self,
        device: DeviceIdentity,
        slot: SlotNumber,
        unit_bytes: usize,
    ) -> Option<Vec<u8>> {
        (device == REGION_DEVICE || device == MIRROR_DEVICE)
            .then(|| {
                self.unit_bytes_by_slot
                    .iter()
                    .find(|(unit_slot, bytes)| *unit_slot == slot && bytes.len() == unit_bytes)
                    .map(|(_slot, bytes)| bytes.clone())
            })
            .flatten()
    }
}

/// 映射条目 1032 条与 2056 条两档（R 里 8 条、R 外 1024 条与 2048 条），同一个 R：扫描结果相同、全部可搬；
/// 扫描在索引上查的次数与看过的条目数两档逐个相同（每条记录按落点查一次、看指着它的那一条，按 key 查一次、看带这把 key 的那一条），
/// 不随条目数涨；改前在条目上线性找，每条记录要看全部条目两遍，第二档是第一档的两倍。
#[test]
fn region_scan_examines_no_more_mapping_entries_when_the_mapping_doubles() {
    let pinned_slots = BTreeSet::new();
    let quarantined_slots = BTreeSet::new();
    let scan_and_its_lookup_work = |entries_outside_the_region: u64| {
        let pool =
            PoolWithAGrowingMapping::with_entries_outside_the_region(entries_outside_the_region);
        let index = CentralMappingIdentityIndex::of_the_mapping_entries(&pool.mapping_entries);
        let scan = scan_region_over_the_identity_index(
            region_under_test(),
            &pool.records,
            &index,
            &pinned_slots,
            &quarantined_slots,
            |device, slot, unit_bytes| pool.read_unit_bytes(device, slot, unit_bytes),
        )
        .expect("R 里没有跨边界的记录、key 两两不同");
        let scan_building_its_own_index = scan_region(
            region_under_test(),
            &pool.records,
            &pool.mapping_entries,
            &pinned_slots,
            &quarantined_slots,
            |device, slot, unit_bytes| pool.read_unit_bytes(device, slot, unit_bytes),
        )
        .expect("同一个池");
        assert_eq!(
            scan, scan_building_its_own_index,
            "调用方交进来的索引与扫描自己建的，扫描结果相同"
        );
        (scan, index.lookup_work(), pool.mapping_entries.len())
    };

    let (smaller_scan, smaller_work, smaller_entries) =
        scan_and_its_lookup_work(MAPPING_ENTRIES_OF_THE_SMALLER_MAPPING);
    let (larger_scan, larger_work, larger_entries) =
        scan_and_its_lookup_work(2 * MAPPING_ENTRIES_OF_THE_SMALLER_MAPPING);

    let expected_movable: Vec<AllocationInRegion> = (0..MOVABLE_UNITS_IN_THE_REGION)
        .map(|unit_number| {
            AllocationInRegion::Movable(MovableUnit {
                mapping_key: data_unit_mapping_key(unit_number),
                unit_class: MappedUnitClass::Data,
                birth_txg: BIRTH_TXG,
            })
        })
        .collect();
    for scan in [&smaller_scan, &larger_scan] {
        assert_eq!(
            scan.allocations
                .iter()
                .map(|allocation| allocation.class.clone())
                .collect::<Vec<_>>(),
            expected_movable,
            "R 里 8 个单元都可搬"
        );
    }
    assert_eq!(larger_entries, 2 * smaller_entries - 8, "R 外的条目翻倍");
    assert_eq!(
        smaller_work.index_lookups,
        2 * MOVABLE_UNITS_IN_THE_REGION,
        "每条可搬的记录按落点、按 key 各查一次：{smaller_work:?}"
    );
    assert_eq!(
        smaller_work.mapping_entries_examined,
        2 * MOVABLE_UNITS_IN_THE_REGION,
        "每查一次只看一条条目：{smaller_work:?}"
    );
    assert_eq!(
        larger_work, smaller_work,
        "映射条目由 {smaller_entries} 条到 {larger_entries} 条，扫描查的次数与看过的条目数不变"
    );
}

// ───────────────────────────── CRC-32C 换硬件指令 ─────────────────────────────

/// 已发布的校验值：校验串 "123456789"（CRC 目录的 check 值）；RFC 3720 附录 B.4 的四个 32 字节例子。
fn published_check_values() -> Vec<(&'static str, Vec<u8>, u32)> {
    vec![
        ("check string 123456789", b"123456789".to_vec(), 0xE306_9283),
        (
            "RFC 3720 B.4 32 bytes of zeroes",
            vec![0u8; 32],
            0x8A91_36AA,
        ),
        (
            "RFC 3720 B.4 32 bytes of ones",
            vec![0xFFu8; 32],
            0x62A8_AB43,
        ),
        (
            "RFC 3720 B.4 32 incrementing bytes",
            (0u8..32).collect(),
            0x46DD_794E,
        ),
        (
            "RFC 3720 B.4 32 decrementing bytes",
            (0u8..32).rev().collect(),
            0x113F_DB5C,
        ),
    ]
}

/// 写死的长度：0、1、7、8、9（一条 `crc32q` 前后）、4095、4096、4097（一页前后）、三路一块与两块的边界、节点与数据单元。
const FIXED_LENGTHS: [usize; 27] = [
    0, 1, 2, 7, 8, 9, 15, 16, 17, 511, 512, 513, 1535, 1536, 1537, 3071, 3072, 3073, 4095, 4096,
    4097, 8191, 8192, 12288, 16384, 32767, 32768,
];
/// 另抽这么多个随机长度，至多 [`RANDOM_LENGTH_AT_MOST`] 字节。
const RANDOM_LENGTHS: usize = 48;
const RANDOM_LENGTH_AT_MOST: u64 = 70_000;
const PAGE_BYTES: usize = 4096;

/// 一段随机字节里一个页对齐的位置之后放得下最长的长度再多两页，起点有四种：正好在页边界上（页对齐）、页边界后 1 字节、
/// 页边界后 8 字节（8 字节对齐而不是页对齐）、页边界前 3 字节（不对齐、跨页）。交回（长度，起点名，那一段字节）。
fn lengths_and_starts_over_random_bytes() -> (Vec<u8>, Vec<(usize, &'static str, usize)>) {
    let mut sequence = SplitMixSequence::seeded(0x0007_0003_0000_0002);
    let mut lengths: Vec<usize> = FIXED_LENGTHS.to_vec();
    lengths.extend((0..RANDOM_LENGTHS).map(|_length| {
        usize::try_from(sequence.below(RANDOM_LENGTH_AT_MOST + 1)).expect("70000 装得进 usize")
    }));
    let longest = lengths.iter().copied().max().expect("长度表不空");
    let buffer: Vec<u8> = (0..longest + 4 * PAGE_BYTES)
        .map(|_byte| sequence.next_value().to_le_bytes()[0])
        .collect();
    let page_boundary = 2 * PAGE_BYTES - buffer.as_ptr().addr() % PAGE_BYTES;
    let starts = [
        ("page aligned", page_boundary),
        ("one byte past a page boundary", page_boundary + 1),
        ("eight bytes past a page boundary", page_boundary + 8),
        ("three bytes before a page boundary", page_boundary - 3),
    ];
    let cases = lengths
        .iter()
        .flat_map(|length| {
            starts
                .iter()
                .map(move |(start_name, start)| (*length, *start_name, *start))
        })
        .collect();
    (buffer, cases)
}

/// `crc_on_the_route` 在每个长度与起点、每个已发布的校验值上与池级 checker 的按位实现相同；交回比过几段。
fn assert_the_route_matches_the_checker_bitwise_crc(
    route_name: &str,
    crc_on_the_route: impl Fn(&[u8]) -> u32,
) -> usize {
    let mut compared = 0_usize;
    for (name, bytes, published) in published_check_values() {
        assert_eq!(
            crc32_castagnoli_bitwise(&bytes),
            published,
            "{name}：池级 checker 的按位实现给的不是已发布的值"
        );
        assert_eq!(
            crc_on_the_route(&bytes),
            published,
            "{route_name}，{name}：与已发布的值不同"
        );
        compared += 1;
    }
    let (buffer, cases) = lengths_and_starts_over_random_bytes();
    for (length, start_name, start) in cases {
        let bytes = &buffer[start..start + length];
        assert_eq!(
            crc_on_the_route(bytes),
            crc32_castagnoli_bitwise(bytes),
            "{route_name}，长度 {length}、起点 {start_name}：与池级 checker 的按位实现不同"
        );
        compared += 1;
    }
    compared
}

/// 这台处理器走的路由判据「查到 SSE4.2」定：x86_64 上查到就是硬件指令那条，查不到是 slicing-by-8；别的架构只有 slicing-by-8。
#[test]
fn the_route_of_this_processor_is_the_hardware_instruction_exactly_when_the_crc_instruction_is_detected(
) {
    let route = castagnoli_route_of_this_processor();
    #[cfg(target_arch = "x86_64")]
    {
        let is_the_crc_instruction_detected = std::arch::is_x86_feature_detected!("sse4.2");
        assert_eq!(
            matches!(route, CastagnoliRoute::HardwareCrcInstructionThreeWay(_)),
            is_the_crc_instruction_detected,
            "查到 SSE4.2 = {is_the_crc_instruction_detected}，走的路却是 {route:?}"
        );
    }
    #[cfg(not(target_arch = "x86_64"))]
    assert_eq!(route, CastagnoliRoute::SoftwareSlicingByEight);
}

/// 硬件指令那条路（这台处理器查到 SSE4.2 才走得到）与不点名走的那条（`crc32_castagnoli`），在每个长度、起点与已发布的值上
/// 与池级 checker 的按位实现相同。
#[test]
fn the_hardware_crc_route_matches_the_checker_bitwise_crc_on_every_length_alignment_and_published_value(
) {
    let compared_by_default =
        assert_the_route_matches_the_checker_bitwise_crc("不点名走的那条", crc32_castagnoli);
    assert!(compared_by_default > FIXED_LENGTHS.len());
    #[cfg(target_arch = "x86_64")]
    {
        assert!(
            FIXED_LENGTHS.contains(&(3 * singlefs_core::checksum::HARDWARE_CRC_LANE_BYTES))
                && FIXED_LENGTHS
                    .contains(&(3 * singlefs_core::checksum::HARDWARE_CRC_LANE_BYTES + 1)),
            "长度表要罩住三路一块的边界"
        );
        let Some(detected) = singlefs_core::checksum::CrcInstructionDetected::on_this_processor()
        else {
            println!(
                "本次未跑：这台处理器没有 SSE4.2，硬件指令那条路走不到（不点名走的那条已比过）"
            );
            return;
        };
        let hardware_route = CastagnoliRoute::HardwareCrcInstructionThreeWay(detected);
        assert_eq!(castagnoli_route_of_this_processor(), hardware_route);
        assert_the_route_matches_the_checker_bitwise_crc("硬件指令那条", |bytes| {
            crc32_castagnoli_on_route(hardware_route, bytes)
        });
    }
}

/// 只供测试的开关强制走 slicing-by-8（这台处理器有 SSE4.2 时 `crc32_castagnoli` 不走它）：每个长度、起点与已发布的值上
/// 与池级 checker 的按位实现相同。
#[test]
fn forcing_the_software_crc_route_matches_the_checker_bitwise_crc_on_every_length_alignment_and_published_value(
) {
    let compared =
        assert_the_route_matches_the_checker_bitwise_crc("强制走的 slicing-by-8", |bytes| {
            crc32_castagnoli_on_route(CastagnoliRoute::SoftwareSlicingByEight, bytes)
        });
    assert!(compared > FIXED_LENGTHS.len());
}
