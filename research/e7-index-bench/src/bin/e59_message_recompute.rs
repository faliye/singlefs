//! E59：缓冲里那条消息能不能只从单元重算出来 —— D11 已定项 4（跑这个实验时它还未定）。
//!
//! **它答的是一个「态别」问题，不是性能问题。**
//! D11 已定项 1（2026-08-31 用户定案）已经定了「留消息缓冲区」、已定项 2 定了 ε=0.65。
//! 剩下的生死前置逐字是：
//!
//! > 缓冲里那条消息能不能只从单元重算出来。
//! > 能 ⇒ 缓冲仍是纯派生态；不能 ⇒ 它已晋升为权威态，
//! > D21 的「权威态 = 单元 + 记账 + 根」必须扩项。
//!
//! ## 被引用条款逐字贴在这里（verify-before-claiming.md：不许照印象建模）
//!
//! - D18 已定项 3（2026-08-29 用户定案）：每个数据单元带一个明文自描述头，字段取
//!   `(单元类型标签, 树 ID, 对象 ID, 对象出生代, 锚点偏移)`。**禁放清单**：物理落点 /
//!   设备 ID、文件名 / 分隔 key。
//!   ⇒ 头里**没有**「这一次写是第几次写」——`对象出生代` 是对象的出生代，不是本次写的序号。
//! - D18 已定项 1：谱系重写序号「要不要做、宽度多少」**未定**。
//! - D20：单元自包含 =「我是谁、我属于谁、我是第几代」。
//! - D21（权威态与派生态的分界）（2026-08-28 用户定案）：权威态 = 单元 + 记账 + 根；索引是派生态。
//! - D8 已定项 1（2026-08-29 用户定案）：条目形态取**幂等完整值**（含 tombstone），不是增量 Δ。
//! - D11 已定项 2：16 KiB 节点、ε=0.65 ⇒ 扇出 119、缓冲 665 条。
//!
//! ## 判据（跑前写死，跑完不许改）
//!
//! 甲臂（只从单元）四类消息的分歧数：
//!
//! | 观测 | 判 |
//! |---|---|
//! | 四类全 0 | **能重算**：缓冲是纯派生态，D11 已定项 4 答「能」 |
//! | 部分类 > 0 | **按类拆**：只有那几类晋升权威态，D21 扩项限于它们 |
//! | 四类全 > 0 | **不能重算**：D21 必须扩项 |
//! | 陈旧副本不可见时全 0、可见时 > 0 | 结论**条件于「陈旧副本的可见性」**，那条前提要单列成欠账 |
//!
//! ## 怎样才算失败（跑前写死）
//!
//! | 条款 | 触发时怎么判 |
//! |---|---|
//! | 扫到的单元数 ≠ 独立算出的应有数 | **整轮作废**：扫描没在读盘（读不到 ≠ 读到 0） |
//! | 丁臂（序号 + 显式墓碑单元）任一类 ≠ 0 | **整轮作废**：全对照都闭不上 ⇒ 模型另有 bug |
//! | 往任一臂的重建结果注入 k 处破坏，报出的 `wrong` ≠ k | **整轮作废**：比较器没有判别力 |
//! | 驻留容量算出来是 0 | **整轮作废**：没测到「消息驻留在缓冲里」这件事 |
//! | 结论方向与 D11 已定项 1 相左（例如答「不能」） | **如实记录。** 已定项 1 自陈是「在两个已知未决之上做的裁决」 |
//!
//! ## 它测的是什么、不是什么
//!
//! 测的是**一条消息的内容能不能由扫单元重新产生**，与它此刻躺在哪个节点无关。
//! 所以模型不重建整棵 Bε 树，只按 D11 已定项 2 的几何算出「全树缓冲能扣住多少条」，
//! 用它划出驻留窗口。**几何只决定窗口大小，不决定某一类消息可不可重算。**

use e7_index_bench::Emitter;
use std::collections::{BTreeMap, BTreeSet};

// ── D11 已定项 2 的几何。改这三个数要同步改 kb（gate.d/27-format-constants.sh）──
const MESSAGE_BUFFER_DECISION_NODE_BYTES: usize = 16384;
const MESSAGE_BUFFER_DECISION_FANOUT: usize = 119;
const MESSAGE_BUFFER_DECISION_BUFFERED_MESSAGES: usize = 665;
/// ε 以万分比记，0.65 ⇒ 6500。
const MESSAGE_BUFFER_DECISION_EPSILON_BASIS_POINTS: usize = 6500;
// 只有几何自洽那条断言用得到它们（把 ε 反算回 119 / 665）。
#[cfg(test)] const MESSAGE_BYTES: usize = 16;
#[cfg(test)] const PIVOT_BYTES: usize = 48;
#[cfg(test)] const NODE_HEADER_BYTES: usize = 16;

