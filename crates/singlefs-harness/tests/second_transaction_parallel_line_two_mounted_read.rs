//! 里程碑「第二个事务」并行线二（挂载态的读：打开一个文件、按偏移反复读）的验收。
//!
//! 四条验收标准各一条用例：
//! 1. 打开之后随机读 K 次 4 KiB，每次内容与写入逐字节相同；K 次读期间 journal 环一次都不扫
//!    （块层读计数由 `singlefs_harness::read_tally` 数，整扫一遍环是 196 608 次 / 盘，读期间要恒 0）；
//! 2. 改坏一个数据单元的一个字节 ⇒ 落在它上面的读报校验和错，**别的读不受影响**；
//! 3. 同一个镜像冷走读（`recovery::recover`）与挂载态读逐字节相同——层 0 的每个恢复结果上再跑一遍那一半归
//!    crash-verifier，这里只把接口（`mounted_read::mount_read_only`）做出来并钉住「同一个镜像两条路同字节」；
//! 4. 读路径报得出「这次读了几个单元」与「位置提示过期的多跳次数」（D19（块指针的结构与宽度预算） 已定项 5 硬规则 3）。
//!
//! **跨单元的页**：净荷 32634 不是 4096 的整数倍 ⇒ 约 12.5% 的 4 KiB 页跨两个单元（D4（校验和位置） 已定项 5）。
//! 跨单元那一格喂给读路径的是本文件自己拼的一份多单元镜像：
//! **它不是发布路径产出的**，单元、树节点、映射条目、根记录都由这里按字节表直接装出来写进内存盘（提示指错、映射根成了内部节点、
//! 条目宽窄于字段表那几格发布路径造不出来）。能这么做的依据是读路径只认盘上的字节。发布路径写出的多单元文件
//! （并行线一，`publish_sequential_write`）的冷启动顺序读在 `second_transaction_parallel_line_one_multi_unit_file.rs`。
//!
//! extent 叶记录 key 第三段是文件字节偏移（D8（核心索引结构） 已定项 3，C490（extent 叶 key 的 offset 段没定单位） 2026-09-23 定）：
//! 合成镜像把它写成单元序号时，挂载态打开这个文件当场拒绝、不按位次猜着读
//! （`a_mount_state_open_refuses_an_extent_key_whose_offset_segment_is_the_unit_index_instead_of_the_file_byte_offset`）。

mod common;

use common::{build_pool, E142_FILESYSTEM_IDENTIFIER, FILE_BYTES, IMAGE_BYTES};
use singlefs_core::address::{
    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes,
    InodeNumber, InstanceGeneration, SlotNumber, TreeIdentifier,
};
use singlefs_core::extent_tree::{
    build_upper_leaf, ExtentLowerNodePosition, ExtentTreeNodeIdentity, ExtentUpperLeafEntry,
    ExtentUpperLeafTarget, ExtentUpperNodePosition,
};
use singlefs_core::make_filesystem::{location_entries, TREE_TABLE_KEY_WIDTH};
use singlefs_core::mounted_read::{
    data_unit_span_covering, mount_read_only, open_pool_for_read, CentralMappingTreeReadsAtOpen,
    FileReadFailure, OpenFileFailure, OpenPoolForReadFailure, ReadPathObservation,
};
use singlefs_core::pointer::{BirthSequence, DataPointer, LocationEntry, NodePointer, PointerHead};
use singlefs_core::records::{
    build_extent_record, build_inode_internal_entry, build_mapping_entry, mapping_key_for_data,
    InodeRecord, TreeTableEntry, TREE_KIND_EXTENT, TREE_KIND_INODE,
};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryFailure, RecoveryOutcome};
use singlefs_core::root_record::RootRecord;
use singlefs_core::transaction::{FIRST_INODE_NUMBER, TREE_IDENTIFIER_NONE};
use singlefs_core::unit::{
    build_data_unit, build_index_node, build_packed_unit, data_unit_payload_capacity,
    DataUnitIdentity, PackedIdentity, WriteOrder, PACKED_TYPE_INODE,
};
use singlefs_format::{
    EXTENT_LEAF_RECORD_BYTES, INODE_INTERNAL_ENTRY, INODE_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES,
    MAPPING_ENTRY_BYTES, MAPPING_KEY_BYTES, TREE_IDENTIFIER_EXTENT, TREE_IDENTIFIER_INODE,
    TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH, TREE_TABLE_ENTRY_BYTES, UNIT_AREA_START_SLOT,
};
use singlefs_harness::crash::MemoryPool;
use singlefs_harness::read_tally::{JournalRingRegion, ReadCountingPoolReader};

/// 一页 4 KiB：E152（按里程碑对比六家文件系统的文件性能） 的随机读作业按这个粒度发。
const PAGE_BYTES: u64 = 4096;
/// 这一轮随机读几次。
const RANDOM_READS: u64 = 64;
/// 合成镜像放在单元区起点之后 4096 个槽处：发布路径从单元区起点往上取最低空槽对，离得远一点，
/// 免得与「这份镜像是拼出来的」这件事混在一起。
const SYNTHETIC_BASE_SLOT: u64 = UNIT_AREA_START_SLOT + 4096;
/// 合成镜像里这个文件的 inode 出生代与每个单元的诞生代号。
const SYNTHETIC_TXG: CheckpointTxg = CheckpointTxg(3);
const SYNTHETIC_INSTANCE: InstanceGeneration = InstanceGeneration(1);

fn journal_ring() -> JournalRingRegion {
    JournalRingRegion::starting_at_the_standard_slot(JOURNAL_RING_DEFAULT_BYTES)
}

fn payload_capacity() -> u64 {
    u64::try_from(data_unit_payload_capacity()).expect("32634")
}

/// 定长的伪随机字节流（xorshift64，固定种子）：周期远长于这几个单元，错位一个字节就对不上——
/// 按 `index % 251` 那种周期性填充会让偏移差 251 的倍数的错位照样比中。
fn pseudo_random_bytes(length: usize, seed: u64) -> Vec<u8> {
    let mut state = seed;
    (0..length)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            u8::try_from(state & 0xff).expect("低 8 位")
        })
        .collect()
}

/// 固定种子的伪随机数，取来选随机读的偏移。
struct PseudoRandomOffsets {
    state: u64,
}

impl PseudoRandomOffsets {
    fn seeded(seed: u64) -> Self {
        Self { state: seed }
    }
    fn next_below(&mut self, exclusive_upper_bound: u64) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state % exclusive_upper_bound
    }
}

/// extent 叶记录 key 第三段（与数据单元头里的锚点偏移）按哪种读法写：条款定的是文件字节偏移，单元序号那一档造来判读路径拒绝它。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExtentKeyThirdSegmentReading {
    FileByteOffset,
    DataUnitIndex,
}

impl ExtentKeyThirdSegmentReading {
    fn value_for(self, unit_index: u64) -> u64 {
        match self {
            ExtentKeyThirdSegmentReading::FileByteOffset => unit_index * payload_capacity(),
            ExtentKeyThirdSegmentReading::DataUnitIndex => unit_index,
        }
    }
}

/// 造这份镜像时位置提示要不要指错：指错的那一个单元靠中央映射才读得到（D19（块指针的结构与宽度预算） 已定项 5）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LocationHintDamage {
    None,
    PointTheHintOfThisUnitAtAnEmptySlot(DataUnitIndexInFile),
}

/// 一份拼出来的多单元镜像：两块内存盘、一条根记录、这个文件的内容，以及每个数据单元的落点。
struct SyntheticMultiUnitImage {
    image: MemoryPool,
    root: RootRecord,
    content: Vec<u8>,
    data_unit_slots: Vec<SlotNumber>,
}

impl SyntheticMultiUnitImage {
    fn file_size_in_bytes(&self) -> u64 {
        u64::try_from(self.content.len()).expect("文件大小")
    }

    /// 一个数据单元在盘上的字节偏移。
    fn data_unit_offset(&self, unit_index: DataUnitIndexInFile) -> DeviceOffsetInBytes {
        self.data_unit_slots[usize::try_from(unit_index.0).expect("单元序号")].to_device_offset()
    }

