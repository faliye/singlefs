//! E104：扫描重建时多版单元的现行版本判定 —— C113 定案提案（第一版）P1–P6 的机制验证。
//!
//! **它答的是**：只剩单元 + 记账 + 根的时候，同一逻辑身份的多个物理版本里哪一版是哪个根的
//! 现行版本，提案给的那组输入（写序、实例水位、祖先表、死亡写序、容器退役记录、scrub 门）
//! 够不够；每一样是不是都承重。E59 乙臂的「记账」是写入时置好的 `live: bool` 神谕，
//! 这里把它换成从盘上算出来的机制，并保留神谕臂当阳性对照。
//!
//! ## 逐字贴的被引条款（verify-before-claiming.md：不许照印象建模）
//!
//! - D5 定义表：`birth(b)` = 块被**发布**的那个 checkpoint 号，「不是写请求发出时所在的开放 txg——
//!   崩溃后该号会被重发」。
//! - D16 已定项 6：每次发布 checkpoint_txg + 1；已定项 7：持久顺序恒为
//!   COW 单元 → 屏障 → journal 记录 → 屏障 → 根槽。
//! - D23 已定项 9：jsn = 实例代号 32 位 + 计数器 48 位；实例代号每次恢复递增。
//!   已定项 14：恢复只施加 (实例代号, checkpoint_txg) 严格大于所选根的记录，jsn 严格连续、断号即止。
//! - D18 已定项 1：谱系重写序号不做；射程是「为分辨陈旧副本而设的序号」。
//! - D18 已定项 10：墓碑「在它指认的死亡已被全部还引用它的快照送走之后可回收」。
//! - D8 已定项 6：类型 2 容器身份 (出生树, 2, 容器号, 出生代)，同一头内 COW 重写身份不变，
//!   合并只许左吸收右、右半退役；inode 号单调不复用。
//! - D3 已定项 1 / D21 已定项 5：谓词的权威记录点是运行时口径；分配记录是派生态。
//!
//! ## 判据（跑前写死，写在提案 P8，跑完不许改）
//!
//! 1. 全规则臂对全部根、全部种子：错版本 = 复活 = 判死实活 = 歧义 = 0。任一非 0 ⇒ 提案不完整，
//!    记下是哪一类世界，不许回头改规则再跑同一个实验。
//! 2. 每条消融臂在它对应的靶子世界上必须 > 0，且数值钉闭式；闭式由世界生成器独立计数，
//!    不从被测代码读回。
//! 3. 神谕臂恒 0；扫到的单元数 ≠ 独立算出的应有数 ⇒ 整轮作废；注入 k 处破坏必须报出恰 k。
//! 4. 若全规则臂在某类世界上非 0 而消融臂也非 0，缺的构件不在六个之内，提案回到靶子重画。
//!
//! ## 它测的是什么、不是什么
//!
//! 计数模型，零文件操作，没有并发。「盘」是 loc → 单元的映射，复用一个 loc 就是把旧头盖掉。
//! 真值由生成器在每个根发布那一刻记下，不经过任何重建代码。

use e7_index_bench::Emitter;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

type Tree = u32;
type Txg = u64;

/// 写序 = 点名该单元的 journal 记录的 jsn（实例代号 32 + 计数器 48）。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
struct WriteSequence {
    instance: u32,
    counter: u64,
}

/// 逻辑身份。数据单元 = (对象 ID, 对象出生代)，锚点恒 0；容器 = 码 3 的四元组。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
enum Key {
    Data { object_number: u64, object_birth: Txg },
    Container { birth_tree: Tree, kind: u8, container_number: u64, birth: Txg },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TombstoneRecord {
    /// 对象死亡：指认 key，带死亡写序与死亡代号。
    Kill { key: Key, death: WriteSequence, death_txg: Txg },
    /// 容器退役（打包记录类型 3）。第三版去掉了退役写序：身份不复用，退役不需要与什么比大小；
    /// 对某个根可见与否由装着它的容器版本的诞生代号决定。`at_txg` 只给回收门用。
    Retire { retired_container: Key, at_txg: Txg },
}

#[derive(Clone, Debug)]
enum Payload {
    /// 数据单元的载荷内容不参与判定（现行与否看落点），不建模
    Data,
    Inodes(BTreeMap<u64, u64>),
    Tombstones(Vec<TombstoneRecord>),
}

#[derive(Clone, Debug)]
struct Unit {
    key: Key,
    writer: Tree,
    birth: Txg,
    write_sequence: WriteSequence,
    payload: Payload,
}

/// 一个根（可写头的最新发布态，或一个快照）的真值视图。
#[derive(Clone, Debug, Default)]
struct View {
    tree: Tree,
    txg: Txg,
    /// 对象 → 落点
    data: BTreeMap<Key, u64>,
    /// inode 号 → (容器身份, 容器落点, 内容版本)
    inodes: BTreeMap<u64, (Key, u64, u64)>,
    /// 这个根引用的全部落点（含墓碑容器）：神谕臂用它当 `live`
    referenced_locations: BTreeSet<u64>,
    /// 回退时重建头要用的：容器与墓碑容器的身份 / 落点、下一个墓碑容器号
    containers: Vec<(Key, u64)>,
    open_tombstone_container: Option<(Key, u64)>,
    closed_tombstone_containers: Vec<(Key, u64)>,
    next_tombstone_container_number: u64,
}

#[derive(Clone, Debug)]
struct ContainerLeaf {
    key: Key,
    location: u64,
    inode_records: BTreeMap<u64, u64>,
    dirty: bool,
    /// 这片叶是不是从 origin 共享来的（还没被本头 COW 过）
    shared: bool,
}

#[derive(Clone, Debug)]
struct Head {
    tree: Tree,
    data: BTreeMap<Key, u64>,
    containers: Vec<ContainerLeaf>,
    /// 当前开放的墓碑容器（身份, 落点, 记录）
    open_tombstone_container: Option<(Key, u64, Vec<TombstoneRecord>)>,
    /// 已关闭但仍持有记录的墓碑容器
    closed_tombstone_containers: Vec<(Key, u64, Vec<TombstoneRecord>)>,
    /// 本 txg 内已施加事务的暂存视图（发布时提升为真值）
    next_tombstone_container_number: u64,
}

/// 写路径与回收纪律的开关——每个都对应提案里一条纪律，关掉是为了证明它承重。
#[derive(Clone, Copy, Debug)]
struct WorldConfiguration {
    /// P6：跨头首次 COW 让容器重生
    rebirth: bool,
    /// P5：合并 / 清空时写容器退役记录
    retire_record: bool,
    /// P4：墓碑回收多一道抹头水位门
    scrub_gate: bool,
    /// P4 第二版：门的下限取「被杀版本变成垃圾的时刻」= max(死亡代号, 最近一次根被销毁的代号)，
    /// 而不只是死亡代号（第一版的写法，E104 第一轮在混合世界上打出复活）
    gate_floor_includes_root_removal: bool,
    /// W1 第二句：事务内先写后删的单元在记录写出前作废
    invalidate_in_transaction: bool,
    /// 恢复时给根实例到新实例之间的每个实例写水位行
    write_watermarks: bool,
    /// 回退时丢掉被抛弃时间线的 defer 条目。关掉也不出错：释放时的引用检查会把它们放进 deadlist——
    /// 这条等价性有单测留档，所以变异表不拿它当变异
    retain_abandoned_defer: bool,
    /// 分配器复用释放空间的概率（万分比）
    reuse_basis_points: u32,
    /// 每个单元写几份副本（D2 已定项 6：w ≥ 2；模型默认 1，靶子世界开 2）
    replica_count: u8,
    /// 回退后第一个新根的 txg 取根环里全部根 txg 的最大值 + 1（关掉 = 取 T_old + 1，被抛弃的根会在下次择新时赢）
    rollback_skips_ring: bool,
    /// 第六版 P1 失败表 ③：一个事务在发出单元写之后失败 ⇒ 实例结束，号更大的事务的记录一条都不再追加
    /// （记录按事务号顺序追加）。关掉 = 第五版的形态：后续事务照常追加，失败事务的作废信息随崩溃丢失
    records_stop_at_failure: bool,
    /// 第六版 P2 行回收：kind 0 行可删还要求根环里没有该实例发布的根（第五轮反推腿 4.1）
    row_reclaim_checks_ring: bool,
    /// 第八版 P2：恢复行的 W 按实例分——只有所选根那个实例的 W 可能非 0（jsn 在实例边界必然断号）。
    /// 关掉 = 第七版落地那句的字面：一个全局的「最大已施加事务号」写给每一个实例（第七轮反推腿 3.1）
    row_applied_counter_per_instance: bool,
    /// 第八版 P3：码 3 的择新键取 (诞生代号, 实例代号)。关掉 = 第七版的「取写序最大者」——
    /// 同实例连着两个 checkpoint 都不分配新事务号时两版容器写序逐字节相同（第七轮反推腿 2.1）
    container_key_by_birth: bool,
    /// 生成器手工冻结容器写序的事务号那一维（只有第七轮反推腿 2.1 的世界用）：
    /// 真实形态里容器只在 checkpoint 的固定点写出、写序 = 那次 checkpoint 已分配的最大事务号，
    /// 模型按事务写容器，冻结是把「同一 checkpoint 内写序不变」这个性质手工造出来
    container_write_sequence_frozen: bool,
    /// 第七版 P2：恢复行的 T_pub 取所选根的 checkpoint_txg。关掉 = 第六版的字面「在飞 txg − 1」——
    /// 所选根不是最新已发布根时（最新根的槽坏了、它之后的记录又不可重放），把丢掉的那个 checkpoint 判成已发布（第六轮反推腿 2.1）
    row_published_txg_from_chosen_root: bool,
}

impl WorldConfiguration {
    const FULL: WorldConfiguration = WorldConfiguration {
        rebirth: true,
        retire_record: true,
        scrub_gate: true,
        gate_floor_includes_root_removal: true,
        invalidate_in_transaction: true,
        write_watermarks: true,
        retain_abandoned_defer: true,
        reuse_basis_points: 5000,
        replica_count: 1,
        rollback_skips_ring: true,
        records_stop_at_failure: true,
        row_reclaim_checks_ring: true,
        row_published_txg_from_chosen_root: true,
        row_applied_counter_per_instance: true,
        container_key_by_birth: true,
        container_write_sequence_frozen: false,
    };
}

const CONTAINER_CAPACITY_IN_RECORDS: usize = 4;

/// 确定性伪随机源（xorshift64*）。
struct RandomSource(u64);
impl RandomSource {
    fn new(seed: u64) -> Self {
        RandomSource(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }
    fn next(&mut self) -> u64 {
        let mut state = self.0;
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        self.0 = state;
        state.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, bound: u64) -> u64 {
        if bound == 0 { 0 } else { self.next() % bound }
    }
    fn chance_basis_points(&mut self, basis_points: u32) -> bool {
        self.below(10_000) < basis_points as u64
    }
}

/// 分配记录条目的两种状态（D3 已定项 7 的 value 加一位：已释放 + 释放代）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AllocationState {
    Allocated,
    Freed(Txg),
}

/// 一个事务里对一个头的操作（生成器给的，已按 W1 合并成每 key 至多一条净效果）。
#[derive(Clone, Copy, Debug)]
enum Operation {
    CreateObject,
    Overwrite(Key),
    DeleteObject(Key),
    /// 先把新版本写到盘上，再在同一事务里删掉——只在单测里构造
    #[cfg_attr(not(test), allow(dead_code))]
    WriteThenDelete(Key),
    /// 一个已死的 key 重新写出（同一对象的同一锚点在 truncate 之后再写）
    Recreate(Key),
    CreateInode,
    DeleteInode(u64),
}

struct World {
    configuration: WorldConfiguration,
    random_source: RandomSource,
    disk: BTreeMap<u64, Unit>,
    next_location: u64,
    free_pool: VecDeque<u64>,
    /// 本 txg 内产生、发布后才能进 free_pool 的释放
    pending_free: Vec<u64>,
    /// defer 窗口：发布后再等 DEFER 次发布才进可分配集合（I-7.4 的 K 代保护，回退用）
    defer: VecDeque<(Txg, Vec<u64>)>,
    /// 最近几次发布的全部根视图（回退用）：(发布的 txg, 实例, 头视图, 当时的快照, 当时的祖先表)
    history: Vec<(Txg, u32, BTreeMap<Tree, View>, Vec<View>, BTreeMap<Tree, (Tree, Txg)>)>,
    /// 回退抛弃的根仍留在根环里：(txg, 实例)。下次择新按 txg 最大者——新时间线的根必须压过它们
    abandoned_roots: Vec<(Txg, u32)>,
    /// 生成器指定下一次分配复用哪个落点（模拟分配器把某个垃圾落点盖掉）
    force_reuse: Option<u64>,
    /// 主落点 → 该单元全部副本的落点（含主落点本身）
    replicas: BTreeMap<u64, Vec<u64>>,
    /// 运行时分配记录：落点 → 已分配 / 已释放(释放代)。第四版 P4：释放时不删条目，value 改成已释放 + 释放代
    allocation_records: BTreeMap<u64, AllocationState>,
    /// 只被快照引用的落点：快照销毁时才回到 free_pool
    deadlisted: BTreeSet<u64>,
    instance: u32,
    counter: u64,
    txg: Txg,
    heads: BTreeMap<Tree, Head>,
    /// 可写头的最新发布真值
    head_views: BTreeMap<Tree, View>,
    snapshots: Vec<View>,
    ancestry: BTreeMap<Tree, (Tree, Txg)>,
    /// 实例表：实例代号 → (最后发布的 txg T_pub, 在飞 txg 里最后施加的计数器 W)。
    /// 崩溃行 = (在飞 txg − 1, W)，回退行 = (T_old, 0)，中间实例 = (0, 0)。谓词见 `row_published`
    watermarks: BTreeMap<u32, (Txg, u64)>,
    /// 每个以崩溃结束的实例：(最后发布的 txg, 在飞 txg 有没有被部分重放)——inst_only 臂能拿到的全部信息
    instance_last_txg: BTreeMap<u32, (Txg, bool)>,
    /// 哪些行是回退行（第八版：不许被后来的恢复覆盖）
    rollback_rows: BTreeSet<u32>,
    /// 回退行保护实际挡下过几次写（第八轮反推腿 12.1：这条规则今天就量得出，不必等崩溃点重放）
    rollback_row_writes_blocked: u32,
    /// 冻结容器写序时用的事务号（`container_write_sequence_frozen`）
    frozen_container_counter: u64,
    /// 本 txg 里有没有已施加的事务
    applied_in_txg: bool,
    scrub_watermark: Txg,
    /// 最近一次销毁根（快照）的 txg：被它独占的旧版本从那一刻起才是垃圾
    last_root_removal_txg: Txg,
    /// 崩溃那一刻之前、本实例最后一条被施加的计数器值（恢复时写成水位）
    last_applied_counter: u64,
    /// 上一个根的实例代号（恢复时给 [它, 新实例) 都写行）
    root_instance: u32,
    next_tree: Tree,
    next_object_number: u64,
    next_inode: u64,
    // ── 生成器独立计数（闭式的分母，不从重建代码读回）──
    abandoned_unrewritten_count: u64,
    rollback_abandoned_count: u64,
    written_unit_count: u64,
    invalidated_unit_count: u64,
    overwritten_location_count: u64,
}

/// 写一行；回退行按第八版 P2 不许被后来的恢复覆盖（第七轮反推腿 9.1）。
/// ⚠️ 模型没有 journal，那条攻击真正的害处（被抛弃的记录被重放回来、回退被静默撤销）落在 E104 的度量之外，
/// 这里只实现规则、没有能分辨它的世界（见「它答不了的」）。
fn put_row(rows: &mut BTreeMap<u32, (Txg, u64)>, sticky: &BTreeSet<u32>, protect: bool, instance_number: u32, row: (Txg, u64), blocked: &mut u32) {
    if protect && sticky.contains(&instance_number) { *blocked += 1; return; }
    rows.insert(instance_number, row);
}

/// 第七版 P2 的已发布谓词（实例表那一半）：行 (T_pub, W) 说「恢复所选的根是该实例的 T_pub，从它的 tail 起连续重放、
/// 施加到事务号 W 为止」。数据单元（码 1）按写出它的事务判：b ≤ T_pub（在所选根里），或 n ≤ W（被重放施加；
/// 事务号在实例内单调、记录按事务号顺序追加，W 是前缀）；容器（码 3）只在 checkpoint 的固定点写出，
/// 所选根之后的固定点单元一律未发布、由恢复实例重写：b ≤ T_pub。
fn row_published(rows: &BTreeMap<u32, (Txg, u64)>, unit: &Unit) -> bool {
    let Some((published_txg, applied_counter)) = rows.get(&unit.write_sequence.instance) else { return true; };
    match unit.key {
        Key::Data { .. } => unit.birth <= *published_txg || unit.write_sequence.counter <= *applied_counter,
        Key::Container { .. } => unit.birth <= *published_txg,
    }
}

impl World {
    fn new(seed: u64, configuration: WorldConfiguration) -> Self {
        let mut world = World {
            configuration,
            random_source: RandomSource::new(seed),
            disk: BTreeMap::new(),
            next_location: 1_000,
            free_pool: VecDeque::new(),
            pending_free: Vec::new(),
            defer: VecDeque::new(),
            history: Vec::new(),
            abandoned_roots: Vec::new(),
            force_reuse: None,
            replicas: BTreeMap::new(),
            allocation_records: BTreeMap::new(),
            deadlisted: BTreeSet::new(),
            instance: 1,
            counter: 0,
            txg: 1,
            heads: BTreeMap::new(),
            head_views: BTreeMap::new(),
            snapshots: Vec::new(),
            ancestry: BTreeMap::new(),
            watermarks: BTreeMap::new(),
            instance_last_txg: BTreeMap::new(),
            rollback_rows: BTreeSet::new(),
            rollback_row_writes_blocked: 0,
            frozen_container_counter: 0,
            applied_in_txg: false,
            scrub_watermark: 0,
            last_root_removal_txg: 0,
            last_applied_counter: 0,
            root_instance: 1,
            next_tree: 1,
            next_object_number: 1,
            next_inode: 1,
            abandoned_unrewritten_count: 0,
            rollback_abandoned_count: 0,
            written_unit_count: 0,
            invalidated_unit_count: 0,
            overwritten_location_count: 0,
        };
        let tree = world.new_tree();
        world.heads.insert(tree, Head {
            tree: tree,
            data: BTreeMap::new(),
            containers: Vec::new(),
            open_tombstone_container: None,
            closed_tombstone_containers: Vec::new(),
            next_tombstone_container_number: 1,
        });
        world
    }

