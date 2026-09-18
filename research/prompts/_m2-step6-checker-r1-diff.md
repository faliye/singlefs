# 附录二：`m2-step6-checker-r1` 代码 diff（工作区当前内容，整段抄，非 `git diff`）

每段行号用 `awk 'NR==A,NR==B'` 现取（见各段标题），现查时点：2026-09-17。

## `crates/singlefs-checker/src/walk.rs`:47-65　`Walk` 结构体定义

```rust
struct Walk<'reader> {
    reader: &'reader dyn ImageReader,
    judgements: Judgements,
    filesystem_identifier_low: u64,
    /// (设备, 起点槽, 跨度槽) → 第一次引用它的是谁。
    references: BTreeMap<(u32, u64, u64), String>,
    visited_units: BTreeSet<(u32, u64)>,
    walk_failures: Vec<String>,
    /// 走过的全部树表条目里的树 ID（I-7.8 的一半）。
    tree_identifiers_in_tables: BTreeSet<u64>,
    /// inode 号 → 对象出生代；数据单元 → (对象, 对象出生代)。I-9.10 在最后对。
    inode_object_birth: BTreeMap<u64, u64>,
    data_unit_objects: Vec<(u64, u64, String)>,
    /// 最新根下面记账树里的行：(统计量, 设备) → 值。
    accounting: BTreeMap<(u16, u32), u64>,
    accounting_seen: bool,
    /// 最新根指着的实例表里的 (实例代号, T)：别的根按它判有效（回退之后被抛弃时间线的根不进 I-3.1 的并集）。
    instance_table_rows: Vec<(u32, u64)>,
}
```

## `crates/singlefs-checker/src/walk.rs`:196-269　`walk_root` 整个函数

```rust
    fn walk_root(&mut self, record: &[u8], is_newest: bool) {
        // 实例表单元：根记录直接持有（自证单元，I-1.3 豁免）。
        let instance_pointer = parse_node_pointer(&record[170..256]);
        judge_location_order(&mut self.judgements, &instance_pointer, "实例表指针");
        for location in &instance_pointer.locations {
            self.note_reference(location.device, location.slot, 2, "实例表单元");
        }
        if let Some(unit) = read_referenced_unit(
            self.reader,
            &mut self.judgements,
            &instance_pointer.locations,
            data_unit_bytes(),
            "实例表单元",
        ) {
            if self.visited_units.insert((
                instance_pointer.locations[0].device,
                instance_pointer.locations[0].slot,
            )) {
                let records =
                    self.judge_packed_container(&unit, PACKED_TYPE_INSTANCE_TABLE, "实例表单元");
                if is_newest {
                    if let Some(records) = records {
                        let mount_root_instance =
                            u32::from_le_bytes(record[24..28].try_into().expect("4 字节"));
                        self.judge_instance_table_rows(&records, mount_root_instance);
                    }
                }
            }
        } else {
            self.walk_failures
                .push("实例表单元两份都读不到对得上的".to_string());
        }
        // 树表单元：不属于任何树（树 ID 0）。
        let Some(tree_table) =
            self.read_index_node(&record[36..122], 0, KEY_SCHEMA_TREE_TABLE, "树表单元")
        else {
            return;
        };
        for entry in &tree_table.entries {
            self.walk_tree_table_entry(entry);
        }
        // 中央映射树的根住根记录（D19（块指针的结构与宽度预算） 已定项 11）。
        if let Some(mapping) = self.read_index_node(
            &record[256..342],
            TREE_IDENTIFIER_CENTRAL_MAPPING,
            KEY_SCHEMA_MAPPING,
            "中央映射树的根",
        ) {
            for entry in &mapping.entries {
                let locations_pointer: Vec<u8> =
                    [vec![0u8; 50], entry[27..55].to_vec(), vec![0u8; 8]].concat();
                let view = parse_node_pointer(&locations_pointer);
                judge_location_order(&mut self.judgements, &view, "映射条目");
                let length = if entry[0] == 2 {
                    node_bytes()
                } else {
                    data_unit_bytes()
                };
                if read_referenced_unit(
                    self.reader,
                    &mut self.judgements,
                    &view.locations,
                    length,
                    "映射条目指的单元",
                )
                .is_none()
                {
                    self.walk_failures
                        .push("映射条目指的单元两份都读不到对得上的".to_string());
                }
            }
        }
        let _ = is_newest;
    }
```

