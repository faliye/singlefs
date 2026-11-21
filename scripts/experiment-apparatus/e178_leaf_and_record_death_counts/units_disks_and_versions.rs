//! 单元与它们的身份、记录与单元体、盘、叶、打包树、版本与每版写了什么。

use super::*;

// ---------------------------------------------------------------------------------------------
// 单元、记录与盘（登记 5.1）
// ---------------------------------------------------------------------------------------------

pub(super) type UnitIndex = usize;

/// 码 3 容器的身份：(出生树, 打包记录类型, 容器号, 容器出生代)（登记第一节读法 1 的容器 key）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct ContainerIdentity {
    pub(super) birth_tree: u64,
    pub(super) record_type: u64,
    pub(super) number: u64,
    pub(super) birth: u64,
}

impl ContainerIdentity {
    pub(super) const fn inode_leaf(number: u64, birth: u64) -> Self {
        Self { birth_tree: TREE_IDENTIFIER_INODE, record_type: PACKED_TYPE_INODE, number, birth }
    }
}

/// 写序 = (诞生 txg, 实例代号, 事务号)；同组择新取最大（登记 5.1）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct WriteOrder {
    pub(super) birth_txg: u64,
    pub(super) instance: u64,
    pub(super) transaction: u64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct InodeRecord {
    pub(super) inode: u64,
    pub(super) object_birth: u64,
    pub(super) nlink: u64,
    pub(super) size_bytes: u64,
    /// 记录偏移 88 的改动计数：最后一次改动所在发布的 txg（读法 11；只有 B-tsz 的 P-C 读它）。
    pub(super) change_counter: u64,
}

/// 改动计数不进 Debug：第一、二段产物里的停机对拍行按加这个字段之前的四个字段打，复跑要逐字节相同。
impl std::fmt::Debug for InodeRecord {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("InodeRecord")
            .field("inode", &self.inode)
            .field("object_birth", &self.object_birth)
            .field("nlink", &self.nlink)
            .field("size_bytes", &self.size_bytes)
            .finish()
    }
}

impl InodeRecord {
    /// ⌈size ÷ 32634⌉：读法 Z 拿它当这个对象的单元数。
    pub(super) const fn data_units(self) -> u64 {
        self.size_bytes.div_ceil(DATA_UNIT_PAYLOAD_BYTES)
    }
}

/// 墓碑记录 (对象, 对象出生代, 区间起, 区间止 = 2⁶⁴ − 1, 死亡 txg)；整对象的区间起是 0。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TombstoneRecord {
    pub(super) object: u64,
    pub(super) object_birth: u64,
    pub(super) range_start_bytes: u64,
    pub(super) death_txg: u64,
}

/// 退役记录 (四元组, 死亡 txg)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RetirementRecord {
    pub(super) identity: ContainerIdentity,
    pub(super) death_txg: u64,
}

#[derive(Clone, Debug)]
pub(super) enum UnitBody {
    /// 码 1 数据单元：类身份 (出生树, 对象 inode 号, 对象出生代, 单元序号 k)。
    Data { object: u64, object_birth: u64, unit_index_in_file: u64 },
    /// 码 3、打包记录类型 2：inode 叶容器。
    InodeLeaf { identity: ContainerIdentity, records: Arc<Vec<InodeRecord>> },
    /// 码 3、打包记录类型 1：墓碑容器，各墓碑写流各有一棵树。
    Tombstones { stream: TombstoneStream, identity: ContainerIdentity, records: Arc<Vec<TombstoneRecord>> },
    /// 码 3、打包记录类型 3：退役容器。
    Retirements { identity: ContainerIdentity, records: Arc<Vec<RetirementRecord>> },
}

/// 择新的组：码 1 按 (对象, 对象出生代, 单元序号)，码 3 按身份（墓碑另带写流）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum GroupKey {
    Data { object: u64, object_birth: u64, unit_index_in_file: u64 },
    InodeLeaf(ContainerIdentity),
    Tombstones(TombstoneStream, ContainerIdentity),
    Retirements(ContainerIdentity),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum GroupCategory {
    Data,
    InodeLeaf,
    Tombstones(TombstoneStream),
    Retirements,
}

pub(super) const fn category_of(key: GroupKey) -> GroupCategory {
    match key {
        GroupKey::Data { .. } => GroupCategory::Data,
        GroupKey::InodeLeaf(_) => GroupCategory::InodeLeaf,
        GroupKey::Tombstones(stream, _) => GroupCategory::Tombstones(stream),
        GroupKey::Retirements(_) => GroupCategory::Retirements,
    }
}

