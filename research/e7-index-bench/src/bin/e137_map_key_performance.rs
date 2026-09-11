//! E137：映射 key 形态的性能差距——每次 fsync 写几块、每次读几次 I/O。
//!
//! 跑前登记 `research/prompts/e137-preregistration.md`（写于本文件之前，补记在第一次运行之前）。
//! E133 / E134 / E136 只报格式量，没碰 key 的顺序；这里用真的有序叶子结构跑 D25 / E16 的负载，
//! 数每次 fsync 触到几个叶、几个内部节点（D25 的公式：叶 + 各层互异节点 + 1 根槽 + 1 记录），
//! 以及有限缓存下每次读的节点未命中数。三条臂与两种映射范围与 E136 同名、同定义。
//! ⚠️ **只报数、不判输赢**：岔路由用户定。
//!
//! 确定性模型：同一个种子跑 N 遍逐字节一致；种子取 1 / 2 / 3 逐个报，它们是三份不同的负载样本，不是「重复测量」。
//! 证据强度来自手算小场景的钉值单测与变异测试。

use e7_index_bench::Emitter;
use std::collections::{BTreeMap, HashMap, HashSet};

const FILE_COUNT: u64 = 4096;
const UNITS_PER_FILE: u64 = 256;
const STREAM_COUNT: u64 = 16;
const IMAGE_CHECKPOINT_EVERY_WRITES: u64 = 64;
const MEASURED_WRITE_COUNT: u64 = 20_000;
const BACKGROUND_UNIT_COUNT: u64 = 20_000;
const BACKGROUND_CHECKPOINT_EVERY_UNITS: u64 = 256;
const SNAPSHOT_OVERWRITE_CHECKPOINT_EVERY_WRITES: u64 = 10;
const DELETED_FILE_COUNT: u64 = 256;
const READ_WARMUP_COUNT: u64 = 100_000;
const READ_MEASURED_COUNT: u64 = 100_000;
const STALE_HINT_PERMILLE: [u64; 2] = [190, 370];
const CACHE_PERMILLE_OF_LOGICAL_ARM: [u64; 3] = [100, 300, 600];
const SEEDS: [u64; 3] = [1, 2, 3];
const BATCHES: [u64; 2] = [1, 10];
const ROOT_SLOT_BLOCKS: u64 = 1;
const JOURNAL_RECORD_BLOCKS: u64 = 1;
const TREE_IDENTIFIER: u64 = 1;
const INSTANCE_IDENTIFIER: u64 = 1;
const CODE1_CLASS_TAG: u64 = 1;
const CODE2_CLASS_TAG: u64 = 2;
/// 量的负载里新建的对象号从这里起，比镜像里的都大。
const NEW_OBJECT_BASE: u64 = 1_000_000;
const PERMILLE: u64 = 1000;

/// 岔路上的三条候选（E136 的臂名）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Candidate {
    LogicalIdentityWriteOrder,
    MixedPath,
    ClassReuseTagged,
}

const CANDIDATES: [Candidate; 3] = [Candidate::LogicalIdentityWriteOrder, Candidate::MixedPath, Candidate::ClassReuseTagged];

fn candidate_name(candidate: Candidate) -> &'static str {
    match candidate {
        Candidate::LogicalIdentityWriteOrder => "k1_seq",
        Candidate::MixedPath => "k1_seq_class_reuse",
        Candidate::ClassReuseTagged => "class_reuse_tagged",
    }
}

/// 映射装哪几类单元（C280 那个开关）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MapScope {
    AllPointers,
    Code1Only,
}

const MAP_SCOPES: [MapScope; 2] = [MapScope::AllPointers, MapScope::Code1Only];

fn map_scope_name(map_scope: MapScope) -> &'static str {
    match map_scope {
        MapScope::AllPointers => "all",
        MapScope::Code1Only => "code1_only",
    }
}

/// 码 1 映射 key 的顺序：按文件（五元组打头）还是按出生时刻（出生树 + 出生 txg + 写序打头）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Code1KeyOrder {
    FileOrder,
    BirthOrder,
}

/// 全部指针进映射时，extent 树自己的节点（码 2）的映射 key 顺序。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Code2KeyOrder {
    NodeIdentityOrder,
    BirthOrder,
}

fn code1_key_order(candidate: Candidate) -> Code1KeyOrder {
    match candidate {
        Candidate::LogicalIdentityWriteOrder => Code1KeyOrder::FileOrder,
        Candidate::MixedPath => Code1KeyOrder::FileOrder,
        Candidate::ClassReuseTagged => Code1KeyOrder::BirthOrder,
    }
}

fn code2_key_order(candidate: Candidate) -> Code2KeyOrder {
    match candidate {
        Candidate::LogicalIdentityWriteOrder => Code2KeyOrder::NodeIdentityOrder,
        Candidate::MixedPath => Code2KeyOrder::BirthOrder,
        Candidate::ClassReuseTagged => Code2KeyOrder::BirthOrder,
    }
}

/// 叶容量与内部扇出（E136 的 64 / 76 口径，由跨装置单测钉到 E136 的留存产物）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Geometry {
    map_leaf_capacity: usize,
    map_internal_fanout: usize,
    extent_leaf_capacity: usize,
    extent_internal_fanout: usize,
}

fn geometry(candidate: Candidate, map_scope: MapScope) -> Geometry {
    match (candidate, map_scope) {
        (Candidate::LogicalIdentityWriteOrder, MapScope::AllPointers) => Geometry { map_leaf_capacity: 206, map_internal_fanout: 129, extent_leaf_capacity: 122, extent_internal_fanout: 164 },
        (Candidate::LogicalIdentityWriteOrder, MapScope::Code1Only) => Geometry { map_leaf_capacity: 206, map_internal_fanout: 129, extent_leaf_capacity: 122, extent_internal_fanout: 164 },
        (Candidate::MixedPath, MapScope::AllPointers) => Geometry { map_leaf_capacity: 206, map_internal_fanout: 121, extent_leaf_capacity: 122, extent_internal_fanout: 152 },
        (Candidate::MixedPath, MapScope::Code1Only) => Geometry { map_leaf_capacity: 206, map_internal_fanout: 129, extent_leaf_capacity: 122, extent_internal_fanout: 164 },
        (Candidate::ClassReuseTagged, MapScope::AllPointers) => Geometry { map_leaf_capacity: 296, map_internal_fanout: 148, extent_leaf_capacity: 149, extent_internal_fanout: 152 },
        (Candidate::ClassReuseTagged, MapScope::Code1Only) => Geometry { map_leaf_capacity: 296, map_internal_fanout: 159, extent_leaf_capacity: 149, extent_internal_fanout: 164 },
    }
}

type MapKey = [u64; 7];
type ExtentKey = [u64; 2];

/// 一个数据单元的一个版本。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UnitVersion {
    object: u64,
    offset: u64,
    object_birth_txg: u64,
    birth_txg: u64,
    write_order: u64,
}

fn code1_map_key(code1_order: Code1KeyOrder, unit_version: &UnitVersion) -> MapKey {
    match code1_order {
        Code1KeyOrder::FileOrder => [CODE1_CLASS_TAG, TREE_IDENTIFIER, unit_version.object, unit_version.object_birth_txg, unit_version.offset, unit_version.birth_txg, unit_version.write_order],
        Code1KeyOrder::BirthOrder => [CODE1_CLASS_TAG, TREE_IDENTIFIER, unit_version.birth_txg, INSTANCE_IDENTIFIER, unit_version.write_order, 0, 0],
    }
}

/// 有序叶子结构：叶是有序数组，满了分裂（最右叶末尾追加另起新叶，其余对半分），删空不合并。叶有稳定的编号。
#[derive(Clone)]
struct OrderedLeaves<Key: Ord + Copy> {
    leaf_capacity: usize,
    leaves: Vec<Vec<Key>>,
    first_keys: Vec<Key>,
    leaf_identifiers: Vec<u64>,
    position_by_leaf_identifier: HashMap<u64, usize>,
    next_leaf_identifier: u64,
}