## `crates/singlefs-checker/src/walk.rs`:688-904　`check_pool_image` 整个函数（文件末尾，含入口文档注释）

```rust
/// 池级 checker 的入口：每条第一版不变量都报，没评估到的报「不适用」并带理由。
#[must_use]
pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, InvariantVerdict)> {
    let superblocks = chosen_superblocks(reader);
    let mut root_ring_judgements = Judgements::default();
    let chosen: Vec<_> = superblocks
        .iter()
        .filter_map(|(device, chosen)| {
            chosen
                .clone()
                .map(|(view, geometry)| (*device, view, geometry))
        })
        .collect();
    if chosen.is_empty() || chosen.len() != superblocks.len() {
        for invariant in crate::image::IMPLEMENTED_INVARIANTS {
            root_ring_judgements.not_applicable(
                invariant,
                "有一块盘两个超级块槽都无效：这不是一个挂得上的镜像",
            );
        }
        return root_ring_judgements.into_report();
    }
    let geometry = chosen[0].2;
    // 各盘择到的超级块要属于同一个池：fsid 逐盘相同（一块别的池的旧盘插进来，实例代号可以恰好也是 1，I-7.7 看不出来）。
    for (device, view, _) in &chosen {
        root_ring_judgements.judge(
            "I-1.4",
            view.filesystem_identifier == geometry.filesystem_identifier,
            || {
                format!(
                    "盘 {device} 择到的超级块 fsid 与盘 {} 的不同：这块盘不属于这个池",
                    chosen[0].0
                )
            },
        );
    }
    let devices = reader.devices();
    let regions = usize::try_from(geometry.regions).expect("R");
    let region_devices = &geometry.region_devices[..regions.min(3)];
    let distinct: BTreeSet<u32> = region_devices.iter().copied().collect();
    let heaviest = devices
        .iter()
        .map(|device| {
            region_devices
                .iter()
                .filter(|candidate| *candidate == device)
                .count()
        })
        .max()
        .unwrap_or(0);
    let pigeonhole = regions.div_ceil(devices.len().max(1));
    root_ring_judgements.judge(
        "I-7.6",
        distinct.len() == regions.min(devices.len()) && heaviest <= pigeonhole && region_devices.iter().all(|device| devices.contains(device)),
        || format!("根环区域的设备 {region_devices:?}：不同值 {}、最重的盘背 {heaviest} 个（上界 {pigeonhole}）", distinct.len()),
    );
    let roots = valid_roots(reader, &geometry);
    judge_instance_carriers(reader, &geometry, &roots, &mut root_ring_judgements);
    root_ring_judgements.judge("I-7.1", !roots.is_empty(), || {
        "根环里一条自证过的根都没有".to_string()
    });
    if roots.is_empty() {
        return root_ring_judgements.into_report();
    }
    let newest_index = roots
        .iter()
        .enumerate()
        .max_by_key(|(_, (_, _, root))| (root.checkpoint_txg, root.instance))
        .map(|(index, _)| index)
        .expect("非空");
    let mut walk = Walk {
        reader,
        judgements: root_ring_judgements,
        filesystem_identifier_low: u64::from_le_bytes(
            geometry.filesystem_identifier[..8]
                .try_into()
                .expect("8 字节"),
        ),
        references: BTreeMap::new(),
        visited_units: BTreeSet::new(),
        walk_failures: Vec::new(),
        tree_identifiers_in_tables: BTreeSet::new(),
        inode_object_birth: BTreeMap::new(),
        data_unit_objects: Vec::new(),
        accounting: BTreeMap::new(),
        accounting_seen: false,
        instance_table_rows: Vec::new(),
    };
    // 先走最新的根：它的走读断没断就是 I-7.2；记账行只取最新根下面的。
    walk.walk_root(&roots[newest_index].2.record_bytes, true);
    let newest_failures = walk.walk_failures.clone();
    // I-4.8（近 K 代根校验和自洽）：从候选集里任一根出发遍历，所有块的校验和都与父指针一致、走读不断——最新根那一次也算一格。
    let newest_txg = roots[newest_index].2.checkpoint_txg;
    walk.judgements.judge(
        "I-4.8",
        newest_failures.is_empty() && walk.judgements.violation_count("I-2.1") == 0,
        || format!("最新根（txg {newest_txg}）出发的遍历有单元对不上或读不出"),
    );
    let accounting_seen = walk.accounting_seen;
    let accounting = walk.accounting.clone();
    // 别的根只取按最新根指着的实例表仍然有效的：(i, T) 有效 ⟺ 无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti
    // （D23（journal 的角色与格式） 已定项 14 回退段的候选集规则）；被抛弃时间线的根引用的单元由影子账隔离、不在当前账里。
    // 再加一条：txg ≥ 最新根带的回退下界 F（D16（发布语义） 已定项 1 的回退候选集）；F 之下的根引用的单元可以已被回收复用，
    // 它们不在当前账里、也不再是「近 K 代」——I-2.1 只在候选集里的根上判。
    let instance_table_rows = walk.instance_table_rows.clone();
    let newest_rollback_floor = u64::from_le_bytes(
        roots[newest_index].2.record_bytes[130..138]
            .try_into()
            .expect("8 字节"),
    );
    let mut older_candidates = 0u64;
    for (index, (_, _, root)) in roots.iter().enumerate() {
        let abandoned = instance_table_rows.iter().any(|(row_instance, row_txg)| {
            *row_instance == root.instance && root.checkpoint_txg > *row_txg
        });
        let below_floor = root.checkpoint_txg < newest_rollback_floor;
        if index != newest_index && !abandoned && !below_floor {
            older_candidates += 1;
            let mismatches_before = walk.judgements.violation_count("I-2.1");
            let failures_before = walk.walk_failures.len();
            walk.walk_root(&root.record_bytes, false);
            // 这条候选根引用的单元有一个校验和对不上或头用不了，就是它指着的块被复用或抹头了（I-7.4（近 K 代块未被复用）），
            // 从它出发的遍历也就不自洽（I-4.8（近 K 代根校验和自洽））；两条按每条候选根各判一格。
            let walked_into_reused_or_erased_unit = walk.judgements.violation_count("I-2.1")
                > mismatches_before
                || walk.walk_failures.len() > failures_before;
            let root_txg = root.checkpoint_txg;
            walk.judgements
                .judge("I-7.4", !walked_into_reused_or_erased_unit, || {
                    format!(
                        "候选根 txg {root_txg} 引用的单元已被复用或抹头（校验和对不上或头用不了）"
                    )
                });
            walk.judgements
                .judge("I-4.8", !walked_into_reused_or_erased_unit, || {
                    format!("候选根 txg {root_txg} 出发的遍历有单元对不上或读不出")
                });
        }
    }
    if older_candidates == 0 {
        walk.judgements
            .not_applicable("I-7.4", "回退候选集里只有最新根，没有更早的根可判");
    }
    let mut judgements = walk.judgements;
    judgements.judge("I-7.2", newest_failures.is_empty(), || {
        format!("最新的根走不完：{}", newest_failures.join("；"))
    });
    for (inode, unit_object_birth, what) in &walk.data_unit_objects {
        if let Some(record_birth) = walk.inode_object_birth.get(inode) {
            judgements.judge("I-9.10", record_birth == unit_object_birth, || {
                format!(
                    "{what} 的对象出生代 {unit_object_birth} 与 inode 记录的 {record_birth} 不符"
                )
            });
        }
    }
    // I-5.1：同一块盘上，不同的引用不许占重叠的槽（两条位置条目落在不同盘上是显式的多副本）。
    let mut per_device: BTreeMap<u32, Vec<(u64, u64, &String)>> = BTreeMap::new();
    for ((device, slot, span), what) in &walk.references {
        per_device
            .entry(*device)
            .or_default()
            .push((*slot, *span, what));
    }
    for (device, mut ranges) in per_device.clone() {
        ranges.sort_unstable_by_key(|(slot, span, _)| (*slot, *span));
        for pair in ranges.windows(2) {
            let (first_slot, first_span, first_what) = pair[0];
            let (second_slot, _, second_what) = pair[1];
            judgements.judge("I-5.1", first_slot + first_span <= second_slot, || {
                format!("盘 {device}：{first_what}（槽 {first_slot} 跨 {first_span}）与 {second_what}（槽 {second_slot}）重叠")
            });
        }
    }
    // I-3.1 / I-5.2：最新根下面记账树的「已分配」「空闲」逐盘对遍历得到的和与容量。
    if accounting_seen {
        for device in &devices {
            let walked: u64 = per_device.get(device).map_or(0, |ranges| {
                ranges.iter().map(|(_, span, _)| span * SLOT_BYTES).sum()
            });
            let allocated = accounting
                .get(&(STATISTIC_ALLOCATED_BYTES, *device))
                .copied();
            judgements.judge("I-3.1", allocated == Some(walked), || {
                format!("盘 {device}：记账的已分配 {allocated:?}，遍历全部有效根得到 {walked}")
            });
            let capacity = reader
                .device_bytes(*device)
                .map(|bytes| (bytes / SLOT_BYTES - geometry.unit_area_start_slot) * SLOT_BYTES);
            let free = accounting.get(&(STATISTIC_FREE_BYTES, *device)).copied();
            judgements.judge("I-5.2", matches!((free, allocated, capacity), (Some(free), Some(allocated), Some(capacity)) if free + allocated == capacity), || {
                format!("盘 {device}：空闲 {free:?} + 已分配 {allocated:?} ≠ 单元区 {capacity:?}")
            });
        }
    } else {
        judgements.not_applicable("I-3.1", "最新的根下面还没有记账树（第 0 代树表）");
        judgements.not_applicable("I-5.2", "最新的根下面还没有记账树（第 0 代树表）");
    }
    // I-7.8：根环全部有效根的水位取 max，要大于盘上出现过的最大树 ID（全部码 2 单元头 ∪ 走过的树表条目）。
    let watermark = roots
        .iter()
        .map(|(_, _, root)| root.tree_identifier_watermark)
        .max()
        .unwrap_or(0);
    let newest_published_txg = roots
        .iter()
        .map(|(_, _, root)| root.checkpoint_txg)
        .max()
        .unwrap_or(0);
    let mut seen = scanned_tree_identifiers(reader, &geometry, newest_published_txg);
    seen.extend(walk.tree_identifiers_in_tables.iter().copied());
    let highest_seen = seen.iter().max().copied().unwrap_or(0);
    judgements.judge("I-7.8", watermark > highest_seen, || {
        format!("根环水位最大 {watermark}，盘上出现过的最大树 ID {highest_seen}")
    });
    judgements.into_report()
}
```