    /// 两块盘上同一个数据单元的同一个字节各翻一下：一条位置条目都不剩下能对上校验和的。
    fn flip_one_byte_in_every_copy_of_data_unit(&mut self, unit_index: DataUnitIndexInFile) {
        let offset = self.data_unit_offset(unit_index);
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            self.image.flip_byte(device, offset, 200);
        }
    }

    /// 把一个数据单元换成另一份**自己的校验和全对**的单元（头校验和、载荷 CRC 都重算过），
    /// 而位置条目里的整单元校验和还是旧的：只有位置条目那一道拦得住它。
    fn reseal_data_unit_with_other_content(
        &mut self,
        unit_index: DataUnitIndexInFile,
        payload: &[u8],
    ) {
        let unit = build_data_unit(
            data_unit_identity(unit_index, ExtentKeyThirdSegmentReading::FileByteOffset),
            SYNTHETIC_TXG,
            &E142_FILESYSTEM_IDENTIFIER,
            write_order_of_data_unit(unit_index),
            payload,
        );
        let offset = self.data_unit_offset(unit_index);
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            self.image
                .devices
                .get_mut(&device)
                .expect("池里有这块盘")
                .write(offset, &unit);
        }
    }
}

/// 一事务一单元（D16（发布语义） 已定项 5 末段）⇒ 每个单元的写序各不相同，中央映射的 key 才不撞
/// （码 1 的 key = 类标签 + 出生树 + 出生 txg + 写序，D19（块指针的结构与宽度预算） 已定项 6）。
fn write_order_of_data_unit(unit_index: DataUnitIndexInFile) -> WriteOrder {
    WriteOrder {
        instance: SYNTHETIC_INSTANCE,
        transaction: unit_index.0 + 1,
    }
}

fn data_unit_identity(
    unit_index: DataUnitIndexInFile,
    reading: ExtentKeyThirdSegmentReading,
) -> DataUnitIdentity {
    DataUnitIdentity {
        tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
        object: FIRST_INODE_NUMBER,
        object_birth: SYNTHETIC_TXG,
        // 锚点偏移与 key 第三段是同一个数（D9（加密） 已定项 6）：这里按同一种读法写。
        anchor_offset: reading.value_for(unit_index.0),
    }
}

/// 把一个单元写进两块盘的同一个槽，交回它的两条位置条目。
fn place_on_both_devices(
    image: &mut MemoryPool,
    slot: SlotNumber,
    unit: &[u8],
) -> [LocationEntry; 2] {
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        image
            .devices
            .get_mut(&device)
            .expect("池里有这块盘")
            .write(slot.to_device_offset(), unit);
    }
    location_entries(&[DeviceIdentity(0), DeviceIdentity(1)], slot, unit)
}

/// 拼这份镜像的参数。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SyntheticImagePlan {
    unit_count: u64,
    reading: ExtentKeyThirdSegmentReading,
    damage: LocationHintDamage,
    /// inode 记录里写的文件大小。`None` = 按单元数算出来的真实大小；写别的值就造出
    /// 「extent 记录条数与文件大小算出的单元数对不上」那一格。
    declared_file_size_in_bytes: Option<u64>,
    /// 中央映射树长什么样：根兼叶，两片叶上面一个根（多层，D8（核心索引结构） 已定项 11），
    /// 或根自述层级 1 而条目仍是 55 字节的映射条目（读者要拒，不把映射条目当内部条目解）。
    central_mapping_shape: SyntheticCentralMappingShape,
    /// 中央映射树根自述的条目宽，条目跟着截到这么宽。第一版恒 55（key 27 + 位置条目 14 × 2）；写 27（= key 宽）
    /// 就造出「条目宽窄于字段表」那一格——`parse_index_node` 只判了它 ≥ key 宽，切到偏移 55 就越界（panic 面普查 R2）。
    central_mapping_entry_width: u16,
}

impl SyntheticImagePlan {
    /// 四个单元、key 第三段按文件字节偏移、提示不指错、大小如实写：别的用例从这一份改一处。
    fn of_four_units() -> Self {
        Self {
            unit_count: 4,
            reading: ExtentKeyThirdSegmentReading::FileByteOffset,
            damage: LocationHintDamage::None,
            declared_file_size_in_bytes: None,
            central_mapping_shape: SyntheticCentralMappingShape::RootAsLeaf,
            central_mapping_entry_width: 55,
        }
    }
}

/// 拼进镜像的中央映射树的形状。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SyntheticCentralMappingShape {
    /// 一个节点（根兼叶），55 字节的映射条目直接装在根里。
    RootAsLeaf,
    /// 条目按 key 升序对半分到两片叶（层级 0），上面一个层级 1 的根：内部条目 = 分隔 key 27 + 子指针 86 = 113，
    /// 根头里的 key 区间是子树覆盖区间（D8（核心索引结构） 已定项 11、D18（块里携带什么信息） 已定项 2）。
    /// 两片叶的出生序号是 0、1，根是 2（树内先叶后根，D19（块指针的结构与宽度预算） 已定项 9）。
    TwoLeavesUnderARoot,
    /// 根自述层级 1、装的却是 55 字节的映射条目：内部条目要 113 字节，读者当场拒。
    RootClaimingLevelOneOverMappingEntries,
}