impl<Key: Ord + Copy> OrderedLeaves<Key> {
    fn new(leaf_capacity: usize) -> Self {
        assert!(leaf_capacity >= 2, "叶容量至少 2");
        Self { leaf_capacity, leaves: Vec::new(), first_keys: Vec::new(), leaf_identifiers: Vec::new(), position_by_leaf_identifier: HashMap::new(), next_leaf_identifier: 0 }
    }

    fn leaf_count(&self) -> usize {
        self.leaves.len()
    }

    /// key 该落在哪个叶（按位置）。
    fn position_for(&self, key: &Key) -> usize {
        self.first_keys.partition_point(|first_key| first_key <= key).saturating_sub(1)
    }

    fn position_of_leaf(&self, leaf_identifier: u64) -> usize {
        *self.position_by_leaf_identifier.get(&leaf_identifier).expect("叶编号不在位置表里：分裂之后位置表没跟上")
    }

    fn first_key_at(&self, position: usize) -> Key {
        self.first_keys[position]
    }

    fn allocate_leaf_identifier(&mut self) -> u64 {
        let leaf_identifier = self.next_leaf_identifier;
        self.next_leaf_identifier += 1;
        leaf_identifier
    }

    /// 插入一个新 key，返回这一步弄脏的叶的编号。
    fn insert(&mut self, key: Key) -> Vec<u64> {
        if self.leaves.is_empty() {
            let leaf_identifier = self.allocate_leaf_identifier();
            self.leaves.push(vec![key]);
            self.first_keys.push(key);
            self.leaf_identifiers.push(leaf_identifier);
            self.position_by_leaf_identifier.insert(leaf_identifier, 0);
            return vec![leaf_identifier];
        }
        let position = self.position_for(&key);
        let is_rightmost_leaf = position + 1 == self.leaves.len();
        let insertion_index = match self.leaves[position].binary_search(&key) {
            Ok(_) => panic!("重复的 key：同一个版本身份插了两次"),
            Err(index) => index,
        };
        self.leaves[position].insert(insertion_index, key);
        if key < self.first_keys[position] {
            self.first_keys[position] = key;
        }
        let leaf_identifier = self.leaf_identifiers[position];
        let leaf_length = self.leaves[position].len();
        if leaf_length <= self.leaf_capacity {
            return vec![leaf_identifier];
        }
        let appended_at_right_edge = is_rightmost_leaf && insertion_index + 1 == leaf_length;
        let split_index = if appended_at_right_edge { leaf_length - 1 } else { leaf_length / 2 };
        let moved_keys = self.leaves[position].split_off(split_index);
        let new_first_key = moved_keys[0];
        let new_leaf_identifier = self.allocate_leaf_identifier();
        self.leaves.insert(position + 1, moved_keys);
        self.first_keys.insert(position + 1, new_first_key);
        self.leaf_identifiers.insert(position + 1, new_leaf_identifier);
        for (distance, shifted_leaf_identifier) in self.leaf_identifiers[position + 1..].iter().enumerate() {
            self.position_by_leaf_identifier.insert(*shifted_leaf_identifier, position + 1 + distance);
        }
        if appended_at_right_edge {
            vec![new_leaf_identifier]
        } else {
            vec![leaf_identifier, new_leaf_identifier]
        }
    }

    /// 删掉一个已有的 key，返回弄脏的叶。
    fn remove(&mut self, key: &Key) -> u64 {
        let position = self.position_for(key);
        let index = self.leaves[position].binary_search(key).expect("要删的 key 不在树里");
        self.leaves[position].remove(index);
        self.leaf_identifiers[position]
    }

    /// 改一个已有 key 的 value（key 不变），返回弄脏的叶。
    fn touch(&self, key: &Key) -> u64 {
        let position = self.position_for(key);
        assert!(self.leaves[position].binary_search(key).is_ok(), "要改的 key 不在树里");
        self.leaf_identifiers[position]
    }
}

/// 被触到的叶（按位置）在各内部层触到几个互异节点，一直数到根（含根）。只有一个叶时它自己就是根，内部层为 0。
fn touched_internal_node_count(touched_positions: &[usize], leaf_count: usize, internal_fanout: usize) -> u64 {
    if touched_positions.is_empty() || leaf_count <= 1 {
        return 0;
    }
    let mut internal_node_count = 0u64;
    let mut group_size = internal_fanout;
    loop {
        let distinct_groups: HashSet<usize> = touched_positions.iter().map(|position| position / group_size).collect();
        internal_node_count += u64::try_from(distinct_groups.len()).expect("互异节点数装得进 u64");
        if leaf_count.div_ceil(group_size) <= 1 {
            break;
        }
        group_size = group_size.saturating_mul(internal_fanout);
    }
    internal_node_count
}

/// 一棵树全部节点数（叶 + 各内部层，含根）。
fn total_node_count(leaf_count: usize, internal_fanout: usize) -> u64 {
    let all_positions: Vec<usize> = (0..leaf_count).collect();
    u64::try_from(leaf_count).expect("叶数装得进 u64") + touched_internal_node_count(&all_positions, leaf_count, internal_fanout)
}

/// 一次 fsync（一次发布）写了几块。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CheckpointBlocks {
    extent_leaves: u64,
    extent_internal: u64,
    map_leaves: u64,
    map_internal: u64,
}

impl CheckpointBlocks {
    fn zero() -> Self {
        Self { extent_leaves: 0, extent_internal: 0, map_leaves: 0, map_internal: 0 }
    }
    fn total_with_root_and_record(&self) -> u64 {
        self.extent_leaves + self.extent_internal + self.map_leaves + self.map_internal + ROOT_SLOT_BLOCKS + JOURNAL_RECORD_BLOCKS
    }
    fn add(&mut self, other: &CheckpointBlocks) {
        self.extent_leaves += other.extent_leaves;
        self.extent_internal += other.extent_internal;
        self.map_leaves += other.map_leaves;
        self.map_internal += other.map_internal;
    }
}

/// extent 树的一个节点身份（全部指针进映射时它的映射条目按这个身份删旧插新）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum ExtentNodeIdentity {
    Leaf(u64),
    Internal { level: u32, group: usize },
}

/// 镜像的形状（量的负载用登记的那一份；单测用小的）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ImageShape {
    file_count: u64,
    units_per_file: u64,
    stream_count: u64,
    checkpoint_every_writes: u64,
}

const REGISTERED_IMAGE: ImageShape = ImageShape { file_count: FILE_COUNT, units_per_file: UNITS_PER_FILE, stream_count: STREAM_COUNT, checkpoint_every_writes: IMAGE_CHECKPOINT_EVERY_WRITES };

#[derive(Clone)]
struct Image {
    map_scope: MapScope,
    code1_order: Code1KeyOrder,
    code2_order: Code2KeyOrder,
    geometry: Geometry,
    extent: OrderedLeaves<ExtentKey>,
    map: OrderedLeaves<MapKey>,
    live_units: HashMap<(u64, u64), UnitVersion>,
    object_birth_txg: HashMap<u64, u64>,
    extent_node_map_keys: HashMap<ExtentNodeIdentity, MapKey>,
    touched_extent_leaves: HashSet<u64>,
    touched_map_leaves: HashSet<u64>,
    current_txg: u64,
    next_write_order: u64,
    next_birth_sequence: u64,
    allocation_order: Vec<(u64, u64)>,
}