## `crates/singlefs-checker/src/image.rs`:300-325　`read_referenced_unit` 整个函数——实际定义在 image.rs，不在 walk.rs（walk.rs 只 `use` 它，见该文件第 13 行）

```rust
/// 按位置条目读一个被引用的单元：每条位置条目都读、都判 I-2.1（校验和与内容匹配），返回第一份对得上的。
pub fn read_referenced_unit(
    reader: &dyn ImageReader,
    judgements: &mut Judgements,
    locations: &[PointerLocation; 2],
    unit_bytes: usize,
    what: &str,
) -> Option<Vec<u8>> {
    let mut first_good = None;
    for location in locations {
        let bytes = reader.read(location.device, location.slot * SLOT_BYTES, unit_bytes);
        let matches = bytes
            .as_ref()
            .is_some_and(|content| crate::crc32_castagnoli_table(content) == location.checksum);
        judgements.judge("I-2.1", matches, || {
            format!(
                "{what} 在盘 {} 槽 {} 的那一份与位置条目里的校验和对不上",
                location.device, location.slot
            )
        });
        if matches && first_good.is_none() {
            first_good = bytes;
        }
    }
    first_good
}
```

## `crates/singlefs-checker/src/image.rs`:35-40　`IMPLEMENTED_INVARIANTS` 常量