/// 拼一份 `plan.unit_count` 个数据单元的镜像：最后一个单元装不满（声明长度那一格才有内容可判）。
///
/// 槽位排布（都写两块盘、同槽）：数据单元各占 2 槽，之后依次是 extent 树下段那片叶、inode 树根、inode 叶容器（2 槽）、
/// 中央映射树根、树表单元，留一个空槽给「提示指错」那一档，再往后是两层映射那一档的两片映射叶，最后是 extent 树上段根兼叶。
fn build_synthetic_multi_unit_image(plan: SyntheticImagePlan) -> SyntheticMultiUnitImage {
    let SyntheticImagePlan {
        unit_count,
        reading,
        damage,
        declared_file_size_in_bytes,
        central_mapping_shape,
        central_mapping_entry_width,
    } = plan;
    assert!(unit_count >= 2, "拼这份镜像就是为了跨单元那一格");
    let capacity = payload_capacity();
    let file_size = capacity * (unit_count - 1) + 5000;
    let content = pseudo_random_bytes(usize::try_from(file_size).expect("文件大小"), 0x5f_53_46_53);
    let mut image = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);

    // 数据单元与 extent 叶记录：第 i 个单元装文件字节 [i × 净荷容量, min(文件大小, (i+1) × 净荷容量))。
    let mut data_unit_slots = Vec::new();
    let mut extent_records = Vec::new();
    let mut mapping_entries = Vec::new();
    let empty_slot = SlotNumber(SYNTHETIC_BASE_SLOT + 2 * unit_count + 6);
    for unit_number in 0..unit_count {
        let unit_index = DataUnitIndexInFile(unit_number);
        let payload_start = usize::try_from(unit_number * capacity).expect("载荷起点");
        let payload_end =
            usize::try_from(((unit_number + 1) * capacity).min(file_size)).expect("载荷终点");
        let unit = build_data_unit(
            data_unit_identity(unit_index, reading),
            SYNTHETIC_TXG,
            &E142_FILESYSTEM_IDENTIFIER,
            write_order_of_data_unit(unit_index),
            &content[payload_start..payload_end],
        );
        let slot = SlotNumber(SYNTHETIC_BASE_SLOT + 2 * unit_number);
        let true_locations = place_on_both_devices(&mut image, slot, &unit);
        data_unit_slots.push(slot);
        let hinted_locations = match damage {
            LocationHintDamage::None => true_locations,
            LocationHintDamage::PointTheHintOfThisUnitAtAnEmptySlot(damaged) => {
                if damaged == unit_index {
                    // 槽换成一个从来没写过的：读回来是全 0，整单元校验和对不上 ⇒ 只能经映射再解一次。
                    [
                        LocationEntry {
                            slot: empty_slot,
                            ..true_locations[0]
                        },
                        LocationEntry {
                            slot: empty_slot,
                            ..true_locations[1]
                        },
                    ]
                } else {
                    true_locations
                }
            }
        };
        let head = PointerHead {
            birth_tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
            birth_txg: SYNTHETIC_TXG,
        };
        let write_order = write_order_of_data_unit(unit_index);
        extent_records.push(build_extent_record(
            FIRST_INODE_NUMBER,
            reading.value_for(unit_number),
            DataPointer {
                head,
                locations: hinted_locations,
                write_order,
            },
        ));
        // 中央映射里恒是真落点（D19（块指针的结构与宽度预算） 已定项 5：搬迁只改这里一条，提示可以过期）。
        mapping_entries.push(build_mapping_entry(
            &mapping_key_for_data(head, write_order),
            true_locations,
        ));
    }

    // extent 树两段（D8（核心索引结构） 已定项 14）：下段一片叶（单元不到 144 个）罩单元 0 起的那一段，头里写位置规定的 key 区间；
    // 上段根兼叶罩 inode 0 起的那一段，inode 1 那条条目标签 1、指着下段根。先下段后上段（树内 bump 次序）。
    assert!(
        unit_count <= singlefs_format::EXTENT_TREE_LOWER_LEAF_DATA_UNITS,
        "这份镜像的下段只拼一片叶"
    );
    let lower_leaf_position = ExtentLowerNodePosition { level: 0, index: 0 };
    let (lower_leaf_smallest_key, lower_leaf_largest_key) =
        lower_leaf_position.key_range(FIRST_INODE_NUMBER);
    let lower_leaf_slot = SlotNumber(SYNTHETIC_BASE_SLOT + 2 * unit_count);
    let lower_leaf_sequence = BirthSequence(0);
    let lower_leaf = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_EXTENT),
        0,
        usize::try_from(singlefs_format::EXTENT_KEY_BYTES).expect("24"),
        &lower_leaf_smallest_key,
        &lower_leaf_largest_key,
        SYNTHETIC_TXG,
        &E142_FILESYSTEM_IDENTIFIER,
        SYNTHETIC_INSTANCE,
        lower_leaf_sequence,
        u16::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112"),
        &extent_records,
    );
    let lower_leaf_locations = place_on_both_devices(&mut image, lower_leaf_slot, &lower_leaf);
    let extent_root_slot = SlotNumber(SYNTHETIC_BASE_SLOT + 2 * unit_count + 9);
    let extent_root_sequence = BirthSequence(1);
    let extent_root = build_upper_leaf(
        &ExtentTreeNodeIdentity {
            tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
            birth_txg: SYNTHETIC_TXG,
            filesystem_identifier: &E142_FILESYSTEM_IDENTIFIER,
            instance: SYNTHETIC_INSTANCE,
        },
        ExtentUpperNodePosition { level: 0, index: 0 },
        &[ExtentUpperLeafEntry {
            inode: FIRST_INODE_NUMBER,
            target: ExtentUpperLeafTarget::LowerSegmentRoot(NodePointer {
                head: PointerHead {
                    birth_tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
                    birth_txg: SYNTHETIC_TXG,
                },
                locations: lower_leaf_locations,
                instance: SYNTHETIC_INSTANCE,
                birth_sequence: lower_leaf_sequence,
            }),
        }],
        extent_root_sequence,
    );
    let extent_root_locations = place_on_both_devices(&mut image, extent_root_slot, &extent_root);

    // inode 树：根（码 2、层级 1）一条条目 → 一片码 3 叶容器 → 这个文件那条记录。
    let inode_record = InodeRecord {
        inode: FIRST_INODE_NUMBER,
        object_birth: SYNTHETIC_TXG,
        size: declared_file_size_in_bytes.unwrap_or(file_size),
        change_count: SYNTHETIC_TXG.0,
        write_time_seconds: common::FIXED_WRITE_TIME_SECONDS,
    };
    let container_identity = PackedIdentity {
        birth_tree: TreeIdentifier(TREE_IDENTIFIER_INODE),
        record_type: PACKED_TYPE_INODE,
        container: FIRST_INODE_NUMBER,
        container_birth: SYNTHETIC_TXG,
    };
    let leaf_sequence = BirthSequence(0);
    let leaf_container = build_packed_unit(
        container_identity,
        u16::try_from(INODE_RECORD_BYTES).expect("140"),
        &[inode_record.to_bytes()],
        SYNTHETIC_TXG,
        &E142_FILESYSTEM_IDENTIFIER,
        write_order_of_data_unit(DataUnitIndexInFile(0)),
        leaf_sequence,
    );
    let leaf_slot = SlotNumber(SYNTHETIC_BASE_SLOT + 2 * unit_count + 2);
    let leaf_locations = place_on_both_devices(&mut image, leaf_slot, &leaf_container);
    let inode_internal_entry = build_inode_internal_entry(
        FIRST_INODE_NUMBER,
        container_identity,
        NodePointer {
            head: PointerHead {
                birth_tree: TreeIdentifier(TREE_IDENTIFIER_INODE),
                birth_txg: SYNTHETIC_TXG,
            },
            locations: leaf_locations,
            instance: SYNTHETIC_INSTANCE,
            birth_sequence: leaf_sequence,
        },
    );
    let inode_root_sequence = BirthSequence(1);
    let inode_root = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_INODE),
        1,
        8,
        &FIRST_INODE_NUMBER.to_le_bytes(),
        &FIRST_INODE_NUMBER.to_le_bytes(),
        SYNTHETIC_TXG,
        &E142_FILESYSTEM_IDENTIFIER,
        SYNTHETIC_INSTANCE,
        inode_root_sequence,
        u16::try_from(INODE_INTERNAL_ENTRY).expect("120"),
        std::slice::from_ref(&inode_internal_entry),
    );
    let inode_root_slot = SlotNumber(SYNTHETIC_BASE_SLOT + 2 * unit_count + 1);
    let inode_root_locations = place_on_both_devices(&mut image, inode_root_slot, &inode_root);

    let mapping_key_width = usize::try_from(MAPPING_KEY_BYTES).expect("27");
    // 条目跟着自述的条目宽截：`central_mapping_entry_width` 不是 55 的那一档，节点里装的就是被截短的条目
    // （节点自己仍然自洽——条目数 × 条目宽 = 声明长度、两道校验和都对得上，挡着它的只能是读者那一判）。
    assert!(
        usize::from(central_mapping_entry_width) >= mapping_key_width
            && u64::from(central_mapping_entry_width) <= MAPPING_ENTRY_BYTES,
        "自述的条目宽要在 key 宽与字段表宽之间，不然拼出来的不是这一格"
    );
    let mapping_entries_at_the_declared_width: Vec<Vec<u8>> = mapping_entries
        .iter()
        .map(|entry| entry[..usize::from(central_mapping_entry_width)].to_vec())
        .collect();
    let mapping_node = |level: u8,
                        entries: &[Vec<u8>],
                        entry_width: u16,
                        smallest: &[u8],
                        largest: &[u8],
                        sequence: BirthSequence| {
        build_index_node(
            TreeIdentifier(singlefs_format::TREE_IDENTIFIER_CENTRAL_MAPPING),
            level,
            mapping_key_width,
            smallest,
            largest,
            SYNTHETIC_TXG,
            &E142_FILESYSTEM_IDENTIFIER,
            SYNTHETIC_INSTANCE,
            sequence,
            entry_width,
            entries,
        )
    };
    let key_of = |entry: &Vec<u8>| entry[..mapping_key_width].to_vec();
    let mapping_root_slot = SlotNumber(SYNTHETIC_BASE_SLOT + 2 * unit_count + 4);
    let (mapping_root, mapping_root_sequence) = match central_mapping_shape {
        SyntheticCentralMappingShape::RootAsLeaf
        | SyntheticCentralMappingShape::RootClaimingLevelOneOverMappingEntries => {
            let level = match central_mapping_shape {
                SyntheticCentralMappingShape::RootClaimingLevelOneOverMappingEntries => 1,
                SyntheticCentralMappingShape::RootAsLeaf
                | SyntheticCentralMappingShape::TwoLeavesUnderARoot => 0,
            };
            (
                mapping_node(
                    level,
                    &mapping_entries_at_the_declared_width,
                    central_mapping_entry_width,
                    &key_of(&mapping_entries[0]),
                    &key_of(&mapping_entries[mapping_entries.len() - 1]),
                    BirthSequence(0),
                ),
                BirthSequence(0),
            )
        }
        SyntheticCentralMappingShape::TwoLeavesUnderARoot => {
            let half = mapping_entries_at_the_declared_width.len().div_ceil(2);
            let mut internal_entries = Vec::new();
            for (leaf_number, leaf_entries) in mapping_entries_at_the_declared_width
                .chunks(half)
                .enumerate()
            {
                let sequence = BirthSequence(u32::try_from(leaf_number).expect("两片叶"));
                let leaf = mapping_node(
                    0,
                    leaf_entries,
                    central_mapping_entry_width,
                    &key_of(&leaf_entries[0]),
                    &key_of(&leaf_entries[leaf_entries.len() - 1]),
                    sequence,
                );
                let mapping_leaf_slot = SlotNumber(
                    SYNTHETIC_BASE_SLOT
                        + 2 * unit_count
                        + 7
                        + u64::try_from(leaf_number).expect("叶序"),
                );
                let mapping_leaf_locations =
                    place_on_both_devices(&mut image, mapping_leaf_slot, &leaf);
                let mut internal_entry = key_of(&leaf_entries[0]);
                let mut writer = singlefs_core::bytes::ByteWriter::new(
                    usize::try_from(singlefs_format::NODE_POINTER_BYTES).expect("86"),
                );
                NodePointer {
                    head: PointerHead {
                        birth_tree: TreeIdentifier(
                            singlefs_format::TREE_IDENTIFIER_CENTRAL_MAPPING,
                        ),
                        birth_txg: SYNTHETIC_TXG,
                    },
                    locations: mapping_leaf_locations,
                    instance: SYNTHETIC_INSTANCE,
                    birth_sequence: sequence,
                }
                .write_to(&mut writer);
                internal_entry.extend(writer.into_bytes());
                internal_entries.push(internal_entry);
            }
            let root_sequence = BirthSequence(2);
            (
                mapping_node(
                    1,
                    &internal_entries,
                    u16::try_from(mapping_key_width).expect("27")
                        + u16::try_from(singlefs_format::NODE_POINTER_BYTES).expect("86"),
                    &key_of(&mapping_entries[0]),
                    &key_of(&mapping_entries[mapping_entries.len() - 1]),
                    root_sequence,
                ),
                root_sequence,
            )
        }
    };
    let mapping_root_locations =
        place_on_both_devices(&mut image, mapping_root_slot, &mapping_root);

    let tree_table_entries = vec![
        TreeTableEntry {
            kind: TREE_KIND_EXTENT,
            tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
            root: NodePointer {
                head: PointerHead {
                    birth_tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
                    birth_txg: SYNTHETIC_TXG,
                },
                locations: extent_root_locations,
                instance: SYNTHETIC_INSTANCE,
                birth_sequence: extent_root_sequence,
            },
            birth_txg: SYNTHETIC_TXG,
            head_identifier: 0,
        }
        .to_bytes(),
        TreeTableEntry {
            kind: TREE_KIND_INODE,
            tree: TreeIdentifier(TREE_IDENTIFIER_INODE),
            root: NodePointer {
                head: PointerHead {
                    birth_tree: TreeIdentifier(TREE_IDENTIFIER_INODE),
                    birth_txg: SYNTHETIC_TXG,
                },
                locations: inode_root_locations,
                instance: SYNTHETIC_INSTANCE,
                birth_sequence: inode_root_sequence,
            },
            birth_txg: SYNTHETIC_TXG,
            head_identifier: 0,
        }
        .to_bytes(),
    ];
    let tree_table = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_NONE),
        0,
        TREE_TABLE_KEY_WIDTH,
        &TREE_IDENTIFIER_EXTENT.to_le_bytes(),
        &TREE_IDENTIFIER_INODE.to_le_bytes(),
        SYNTHETIC_TXG,
        &E142_FILESYSTEM_IDENTIFIER,
        SYNTHETIC_INSTANCE,
        BirthSequence(0),
        u16::try_from(TREE_TABLE_ENTRY_BYTES).expect("200"),
        &tree_table_entries,
    );
    let tree_table_slot = SlotNumber(SYNTHETIC_BASE_SLOT + 2 * unit_count + 5);
    let tree_table_locations = place_on_both_devices(&mut image, tree_table_slot, &tree_table);

    // 根记录只住内存：这份镜像不进根环、不走恢复，读路径按这条根直接打开挂载态。
    let root = RootRecord {
        filesystem_identifier: E142_FILESYSTEM_IDENTIFIER,
        instance: SYNTHETIC_INSTANCE,
        checkpoint_txg: SYNTHETIC_TXG,
        tree_table: NodePointer {
            head: PointerHead {
                birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
                birth_txg: SYNTHETIC_TXG,
            },
            locations: tree_table_locations,
            instance: SYNTHETIC_INSTANCE,
            birth_sequence: BirthSequence(0),
        },
        tree_identifier_watermark: TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
        rollback_floor: CheckpointTxg(0),
        instance_table: NodePointer::empty_root(),
        mapping_root: NodePointer {
            head: PointerHead {
                birth_tree: TreeIdentifier(singlefs_format::TREE_IDENTIFIER_CENTRAL_MAPPING),
                birth_txg: SYNTHETIC_TXG,
            },
            locations: mapping_root_locations,
            instance: SYNTHETIC_INSTANCE,
            birth_sequence: mapping_root_sequence,
        },
        // 这是一条带文件的根（树表里有条目）：分配记录树住树表条目，根记录那一项恒全零。
        allocation_record_tree_root: NodePointer::empty_root(),
    };

    SyntheticMultiUnitImage {
        image,
        root,
        content,
        data_unit_slots,
    }
}