impl Image {
    fn empty(map_scope: MapScope, code1_order: Code1KeyOrder, code2_order: Code2KeyOrder, geometry: Geometry) -> Self {
        Self {
            map_scope,
            code1_order,
            code2_order,
            geometry,
            extent: OrderedLeaves::new(geometry.extent_leaf_capacity),
            map: OrderedLeaves::new(geometry.map_leaf_capacity),
            live_units: HashMap::new(),
            object_birth_txg: HashMap::new(),
            extent_node_map_keys: HashMap::new(),
            touched_extent_leaves: HashSet::new(),
            touched_map_leaves: HashSet::new(),
            current_txg: 1,
            next_write_order: 0,
            next_birth_sequence: 0,
            allocation_order: Vec::new(),
        }
    }

    fn new_version(&mut self, object: u64, offset: u64) -> UnitVersion {
        let current_txg = self.current_txg;
        let object_birth_txg = *self.object_birth_txg.entry(object).or_insert(current_txg);
        let write_order = self.next_write_order;
        self.next_write_order += 1;
        UnitVersion { object, offset, object_birth_txg, birth_txg: current_txg, write_order }
    }

    /// 写一个单元的新版本；旧版的映射条目不删，交给调用方决定（放掉还是被快照留着）。
    fn write_unit_keeping_old(&mut self, object: u64, offset: u64) -> Option<UnitVersion> {
        let new_version = self.new_version(object, offset);
        let old_version = self.live_units.insert((object, offset), new_version);
        match old_version {
            Some(_) => {
                let touched_leaf = self.extent.touch(&[object, offset]);
                self.touched_extent_leaves.insert(touched_leaf);
            }
            None => {
                for touched_leaf in self.extent.insert([object, offset]) {
                    self.touched_extent_leaves.insert(touched_leaf);
                }
            }
        }
        for touched_leaf in self.map.insert(code1_map_key(self.code1_order, &new_version)) {
            self.touched_map_leaves.insert(touched_leaf);
        }
        self.allocation_order.push((object, offset));
        old_version
    }

    /// 写一个单元的新版本，旧版随即放掉（没有快照留着它）。
    fn write_unit(&mut self, object: u64, offset: u64) {
        if let Some(old_version) = self.write_unit_keeping_old(object, offset) {
            self.free_version(&old_version);
        }
    }

    fn free_version(&mut self, version: &UnitVersion) {
        let touched_leaf = self.map.remove(&code1_map_key(self.code1_order, version));
        self.touched_map_leaves.insert(touched_leaf);
    }

    fn delete_unit(&mut self, object: u64, offset: u64) {
        let version = self.live_units.remove(&(object, offset)).expect("要删的单元不在");
        let touched_extent_leaf = self.extent.remove(&[object, offset]);
        self.touched_extent_leaves.insert(touched_extent_leaf);
        self.free_version(&version);
    }

    /// 搬迁：映射条目的 value（落点）变，key 不变。
    fn relocate_unit(&mut self, object: u64, offset: u64) {
        let version = *self.live_units.get(&(object, offset)).expect("要搬的单元不在");
        let touched_leaf = self.map.touch(&code1_map_key(self.code1_order, &version));
        self.touched_map_leaves.insert(touched_leaf);
    }

    fn code2_map_key(&mut self, identity: ExtentNodeIdentity, extent_leaf_count: usize) -> MapKey {
        match self.code2_order {
            Code2KeyOrder::NodeIdentityOrder => {
                let (level, lower_bound_position, disambiguator) = match identity {
                    ExtentNodeIdentity::Leaf(leaf_identifier) => (0u32, self.extent.position_of_leaf(leaf_identifier), leaf_identifier),
                    ExtentNodeIdentity::Internal { level, group } => {
                        let first_leaf = group.saturating_mul(self.geometry.extent_internal_fanout.saturating_pow(level)).min(extent_leaf_count - 1);
                        (level, first_leaf, u64::try_from(group).expect("组号装得进 u64"))
                    }
                };
                let lower_bound = self.extent.first_key_at(lower_bound_position);
                [CODE2_CLASS_TAG, TREE_IDENTIFIER, u64::from(level), lower_bound[0], lower_bound[1], self.current_txg, disambiguator]
            }
            Code2KeyOrder::BirthOrder => {
                let birth_sequence = self.next_birth_sequence;
                self.next_birth_sequence += 1;
                [CODE2_CLASS_TAG, TREE_IDENTIFIER, self.current_txg, INSTANCE_IDENTIFIER, birth_sequence, 0, 0]
            }
        }
    }

    /// 全部指针进映射时：这一次发布被 COW 的每个 extent 节点，映射条目删旧插新。
    fn refresh_extent_node_entries(&mut self) {
        let extent_leaf_count = self.extent.leaf_count();
        let touched_positions: Vec<usize> = self.touched_extent_leaves.iter().map(|leaf_identifier| self.extent.position_of_leaf(*leaf_identifier)).collect();
        let mut identities: Vec<ExtentNodeIdentity> = self.touched_extent_leaves.iter().map(|leaf_identifier| ExtentNodeIdentity::Leaf(*leaf_identifier)).collect();
        if extent_leaf_count > 1 {
            let mut group_size = self.geometry.extent_internal_fanout;
            let mut level = 1u32;
            loop {
                let groups: HashSet<usize> = touched_positions.iter().map(|position| position / group_size).collect();
                let mut sorted_groups: Vec<usize> = groups.into_iter().collect();
                sorted_groups.sort_unstable();
                identities.extend(sorted_groups.into_iter().map(|group| ExtentNodeIdentity::Internal { level, group }));
                if extent_leaf_count.div_ceil(group_size) <= 1 {
                    break;
                }
                group_size = group_size.saturating_mul(self.geometry.extent_internal_fanout);
                level += 1;
            }
        }
        identities.sort_unstable_by_key(|identity| match identity {
            ExtentNodeIdentity::Leaf(leaf_identifier) => (0u32, *leaf_identifier),
            ExtentNodeIdentity::Internal { level, group } => (*level, u64::try_from(*group).expect("组号装得进 u64")),
        });
        for identity in identities {
            if let Some(old_key) = self.extent_node_map_keys.get(&identity).copied() {
                let touched_leaf = self.map.remove(&old_key);
                self.touched_map_leaves.insert(touched_leaf);
            }
            let new_key = self.code2_map_key(identity, extent_leaf_count);
            for touched_leaf in self.map.insert(new_key) {
                self.touched_map_leaves.insert(touched_leaf);
            }
            self.extent_node_map_keys.insert(identity, new_key);
        }
    }

    /// 一次 fsync = 一次发布：先把被 COW 的 extent 节点的映射条目删旧插新（只在全部指针进映射时），再数两棵树各写了几块。
    fn checkpoint(&mut self) -> CheckpointBlocks {
        match self.map_scope {
            MapScope::AllPointers => self.refresh_extent_node_entries(),
            MapScope::Code1Only => {}
        }
        let extent_positions: Vec<usize> = self.touched_extent_leaves.iter().map(|leaf_identifier| self.extent.position_of_leaf(*leaf_identifier)).collect();
        let map_positions: Vec<usize> = self.touched_map_leaves.iter().map(|leaf_identifier| self.map.position_of_leaf(*leaf_identifier)).collect();
        let blocks = CheckpointBlocks {
            extent_leaves: u64::try_from(extent_positions.len()).expect("叶数装得进 u64"),
            extent_internal: touched_internal_node_count(&extent_positions, self.extent.leaf_count(), self.geometry.extent_internal_fanout),
            map_leaves: u64::try_from(map_positions.len()).expect("叶数装得进 u64"),
            map_internal: touched_internal_node_count(&map_positions, self.map.leaf_count(), self.geometry.map_internal_fanout),
        };
        self.touched_extent_leaves.clear();
        self.touched_map_leaves.clear();
        self.current_txg += 1;
        self.next_birth_sequence = 0;
        blocks
    }

    fn total_nodes(&self) -> u64 {
        total_node_count(self.extent.leaf_count(), self.geometry.extent_internal_fanout) + total_node_count(self.map.leaf_count(), self.geometry.map_internal_fanout)
    }
}