    fn new_tree(&mut self) -> Tree {
        let tree = self.next_tree;
        self.next_tree += 1;
        tree
    }

    fn allocate_location(&mut self) -> u64 {
        if let Some(location) = self.force_reuse.take() {
            self.free_pool.retain(|free_location| *free_location != location);
            if self.disk.remove(&location).is_some() { self.overwritten_location_count += 1; }
            return location;
        }
        if !self.free_pool.is_empty() && self.random_source.chance_basis_points(self.configuration.reuse_basis_points) {
            let location = self.free_pool.pop_front().unwrap();
            if self.disk.remove(&location).is_some() { self.overwritten_location_count += 1; }
            location
        } else {
            let location = self.next_location;
            self.next_location += 1;
            location
        }
    }

    fn write_unit(&mut self, key: Key, writer: Tree, payload: Payload) -> u64 {
        // 容器（码 3）在真实形态里只由 checkpoint 的固定点写出，写序的事务号是那次 checkpoint 的属性、不是单元的属性；
        // 模型按事务写容器，`container_write_sequence_frozen` 把「同一 checkpoint 内容器写序不变」这个性质手工造出来
        let counter = if self.configuration.container_write_sequence_frozen && matches!(key, Key::Container { .. }) { self.frozen_container_counter } else { self.counter };
        let write_sequence = WriteSequence { instance: self.instance, counter };
        // 码 3 的头里只有出生树，没有写者树：模型不许给重建多于头里有的信息
        let writer = match key { Key::Container { birth_tree, .. } => birth_tree, Key::Data { .. } => writer };
        // w 份副本逐字节相同（同头、同写序、同载荷），各占一个落点；主落点 = 最小的那个
        let mut new_replica_locations: Vec<u64> = (0..self.configuration.replica_count.max(1)).map(|_| self.allocate_location()).collect();
        new_replica_locations.sort();
        for location in &new_replica_locations {
            self.disk.insert(*location, Unit { key, writer, birth: self.txg, write_sequence, payload: payload.clone() });
            self.allocation_records.insert(*location, AllocationState::Allocated);
            self.written_unit_count += 1;
        }
        let primary = new_replica_locations[0];
        self.replicas.insert(primary, new_replica_locations);
        primary
    }

    /// 一个主落点的全部副本落点（没有登记的就是它自己）。
    fn replica_locations(&self, primary: u64) -> Vec<u64> {
        self.replicas.get(&primary).cloned().unwrap_or_else(|| vec![primary])
    }

    /// 某个落点此刻被哪个已发布根引用（真值口径，运行时靠分配记录 + deadlist 答）。
    fn referenced_by_root(&self, location: u64) -> bool {
        self.head_views.values().chain(self.snapshots.iter()).any(|view| view.referenced_locations.contains(&location))
    }

    /// 释放一个落点：发布之后按引用情况进 free_pool 或 deadlist。
    fn free_later(&mut self, location: u64) {
        for replica_location in self.replica_locations(location) { self.pending_free.push(replica_location); }
    }

    // ── 事务 ──

    /// 施加一个事务。`abandon` = 单元写到盘上、记录没落盘（崩溃时被抛弃的形态）。
    fn apply_transaction(&mut self, tree: Tree, operations: &[Operation], abandon: bool) {
        self.counter += 1;
        let mut head = self.heads.remove(&tree).expect("头不存在");
        let saved = head.clone();
        let mut to_free: Vec<u64> = Vec::new();
        let mut new_tombstone_records: Vec<TombstoneRecord> = Vec::new();
        let mut written_this_transaction: Vec<u64> = Vec::new();
        let write_sequence = WriteSequence { instance: self.instance, counter: self.counter };

        for operation in operations {
            match *operation {
                Operation::CreateObject => {
                    let key = Key::Data { object_number: self.next_object_number, object_birth: self.txg };
                    self.next_object_number += 1;
                    let location = self.write_unit(key, tree, Payload::Data);
                    written_this_transaction.push(location);
                    head.data.insert(key, location);
                }
                Operation::Overwrite(key) => {
                    if let Some(old_location) = head.data.get(&key).copied() {
                        let location = self.write_unit(key, tree, Payload::Data);
                        written_this_transaction.push(location);
                        head.data.insert(key, location);
                        to_free.push(old_location);
                    }
                }
                Operation::DeleteObject(key) => {
                    if let Some(old_location) = head.data.remove(&key) {
                        to_free.push(old_location);
                        new_tombstone_records.push(TombstoneRecord::Kill { key, death: write_sequence, death_txg: self.txg });
                    }
                }
                Operation::Recreate(key) => {
                    if !head.data.contains_key(&key) {
                        let location = self.write_unit(key, tree, Payload::Data);
                        written_this_transaction.push(location);
                        head.data.insert(key, location);
                    }
                }
                Operation::WriteThenDelete(key) => {
                    if let Some(old_location) = head.data.remove(&key) {
                        let location = self.write_unit(key, tree, Payload::Data);
                        // W1 第二句：这个单元在记录写出前作废（原地重写，它不在任何已发布状态里）
                        if self.configuration.invalidate_in_transaction {
                            self.disk.remove(&location);
                            self.invalidated_unit_count += 1;
                        } else {
                            written_this_transaction.push(location);
                        }
                        to_free.push(old_location);
                        new_tombstone_records.push(TombstoneRecord::Kill { key, death: write_sequence, death_txg: self.txg });
                    }
                }
                Operation::CreateInode => {
                    let inode_number = self.next_inode;
                    self.next_inode += 1;
                    self.inode_insert(&mut head, inode_number);
                }
                Operation::DeleteInode(inode_number) => {
                    self.inode_delete(&mut head, inode_number, &mut new_tombstone_records, write_sequence, &mut to_free);
                }
            }
        }

        // 脏容器重写（同一头内身份不变；跨头首次 COW 按 cfg 重生）
        for container in head.containers.iter_mut().filter(|container| container.dirty) {
            let mut key = container.key;
            if container.shared && self.configuration.rebirth {
                // 重生保留容器号、只换出生树与出生代：容器号是 inode 号、树内不复用，保得住「记录号 ≥ 容器号」与树内唯一。
                // 第六版模型发现：取「最小 inode 号」当容器号时，同一 txg 里「合并退役 (t, 2, 5, T)」之后左邻重生又得到
                // (t, 2, 5, T)——退役记录把新身份一起杀掉（mixed_w2 seed=5 神谕臂也丢 1）。
                let container_number = match container.key { Key::Container { container_number, .. } => container_number, _ => unreachable!() };
                key = Key::Container { birth_tree: tree, kind: 2, container_number, birth: self.txg };
                assert_ne!(key, container.key, "重生产出了同一个身份：tree={tree} txg={} inst={} ctr={} loc={}", self.txg, self.instance, self.counter, container.location);
                // 第二轮模型发现：新身份只覆盖冲突的 inode，覆盖不了「旧身份里有、新身份里没有」的那些
                // ⇒ 重生同时要在本头的墓碑容器里给旧身份写一条退役记录（只对本头的谱系可见）
                if self.configuration.retire_record {
                    new_tombstone_records.push(TombstoneRecord::Retire { retired_container: container.key, at_txg: self.txg });
                }
            }
            let location = self.write_unit(key, tree, Payload::Inodes(container.inode_records.clone()));
            written_this_transaction.push(location);
            if container.location != 0 { to_free.push(container.location); }
            container.key = key;
            container.location = location;
            container.dirty = false;
            container.shared = false;
        }

        // 墓碑记录追加进开放容器，容器重写一版
        if !new_tombstone_records.is_empty() {
            let (key, old_tombstone_location, mut tombstone_records) = match head.open_tombstone_container.take() {
                Some(open_tombstone) => open_tombstone,
                None => {
                    let key = Key::Container { birth_tree: tree, kind: 1, container_number: head.next_tombstone_container_number, birth: self.txg };
                    head.next_tombstone_container_number += 1;
                    (key, 0, Vec::new())
                }
            };
            tombstone_records.extend(new_tombstone_records);
            let location = self.write_unit(key, tree, Payload::Tombstones(tombstone_records.clone()));
            written_this_transaction.push(location);
            if old_tombstone_location != 0 { to_free.push(old_tombstone_location); }
            head.open_tombstone_container = Some((key, location, tombstone_records));
        }

        if abandon {
            // 记录没落盘：头的状态回滚，单元留在盘上当垃圾；释放不发生；分配记录里也没有它们（从未发布）
            for location in &written_this_transaction { for replica_location in self.replica_locations(*location) { self.allocation_records.remove(&replica_location); } }
            self.heads.insert(tree, saved);
            // 生成器独立数：这些单元的 key 若崩溃后没再被写，就是「没有水位就会被判成现行」的那批
            for location in written_this_transaction {
                if let Some(unit) = self.disk.get(&location) {
                    if matches!(unit.key, Key::Data { .. }) { self.abandoned_unrewritten_count += 1; }
                }
            }
        } else {
            self.last_applied_counter = self.counter;
            self.applied_in_txg = true;
            for location_to_free in to_free { self.free_later(location_to_free); }
            self.heads.insert(tree, head);
        }
    }

    fn inode_insert(&mut self, head: &mut Head, inode_number: u64) {
        let tree = head.tree;
        if head.containers.is_empty() || head.containers.last().unwrap().inode_records.len() >= CONTAINER_CAPACITY_IN_RECORDS {
            // 末尾分裂：右半 = 新记录起，容器号 = 新记录的 inode 号
            let key = Key::Container { birth_tree: tree, kind: 2, container_number: inode_number, birth: self.txg };
            head.containers.push(ContainerLeaf { key, location: 0, inode_records: BTreeMap::new(), dirty: true, shared: false });
        }
        let container = head.containers.last_mut().unwrap();
        container.inode_records.insert(inode_number, self.counter);
        container.dirty = true;
    }

    fn inode_delete(&mut self, head: &mut Head, inode_number: u64, new_tombstone_records: &mut Vec<TombstoneRecord>, _write_sequence: WriteSequence, to_free: &mut Vec<u64>) {
        let Some(container_index) = head.containers.iter().position(|container| container.inode_records.contains_key(&inode_number)) else { return };
        head.containers[container_index].inode_records.remove(&inode_number);
        head.containers[container_index].dirty = true;
        // 合并纪律：右半记录 ≤ 1 且左邻装得下 ⇒ 左吸收右，右退役
        if container_index > 0 && head.containers[container_index].inode_records.len() <= 1
            && head.containers[container_index - 1].inode_records.len() + head.containers[container_index].inode_records.len() <= CONTAINER_CAPACITY_IN_RECORDS
        {
            let right = head.containers.remove(container_index);
            let left = &mut head.containers[container_index - 1];
            left.inode_records.extend(right.inode_records);
            left.dirty = true;
            if right.location != 0 { to_free.push(right.location); }
            if self.configuration.retire_record {
                new_tombstone_records.push(TombstoneRecord::Retire { retired_container: right.key, at_txg: self.txg });
            }
        } else if head.containers[container_index].inode_records.is_empty() {
            let gone = head.containers.remove(container_index);
            if gone.location != 0 { to_free.push(gone.location); }
            if self.configuration.retire_record {
                new_tombstone_records.push(TombstoneRecord::Retire { retired_container: gone.key, at_txg: self.txg });
            }
        }
    }

    // ── 发布 / 快照 / 克隆 / 崩溃 / 回收 / scrub ──

    fn view_of(&self, head: &Head) -> View {
        let mut view = View { tree: head.tree, txg: self.txg, ..Default::default() };
        for (key, location) in &head.data {
            view.data.insert(*key, *location);
            for replica_location in self.replica_locations(*location) { view.referenced_locations.insert(replica_location); }
        }
        for container in &head.containers {
            for replica_location in self.replica_locations(container.location) { view.referenced_locations.insert(replica_location); }
            for (inode_number, inode_version) in &container.inode_records {
                view.inodes.insert(*inode_number, (container.key, container.location, *inode_version));
            }
        }
        if let Some((_, location, _)) = &head.open_tombstone_container { for replica_location in self.replica_locations(*location) { view.referenced_locations.insert(replica_location); } }
        for (_, location, _) in &head.closed_tombstone_containers { for replica_location in self.replica_locations(*location) { view.referenced_locations.insert(replica_location); } }
        view.containers = head.containers.iter().map(|container| (container.key, container.location)).collect();
        view.open_tombstone_container = head.open_tombstone_container.as_ref().map(|(key, location, _)| (*key, *location));
        view.closed_tombstone_containers = head.closed_tombstone_containers.iter().map(|(key, location, _)| (*key, *location)).collect();
        view.next_tombstone_container_number = head.next_tombstone_container_number;
        view
    }

    /// 从一个已发布视图重建头（回退用）：记录内容从盘上那一版读回。
    fn head_from_view(&self, view: &View) -> Head {
        let inode_records_of = |location: u64| -> BTreeMap<u64, u64> { match &self.disk[&location].payload { Payload::Inodes(inode_map) => inode_map.clone(), _ => unreachable!() } };
        let tombstone_records_of = |location: u64| -> Vec<TombstoneRecord> { match &self.disk[&location].payload { Payload::Tombstones(tombstone_list) => tombstone_list.clone(), _ => unreachable!() } };
        Head {
            tree: view.tree,
            data: view.data.clone(),
            containers: view.containers.iter().map(|(key, location)| ContainerLeaf { key: *key, location: *location, inode_records: inode_records_of(*location), dirty: false, shared: matches!(key, Key::Container { birth_tree, .. } if *birth_tree != view.tree) }).collect(),
            open_tombstone_container: view.open_tombstone_container.map(|(key, location)| (key, location, tombstone_records_of(location))),
            closed_tombstone_containers: view.closed_tombstone_containers.iter().map(|(key, location)| (*key, *location, tombstone_records_of(*location))).collect(),
            next_tombstone_container_number: view.next_tombstone_container_number,
        }
    }

    const DEFER: Txg = 2;

    fn publish(&mut self) {
        let views: Vec<View> = self.heads.values().map(|head| self.view_of(head)).collect();
        for view in views { self.head_views.insert(view.tree, view); }
        self.root_instance = self.instance;
        self.history.push((self.txg, self.instance, self.head_views.clone(), self.snapshots.clone(), self.ancestry.clone()));
        if self.history.len() > 4 { self.history.remove(0); }
        // 本 txg 的释放：发布之后再等 DEFER 次发布才进可分配集合（D16 新规则 2 + I-7.4）
        let pending = std::mem::take(&mut self.pending_free);
        for location in &pending { if !self.referenced_by_root(*location) { self.allocation_records.insert(*location, AllocationState::Freed(self.txg)); } }
        self.defer.push_back((self.txg, pending));
        while let Some((deferred_txg, _)) = self.defer.front() {
            if *deferred_txg + Self::DEFER > self.txg { break; }
            let (_, deferred_locations) = self.defer.pop_front().unwrap();
            for location in deferred_locations {
                if self.referenced_by_root(location) { self.deadlisted.insert(location); }
                else if !self.free_pool.contains(&location) && self.disk.contains_key(&location) { self.free_pool.push_back(location); }
            }
        }
        self.txg += 1;
        self.applied_in_txg = false;
    }

    /// 管理员回退到上一次发布的根：一次恢复——实例代号 +1，被抛弃的那段时间线按实例表行判未发布。
    /// 行：(r_old, T_old, 0)——旧实例发布到 T_old 为止的全部有效、T_old 之后一个都不算；(i, 0, 0)——夹在中间的实例一个都不算。
    fn rollback_to_previous(&mut self) { self.rollback(1); }

    /// 回退 `depth` 次发布（1 = 上一个根）。
    fn rollback(&mut self, depth: usize) {
        assert!(self.history.len() > depth, "回退深度超过保留的根数");
        let (old_root_txg, old_root_instance, views, snapshots, ancestry) = self.history[self.history.len() - 1 - depth].clone();
        let new_instance = self.instance + 1;
        let mut rows: Vec<(u32, (Txg, u64))> = vec![(old_root_instance, (old_root_txg, 0))];
        for instance_number in (old_root_instance + 1)..new_instance { rows.push((instance_number, (0, 0))); }
        self.instance_last_txg.insert(old_root_instance, (old_root_txg, false));
        for instance_number in (old_root_instance + 1)..new_instance { self.instance_last_txg.insert(instance_number, (0, false)); }
        if self.configuration.write_watermarks {
            for (instance_number, row) in rows { self.watermarks.insert(instance_number, row); self.rollback_rows.insert(instance_number); }
        }
        // 被抛弃时间线里写下的单元数（生成器闭式）：诞生代号 > T_old 的数据单元
        self.rollback_abandoned_count = self.disk.values().filter(|unit| unit.birth > old_root_txg && matches!(unit.key, Key::Data { .. })).count() as u64;
        self.heads = views.values().map(|view| (view.tree, self.head_from_view(view))).collect();
        self.head_views = views;
        self.snapshots = snapshots;
        self.ancestry = ancestry;
        // 分配记录从 R_old 重载：被抛弃时间线里的分配没有条目，它的释放也撤销
        let live: BTreeSet<u64> = self.head_views.values().chain(self.snapshots.iter()).flat_map(|view| view.referenced_locations.iter().copied()).collect();
        for (location, allocation_state) in self.allocation_records.iter_mut() {
            if live.contains(location) { *allocation_state = AllocationState::Allocated; }
            else if let AllocationState::Freed(freed_txg) = *allocation_state { if freed_txg > old_root_txg { *allocation_state = AllocationState::Freed(old_root_txg); } }
        }
        let abandoned_allocations: Vec<u64> = self.allocation_records.iter().filter(|(location, allocation_state)| **allocation_state == AllocationState::Allocated && !live.contains(location)).map(|(location, _)| *location).collect();
        for location in abandoned_allocations { self.allocation_records.remove(&location); }
        for (root_txg, root_instance, _, _, _) in &self.history[self.history.len() - depth..] { self.abandoned_roots.push((*root_txg, *root_instance)); }
        // 回退后第一个新根的 txg 必须压过根环里全部根（第三轮反推腿 5.2：否则下次择新挑回被抛弃的线）
        let highest_ring_txg = self.history.iter().map(|history_entry| history_entry.0).max().unwrap();
        self.history.truncate(self.history.len() - depth);
        self.pending_free.clear();
        if self.configuration.retain_abandoned_defer { self.defer.retain(|(deferred_txg, _)| *deferred_txg <= old_root_txg); }
        self.instance = new_instance;
        self.counter = 0;
        self.last_applied_counter = 0;
        self.txg = if self.configuration.rollback_skips_ring { highest_ring_txg + 1 } else { old_root_txg + 1 };
        self.publish();
    }