/// 验收第 3 条（的可做那一半）：同一个镜像，冷走读（择根 → 扫环 → 沿树走到数据单元）与挂载态读逐字节相同。
///
/// 顺带把验收第 1 条那个对照摆出来：**挂载那一步**要全环扫一遍（两块盘各 196 608 次记录读，
/// 因为文件后端交不出 journal 记录落点的提示），**打开之后的读**一次都不碰环。
/// 层 0 的每个恢复结果上再跑一遍这一条归 crash-verifier：接口是 `mount_read_only`，`reader` 换成崩溃镜像即可。
#[test]
fn the_same_image_read_cold_and_read_through_the_mount_state_gives_the_same_bytes() {
    let mut pool = build_pool("parallel-line-two-cold-versus-mounted");
    let reopened = pool.reopen_cold();

    let cold = recover(&reopened, JournalPolicy::Consult);
    let RecoveryOutcome::FileRead {
        root: cold_root,
        content: cold_content,
    } = cold.outcome
    else {
        panic!("冷走读该读回第一个事务的文件，实际 {:?}", cold.outcome);
    };
    assert_eq!(
        cold_content.len(),
        FILE_BYTES,
        "E142 的第一个文件 3000 字节"
    );

    let counting = ReadCountingPoolReader::new(&reopened, journal_ring());
    let mounted = mount_read_only(&counting).expect("只读挂载");
    assert_eq!(
        (
            mounted.effective_root.instance,
            mounted.effective_root.checkpoint_txg
        ),
        cold_root,
        "两条路择到同一条根"
    );
    let while_mounting = counting.tally();
    assert_eq!(
        while_mounting.reads_inside_the_journal_ring,
        2 * counting.journal_ring_full_scan_reads_per_device(),
        "挂载那一步全环扫一遍、两块盘各一遍：这就是 196 608 那一档，计数器数得出它"
    );

    counting.reset_tally();
    let file = mounted
        .mounted
        .open_file(&counting, InodeNumber(FIRST_INODE_NUMBER))
        .expect("打开文件");
    // extent 树按需读（D8（核心索引结构） 已定项 14，K4）：打开文件那一步按位置走到 inode 1 的上段叶条目，一个单元的文件内联在里面，
    // 上段只有根兼叶 ⇒ 读一个节点。
    assert_eq!(
        (
            counting.tally().reads,
            file.extent_tree_reads_at_open().node_reads
        ),
        (1, 1),
        "打开文件读 extent 树上段根兼叶那一个节点，块层数到的与实现自报的相等"
    );
    counting.reset_tally();
    let output = file
        .read_at(
            &counting,
            FileOffsetInBytes(0),
            u64::try_from(FILE_BYTES).expect("3000"),
        )
        .expect("挂载态读");

    assert_eq!(output.bytes, cold_content, "挂载态读与冷走读逐字节相同");
    assert_eq!(
        output.observation,
        ReadPathObservation {
            data_units_read: 1,
            data_unit_dereferences: 1,
            stale_location_hint_hops: 0,
            device_reads_issued: 1,
        },
        "3000 字节住一个单元：一次解引用、一次块层读、不多跳"
    );
    let while_reading = counting.tally();
    assert_eq!(
        while_reading.reads_inside_the_journal_ring, 0,
        "读期间 journal 环一次都不扫"
    );
    assert_eq!(
        while_reading.reads,
        1,
        "块层读计数 = 这一次读的单元数，不是 196 608 那一档（整扫一遍是 {} 次 / 盘）",
        counting.journal_ring_full_scan_reads_per_device()
    );
    assert_eq!(
        while_reading.reads, output.observation.device_reads_issued,
        "实现自报的设备级读数与块层数到的要相等（D17（实现分层与第三方管道） 已定项 5 射程 ①）"
    );
    assert_eq!(while_reading.reads_of_a_whole_unit, 1, "读的是整单元 32768");
}