```rust
/// 第一版 checker 判的不变量，按这个次序报；每次都全部报出来，没评估到的报「不适用」。
pub const IMPLEMENTED_INVARIANTS: [&str; 26] = [
    "I-1.1", "I-1.3", "I-1.4", "I-1.6", "I-1.7", "I-2.1", "I-2.3", "I-2.4", "I-2.5", "I-3.1",
    "I-3.8", "I-4.8", "I-5.1", "I-5.2", "I-7.1", "I-7.2", "I-7.4", "I-7.6", "I-7.7", "I-7.8",
    "I-9.1", "I-9.2", "I-9.4", "I-9.7", "I-9.10", "I-9.13",
];
```

## `crates/singlefs-checker/src/image.rs`:50-99　`Judgements` 整个 `impl` 块（不含 struct 字段定义，任务只点名 impl）

```rust

impl Judgements {
    /// 评估一次：条件不成立就记下这一处（只留第一处）。
    pub fn judge(&mut self, invariant: &'static str, holds: bool, detail: impl FnOnce() -> String) {
        assert!(
            IMPLEMENTED_INVARIANTS.contains(&invariant),
            "{invariant} 不在第一版 checker 的清单里"
        );
        *self.evaluated.entry(invariant).or_insert(0) += 1;
        if !holds {
            *self.violations.entry(invariant).or_insert(0) += 1;
            self.first_violation.entry(invariant).or_insert_with(detail);
        }
    }

    /// 到此为止判了几次违例：走读一条候选根前后各读一次，差就是这条根引用的单元里对不上的那些（I-7.4、I-4.8 按根判）。
    #[must_use]
    pub fn violation_count(&self, invariant: &'static str) -> u64 {
        self.violations.get(invariant).copied().unwrap_or(0)
    }
    /// 这个镜像上整条不适用（例如第 0 代根下面没有记账树）。已经评估过的不改。
    pub fn not_applicable(&mut self, invariant: &'static str, reason: &'static str) {
        assert!(
            IMPLEMENTED_INVARIANTS.contains(&invariant),
            "{invariant} 不在第一版 checker 的清单里"
        );
        self.not_applicable.entry(invariant).or_insert(reason);
    }
    #[must_use]
    pub fn into_report(self) -> Vec<(&'static str, InvariantVerdict)> {
        IMPLEMENTED_INVARIANTS
            .iter()
            .map(|invariant| {
                let verdict = if let Some(detail) = self.first_violation.get(invariant) {
                    InvariantVerdict::Violated(detail.clone())
                } else if self.evaluated.get(invariant).copied().unwrap_or(0) > 0 {
                    InvariantVerdict::Holds
                } else {
                    InvariantVerdict::NotApplicable(
                        self.not_applicable
                            .get(invariant)
                            .copied()
                            .unwrap_or("这个镜像上没有它判的对象"),
                    )
                };
                (*invariant, verdict)
            })
            .collect()
    }
}
```