    fn snapshot(&mut self, tree: Tree) {
        // 快照钉在本 checkpoint 全部变更之后：先发布，再记视图
        // 快照发布之前关闭开放的墓碑容器（D18 已定项 10 按代际装载）
        let head = self.heads.get_mut(&tree).unwrap();
        if let Some(open_tombstone) = head.open_tombstone_container.take() { head.closed_tombstone_containers.push(open_tombstone); }
        self.publish();
        let snapshot_view = self.head_views[&tree].clone();
        let snapshot_view = View { txg: self.txg - 1, ..snapshot_view };
        self.snapshots.push(snapshot_view);
    }

    fn clone_head(&mut self, snapshot_index: usize) -> Tree {
        let snapshot_view = self.snapshots[snapshot_index].clone();
        let origin = self.heads[&snapshot_view.tree].clone();
        let clone_tree = self.new_tree();
        // 新头共享 origin 在快照那一刻的单元：数据 map 与容器都照抄，容器标 shared
        let mut containers: Vec<ContainerLeaf> = Vec::new();
        let mut seen: BTreeSet<Key> = BTreeSet::new();
        for (_, (container_key, container_location, _)) in &snapshot_view.inodes {
            if seen.insert(*container_key) {
                let inode_records = match &self.disk[container_location].payload { Payload::Inodes(inode_map) => inode_map.clone(), _ => unreachable!() };
                containers.push(ContainerLeaf { key: *container_key, location: *container_location, inode_records, dirty: false, shared: true });
            }
        }
        containers.sort_by_key(|container| match container.key { Key::Container { container_number, .. } => container_number, _ => 0 });
        self.heads.insert(clone_tree, Head {
            tree: clone_tree,
            data: snapshot_view.data.clone(),
            containers,
            open_tombstone_container: None,
            closed_tombstone_containers: Vec::new(),
            next_tombstone_container_number: origin.next_tombstone_container_number + 1_000,
        });
        self.ancestry.insert(clone_tree, (snapshot_view.tree, snapshot_view.txg));
        // 建克隆那次发布要把新头的视图写出来
        self.publish();
        clone_tree
    }

    /// 崩溃：本 txg 已施加的事务在恢复时被重放并发布，实例代号 +1，txg 重发。
    /// `double` = 恢复实例在写出第一个根之前又崩一次。
    fn crash_and_recover(&mut self, double: bool) {
        let crashed_instance = self.instance;
        let applied_counter = self.last_applied_counter;
        // 崩溃实例最后发布的 txg = 当前在飞 txg − 1（在飞的这个没有它的根）
        self.instance_last_txg.insert(crashed_instance, (self.txg - 1, self.applied_in_txg));
        self.instance += 1;
        self.counter = 0;
        // 第七版落地逐字：给 [所选根的实例, 新实例) 每个实例写行。恢复恒选最新已发布根 ⇒ 所选根的实例 = 崩溃实例，
        // 范围里只有它自己（`double` 那一格多一个恢复实例，见下）
        let mut rows: Vec<(u32, (Txg, u64))> = vec![(crashed_instance, (self.txg - 1, applied_counter))];
        // 崩溃实例在飞 checkpoint 的固定点单元（容器）留在盘上、一律未发布；恢复实例重放之后按自己的写序重写它们
        self.rewrite_inflight_containers(crashed_instance);
        if double {
            // 恢复实例写了一些单元（重放后的第一个根的节点）就崩了：它没有施加任何自己的记录
            let key = Key::Data { object_number: 0, object_birth: self.txg };
            self.counter += 1;
            let _ = self.write_unit(key, 1, Payload::Data);
            self.abandoned_unrewritten_count += 1;
            // 第八版：W 按实例分——恢复实例自己施加过的事务，下一次重放跨不过实例边界（jsn 断号）⇒ 它的 W 恒 0。
            // 关掉开关 = 第七版落地那句的字面：把同一个全局量写给范围里的每一个实例
            let own_applied_counter = if self.configuration.row_applied_counter_per_instance { 0 } else { applied_counter };
            rows.push((self.instance, (self.txg - 1, own_applied_counter)));
            self.instance_last_txg.insert(self.instance, (self.txg - 1, false));
            let crashed_again = self.instance;
            self.instance += 1;
            self.counter = 0;
            self.rewrite_inflight_containers(crashed_again);
        }
        if self.configuration.write_watermarks {
            let sticky = self.rollback_rows.clone();
            for (instance_number, row) in rows { put_row(&mut self.watermarks, &sticky, true, instance_number, row, &mut self.rollback_row_writes_blocked); }
        }
        self.last_applied_counter = 0;
        // 恢复发布：重放施加的事务 + 水位行 = 第一个新根（txg 号不变，就是被重发的那个）
        self.publish();
    }

    /// 恢复实例把上一个实例在在飞 checkpoint 里写出的容器版本重写一遍（新写序、同诞生代号）：
    /// 那些旧版本的诞生代号等于没发布的那个 checkpoint，按 `row_published` 一律未发布，留在盘上当孤儿。
    fn rewrite_inflight_containers(&mut self, previous_instance: u32) {
        let txg = self.txg;
        let trees: Vec<Tree> = self.heads.keys().copied().collect();
        for tree in trees {
            let mut head = self.heads.remove(&tree).unwrap();
            let stale = |disk: &BTreeMap<u64, Unit>, location: u64| location != 0 && disk.get(&location).map_or(false, |unit| unit.birth == txg && unit.write_sequence.instance == previous_instance);
            for container in head.containers.iter_mut() {
                if stale(&self.disk, container.location) {
                    let old_location = container.location;
                    container.location = self.write_unit(container.key, tree, Payload::Inodes(container.inode_records.clone()));
                    for replica_location in self.replica_locations(old_location) { self.allocation_records.remove(&replica_location); }
                }
            }
            if let Some((tombstone_key, tombstone_location, tombstone_records)) = head.open_tombstone_container.take() {
                let rewritten_tombstone_location = if stale(&self.disk, tombstone_location) {
                    let new_tombstone_location = self.write_unit(tombstone_key, tree, Payload::Tombstones(tombstone_records.clone()));
                    for replica_location in self.replica_locations(tombstone_location) { self.allocation_records.remove(&replica_location); }
                    new_tombstone_location
                } else { tombstone_location };
                head.open_tombstone_container = Some((tombstone_key, rewritten_tombstone_location, tombstone_records));
            }
            for (tombstone_key, tombstone_location, tombstone_records) in head.closed_tombstone_containers.iter_mut() {
                if stale(&self.disk, *tombstone_location) {
                    let old_location = *tombstone_location;
                    *tombstone_location = self.write_unit(*tombstone_key, tree, Payload::Tombstones(tombstone_records.clone()));
                    for replica_location in self.replica_locations(old_location) { self.allocation_records.remove(&replica_location); }
                }
            }
            self.heads.insert(tree, head);
        }
    }

    /// 第六版 P2 的行回收：kind 0 行可删 ⟺ 盘上不再有该实例按行判未发布的可读单元，
    /// 且（开关）根环里没有该实例发布的根——被抛弃的根留在环里时，它的行是回退候选判据的输入，不是垃圾的影子。
    fn reclaim_rows(&mut self) {
        let ring_instances: BTreeSet<u32> = self.history.iter().map(|history_entry| history_entry.1).chain(self.abandoned_roots.iter().map(|abandoned_root| abandoned_root.1)).collect();
        let watermark_instances: Vec<u32> = self.watermarks.keys().copied().collect();
        for instance_number in watermark_instances {
            if self.configuration.row_reclaim_checks_ring && ring_instances.contains(&instance_number) { continue; }
            let has_unpublished = self.disk.values().any(|unit| unit.write_sequence.instance == instance_number && !row_published(&self.watermarks, unit));
            if !has_unpublished { self.watermarks.remove(&instance_number); }
        }
    }

    /// 第六版 P3 的回退候选集：根环里（有效的 + 被抛弃的）按实例表判仍有效的根。真值 = history 里的根。
    fn rollback_candidates(&self) -> Vec<(Txg, u32)> {
        self.history.iter().map(|history_entry| (history_entry.0, history_entry.1)).chain(self.abandoned_roots.iter().copied())
            .filter(|(root_txg, root_instance)| match self.watermarks.get(root_instance) { None => true, Some((published_txg, _)) => *root_txg <= *published_txg })
            .collect()
    }

    /// 恢复选了非最新根（第六轮反推腿 2.1）：最新已发布根的槽自证不过，它之后的记录又已回收、不可重放
    /// ⇒ 所选根 = 上一个根，什么都重放不了，行 = (崩溃实例, 所选根的 txg, 0)。丢掉的那个 checkpoint 与在飞 checkpoint
    /// 里写下的单元全是孤儿。开关关掉时行按第六版字面写 (在飞 txg − 1, 0)，把丢掉的 checkpoint 判成已发布。
    fn crash_and_recover_lost_latest_root(&mut self) { self.crash_and_recover_lost_roots(1); }

    /// 最新的 `depth` 个根都不可读，恢复退到再往前那一个。
    fn crash_and_recover_lost_roots(&mut self, depth: usize) {
        assert!(self.history.len() > depth, "要有更早的根可选");
        let crashed_instance = self.instance;
        let mut lost = None;
        for _ in 0..depth { lost = self.history.pop(); }
        let chosen_txg = self.history.last().unwrap().0;
        let (_, _, views, snapshots, ancestry) = self.history.last().unwrap().clone();
        // 丢掉的 checkpoint 与在飞 checkpoint 里的数据单元：生成器独立计数（闭式）
        self.abandoned_unrewritten_count = self.disk.values().filter(|unit| unit.birth > chosen_txg && matches!(unit.key, Key::Data { .. })).count() as u64;
        self.instance_last_txg.insert(crashed_instance, (chosen_txg, false));
        self.instance += 1;
        self.counter = 0;
        let row_published_txg = if self.configuration.row_published_txg_from_chosen_root { chosen_txg } else { self.txg - 1 };
        // 第七版落地逐字：给 [所选根的实例, 新实例) 每个实例写行。所选根可能是更早的实例发布的
        // ⇒ 这一段会覆盖到回退行（第七轮反推腿 9.1），第八版靠 `rollback_row_sticky` 挡住
        let chosen_instance = self.history.last().unwrap().1;
        if self.configuration.write_watermarks {
            let sticky = self.rollback_rows.clone();
            for instance_number in chosen_instance..self.instance {
                put_row(&mut self.watermarks, &sticky, true, instance_number, (row_published_txg, 0), &mut self.rollback_row_writes_blocked);
            }
        }
        // 状态退回所选根：与回退同一套重载（头、快照、祖先表、分配记录、defer）
        self.heads = views.values().map(|view| (view.tree, self.head_from_view(view))).collect();
        self.head_views = views;
        self.snapshots = snapshots;
        self.ancestry = ancestry;
        let live: BTreeSet<u64> = self.head_views.values().chain(self.snapshots.iter()).flat_map(|view| view.referenced_locations.iter().copied()).collect();
        for (location, allocation_state) in self.allocation_records.iter_mut() {
            if live.contains(location) { *allocation_state = AllocationState::Allocated; }
            else if let AllocationState::Freed(freed_txg) = *allocation_state { if freed_txg > chosen_txg { *allocation_state = AllocationState::Freed(chosen_txg); } }
        }
        let abandoned_allocations: Vec<u64> = self.allocation_records.iter().filter(|(location, allocation_state)| **allocation_state == AllocationState::Allocated && !live.contains(location)).map(|(location, _)| *location).collect();
        for location in abandoned_allocations { self.allocation_records.remove(&location); }
        self.pending_free.clear();
        self.defer.retain(|(deferred_txg, _)| *deferred_txg <= chosen_txg);
        let _ = lost; // 槽坏了的根不在环里、也不是回退候选
        self.last_applied_counter = 0;
        // 恢复发布：txg 号照在飞的那个重发
        self.publish();
    }

    /// 第七轮反推腿 3.1：恢复实例重放完上一实例的事务并发布之后，又施加了自己的几个事务，然后在下一个根之前再崩。
    /// 下一次恢复的所选根仍是恢复实例发布的那个之前的那一个吗——不是：恢复实例发布过根，所选根就是它发的那个；
    /// 真正承重的是**它自己的记录跨不过实例边界**（jsn 断号）⇒ 它在下一个 checkpoint 里施加的事务一条也重放不了。
    /// 行的 W 按实例分 ⇒ 它那行 W = 0；按第七版落地那句的字面写全局量 ⇒ 它的孤儿 n ≤ W 判已发布。
    fn crash_with_own_applied(&mut self, own_transaction_count: usize, tree: Tree) {
        let recovery_instance = self.instance;
        // 上一次恢复给所选根那个实例算出来的 W（全局量就是它）
        let chosen_applied_counter = (1..recovery_instance).filter_map(|instance_number| self.watermarks.get(&instance_number).map(|row| row.1)).max().unwrap_or(0);
        for _ in 0..own_transaction_count { self.apply_transaction(tree, &[Operation::CreateObject], true); }
        self.instance_last_txg.insert(recovery_instance, (self.txg - 1, false));
        self.instance += 1;
        self.counter = 0;
        let row_applied_counter = if self.configuration.row_applied_counter_per_instance { 0 } else { chosen_applied_counter };
        if self.configuration.write_watermarks {
            let sticky = self.rollback_rows.clone();
            put_row(&mut self.watermarks, &sticky, true, recovery_instance, (self.txg - 1, row_applied_counter), &mut self.rollback_row_writes_blocked);
        }
        self.rewrite_inflight_containers(recovery_instance);
        self.last_applied_counter = 0;
        self.publish();
    }

    fn destroy_snapshot(&mut self, snapshot_index: usize) {
        let snapshot_view = self.snapshots.remove(snapshot_index);
        self.last_root_removal_txg = self.txg;
        for location in snapshot_view.referenced_locations {
            if self.deadlisted.remove(&location) && !self.referenced_by_root(location) {
                self.pending_free.push(location);
            }
        }
        // 克隆祖先表里指向它的克隆点保留（分叉点快照不许销毁，生成器不会销毁它）
    }

    /// 墓碑回收：D18 已定项 10 的条件 ∧（cfg.scrub_gate ⇒ 抹头水位 ≥ 死亡代号）。
    fn reclaim_tombstones(&mut self) {
        let snapshots = self.snapshots.clone();
        let heads = self.head_views.clone();
        let scrub_watermark = self.scrub_watermark;
        let gate = self.configuration.scrub_gate;
        let floor = if self.configuration.gate_floor_includes_root_removal { self.last_root_removal_txg } else { 0 };
        // 某个根还引用着这个容器身份的某一版吗（D18 已定项 10 的条件用在容器上）
        let container_referenced_by_root = |world: &World, container_key: &Key| -> bool {
            world.disk.iter().any(|(location, unit)| unit.key == *container_key && world.referenced_by_root(*location))
        };
        let trees: Vec<Tree> = self.heads.keys().copied().collect();
        for tree in trees {
            let mut head = self.heads.remove(&tree).unwrap();
            let mut rewrite: Vec<(Key, u64, Vec<TombstoneRecord>)> = Vec::new();
            let mut kept: Vec<(Key, u64, Vec<TombstoneRecord>)> = Vec::new();
            for (key, location, tombstone_records) in head.closed_tombstone_containers.drain(..) {
                let keep: Vec<TombstoneRecord> = tombstone_records.iter().copied().filter(|tombstone_record| match tombstone_record {
                    TombstoneRecord::Kill { key, death_txg, .. } => {
                        // D18 已定项 10 的量词 2026-09-05 从「全部还引用它的快照」放宽到「全部还引用它的根（含快照与可写头）」：
                        // 克隆头是头不是快照，凡是它里面还活着、而 origin 后来删掉的对象，按旧量词它的 kill 记录可以被回收，
                        // 而克隆引用的那具尸体一直可读（第七轮反推腿 7.2、第八轮反推腿 14.1）
                        // D18 已定项 10 的量词 2026-09-05 从「快照」放宽到「根（含快照与可写头）」。
                        // ⚠️ 模型这道门是**按 key** 判的，而规则问的是「还有没有根引用那个死掉的版本」——
                        // 两者在「先删后重建」上就分岔（同一个 key 既有 kill 记录、又被本头引用着新版本）。
                        // 所以模型分不出两个量词：加上可写头这一半只是更保守，没有能让它承重的世界（第八轮反推腿 14.1）。
                        let referenced_by_snapshot_or_head = snapshots.iter().any(|snapshot_view| snapshot_view.data.contains_key(key)) || heads.values().any(|head_view: &View| head_view.data.contains_key(key));
                        let gated = gate && scrub_watermark < (*death_txg).max(floor);
                        referenced_by_snapshot_or_head || gated
                    }
                    TombstoneRecord::Retire { retired_container, at_txg, .. } => {
                        let is_container_referenced = container_referenced_by_root(self, retired_container);
                        let gated = gate && scrub_watermark < (*at_txg).max(floor);
                        is_container_referenced || gated
                    }
                }).collect();
                if keep.len() == tombstone_records.len() { kept.push((key, location, tombstone_records)); } else { rewrite.push((key, location, keep)); }
            }
            head.closed_tombstone_containers = kept;
            for (key, old_location, keep) in rewrite {
                self.free_later(old_location);
                if !keep.is_empty() {
                    self.counter += 1;
                    let location = self.write_unit(key, tree, Payload::Tombstones(keep.clone()));
                    self.last_applied_counter = self.counter;
                    head.closed_tombstone_containers.push((key, location, keep));
                }
            }
            self.heads.insert(tree, head);
        }
    }

    /// scrub：抹掉可读、已发布、不属于任何根现行版本的单元的头；抹头水位 = 上一个已发布 txg。
    /// 神谕清扫（真值判据）：只给单测当对照，世界里不用。
    #[cfg_attr(not(test), allow(dead_code))]
    fn scrub_oracle_set(&self) -> BTreeSet<u64> {
        let deferred: BTreeSet<u64> = self.defer.iter().flat_map(|(_, deferred_locations)| deferred_locations.iter().copied()).chain(self.pending_free.iter().copied()).collect();
        self.disk.keys().copied().filter(|location| !self.referenced_by_root(*location) && !deferred.contains(location)).collect()
    }