/// 单元的自描述头。字段集逐字取 D18 已定项 3 的那五个。
/// `seq` 是 D18 已定项 1 的谱系重写序号——**未定**，所以它是臂的变量，不是既有事实。
/// `is_tombstone` 是「删除也写一个单元」这条候选，同样未定。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct UnitHeader {
    kind: u8,
    tree: u64,
    object_identifier: u64,
    object_birth_generation: u64,
    anchor: u64,
    rewrite_sequence: Option<u64>,
    is_tombstone: bool,
}

/// 盘上的一个单元。`physical_address` 不进头（D18 已定项 3 的禁放清单），
/// 但扫描的人天然知道自己读的是哪个落点——这是观测，不是权威记录。
#[derive(Clone, Copy, Debug)]
struct Unit {
    physical_address: u64,
    header: Option<UnitHeader>,
    /// 记账（D21 已定的权威态之一）说这个落点还活着吗。
    /// **只有乙臂的重建路径看得到它**——甲臂按 D11 已定项 4 的字面问题，只许看单元。
    live: bool,
}

type Key = (u64, u64, u64); // (tree, object_identifier, anchor)

/// 一次重建要产出的四类消息。全部按 D8 已定项 1 的「幂等完整值」形态。
#[derive(Clone, Default, Debug)]
struct State {
    /// 映射类：逻辑身份 → 物理落点
    map: BTreeMap<Key, u64>,
    /// 反向索引类：物理落点 → 逻辑身份
    reverse_index: BTreeMap<u64, Key>,
    /// 记账类：每棵树的活单元数（幂等完整值，不是增量 Δ）
    accounting: BTreeMap<u64, u64>,
    /// 墓碑类：这些 key 必须**不在** map 里
    dead_keys: BTreeSet<Key>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Arm {
    /// 头里带谱系重写序号（D18 已定项 1）
    carries_rewrite_sequence: bool,
    /// 删除也写一个墓碑单元
    writes_tombstone_unit: bool,
    /// 重建路径可以问记账「这个落点还活着吗」（= 不再是「只从单元」）
    consults_accounting: bool,
    /// 被覆写 / 被删的旧副本仍然物理可读
    stale_copies_visible: bool,
    /// 头全被抹掉（阴性对照）
    headers_erased: bool,
    /// 分配器复用了释放出来的低地址（新副本的落点比旧副本**小**）
    /// ⇒ 按物理地址扫盘时，先扫到的反而是新的。
    allocator_reuses_low_addresses: bool,
}

impl Arm {
    /// 甲：只从单元。**这是 D11 已定项 4 的字面问题。**
    fn jia_units_only() -> Self { Arm { carries_rewrite_sequence: false, writes_tombstone_unit: false, consults_accounting: false, stale_copies_visible: true, headers_erased: false, allocator_reuses_low_addresses: false } }
    /// 乙：单元 + 记账（D21 已定记账也是权威态，它没跟着索引一起丢）
    fn yi_units_plus_accounting() -> Self { Arm { consults_accounting: true, ..Arm::jia_units_only() } }
    /// 丙：单元 + 谱系重写序号（D18 已定项 1 的候选）
    fn bing_units_plus_rewrite_sequence() -> Self { Arm { carries_rewrite_sequence: true, ..Arm::jia_units_only() } }
    /// 丁：序号 + 显式墓碑单元。**全对照：这一臂必须四类全 0，否则整轮作废。**
    fn ding_rewrite_sequence_plus_tombstone() -> Self { Arm { carries_rewrite_sequence: true, writes_tombstone_unit: true, ..Arm::jia_units_only() } }
    /// 戊：陈旧副本不可读（旧副本一释放就物理消失）
    fn wu_stale_invisible() -> Self { Arm { stale_copies_visible: false, ..Arm::jia_units_only() } }
    /// 己：头全抹掉。**阴性对照：分歧必须等于全部 key。**
    fn ji_headers_erased() -> Self { Arm { headers_erased: true, ..Arm::jia_units_only() } }
    /// 庚：甲臂 + 分配器复用低地址。**它拆掉甲臂里那个隐藏的运气**——
    /// 单调向前分配时「按地址扫、取最后一个」恰好等于「取最新的一个」，
    /// 而那是模型的产物，不是任何条款给的保证。
    fn geng_allocator_reuses_low_addresses() -> Self { Arm { allocator_reuses_low_addresses: true, ..Arm::jia_units_only() } }
}

const ARMS: [(&str, fn() -> Arm); 7] = [
    ("jia_units_only", Arm::jia_units_only),
    ("yi_units_plus_acct", Arm::yi_units_plus_accounting),
    ("bing_units_plus_seq", Arm::bing_units_plus_rewrite_sequence),
    ("ding_seq_plus_tombstone", Arm::ding_rewrite_sequence_plus_tombstone),
    ("wu_stale_invisible", Arm::wu_stale_invisible),
    ("ji_headers_erased", Arm::ji_headers_erased),
    ("geng_allocator_reuses_low_addresses", Arm::geng_allocator_reuses_low_addresses),
];

/// 负载参数。三个数各自独立驱动一类分歧，所以绝对值断言钉得住。
#[derive(Clone, Copy, Debug)]
struct Load {
    object_count: u64,
    /// 覆写次数（每次覆写造一个新单元，旧的成为陈旧副本）
    overwrite_count: u64,
    /// 删除次数（删掉的是**没被覆写过**的那一段 key，两类互不重叠）
    delete_count: u64,
    tree_count: u64,
}

impl Load {
    /// 独立算出的应有单元数：建 object_count 个 + 覆写 overwrite_count 个 + （墓碑臂）delete_count 个。
    /// **这是绝对值断言的分母**，不是从被测代码里读回来的。
    fn expected_units(&self, arm: Arm) -> u64 {
        let base = self.object_count + self.overwrite_count + if arm.writes_tombstone_unit { self.delete_count } else { 0 };
        if arm.stale_copies_visible { base } else { base - self.overwrite_count - self.delete_count }
    }
}

/// 造盘：按臂的规则写单元，同时算出真值状态。
fn build(load: Load, arm: Arm) -> (Vec<Unit>, State) {
    assert!(load.overwrite_count + load.delete_count <= load.object_count, "覆写段与删除段不许重叠、不许超过对象数");
    let mut units: Vec<Unit> = Vec::new();
    let mut truth = State::default();
    // 三段互不相交的落点空间。**覆写落在哪一段是臂的变量**：
    // 单调向前 ⇒ 新副本地址更大；复用释放空间 ⇒ 新副本地址更小。
    let mut physical_address = 1_000_000u64;      // ① 建对象
    let mut overwrite_physical_address = 2_000_000u64;   // ② 覆写（单调向前）
    let mut reused_physical_address = 999_999u64;  // ② 覆写（复用低地址）
    let mut tombstone_physical_address = 3_000_000u64; // ③ 墓碑单元
    let mut write_sequence_counter = 0u64;
    let mut current_mapping: BTreeMap<Key, u64> = BTreeMap::new();

    let make_header = |load: &Load, object_index: u64, write_sequence: u64, is_tombstone: bool, arm: Arm| UnitHeader {
        kind: if is_tombstone { 2 } else { 1 },
        tree: object_index % load.tree_count,
        object_identifier: object_index,
        object_birth_generation: 1, // 对象出生代：覆写不改它，这正是 D18 已定项 3 字段集的形状
        anchor: 0,
        rewrite_sequence: if arm.carries_rewrite_sequence { Some(write_sequence) } else { None },
        is_tombstone,
    };

    // ① 建 object_count 个对象
    for object_index in 0..load.object_count {
        write_sequence_counter += 1;
        let unit_header = make_header(&load, object_index, write_sequence_counter, false, arm);
        units.push(Unit { physical_address, header: Some(unit_header), live: true });
        current_mapping.insert((unit_header.tree, unit_header.object_identifier, unit_header.anchor), physical_address);
        physical_address += 1;
    }
    // ② 覆写前 overwrite_count 个：新单元，旧单元按臂决定留不留
    for object_index in 0..load.overwrite_count {
        write_sequence_counter += 1;
        let unit_header = make_header(&load, object_index, write_sequence_counter, false, arm);
        let key = (unit_header.tree, unit_header.object_identifier, unit_header.anchor);
        let old_physical_address = current_mapping[&key];
        for unit in units.iter_mut() {
            if unit.physical_address == old_physical_address { unit.live = false; }
        }
        let new_physical_address = if arm.allocator_reuses_low_addresses { reused_physical_address -= 1; reused_physical_address + 1 } else { overwrite_physical_address += 1; overwrite_physical_address - 1 };
        units.push(Unit { physical_address: new_physical_address, header: Some(unit_header), live: true });
        current_mapping.insert(key, new_physical_address);
    }
    // ③ 删掉最后 delete_count 个（与覆写段不重叠）
    for object_index in (load.object_count - load.delete_count)..load.object_count {
        write_sequence_counter += 1;
        let unit_header = make_header(&load, object_index, write_sequence_counter, true, arm);
        let key = (unit_header.tree, unit_header.object_identifier, unit_header.anchor);
        let old_physical_address = current_mapping[&key];
        for unit in units.iter_mut() {
            if unit.physical_address == old_physical_address { unit.live = false; }
        }
        current_mapping.remove(&key);
        truth.dead_keys.insert(key);
        if arm.writes_tombstone_unit {
            units.push(Unit { physical_address: tombstone_physical_address, header: Some(unit_header), live: true });
            tombstone_physical_address += 1;
        }
    }
    // 陈旧副本不可读的臂：旧副本物理消失
    if !arm.stale_copies_visible {
        units.retain(|unit| unit.live);
    }
    // 阴性对照：头全抹掉
    if arm.headers_erased {
        for unit in units.iter_mut() { unit.header = None; }
    }

    for (key, physical_address) in &current_mapping {
        truth.map.insert(*key, *physical_address);
        truth.reverse_index.insert(*physical_address, *key);
        *truth.accounting.entry(key.0).or_insert(0) += 1;
    }
    (units, truth)
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct RebuildOutcome {
    units_scanned: u64,
    /// 同一个 key 有多份可读副本、且这一臂没有任何判据能分出哪份是现行值
    ambiguous: u64,
    /// 真值里有、重建里没有
    map_missing: u64,
    /// 重建里有、但指向别的落点（含「赌扫描顺序赌错」的那些）
    map_wrong: u64,
    /// 已删的 key 在重建结果里活了过来
    resurrected: u64,
    /// 反向索引条目缺 / 错
    reverse_index_bad: u64,
    /// 记账类对不上的树数
    accounting_bad: u64,
    /// 判别力自证真的注入了几处（重建结果是空集时注入不进去，见 E57 的「空集不是能重建」）
    injected: u64,
    /// 重建出来的映射条数
    rebuilt: u64,
}

impl RebuildOutcome {
    fn total_divergence(&self) -> u64 {
        self.map_missing + self.map_wrong + self.resurrected + self.reverse_index_bad + self.accounting_bad
    }
}

/// 只按这一臂允许看到的输入重建四类消息。
///
/// ⚠️ 没有判据可分辨同一 key 的多份副本时，**记成 ambiguous 并按「赌扫描顺序」落值**——
/// 两个数都报出来：`ambiguous` 是「重建方知道自己不知道」，`map_wrong` 是「赌错了几处」。
fn rebuild(units: &[Unit], arm: Arm) -> (State, u64, u64) {
    let mut candidates_by_key: BTreeMap<Key, Vec<(u64, Option<u64>, bool)>> = BTreeMap::new(); // key → [(physical_address, rewrite_sequence, is_tombstone)]
    let mut scanned = 0u64;
    // **按物理地址扫盘**，不是按写入顺序——扫的人手里只有盘，没有写入顺序。
    let mut units_by_physical_address: Vec<&Unit> = units.iter().collect();
    units_by_physical_address.sort_by_key(|unit| unit.physical_address);
    for unit in units_by_physical_address {
        scanned += 1;
        let Some(unit_header) = unit.header else { continue };
        if arm.consults_accounting && !unit.live { continue; } // 记账说它已经不是现行落点
        candidates_by_key.entry((unit_header.tree, unit_header.object_identifier, unit_header.anchor)).or_default().push((unit.physical_address, unit_header.rewrite_sequence, unit_header.is_tombstone));
    }
    let mut rebuilt_state = State::default();
    let mut ambiguous = 0u64;
    for (key, candidates) in &candidates_by_key {
        let chosen_candidate = if candidates.len() == 1 {
            candidates[0]
        } else if arm.carries_rewrite_sequence {
            // 序号能分出哪一份是最后写的
            *candidates.iter().max_by_key(|(_, candidate_sequence, _)| candidate_sequence.expect("seq 臂的每个头都该带序号")).unwrap()
        } else {
            ambiguous += 1;
            *candidates.last().unwrap() // 赌地址顺序：取扫到的最后一份（= 落点最大的那份）
        };
        if chosen_candidate.2 {
            continue; // 墓碑：这个 key 已经死了
        }
        rebuilt_state.map.insert(*key, chosen_candidate.0);
        rebuilt_state.reverse_index.insert(chosen_candidate.0, *key);
        *rebuilt_state.accounting.entry(key.0).or_insert(0) += 1;
    }
    (rebuilt_state, scanned, ambiguous)
}

/// 逐条比对重建结果与真值。`corruption_count` = 往重建结果里注入几处破坏（判别力自证）。
fn measure(load: Load, arm: Arm, corruption_count: usize) -> RebuildOutcome {
    let (units, truth) = build(load, arm);
    let (mut rebuilt_state, scanned, ambiguous) = rebuild(&units, arm);
    // 判别力自证：把 corruption_count 条重建出来的映射改到别处去，比较器必须逐条报出来。
    let rebuilt = rebuilt_state.map.len() as u64;
    // **只往「本来是对的」那些条目上注入**——往已经错的条目上再改一次，
    // 计数不会动，比较器会被冤枉成没有判别力（庚臂实测踩到）。
    let victims: Vec<Key> = rebuilt_state.map.iter()
        .filter(|(key, physical_address)| truth.map.get(*key) == Some(*physical_address))
        .map(|(key, _)| *key).take(corruption_count).collect();
    let injected = victims.len() as u64;
    for key in victims {
        let old_physical_address = rebuilt_state.map[&key];
        rebuilt_state.map.insert(key, old_physical_address + 500_000);
    }

    let mut outcome = RebuildOutcome { units_scanned: scanned, ambiguous, injected, rebuilt, ..Default::default() };
    for (key, true_physical_address) in &truth.map {
        match rebuilt_state.map.get(key) {
            None => outcome.map_missing += 1,
            Some(rebuilt_physical_address) if rebuilt_physical_address != true_physical_address => outcome.map_wrong += 1,
            Some(_) => {}
        }
    }
    for key in &truth.dead_keys {
        if rebuilt_state.map.contains_key(key) { outcome.resurrected += 1; }
    }
    for (physical_address, key) in &truth.reverse_index {
        match rebuilt_state.reverse_index.get(physical_address) {
            None => outcome.reverse_index_bad += 1,
            // ⚠️ **「同一个落点指向别的 key」这一支不可达，所以它不在这里。**
            // 一个落点的主人由**那个落点自己的头**说了算：重建方给 key K 选了落点 p，
            // 说明 p 的头写着 K；真值里 p 属于 K2，说明 p 的头写着 K2。两者不能同真。
            // 2026-08-31 变异测试报它是盲区（M12 改掉计数，一个测试都没红）——
            // 查下去它是**死代码**，不是缺检查。现在写成断言：模型一旦自相矛盾就炸。
            Some(rebuilt_key) => assert_eq!(rebuilt_key, key, "落点 {physical_address} 在两侧指向不同的 key，模型自相矛盾"),
        }
    }
    for tree_identifier in 0..load.tree_count {
        let true_accounting_count = truth.accounting.get(&tree_identifier).copied().unwrap_or(0);
        let rebuilt_accounting_count = rebuilt_state.accounting.get(&tree_identifier).copied().unwrap_or(0);
        if true_accounting_count != rebuilt_accounting_count { outcome.accounting_bad += 1; }
    }
    outcome
}

/// 全树缓冲能扣住多少条消息：按 D11 已定项 2 的几何算，`leaf_count` 个叶。
/// **这个数只用来划驻留窗口**，不参与任何一类消息可不可重算的判定。
fn resident_capacity(leaf_count: u64) -> u64 {
    let mut internal_node_count = 0u64;
    let mut level_node_count = leaf_count;
    while level_node_count > 1 {
        level_node_count = level_node_count.div_ceil(MESSAGE_BUFFER_DECISION_FANOUT as u64);
        internal_node_count += level_node_count;
    }
    internal_node_count * MESSAGE_BUFFER_DECISION_BUFFERED_MESSAGES as u64
}

const LOADS: [Load; 3] = [
    Load { object_count: 256, overwrite_count: 64, delete_count: 32, tree_count: 4 },
    Load { object_count: 4096, overwrite_count: 1024, delete_count: 512, tree_count: 8 },
    Load { object_count: 16384, overwrite_count: 4096, delete_count: 2048, tree_count: 16 },
];

fn main() {
    let mut emitter = Emitter::new();
    println!("{}", emitter.emit_raw(&format!(
        "name=config note=只扫单元头重算缓冲消息 node_bytes={MESSAGE_BUFFER_DECISION_NODE_BYTES} eps_bp={MESSAGE_BUFFER_DECISION_EPSILON_BASIS_POINTS} \
         fanout={MESSAGE_BUFFER_DECISION_FANOUT} buf_msgs={MESSAGE_BUFFER_DECISION_BUFFERED_MESSAGES}")));
    for leaf_count in [1024u64, 8192, 32768] {
        println!("{}", emitter.emit_raw(&format!(
            "name=geom nleaf={leaf_count} resident_capacity={}", resident_capacity(leaf_count))));
    }
    for load in LOADS {
        for (arm_name, make_arm) in ARMS {
            let arm = make_arm();
            let outcome = measure(load, arm, 0);
            println!("{}", emitter.emit_raw(&format!(
                "name=cell arm={arm_name} n_obj={} n_ow={} n_del={} units={} expected_units={} \
                 ambiguous={} map_missing={} map_wrong={} resurrected={} backref_bad={} \
                 acct_bad={} divergence={}",
                load.object_count, load.overwrite_count, load.delete_count, outcome.units_scanned, load.expected_units(arm),
                outcome.ambiguous, outcome.map_missing, outcome.map_wrong, outcome.resurrected, outcome.reverse_index_bad,
                outcome.accounting_bad, outcome.total_divergence())));
        }
    }
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    const MEDIUM_LOAD: Load = LOADS[1];

    /// **作废条款①**：扫到的单元数必须等于独立算出的应有数。
    /// 读不到 ≠ 读到 0——扫描没在读盘时，所有「分歧为 0」都是假的。
    #[test]
    fn scan_reads_exactly_the_units_that_were_written() {
        for load in LOADS {
            for (arm_name, make_arm) in ARMS {
                let arm = make_arm();
                let outcome = measure(load, arm, 0);
                assert_eq!(outcome.units_scanned, load.expected_units(arm),
                    "{arm_name}：扫到 {} 个单元，独立算出应有 {}", outcome.units_scanned, load.expected_units(arm));
                assert!(outcome.units_scanned > 0, "{arm_name}：一个单元都没扫到");
            }
        }
    }

    /// **作废条款②（全对照）**：序号 + 显式墓碑单元这一臂必须四类全 0。
    /// 闭不上说明模型另有 bug，整轮作废。
    #[test]
    fn the_full_control_arm_closes_every_class_to_zero() {
        for load in LOADS {
            let outcome = measure(load, Arm::ding_rewrite_sequence_plus_tombstone(), 0);
            assert_eq!(outcome.map_missing, 0);
            assert_eq!(outcome.map_wrong, 0);
            assert_eq!(outcome.resurrected, 0);
            assert_eq!(outcome.reverse_index_bad, 0);
            assert_eq!(outcome.accounting_bad, 0);
            assert_eq!(outcome.ambiguous, 0, "有序号就不该有分不出来的副本");
        }
    }

    /// **作废条款③：判别力自证，且对每一条臂都跑。**
    /// 只对一条臂跑等于另外五条从没过闸（test-discipline.md）。
    ///
    /// ⚠️ **注入不进去的那一臂要单独说清楚，不许算它通过**：己臂重建出的是**空集**，
    /// 没有条目可破坏 —— 这正是 E57 记下的「0 是空集造成的，不是『能重建』」那个坑。
    #[test]
    fn the_comparator_reports_injected_damage_on_every_arm() {
        for (arm_name, make_arm) in ARMS {
            let arm = make_arm();
            let clean_outcome = measure(MEDIUM_LOAD, arm, 0);
            for injected_damage_count in [1usize, 5, 17] {
                let dirty_outcome = measure(MEDIUM_LOAD, arm, injected_damage_count);
                assert_eq!(dirty_outcome.map_wrong, clean_outcome.map_wrong + dirty_outcome.injected,
                    "{arm_name}：注入 {} 处破坏，比较器该逐条报出来", dirty_outcome.injected);
                if arm.headers_erased {
                    assert_eq!(dirty_outcome.rebuilt, 0, "{arm_name}：头全抹掉时重建结果本该是空集");
                    assert_eq!(dirty_outcome.injected, 0, "{arm_name}：空集里注入不进任何破坏");
                } else {
                    assert_eq!(dirty_outcome.injected, injected_damage_count as u64,
                        "{arm_name}：本该注入 {injected_damage_count} 处，实际只注入了 {}", dirty_outcome.injected);
                }
                assert_eq!(dirty_outcome.map_missing, clean_outcome.map_missing, "{arm_name}：注入不该改变缺条数");
            }
        }
    }

    /// **作废条款④**：驻留容量不许是 0，否则「消息驻留在缓冲里」这件事根本没被测到。
    #[test]
    fn the_resident_window_is_not_empty() {
        for leaf_count in [1024u64, 8192, 32768] {
            assert!(resident_capacity(leaf_count) > 0, "nleaf={leaf_count} 的驻留容量算出来是 0");
        }
        // 绝对值：1024 叶 ⇒ 内部节点 9+1=10 个（1024→9→1），扣得住 10×665=6650 条。
        assert_eq!(resident_capacity(1024), 6650);
    }

    /// **绝对值①**：D11 已定项 2 的几何自洽——ε=0.65 的 16 KiB 节点确实给出 119 / 665。
    /// 三条臂互比测不出「三条一起错」，所以这里钉死绝对值（test-discipline.md）。
    #[test]
    fn the_geometry_matches_what_the_message_buffer_decision_settled() {
        let buffer_bytes = MESSAGE_BUFFER_DECISION_NODE_BYTES * MESSAGE_BUFFER_DECISION_EPSILON_BASIS_POINTS / 10_000;
        assert_eq!(buffer_bytes / MESSAGE_BYTES, MESSAGE_BUFFER_DECISION_BUFFERED_MESSAGES, "ε=0.65 的缓冲该装 665 条消息");
        let pivot_bytes = MESSAGE_BUFFER_DECISION_NODE_BYTES - NODE_HEADER_BYTES - buffer_bytes;
        assert_eq!(pivot_bytes / PIVOT_BYTES, MESSAGE_BUFFER_DECISION_FANOUT, "剩下的字节该给出扇出 119");
        assert_eq!(MESSAGE_BUFFER_DECISION_NODE_BYTES, 16384, "D8 已定项 2 钉的是 16 KiB");
    }

    /// **绝对值②**：甲臂分不出来的副本数**恰好等于覆写次数**，不是「有一些」。
    #[test]
    fn ambiguity_equals_exactly_the_overwrite_count() {
        for load in LOADS {
            assert_eq!(measure(load, Arm::jia_units_only(), 0).ambiguous, load.overwrite_count,
                "只从单元时，分不出现行值的 key 数该恰好等于覆写次数");
        }
    }

    /// **绝对值③**：甲臂复活的 key 数**恰好等于删除次数**。
    /// ⚠️ 这一条与序号无关——丙臂（有序号）同样全数复活，因为删除在盘上没留任何单元。
    #[test]
    fn every_deleted_key_comes_back_when_no_tombstone_unit_exists() {
        for load in LOADS {
            assert_eq!(measure(load, Arm::jia_units_only(), 0).resurrected, load.delete_count);
            assert_eq!(measure(load, Arm::bing_units_plus_rewrite_sequence(), 0).resurrected, load.delete_count,
                "谱系重写序号解决不了删除——盘上没有那条消息的任何痕迹");
        }
    }

    /// **绝对值④（阴性对照）**：头全抹掉 ⇒ 缺的条数等于真值里的全部 key。
    #[test]
    fn erasing_headers_loses_exactly_every_key() {
        for load in LOADS {
            let live_keys = load.object_count - load.delete_count;
            let outcome = measure(load, Arm::ji_headers_erased(), 0);
            assert_eq!(outcome.map_missing, live_keys, "头没了该一条都重建不出来");
            assert_eq!(outcome.resurrected, 0, "什么都没重建出来时不存在复活");
        }
    }

    /// **谱系重写序号买到的和买不到的，各自钉一个绝对值。**
    /// 买到：覆写造成的歧义归零。买不到：删除造成的复活一条不减。
    #[test]
    fn the_rewrite_sequence_number_fixes_overwrites_but_not_deletes() {
        for load in LOADS {
            let bing_outcome = measure(load, Arm::bing_units_plus_rewrite_sequence(), 0);
            assert_eq!(bing_outcome.ambiguous, 0);
            assert_eq!(bing_outcome.map_wrong, 0, "有序号就该逐条指对落点");
            assert_eq!(bing_outcome.resurrected, load.delete_count);
        }
    }

    /// **记账那条腿买到什么**：它同时解决覆写与删除——因为「哪个落点还活着」正是它记的东西。
    /// ⚠️ 但它已经不是「只从单元」了，所以它是**答案的边界**，不是答案。
    #[test]
    fn asking_the_accounting_layer_closes_both_gaps_but_changes_the_question() {
        for load in LOADS {
            let outcome = measure(load, Arm::yi_units_plus_accounting(), 0);
            assert_eq!(outcome.map_wrong, 0);
            assert_eq!(outcome.resurrected, 0);
            assert_eq!(outcome.ambiguous, 0);
            assert_eq!(outcome.accounting_bad, 0);
        }
    }

    /// **陈旧副本不可读时，甲臂的分歧全部消失。**
    /// ⇒ 结论条件于「陈旧副本可不可见」这条前提，它必须被写下来。
    #[test]
    fn the_whole_divergence_is_conditional_on_stale_copies_being_readable() {
        for load in LOADS {
            let visible_outcome = measure(load, Arm::jia_units_only(), 0);
            let invisible_outcome = measure(load, Arm::wu_stale_invisible(), 0);
            assert!(visible_outcome.total_divergence() > 0, "陈旧副本可读时本该有分歧");
            assert_eq!(invisible_outcome.total_divergence(), 0, "陈旧副本不可读时本该一处不差");
            // 绝对值：不可读那一臂扫到的单元数恰好等于活单元数
            assert_eq!(invisible_outcome.units_scanned, load.object_count - load.delete_count);
        }
    }

    /// **记账类消息跟着映射类一起错**：它是从同一份「哪些单元是活的」推出来的。
    /// 绝对值：甲臂对不上的树数等于全部树数（每棵树都被复活的 key 顶高了）。
    #[test]
    fn the_accounting_class_breaks_together_with_the_mapping_class() {
        for load in LOADS {
            let outcome = measure(load, Arm::jia_units_only(), 0);
            assert_eq!(outcome.accounting_bad, load.tree_count, "每棵树的活单元数都该被复活的 key 顶高");
        }
    }

    /// **甲臂那个 0 是运气，不是保证。**
    /// 单调向前分配时「按地址扫、取地址最大的一份」恰好等于「取最新的一份」；
    /// 分配器一旦复用释放出来的低地址（庚臂），同一段代码就**逐条赌错**。
    /// ⚠️ 两条绝对值一起钉：甲臂赌对 0 处、庚臂赌错的处数**恰好等于覆写次数**。
    #[test]
    fn guessing_by_address_order_is_luck_and_the_reuse_arm_shows_it() {
        for load in LOADS {
            let jia_outcome = measure(load, Arm::jia_units_only(), 0);
            let geng_outcome = measure(load, Arm::geng_allocator_reuses_low_addresses(), 0);
            assert_eq!(jia_outcome.map_wrong, 0, "单调向前分配时恰好赌对");
            assert_eq!(jia_outcome.ambiguous, load.overwrite_count, "但重建方并不知道自己赌对了");
            assert_eq!(geng_outcome.map_wrong, load.overwrite_count, "复用低地址时逐条赌错");
            assert_eq!(geng_outcome.ambiguous, load.overwrite_count, "歧义数与分配顺序无关");
        }
    }

    /// **反向索引类：三条臂各钉一个绝对值。**
    ///
    /// ⚠️ 这条的第一版只写了甲臂的 `reverse_index_bad == map_wrong`，而甲臂上**两个数都是 0**——
    /// `0 == 0` 什么也没证明。2026-08-31 变异测试实测：把缺条那一支的计数改成 `+= 0`，
    /// **一个测试都没红**。互比断言旁边必须有钉绝对值的那条（test-discipline.md）。
    #[test]
    fn the_reverse_index_class_breaks_by_exactly_these_counts() {
        for load in LOADS {
            let live_keys = load.object_count - load.delete_count;
            assert_eq!(measure(load, Arm::jia_units_only(), 0).reverse_index_bad, 0, "甲臂赌对了落点，反向索引不缺条");
            assert_eq!(measure(load, Arm::geng_allocator_reuses_low_addresses(), 0).reverse_index_bad, load.overwrite_count,
                "庚臂赌错几处落点，反向索引就缺几条");
            assert_eq!(measure(load, Arm::ji_headers_erased(), 0).reverse_index_bad, live_keys,
                "头全抹掉时反向索引一条都建不出来");
            assert_eq!(measure(load, Arm::ding_rewrite_sequence_plus_tombstone(), 0).reverse_index_bad, 0);
        }
    }
}