## `crates/singlefs-harness/tests/checker_known_bad_images.rs`:1-262　文件头到 `known_bad_images` 函数里「I-1.1」那一项之前（含两份新坏镜像 I-7.4 / I-4.8，第 241-262 行）

```rust
//! checker 的已知坏镜像语料（C13（checker 判定失效））：第一版判的 23 条不变量，每条配一份「改坏了正好触发它」的镜像。
//! 每份都从写完第一个事务的干净镜像出发，改一处；要让被改的那一条之前的判定不先挡住，被改的单元重封校验和，
//! 再把新的整单元校验和沿引用链一路补到根记录（父节点里的位置条目、中央映射条目、根槽的自证校验和）。

mod common;

use common::build_pool;
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
use singlefs_core::recovery::PoolReader;
use singlefs_harness::crash::MemoryPool;

const SLOT: u64 = 16384;
const DEVICES: [u32; 2] = [0, 1];
/// 写完第一个事务之后单元区里的单元：(起点槽, 字节数)。
const UNITS: [(u64, usize); 10] = [
    (50176, 32768),
    (50178, 16384),
    (50180, 32768),
    (50240, 16384),
    (50242, 32768),
    (50244, 16384),
    (50245, 16384),
    (50246, 16384),
    (50247, 16384),
    (50248, 16384),
];
const INSTANCE_TABLE: u64 = 50176;
/// mkfs 种下的第 0 版树表单元：第一个文件版本换下它之后只有第 0 代根还引用。
const GENESIS_TREE_TABLE: u64 = 50178;
const DATA_UNIT: u64 = 50180;
const EXTENT_ROOT: u64 = 50240;
const INODE_LEAF: u64 = 50242;
const INODE_ROOT: u64 = 50244;
const ACCOUNTING_ROOT: u64 = 50246;
const TREE_TABLE: u64 = 50248;

fn read(pool: &MemoryPool, device: u32, offset: u64, length: usize) -> Vec<u8> {
    PoolReader::read(
        pool,
        DeviceIdentity(device),
        DeviceOffsetInBytes(offset),
        length,
    )
    .expect("读")
}

fn write(pool: &mut MemoryPool, device: u32, offset: u64, bytes: &[u8]) {
    pool.devices
        .get_mut(&DeviceIdentity(device))
        .expect("盘")
        .write(DeviceOffsetInBytes(offset), bytes);
}

/// 按类重封：先算载荷 CRC，再算头校验和（码 1：头 105、CRC 在 101；码 3：头 107、CRC 在 89；码 2：头 86 + 2k、CRC 在 76 + 2k）。
fn reseal_unit(bytes: &mut [u8]) {
    let (header_end, payload_crc_offset) = match bytes[6] {
        1 => (105usize, 101usize),
        3 => (107, 89),
        _ => (
            86 + 2 * usize::from(bytes[51]),
            76 + 2 * usize::from(bytes[51]),
        ),
    };
    let payload_crc = crc32_castagnoli(&bytes[header_end..]);
    bytes[payload_crc_offset..payload_crc_offset + 4].copy_from_slice(&payload_crc.to_le_bytes());
    singlefs_core::unit::seal_header_checksum(bytes, header_end);
}

/// 根环全部槽：(盘, 偏移)。区域 r 在盘 [0, 1, 0][r] 上，起点 64 × 16384 + r × 3 MiB，槽距 4096。
fn root_slots() -> Vec<(u32, u64)> {
    (0..3u64)
        .flat_map(|region| {
            (0..8u64).map(move |slot| {
                (
                    [0u32, 1, 0][usize::try_from(region).expect("区域")],
                    64 * SLOT + region * 3 * (1 << 20) + slot * 4096,
                )
            })
        })
        .collect()
}

fn location_pattern(device: u32, slot: u64, checksum: u32) -> Vec<u8> {
    [
        device.to_le_bytes().to_vec(),
        slot.to_le_bytes()[..6].to_vec(),
        checksum.to_le_bytes().to_vec(),
    ]
    .concat()
}

fn replace_all(haystack: &mut [u8], needle: &[u8], replacement: &[u8]) -> bool {
    let mut changed = false;
    let mut index = 0;
    while index + needle.len() <= haystack.len() {
        if haystack[index..index + needle.len()] == *needle {
            haystack[index..index + needle.len()].copy_from_slice(replacement);
            changed = true;
            index += needle.len();
        } else {
            index += 1;
        }
    }
    changed
}

/// 一个单元（两盘同槽）的整单元校验和从 old 变成 new：把引用它的位置条目逐个补上，被补的单元重封、再往上补，直到根槽。
fn propagate(pool: &mut MemoryPool, mut changes: Vec<(u64, u32, u32)>) {
    while let Some((slot, old, new)) = changes.pop() {
        if old == new {
            continue;
        }
        for (container, length) in UNITS {
            if container == slot {
                continue;
            }
            let mut bytes = read(pool, 0, container * SLOT, length);
            let mut touched = false;
            for device in DEVICES {
                touched |= replace_all(
                    &mut bytes,
                    &location_pattern(device, slot, old),
                    &location_pattern(device, slot, new),
                );
            }
            if touched {
                let before = crc32_castagnoli(&read(pool, 0, container * SLOT, length));
                reseal_unit(&mut bytes);
                for device in DEVICES {
                    write(pool, device, container * SLOT, &bytes);
                }
                changes.push((container, before, crc32_castagnoli(&bytes)));
            }
        }
        for (device, offset) in root_slots() {
            let mut bytes = read(pool, device, offset, 512);
            let mut touched = false;
            for location_device in DEVICES {
                touched |= replace_all(
                    &mut bytes,
                    &location_pattern(location_device, slot, old),
                    &location_pattern(location_device, slot, new),
                );
            }
            if touched {
                let digest = wide_checksum_with_field_zeroed(&bytes, 512, 138);
                bytes[138..170].copy_from_slice(&digest);
                write(pool, device, offset, &bytes);
            }
        }
    }
}

/// 改一个单元（两盘一起改）：`reseal` 为真时重封它的两道校验和；新的整单元校验和沿引用链补上去。
fn mutate_unit(pool: &mut MemoryPool, slot: u64, change: impl Fn(&mut Vec<u8>), reseal: bool) {
    let length = UNITS
        .iter()
        .find(|(start, _)| *start == slot)
        .expect("登记过的单元")
        .1;
    let before = read(pool, 0, slot * SLOT, length);
    let mut bytes = before.clone();
    change(&mut bytes);
    if reseal {
        reseal_unit(&mut bytes);
    }
    for device in DEVICES {
        write(pool, device, slot * SLOT, &bytes);
    }
    propagate(
        pool,
        vec![(slot, crc32_castagnoli(&before), crc32_castagnoli(&bytes))],
    );
}

fn verdict(pool: &MemoryPool, invariant: &str) -> InvariantVerdict {
    check_pool_image(pool)
        .into_iter()
        .find(|(identifier, _)| *identifier == invariant)
        .expect("清单里有")
        .1
}

/// 超级块槽重封：自证校验和在 155，罩整槽 4096。
fn mutate_superblock_slot(
    pool: &mut MemoryPool,
    device: u32,
    slot_offset: u64,
    change: impl Fn(&mut Vec<u8>),
) {
    let mut bytes = read(pool, device, slot_offset, 4096);
    change(&mut bytes);
    let digest = wide_checksum_with_field_zeroed(&bytes, 4096, 155);
    bytes[155..187].copy_from_slice(&digest);
    write(pool, device, slot_offset, &bytes);
}

fn set_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}
fn set_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

/// 树表单元里第 index 条条目（条目区从 115 + 2 × 8 = 131 起，每条 200）。
fn tree_table_entry_offset(index: usize) -> usize {
    131 + 200 * index
}

/// 一份坏镜像：它要让哪一条判违例，以及怎么从干净镜像改出来。
type Mutation = Box<dyn Fn(&mut MemoryPool)>;

fn known_bad_images(clean: &MemoryPool) -> Vec<(&'static str, Mutation)> {
    let leaf_head_checksum = crc32_castagnoli(&read(clean, 0, INODE_LEAF * SLOT, 16384));
    let stray_node = {
        let mut stray = read(clean, 0, EXTENT_ROOT * SLOT, 16384);
        set_u64(&mut stray, 42, 99);
        reseal_unit(&mut stray);
        stray
    };
    let orphan_with_instance_two = {
        let mut orphan = read(clean, 0, DATA_UNIT * SLOT, 32768);
        orphan[91..95].copy_from_slice(&2u32.to_le_bytes());
        reseal_unit(&mut orphan);
        orphan
    };
    let orphan_for_second_sentence = orphan_with_instance_two.clone();
    let record_with_instance_two = {
        let offset =
            singlefs_core::journal::record_offset(3, singlefs_format::JOURNAL_RING_DEFAULT_BYTES).0;
        let mut record = read(clean, 0, offset, 4096);
        record[16..20].copy_from_slice(&2u32.to_le_bytes());
        let digest = wide_checksum_with_field_zeroed(&record, 4096, 46);
        record[46..78].copy_from_slice(&digest);
        record
    };
    vec![
        // 第 0 代根（暖机 txg 1、2）引用的 mkfs 树表单元（50178，A 换下之后只有它们引用）内容改掉再重封：那条候选根指着的块被复用。
        (
            "I-7.4",
            Box::new(|image: &mut MemoryPool| {
                let mut unit = read(image, 0, GENESIS_TREE_TABLE * SLOT, 16384);
                set_u64(&mut unit, 42, 77);
                reseal_unit(&mut unit);
                for device in DEVICES {
                    write(image, device, GENESIS_TREE_TABLE * SLOT, &unit);
                }
            }),
        ),
        // 同一个单元的头抹成零、不重封：从第 0 代根出发的遍历读不到它。
        (
            "I-4.8",
            Box::new(|image: &mut MemoryPool| {
                let erased = vec![0u8; 16384];
                for device in DEVICES {
                    write(image, device, GENESIS_TREE_TABLE * SLOT, &erased);
                }
            }),
        ),
```