    /// 第四版 P4 的清扫准入（全部来自运行时结构，不用 P3 的择版本）：
    /// 分配记录条目标已释放 ∧ 释放代 ≤ 环里最旧根的 txg ∧ 头可读；
    /// 孤儿（没有条目、不在在飞 overlay 里）按实例表判：所属实例 < 当前实例 ∧ 未发布。
    fn sweep_candidates(&self) -> Vec<u64> {
        let oldest_ring_txg = self.history.first().map(|history_entry| history_entry.0).unwrap_or(0);
        let inflight: BTreeSet<u64> = self.pending_free.iter().copied().collect();
        let mut candidates = Vec::new();
        for (location, unit) in &self.disk {
            if inflight.contains(location) { continue; }
            match self.allocation_records.get(location) {
                Some(AllocationState::Allocated) => {}
                Some(AllocationState::Freed(freed_txg)) => { if *freed_txg <= oldest_ring_txg { candidates.push(*location); } }
                None => {
                    if unit.write_sequence.instance < self.instance {
                        if !row_published(&self.watermarks, unit) { candidates.push(*location); }
                    }
                }
            }
        }
        candidates
    }

    fn scrub(&mut self) {
        for location in self.sweep_candidates() {
            self.disk.remove(&location);
            self.invalidated_unit_count += 1;
        }
        // 清扫水位 = 截至哪个 txg 的释放已确认不可读：规则只清得到释放代 ≤ 最旧根 txg 的落点，
        // 水位就只能推进到那里（取 txg − 1 会让 P5 的门放走还被根环保护着的旧版本）
        self.scrub_watermark = self.history.first().map(|history_entry| history_entry.0).unwrap_or(0);
    }