/// 验收第 1 条与第 4 条：随机读 64 次 4 KiB，每次逐字节相同；期间 journal 环一次都不扫；
/// 读路径报得出这次读了几个单元与多跳次数。**这 64 次里跨单元的次数要大于 0**——为 0 就是取样点选窄了
/// （`.claude/rules/mutation-sampling.md` 第四类）。
///
/// 不并行：这 64 次读共用一份挂载态与一个计数器，被测的正是那组计数（`implementation-workflow.md`
/// 「测试与崩溃检测优先多线程」的「不能并行的写明为什么」）。
#[test]
fn random_four_kibibyte_reads_return_the_written_bytes_and_never_scan_the_journal_ring() {
    let built = build_synthetic_multi_unit_image(SyntheticImagePlan::of_four_units());
    let counting = ReadCountingPoolReader::new(&built.image, journal_ring());
    let mounted = open_pool_for_read(&counting, &built.root).expect("打开挂载态");
    counting.reset_tally();
    let file = mounted
        .open_file(&counting, InodeNumber(FIRST_INODE_NUMBER))
        .expect("打开文件");
    // 四个单元的文件有下段（D8（核心索引结构） 已定项 14）：打开时读上段根兼叶、再读下段那片叶，两个节点。
    assert_eq!(
        (
            counting.tally().reads,
            file.extent_tree_reads_at_open().node_reads
        ),
        (2, 2),
        "打开文件按需读 extent 树：上段根兼叶一个、下段叶一个"
    );
    counting.reset_tally();

    let mut offsets = PseudoRandomOffsets::seeded(0x9e37_79b9_7f4a_7c15);
    let capacity = payload_capacity();
    let highest_offset = built.file_size_in_bytes() - PAGE_BYTES;
    let mut pages_crossing_a_unit_boundary = 0u64;
    let mut expected_reads = 0u64;
    for _read in 0..RANDOM_READS {
        let offset = FileOffsetInBytes(offsets.next_below(highest_offset + 1));
        let output = file
            .read_at(&counting, offset, PAGE_BYTES)
            .expect("随机读一页");
        let start = usize::try_from(offset.0).expect("偏移");
        let end = usize::try_from(offset.0 + PAGE_BYTES).expect("偏移");
        assert_eq!(
            output.bytes,
            built.content[start..end],
            "偏移 {} 起 4 KiB 与写入逐字节相同",
            offset.0
        );
        // 这次读该碰几个单元由装置自己除一遍（D4（校验和位置） 已定项 5），再拿实现那个
        // `data_unit_span_covering` 对一次：两份算术分开写，口径打架才看得出来
        // （`.claude/rules/mutation-sampling.md` 第五类）。
        let first_unit_of_this_read = offset.0 / capacity;
        let last_unit_of_this_read = (offset.0 + PAGE_BYTES - 1) / capacity;
        let units_this_read = last_unit_of_this_read - first_unit_of_this_read + 1;
        let span = data_unit_span_covering(offset, FileOffsetInBytes(offset.0 + PAGE_BYTES - 1));
        assert_eq!(
            (span.first.0, span.last_inclusive.0),
            (first_unit_of_this_read, last_unit_of_this_read),
            "实现算出来的单元区间要与装置这一侧的除法相同"
        );
        assert_eq!(
            output.observation.data_units_read, units_this_read,
            "读路径报出的单元数要等于偏移除净荷容量算出来的"
        );
        assert_eq!(
            output.observation.stale_location_hint_hops, 0,
            "提示都没过期"
        );
        if span.spans_more_than_one_data_unit() {
            pages_crossing_a_unit_boundary += 1;
        }
        expected_reads += units_this_read;
    }

    assert!(
        pages_crossing_a_unit_boundary > 0,
        "这 {RANDOM_READS} 次随机读里一次跨单元的都没有：取样点选窄了（净荷 32634 ⇒ 约 12.5% 的页跨两个单元）"
    );
    assert_eq!(
        expected_reads,
        RANDOM_READS + pages_crossing_a_unit_boundary,
        "这 {RANDOM_READS} 次里跨单元 {pages_crossing_a_unit_boundary} 次（理论上约 12.5%，也就是约 8 次），\
         每次一个单元、跨单元的各多一个 ⇒ 一次读最多碰两个单元"
    );
    let tally = counting.tally();
    assert_eq!(
        tally.reads_inside_the_journal_ring,
        0,
        "K 次读期间 journal 环一次都不扫（整扫一遍是 {} 次 / 盘）",
        counting.journal_ring_full_scan_reads_per_device()
    );
    assert_eq!(
        tally.reads, expected_reads,
        "块层读计数 = 这 {RANDOM_READS} 次各自的单元数之和，跨单元的那几次各算两个；其中跨单元 {pages_crossing_a_unit_boundary} 次"
    );
    assert_eq!(
        mounted.observation_since_open().device_reads_issued,
        tally.reads,
        "实现自报的设备级读数与块层数到的要相等"
    );
    assert_eq!(
        mounted
            .observation_since_open()
            .stale_location_hint_hops_per_million_dereferences(),
        0,
        "提示一次都没过期 ⇒ 多跳率 0"
    );
}

/// 取样点覆盖到跨单元的页：把文件里每一页对齐的读都跑一遍，逐字节比，并数出跨单元的那几页。
/// 四个单元的文件里边界落在第 7、15、23 页（32634 / 65268 / 97902 各自所在的页）。
#[test]
fn every_aligned_page_reads_back_and_the_pages_that_cross_a_unit_boundary_read_two_units() {
    let built = build_synthetic_multi_unit_image(SyntheticImagePlan::of_four_units());
    let counting = ReadCountingPoolReader::new(&built.image, journal_ring());
    let mounted = open_pool_for_read(&counting, &built.root).expect("打开挂载态");
    let file = mounted
        .open_file(&counting, InodeNumber(FIRST_INODE_NUMBER))
        .expect("打开文件");
    counting.reset_tally();

    let whole_pages = built.file_size_in_bytes() / PAGE_BYTES;
    let mut pages_crossing_a_unit_boundary = Vec::new();
    for page in 0..whole_pages {
        let offset = FileOffsetInBytes(page * PAGE_BYTES);
        let output = file.read_at(&counting, offset, PAGE_BYTES).expect("读一页");
        let start = usize::try_from(offset.0).expect("偏移");
        assert_eq!(
            output.bytes,
            built.content[start..start + usize::try_from(PAGE_BYTES).expect("4096")],
            "第 {page} 页逐字节相同"
        );
        if output.observation.data_units_read == 2 {
            pages_crossing_a_unit_boundary.push(page);
        }
    }

    assert_eq!(
        pages_crossing_a_unit_boundary,
        vec![7, 15, 23],
        "三个单元边界（32634 / 65268 / 97902）各落在一页里，那一页要读两个单元"
    );
    assert_eq!(
        counting.tally().reads,
        whole_pages + 3,
        "每页一次单元读，跨单元的三页各多一次"
    );
    assert_eq!(counting.tally().reads_inside_the_journal_ring, 0);
}