/// 初始镜像：`stream_count` 条流轮流追加，每条流顺着自己名下的文件依次写满，每写 `checkpoint_every_writes` 个单元一次 checkpoint。
fn build_image(shape: ImageShape, map_scope: MapScope, code1_order: Code1KeyOrder, code2_order: Code2KeyOrder, geometry: Geometry) -> Image {
    let mut image = Image::empty(map_scope, code1_order, code2_order, geometry);
    let total_units = shape.file_count * shape.units_per_file;
    for step in 0..total_units {
        let stream = step % shape.stream_count;
        let unit_index_in_stream = step / shape.stream_count;
        let object = stream + shape.stream_count * (unit_index_in_stream / shape.units_per_file);
        let offset = unit_index_in_stream % shape.units_per_file;
        image.write_unit(object, offset);
        if (step + 1) % shape.checkpoint_every_writes == 0 {
            image.checkpoint();
        }
    }
    if total_units % shape.checkpoint_every_writes != 0 {
        image.checkpoint();
    }
    image
}

fn build_registered_image(candidate: Candidate, map_scope: MapScope) -> Image {
    build_image(REGISTERED_IMAGE, map_scope, code1_key_order(candidate), code2_key_order(candidate), geometry(candidate, map_scope))
}

/// 确定性伪随机源（xorshift64*），种子相同则序列相同。
struct DeterministicRandom {
    state: u64,
}

impl DeterministicRandom {
    fn new(seed: u64) -> Self {
        Self { state: seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1 }
    }
    fn next_value(&mut self) -> u64 {
        self.state ^= self.state >> 12;
        self.state ^= self.state << 25;
        self.state ^= self.state >> 27;
        self.state.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    /// [0, bound) 里的一个数。
    fn below(&mut self, bound: u64) -> u64 {
        self.next_value() % bound
    }
}

/// 每次 fsync 的负载。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FsyncWorkload {
    SequentialAppend,
    MultistreamAppend,
    RandomOverwriteOneFile,
    RandomOverwriteGlobal,
    MetadataHeavyCreate,
}

const FSYNC_WORKLOADS: [FsyncWorkload; 5] = [
    FsyncWorkload::SequentialAppend,
    FsyncWorkload::MultistreamAppend,
    FsyncWorkload::RandomOverwriteOneFile,
    FsyncWorkload::RandomOverwriteGlobal,
    FsyncWorkload::MetadataHeavyCreate,
];

fn fsync_workload_name(workload: FsyncWorkload) -> &'static str {
    match workload {
        FsyncWorkload::SequentialAppend => "seq_append",
        FsyncWorkload::MultistreamAppend => "multistream_append",
        FsyncWorkload::RandomOverwriteOneFile => "rand_overwrite",
        FsyncWorkload::RandomOverwriteGlobal => "rand_overwrite_global",
        FsyncWorkload::MetadataHeavyCreate => "metaheavy_create",
    }
}

/// 一条流「当前的文件」：初始镜像里它最后写满的那个文件（`build_image` 按流轮流、每条流顺着自己名下的文件依次写）。
fn current_file_of_stream(shape: ImageShape, stream: u64) -> u64 {
    assert_eq!(shape.file_count % shape.stream_count, 0, "文件数要能按流均分，每条流才有同样多的文件");
    let files_per_stream = shape.file_count / shape.stream_count;
    stream + shape.stream_count * (files_per_stream - 1)
}

/// 在镜像的一份拷贝上跑一条每次 fsync 的负载，返回 fsync 次数与各项的总块数。
fn run_fsync_workload(base: &Image, shape: ImageShape, workload: FsyncWorkload, batch: u64, seed: u64, write_count: u64) -> (u64, CheckpointBlocks) {
    let mut image = base.clone();
    let mut random = DeterministicRandom::new(seed);
    let chosen_file = random.below(shape.file_count);
    let mut totals = CheckpointBlocks::zero();
    let mut fsync_count = 0u64;
    for write_index in 0..write_count {
        let (object, offset) = match workload {
            FsyncWorkload::SequentialAppend => (NEW_OBJECT_BASE, write_index),
            FsyncWorkload::MultistreamAppend => (current_file_of_stream(shape, write_index % shape.stream_count), shape.units_per_file + write_index / shape.stream_count),
            FsyncWorkload::RandomOverwriteOneFile => (chosen_file, random.below(shape.units_per_file)),
            FsyncWorkload::RandomOverwriteGlobal => (random.below(shape.file_count), random.below(shape.units_per_file)),
            FsyncWorkload::MetadataHeavyCreate => (NEW_OBJECT_BASE + 1 + write_index, 0),
        };
        image.write_unit(object, offset);
        if (write_index + 1) % batch == 0 {
            totals.add(&image.checkpoint());
            fsync_count += 1;
        }
    }
    (fsync_count, totals)
}

/// 后台负载（不按 fsync 算，按每个单元摊）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BackgroundWorkload {
    FileDelete,
    SnapshotDelete(SnapshotOverwriteSpread),
    Relocation,
}

/// 建快照之后那 20 000 次覆写落在哪：登记的是一个已有文件里（同 `rand_overwrite`），补记二加了全镜像（同 `rand_overwrite_global`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SnapshotOverwriteSpread {
    OneFile,
    WholeImage,
}

const BACKGROUND_WORKLOADS: [BackgroundWorkload; 4] = [
    BackgroundWorkload::FileDelete,
    BackgroundWorkload::SnapshotDelete(SnapshotOverwriteSpread::OneFile),
    BackgroundWorkload::SnapshotDelete(SnapshotOverwriteSpread::WholeImage),
    BackgroundWorkload::Relocation,
];

fn background_workload_name(workload: BackgroundWorkload) -> &'static str {
    match workload {
        BackgroundWorkload::FileDelete => "file_delete",
        BackgroundWorkload::SnapshotDelete(SnapshotOverwriteSpread::OneFile) => "snapshot_delete",
        BackgroundWorkload::SnapshotDelete(SnapshotOverwriteSpread::WholeImage) => "snapshot_delete_global",
        BackgroundWorkload::Relocation => "relocation",
    }
}

/// 后台负载的结果：动了几个单元、几次 checkpoint、各项总块数。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BackgroundResult {
    unit_count: u64,
    checkpoint_count: u64,
    totals: CheckpointBlocks,
}