    fn snapshot_count(&self) -> usize { self.snapshots.len() }
}

// ── 重建 ──

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Arm {
    name: &'static str,
    /// 用真值 alloc 集当 `live` 神谕（E59 乙臂形态，阳性对照）
    oracle: bool,
    use_write_sequence: bool,
    use_watermark: bool,
    use_ancestry: bool,
    use_death_write_sequence: bool,
    use_retire: bool,
    /// 消融：写序相等时让单元赢（提案的规则是墓碑赢）
    tie_unit_wins: bool,
    /// 消融：头里只有 4 字节实例代号、没有计数器——水位只能说「实例 i 发布到了 txg T」
    instance_only: bool,
    /// 择版本之前先把同一逻辑版本的副本归并成一个候选（第三版 P3 的「副本归并」）
    merge_replicas: bool,
    /// 第八版 P3：码 3 的择新键取 (诞生代号, 实例代号)；关掉 = 第七版的「取写序最大者」
    container_key_by_birth: bool,
    /// 对立臂 B：写序只上码 3 容器，数据单元头里没有写序——数据单元只按诞生代号定序、只按诞生代号 ≤ 根 txg 判已发布
    write_sequence_metadata_only: bool,
}

const FULL: Arm = Arm { name: "full", oracle: false, use_write_sequence: true, use_watermark: true, use_ancestry: true, use_death_write_sequence: true, use_retire: true, tie_unit_wins: false, instance_only: false, merge_replicas: true, container_key_by_birth: true, write_sequence_metadata_only: false };
/// 消融：码 3 也按写序择新（第七版的规则）——同实例连着两个 checkpoint 不分配新事务号时两版写序相同 ⇒ 平局
const CONTAINER_KEY_BY_WRITE_SEQUENCE: Arm = Arm { name: "cont_key_wseq", container_key_by_birth: false, ..FULL };
const WRITE_SEQUENCE_METADATA_ONLY: Arm = Arm { name: "wseq_meta_only", write_sequence_metadata_only: true, ..FULL };
const NO_REPLICA_MERGE: Arm = Arm { name: "no_replica_merge", merge_replicas: false, ..FULL };
const INSTANCE_ONLY: Arm = Arm { name: "inst_only", instance_only: true, ..FULL };
const TIE_UNIT_WINS: Arm = Arm { name: "tie_unit_wins", tie_unit_wins: true, ..FULL };
const ORACLE: Arm = Arm { name: "oracle", oracle: true, ..FULL };
const NO_WRITE_SEQUENCE: Arm = Arm { name: "no_wseq", use_write_sequence: false, ..FULL };
const NO_WATERMARK: Arm = Arm { name: "no_watermark", use_watermark: false, ..FULL };
const NO_ANCESTRY: Arm = Arm { name: "no_ancestry", use_ancestry: false, ..FULL };
const NO_DEATH_WRITE_SEQUENCE: Arm = Arm { name: "no_death_wseq", use_death_write_sequence: false, ..FULL };
const NO_RETIRE: Arm = Arm { name: "no_retire", use_retire: false, ..FULL };
const ARMS: [Arm; 12] = [ORACLE, FULL, NO_WRITE_SEQUENCE, NO_WATERMARK, NO_ANCESTRY, NO_DEATH_WRITE_SEQUENCE, NO_RETIRE, TIE_UNIT_WINS, INSTANCE_ONLY, NO_REPLICA_MERGE, WRITE_SEQUENCE_METADATA_ONLY, CONTAINER_KEY_BY_WRITE_SEQUENCE];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct MeasureOutcome {
    roots: u64,
    scanned: u64,
    /// 真值活、重建活、落点不同
    wrong: u64,
    /// 真值死 / 不存在、重建活
    resurrected: u64,
    /// 真值活、重建死或缺
    lost: u64,
    /// 同一 key 多份候选、这一臂没有判据分出
    ambiguous: u64,
    injected: u64,
}

impl MeasureOutcome {
    fn divergence(&self) -> u64 { self.wrong + self.resurrected + self.lost + self.ambiguous }
}

struct Rebuilt {
    data: BTreeMap<Key, u64>,
    inodes: BTreeMap<u64, u64>,
    ambiguous: u64,
}

fn ancestry_of(ancestry_table: &BTreeMap<Tree, (Tree, Txg)>, tree: Tree) -> Vec<(Tree, Txg)> {
    let mut ancestor_chain = vec![(tree, u64::MAX)];
    let mut current_tree = tree;
    let mut limit = u64::MAX;
    while let Some((origin_tree, clone_txg)) = ancestry_table.get(&current_tree) {
        limit = limit.min(*clone_txg);
        ancestor_chain.push((*origin_tree, limit));
        current_tree = *origin_tree;
    }
    ancestor_chain
}

fn rebuild_root(world: &World, root: &View, arm: Arm) -> Rebuilt {
    let current_instance = world.instance;
    let current_txg = world.txg - 1;
    let ancestry_chain = ancestry_of(&world.ancestry, root.tree);
    let mut ambiguous_count = 0u64;

    let published = |unit: &Unit| -> bool {
        if !arm.use_watermark { return true; }
        // 臂 B：数据单元头里没有实例代号也没有事务号，能问的只有「诞生代号 ≤ 挂载根 txg」
        if arm.write_sequence_metadata_only && matches!(unit.key, Key::Data { .. }) { return unit.birth <= current_txg; }
        if arm.instance_only {
            // 只有实例代号：崩溃实例在它最后发布的 txg 之后写的单元一律判未发布；
            // 与那个 txg 同号的（部分重放那一格）分不出，只能判已发布
            return match world.instance_last_txg.get(&unit.write_sequence.instance) {
                None => true,
                // 在飞 txg 没有任何事务被重放 ⇒ 整个 txg 烧掉；被部分重放 ⇒ 同实例同 txg 分不出，只能判已发布
                Some((last_published_txg, is_partially_replayed)) => unit.birth <= *last_published_txg || *is_partially_replayed,
            };
        }
        if unit.write_sequence.instance < current_instance {
            row_published(&world.watermarks, unit)
        } else if unit.write_sequence.instance == current_instance {
            unit.birth <= current_txg
        } else { false }
    };
    let visible = |location: u64, unit: &Unit| -> bool {
        if arm.oracle { return root.referenced_locations.contains(&location); }
        if !published(unit) { return false; }
        if unit.birth > root.txg { return false; }
        if arm.use_ancestry {
            ancestry_chain.iter().any(|(ancestor_tree, birth_limit)| *ancestor_tree == unit.writer && unit.birth <= *birth_limit)
        } else { true }
    };

    // 每个身份的候选。副本归并：同 (key, 写者树, 诞生代号, 写序) 的多份可读单元是同一个逻辑版本的副本
    // （D2 已定项 6 的 w ≥ 2：同一份内容写 N 遍），先归并再择版本——bcachefs 的 cookie 合并是同一步。
    // 消融 no_replica_merge：不归并，每份副本各算一个候选 ⇒ 写序相等的平局全部记成歧义。
    let mut candidates: BTreeMap<Key, Vec<(u64, &Unit)>> = BTreeMap::new();
    let mut seen_versions: BTreeSet<(Key, Tree, Txg, WriteSequence)> = BTreeSet::new();
    for (location, unit) in &world.disk {
        if !visible(*location, unit) { continue; }
        if arm.merge_replicas && !seen_versions.insert((unit.key, unit.writer, unit.birth, unit.write_sequence)) {
            continue; // 同一逻辑版本的另一份副本：归并进已登记的那一份
        }
        candidates.entry(unit.key).or_default().push((*location, unit));
    }
    // 择新
    let pick = |versions: &Vec<(u64, &Unit)>, ambiguous_count: &mut u64| -> (u64, WriteSequence, Txg) {
        let data_by_birth = arm.write_sequence_metadata_only && versions.iter().all(|(_, unit)| matches!(unit.key, Key::Data { .. }));
        // 第八版 P3：择新键按单元类分——码 1 取写序；码 3 只在 checkpoint 的固定点写出，
        // 写序的事务号是 checkpoint 的属性、给不出全序，键取 (诞生代号, 实例代号)。
        // ⚠️ 模型按事务写容器（真实形态是固定点一次），同一 checkpoint 里同身份能出现多版 ⇒ 键末尾补一个写序当平局破除器，
        // **那一维是模型的补丁不是规则的一部分**：规则的前提「一个实例在一个 checkpoint 只有一个固定点」模型不满足。
        let use_container_birth_key = arm.container_key_by_birth && arm.use_write_sequence && versions.iter().all(|(_, unit)| matches!(unit.key, Key::Container { .. }));
        if use_container_birth_key {
            let highest_container_key = versions.iter().map(|(_, unit)| (unit.birth, unit.write_sequence.instance, unit.write_sequence.counter)).max().unwrap();
            let ties: Vec<&(u64, &Unit)> = versions.iter().filter(|(_, unit)| (unit.birth, unit.write_sequence.instance, unit.write_sequence.counter) == highest_container_key).collect();
            if ties.len() > 1 { *ambiguous_count += 1; }
            let (location, unit) = ties.iter().min_by_key(|(location, _)| *location).unwrap();
            (*location, unit.write_sequence, unit.birth)
        } else if arm.use_write_sequence && !data_by_birth {
            let highest_write_sequence = versions.iter().map(|(_, unit)| unit.write_sequence).max().unwrap();
            let ties: Vec<&(u64, &Unit)> = versions.iter().filter(|(_, unit)| unit.write_sequence == highest_write_sequence).collect();
            if ties.len() > 1 { *ambiguous_count += 1; }
            let (location, unit) = ties.iter().min_by_key(|(location, _)| *location).unwrap();
            (*location, unit.write_sequence, unit.birth)
        } else {
            let highest_birth = versions.iter().map(|(_, unit)| unit.birth).max().unwrap();
            let ties: Vec<&(u64, &Unit)> = versions.iter().filter(|(_, unit)| unit.birth == highest_birth).collect();
            if ties.len() > 1 { *ambiguous_count += 1; }
            let (location, unit) = ties.iter().max_by_key(|(location, _)| *location).unwrap();
            (*location, unit.write_sequence, unit.birth)
        }
    };
    let mut current: BTreeMap<Key, (u64, WriteSequence, Txg)> = BTreeMap::new();
    for (candidate_key, candidate_versions) in &candidates { current.insert(*candidate_key, pick(candidate_versions, &mut ambiguous_count)); }

    // 墓碑记录：来自可见的、现行版本的类型 1 容器
    let mut kills: BTreeMap<Key, Vec<(WriteSequence, Txg)>> = BTreeMap::new();
    let mut retired: BTreeSet<Key> = BTreeSet::new();
    for (current_key, (location, _, _)) in &current {
        if let Key::Container { kind: 1, .. } = current_key {
            if let Payload::Tombstones(tombstone_records) = &world.disk[location].payload {
                for tombstone_record in tombstone_records {
                    match tombstone_record {
                        TombstoneRecord::Kill { key, death, death_txg } => kills.entry(*key).or_default().push((*death, *death_txg)),
                        TombstoneRecord::Retire { retired_container, .. } => { if arm.use_retire { retired.insert(*retired_container); } }
                    }
                }
            }
        }
    }
    let mut data: BTreeMap<Key, u64> = BTreeMap::new();
    let mut inode_candidates: BTreeMap<u64, Vec<(WriteSequence, u64)>> = BTreeMap::new();
    for (current_key, (location, write_sequence, birth)) in &current {
        match current_key {
            Key::Data { .. } => {
                let dead = kills.get(current_key).map_or(false, |kill_entries| kill_entries.iter().any(|(kill_death_write_sequence, kill_death_txg)| {
                    if arm.use_death_write_sequence && !arm.write_sequence_metadata_only { if arm.tie_unit_wins { kill_death_write_sequence > write_sequence } else { kill_death_write_sequence >= write_sequence } } else {
                        if kill_death_txg == birth { ambiguous_count += 1; false } else { kill_death_txg > birth }
                    }
                }));
                if !dead { data.insert(*current_key, *location); }
            }
            Key::Container { kind: 2, .. } => {
                if retired.contains(current_key) { continue; }
                if let Payload::Inodes(inode_map) = &world.disk[location].payload {
                    for (inode_number, inode_version) in inode_map { inode_candidates.entry(*inode_number).or_default().push((*write_sequence, *inode_version)); }
                }
            }
            _ => {}
        }
    }
    let mut inodes: BTreeMap<u64, u64> = BTreeMap::new();
    for (inode_number, inode_candidate_versions) in inode_candidates {
        // 两个现行容器装着同一个 inode 号：规则上不该发生，发生了记成歧义（取写序大的那条）
        if inode_candidate_versions.len() > 1 { ambiguous_count += 1; }
        let (_, inode_version) = inode_candidate_versions.iter().max_by_key(|(candidate_write_sequence, _)| *candidate_write_sequence).unwrap();
        inodes.insert(inode_number, *inode_version);
    }
    Rebuilt { data, inodes, ambiguous: ambiguous_count }
}

fn measure(world: &World, arm: Arm, corrupt: usize) -> MeasureOutcome {
    let roots: Vec<&View> = world.head_views.values().chain(world.snapshots.iter()).collect();
    let mut outcome = MeasureOutcome { roots: roots.len() as u64, scanned: world.disk.len() as u64, ..Default::default() };
    let mut to_inject = corrupt;
    for root_view in roots {
        let mut rebuilt = rebuild_root(world, root_view, arm);
        outcome.ambiguous += rebuilt.ambiguous;
        // 判别力自证：只往本来对的条目上注入
        let victims: Vec<Key> = rebuilt.data.iter().filter(|(key, location)| root_view.data.get(*key) == Some(*location)).map(|(key, _)| *key).take(to_inject).collect();
        for key in victims { let location = rebuilt.data[&key]; rebuilt.data.insert(key, location + 500_000); outcome.injected += 1; to_inject -= 1; }
        for (key, location) in &root_view.data {
            match rebuilt.data.get(key) { None => outcome.lost += 1, Some(rebuilt_location) if rebuilt_location != location => outcome.wrong += 1, _ => {} }
        }
        for key in rebuilt.data.keys() { if !root_view.data.contains_key(key) { outcome.resurrected += 1; } }
        for (inode_number, (_, _, inode_version)) in &root_view.inodes {
            match rebuilt.inodes.get(inode_number) { None => outcome.lost += 1, Some(rebuilt_version) if rebuilt_version != inode_version => outcome.wrong += 1, _ => {} }
        }
        for inode_number in rebuilt.inodes.keys() { if !root_view.inodes.contains_key(inode_number) { outcome.resurrected += 1; } }
    }
    outcome
}

// ── 世界 ──

#[derive(Clone, Copy, Debug)]
struct ClosedFormCounts {
    /// 崩溃后没再被写过的被抛弃单元数（去水位臂应当把它们全判成现行）
    abandoned_unrewritten: u64,
    /// 墓碑已回收且旧版本仍可读的 key 数（去 scrub 门时应当全数复活）
    reclaimed_readable: u64,
    /// 克隆头与 origin 各写一版的 key 数（去祖先表时 origin 视野应当全错）
    cross_head_keys: u64,
    /// 同一 txg 内跨事务先删后写 / 先写后删的 key 数（只有代号时应当全部歧义）
    same_txg_pairs: u64,
    /// 同一 txg 内跨事务两次覆写的 key 数（只有代号时应当全部歧义）
    same_txg_overwrites: u64,
    /// 退役容器里、后来在左半被删掉的 inode 数（去退役记录时应当复活）
    retired_then_deleted: u64,
    /// 回退抛弃的那段时间线里写下的数据单元数（没有回退行时应当全部复活）
    rollback_abandoned: u64,
    /// w = 2 时各根引用的身份数之和（不归并副本时每个身份在每个根上各一次平局）
    live_versions: u64,
}

/// 靶子世界一：崩溃 + 部分重放 + 连崩两次。
fn world_crash(seed: u64, configuration: WorldConfiguration, double: bool) -> (World, ClosedFormCounts) {
    let mut world = World::new(seed, configuration);
    let tree = 1;
    for _ in 0..3 { world.apply_transaction(tree, &[Operation::CreateObject, Operation::CreateObject], false); }
    world.publish();
    // 在飞 txg：两个事务被重放，三个被抛弃（各建两个对象，崩溃后不再写）
    world.apply_transaction(tree, &[Operation::CreateObject], false);
    world.apply_transaction(tree, &[Operation::CreateObject], false);
    for _ in 0..3 { world.apply_transaction(tree, &[Operation::CreateObject, Operation::CreateObject], true); }
    world.crash_and_recover(double);
    // 重发的 txg 里再建一个对象，然后正常发布几轮
    world.apply_transaction(tree, &[Operation::CreateObject], false);
    world.publish();
    world.apply_transaction(tree, &[Operation::CreateObject], false);
    world.publish();
    let closed_form = ClosedFormCounts { abandoned_unrewritten: world.abandoned_unrewritten_count, ..ClosedFormCounts::default() };
    (world, closed_form)
}

/// 靶子世界一之六（第五轮反推腿 1.1）：在飞 txg 里事务 A 施加，事务 B 发出单元写之后失败，C、D 也写了单元。
/// 第六版：失败 ⇒ 实例结束，C、D 的记录一条都不追加（记录按事务号顺序），恢复行 W = A 的号 ⇒ B、C、D 全部判未发布。
/// 第五版的形态（开关关掉）：C、D 照常追加并被重放，W = D 的号 ⇒ B 的单元 n ≤ W 判已发布 ⇒ 复活。
fn world_transaction_fail(seed: u64, configuration: WorldConfiguration) -> (World, ClosedFormCounts) {
    let mut world = World::new(seed, configuration);
    let tree = 1;
    for _ in 0..3 { world.apply_transaction(tree, &[Operation::CreateObject, Operation::CreateObject], false); }
    world.publish();
    world.apply_transaction(tree, &[Operation::CreateObject], false); // A
    world.apply_transaction(tree, &[Operation::CreateObject, Operation::CreateObject], true); // B：单元在盘上，记录永远不会追加
    let later_abandoned = world.configuration.records_stop_at_failure;
    world.apply_transaction(tree, &[Operation::CreateObject], later_abandoned); // C
    world.apply_transaction(tree, &[Operation::CreateObject], later_abandoned); // D
    world.crash_and_recover(false); // 实例结束 = 下次挂载走恢复
    world.apply_transaction(tree, &[Operation::CreateObject], false);
    world.publish();
    let closed_form = ClosedFormCounts { abandoned_unrewritten: world.abandoned_unrewritten_count, ..ClosedFormCounts::default() };
    (world, closed_form)
}

/// 靶子世界一之十一（第八轮反推腿 12.1）：回退之后新时间线的根不可读，恢复落回 R_old。
/// 那次恢复要给 [所选根的实例, 新实例) 每个实例写行，范围里包含 r_old ⇒ 正好盖到回退行；第八版靠标志挡住。
/// ⚠️ 模型量得出的只有「保护确实挡下过写」这一件事：模型没有 journal，恢复算不出非 0 的 W，
/// 盖上去的值与回退行恰好相同 ⇒ 值层面看不出差别（被抛弃的记录被重放回来那一半落在 C124 里）。
fn world_rollback_row_protected(seed: u64, configuration: WorldConfiguration) -> (World, ClosedFormCounts) {
    let mut world = World::new(seed, configuration);
    let tree = 1;
    world.apply_transaction(tree, &[Operation::CreateObject; 4], false);
    world.publish();
    world.apply_transaction(tree, &[Operation::CreateObject; 3], false);
    world.publish(); // 将被回退抛弃的那次发布
    world.rollback_to_previous();
    world.apply_transaction(tree, &[Operation::CreateObject], false);
    world.publish();
    world.crash_and_recover_lost_roots(2); // 回退后的两个根都不可读 ⇒ 退回 R_old
    world.apply_transaction(tree, &[Operation::CreateObject], false);
    world.publish();
    let closed_form = ClosedFormCounts { rollback_abandoned: world.rollback_abandoned_count, ..ClosedFormCounts::default() };
    (world, closed_form)
}

/// 靶子世界一之八（第七轮反推腿 3.1）：恢复实例重放完上一实例的事务、又施加了自己的几个事务，然后在发布第一个根之前崩。
/// 下一次恢复要给 [所选根的实例, 新实例) 每个实例写行：按第八版 W 按实例分 ⇒ 恢复实例那行 W = 0（jsn 在实例边界断号，
/// 它的记录一条也重放不了）⇒ 它的单元判未发布；按第七版落地那句的字面把全局最大已施加号写给它 ⇒ 判已发布 ⇒ 复活。
fn world_recovery_own_transactions(seed: u64, configuration: WorldConfiguration) -> (World, ClosedFormCounts) {
    let mut world = World::new(seed, configuration);
    let tree = 1;
    for _ in 0..3 { world.apply_transaction(tree, &[Operation::CreateObject, Operation::CreateObject], false); }
    world.publish();
    // 实例 1 的在飞 txg：两个事务被重放
    world.apply_transaction(tree, &[Operation::CreateObject], false);
    world.apply_transaction(tree, &[Operation::CreateObject], false);
    world.crash_and_recover(false); // 实例 2 发布重放结果
    // 实例 2 在下一个 checkpoint 里施加自己的三个事务（记录已追加），然后在下一个根之前崩
    world.crash_with_own_applied(3, tree);
    world.apply_transaction(tree, &[Operation::CreateObject], false);
    world.publish();
    let closed_form = ClosedFormCounts { abandoned_unrewritten: world.abandoned_unrewritten_count, ..ClosedFormCounts::default() };
    (world, closed_form)
}

/// 靶子世界一之九（第七轮反推腿 2.1）：同一实例连着两个 checkpoint 都不分配新事务号（后台把同一个已关闭墓碑容器
/// 重写两版），容器写序逐字节相同。按写序择新是平局；按 (诞生代号, 实例代号) 择新分得开。
/// 写序冻结是生成器手工造的形状——模型按事务写容器，真实形态是固定点一次（见「它答不了的」）。
fn world_container_same_write_sequence(seed: u64, configuration: WorldConfiguration) -> (World, ClosedFormCounts) {
    let mut world = World::new(seed, WorldConfiguration { container_write_sequence_frozen: true, reuse_basis_points: 0, ..configuration });
    let tree = 1;
    world.apply_transaction(tree, &[Operation::CreateObject; 3], false);
    world.apply_transaction(tree, &[Operation::CreateInode; 2], false);
    world.publish();
    let keys: Vec<Key> = world.heads[&tree].data.keys().copied().collect();
    // 两次删除各自重写墓碑容器一版，两版之间不推进冻结的事务号 ⇒ 同身份两版、写序逐字节相同、诞生代号不同
    world.apply_transaction(tree, &[Operation::DeleteObject(keys[0])], false);
    world.publish();
    world.apply_transaction(tree, &[Operation::DeleteObject(keys[1])], false);
    world.publish();
    world.apply_transaction(tree, &[Operation::CreateObject], false);
    world.publish();
    (world, ClosedFormCounts::default())
}

/// 靶子世界一之七（第六轮反推腿 2.1）：最新已发布根的槽坏了、它之后的记录不可重放，恢复退到上一个根。
fn world_lost_root(seed: u64, configuration: WorldConfiguration) -> (World, ClosedFormCounts) {
    let mut world = World::new(seed, configuration);
    let tree = 1;
    for _ in 0..3 { world.apply_transaction(tree, &[Operation::CreateObject, Operation::CreateObject], false); }
    world.publish();
    let keys: Vec<Key> = world.heads[&tree].data.keys().copied().collect();
    world.apply_transaction(tree, &[Operation::CreateObject, Operation::CreateObject, Operation::Overwrite(keys[0])], false);
    world.publish(); // 这个根的槽将坏掉
    world.apply_transaction(tree, &[Operation::CreateObject], false); // 在飞
    world.crash_and_recover_lost_latest_root();
    world.apply_transaction(tree, &[Operation::CreateObject], false);
    world.publish();
    let closed_form = ClosedFormCounts { abandoned_unrewritten: world.abandoned_unrewritten_count, ..ClosedFormCounts::default() };
    (world, closed_form)
}

/// 靶子世界一之二：在飞 txg 里没有任何事务被重放（整个 txg 可以烧掉）。
fn world_crash_clean(seed: u64, configuration: WorldConfiguration) -> (World, ClosedFormCounts) {
    let mut world = World::new(seed, configuration);
    let tree = 1;
    for _ in 0..3 { world.apply_transaction(tree, &[Operation::CreateObject, Operation::CreateObject], false); }
    world.publish();
    for _ in 0..3 { world.apply_transaction(tree, &[Operation::CreateObject, Operation::CreateObject], true); }
    world.crash_and_recover(false);
    world.apply_transaction(tree, &[Operation::CreateObject], false);
    world.publish();
    let closed_form = ClosedFormCounts { abandoned_unrewritten: world.abandoned_unrewritten_count, ..ClosedFormCounts::default() };
    (world, closed_form)
}

/// 靶子世界一之三：管理员回退到上一个根，抛弃一段已发布的时间线。
fn world_rollback(seed: u64, configuration: WorldConfiguration) -> (World, ClosedFormCounts) {
    let mut world = World::new(seed, configuration);
    let tree = 1;
    world.apply_transaction(tree, &[Operation::CreateObject; 4], false);
    world.publish();
    let keys: Vec<Key> = world.heads[&tree].data.keys().copied().collect();
    world.apply_transaction(tree, &[Operation::Overwrite(keys[0]), Operation::Overwrite(keys[1])], false);
    world.publish();
    // 被抛弃的那次发布：建三个对象
    world.apply_transaction(tree, &[Operation::CreateObject; 3], false);
    world.publish();
    world.rollback_to_previous();
    world.apply_transaction(tree, &[Operation::CreateObject], false);
    world.publish();
    let closed_form = ClosedFormCounts { rollback_abandoned: world.rollback_abandoned_count, ..ClosedFormCounts::default() };
    (world, closed_form)
}

/// 靶子世界一之四：崩溃之后再回退到崩溃前实例的根——中间那个实例整个被抛弃，行 (i, 0, 0) 承重。
fn world_rollback_after_crash(seed: u64, configuration: WorldConfiguration) -> (World, ClosedFormCounts) {
    let mut world = World::new(seed, configuration);
    let tree = 1;
    world.apply_transaction(tree, &[Operation::CreateObject; 4], false);
    world.publish();
    world.apply_transaction(tree, &[Operation::CreateObject], false);
    world.apply_transaction(tree, &[Operation::CreateObject, Operation::CreateObject], true);
    world.crash_and_recover(false); // 实例 2 的恢复根发布了被重放的那个对象
    world.apply_transaction(tree, &[Operation::CreateObject; 3], false);
    world.publish();
    world.rollback(2); // 回到实例 1 崩溃前的根：实例 2 发布的两个根全部抛弃
    world.apply_transaction(tree, &[Operation::CreateObject], false);
    world.publish();
    let closed_form = ClosedFormCounts { rollback_abandoned: world.rollback_abandoned_count, ..ClosedFormCounts::default() };
    (world, closed_form)
}

/// 靶子世界一之五：w = 2，每个单元两份逐字节相同的副本（第三轮反推腿 2.1：第一版每个单元都破「写序全序」的字面）。
fn world_replicas(seed: u64, configuration: WorldConfiguration) -> (World, ClosedFormCounts) {
    let mut world = World::new(seed, WorldConfiguration { replica_count: 2, ..configuration });
    let tree = 1;
    world.apply_transaction(tree, &[Operation::CreateObject; 5], false);
    world.apply_transaction(tree, &[Operation::CreateInode; 3], false);
    world.publish();
    let keys: Vec<Key> = world.heads[&tree].data.keys().copied().collect();
    world.apply_transaction(tree, &[Operation::Overwrite(keys[0]), Operation::Overwrite(keys[1])], false);
    world.publish();
    world.snapshot(tree);
    let clone_tree = world.clone_head(0);
    world.apply_transaction(clone_tree, &[Operation::Overwrite(keys[2])], false);
    world.publish();
    // 闭式：逐根数它引用的身份数（没有删除 ⇒ 每个根可见的身份恰是它引用的），不归并时每个身份一次平局
    let mut referenced_identity_total = 0u64;
    for root_view in world.head_views.values().chain(world.snapshots.iter()) {
        let referenced_identities: BTreeSet<Key> = root_view.referenced_locations.iter().filter_map(|location| world.disk.get(location).map(|unit| unit.key)).collect();
        referenced_identity_total += referenced_identities.len() as u64;
    }
    let closed_form = ClosedFormCounts { live_versions: referenced_identity_total, ..ClosedFormCounts::default() };
    (world, closed_form)
}

/// 靶子世界二：删除 + 快照送走 + 墓碑回收，scrub 从未跑。
fn world_reclaim(seed: u64, configuration: WorldConfiguration) -> (World, ClosedFormCounts) {
    let mut world = World::new(seed, WorldConfiguration { reuse_basis_points: 0, ..configuration });
    let tree = 1;
    world.apply_transaction(tree, &[Operation::CreateObject; 6], false);
    world.publish();
    let keys: Vec<Key> = world.heads[&tree].data.keys().copied().collect();
    world.snapshot(tree);
    let delete_operations: Vec<Operation> = keys.iter().take(4).map(|key| Operation::DeleteObject(*key)).collect();
    world.apply_transaction(tree, &delete_operations, false);
    world.publish();
    world.snapshot(tree); // 关掉开放的墓碑容器（第二个快照仍引用它的这一版）
    world.destroy_snapshot(0); // 引用死者的那个快照走了 ⇒ 按旧规则可回收
    world.destroy_snapshot(0); // 第二个快照也走了：没有任何根再引用墓碑容器的旧版本
    world.reclaim_tombstones();
    world.publish();
    // 分配器把被回收的墓碑容器的旧版本盖掉（它的落点已在可分配集合里），死者的旧版本仍可读
    let stale_tombstone_locations: Vec<u64> = world.disk.iter().filter(|(location, unit)| matches!(unit.payload, Payload::Tombstones(_)) && !world.referenced_by_root(**location)).map(|(location, _)| *location).collect();
    for location in stale_tombstone_locations { world.force_reuse = Some(location); world.apply_transaction(tree, &[Operation::CreateObject], false); }
    world.publish();
    let readable = keys.iter().take(4).filter(|key| world.disk.values().any(|unit| unit.key == **key)).count() as u64;
    let closed_form = ClosedFormCounts { reclaimed_readable: if configuration.scrub_gate { 0 } else { readable }, ..ClosedFormCounts::default() };
    (world, closed_form)
}

/// 靶子世界三：克隆头与 origin 各自覆写同一批 key。
fn world_clone(seed: u64, configuration: WorldConfiguration) -> (World, ClosedFormCounts) {
    let mut world = World::new(seed, configuration);
    let origin_tree = 1;
    world.apply_transaction(origin_tree, &[Operation::CreateObject; 5], false);
    world.publish();
    world.snapshot(origin_tree);
    let clone_tree = world.clone_head(0);
    let keys: Vec<Key> = world.heads[&origin_tree].data.keys().copied().take(3).collect();
    let overwrite_operations: Vec<Operation> = keys.iter().map(|key| Operation::Overwrite(*key)).collect();
    world.apply_transaction(origin_tree, &overwrite_operations, false);
    world.publish();
    world.apply_transaction(clone_tree, &overwrite_operations, false);
    world.publish();
    let closed_form = ClosedFormCounts { cross_head_keys: keys.len() as u64, ..ClosedFormCounts::default() };
    (world, closed_form)
}

/// 靶子世界四：同一 txg 内跨事务的删 / 写与两次覆写。
fn world_same_txg(seed: u64, configuration: WorldConfiguration) -> (World, ClosedFormCounts) {
    let mut world = World::new(seed, configuration);
    let tree = 1;
    world.apply_transaction(tree, &[Operation::CreateObject; 6], false);
    world.publish();
    let keys: Vec<Key> = world.heads[&tree].data.keys().copied().collect();
    // k0：先删后写（两个事务）；k1：先写后删（两个事务）；k2、k3：两次覆写（两个事务）
    world.apply_transaction(tree, &[Operation::DeleteObject(keys[0])], false);
    world.apply_transaction(tree, &[Operation::Overwrite(keys[1]), Operation::Overwrite(keys[2]), Operation::Overwrite(keys[3])], false);
    world.apply_transaction(tree, &[Operation::Recreate(keys[0])], false); // k0：同一 txg 里先删后重建
    world.apply_transaction(tree, &[Operation::DeleteObject(keys[1]), Operation::Overwrite(keys[2]), Operation::Overwrite(keys[3])], false);
    world.publish();
    let closed_form = ClosedFormCounts { same_txg_pairs: count_same_txg_pairs(&world), same_txg_overwrites: count_same_birth_ties(&world), ..ClosedFormCounts::default() };
    (world, closed_form)
}

/// 生成器侧闭式：墓碑的死亡代号与该 key 某个可读单元的诞生代号相同的 (key, 死亡) 对数。
fn count_same_txg_pairs(world: &World) -> u64 {
    let mut seen: BTreeSet<(Key, WriteSequence)> = BTreeSet::new();
    for unit in world.disk.values() {
        if let Payload::Tombstones(tombstone_records) = &unit.payload {
            for tombstone_record in tombstone_records {
                if let TombstoneRecord::Kill { key, death, death_txg } = tombstone_record {
                    if world.disk.values().any(|other_unit| other_unit.key == *key && other_unit.birth == *death_txg) { seen.insert((*key, *death)); }
                }
            }
        }
    }
    seen.len() as u64
}

/// 生成器侧闭式：同一身份在它最大诞生代号上有 ≥ 2 个可读版本的身份数（含墓碑容器）。
fn count_same_birth_ties(world: &World) -> u64 {
    let mut by_key: BTreeMap<Key, Vec<Txg>> = BTreeMap::new();
    for unit in world.disk.values() { by_key.entry(unit.key).or_default().push(unit.birth); }
    by_key.values().filter(|unit_births| { let highest_birth = *unit_births.iter().max().unwrap(); unit_births.iter().filter(|birth| **birth == highest_birth).count() >= 2 }).count() as u64
}

/// 靶子世界五：容器分裂、合并退役、再删。
fn world_merge(seed: u64, configuration: WorldConfiguration) -> (World, ClosedFormCounts) {
    // 不复用释放空间：退役容器的最后一版要留在盘上可读，闭式才不是 0 == 0
    let mut world = World::new(seed, WorldConfiguration { reuse_basis_points: 0, ..configuration });
    let tree = 1;
    // 建 6 个 inode ⇒ 两片叶 [1..4]、[5,6]
    world.apply_transaction(tree, &[Operation::CreateInode; 6], false);
    world.publish();
    // 删 1、2、3 ⇒ 左半剩 [4]；删 6 ⇒ 右半剩 [5]，合并：左吸收右 ⇒ [4,5]，右容器退役
    world.apply_transaction(tree, &[Operation::DeleteInode(1), Operation::DeleteInode(2), Operation::DeleteInode(3)], false);
    world.publish();
    world.apply_transaction(tree, &[Operation::DeleteInode(6)], false);
    world.publish();
    // 再删 5 ⇒ 5 曾住在退役容器里，去退役记录时它会从退役容器的旧版本里复活
    world.apply_transaction(tree, &[Operation::DeleteInode(5)], false);
    world.publish();
    // 闭式：退役容器最后一版里、在头的真值里已经不在的 inode 数（5 与 6）
    let alive: BTreeSet<u64> = world.head_views[&tree].inodes.keys().copied().collect();
    let live_container_keys: BTreeSet<Key> = world.heads[&tree].containers.iter().map(|container| container.key).collect();
    let mut dead_in_retired = 0u64;
    for unit in world.disk.values() {
        if let (Key::Container { kind: 2, .. }, Payload::Inodes(inode_map)) = (unit.key, &unit.payload) {
            if !live_container_keys.contains(&unit.key) { dead_in_retired += inode_map.keys().filter(|inode_number| !alive.contains(inode_number)).count() as u64; }
        }
    }
    let closed_form = ClosedFormCounts { retired_then_deleted: dead_in_retired, ..ClosedFormCounts::default() };
    (world, closed_form)
}

/// 靶子世界六：旧版本被快照钉住，scrub 在快照销毁之前跑过，之后回收墓碑。
fn world_snapshot_after_scrub(seed: u64, configuration: WorldConfiguration) -> (World, ClosedFormCounts) {
    let mut world = World::new(seed, WorldConfiguration { reuse_basis_points: 0, ..configuration });
    let tree = 1;
    world.apply_transaction(tree, &[Operation::CreateObject; 6], false);
    world.publish();
    let keys: Vec<Key> = world.heads[&tree].data.keys().copied().collect();
    world.snapshot(tree); // S1 钉住六个旧版本
    let delete_operations: Vec<Operation> = keys.iter().take(4).map(|key| Operation::DeleteObject(*key)).collect();
    world.apply_transaction(tree, &delete_operations, false);
    world.publish();
    world.snapshot(tree); // S2：关掉墓碑容器
    world.destroy_snapshot(1); // S2 走了，S1 还钉着死者的旧版本
    for _ in 0..5 { world.publish(); } // 让死亡代号落到根环窗口之外，清扫水位才推得过它
    world.scrub(); // 抹头水位越过死亡代号，但死者的旧版本被 S1 钉着、没被抹
    world.apply_transaction(tree, &[Operation::CreateObject], false);
    world.publish();
    world.destroy_snapshot(0); // S1 走了：四个旧版本从此是可读垃圾
    world.reclaim_tombstones(); // 第一版的门：抹头水位 ≥ 死亡代号 ⇒ 放行
    world.publish();
    let stale_tombstone_locations: Vec<u64> = world.disk.iter().filter(|(location, unit)| matches!(unit.payload, Payload::Tombstones(_)) && !world.referenced_by_root(**location)).map(|(location, _)| *location).collect();
    for location in stale_tombstone_locations { world.force_reuse = Some(location); world.apply_transaction(tree, &[Operation::CreateObject], false); }
    world.publish();
    let readable = keys.iter().take(4).filter(|key| world.disk.values().any(|unit| unit.key == **key)).count() as u64;
    let closed_form = ClosedFormCounts { reclaimed_readable: if configuration.gate_floor_includes_root_removal { 0 } else { readable }, ..ClosedFormCounts::default() };
    (world, closed_form)
}

/// 混合随机世界：覆写 / 删除 / 快照 / 克隆 / 崩溃 / 回收 / scrub 全开。
fn world_mixed(seed: u64, configuration: WorldConfiguration, rounds: u32) -> (World, ClosedFormCounts) {
    let mut world = World::new(seed, configuration);
    world.apply_transaction(1, &[Operation::CreateObject; 8], false);
    world.apply_transaction(1, &[Operation::CreateInode; 6], false);
    world.publish();
    for round in 0..rounds {
        let trees: Vec<Tree> = world.heads.keys().copied().collect();
        let transaction_count = 1 + world.random_source.below(3);
        let mut crash_at: Option<u64> = None;
        if world.random_source.chance_basis_points(1500) { crash_at = Some(world.random_source.below(transaction_count)); }
        for transaction_index in 0..transaction_count {
            let tree = trees[world.random_source.below(trees.len() as u64) as usize];
            let mut operations: Vec<Operation> = Vec::new();
            let keys: Vec<Key> = world.heads[&tree].data.keys().copied().collect();
            let inode_numbers: Vec<u64> = world.heads[&tree].containers.iter().flat_map(|container| container.inode_records.keys().copied()).collect();
            for _ in 0..(1 + world.random_source.below(4)) {
                match world.random_source.below(6) {
                    0 => operations.push(Operation::CreateObject),
                    1 | 2 if !keys.is_empty() => operations.push(Operation::Overwrite(keys[world.random_source.below(keys.len() as u64) as usize])),
                    3 if !keys.is_empty() => operations.push(Operation::DeleteObject(keys[world.random_source.below(keys.len() as u64) as usize])),
                    4 => operations.push(Operation::CreateInode),
                    5 if !inode_numbers.is_empty() => operations.push(Operation::DeleteInode(inode_numbers[world.random_source.below(inode_numbers.len() as u64) as usize])),
                    _ => operations.push(Operation::CreateObject),
                }
            }
            // W1：同一事务里同一 key 只留最后一条净效果
            let mut seen: BTreeSet<Key> = BTreeSet::new();
            let mut seen_inode_numbers: BTreeSet<u64> = BTreeSet::new();
            operations.retain(|operation| match operation {
                Operation::Overwrite(key) | Operation::DeleteObject(key) | Operation::WriteThenDelete(key) | Operation::Recreate(key) => seen.insert(*key),
                Operation::DeleteInode(inode_number) => seen_inode_numbers.insert(*inode_number),
                _ => true,
            });
            let abandon = crash_at.map_or(false, |crash_index| transaction_index >= crash_index);
            world.apply_transaction(tree, &operations, abandon);
        }
        if crash_at.is_some() {
            let double = world.random_source.chance_basis_points(2000);
            world.crash_and_recover(double);
        } else {
            world.publish();
        }
        match world.random_source.below(10) {
            0 | 1 => { let tree = trees[world.random_source.below(trees.len() as u64) as usize]; world.snapshot(tree); }
            2 if world.snapshot_count() > 0 && world.heads.len() < 4 => { let snapshot_index = world.random_source.below(world.snapshot_count() as u64) as usize; world.clone_head(snapshot_index); }
            3 if world.snapshot_count() > 0 => {
                let snapshot_index = world.random_source.below(world.snapshot_count() as u64) as usize;
                let snapshot_view = &world.snapshots[snapshot_index];
                let is_fork = world.ancestry.values().any(|(origin_tree, clone_txg)| *origin_tree == snapshot_view.tree && *clone_txg == snapshot_view.txg);
                if !is_fork { world.destroy_snapshot(snapshot_index); }
            }
            4 => { world.reclaim_tombstones(); world.publish(); }
            5 if round % 3 == 0 => { world.scrub(); }
            6 if round % 7 == 3 && world.history.len() >= 2 => { world.rollback_to_previous(); }
            _ => {}
        }
    }
    (world, ClosedFormCounts::default())
}

impl Default for ClosedFormCounts {
    fn default() -> Self {
        ClosedFormCounts { abandoned_unrewritten: 0, reclaimed_readable: 0, cross_head_keys: 0, same_txg_pairs: 0, same_txg_overwrites: 0, retired_then_deleted: 0, rollback_abandoned: 0, live_versions: 0 }
    }
}

fn emit_cell(emitter: &mut Emitter, world_name: &str, seed: u64, arm: Arm, measured: MeasureOutcome, closed_form: ClosedFormCounts) -> String {
    emitter.emit_raw(&format!(
        "name=cell world={world_name} seed={seed} arm={} roots={} scanned={} wrong={} resurrected={} lost={} ambiguous={} divergence={} \
         cf_abandoned={} cf_reclaimed={} cf_cross={} cf_pairs={} cf_ow={} cf_retired={} cf_rollback={} cf_versions={}",
        arm.name, measured.roots, measured.scanned, measured.wrong, measured.resurrected, measured.lost, measured.ambiguous, measured.divergence(),
        closed_form.abandoned_unrewritten, closed_form.reclaimed_readable, closed_form.cross_head_keys, closed_form.same_txg_pairs, closed_form.same_txg_overwrites, closed_form.retired_then_deleted, closed_form.rollback_abandoned, closed_form.live_versions))
}

fn main() {
    let mut emitter = Emitter::new();
    println!("{}", emitter.emit_raw(&format!(
        "name=config note=扫描重建的现行版本判定 cont_cap={CONTAINER_CAPACITY_IN_RECORDS} arms={} model=counting file_ops=0", ARMS.len())));
    let seeds = [1u64, 2, 3];
    for &seed in &seeds {
        let worlds: Vec<(&str, World, ClosedFormCounts)> = vec![
            { let (world, closed_form) = world_crash(seed, WorldConfiguration::FULL, false); ("crash", world, closed_form) },
            { let (world, closed_form) = world_crash(seed, WorldConfiguration::FULL, true); ("crash_double", world, closed_form) },
            { let (world, closed_form) = world_crash_clean(seed, WorldConfiguration::FULL); ("crash_clean", world, closed_form) },
            { let (world, closed_form) = world_transaction_fail(seed, WorldConfiguration::FULL); ("txn_fail", world, closed_form) },
            { let (world, closed_form) = world_transaction_fail(seed, WorldConfiguration { records_stop_at_failure: false, ..WorldConfiguration::FULL }); ("txn_fail_records_continue", world, closed_form) },
            { let (world, closed_form) = world_lost_root(seed, WorldConfiguration::FULL); ("lost_root", world, closed_form) },
            { let (world, closed_form) = world_lost_root(seed, WorldConfiguration { row_published_txg_from_chosen_root: false, ..WorldConfiguration::FULL }); ("lost_root_tpub_inflight", world, closed_form) },
            { let (world, closed_form) = world_recovery_own_transactions(seed, WorldConfiguration::FULL); ("recovery_own_txns", world, closed_form) },
            { let (world, closed_form) = world_recovery_own_transactions(seed, WorldConfiguration { row_applied_counter_per_instance: false, ..WorldConfiguration::FULL }); ("recovery_own_txns_global_w", world, closed_form) },
            { let (world, closed_form) = world_container_same_write_sequence(seed, WorldConfiguration::FULL); ("cont_same_wseq", world, closed_form) },
            { let (world, closed_form) = world_rollback_row_protected(seed, WorldConfiguration::FULL); ("rollback_row_protected", world, closed_form) },
            { let (world, closed_form) = world_rollback(seed, WorldConfiguration::FULL); ("rollback", world, closed_form) },
            { let (world, closed_form) = world_rollback_after_crash(seed, WorldConfiguration::FULL); ("rollback_after_crash", world, closed_form) },
            { let (world, closed_form) = world_replicas(seed, WorldConfiguration::FULL); ("replicas", world, closed_form) },
            { let (world, closed_form) = world_reclaim(seed, WorldConfiguration { scrub_gate: false, ..WorldConfiguration::FULL }); ("reclaim_nogate", world, closed_form) },
            { let (world, closed_form) = world_reclaim(seed, WorldConfiguration::FULL); ("reclaim_gate", world, closed_form) },
            { let (world, closed_form) = world_snapshot_after_scrub(seed, WorldConfiguration { gate_floor_includes_root_removal: false, ..WorldConfiguration::FULL }); ("snap_after_scrub_gate_v1", world, closed_form) },
            { let (world, closed_form) = world_snapshot_after_scrub(seed, WorldConfiguration::FULL); ("snap_after_scrub_gate_v2", world, closed_form) },
            { let (world, closed_form) = world_clone(seed, WorldConfiguration::FULL); ("clone", world, closed_form) },
            { let (world, closed_form) = world_same_txg(seed, WorldConfiguration::FULL); ("same_txg", world, closed_form) },
            { let (world, closed_form) = world_merge(seed, WorldConfiguration::FULL); ("merge", world, closed_form) },
            { let (world, closed_form) = world_mixed(seed, WorldConfiguration { gate_floor_includes_root_removal: false, ..WorldConfiguration::FULL }, 60); ("mixed_gate_v1", world, closed_form) },
            { let (world, closed_form) = world_mixed(seed, WorldConfiguration::FULL, 60); ("mixed", world, closed_form) },
        ];
        for (name, world, closed_form) in &worlds {
            for arm in ARMS {
                let outcome = measure(world, arm, 0);
                println!("{}", emit_cell(&mut emitter, name, seed, arm, outcome, *closed_form));
            }
        }
    }
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_worlds(seed: u64) -> Vec<(&'static str, World, ClosedFormCounts)> {
        vec![
            { let (world, closed_form) = world_crash(seed, WorldConfiguration::FULL, false); ("crash", world, closed_form) },
            { let (world, closed_form) = world_crash(seed, WorldConfiguration::FULL, true); ("crash_double", world, closed_form) },
            { let (world, closed_form) = world_crash_clean(seed, WorldConfiguration::FULL); ("crash_clean", world, closed_form) },
            { let (world, closed_form) = world_transaction_fail(seed, WorldConfiguration::FULL); ("txn_fail", world, closed_form) },
            { let (world, closed_form) = world_lost_root(seed, WorldConfiguration::FULL); ("lost_root", world, closed_form) },
            { let (world, closed_form) = world_recovery_own_transactions(seed, WorldConfiguration::FULL); ("recovery_own_txns", world, closed_form) },
            { let (world, closed_form) = world_container_same_write_sequence(seed, WorldConfiguration::FULL); ("cont_same_wseq", world, closed_form) },
            { let (world, closed_form) = world_rollback_row_protected(seed, WorldConfiguration::FULL); ("rollback_row_protected", world, closed_form) },
            { let (world, closed_form) = world_rollback(seed, WorldConfiguration::FULL); ("rollback", world, closed_form) },
            { let (world, closed_form) = world_rollback_after_crash(seed, WorldConfiguration::FULL); ("rollback_after_crash", world, closed_form) },
            { let (world, closed_form) = world_replicas(seed, WorldConfiguration::FULL); ("replicas", world, closed_form) },
            { let (world, closed_form) = world_mixed(seed, WorldConfiguration { replica_count: 2, ..WorldConfiguration::FULL }, 40); ("mixed_w2", world, closed_form) },
            { let (world, closed_form) = world_reclaim(seed, WorldConfiguration::FULL); ("reclaim_gate", world, closed_form) },
            { let (world, closed_form) = world_snapshot_after_scrub(seed, WorldConfiguration::FULL); ("snap_after_scrub_gate_v2", world, closed_form) },
            { let (world, closed_form) = world_clone(seed, WorldConfiguration::FULL); ("clone", world, closed_form) },
            { let (world, closed_form) = world_same_txg(seed, WorldConfiguration::FULL); ("same_txg", world, closed_form) },
            { let (world, closed_form) = world_merge(seed, WorldConfiguration::FULL); ("merge", world, closed_form) },
            { let (world, closed_form) = world_mixed(seed, WorldConfiguration::FULL, 60); ("mixed", world, closed_form) },
        ]
    }

    /// 第一版的门（抹头水位 ≥ 死亡代号）在「快照销毁晚于 scrub」的世界上放走墓碑 ⇒ 全规则臂复活；
    /// 第二版的门（下限取最近一次根销毁的代号）⇒ 0。钉闭式。
    #[test]
    fn first_version_gate_lets_snapshot_held_garbage_resurrect() {
        let (world, closed_form) = world_snapshot_after_scrub(1, WorldConfiguration { gate_floor_includes_root_removal: false, ..WorldConfiguration::FULL });
        assert_eq!(closed_form.reclaimed_readable, 4);
        let outcome = measure(&world, FULL, 0);
        assert_eq!(outcome.resurrected, closed_form.reclaimed_readable, "{outcome:?}");
        let (second_world, _) = world_snapshot_after_scrub(1, WorldConfiguration::FULL);
        assert_eq!(measure(&second_world, FULL, 0).divergence(), 0);
    }

    /// 判据 1：全规则臂在全部世界、全部种子上四个数全 0。
    #[test]
    fn full_rule_is_exact_everywhere() {
        for seed in 1u64..=24 {
            for (name, world, _) in all_worlds(seed) {
                let outcome = measure(&world, FULL, 0);
                assert_eq!(outcome.divergence(), 0, "{name} seed={seed}: {outcome:?}");
                assert!(outcome.roots >= 1 && outcome.scanned >= 1, "{name}: 没扫到东西");
            }
        }
    }

    /// 判据 3：神谕臂恒 0（阳性对照）。
    #[test]
    fn oracle_is_exact_everywhere() {
        for seed in 1u64..=24 {
            for (name, world, _) in all_worlds(seed) {
                let outcome = measure(&world, ORACLE, 0);
                assert_eq!(outcome.divergence(), 0, "{name} seed={seed}: {outcome:?}");
            }
        }
    }

    /// 判据 3：扫到的单元数 = 生成器独立记的（写出 − 被盖掉 − 被抹掉）。
    #[test]
    fn scan_count_matches_generator() {
        for seed in 1u64..=12 {
            for (name, world, _) in all_worlds(seed) {
                let expect = world.written_unit_count - world.overwritten_location_count - world.invalidated_unit_count;
                let outcome = measure(&world, FULL, 0);
                assert_eq!(outcome.scanned, expect, "{name} seed={seed}");
            }
        }
    }

    /// 判据 3：注入 k 处破坏必须报出恰 k。
    #[test]
    fn comparator_has_discriminating_power() {
        let (world, _) = world_mixed(7, WorldConfiguration::FULL, 40);
        for injection_count in [1usize, 3, 5] {
            let outcome = measure(&world, FULL, injection_count);
            assert_eq!(outcome.injected, injection_count as u64);
            assert_eq!(outcome.wrong, injection_count as u64, "注入 {injection_count} 处只报出 {}", outcome.wrong);
        }
    }

    /// 判据 2：去掉实例水位 ⇒ 被抛弃且崩溃后没再写过的单元全部被判成现行。钉闭式。
    #[test]
    fn ablation_watermark_pins_abandoned_count() {
        for double in [false, true] {
            let (world, closed_form) = world_crash(1, WorldConfiguration::FULL, double);
            assert!(closed_form.abandoned_unrewritten >= 6, "靶子世界至少要有 6 个被抛弃单元");
            let outcome = measure(&world, NO_WATERMARK, 0);
            // 被抛弃的单元对每个根都可见（birth = 重发的 txg ≤ 根 txg），每个根各复活一次
            assert_eq!(outcome.resurrected, closed_form.abandoned_unrewritten * outcome.roots, "double={double}: {outcome:?}");
            assert_eq!(measure(&world, FULL, 0).divergence(), 0);
        }
    }

    /// 反推腿 1.4 的拆账：头里只有实例代号、没有计数器时，整个 txg 被抛弃的那一格分得出，
    /// 部分重放那一格分不出——被抛弃单元与已重放单元同实例同 txg。钉闭式。
    #[test]
    fn instance_only_fails_exactly_on_partial_replay() {
        let (world, closed_form) = world_crash_clean(1, WorldConfiguration::FULL);
        assert!(closed_form.abandoned_unrewritten >= 6);
        assert_eq!(measure(&world, INSTANCE_ONLY, 0).divergence(), 0, "整 txg 被抛弃：实例代号 + 最后发布 txg 就够");
        assert_eq!(measure(&world, FULL, 0).divergence(), 0);
        let (second_world, second_closed_form) = world_crash(1, WorldConfiguration::FULL, false);
        let outcome = measure(&second_world, INSTANCE_ONLY, 0);
        assert_eq!(outcome.resurrected, second_closed_form.abandoned_unrewritten * outcome.roots, "部分重放：{outcome:?}");
    }

    /// 第二轮正推腿：管理员回退到旧根是一次恢复。回退行缺席 ⇒ 被抛弃时间线里的单元全部复活。钉闭式。
    #[test]
    fn rollback_rows_pin_abandoned_timeline() {
        let (world, closed_form) = world_rollback(1, WorldConfiguration::FULL);
        assert_eq!(closed_form.rollback_abandoned, 3, "被抛弃的那次发布建了三个对象");
        assert_eq!(measure(&world, FULL, 0).divergence(), 0);
        let outcome = measure(&world, NO_WATERMARK, 0);
        assert_eq!(outcome.resurrected, closed_form.rollback_abandoned * outcome.roots, "{outcome:?}");
        let (second_world, _) = world_rollback(1, WorldConfiguration { write_watermarks: false, ..WorldConfiguration::FULL });
        let second_outcome = measure(&second_world, FULL, 0);
        assert_eq!(second_outcome.resurrected, closed_form.rollback_abandoned * second_outcome.roots, "不写回退行：{second_outcome:?}");
    }

    /// 回退必须丢掉被抛弃时间线的 defer：那些「释放」从未发生，旧根还引用着它们。
    /// 不丢的话两次发布之后它们进可分配集合、被复用，旧根的单元被盖掉——连神谕臂都会丢。
    #[test]
    fn rollback_discards_abandoned_frees() {
        let mut world = World::new(2, WorldConfiguration { reuse_basis_points: 10_000, ..WorldConfiguration::FULL });
        world.apply_transaction(1, &[Operation::CreateObject; 4], false);
        world.publish();
        let keys: Vec<Key> = world.heads[&1].data.keys().copied().collect();
        world.apply_transaction(1, &[Operation::Overwrite(keys[0])], false);
        world.publish();
        // 被抛弃的那次发布覆写 k2、k3：它们的旧落点进了 defer
        world.apply_transaction(1, &[Operation::Overwrite(keys[2]), Operation::Overwrite(keys[3])], false);
        world.publish();
        world.rollback_to_previous();
        // 再发布几次让 defer 释放，然后以 100% 复用建对象：若被抛弃的释放没丢，旧根的 k2 / k3 落点会被盖掉
        for _ in 0..3 { world.apply_transaction(1, &[Operation::CreateObject; 2], false); world.publish(); }
        let alive_locations: Vec<u64> = world.head_views[&1].data.values().copied().collect();
        assert!(alive_locations.iter().all(|location| world.disk.contains_key(location)), "旧根引用的落点被复用盖掉了");
        assert_eq!(measure(&world, ORACLE, 0).divergence(), 0);
        assert_eq!(measure(&world, FULL, 0).divergence(), 0);
    }

    /// 跨过一次崩溃回退：中间实例的行 (i, 0, 0) 承重——它发布过的根被整个抛弃。钉闭式。
    #[test]
    fn rollback_across_crash_pins_intermediate_instance() {
        let (world, closed_form) = world_rollback_after_crash(1, WorldConfiguration::FULL);
        // 被抛弃的：崩溃 txg 里被重放的 1 个 + 实例 2 后来建的 3 个 + 崩溃时被抛弃但没再写过的 2 个
        assert_eq!(closed_form.rollback_abandoned, 6, "{closed_form:?}");
        assert_eq!(measure(&world, FULL, 0).divergence(), 0);
        let outcome = measure(&world, NO_WATERMARK, 0);
        assert_eq!(outcome.resurrected, closed_form.rollback_abandoned * outcome.roots, "{outcome:?}");
    }

    /// 第三轮反推腿 5.2：回退后的新根必须压过根环里被抛弃的根，否则下次择新（按 txg 最大者）挑回被抛弃的线。
    #[test]
    fn rollback_new_root_outranks_abandoned_roots() {
        for skip in [true, false] {
            let (world, _) = world_rollback_after_crash(1, WorldConfiguration { rollback_skips_ring: skip, ..WorldConfiguration::FULL });
            let newest_live = world.history.iter().map(|history_entry| history_entry.0).max().unwrap();
            let newest_abandoned = world.abandoned_roots.iter().map(|abandoned_root| abandoned_root.0).max().unwrap();
            if skip {
                assert!(newest_live > newest_abandoned, "新时间线的根 {newest_live} 没压过被抛弃的根 {newest_abandoned}");
            } else {
                // 只回退一格时 T_old + 1 恰好盖掉被抛弃的第一个根；回退两格就压不过第二个
                assert!(newest_live <= newest_abandoned, "取 T_old + 1 本该压不过被抛弃的根");
            }
        }
    }

    /// 第四轮反推腿 6.3：对立臂 B（写序只上码 3、数据单元只有诞生代号）的残余错误钉成闭式。
    /// 崩溃部分重放那一格：数据孤儿的诞生代号 == 重发后已发布的 txg，只靠诞生代号判不出 ⇒ 全部复活；
    /// 同 txg 跨事务改同一数据 key ⇒ 歧义；容器那一格由写序解掉。
    #[test]
    fn write_sequence_metadata_only_residual_is_pinned() {
        let (world, closed_form) = world_crash(1, WorldConfiguration::FULL, false);
        let outcome = measure(&world, WRITE_SEQUENCE_METADATA_ONLY, 0);
        assert_eq!(outcome.resurrected, closed_form.abandoned_unrewritten * outcome.roots, "崩溃孤儿：{outcome:?}");
        let (second_world, second_closed_form) = world_same_txg(1, WorldConfiguration::FULL);
        let second_outcome = measure(&second_world, WRITE_SEQUENCE_METADATA_ONLY, 0);
        // same_txg 世界里同诞生代号多版的身份是 3（两个数据 key + 墓碑容器），臂 B 解掉容器那一个；
        // 同 txg 里删与写的两对（先删后重建、先写后删）只有诞生代号时都排不了序 ⇒ 各记一次歧义，
        // 其中先写后删那一对被判成活的（复活 1）
        assert_eq!(second_outcome.ambiguous, (second_closed_form.same_txg_overwrites - 1) + second_closed_form.same_txg_pairs, "同 txg：{second_outcome:?}");
        assert_eq!(second_outcome.resurrected, 1, "先写后删那一对：{second_outcome:?}");
        // 没有崩溃、没有同 txg 多版的世界上臂 B 与全规则臂一样对
        for (name, third_world, _) in [("clone", world_clone(1, WorldConfiguration::FULL).0, ()), ("merge", world_merge(1, WorldConfiguration::FULL).0, ())] {
            assert_eq!(measure(&third_world, WRITE_SEQUENCE_METADATA_ONLY, 0).divergence(), 0, "{name}");
        }
    }

    /// 第三轮反推腿 7.4：清扫按规则（已释放标志 + 释放代 ≤ 最旧根 txg；孤儿按实例表）而不是真值。
    /// 规则清掉的 ⊆ 真值清掉的，且从不碰任何一个根环里的根引用的落点。
    #[test]
    fn rule_sweep_is_sound_against_oracle() {
        let mut cleared_total = 0u64;
        for seed in 1u64..=12 {
            let (world, _) = world_mixed(seed, WorldConfiguration::FULL, 60);
            let rule: BTreeSet<u64> = world.sweep_candidates().into_iter().collect();
            let oracle = world.scrub_oracle_set();
            assert!(rule.is_subset(&oracle), "seed={seed}: 规则清掉了真值不许清的落点 {:?}", rule.difference(&oracle).take(3).collect::<Vec<_>>());
            let ring_referenced_locations: BTreeSet<u64> = world.history.iter().flat_map(|history_entry| history_entry.2.values().chain(history_entry.3.iter()).flat_map(|view| view.referenced_locations.iter().copied())).collect();
            assert!(rule.is_disjoint(&ring_referenced_locations), "seed={seed}: 规则碰了根环里某个根引用的落点");
            cleared_total += rule.len() as u64;
        }
        assert!(cleared_total > 0, "十二个种子一个落点都没清到：规则没在工作");
    }

    /// 等价性留档：回退时不丢被抛弃时间线的 defer，释放时的引用检查同样挡得住复用。
    #[test]
    fn abandoned_defer_is_also_guarded_by_reference_check() {
        for keep in [true, false] {
            let mut world = World::new(2, WorldConfiguration { reuse_basis_points: 10_000, retain_abandoned_defer: keep, ..WorldConfiguration::FULL });
            world.apply_transaction(1, &[Operation::CreateObject; 4], false);
            world.publish();
            let keys: Vec<Key> = world.heads[&1].data.keys().copied().collect();
            world.apply_transaction(1, &[Operation::Overwrite(keys[0])], false);
            world.publish();
            world.apply_transaction(1, &[Operation::Overwrite(keys[2]), Operation::Overwrite(keys[3])], false);
            world.publish();
            world.rollback_to_previous();
            for _ in 0..3 { world.apply_transaction(1, &[Operation::CreateObject; 2], false); world.publish(); }
            assert_eq!(measure(&world, ORACLE, 0).divergence(), 0, "keep={keep}");
            assert_eq!(measure(&world, FULL, 0).divergence(), 0, "keep={keep}");
        }
    }

    /// 第三轮反推腿 2.1：w = 2 下每个逻辑版本都有两份同写序的可读副本。
    /// 先归并副本再择版本 ⇒ 全规则臂 0；不归并 ⇒ 每个活着的逻辑版本一次平局。钉闭式。
    #[test]
    fn replica_merge_is_load_bearing() {
        let (world, closed_form) = world_replicas(1, WorldConfiguration::FULL);
        assert!(closed_form.live_versions >= 10, "{closed_form:?}");
        assert_eq!(measure(&world, FULL, 0).divergence(), 0);
        assert_eq!(measure(&world, ORACLE, 0).divergence(), 0);
        let outcome = measure(&world, NO_REPLICA_MERGE, 0);
        assert_eq!(outcome.ambiguous, closed_form.live_versions, "{outcome:?}");
        // 混合世界开 w = 2 同样全 0
        for seed in 1u64..=8 {
            let (second_world, _) = world_mixed(seed, WorldConfiguration { replica_count: 2, ..WorldConfiguration::FULL }, 40);
            assert_eq!(measure(&second_world, FULL, 0).divergence(), 0, "mixed_w2 seed={seed}");
        }
    }

    /// 第五轮反推腿 1.1：事务在发出单元写之后失败。第六版让它成为实例结束、后续记录不追加 ⇒ 行的 W 是前缀、全规则臂 0；
    /// 第五版的形态（后续事务照常追加、作废信息随崩溃丢失）⇒ 失败事务的单元 n ≤ W 判已发布 ⇒ 复活。钉闭式。
    #[test]
    fn failed_transaction_after_write_is_pinned() {
        let (world, closed_form) = world_transaction_fail(1, WorldConfiguration::FULL);
        assert_eq!(closed_form.abandoned_unrewritten, 4, "B 两个 + C、D 各一个都没能追加记录");
        assert_eq!(measure(&world, FULL, 0).divergence(), 0);
        assert_eq!(measure(&world, ORACLE, 0).divergence(), 0);
        let (second_world, second_closed_form) = world_transaction_fail(1, WorldConfiguration { records_stop_at_failure: false, ..WorldConfiguration::FULL });
        assert_eq!(second_closed_form.abandoned_unrewritten, 2, "只有 B 的两个单元没有记录");
        let outcome = measure(&second_world, FULL, 0);
        assert_eq!(outcome.resurrected, second_closed_form.abandoned_unrewritten * outcome.roots, "作废信息丢失：{outcome:?}");
    }

    /// 第八轮反推腿 12.1：回退行不许被后来的恢复覆盖这条规则，今天在模型里就量得出「保护确实挡下过写」——
    /// 不必等崩溃点重放。关掉保护 ⇒ 挡下次数归零（值层面看不出差别：模型没有 journal，恢复算不出非 0 的 W）。
    #[test]
    fn rollback_row_protection_actually_blocks_a_write() {
        // 就地造，好在回退那一刻把行的值抓下来
        let mut world = World::new(1, WorldConfiguration::FULL);
        let tree = 1;
        world.apply_transaction(tree, &[Operation::CreateObject; 4], false);
        world.publish();
        world.apply_transaction(tree, &[Operation::CreateObject; 3], false);
        world.publish();
        world.rollback_to_previous();
        let row_at_rollback = world.watermarks[&1];
        assert!(world.rollback_rows.contains(&1), "回退行登记在实例 1 上");
        assert_eq!(row_at_rollback.1, 0, "回退行的 W 恒 0");
        world.apply_transaction(tree, &[Operation::CreateObject], false);
        world.publish();
        world.crash_and_recover_lost_roots(2); // 恢复落回 R_old：范围 [R_old 的实例, 新实例) 正好盖到回退行
        assert!(world.rollback_row_writes_blocked >= 1, "本该盖到回退行、被保护挡下：{}", world.rollback_row_writes_blocked);
        assert_eq!(world.watermarks[&1], row_at_rollback, "回退行的三段一个字节没变");
        world.apply_transaction(tree, &[Operation::CreateObject], false);
        world.publish();
        assert_eq!(measure(&world, FULL, 0).divergence(), 0);
        assert_eq!(measure(&world, ORACLE, 0).divergence(), 0);
        assert_eq!(world.rollback_abandoned_count, 3, "被回退抛弃的那次发布建了三个对象");
    }

    /// 第七轮反推腿 3.1：恢复实例施加过自己的事务之后再崩。W 按实例分 ⇒ 它那行 W = 0、它的单元判未发布；
    /// 按第七版落地那句的字面把全局最大已施加号写给它 ⇒ 它的单元 n ≤ W 判已发布 ⇒ 复活。钉闭式。
    #[test]
    fn recovery_instance_row_applied_counter_is_per_instance() {
        let (world, closed_form) = world_recovery_own_transactions(1, WorldConfiguration::FULL);
        assert_eq!(closed_form.abandoned_unrewritten, 3, "恢复实例自己写的 3 个单元跨不过实例边界");
        assert_eq!(measure(&world, FULL, 0).divergence(), 0);
        assert_eq!(measure(&world, ORACLE, 0).divergence(), 0);
        let (second_world, second_closed_form) = world_recovery_own_transactions(1, WorldConfiguration { row_applied_counter_per_instance: false, ..WorldConfiguration::FULL });
        let outcome = measure(&second_world, FULL, 0);
        assert_eq!(outcome.resurrected, second_closed_form.abandoned_unrewritten * outcome.roots, "全局 W 写给恢复实例：{outcome:?}");
    }

    /// 第七轮反推腿 2.1：同实例连着两个 checkpoint 都不分配新事务号时，两版容器的写序逐字节相同。
    /// 码 3 按 (诞生代号, 实例代号) 择新 ⇒ 0；按写序择新 ⇒ 平局记歧义。钉闭式。
    #[test]
    fn container_key_by_birth_breaks_the_write_sequence_tie() {
        let (world, _) = world_container_same_write_sequence(1, WorldConfiguration::FULL);
        let versions: Vec<(Key, Txg, WriteSequence)> = world.disk.values().filter(|unit| matches!(unit.key, Key::Container { kind: 1, .. })).map(|unit| (unit.key, unit.birth, unit.write_sequence)).collect();
        let same_write_sequence_pairs = versions.iter().enumerate().flat_map(|(first_index, first_version)| versions[first_index + 1..].iter().map(move |second_version| (first_version, second_version)))
            .filter(|(first_version, second_version)| first_version.0 == second_version.0 && first_version.2 == second_version.2 && first_version.1 != second_version.1).count();
        assert!(same_write_sequence_pairs >= 1, "靶子世界要造出同身份、同写序、不同诞生代号的两版：{versions:?}");
        assert_eq!(measure(&world, FULL, 0).divergence(), 0);
        assert_eq!(measure(&world, ORACLE, 0).divergence(), 0);
        let outcome = measure(&world, CONTAINER_KEY_BY_WRITE_SEQUENCE, 0);
        assert_eq!(outcome.ambiguous, same_write_sequence_pairs as u64, "按写序择新的平局数：{outcome:?}");
    }

    /// 第六轮反推腿 2.1：恢复选了非最新根时，行的 T_pub 必须是所选根的 txg。按第六版字面取「在飞 txg − 1」
    /// 会把槽坏掉的那个 checkpoint 里的单元判成已发布 ⇒ 复活；取所选根 txg ⇒ 0。钉闭式。
    #[test]
    fn lost_root_row_uses_chosen_root_txg() {
        let (world, closed_form) = world_lost_root(1, WorldConfiguration::FULL);
        assert_eq!(closed_form.abandoned_unrewritten, 4, "丢掉的 checkpoint 建 2 覆写 1 + 在飞建 1");
        assert_eq!(measure(&world, FULL, 0).divergence(), 0);
        assert_eq!(measure(&world, ORACLE, 0).divergence(), 0);
        let (second_world, second_closed_form) = world_lost_root(1, WorldConfiguration { row_published_txg_from_chosen_root: false, ..WorldConfiguration::FULL });
        let outcome = measure(&second_world, FULL, 0);
        // 丢掉的 checkpoint 里的 3 个单元 b = T_pub 判已发布（其中覆写那一版压过旧版：复活 2 + 错 1）；在飞那 1 个 b = T_pub + 1、W = 0 仍未发布
        assert_eq!(outcome.resurrected + outcome.wrong, second_closed_form.abandoned_unrewritten - 1, "{outcome:?}");
    }

    /// 第五轮反推腿 4.1：回退之后被抛弃的根留在环里；清扫把它独占的单元抹掉之后，若 kind 0 行按「没有未发布的可读单元」
    /// 就回收，被抛弃的根整批回到回退候选集。第六版加合取「根环里没有该实例的根」⇒ 行留着、候选集正确。钉闭式。
    #[test]
    fn row_reclaim_keeps_abandoned_roots_out_of_candidates() {
        for checks_ring in [true, false] {
            let mut world = World::new(4, WorldConfiguration { row_reclaim_checks_ring: checks_ring, reuse_basis_points: 0, ..WorldConfiguration::FULL });
            world.apply_transaction(1, &[Operation::CreateObject; 4], false);
            world.publish();
            world.apply_transaction(1, &[Operation::CreateObject; 2], false);
            world.publish();
            world.apply_transaction(1, &[Operation::CreateObject; 3], false);
            world.publish(); // 将被抛弃的根
            world.rollback_to_previous();
            let truth: BTreeSet<(Txg, u32)> = world.history.iter().map(|history_entry| (history_entry.0, history_entry.1)).collect();
            assert_eq!(world.abandoned_roots.len(), 1);
            let before: BTreeSet<(Txg, u32)> = world.rollback_candidates().into_iter().collect();
            assert_eq!(before, truth, "回退刚做完：被抛弃的根不在候选集里");
            world.scrub(); // 抹掉被抛弃时间线独占的单元
            world.reclaim_rows();
            let after: BTreeSet<(Txg, u32)> = world.rollback_candidates().into_iter().collect();
            let wrong = after.difference(&truth).count();
            if checks_ring { assert_eq!(wrong, 0, "{after:?}"); assert!(world.watermarks.contains_key(&1)); }
            else { assert_eq!(wrong, world.abandoned_roots.len(), "行被回收后被抛弃的根回到候选集：{after:?}"); }
            assert_eq!(measure(&world, FULL, 0).divergence(), 0, "checks_ring={checks_ring}");
        }
    }

    /// 第五轮反推腿 2.2：容器在 checkpoint 固定点写出，一个没发布的 checkpoint 的容器版本一律未发布，
    /// 恢复实例重写它们。崩溃世界里每个被重放事务碰过的容器都留下一版旧实例的孤儿，清扫按行把它们清掉。
    #[test]
    fn inflight_containers_are_rewritten_by_recovery() {
        let mut world = World::new(6, WorldConfiguration { reuse_basis_points: 0, ..WorldConfiguration::FULL });
        world.apply_transaction(1, &[Operation::CreateInode; 3], false);
        world.publish();
        world.apply_transaction(1, &[Operation::CreateInode], false); // 在飞：被重放
        world.apply_transaction(1, &[Operation::CreateInode], true);  // 在飞：被抛弃
        world.crash_and_recover(false);
        let inflight_txg = world.txg - 1;
        let stale: Vec<u64> = world.disk.iter().filter(|(_, unit)| matches!(unit.key, Key::Container { .. }) && unit.birth == inflight_txg && unit.write_sequence.instance == 1).map(|(location, _)| *location).collect();
        assert!(stale.len() >= 2, "至少一版被重放事务写的容器 + 一版被抛弃事务写的容器留在盘上：{}", stale.len());
        assert!(stale.iter().all(|location| !row_published(&world.watermarks, &world.disk[location])), "旧实例在飞 checkpoint 的容器一律未发布");
        let swept: BTreeSet<u64> = world.sweep_candidates().into_iter().collect();
        assert!(stale.iter().all(|location| swept.contains(location)), "它们是孤儿，清扫要清掉");
        assert_eq!(measure(&world, FULL, 0).divergence(), 0);
        assert_eq!(measure(&world, ORACLE, 0).divergence(), 0);
    }

    /// 判据 2：写路径不写水位行 ⇒ 全规则臂也救不了（承重的是行，不是规则）。
    #[test]
    fn missing_watermark_rows_break_full_rule() {
        let (world, closed_form) = world_crash(1, WorldConfiguration { write_watermarks: false, ..WorldConfiguration::FULL }, false);
        let outcome = measure(&world, FULL, 0);
        assert_eq!(outcome.resurrected, closed_form.abandoned_unrewritten * outcome.roots, "{outcome:?}");
    }

    /// 判据 2：去掉 scrub 门 ⇒ 墓碑回收之后仍可读的旧版本全数复活；门开着则 0。钉闭式。
    #[test]
    fn ablation_scrub_gate_pins_resurrection() {
        let (world, closed_form) = world_reclaim(1, WorldConfiguration { scrub_gate: false, ..WorldConfiguration::FULL });
        assert_eq!(closed_form.reclaimed_readable, 4, "四个死者的旧版本都还可读");
        let outcome = measure(&world, FULL, 0);
        assert_eq!(outcome.resurrected, closed_form.reclaimed_readable, "{outcome:?}");
        let (second_world, _) = world_reclaim(1, WorldConfiguration::FULL);
        assert_eq!(measure(&second_world, FULL, 0).divergence(), 0);
    }

    /// 判据 2：去掉祖先表 ⇒ origin 头视野里克隆头写的更新版本被当成现行。钉闭式。
    #[test]
    fn ablation_ancestry_pins_cross_head_wrong() {
        let (world, closed_form) = world_clone(1, WorldConfiguration::FULL);
        assert_eq!(closed_form.cross_head_keys, 3);
        let outcome = measure(&world, NO_ANCESTRY, 0);
        // origin 头 A 的 3 个 key 错到 B 的版本；快照 S 靠「诞生代号 ≤ 根 txg」挡住；B 自己对
        assert_eq!(outcome.wrong, closed_form.cross_head_keys, "{outcome:?}");
        assert_eq!(measure(&world, FULL, 0).divergence(), 0);
    }

    /// 判据 2：只有代号没有写序 ⇒ 同一 txg 里两次覆写分不出。钉闭式。
    #[test]
    fn ablation_write_sequence_pins_same_txg_ambiguity() {
        let (world, closed_form) = world_same_txg(1, WorldConfiguration::FULL);
        assert_eq!(closed_form.same_txg_overwrites, 3, "两个数据 key 各两次覆写 + 墓碑容器自己的两版");
        let outcome = measure(&world, NO_WRITE_SEQUENCE, 0);
        assert_eq!(outcome.ambiguous, closed_form.same_txg_overwrites, "{outcome:?}");
        assert_eq!(measure(&world, FULL, 0).divergence(), 0);
    }

    /// 判据 2：墓碑只有死亡代号没有死亡写序 ⇒ 同一 txg 里的删 / 写分不出。钉闭式。
    #[test]
    fn ablation_death_write_sequence_pins_same_txg_pairs() {
        let (world, closed_form) = world_same_txg(1, WorldConfiguration::FULL);
        assert_eq!(closed_form.same_txg_pairs, 2, "先删后重建一对、先写后删一对");
        let outcome = measure(&world, NO_DEATH_WRITE_SEQUENCE, 0);
        assert_eq!(outcome.ambiguous, closed_form.same_txg_pairs, "{outcome:?}");
    }

    /// 判据 2：去掉容器退役记录 ⇒ 退役容器里后来被删的 inode 复活。钉闭式。
    #[test]
    fn ablation_retire_pins_inode_resurrection() {
        let (world, closed_form) = world_merge(1, WorldConfiguration::FULL);
        assert_eq!(closed_form.retired_then_deleted, 2, "退役容器最后一版里 5 与 6 都已死");
        let outcome = measure(&world, NO_RETIRE, 0);
        assert_eq!(outcome.resurrected, closed_form.retired_then_deleted, "{outcome:?}");
        assert_eq!(measure(&world, FULL, 0).divergence(), 0);
        // 写路径不写退役记录 ⇒ 全规则臂同样复活（承重的是记录）
        let (second_world, _) = world_merge(1, WorldConfiguration { retire_record: false, ..WorldConfiguration::FULL });
        assert_eq!(measure(&second_world, FULL, 0).resurrected, closed_form.retired_then_deleted);
    }

    /// 事务内先写后删：作废与否全规则臂都对（墓碑赢平局）；让单元赢平局的消融臂在不作废时复活。
    #[test]
    fn write_then_delete_tie_rule() {
        for is_invalidated_in_transaction in [true, false] {
            let mut world = World::new(3, WorldConfiguration { invalidate_in_transaction: is_invalidated_in_transaction, ..WorldConfiguration::FULL });
            world.apply_transaction(1, &[Operation::CreateObject; 3], false);
            world.publish();
            let keys: Vec<Key> = world.heads[&1].data.keys().copied().collect();
            world.apply_transaction(1, &[Operation::WriteThenDelete(keys[0]), Operation::WriteThenDelete(keys[1])], false);
            world.publish();
            assert_eq!(measure(&world, FULL, 0).divergence(), 0, "inval={is_invalidated_in_transaction}");
            let outcome = measure(&world, TIE_UNIT_WINS, 0);
            if is_invalidated_in_transaction { assert_eq!(outcome.divergence(), 0, "{outcome:?}"); } else { assert_eq!(outcome.resurrected, 2, "{outcome:?}"); }
        }
    }

    /// 跨头首次 COW 不重生 ⇒ origin 视野里看到克隆写的同身份新版本。
    #[test]
    fn rebirth_is_load_bearing() {
        for rebirth in [true, false] {
            let mut world = World::new(5, WorldConfiguration { rebirth, ..WorldConfiguration::FULL });
            world.apply_transaction(1, &[Operation::CreateInode; 3], false);
            world.publish();
            world.snapshot(1);
            let clone_tree = world.clone_head(0);
            world.apply_transaction(clone_tree, &[Operation::CreateInode], false);
            world.publish();
            let outcome = measure(&world, FULL, 0);
            if rebirth { assert_eq!(outcome.divergence(), 0, "{outcome:?}"); } else { assert!(outcome.resurrected >= 1, "{outcome:?}"); }
        }
    }

    /// 祖先表：克隆的克隆与旁支克隆各自只看自己那条链。
    #[test]
    fn ancestry_chain_limits() {
        let mut ancestry_table = BTreeMap::new();
        ancestry_table.insert(2, (1, 10));
        ancestry_table.insert(3, (2, 20));
        ancestry_table.insert(4, (1, 10));
        assert_eq!(ancestry_of(&ancestry_table, 3), vec![(3, u64::MAX), (2, 20), (1, 10)]);
        assert_eq!(ancestry_of(&ancestry_table, 4), vec![(4, u64::MAX), (1, 10)]);
        assert_eq!(ancestry_of(&ancestry_table, 1), vec![(1, u64::MAX)]);
    }

    /// 调试用：打印神谕臂在混合世界上对不上的根与 key。
    #[test]
    #[ignore]
    fn debug_mixed_oracle() {
        let seed: u64 = std::env::var("E104_SEED").ok().and_then(|environment_text| environment_text.parse().ok()).unwrap_or(1);
        let replica_count_from_environment: u8 = std::env::var("E104_W").ok().and_then(|environment_text| environment_text.parse().ok()).unwrap_or(1);
        let rounds: u32 = std::env::var("E104_ROUNDS").ok().and_then(|environment_text| environment_text.parse().ok()).unwrap_or(60);
        let (world, _) = world_mixed(seed, WorldConfiguration { replica_count: replica_count_from_environment, ..WorldConfiguration::FULL }, rounds);
        println!("inst={} txg={} watermarks={:?} ancestry={:?} scrub_wm={}", world.instance, world.txg, world.watermarks, world.ancestry, world.scrub_watermark);
        let roots: Vec<&View> = world.head_views.values().chain(world.snapshots.iter()).collect();
        let arm = if std::env::var("E104_ARM").ok().as_deref() == Some("full") { FULL } else { ORACLE };
        for root_view in roots {
            let rebuilt = rebuild_root(&world, root_view, arm);
            for (root_key, location) in &root_view.data {
                match rebuilt.data.get(root_key) { None => println!("root tree={} txg={} LOST {root_key:?} loc={location} on_disk={} unit={:?} in_alloc={}", root_view.tree, root_view.txg, world.disk.contains_key(location), world.disk.get(location).map(|unit| (unit.key, unit.writer, unit.birth, unit.write_sequence)), root_view.referenced_locations.contains(location)), Some(rebuilt_location) if rebuilt_location != location => println!("WRONG {root_key:?}"), _ => {} }
            }
            for (resurrected_key, location) in &rebuilt.data { if !root_view.data.contains_key(resurrected_key) {
                println!("root tree={} txg={} RESURRECTED {resurrected_key:?} at loc={location} unit={:?}", root_view.tree, root_view.txg, world.disk.get(location).map(|unit| (unit.writer, unit.birth, unit.write_sequence)));
                for (disk_location, unit) in &world.disk { if unit.key == *resurrected_key { println!("    version loc={disk_location} writer={} birth={} wseq={:?} referenced={}", unit.writer, unit.birth, unit.write_sequence, world.referenced_by_root(*disk_location)); } }
                for (disk_location, unit) in &world.disk { if let Payload::Tombstones(tombstone_records) = &unit.payload { for tombstone_record in tombstone_records { if let TombstoneRecord::Kill { key, death, death_txg } = tombstone_record { if key == resurrected_key { println!("    tomb in loc={disk_location} cont={:?} birth={} wseq={:?} referenced={} death={death:?} death_txg={death_txg}", unit.key, unit.birth, unit.write_sequence, world.referenced_by_root(*disk_location)); } } } } }
                for (head_tree, head) in world.heads.iter() { if let Some((container_key, container_location, tombstone_records)) = &head.open_tombstone_container { if tombstone_records.iter().any(|tombstone_record| matches!(tombstone_record, TombstoneRecord::Kill { key, .. } if key == resurrected_key)) { println!("    open tomb of head {head_tree}: {container_key:?} loc={container_location}"); } } for (container_key, container_location, tombstone_records) in &head.closed_tombstone_containers { if tombstone_records.iter().any(|tombstone_record| matches!(tombstone_record, TombstoneRecord::Kill { key, .. } if key == resurrected_key)) { println!("    closed tomb of head {head_tree}: {container_key:?} loc={container_location}"); } } }
            } }
            for (inode_number, (container_key, container_location, inode_version)) in &root_view.inodes {
                match rebuilt.inodes.get(inode_number) { None => {
                    println!("root tree={} txg={} LOST inode {inode_number} cont={container_key:?} loc={container_location} on_disk={} unit={:?} in_alloc={}", root_view.tree, root_view.txg, world.disk.contains_key(container_location), world.disk.get(container_location).map(|unit| (unit.key, unit.birth, unit.write_sequence)), root_view.referenced_locations.contains(container_location));
                    for (disk_location, unit) in &world.disk { if unit.key == *container_key { println!("    cont version loc={disk_location} birth={} wseq={:?} referenced={} in_root_alloc={} recs={:?}", unit.birth, unit.write_sequence, world.referenced_by_root(*disk_location), root_view.referenced_locations.contains(disk_location), match &unit.payload { Payload::Inodes(inode_map) => inode_map.keys().collect::<Vec<_>>(), _ => vec![] }); } }
                    for (disk_location, unit) in &world.disk { if let Payload::Tombstones(tombstone_records) = &unit.payload { for tombstone_record in tombstone_records { if let TombstoneRecord::Retire { retired_container, at_txg } = tombstone_record { if retired_container == container_key { println!("    retire rec in loc={disk_location} tomb={:?} birth={} wseq={:?} referenced={} in_root_alloc={} at_txg={at_txg}", unit.key, unit.birth, unit.write_sequence, world.referenced_by_root(*disk_location), root_view.referenced_locations.contains(disk_location)); } } } } }
                    for (head_tree, head) in world.heads.iter() { for container in &head.containers { if container.key == *container_key { println!("    head {head_tree} cont loc={} dirty={} shared={}", container.location, container.dirty, container.shared); } } }
                }, Some(rebuilt_version) if rebuilt_version != inode_version => println!("WRONG inode {inode_number}"), _ => {} }
            }
            for inode_number in rebuilt.inodes.keys() { if !root_view.inodes.contains_key(inode_number) {
                println!("root tree={} txg={} RESURRECTED inode {inode_number}", root_view.tree, root_view.txg);
                if root_view.txg == 79 && root_view.tree == 1 {
                    for (disk_location, unit) in &world.disk { if let Payload::Inodes(inode_map) = &unit.payload { if inode_map.contains_key(inode_number) { println!("    cont version loc={disk_location} key={:?} writer={} birth={} wseq={:?} referenced={} live_cont={}", unit.key, unit.writer, unit.birth, unit.write_sequence, world.referenced_by_root(*disk_location), world.heads.values().any(|head| head.containers.iter().any(|container| container.key == unit.key))); } } }
                    for (disk_location, unit) in &world.disk { if let Payload::Tombstones(tombstone_records) = &unit.payload { for tombstone_record in tombstone_records { if let TombstoneRecord::Retire { retired_container, at_txg } = tombstone_record { if world.disk.values().any(|other_unit| other_unit.key == *retired_container && matches!(&other_unit.payload, Payload::Inodes(inode_map) if inode_map.contains_key(inode_number))) { println!("    retire rec in loc={disk_location} cont={retired_container:?} at_txg={at_txg} referenced={}", world.referenced_by_root(*disk_location)); } } } } }
                }
            } }
        }
    }

    /// 确定性：同一种子两次跑逐字节相同。
    #[test]
    fn deterministic() {
        let first_measurement = measure(&world_mixed(9, WorldConfiguration::FULL, 50).0, NO_WRITE_SEQUENCE, 0);
        let second_measurement = measure(&world_mixed(9, WorldConfiguration::FULL, 50).0, NO_WRITE_SEQUENCE, 0);
        assert_eq!(first_measurement, second_measurement);
    }
}