/// 验收第 2 条：改坏一个数据单元的一个字节 ⇒ 落在它上面的读报校验和错，**别的读不受影响**。
/// 两块盘的同一个字节都翻（一条位置条目都不剩下对得上的），中央映射指的也是这两个落点 ⇒ 多跳一次之后仍是校验和错。
#[test]
fn corrupting_one_byte_in_a_data_unit_makes_reads_over_it_report_a_checksum_error_and_leaves_the_other_reads_alone(
) {
    let mut built = build_synthetic_multi_unit_image(SyntheticImagePlan::of_four_units());
    // 挂载态不缓存任何数据单元的载荷（D21（权威态与派生态的分界）：派生态，丢了从盘上重走），
    // 所以改坏盘面在挂载之前还是之后，读路径看到的是同一件事。
    built.flip_one_byte_in_every_copy_of_data_unit(DataUnitIndexInFile(2));
    let counting = ReadCountingPoolReader::new(&built.image, journal_ring());
    let mounted = open_pool_for_read(&counting, &built.root).expect("打开挂载态");
    let file = mounted
        .open_file(&counting, InodeNumber(FIRST_INODE_NUMBER))
        .expect("打开文件");

    // 第 16 页整个落在单元 2 里（65536 起，单元 2 是 [65268, 97902)）。
    let failure = file
        .read_at(&counting, FileOffsetInBytes(16 * PAGE_BYTES), PAGE_BYTES)
        .expect_err("落在改坏的那个单元上的读要报错");
    let FileReadFailure::DataUnitChecksumMismatchEverywhere {
        unit_index_in_file,
        hinted_slot: _,
    } = failure
    else {
        panic!("该报校验和错，实际 {failure:?}");
    };
    assert_eq!(unit_index_in_file, DataUnitIndexInFile(2));

    // 跨单元 1|2 的那一页也报错：它碰到了坏的那个单元。
    let across = file
        .read_at(&counting, FileOffsetInBytes(15 * PAGE_BYTES), PAGE_BYTES)
        .expect_err("跨到坏单元上的那一页也要报错");
    assert!(
        matches!(
            across,
            FileReadFailure::DataUnitChecksumMismatchEverywhere {
                unit_index_in_file: DataUnitIndexInFile(2),
                ..
            }
        ),
        "跨单元的那一页报的仍是单元 2 校验和错，实际 {across:?}"
    );

    // 别的读不受影响：取样点覆盖单元 0、1、3，外加**跨单元的那一档**——页 7 跨单元 0|1，一个字节都不碰单元 2。
    // 为什么是这几页：文件 102902 字节 ⇒ 最后一张对齐的整页是页 24（98304..102400），页 25 起点就越过末尾、不是合法取样点；
    // 跨单元的页共三张（7、15、23），15 与 23 都碰单元 2（上面两条已经钉住它们报错），能留给这一格的只有页 7。
    let file_size = built.file_size_in_bytes();
    let capacity = payload_capacity();
    let mut pages_crossing_a_unit_boundary = 0u64;
    for page in [0u64, 4, 7, 8, 14, 24] {
        let offset = FileOffsetInBytes(page * PAGE_BYTES);
        assert!(
            offset.0 + PAGE_BYTES <= file_size,
            "第 {page} 页越过文件末尾（{file_size} 字节）：取样点选错了，不是实现的错"
        );
        let output = file
            .read_at(&counting, offset, PAGE_BYTES)
            .expect("没碰到坏单元的读照常读回来");
        let start = usize::try_from(offset.0).expect("偏移");
        assert_eq!(
            output.bytes,
            built.content[start..start + usize::try_from(PAGE_BYTES).expect("4096")],
            "第 {page} 页不碰单元 2，逐字节相同"
        );
        assert_eq!(
            output.observation.stale_location_hint_hops, 0,
            "第 {page} 页的提示没动过，不该多跳"
        );
        // 这一页该读几个单元在装置这一侧自己除一遍（D4（校验和位置） 已定项 5：文件偏移到单元做除法），
        // 不调实现那个 `data_unit_span_covering`：两份算术分开写，口径打架才看得出来
        // （`.claude/rules/mutation-sampling.md` 第五类）。
        let units_this_page = (offset.0 + PAGE_BYTES - 1) / capacity - offset.0 / capacity + 1;
        assert_eq!(
            output.observation.data_units_read, units_this_page,
            "第 {page} 页读了几个单元"
        );
        if units_this_page > 1 {
            pages_crossing_a_unit_boundary += 1;
        }
    }
    assert_eq!(
        pages_crossing_a_unit_boundary, 1,
        "取样点里要有跨单元的那一档（页 7 跨单元 0|1）；为 0 就是只测了整页落在一个单元里的情形"
    );
}

/// 变异「关掉校验和比对」的正向用例：把一个单元换成另一份**自己的校验和全对**的单元
/// （头校验和与载荷 CRC 都重算过），位置条目里的整单元校验和还是旧的——
/// 这时只有位置条目那一道拦得住它。比对一关，那次读就会返回坏数据。
#[test]
fn a_data_unit_resealed_with_only_its_own_checksums_is_caught_by_the_location_entry_checksum() {
    let mut built = build_synthetic_multi_unit_image(SyntheticImagePlan::of_four_units());
    let other_payload = pseudo_random_bytes(
        usize::try_from(payload_capacity()).expect("32634"),
        0xdead_beef,
    );
    built.reseal_data_unit_with_other_content(DataUnitIndexInFile(1), &other_payload);
    let counting = ReadCountingPoolReader::new(&built.image, journal_ring());
    let mounted = open_pool_for_read(&counting, &built.root).expect("打开挂载态");
    let file = mounted
        .open_file(&counting, InodeNumber(FIRST_INODE_NUMBER))
        .expect("打开文件");

    // 第 9 页整个落在单元 1 里（36864 起，单元 1 是 [32634, 65268)）。
    let offset = FileOffsetInBytes(9 * PAGE_BYTES);
    let failure = file
        .read_at(&counting, offset, PAGE_BYTES)
        .expect_err("重封过的单元要被位置条目里的整单元校验和拦住");
    assert!(
        matches!(
            failure,
            FileReadFailure::DataUnitChecksumMismatchEverywhere {
                unit_index_in_file: DataUnitIndexInFile(1),
                ..
            }
        ),
        "该报校验和错，实际 {failure:?}"
    );

    // 读回来的绝不能是那份重封的内容：比对一关，下面这条就会变成「读到了 other_payload」。
    let start = usize::try_from(offset.0).expect("偏移");
    assert_ne!(
        file.read_at(&counting, offset, PAGE_BYTES)
            .map(|output| output.bytes)
            .unwrap_or_default(),
        built.content[start..start + usize::try_from(PAGE_BYTES).expect("4096")].to_vec(),
        "盘上已经不是写入的那份字节了，读不回原内容才对"
    );
}