impl UnitBody {
    pub(super) const fn group_key(&self) -> GroupKey {
        match self {
            UnitBody::Data { object, object_birth, unit_index_in_file } => GroupKey::Data {
                object: *object,
                object_birth: *object_birth,
                unit_index_in_file: *unit_index_in_file,
            },
            UnitBody::InodeLeaf { identity, .. } => GroupKey::InodeLeaf(*identity),
            UnitBody::Tombstones { stream, identity, .. } => GroupKey::Tombstones(*stream, *identity),
            UnitBody::Retirements { identity, .. } => GroupKey::Retirements(*identity),
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct Unit {
    pub(super) write_order: WriteOrder,
    /// 写它的那次发布（含崩溃时没发布成的那一次）的发布序号。
    pub(super) written_at_publish: u64,
    /// 被换下的那次发布的序号；现行那一版还引用它时是 None（回退把 R_old 的单元重新算作被引用）。
    pub(super) replaced_at_publish: Option<u64>,
    pub(super) body: UnitBody,
}

/// 数据单元的 key (对象, 对象出生代, 单元序号)；别的单元是 None。
pub(super) fn data_key_of(unit: &Unit) -> Option<(u64, u64, u64)> {
    match &unit.body {
        UnitBody::Data { object, object_birth, unit_index_in_file } => Some((*object, *object_birth, *unit_index_in_file)),
        UnitBody::InodeLeaf { .. } | UnitBody::Tombstones { .. } | UnitBody::Retirements { .. } => None,
    }
}

/// 盘：写过的每个单元（只增），与按组的版本清单（按写入次序，也就是写序次序）。
#[derive(Clone, Debug, Default)]
pub(super) struct Disk {
    pub(super) units: Vec<Unit>,
    pub(super) group_of_key: HashMap<GroupKey, usize>,
    pub(super) versions_of_group: Vec<Vec<UnitIndex>>,
    pub(super) groups_by_category: BTreeMap<GroupCategory, Vec<usize>>,
}

impl Disk {
    pub(super) fn write(&mut self, unit: Unit) -> UnitIndex {
        let unit_index = self.units.len();
        let key = unit.body.group_key();
        let group = if let Some(existing) = self.group_of_key.get(&key) {
            *existing
        } else {
            let created = self.versions_of_group.len();
            self.group_of_key.insert(key, created);
            self.versions_of_group.push(Vec::new());
            self.groups_by_category.entry(category_of(key)).or_default().push(created);
            created
        };
        self.versions_of_group[group].push(unit_index);
        self.units.push(unit);
        unit_index
    }
}

// ---------------------------------------------------------------------------------------------
// 现行那一版（模型的真值；读者不读它，只拿它当计数时的 T）
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub(super) struct Leaf {
    pub(super) identity: ContainerIdentity,
    pub(super) separator: u64,
    pub(super) records: Arc<Vec<InodeRecord>>,
    pub(super) unit: UnitIndex,
}

#[derive(Clone, Debug)]
pub(super) struct FileUnits {
    pub(super) object_birth: u64,
    /// 按单元序号。
    pub(super) units: Vec<UnitIndex>,
}

#[derive(Clone, Debug)]
pub(super) struct PackedContainer<Record> {
    pub(super) identity: ContainerIdentity,
    pub(super) records: Arc<Vec<Record>>,
    pub(super) unit: UnitIndex,
}

/// 墓碑树、退役树：开放容器有新记录那次才重写、满了才关、容器号从 1 起（D18 已定项 24 第 1、3 条）。
#[derive(Clone, Debug)]
pub(super) struct PackedTree<Record> {
    pub(super) containers: Vec<PackedContainer<Record>>,
    pub(super) next_container_number: u64,
}

impl<Record> PackedTree<Record> {
    pub(super) const fn empty() -> Self {
        Self { containers: Vec::new(), next_container_number: 1 }
    }

    pub(super) fn record_count(&self) -> u64 {
        self.containers.iter().map(|container| length_as_u64(container.records.len())).sum()
    }
}

#[derive(Clone, Debug)]
pub(super) struct Version {
    pub(super) txg: u64,
    pub(super) instance: u64,
    pub(super) transaction: u64,
    pub(super) next_inode: u64,
    pub(super) leaves: Vec<Leaf>,
    pub(super) files: BTreeMap<u64, FileUnits>,
    pub(super) deleted_queue: Option<u64>,
    pub(super) tombstone_trees: Vec<PackedTree<TombstoneRecord>>,
    pub(super) retirement_tree: PackedTree<RetirementRecord>,
    /// 谱系记录 (T_rb, T_old)。
    pub(super) lineage: Vec<(u64, u64)>,
    /// 意图已发布的对象（O2 的「p 里它的意图已发布」）。
    pub(super) objects_with_published_intent: BTreeSet<u64>,
}

impl Version {
    pub(super) fn genesis() -> Self {
        Self {
            txg: NEW_POOL_FILE_CREATION_TXG - 1,
            instance: MAKE_FILESYSTEM_PROCESS_INSTANCE,
            transaction: 0,
            next_inode: FIRST_FILE_INODE,
            leaves: Vec::new(),
            files: BTreeMap::new(),
            deleted_queue: None,
            tombstone_trees: TOMBSTONE_STREAMS.iter().map(|_| PackedTree::empty()).collect(),
            retirement_tree: PackedTree::empty(),
            lineage: Vec::new(),
            objects_with_published_intent: BTreeSet::new(),
        }
    }

    pub(super) fn record_of(&self, inode: u64) -> Option<InodeRecord> {
        self.leaves.iter().find_map(|leaf| {
            leaf.records
                .binary_search_by_key(&inode, |record| record.inode)
                .ok()
                .map(|position| leaf.records[position])
        })
    }

    pub(super) fn is_live(&self, inode: u64) -> bool {
        self.record_of(inode).is_some_and(|record| record.nlink > 0)
    }

    /// 活 inode（nlink > 0），按叶序（= 号升序）。
    pub(super) fn live_inodes(&self) -> Vec<u64> {
        self.leaves
            .iter()
            .flat_map(|leaf| leaf.records.iter())
            .filter(|record| record.nlink > 0)
            .map(|record| record.inode)
            .collect()
    }

    pub(super) fn data_units_of(&self, inode: u64) -> u64 {
        self.files.get(&inode).map_or(0, |file| length_as_u64(file.units.len()))
    }

    /// 「有树、文件集合为空」（D8 已定项 16 第 5 条；E177 装置 `is_with_trees_and_no_file`）。B-stub 写路径的桩不算记录。
    pub(super) fn is_with_trees_and_no_file(&self) -> bool {
        self.leaves.iter().all(|leaf| leaf.records.is_empty()) && self.files.is_empty() && self.deleted_queue.is_none()
    }

    /// B-stub 写路径留在叶序里的零条目版本个数（原写路径恒 0）。
    pub(super) fn stub_count(&self) -> u64 {
        length_as_u64(self.leaves.iter().filter(|leaf| leaf.records.is_empty()).count())
    }

    /// 数据单元的摘要：每个对象的出生代与单元数（第四段 V-Bs1 比两条写路径的真值）。
    pub(super) fn extent_digest(&self) -> u64 {
        let mut digest = DIGEST_START;
        for (object, file) in &self.files {
            for value in [*object, file.object_birth, length_as_u64(file.units.len())] {
                digest = fold_into_digest(digest, value);
            }
        }
        digest
    }

    /// 这一版里有 nlink = 0 的记录：在某个对象的 unlink 途中（登记第一节读法 4）。
    pub(super) fn is_inside_an_unlink(&self) -> bool {
        self.leaves.iter().any(|leaf| leaf.records.iter().any(|record| record.nlink == 0))
    }

    /// 这一版引用的全部单元（升序、不重复）。
    pub(super) fn referenced_units(&self) -> Vec<UnitIndex> {
        let mut units: Vec<UnitIndex> = self.leaves.iter().map(|leaf| leaf.unit).collect();
        units.extend(self.files.values().flat_map(|file| file.units.iter().copied()));
        for tree in &self.tombstone_trees {
            units.extend(tree.containers.iter().map(|container| container.unit));
        }
        units.extend(self.retirement_tree.containers.iter().map(|container| container.unit));
        units.sort_unstable();
        units.dedup();
        units
    }

    pub(super) fn shape(&self) -> TreeShape {
        TreeShape {
            leaves: self
                .leaves
                .iter()
                .map(|leaf| ShapeLeaf {
                    container_number: leaf.identity.number,
                    record_keys: leaf.records.iter().map(|record| record.inode).collect(),
                })
                .collect(),
            separators: self.leaves.iter().map(|leaf| leaf.separator).collect(),
        }
    }

    /// 活叶容器的摘要：身份、分隔 key、每条记录（A5 比两条写流逐发布相同；第四段 V-Bs1 比 B-stub 与原写路径的活叶）。
    /// 桩（记录数 0）不进摘要，原写路径上没有桩，这一句对它不改任何值。
    pub(super) fn leaf_digest(&self) -> u64 {
        let mut digest = DIGEST_START;
        for leaf in self.leaves.iter().filter(|leaf| !leaf.records.is_empty()) {
            for value in [leaf.identity.birth_tree, leaf.identity.number, leaf.identity.birth, leaf.separator] {
                digest = fold_into_digest(digest, value);
            }
            for record in leaf.records.iter() {
                for value in [record.inode, record.object_birth, record.nlink, record.size_bytes] {
                    digest = fold_into_digest(digest, value);
                }
            }
        }
        digest
    }
}

// ---------------------------------------------------------------------------------------------
// 一次发布写哪些单元（登记 5.1「一次发布写哪些单元」、5.3 的臂、合并策略 M(θ)）
// ---------------------------------------------------------------------------------------------

/// 一次发布写的单元数，每份计（两盘镜像同内容）。码 2 只数 inode 树根、墓碑树根、退役树根。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct UnitsWritten {
    pub(super) data_units: u64,
    pub(super) inode_leaf_containers: u64,
    pub(super) inode_roots: u64,
    pub(super) retirement_containers: u64,
    pub(super) retirement_roots: u64,
    pub(super) retirement_records: u64,
    pub(super) tombstone_containers: [u64; TOMBSTONE_STREAM_COUNT],
    pub(super) tombstone_roots: [u64; TOMBSTONE_STREAM_COUNT],
}

impl UnitsWritten {
    /// 码 3：叶容器 + 退役容器 + 今天那条墓碑写流的墓碑容器（读法 8）。
    pub(super) const fn code_three_counted(&self) -> u64 {
        self.inode_leaf_containers
            + self.retirement_containers
            + self.tombstone_containers[TombstoneStream::WholeAtIntent.position()]
    }

    /// 码 2：inode 树根 + 退役树根 + 今天那条墓碑写流的墓碑树根。
    pub(super) const fn code_two_counted(&self) -> u64 {
        self.inode_roots + self.retirement_roots + self.tombstone_roots[TombstoneStream::WholeAtIntent.position()]
    }

    pub(super) const fn every_counted_unit(&self) -> u64 {
        self.data_units + self.code_three_counted() + self.code_two_counted()
    }
}

/// 一次删记录之后合并策略做了什么。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MergeAction {
    NoMerge,
    EmptiedAndDropped,
    AbsorbedTheRightNeighbour,
    BorrowedFromTheRightNeighbour { records_moved: u64 },
    AbsorbedIntoTheLeftNeighbour,
}

#[derive(Clone, Debug)]
pub(super) enum Operation {
    FirstFile,
    RowPublish,
    WarmUp,
    /// 覆盖写、顺序写：对象的内容整份换成 `data_units` 个新单元（登记第一节读法 10）。
    WriteFile { object: u64, data_units: u64 },
    NewInodes { count: u64 },
    UnlinkIntent { object: u64 },
    UnlinkBatch { object: u64, data_units_kept: u64 },
    UnlinkFinal { object: u64 },
    /// 管理员回退：一次向前发布，inode 树、文件、删除队列取 R_old 的，墓碑树、退役树、谱系树随现行那一版。
    RollBack { target: Box<Version> },
}

/// 变短：用户改动把对象的单元数改小（覆盖写、顺序写）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ShrinkEvent {
    pub(super) object: u64,
    pub(super) size_before_bytes: u64,
}

#[derive(Clone, Debug)]
pub(super) struct WorkingLeaf {
    pub(super) identity: ContainerIdentity,
    pub(super) separator: u64,
    pub(super) records: Arc<Vec<InodeRecord>>,
    /// 照抄上一版的单元；None 是这次重写。
    pub(super) carried_unit: Option<UnitIndex>,
}

/// 根装不下那么多片叶容器（`crates/` 的 `MoreLeafContainersThanOneRootNodeHolds`），那一格记「装不下」。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct LeafContainersOverflowTheRoot;

/// 容器身份消失（删空、被吸收、N2 换下）那次写什么：原写路径追加退役记录，B-stub 写路径原位落一个零条目版本（修订 2 之五）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ContainerDeathWritePath {
    RetirementRecords,
    ZeroRecordStubs,
}