fn run_background_workload(base: &Image, shape: ImageShape, workload: BackgroundWorkload, seed: u64, unit_budget: u64, deleted_file_count: u64) -> BackgroundResult {
    let mut image = base.clone();
    let mut random = DeterministicRandom::new(seed);
    let mut totals = CheckpointBlocks::zero();
    let mut unit_count = 0u64;
    let mut checkpoint_count = 0u64;
    match workload {
        BackgroundWorkload::FileDelete => {
            let mut files: Vec<u64> = (0..shape.file_count).collect();
            for pick in 0..deleted_file_count.min(shape.file_count) {
                let pick_index = usize::try_from(pick).expect("下标装得进 usize");
                let remaining = u64::try_from(files.len() - pick_index).expect("剩余数装得进 u64");
                let swap_index = pick_index + usize::try_from(random.below(remaining)).expect("下标装得进 usize");
                files.swap(pick_index, swap_index);
                let object = files[pick_index];
                for offset in 0..shape.units_per_file {
                    image.delete_unit(object, offset);
                    unit_count += 1;
                }
                totals.add(&image.checkpoint());
                checkpoint_count += 1;
            }
        }
        BackgroundWorkload::SnapshotDelete(spread) => {
            let chosen_file = match spread {
                SnapshotOverwriteSpread::OneFile => Some(random.below(shape.file_count)),
                SnapshotOverwriteSpread::WholeImage => None,
            };
            let snapshot_txg = image.current_txg;
            image.checkpoint();
            let mut retained: Vec<UnitVersion> = Vec::new();
            for write_index in 0..unit_budget {
                let object = match chosen_file {
                    Some(file) => file,
                    None => random.below(shape.file_count),
                };
                let offset = random.below(shape.units_per_file);
                if let Some(old_version) = image.write_unit_keeping_old(object, offset) {
                    if old_version.birth_txg < snapshot_txg {
                        retained.push(old_version);
                    } else {
                        image.free_version(&old_version);
                    }
                }
                if (write_index + 1) % SNAPSHOT_OVERWRITE_CHECKPOINT_EVERY_WRITES == 0 {
                    image.checkpoint();
                }
            }
            image.checkpoint();
            retained.sort_unstable_by_key(|version| (version.birth_txg, version.write_order));
            for (freed_index, version) in retained.iter().enumerate() {
                image.free_version(version);
                unit_count += 1;
                if (freed_index + 1) % usize::try_from(BACKGROUND_CHECKPOINT_EVERY_UNITS).expect("批装得进 usize") == 0 {
                    totals.add(&image.checkpoint());
                    checkpoint_count += 1;
                }
            }
            if unit_count % BACKGROUND_CHECKPOINT_EVERY_UNITS != 0 {
                totals.add(&image.checkpoint());
                checkpoint_count += 1;
            }
        }
        BackgroundWorkload::Relocation => {
            let image_unit_count = u64::try_from(image.allocation_order.len()).expect("单元数装得进 u64");
            let span = unit_budget.min(image_unit_count);
            let start = usize::try_from(random.below(image_unit_count - span + 1)).expect("下标装得进 usize");
            let span_length = usize::try_from(span).expect("跨度装得进 usize");
            let chosen: Vec<(u64, u64)> = image.allocation_order[start..start + span_length].to_vec();
            for (moved_index, (object, offset)) in chosen.iter().enumerate() {
                image.relocate_unit(*object, *offset);
                unit_count += 1;
                if (moved_index + 1) % usize::try_from(BACKGROUND_CHECKPOINT_EVERY_UNITS).expect("批装得进 usize") == 0 {
                    totals.add(&image.checkpoint());
                    checkpoint_count += 1;
                }
            }
            if unit_count % BACKGROUND_CHECKPOINT_EVERY_UNITS != 0 {
                totals.add(&image.checkpoint());
                checkpoint_count += 1;
            }
        }
    }
    BackgroundResult { unit_count, checkpoint_count, totals }
}

/// 定容 LRU，按节点计；`access` 返回是否命中。
struct NodeCache {
    capacity: u64,
    clock: u64,
    stamp_by_node: HashMap<u64, u64>,
    node_by_stamp: BTreeMap<u64, u64>,
}

impl NodeCache {
    fn new(capacity: u64) -> Self {
        assert!(capacity >= 1, "缓存至少一个节点");
        Self { capacity, clock: 0, stamp_by_node: HashMap::new(), node_by_stamp: BTreeMap::new() }
    }
    fn access(&mut self, node: u64) -> bool {
        self.clock += 1;
        let clock = self.clock;
        if let Some(old_stamp) = self.stamp_by_node.insert(node, clock) {
            self.node_by_stamp.remove(&old_stamp);
            self.node_by_stamp.insert(clock, node);
            return true;
        }
        self.node_by_stamp.insert(clock, node);
        if u64::try_from(self.stamp_by_node.len()).expect("缓存大小装得进 u64") > self.capacity {
            let (oldest_stamp, oldest_node) = self.node_by_stamp.pop_first().expect("缓存非空");
            debug_assert!(oldest_stamp < clock);
            self.stamp_by_node.remove(&oldest_node);
        }
        false
    }
}

const EXTENT_TREE_TAG: u64 = 0;
const MAP_TREE_TAG: u64 = 1;

/// 节点在缓存里的名字：哪棵树、第几层、第几个。
fn node_token(tree_tag: u64, level: u32, index: u64) -> u64 {
    assert!(index < (1u64 << 54), "节点号超出编码范围");
    (tree_tag << 62) | (u64::from(level) << 54) | index
}

/// 从根走到 `leaf_position` 那个叶经过的节点（根在前、叶在最后）。
fn path_tokens(tree_tag: u64, leaf_position: usize, leaf_identifier: u64, leaf_count: usize, internal_fanout: usize) -> Vec<u64> {
    let mut internal_tokens: Vec<u64> = Vec::new();
    if leaf_count > 1 {
        let mut group_size = internal_fanout;
        let mut level = 1u32;
        loop {
            internal_tokens.push(node_token(tree_tag, level, u64::try_from(leaf_position / group_size).expect("组号装得进 u64")));
            if leaf_count.div_ceil(group_size) <= 1 {
                break;
            }
            group_size = group_size.saturating_mul(internal_fanout);
            level += 1;
        }
    }
    internal_tokens.reverse();
    internal_tokens.push(node_token(tree_tag, 0, leaf_identifier));
    internal_tokens
}

/// 读一个单元走过的节点：extent 树从根到叶；提示过期时再走映射树从根到叶。
fn read_path_tokens(image: &Image, object: u64, offset: u64, through_map: bool) -> Vec<u64> {
    let extent_position = image.extent.position_for(&[object, offset]);
    let mut tokens = path_tokens(EXTENT_TREE_TAG, extent_position, image.extent.leaf_identifiers[extent_position], image.extent.leaf_count(), image.geometry.extent_internal_fanout);
    if through_map {
        let version = image.live_units.get(&(object, offset)).expect("要读的单元不在");
        let map_position = image.map.position_for(&code1_map_key(image.code1_order, version));
        tokens.extend(path_tokens(MAP_TREE_TAG, map_position, image.map.leaf_identifiers[map_position], image.map.leaf_count(), image.geometry.map_internal_fanout));
    }
    tokens
}

/// 均匀随机读：热身 `warmup` 次不计，再读 `measured` 次，返回未命中数。
fn run_random_reads(image: &Image, shape: ImageShape, cache_capacity: u64, stale_permille: u64, seed: u64, warmup: u64, measured: u64) -> u64 {
    let mut random = DeterministicRandom::new(seed);
    let mut cache = NodeCache::new(cache_capacity);
    let mut misses = 0u64;
    for read_index in 0..warmup + measured {
        let object = random.below(shape.file_count);
        let offset = random.below(shape.units_per_file);
        let through_map = random.below(PERMILLE) < stale_permille;
        for token in read_path_tokens(image, object, offset, through_map) {
            let hit = cache.access(token);
            if !hit && read_index >= warmup {
                misses += 1;
            }
        }
    }
    misses
}

/// 千分之一精度的平均数：`total / count`，保留三位小数（向下取整）。
fn milli_average_text(total: u64, count: u64) -> String {
    assert!(count > 0, "零次操作不是测到了 0，是没测");
    let scaled = u128::from(total) * u128::from(PERMILLE) / u128::from(count);
    format!("{}.{:03}", scaled / u128::from(PERMILLE), scaled % u128::from(PERMILLE))
}