## `crates/singlefs-harness/tests/checker_known_bad_images.rs`:541-571　文件末尾那条 `#[test]`

```rust
#[test]
fn the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target() {
    let clean = build_pool("known-bad").memory_pool();
    for (invariant, found) in check_pool_image(&clean) {
        assert_eq!(
            found,
            InvariantVerdict::Holds,
            "干净镜像上 {invariant} 要成立"
        );
    }
    let cases = known_bad_images(&clean);
    let targets: std::collections::BTreeSet<&str> =
        cases.iter().map(|(invariant, _)| *invariant).collect();
    let listed: std::collections::BTreeSet<&str> = singlefs_checker::image::IMPLEMENTED_INVARIANTS
        .iter()
        .copied()
        .collect();
    assert_eq!(
        targets, listed,
        "第一版每条不变量都至少有一份坏镜像，也没有清单之外的"
    );
    for (invariant, mutation) in cases {
        let mut image = clean.clone();
        mutation(&mut image);
        let found = verdict(&image, invariant);
        assert!(
            matches!(found, InvariantVerdict::Violated(_)),
            "{invariant} 那份坏镜像应当判违例，实际 {found:?}"
        );
    }
}
```

## `crates/mutations.tsv`:59-60　以「步 6」开头的两行

```tsv
步 6：I-7.4 不判候选根引用的单元被复用或抹头	crates/singlefs-checker/src/walk.rs	                .judge("I-7.4", !walked_into_reused_or_erased_unit, || {	                .judge("I-7.4", true, || {	-p singlefs-harness --test checker_known_bad_images	the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target
步 6：I-4.8 只看最新根、不看别的候选根	crates/singlefs-checker/src/walk.rs	                .judge("I-4.8", !walked_into_reused_or_erased_unit, || {	                .judge("I-4.8", true, || {	-p singlefs-harness --test checker_known_bad_images	the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target
```