/// 变异「把位置提示故意指到别的槽」的正向用例：提示指到一个从没写过的空槽 ⇒ 经中央映射仍读对，多跳计数加一
/// （D19（块指针的结构与宽度预算） 已定项 5：位置条目降为提示、中央映射是唯一入口；硬规则 3 的观测点）。
#[test]
fn a_location_hint_pointing_at_another_slot_still_reads_through_the_central_mapping_and_counts_one_extra_hop(
) {
    let built = build_synthetic_multi_unit_image(SyntheticImagePlan {
        damage: LocationHintDamage::PointTheHintOfThisUnitAtAnEmptySlot(DataUnitIndexInFile(1)),
        ..SyntheticImagePlan::of_four_units()
    });
    let counting = ReadCountingPoolReader::new(&built.image, journal_ring());
    let mounted = open_pool_for_read(&counting, &built.root).expect("打开挂载态");
    let file = mounted
        .open_file(&counting, InodeNumber(FIRST_INODE_NUMBER))
        .expect("打开文件");
    counting.reset_tally();

    // 第 9 页整个落在单元 1 里（36864 起，单元 1 是 [32634, 65268)）。
    let offset = FileOffsetInBytes(9 * PAGE_BYTES);
    let device_reads_before_the_stale_hint_read = counting.tally().reads;
    let output = file
        .read_at(&counting, offset, PAGE_BYTES)
        .expect("提示指错，经映射仍读得到");
    let device_reads_of_the_stale_hint_read =
        counting.tally().reads - device_reads_before_the_stale_hint_read;
    let start = usize::try_from(offset.0).expect("偏移");
    assert_eq!(
        output.bytes,
        built.content[start..start + usize::try_from(PAGE_BYTES).expect("4096")],
        "经映射读回来的字节与写入相同"
    );
    assert_eq!(
        output.observation,
        ReadPathObservation {
            data_units_read: 1,
            data_unit_dereferences: 1,
            stale_location_hint_hops: 1,
            device_reads_issued: 3,
        },
        "多跳一次：两条提示各试一次（读回全 0、校验和不对），再按映射的两条试，第一条就中 ⇒ 2 + 1 = 3。\
         查中央映射那一步**一次设备读都不发**：整片映射在 open_pool_for_read 里就读进了挂载态（派生态，D21（权威态与派生态的分界）），\
         read_at 里只是在内存里找一条 key。映射长到一个节点装不下、要按需走树的那天这个数会涨——\
         那一维正是 D19（块指针的结构与宽度预算） 已定项 5 自陈零测量的缓存命中维（C418（位置权威三臂的缓存命中维零测量））。"
    );
    assert_eq!(
        device_reads_of_the_stale_hint_read, output.observation.device_reads_issued,
        "实现自报的设备级读数与装置在块层数到的要相等（D17（实现分层与第三方管道） 已定项 5 射程 ①）"
    );

    // 提示没指错的单元照旧不多跳，而且只发一次设备读：两者相减正好是提示过期多付的那两次白试，
    // 「查映射本身不发设备读」这句话在这里被两个独立的计数器同时钉住。
    let device_reads_before_the_good_hint_read = counting.tally().reads;
    let untouched = file
        .read_at(&counting, FileOffsetInBytes(0), PAGE_BYTES)
        .expect("单元 0 的提示没动");
    let device_reads_of_the_good_hint_read =
        counting.tally().reads - device_reads_before_the_good_hint_read;
    assert_eq!(untouched.observation.stale_location_hint_hops, 0);
    assert_eq!(
        device_reads_of_the_good_hint_read, 1,
        "提示没过期 ⇒ 第一条位置条目就中，一次设备读"
    );
    assert_eq!(
        device_reads_of_the_stale_hint_read - device_reads_of_the_good_hint_read,
        2,
        "提示过期多付的正是两条提示各白试一次；查中央映射那一步一次设备读都不发"
    );
    assert_eq!(
        mounted.observation_since_open().stale_location_hint_hops,
        1,
        "整份挂载态累计的多跳次数"
    );
    assert_eq!(
        mounted
            .observation_since_open()
            .stale_location_hint_hops_per_million_dereferences(),
        500_000,
        "两次解引用里一次多跳 ⇒ 多跳率百万分之五十万"
    );
}

/// extent 叶记录 key 第三段是文件字节偏移（D8（核心索引结构） 已定项 3）：按文件字节偏移写的镜像读回来的每一页与写入逐字节相同；
/// 同一份内容把 key 第三段（连同锚点偏移）写成单元序号时，挂载态打开这个文件当场拒绝，拒在下段叶里第 1 条记录（offset 1 除不尽净荷容量，
/// 按位置寻址认不出它是哪个单元；第 0 条两种读法都是 0、分不开），不按位次猜着往下读。C490（extent 叶 key 的 offset 段没定单位） 的读侧那一半。
#[test]
fn a_mount_state_open_refuses_an_extent_key_whose_offset_segment_is_the_unit_index_instead_of_the_file_byte_offset(
) {
    let by_byte_offset = build_synthetic_multi_unit_image(SyntheticImagePlan {
        reading: ExtentKeyThirdSegmentReading::FileByteOffset,
        ..SyntheticImagePlan::of_four_units()
    });
    let by_unit_index = build_synthetic_multi_unit_image(SyntheticImagePlan {
        reading: ExtentKeyThirdSegmentReading::DataUnitIndex,
        ..SyntheticImagePlan::of_four_units()
    });
    assert_eq!(
        by_byte_offset.content, by_unit_index.content,
        "两份镜像装的是同一份内容，差的只有 key 第三段与锚点偏移的读法"
    );

    let mounted =
        open_pool_for_read(&by_byte_offset.image, &by_byte_offset.root).expect("打开挂载态");
    let file = mounted
        .open_file(&by_byte_offset.image, InodeNumber(FIRST_INODE_NUMBER))
        .expect("key 第三段是文件字节偏移的那一份打得开");
    for page in 0..by_byte_offset.file_size_in_bytes() / PAGE_BYTES {
        let output = file
            .read_at(
                &by_byte_offset.image,
                FileOffsetInBytes(page * PAGE_BYTES),
                PAGE_BYTES,
            )
            .expect("读一页");
        let start = usize::try_from(page * PAGE_BYTES).expect("页起点");
        assert_eq!(
            output.bytes,
            by_byte_offset.content[start..start + usize::try_from(PAGE_BYTES).expect("页长")],
            "第 {page} 页读回写入的字节"
        );
    }

    let mounted_by_unit_index = open_pool_for_read(&by_unit_index.image, &by_unit_index.root)
        .expect("打开挂载态：树与映射都解得开，拒绝落在打开文件那一步");
    let refusal = mounted_by_unit_index
        .open_file(&by_unit_index.image, InodeNumber(FIRST_INODE_NUMBER))
        .err()
        .expect("key 第三段写成单元序号的那一份，打开文件当场拒绝");
    // 下段叶按位置寻址（D8（核心索引结构） 已定项 14）：读者从 key 的 offset 段除净荷容量认单元号，除不尽就是位置不对（I-1.1），
    // 第 1 条记录（offset 1）当场拒，走不到「第 i 条是不是单元 i」那一判。
    assert_eq!(
        refusal,
        OpenFileFailure::ExtentTreeWalk(RecoveryFailure::InvariantViolated {
            invariant: "I-1.1",
            detail: "extent 叶记录的 key 不是 (0, 这个文件, 单元号 × 净荷容量)",
        })
    );
}