fn main() {
    let mut emitter = Emitter::new();
    println!("E137 映射 key 形态的性能差距：每次 fsync 写几块、每次读几次 I/O");
    println!("判据写死在 research/prompts/e137-preregistration.md（写于本装置之前，补记在第一次运行之前）；只报数、不判输赢");
    println!(
        "镜像：{FILE_COUNT} 文件 × {UNITS_PER_FILE} 单元，{STREAM_COUNT} 条流轮流追加，每 {IMAGE_CHECKPOINT_EVERY_WRITES} 次写一次 checkpoint；每条负载写 {MEASURED_WRITE_COUNT} 次；种子 1 / 2 / 3"
    );
    for map_scope in MAP_SCOPES {
        let images: Vec<(Candidate, Image)> = CANDIDATES.iter().map(|candidate| (*candidate, build_registered_image(*candidate, map_scope))).collect();
        println!("—— 配置 {}：镜像", map_scope_name(map_scope));
        for (candidate, image) in images.iter() {
            let geometry_of_image = image.geometry;
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=image config={} arm={} map_leaf_capacity={} map_internal_fanout={} extent_leaf_capacity={} extent_internal_fanout={} map_leaves={} extent_leaves={} total_nodes={}",
                    map_scope_name(map_scope),
                    candidate_name(*candidate),
                    geometry_of_image.map_leaf_capacity,
                    geometry_of_image.map_internal_fanout,
                    geometry_of_image.extent_leaf_capacity,
                    geometry_of_image.extent_internal_fanout,
                    image.map.leaf_count(),
                    image.extent.leaf_count(),
                    image.total_nodes()
                ))
            );
        }
        println!("—— 配置 {}：每次 fsync 写几块（平均，含 1 根槽 + 1 记录）", map_scope_name(map_scope));
        for workload in FSYNC_WORKLOADS {
            for batch in BATCHES {
                for (candidate, image) in images.iter() {
                    for seed in SEEDS {
                        let (fsync_count, totals) = run_fsync_workload(image, REGISTERED_IMAGE, workload, batch, seed, MEASURED_WRITE_COUNT);
                        println!(
                            "{}",
                            emitter.emit_raw(&format!(
                                "name=fsync config={} workload={} batch={batch} arm={} seed={seed} fsyncs={fsync_count} extent_leaves={} extent_internal={} map_leaves={} map_internal={} blocks={}",
                                map_scope_name(map_scope),
                                fsync_workload_name(workload),
                                candidate_name(*candidate),
                                milli_average_text(totals.extent_leaves, fsync_count),
                                milli_average_text(totals.extent_internal, fsync_count),
                                milli_average_text(totals.map_leaves, fsync_count),
                                milli_average_text(totals.map_internal, fsync_count),
                                milli_average_text(totals.total_with_root_and_record() + (fsync_count - 1) * (ROOT_SLOT_BLOCKS + JOURNAL_RECORD_BLOCKS), fsync_count)
                            ))
                        );
                    }
                }
            }
        }
        println!("—— 配置 {}：后台负载，按每个单元摊的映射树块数", map_scope_name(map_scope));
        for workload in BACKGROUND_WORKLOADS {
            for (candidate, image) in images.iter() {
                for seed in SEEDS {
                    let result = run_background_workload(image, REGISTERED_IMAGE, workload, seed, BACKGROUND_UNIT_COUNT, DELETED_FILE_COUNT);
                    println!(
                        "{}",
                        emitter.emit_raw(&format!(
                            "name=background config={} workload={} arm={} seed={seed} units={} checkpoints={} map_leaves={} map_internal={} extent_leaves={} map_blocks_per_unit={}",
                            map_scope_name(map_scope),
                            background_workload_name(workload),
                            candidate_name(*candidate),
                            result.unit_count,
                            result.checkpoint_count,
                            result.totals.map_leaves,
                            result.totals.map_internal,
                            result.totals.extent_leaves,
                            milli_average_text(result.totals.map_leaves + result.totals.map_internal, result.unit_count)
                        ))
                    );
                }
            }
        }
        println!("—— 配置 {}：均匀随机读，每次读的节点未命中数（缓存容量取逻辑身份 + 写序键同配置两棵树节点总数的千分比）", map_scope_name(map_scope));
        let logical_total_nodes = images[0].1.total_nodes();
        for cache_permille in CACHE_PERMILLE_OF_LOGICAL_ARM {
            let cache_capacity = (logical_total_nodes * cache_permille / PERMILLE).max(1);
            for stale_permille in STALE_HINT_PERMILLE {
                for (candidate, image) in images.iter() {
                    for seed in SEEDS {
                        let misses = run_random_reads(image, REGISTERED_IMAGE, cache_capacity, stale_permille, seed, READ_WARMUP_COUNT, READ_MEASURED_COUNT);
                        println!(
                            "{}",
                            emitter.emit_raw(&format!(
                                "name=read config={} arm={} cache_permille={cache_permille} cache_nodes={cache_capacity} stale_permille={stale_permille} seed={seed} reads={READ_MEASURED_COUNT} misses={misses} misses_per_read={}",
                                map_scope_name(map_scope),
                                candidate_name(*candidate),
                                milli_average_text(misses, READ_MEASURED_COUNT)
                            ))
                        );
                    }
                }
            }
        }
        println!("—— 配置 {}：阳性对照（缓存装得下全部节点 ⇒ 热身后 0 未命中；缓存 1 个节点 ⇒ 未命中数 = 走过的节点数）", map_scope_name(map_scope));
        for (candidate, image) in images.iter() {
            let everything_cached = run_random_reads(image, REGISTERED_IMAGE, image.total_nodes() + 1, 1000, 1, READ_WARMUP_COUNT * 20, 10_000);
            let mut random = DeterministicRandom::new(1);
            let mut walked_nodes = 0u64;
            for _ in 0..10_000u64 {
                let object = random.below(REGISTERED_IMAGE.file_count);
                let offset = random.below(REGISTERED_IMAGE.units_per_file);
                let through_map = random.below(PERMILLE) < 1000;
                walked_nodes += u64::try_from(read_path_tokens(image, object, offset, through_map).len()).expect("路径长装得进 u64");
            }
            let one_node_cache = run_random_reads(image, REGISTERED_IMAGE, 1, 1000, 1, 0, 10_000);
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=positive_control config={} arm={} all_cached_misses={everything_cached} one_node_cache_misses={one_node_cache} walked_nodes={walked_nodes}",
                    map_scope_name(map_scope),
                    candidate_name(*candidate)
                ))
            );
        }
    }
    println!("—— 判别力对照：按类复用写序键的码 1 key 换成按文件的顺序（宽度不动），映射树叶数必须变");
    let geometry_of_class_reuse = geometry(Candidate::ClassReuseTagged, MapScope::Code1Only);
    let own_order = build_image(REGISTERED_IMAGE, MapScope::Code1Only, Code1KeyOrder::BirthOrder, Code2KeyOrder::BirthOrder, geometry_of_class_reuse);
    let swapped_order = build_image(REGISTERED_IMAGE, MapScope::Code1Only, Code1KeyOrder::FileOrder, Code2KeyOrder::BirthOrder, geometry_of_class_reuse);
    let (_, own_multistream) = run_fsync_workload(&own_order, REGISTERED_IMAGE, FsyncWorkload::MultistreamAppend, 10, 1, MEASURED_WRITE_COUNT);
    let (_, swapped_multistream) = run_fsync_workload(&swapped_order, REGISTERED_IMAGE, FsyncWorkload::MultistreamAppend, 10, 1, MEASURED_WRITE_COUNT);
    let own_delete = run_background_workload(&own_order, REGISTERED_IMAGE, BackgroundWorkload::FileDelete, 1, BACKGROUND_UNIT_COUNT, DELETED_FILE_COUNT);
    let swapped_delete = run_background_workload(&swapped_order, REGISTERED_IMAGE, BackgroundWorkload::FileDelete, 1, BACKGROUND_UNIT_COUNT, DELETED_FILE_COUNT);
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=discrimination_control multistream_map_leaves_own={} multistream_map_leaves_swapped={} file_delete_map_leaves_own={} file_delete_map_leaves_swapped={}",
            own_multistream.map_leaves, swapped_multistream.map_leaves, own_delete.totals.map_leaves, swapped_delete.totals.map_leaves
        ))
    );
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    const TINY_GEOMETRY: Geometry = Geometry { map_leaf_capacity: 2, map_internal_fanout: 2, extent_leaf_capacity: 2, extent_internal_fanout: 2 };
    const TINY_IMAGE: ImageShape = ImageShape { file_count: 2, units_per_file: 2, stream_count: 2, checkpoint_every_writes: 2 };

    fn contents(leaves: &OrderedLeaves<u64>) -> Vec<Vec<u64>> {
        leaves.leaves.clone()
    }

    /// 叶容量 4：顺序追加走最右叶另起新叶，别处插入对半分；删除只弄脏一个叶。
    #[test]
    fn leaves_split_at_the_right_edge_by_starting_a_new_leaf_and_elsewhere_in_half() {
        let mut leaves: OrderedLeaves<u64> = OrderedLeaves::new(4);
        let mut last_touched = Vec::new();
        for key in 1..=9u64 {
            last_touched = leaves.insert(key);
        }
        assert_eq!(contents(&leaves), vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8], vec![9]]);
        assert_eq!(last_touched, vec![2], "追加 9 时旧的最右叶不动，只写新叶");
        assert_eq!(leaves.insert(0), vec![0, 3], "往满的最左叶插 0：对半分，两个叶都脏");
        assert_eq!(contents(&leaves), vec![vec![0, 1], vec![2, 3, 4], vec![5, 6, 7, 8], vec![9]]);
        assert_eq!(leaves.first_keys, vec![0, 2, 5, 9]);
        assert_eq!(leaves.remove(&5), 1);
        assert_eq!(leaves.position_of_leaf(1), 2, "编号 1 的叶在分裂之后挪到了第 2 位");
        assert_eq!(leaves.touch(&9), 2);
    }

    /// 被触到的叶 {0, 1, 5}、共 6 个叶、内部扇出 2：第 1 层 2 个、第 2 层 2 个、根 1 个。
    #[test]
    fn touched_internal_nodes_are_counted_level_by_level_up_to_the_root() {
        assert_eq!(touched_internal_node_count(&[0, 1, 5], 6, 2), 5);
        assert_eq!(touched_internal_node_count(&[0], 1, 2), 0, "只有一个叶时它就是根");
        assert_eq!(touched_internal_node_count(&[3], 4, 4), 1);
        assert_eq!(touched_internal_node_count(&[], 6, 2), 0);
        assert_eq!(total_node_count(6, 2), 6 + 3 + 2 + 1);
    }

    #[test]
    fn checkpoint_blocks_add_one_root_slot_and_one_record() {
        let blocks = CheckpointBlocks { extent_leaves: 1, extent_internal: 2, map_leaves: 3, map_internal: 4 };
        assert_eq!(blocks.total_with_root_and_record(), 12);
    }

    /// 容量 2，访问 a b a c b：a、b 未命中，a 命中，c 逐出 b，b 再未命中 ⇒ 4 次未命中。
    #[test]
    fn the_node_cache_evicts_the_least_recently_used_node() {
        let mut cache = NodeCache::new(2);
        let hits: Vec<bool> = [10u64, 20, 10, 30, 20].iter().map(|node| cache.access(*node)).collect();
        assert_eq!(hits, vec![false, false, true, false, false]);
    }

    #[test]
    fn code1_map_keys_order_by_file_or_by_birth() {
        let version = UnitVersion { object: 7, offset: 3, object_birth_txg: 2, birth_txg: 10, write_order: 99 };
        assert_eq!(code1_map_key(Code1KeyOrder::FileOrder, &version), [1, 1, 7, 2, 3, 10, 99]);
        assert_eq!(code1_map_key(Code1KeyOrder::BirthOrder, &version), [1, 1, 10, 1, 99, 0, 0]);
        assert_eq!(code1_key_order(Candidate::MixedPath), Code1KeyOrder::FileOrder, "混合路的码 1 半就是逻辑身份 + 写序键");
        assert_eq!(code2_key_order(Candidate::MixedPath), Code2KeyOrder::BirthOrder, "混合路的码 2 半就是按类复用写序键");
    }

    /// 跨装置闸：叶容量与内部扇出逐格等于 E136 留存产物里对应的行。
    #[test]
    fn geometry_matches_the_e136_product() {
        let e136_product = include_str!("../../../results/e136-fork-cost-rows-2026-09-11.out");
        let field = |line: &str, name: &str| -> usize {
            let prefix = format!("{name}=");
            let value = line.split_whitespace().find(|part| part.starts_with(&prefix)).expect("E136 那一行少字段");
            value[prefix.len()..].parse().expect("字段不是数")
        };
        for candidate in CANDIDATES {
            let all_line = e136_product.lines().find(|line| line.starts_with(&format!("E7RESULT name=fanouts arm={} ", candidate_name(candidate)))).expect("E136 缺 all 配置的扇出行");
            let code1_line = e136_product.lines().find(|line| line.starts_with(&format!("E7RESULT name=code1_only_fanouts arm={} ", candidate_name(candidate)))).expect("E136 缺只装码 1 的扇出行");
            for (map_scope, line) in [(MapScope::AllPointers, all_line), (MapScope::Code1Only, code1_line)] {
                let expected = Geometry {
                    map_leaf_capacity: field(line, "map_leaf"),
                    map_internal_fanout: field(line, "map_internal"),
                    extent_leaf_capacity: field(line, "extent_leaf"),
                    extent_internal_fanout: field(line, "extent_internal"),
                };
                assert_eq!(geometry(candidate, map_scope), expected, "{} / {:?}", candidate_name(candidate), map_scope);
            }
        }
    }

    /// 手算的小镜像：2 文件 × 2 单元，两条流，每 2 次写一次 checkpoint，叶容量 2、内部扇出 2。
    /// extent 叶：[f0o0] [f0o1 f1o0] [f1o1]；按出生排的映射叶：[w0 w1] [w2 w3]；按文件排的映射叶与 extent 同形。
    /// 给两个文件各追加一个单元、一次 fsync：按出生排的映射只写 1 个新叶（最右追加），按文件排的写 3 个叶（一次对半分 + 一次追加）。
    #[test]
    fn hand_computed_multistream_fsync_separates_birth_order_from_file_order() {
        let birth = build_image(TINY_IMAGE, MapScope::Code1Only, Code1KeyOrder::BirthOrder, Code2KeyOrder::BirthOrder, TINY_GEOMETRY);
        let file = build_image(TINY_IMAGE, MapScope::Code1Only, Code1KeyOrder::FileOrder, Code2KeyOrder::NodeIdentityOrder, TINY_GEOMETRY);
        assert_eq!(birth.extent.leaves, vec![vec![[0, 0]], vec![[0, 1], [1, 0]], vec![[1, 1]]]);
        assert_eq!(birth.map.leaf_count(), 2);
        assert_eq!(file.map.leaf_count(), 3);
        let (birth_fsyncs, birth_blocks) = run_fsync_workload(&birth, TINY_IMAGE, FsyncWorkload::MultistreamAppend, 2, 1, 2);
        let (file_fsyncs, file_blocks) = run_fsync_workload(&file, TINY_IMAGE, FsyncWorkload::MultistreamAppend, 2, 1, 2);
        assert_eq!((birth_fsyncs, file_fsyncs), (1, 1));
        assert_eq!(birth_blocks, CheckpointBlocks { extent_leaves: 3, extent_internal: 3, map_leaves: 1, map_internal: 2 });
        assert_eq!(file_blocks, CheckpointBlocks { extent_leaves: 3, extent_internal: 3, map_leaves: 3, map_internal: 3 });
        assert_eq!(birth_blocks.total_with_root_and_record(), 11);
        assert_eq!(file_blocks.total_with_root_and_record(), 14);
    }

    /// 小镜像上删文件 0（两个单元）：按出生排的删 w0（叶 [w0 w1]）与 w2（叶 [w2 w3]），按文件排的删 [f0o0] 与 [f0o1 f1o0] 里各一条 ⇒ 都是 2 个叶。
    #[test]
    fn hand_computed_file_delete_touches_two_map_leaves_either_way() {
        for (code1_order, code2_order) in [(Code1KeyOrder::BirthOrder, Code2KeyOrder::BirthOrder), (Code1KeyOrder::FileOrder, Code2KeyOrder::NodeIdentityOrder)] {
            let image = build_image(TINY_IMAGE, MapScope::Code1Only, code1_order, code2_order, TINY_GEOMETRY);
            let mut copy = image.clone();
            copy.delete_unit(0, 0);
            copy.delete_unit(0, 1);
            let blocks = copy.checkpoint();
            assert_eq!(blocks.map_leaves, 2, "{code1_order:?}");
        }
    }

    /// 一条流当前的文件是它在初始镜像里最后写满的那个：登记镜像 256 个文件一条流，流 0 是 4080、流 15 是 4095；小镜像一条流一个文件，就是它自己。
    #[test]
    fn each_stream_appends_to_the_last_file_it_filled_in_the_image() {
        assert_eq!(current_file_of_stream(REGISTERED_IMAGE, 0), 4080);
        assert_eq!(current_file_of_stream(REGISTERED_IMAGE, 15), 4095);
        assert_eq!(current_file_of_stream(TINY_IMAGE, 1), 1);
        let two_files_per_stream = ImageShape { file_count: 4, units_per_file: 2, stream_count: 2, checkpoint_every_writes: 2 };
        let image = build_image(two_files_per_stream, MapScope::Code1Only, Code1KeyOrder::BirthOrder, Code2KeyOrder::BirthOrder, TINY_GEOMETRY);
        let last_written = *image.allocation_order.last().expect("镜像非空");
        assert_eq!(last_written, (3, 1), "流 1 最后写的是文件 3 的末单元");
        assert_eq!(current_file_of_stream(two_files_per_stream, 1), 3);
    }

    /// 后台负载整条跑一遍（按出生排的小镜像，与随机源无关的那几格）：
    /// 快照之后覆写 200 次，四个（或一个文件的两个）原版全被覆写的概率不到 1e-24，被快照留着的就是它们；
    /// 原版 w0..w3 住在最左两个叶里、之后的插入全落在最右 ⇒ 放掉它们弄脏 2 个叶；四个都不到 256 ⇒ 收尾一次 checkpoint。
    /// 搬迁的跨度等于全部 4 个单元 ⇒ 起点只能是 0，两个叶都脏；删一个文件 ⇒ 两个单元分在两个叶里。
    #[test]
    fn background_workloads_run_end_to_end_on_the_tiny_image() {
        let image = build_image(TINY_IMAGE, MapScope::Code1Only, Code1KeyOrder::BirthOrder, Code2KeyOrder::BirthOrder, TINY_GEOMETRY);
        let whole = run_background_workload(&image, TINY_IMAGE, BackgroundWorkload::SnapshotDelete(SnapshotOverwriteSpread::WholeImage), 1, 200, 1);
        assert_eq!((whole.unit_count, whole.checkpoint_count, whole.totals.map_leaves), (4, 1, 2));
        let one_file = run_background_workload(&image, TINY_IMAGE, BackgroundWorkload::SnapshotDelete(SnapshotOverwriteSpread::OneFile), 1, 200, 1);
        assert_eq!((one_file.unit_count, one_file.checkpoint_count, one_file.totals.map_leaves), (2, 1, 2));
        let relocation = run_background_workload(&image, TINY_IMAGE, BackgroundWorkload::Relocation, 1, 4, 1);
        assert_eq!((relocation.unit_count, relocation.checkpoint_count, relocation.totals.map_leaves, relocation.totals.extent_leaves), (4, 1, 2, 0));
        let file_delete = run_background_workload(&image, TINY_IMAGE, BackgroundWorkload::FileDelete, 1, 4, 1);
        assert_eq!((file_delete.unit_count, file_delete.checkpoint_count, file_delete.totals.map_leaves, file_delete.totals.extent_leaves), (2, 1, 2, 2));
    }

    /// 搬迁只改映射条目的 value：key 不变、叶数不变，弄脏的是那条条目所在的叶。
    #[test]
    fn relocation_touches_the_leaf_holding_the_entry_without_changing_the_tree() {
        let image = build_image(TINY_IMAGE, MapScope::Code1Only, Code1KeyOrder::BirthOrder, Code2KeyOrder::BirthOrder, TINY_GEOMETRY);
        let mut copy = image.clone();
        copy.relocate_unit(0, 0);
        copy.relocate_unit(1, 0);
        let blocks = copy.checkpoint();
        assert_eq!(blocks.map_leaves, 1, "w0 与 w1 在同一个叶里");
        assert_eq!(copy.map.leaf_count(), image.map.leaf_count());
        assert_eq!(blocks.extent_leaves, 0);
    }

    /// 快照留着的旧版按出生顺序放：小镜像上把两个文件的偏移 0 各覆写一次再放旧版 ⇒ 按出生排的两条旧版 w0、w1 在同一个叶里。
    #[test]
    fn retained_versions_freed_in_birth_order_sit_together_under_birth_order() {
        let mut image = build_image(TINY_IMAGE, MapScope::Code1Only, Code1KeyOrder::BirthOrder, Code2KeyOrder::BirthOrder, TINY_GEOMETRY);
        let first = image.write_unit_keeping_old(0, 0).expect("旧版在");
        let second = image.write_unit_keeping_old(1, 0).expect("旧版在");
        image.checkpoint();
        image.free_version(&first);
        image.free_version(&second);
        assert_eq!(image.checkpoint().map_leaves, 1);
    }

    /// 全部指针进映射时，被 COW 的 extent 节点各有一条映射条目删旧插新；只装码 1 时一条都没有。
    #[test]
    fn extent_node_entries_enter_the_map_only_when_all_pointers_do() {
        let all = build_image(TINY_IMAGE, MapScope::AllPointers, Code1KeyOrder::BirthOrder, Code2KeyOrder::BirthOrder, TINY_GEOMETRY);
        let code1 = build_image(TINY_IMAGE, MapScope::Code1Only, Code1KeyOrder::BirthOrder, Code2KeyOrder::BirthOrder, TINY_GEOMETRY);
        assert_eq!(code1.extent_node_map_keys.len(), 0);
        assert_eq!(all.extent_node_map_keys.len(), 3 + 2 + 1, "小镜像最后一次 checkpoint 之后：3 个叶、第 1 层 2 个、根 1 个都有条目");
        let map_entries = |image: &Image| image.map.leaves.iter().map(|leaf| leaf.len()).sum::<usize>();
        assert_eq!(map_entries(&code1), 4);
        assert_eq!(map_entries(&all), 4 + 6);
    }

    /// 阳性对照：缓存装得下全部节点时热身之后 0 未命中；缓存只有 1 个节点时未命中数等于走过的节点数。
    #[test]
    fn the_read_cache_has_both_ends_right_on_the_tiny_image() {
        let image = build_image(TINY_IMAGE, MapScope::Code1Only, Code1KeyOrder::FileOrder, Code2KeyOrder::NodeIdentityOrder, TINY_GEOMETRY);
        assert_eq!(run_random_reads(&image, TINY_IMAGE, image.total_nodes() + 1, 1000, 1, 1000, 1000), 0);
        let path = read_path_tokens(&image, 0, 0, true);
        assert_eq!(path.len(), 3 + 3, "extent 树与映射树各 3 层（叶 + 第 1 层 + 根）");
        let mut random = DeterministicRandom::new(1);
        let mut walked = 0u64;
        for _ in 0..1000u64 {
            let object = random.below(TINY_IMAGE.file_count);
            let offset = random.below(TINY_IMAGE.units_per_file);
            let _ = random.below(PERMILLE);
            walked += u64::try_from(read_path_tokens(&image, object, offset, true).len()).expect("路径长装得进 u64");
        }
        assert_eq!(run_random_reads(&image, TINY_IMAGE, 1, 1000, 1, 0, 1000), walked);
    }

    #[test]
    fn milli_averages_keep_three_decimals_and_refuse_zero_counts() {
        assert_eq!(milli_average_text(7, 3), "2.333");
        assert_eq!(milli_average_text(20, 10), "2.000");
    }
}