/// 中央映射树长成两层（两片叶上面一个根，D8（核心索引结构） 已定项 11）：打开挂载态把整棵树读进内存
/// （D19（块指针的结构与宽度预算） 已定项 5「挂载态怎么读映射」），之后提示过期的一次解引用照旧是 2 + 1 = 3 次设备读——
/// 查映射一次设备读都不发，不随映射树有几层变。这是已定项 5 那句「这个读数是根兼叶时量的，多层落地时由用例重量」的重量：
/// 根兼叶那一档在上面 `a_location_hint_pointing_at_another_slot_...` 里钉着，这里是两层。
/// 判别力：挂载态只读映射树根、不往下读叶，映射里就只剩内部条目、查不到数据单元的 key ⇒ 这一读报 `CentralMappingMiss`。
#[test]
fn a_stale_hint_under_a_two_level_central_mapping_still_costs_three_device_reads_because_the_whole_tree_is_in_the_mount_state(
) {
    let built = build_synthetic_multi_unit_image(SyntheticImagePlan {
        damage: LocationHintDamage::PointTheHintOfThisUnitAtAnEmptySlot(DataUnitIndexInFile(1)),
        central_mapping_shape: SyntheticCentralMappingShape::TwoLeavesUnderARoot,
        ..SyntheticImagePlan::of_four_units()
    });
    let counting = ReadCountingPoolReader::new(&built.image, journal_ring());
    let mounted = open_pool_for_read(&counting, &built.root).expect("两层映射照常打开");
    assert_eq!(
        mounted.central_mapping_tree_reads_at_open(),
        CentralMappingTreeReadsAtOpen {
            node_reads: 3,
            height: 2,
        },
        "打开时整棵映射树读进挂载态：根与两片叶各一次"
    );
    let file = mounted
        .open_file(&counting, InodeNumber(FIRST_INODE_NUMBER))
        .expect("打开文件");
    counting.reset_tally();
    let offset = FileOffsetInBytes(9 * PAGE_BYTES);
    let output = file
        .read_at(&counting, offset, PAGE_BYTES)
        .expect("提示指错，经映射仍读得到");
    let start = usize::try_from(offset.0).expect("偏移");
    assert_eq!(
        output.bytes,
        built.content[start..start + usize::try_from(PAGE_BYTES).expect("4096")],
        "经映射读回来的字节与写入相同"
    );
    assert_eq!(
        output.observation,
        ReadPathObservation {
            data_units_read: 1,
            data_unit_dereferences: 1,
            stale_location_hint_hops: 1,
            device_reads_issued: 3,
        },
        "两层映射：两条提示各试一次，再按映射的一条就中 ⇒ 2 + 1 = 3，与根兼叶那一档同一个数"
    );
    assert_eq!(
        counting.tally().reads,
        3,
        "块层数到的也是 3：查映射没为映射树发任何读"
    );
}

/// 中央映射树的根自述层级 1、装的却是 55 字节的映射条目 ⇒ 打开挂载态当场拒绝：内部条目要 27 + 86 = 113 字节
/// （D8（核心索引结构） 已定项 11），条目宽窄于它就不按内部条目切，也不把它们当映射条目解。
/// 读完映射树根就停，树表、extent / inode 单元一个都不读。
#[test]
fn a_central_mapping_root_claiming_level_one_over_mapping_entries_is_refused_instead_of_being_read_as_entries(
) {
    let built = build_synthetic_multi_unit_image(SyntheticImagePlan {
        central_mapping_shape: SyntheticCentralMappingShape::RootClaimingLevelOneOverMappingEntries,
        ..SyntheticImagePlan::of_four_units()
    });
    let counting = ReadCountingPoolReader::new(&built.image, journal_ring());

    let failure = open_pool_for_read(&counting, &built.root)
        .err()
        .expect("层级 1 的根装着 55 字节的条目就要拒绝");
    assert_eq!(
        failure,
        OpenPoolForReadFailure::Walk(RecoveryFailure::EntryNarrowerThanItsFieldTable {
            what: "中央映射树内部条目",
            entry_bytes: 55,
            field_table_bytes: 113,
        }),
        "错误成员说清是内部条目窄于字段表（D8 已定项 11：记账 108、映射 113）"
    );
    assert_eq!(
        counting.tally().reads,
        1,
        "只读了映射树根那一个节点（第一条位置条目就中）"
    );
}

/// 中央映射树根自述的条目宽缩到 key 宽 27（panic 面普查 R2）⇒ 打开挂载态当场拒绝，不按字段表切到偏移 55。
/// 条目宽是索引节点头里的一个**盘上字段**，`parse_index_node` 只判了它 ≥ key 宽 27；节点自己仍然自洽
/// （条目数 × 27 = 声明长度、两道校验和都对得上），挡着它的只有读者这一判。
///
/// **这一判判的是「今天这个读者要几个字节」，不是「映射条目该有多宽」**：后者归 C307（映射树两种 key 宽怎么装进
/// 一棵定宽 key 的树），那一条还开着。
#[test]
fn a_central_mapping_root_whose_entry_width_is_narrower_than_the_field_table_is_refused_instead_of_slicing_past_the_entry(
) {
    let built = build_synthetic_multi_unit_image(SyntheticImagePlan {
        central_mapping_entry_width: u16::try_from(MAPPING_KEY_BYTES).expect("27"),
        ..SyntheticImagePlan::of_four_units()
    });
    let counting = ReadCountingPoolReader::new(&built.image, journal_ring());

    let failure = open_pool_for_read(&counting, &built.root)
        .err()
        .expect("条目宽窄于字段表就要拒绝");
    assert_eq!(
        failure,
        OpenPoolForReadFailure::RecordMalformed {
            what: "映射条目"
        },
        "错误成员要说清是哪种条目切不动"
    );
}

/// extent 记录条数与「文件大小按净荷容量算出来的单元数」对不上（文件有洞，或 key 第三段的单位与位次不一致）：
/// 位次定位对它不成立，而那两样都要那条没定的条款先定案 ⇒ 在打开文件这一步拒绝，不猜一个读法往下读。
#[test]
fn a_file_whose_extent_record_count_does_not_match_its_size_is_refused_instead_of_guessing() {
    let capacity = payload_capacity();
    let built = build_synthetic_multi_unit_image(SyntheticImagePlan {
        // 盘上四条 extent 记录，inode 记录却自称有六个单元那么大。
        declared_file_size_in_bytes: Some(capacity * 6),
        ..SyntheticImagePlan::of_four_units()
    });
    let counting = ReadCountingPoolReader::new(&built.image, journal_ring());
    let mounted = open_pool_for_read(&counting, &built.root).expect("打开挂载态");

    let failure = mounted
        .open_file(&counting, InodeNumber(FIRST_INODE_NUMBER))
        .err()
        .expect("条数对不上要拒绝");
    assert_eq!(
        failure,
        OpenFileFailure::ExtentRecordCountDoesNotMatchTheFileSize {
            inode: InodeNumber(FIRST_INODE_NUMBER),
            extent_records: 4,
            data_units_implied_by_the_file_size: 6,
            file_size_in_bytes: capacity * 6,
        },
        "错误成员要说清两边各是几"
    );
}

/// 越过文件末尾的读当场拒绝：第一版不做短读、不补零。
#[test]
fn a_read_that_runs_past_the_end_of_the_file_is_refused_before_any_unit_is_dereferenced() {
    let built = build_synthetic_multi_unit_image(SyntheticImagePlan::of_four_units());
    let counting = ReadCountingPoolReader::new(&built.image, journal_ring());
    let mounted = open_pool_for_read(&counting, &built.root).expect("打开挂载态");
    let file = mounted
        .open_file(&counting, InodeNumber(FIRST_INODE_NUMBER))
        .expect("打开文件");
    counting.reset_tally();

    let file_size = built.file_size_in_bytes();
    let failure = file
        .read_at(&counting, FileOffsetInBytes(file_size - 10), PAGE_BYTES)
        .expect_err("越过末尾要拒绝");
    assert_eq!(
        failure,
        FileReadFailure::RangeBeyondEndOfFile {
            file_size_in_bytes: file_size,
            offset: FileOffsetInBytes(file_size - 10),
            length_in_bytes: PAGE_BYTES,
        }
    );
    assert_eq!(counting.tally().reads, 0, "拒绝在任何解引用之前");

    // 正好收在末尾的读做得成；长度为 0 的读一个单元都不碰。
    let to_the_end = file
        .read_at(&counting, FileOffsetInBytes(file_size - 10), 10)
        .expect("正好收在末尾");
    assert_eq!(to_the_end.bytes.len(), 10);
    let empty = file
        .read_at(&counting, FileOffsetInBytes(file_size), 0)
        .expect("长度为 0");
    assert_eq!(empty.observation, ReadPathObservation::default());
    assert!(empty.bytes.is_empty());
}
