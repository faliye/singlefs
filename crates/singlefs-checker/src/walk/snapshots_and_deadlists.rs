//! 快照与 deadlist：I-3.6（deadlist 紧界）、I-3.7（活快照可枚举） 与 I-3.5（引用区间的精确性） 的 deadlist 那几格。

use super::*;

// ───────── 只读快照与 deadlist：I-3.5 ③ ④ 与逆向、I-3.6、I-3.7─────────

/// 只读快照与 deadlist 那一遍要判的一版：称呼（判红与判不了时点名）、`checkpoint_txg`、它的树表指针与中央映射根指针。版本清单与删除态扫描同一份
/// （回退候选集里每条根与由记录施加出来的那几版）。
pub(super) struct VersionToScanTheSnapshotsAndTheDeadlistOf<'record> {
    pub(super) name: String,
    pub(super) checkpoint_txg: u64,
    pub(super) tree_table_pointer: &'record [u8],
    pub(super) mapping_root_pointer: &'record [u8],
}

/// 一个快照在一版里的样子：码 14 条目 flags 位 0。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SnapshotStanding {
    Live,
    /// 销毁意图已写：从写意图那一刻起不再钉住块（D5（快照 / 空间记账机制） 已定项 20 的例外句）。
    BeingDestroyed,
}

/// 一版树表里的一个快照：码 14 条目里要的几段，与头 ID 配上它的那一条码 15 的根指针（恰好一条码 15 配它时才有；一一配对由 I-9.16 ③ 判）。
#[derive(Clone, Copy, Debug)]
struct SnapshotEntryOfTheScan {
    snapshot_inode_root_tree: u64,
    /// 码 14 条目的诞生 txg：就是 S.txg。
    snapshot_txg: u64,
    previous_snapshot_txg: u64,
    standing: SnapshotStanding,
    inode_root: PointerView,
    extent_root: Option<PointerView>,
}

/// 一张树表里的快照，按（诞生 txg, 树 ID）升序。调用方保证每条条目都不窄于字段表。
fn snapshot_entries_of_the_tree_table(
    tree_table_entries: &[Vec<u8>],
) -> Vec<SnapshotEntryOfTheScan> {
    let kind_of = |entry: &Vec<u8>| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET);
    let snapshot_extent_roots: Vec<(u64, PointerView)> = tree_table_entries
        .iter()
        .filter(|entry| kind_of(entry) == TREE_KIND_SNAPSHOT_EXTENT_ROOT)
        .map(|entry| {
            (
                read_u64(entry, TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET),
                parse_node_pointer(&entry[14..100]),
            )
        })
        .collect();
    let mut snapshots: Vec<SnapshotEntryOfTheScan> = tree_table_entries
        .iter()
        .filter(|entry| kind_of(entry) == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .map(|entry| {
            let snapshot_inode_root_tree = read_u64(entry, 0);
            let paired_extent_roots: Vec<PointerView> = snapshot_extent_roots
                .iter()
                .filter(|(head, _)| *head == snapshot_inode_root_tree)
                .map(|(_, root)| *root)
                .collect();
            SnapshotEntryOfTheScan {
                snapshot_inode_root_tree,
                snapshot_txg: read_u64(entry, TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET),
                previous_snapshot_txg: read_u64(
                    entry,
                    TREE_TABLE_ENTRY_PREVIOUS_SNAPSHOT_TXG_OFFSET,
                ),
                standing: if read_u16(entry, TREE_TABLE_ENTRY_FLAGS_OFFSET)
                    & TREE_TABLE_ENTRY_FLAG_SNAPSHOT_BEING_DESTROYED
                    == 0
                {
                    SnapshotStanding::Live
                } else {
                    SnapshotStanding::BeingDestroyed
                },
                inode_root: parse_node_pointer(&entry[14..100]),
                extent_root: match paired_extent_roots.as_slice() {
                    [only] => Some(*only),
                    [] | [_, _, ..] => None,
                },
            }
        })
        .collect();
    snapshots.sort_unstable_by_key(|snapshot| {
        (snapshot.snapshot_txg, snapshot.snapshot_inode_root_tree)
    });
    snapshots
}

/// I-3.7 每一版那一格：这一版每一个活快照都是这一版之前的一次捕获——诞生 txg（S.txg）不大于这一版的 txg，两条根指针（非全零的）
/// 头里的出生 txg 不大于 S.txg（建快照那次独立发布不动活头两棵树，捕获的是那一刻已经发布的根，判决 S2）。一个活快照都没有的一版也判一次、成立：
/// 这一版的活快照集合是空集，与它树表里一条码 14 都没有相符。
fn judge_live_snapshots_are_captures_before_their_version(
    version_name: &str,
    version_txg: u64,
    snapshots: &[SnapshotEntryOfTheScan],
    judgements: &mut Judgements,
) {
    let snapshots_not_captured_before_the_version: Vec<String> = snapshots
        .iter()
        .filter(|snapshot| snapshot.standing == SnapshotStanding::Live)
        .filter_map(|snapshot| {
            let latest_root_birth_txg = [Some(snapshot.inode_root), snapshot.extent_root]
                .into_iter()
                .flatten()
                .filter(|root| !root.all_zero)
                .map(|root| root.birth_txg)
                .max();
            let is_a_capture_before_the_version = snapshot.snapshot_txg <= version_txg
                && latest_root_birth_txg.is_none_or(|birth_txg| birth_txg <= snapshot.snapshot_txg);
            (!is_a_capture_before_the_version).then(|| {
                format!(
                    "快照（码 14 树 {}）诞生 txg {}、根指针最晚的出生 txg {latest_root_birth_txg:?}",
                    snapshot.snapshot_inode_root_tree, snapshot.snapshot_txg
                )
            })
        })
        .collect();
    judgements.judge(
        LIVE_SNAPSHOTS_ARE_ENUMERABLE,
        snapshots_not_captured_before_the_version.is_empty(),
        || {
            format!(
                "{version_name}（txg {version_txg}）的树表里有活快照不是这一版之前的一次捕获（诞生 txg 要不大于这一版的 txg、根指针的出生 txg 要不大于诞生 txg）：{}",
                snapshots_not_captured_before_the_version.join("；")
            )
        },
    );
}

/// 一版的活快照与销毁中的快照：码 14 树 ID → (S.txg, 样子)。I-3.7 跨版本那一格比相接两版的它。
struct SnapshotsOfAVersion {
    pub(super) name: String,
    pub(super) checkpoint_txg: u64,
    snapshots: BTreeMap<u64, (u64, SnapshotStanding)>,
}

/// I-3.7 跨版本那一格：回退候选集里 txg 相接（后一版的 txg = 前一版的 txg + 1）的两版之间，快照集合只照条款变——
/// 后一版多出来的活快照诞生 txg 就是后一版的 txg（建快照那次独立发布，判决 S2）；前一版的活快照在后一版要么还活着、要么带上销毁中记号
/// （写意图那次发布），前一版销毁中的在后一版要么还销毁中、要么没了（最后一批同一次发布删码 14、15，判决 S4）；同一个快照两版里诞生 txg 相同。
/// txg 不相接的两版之间可以隔着几次发布（环上有洞、崩溃恢复抛弃了中间的根），不判；同一个 txg 有不止一版（实例不同）的，那一 txg 不判。
/// 循环：按 txg 升序走一遍（轮数 = 不同 txg 的个数），不跨轮带状态；提前出口只有「不判」的那几样。
fn judge_live_snapshots_of_consecutive_versions(
    versions: &[SnapshotsOfAVersion],
    judgements: &mut Judgements,
) {
    let mut versions_by_txg: BTreeMap<u64, Vec<&SnapshotsOfAVersion>> = BTreeMap::new();
    for version in versions {
        versions_by_txg
            .entry(version.checkpoint_txg)
            .or_default()
            .push(version);
    }
    for (later_txg, later_versions) in &versions_by_txg {
        let Some(earlier_txg) = later_txg.checked_sub(1) else {
            continue;
        };
        let (Some([earlier]), [later]) = (
            versions_by_txg.get(&earlier_txg).map(Vec::as_slice),
            later_versions.as_slice(),
        ) else {
            continue;
        };
        let changes_against_the_clauses =
            snapshot_changes_against_the_clauses_between(earlier, later);
        judgements.judge(
            LIVE_SNAPSHOTS_ARE_ENUMERABLE,
            changes_against_the_clauses.is_empty(),
            || {
                format!(
                    "{}（txg {}）到{}（txg {}）之间快照集合的变化不是建快照、写销毁意图或最后一批造得出来的：{}",
                    earlier.name,
                    earlier.checkpoint_txg,
                    later.name,
                    later.checkpoint_txg,
                    changes_against_the_clauses.join("；")
                )
            },
        );
    }
}

/// 相接两版之间快照集合里不照条款的变化，逐条写成一句（空就是全照条款）。
fn snapshot_changes_against_the_clauses_between(
    earlier: &SnapshotsOfAVersion,
    later: &SnapshotsOfAVersion,
) -> Vec<String> {
    let mut changes = Vec::new();
    for (tree, (snapshot_txg, standing_before)) in &earlier.snapshots {
        match (standing_before, later.snapshots.get(tree)) {
            (_, Some((snapshot_txg_after, _))) if snapshot_txg_after != snapshot_txg => {
                changes.push(format!(
                    "快照（码 14 树 {tree}）的诞生 txg 从 {snapshot_txg} 变成 {snapshot_txg_after}"
                ));
            }
            (SnapshotStanding::Live, Some((_, SnapshotStanding::Live | SnapshotStanding::BeingDestroyed)))
            | (SnapshotStanding::BeingDestroyed, Some((_, SnapshotStanding::BeingDestroyed)) | None) => {}
            (SnapshotStanding::Live, None) => changes.push(format!(
                "活快照（码 14 树 {tree}、诞生 txg {snapshot_txg}）没带销毁中记号就从树表里没了（少一条）"
            )),
            (SnapshotStanding::BeingDestroyed, Some((_, SnapshotStanding::Live))) => {
                changes.push(format!(
                    "销毁中的快照（码 14 树 {tree}、诞生 txg {snapshot_txg}）在后一版又不带销毁中记号了"
                ));
            }
        }
    }
    for (tree, (snapshot_txg, standing_after)) in &later.snapshots {
        if earlier.snapshots.contains_key(tree) {
            continue;
        }
        match standing_after {
            SnapshotStanding::Live if *snapshot_txg == later.checkpoint_txg => {}
            SnapshotStanding::Live => changes.push(format!(
                "活快照（码 14 树 {tree}、诞生 txg {snapshot_txg}）在后一版多出来，诞生 txg 不是后一版的 txg {}（多一条）",
                later.checkpoint_txg
            )),
            SnapshotStanding::BeingDestroyed => changes.push(format!(
                "快照（码 14 树 {tree}、诞生 txg {snapshot_txg}）在后一版多出来就带着销毁中记号"
            )),
        }
    }
    changes
}

/// deadlist 的一条条目（D5（快照 / 空间记账机制） 已定项 12：头的树 ID 8 + 纪元 8 + 中央映射 key 27，value 空）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct DeadlistEntryOfTheScan {
    head: u64,
    /// 块被杀那一刻头的 `previous_snapshot_txg`（分桶就是这一段）。
    epoch: u64,
    pub(super) unit: CentralMappingKey,
}

impl DeadlistEntryOfTheScan {
    /// 调用方保证条目至少 43 字节（走读守过）。
    pub(super) fn of_entry(entry: &[u8]) -> Self {
        Self {
            head: read_u64(entry, DEADLIST_ENTRY_HEAD_OFFSET),
            epoch: read_u64(entry, DEADLIST_ENTRY_EPOCH_OFFSET),
            unit: CentralMappingKey(
                entry[DEADLIST_ENTRY_MAPPING_KEY_OFFSET
                    ..DEADLIST_ENTRY_MAPPING_KEY_OFFSET + mapping_key_bytes()]
                    .to_vec(),
            ),
        }
    }

    /// 被钉单元的 birth：映射 key 里出生 txg 那一段。
    pub(super) fn birth_txg(&self) -> u64 {
        read_u64(&self.unit.0, MAPPING_KEY_BIRTH_TXG_OFFSET)
    }
}

/// 一个活快照：码 14 那几段与它的两棵树（码 14 指的 inode 树、配对码 15 指的 extent 树）下面每个单元的中央映射 key。
#[derive(Clone, Debug)]
struct LiveSnapshotReach {
    snapshot_inode_root_tree: u64,
    snapshot_txg: u64,
    previous_snapshot_txg: u64,
    units: BTreeSet<CentralMappingKey>,
}

/// 一个销毁中的快照：待放区间 [lo, hi) 的 lo = S.txg，P = 它的前驱；hi 从活快照链现取（[`entry_is_pending_release`]）。
#[derive(Clone, Copy, Debug)]
struct SnapshotBeingDestroyedOfTheScan {
    snapshot_inode_root_tree: u64,
    snapshot_txg: u64,
    previous_snapshot_txg: u64,
}

/// 一版上 I-3.5 ③ ④ 与逆向、I-3.6 要的全部输入（只读读出来的）。第一版只有一个活头：树表里恰好一条 inode 树条目与一条头 ID 指它的 extent 树条目。
#[derive(Clone, Debug)]
struct DeadlistAndSnapshotReachOfAVersion {
    head_inode_tree: u64,
    head_previous_snapshot_txg: u64,
    /// 活头两棵树（inode 树、extent 树）下面每个单元的中央映射 key：Ref_头。
    units_of_the_head: BTreeSet<CentralMappingKey>,
    /// 按（S.txg, 树 ID）升序。
    live_snapshots: Vec<LiveSnapshotReach>,
    snapshots_being_destroyed: Vec<SnapshotBeingDestroyedOfTheScan>,
    deadlist_entries: Vec<DeadlistEntryOfTheScan>,
    /// 被判这一版的 checkpoint txg（根环里的根取根记录里的，由记录施加出来的那一版取记录里的）：下一侧是活头时待放集的上界
    /// （[`entry_is_pending_release`]）。
    checkpoint_txg_of_the_version: u64,
    /// 这一版中央映射叶里的 key：每条 deadlist 条目的映射 key 要在里面查得到（C300（deadlist 条目按 key 查映射查不到没有条款）
    /// 的 checker 一半）。
    keys_of_the_central_mapping: CentralMappingKeysOfAVersion,
}

/// 一版中央映射叶里出现过的 key，只读读出来的（[`central_mapping_keys_of_a_version_without_judging`]）。
#[derive(Clone, Debug)]
enum CentralMappingKeysOfAVersion {
    /// 映射树读全了（根指针全零是那一版没有映射，也算读全、集合是空的）。同一把 key 在叶里出现两次的也收：「查得到」只看有没有，
    /// 重复归 I-1.1（块头自述逻辑地址） 与 I-1.13（扫描重建与遍历方向的索引相同）。
    ReadInFull(BTreeSet<CentralMappingKey>),
    /// 映射树读不全（节点读不出、key 宽不是 27、条目窄、层级不对）：deadlist 的 key 查不查得到判不了，读不全由主走读的映射那一遍判。
    NotReadInFull,
}

/// deadlist 的一侧：一个活快照或活头；纪元落在 [前驱, 上界) 里的条目归它（D5（快照 / 空间记账机制） 已定项 12「哪一份」：
/// 快照 X 那一份 = [prev(X).txg, X.txg)，头那一份 = [prev(头).txg, ∞)）。销毁中的快照不是一侧：写意图那次发布已把它的前驱传给了下一侧。
struct DeadlistSide {
    pub(super) name: String,
    previous_snapshot_txg: u64,
    /// `None` 是活头（上界 ∞）。
    epoch_upper_bound: Option<u64>,
}

impl DeadlistSide {
    fn owns_epoch(&self, epoch: u64) -> bool {
        epoch >= self.previous_snapshot_txg
            && self
                .epoch_upper_bound
                .is_none_or(|upper_bound| epoch < upper_bound)
    }
}

fn deadlist_sides(facts: &DeadlistAndSnapshotReachOfAVersion) -> Vec<DeadlistSide> {
    facts
        .live_snapshots
        .iter()
        .map(|snapshot| DeadlistSide {
            name: format!(
                "活快照（码 14 树 {}、S.txg {}）",
                snapshot.snapshot_inode_root_tree, snapshot.snapshot_txg
            ),
            previous_snapshot_txg: snapshot.previous_snapshot_txg,
            epoch_upper_bound: Some(snapshot.snapshot_txg),
        })
        .chain(std::iter::once(DeadlistSide {
            name: format!("活头（inode 树 {}）", facts.head_inode_tree),
            previous_snapshot_txg: facts.head_previous_snapshot_txg,
            epoch_upper_bound: None,
        }))
        .collect()
}

/// 活快照链上排在（`snapshot_txg`, `snapshot_inode_root_tree`）之后的第一个活快照的 S.txg：那一侧的纪元上界；之后没有活快照时是活头（∞，`None`）。
fn next_live_side_upper_bound_after(
    facts: &DeadlistAndSnapshotReachOfAVersion,
    snapshot_txg: u64,
    snapshot_inode_root_tree: u64,
) -> Option<u64> {
    facts
        .live_snapshots
        .iter()
        .find(|live| {
            (live.snapshot_txg, live.snapshot_inode_root_tree)
                > (snapshot_txg, snapshot_inode_root_tree)
        })
        .map(|live| live.snapshot_txg)
}

/// 一条 deadlist 条目在不在某个未完成销毁意图的待放集里（判决 S4：意图 (头, lo = S.txg, hi, P = prev(S).txg)，之后按批放掉 [lo, hi) 里
/// birth > P 的条目）。hi 在下一侧是快照时取它的 S.txg；下一侧是活头时取被判这一版的 checkpoint txg（判决
/// `research/prompts/m4-c145-destruction-bound-r1-main-verification.md` 第二节 B2；条款写的「写意图那次发布的 txg」不落盘）。
/// 这样取与按写意图那次的 txg 取，在盘上读得到的条目上结论相同；撑住这一句的是 core 的两道拒绝，不是条目自己的样子：
/// 意图挂着时不准建快照（`SnapshotCreationRefusal::SnapshotBeingDestroyed`）、不准再写第二个销毁意图
/// （`SnapshotDestructionRefusal::AnotherSnapshotIsBeingDestroyed`）。有这两道，写意图之后到最后一批之前头的前驱一直是 P < lo，这段时间里
/// 新进来的条目纪元都小于 lo；写意图那一刻已在的条目，纪元是被杀那一刻头的前驱，都小于写意图那次的 txg，也就小于这一版的 txg。
/// 拿掉那两道拒绝，这一句就不成立。上界取 ∞ 时，纪元抬到这一版 txg 以上的坏条目也被认成待放、逃过 I-3.6 与 I-3.5 逆向。
fn entry_is_pending_release(
    facts: &DeadlistAndSnapshotReachOfAVersion,
    entry: &DeadlistEntryOfTheScan,
) -> bool {
    entry.head == facts.head_inode_tree
        && facts.snapshots_being_destroyed.iter().any(|destroyed| {
            let upper_bound = next_live_side_upper_bound_after(
                facts,
                destroyed.snapshot_txg,
                destroyed.snapshot_inode_root_tree,
            )
            .unwrap_or(facts.checkpoint_txg_of_the_version);
            entry.epoch >= destroyed.snapshot_txg
                && entry.epoch < upper_bound
                && entry.birth_txg() > destroyed.previous_snapshot_txg
        })
}

/// 活快照与活头的前驱链首尾相接：按（S.txg, 树 ID）升序排的各侧里，每一侧的前驱等于排在它前面那一侧的上界（[`deadlist_sides`] 的次序，
/// 活头在最后）。首尾相接时各侧互不相交，纪元落不进任何一侧的只剩「纪元小于最早那一侧的前驱」一种；不首尾相接归 I-9.25（树表条目的
/// previous_snapshot_txg 取值）。
fn previous_snapshot_chain_of_the_sides_is_contiguous(sides: &[DeadlistSide]) -> bool {
    sides
        .windows(2)
        .all(|pair| pair[0].epoch_upper_bound == Some(pair[1].previous_snapshot_txg))
}

/// I-3.5 ③ ④ 与逆向、I-3.6 在一版上判：
/// - ④ deadlist 里每一条钉的单元，活头都不引用；
/// - 逆向：deadlist 里每一条钉的单元，有一个活快照引用它，或它在待放集里（[`entry_is_pending_release`]）；
/// - ③ 活快照引用而活头不引用的每个单元，deadlist 里恰好一条钉它，头是活头、纪元落在引用它的最晚那个活快照的 S.txg 与链上下一侧的上界之间；
/// - I-3.6 每一条的 birth 不大于它归属那一侧的前驱，待放的豁免；一条条目都没有时判一次、成立；
/// - I-3.6 前驱链首尾相接时每一条都落进某一侧（C145（deadlist 在盘上没有落点） 第一条「每个纪元恰好落进一份区间」并进
///   I-3.6），待放的豁免；
/// - I-3.6 每一条的映射 key 在这一版的中央映射里查得到（C300（deadlist 条目按 key 查映射查不到没有条款） 的 checker 一半）。
///
/// 「续做点之前不许再有这样的条目」那一半没有输入：续做点不存（岔路 2），条目被放掉就是进度，这一格不判。
fn judge_the_deadlist_against_the_head_and_the_live_snapshots(
    version_name: &str,
    facts: &DeadlistAndSnapshotReachOfAVersion,
    judgements: &mut Judgements,
) {
    let sides = deadlist_sides(facts);
    let chain_is_contiguous = previous_snapshot_chain_of_the_sides_is_contiguous(&sides);
    let units_of_some_live_snapshot: BTreeSet<&CentralMappingKey> = facts
        .live_snapshots
        .iter()
        .flat_map(|snapshot| snapshot.units.iter())
        .collect();
    if facts.deadlist_entries.is_empty() {
        judgements.judge(
            DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE,
            true,
            String::new,
        );
    }
    let keys_of_the_central_mapping = match &facts.keys_of_the_central_mapping {
        CentralMappingKeysOfAVersion::ReadInFull(keys) => Some(keys),
        CentralMappingKeysOfAVersion::NotReadInFull => {
            if !facts.deadlist_entries.is_empty() {
                judgements.part_of_the_range_left_unjudged(
                    DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE,
                    CENTRAL_MAPPING_OF_THE_DEADLIST_VERSION_NOT_READ_IN_FULL.to_string(),
                );
            }
            None
        }
    };
    for entry in &facts.deadlist_entries {
        let is_pending_release = entry_is_pending_release(facts, entry);
        if let Some(keys) = keys_of_the_central_mapping {
            judgements.judge(
                DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE,
                keys.contains(&entry.unit),
                || {
                    format!(
                        "{version_name}：deadlist 里一条（头 {}、纪元 {}）钉的单元 {} 的映射 key 在这一版的中央映射里查不到（C300 第一版判损坏：每条 deadlist key 要在同一版中央映射里查得到）",
                        entry.head,
                        entry.epoch,
                        entry.unit.describe()
                    )
                },
            );
        }
        judgements.judge(
            REFERENCE_INTERVALS_ARE_EXACT,
            !facts.units_of_the_head.contains(&entry.unit),
            || {
                format!(
                    "{version_name}：deadlist 里一条（头 {}、纪元 {}）钉的单元 {} 活头还引用着（④ 活头引用的块不许出现在任何 deadlist 里）",
                    entry.head,
                    entry.epoch,
                    entry.unit.describe()
                )
            },
        );
        judgements.judge(
            REFERENCE_INTERVALS_ARE_EXACT,
            units_of_some_live_snapshot.contains(&entry.unit) || is_pending_release,
            || {
                format!(
                    "{version_name}：deadlist 里一条（头 {}、纪元 {}）钉的单元 {} 没有一个活快照引用它，也不在未完成销毁意图的待放集里（③ 逆向）",
                    entry.head,
                    entry.epoch,
                    entry.unit.describe()
                )
            },
        );
        if entry.head != facts.head_inode_tree {
            judgements.part_of_the_range_left_unjudged(
                DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE,
                format!("{version_name}：{DEADLIST_ENTRY_OF_ANOTHER_HEAD}"),
            );
            continue;
        }
        let owning_sides: Vec<&DeadlistSide> = sides
            .iter()
            .filter(|side| side.owns_epoch(entry.epoch))
            .collect();
        let owning_side = match owning_sides.as_slice() {
            [owning_side] => owning_side,
            [] if chain_is_contiguous => {
                let earliest_side = sides
                    .first()
                    .expect("各侧里至少有活头那一侧（deadlist_sides 末尾恒接活头）");
                judgements.judge(
                    DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE,
                    is_pending_release,
                    || {
                        format!(
                            "{version_name}：deadlist 里一条（纪元 {}）钉的单元 {} 落不进任何一侧：活快照与活头的前驱链首尾相接，最早那一侧 {} 的前驱是 {}，纪元比它小（每个纪元恰好落进一份区间），也不在未完成销毁意图的待放集里",
                            entry.epoch,
                            entry.unit.describe(),
                            earliest_side.name,
                            earliest_side.previous_snapshot_txg
                        )
                    },
                );
                continue;
            }
            [] | [_, _, ..] => {
                judgements.part_of_the_range_left_unjudged(
                    DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE,
                    format!("{version_name}：{DEADLIST_EPOCHS_NOT_SPLIT_BY_THE_PREVIOUS_SNAPSHOT_CHAIN}"),
                );
                continue;
            }
        };
        judgements.judge(
            DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE,
            is_pending_release || entry.birth_txg() <= owning_side.previous_snapshot_txg,
            || {
                format!(
                    "{version_name}：deadlist 里一条（纪元 {}）钉的单元 {} birth {} 大于它归属那一侧 {} 的前驱 {}（该直接释放的块进了 deadlist），也不在未完成销毁意图的待放集里",
                    entry.epoch,
                    entry.unit.describe(),
                    entry.birth_txg(),
                    owning_side.name,
                    owning_side.previous_snapshot_txg
                )
            },
        );
    }
    for unit in units_of_some_live_snapshot
        .iter()
        .filter(|unit| !facts.units_of_the_head.contains(**unit))
    {
        let latest_live_snapshot_referencing_it = facts
            .live_snapshots
            .iter()
            .rev()
            .find(|snapshot| snapshot.units.contains(*unit))
            .expect("这个单元取自活快照的单元集，至少一个活快照引用它");
        let epoch_upper_bound = next_live_side_upper_bound_after(
            facts,
            latest_live_snapshot_referencing_it.snapshot_txg,
            latest_live_snapshot_referencing_it.snapshot_inode_root_tree,
        );
        let entries_pinning_it: Vec<&DeadlistEntryOfTheScan> = facts
            .deadlist_entries
            .iter()
            .filter(|entry| entry.unit == **unit)
            .collect();
        let is_in_exactly_its_deadlist = match entries_pinning_it.as_slice() {
            [only] => {
                only.head == facts.head_inode_tree
                    && only.epoch >= latest_live_snapshot_referencing_it.snapshot_txg
                    && epoch_upper_bound.is_none_or(|bound| only.epoch < bound)
            }
            [] | [_, _, ..] => false,
        };
        judgements.judge(REFERENCE_INTERVALS_ARE_EXACT, is_in_exactly_its_deadlist, || {
            let epochs: Vec<(u64, u64)> = entries_pinning_it
                .iter()
                .map(|entry| (entry.head, entry.epoch))
                .collect();
            format!(
                "{version_name}：单元 {} 活快照引用（最晚的是码 14 树 {}、S.txg {}）而活头不引用，deadlist 里钉它的（头, 纪元）是 {epochs:?}，要恰好一条、头是活头 {}、纪元落在 [{}, {epoch_upper_bound:?})（③）",
                unit.describe(),
                latest_live_snapshot_referencing_it.snapshot_inode_root_tree,
                latest_live_snapshot_referencing_it.snapshot_txg,
                facts.head_inode_tree,
                latest_live_snapshot_referencing_it.snapshot_txg
            )
        });
    }
}

/// I-3.6 那一条条目的头不是这一版的活头（第一版只有一个头）。
const DEADLIST_ENTRY_OF_ANOTHER_HEAD: &str = "deadlist 里有条目的头 ID 不是这一版树表里那棵活头 inode 树：第一版只有一个头，它归哪一侧没有条款，这几条的紧界判不了";
/// I-3.6 那一条条目的纪元落不进恰好一侧，而前驱链不首尾相接。
const DEADLIST_EPOCHS_NOT_SPLIT_BY_THE_PREVIOUS_SNAPSHOT_CHAIN: &str = "活快照与活头的前驱链没把纪元分成互不相交、首尾相接的几段（前驱链由 I-9.25 判）：有条目落不进恰好一侧，这几条的紧界判不了";
/// I-3.6 那一版的中央映射读不全：deadlist 的 key 查不查得到判不了（C300 那一格）。
const CENTRAL_MAPPING_OF_THE_DEADLIST_VERSION_NOT_READ_IN_FULL: &str = "这一版的中央映射树读不全（读不全由主走读的映射那一遍判）：deadlist 条目的映射 key 在里面查不查得到判不了";

/// 只读快照与 deadlist 那一遍在一版上读不全、配不上的是哪一样：那一版在射程里判不了，报部分没判、逐条列出、不判红。
/// 读不出、解不开、形状不对由主走读判，这一遍只如实记「判不了」。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SnapshotAndDeadlistScanGap {
    TreeTablePointerAllZero,
    TreeTableUnreadable,
    TreeTableEntryShorterThanTheFieldTable,
    DeadlistTreeEntryNotSingle,
    HeadInodeTreeNotSingle,
    HeadExtentTreeNotSingle,
    DeadlistTreeUnreadable,
    HeadTreesUnreadable,
    SnapshotExtentRootNotPaired,
    SnapshotTreesUnreadable,
}

impl SnapshotAndDeadlistScanGap {
    /// 把这一样记成 `invariant` 射程里判不了的一部分：这一版在射程里、在镜像上，它要的那一样读不出、读不全或配不上，报部分没判、
    /// 逐条列出没判到的是哪一样（[`Judgements::part_of_the_range_left_unjudged`]；C701（别的判法静默跳过读不出的部分）
    /// 「一律报部分没判」，不报不适用）。
    pub(super) fn note_as_a_part_of_the_range_left_unjudged(
        self,
        judgements: &mut Judgements,
        invariant: &'static str,
    ) {
        judgements.part_of_the_range_left_unjudged(invariant, self.reason().to_string());
    }

    pub(super) fn reason(self) -> &'static str {
        match self {
            SnapshotAndDeadlistScanGap::TreeTablePointerAllZero => {
                "走读到的某一版树表指针全零（core 照它读盘 0 槽 0、读不出）：那一版的快照与 deadlist 判不了"
            }
            SnapshotAndDeadlistScanGap::TreeTableUnreadable => {
                "走读到的某一版树表读不出（两份都对不上整单元校验和或解不开）：那一版的快照与 deadlist 判不了"
            }
            SnapshotAndDeadlistScanGap::TreeTableEntryShorterThanTheFieldTable => {
                "走读到的某一版树表条目宽不到字段表的 200 字节：那一版的快照与 deadlist 判不了"
            }
            SnapshotAndDeadlistScanGap::DeadlistTreeEntryNotSingle => {
                "走读到的某一版有快照或 deadlist，而树表里 deadlist 树条目不止一条（I-9.16 判）：那一版的 deadlist 判不了"
            }
            SnapshotAndDeadlistScanGap::HeadInodeTreeNotSingle => {
                "走读到的某一版有快照或 deadlist，而树表里活头 inode 树条目不是恰好一条：第一版只有一个头，那一版的 deadlist 判不了"
            }
            SnapshotAndDeadlistScanGap::HeadExtentTreeNotSingle => {
                "走读到的某一版有快照或 deadlist，而树表里头 ID 指活头 inode 树的 extent 树条目不是恰好一条：那一版活头引用了哪些单元判不了"
            }
            SnapshotAndDeadlistScanGap::DeadlistTreeUnreadable => {
                "走读到的某一版 deadlist 树读不全（节点读不出、key 宽不是 43、条目宽不够、层级不是父层级减一）：那一版的 deadlist 判不了"
            }
            SnapshotAndDeadlistScanGap::HeadTreesUnreadable => {
                "走读到的某一版活头的 inode 树或 extent 树读不全：那一版活头引用了哪些单元判不了"
            }
            SnapshotAndDeadlistScanGap::SnapshotExtentRootNotPaired => {
                "走读到的某一版有活快照的码 14 没有恰好一条码 15 配它（I-9.16 ③ 判）：那个快照引用了哪些单元判不了"
            }
            SnapshotAndDeadlistScanGap::SnapshotTreesUnreadable => {
                "走读到的某一版有活快照的 inode 树或 extent 树读不全（被钉的块读不出、或已被复用）：那个快照引用了哪些单元判不了"
            }
        }
    }
}

/// 中央映射 key 里码 2、码 3 的类标签（D18（块里携带什么信息） 已定项 11 的登记表）。
const MAPPING_KEY_UNIT_CLASS_OF_AN_INDEX_NODE: u8 = 2;
const MAPPING_KEY_UNIT_CLASS_OF_A_PACKED_RECORD_UNIT: u8 = 3;

/// 一条码 2 / 码 3 指针（86 字节）指的那个单元的中央映射 key：类标签 + 出生树 + 出生 txg + 实例代号 4 + 出生序号 4 + 补零 2
/// （D19（块指针的结构与宽度预算） 已定项 6 / 已定项 10），各段取指针头里的——core 写这把 key 也是从指针取（`records::mapping_key_for_node`）。
fn central_mapping_key_of_a_node_pointer(
    pointer: &PointerView,
    unit_class_tag: u8,
) -> CentralMappingKey {
    let mut key = vec![0u8; mapping_key_bytes()];
    key[MAPPING_KEY_UNIT_CLASS_OFFSET] = unit_class_tag;
    key[MAPPING_KEY_BIRTH_TREE_OFFSET..MAPPING_KEY_BIRTH_TXG_OFFSET]
        .copy_from_slice(&pointer.birth_tree.to_le_bytes());
    key[MAPPING_KEY_BIRTH_TXG_OFFSET..MAPPING_KEY_INSTANCE_OFFSET]
        .copy_from_slice(&pointer.birth_txg.to_le_bytes());
    key[MAPPING_KEY_INSTANCE_OFFSET..MAPPING_KEY_TAIL_OFFSET]
        .copy_from_slice(&pointer.instance.to_le_bytes());
    let birth_sequence = u32::try_from(pointer.tail).expect(
        "码 2 / 码 3 指针的尾段是出生序号 4 字节（`parse_node_pointer` 只读 4 字节），装得进 u32",
    );
    key[MAPPING_KEY_TAIL_OFFSET..MAPPING_KEY_TAIL_PADDING.start]
        .copy_from_slice(&birth_sequence.to_le_bytes());
    CentralMappingKey(key)
}

/// 一对用户可见树的根（inode 树、extent 树）下面每个单元的中央映射 key，只读不判：inode 树的根与每个内部节点（码 2）、每片叶容器（码 3），
/// extent 树的每个节点（码 2）与每个非全零的数据指针（码 1）。读不出、层级不对、类型段不认得、条目窄于字段表 ⇒ None（判定归主走读）。
fn units_below_the_user_visible_roots_without_judging(
    reader: &dyn ImageReader,
    inode_root: &PointerView,
    extent_root: &PointerView,
    central_mapping: &CentralMappingLocationsOfAVersion,
    cache: &mut IndexNodeCache,
) -> Option<BTreeSet<CentralMappingKey>> {
    let mut units = BTreeSet::new();
    if !inode_root.all_zero {
        units.insert(central_mapping_key_of_a_node_pointer(
            inode_root,
            MAPPING_KEY_UNIT_CLASS_OF_AN_INDEX_NODE,
        ));
        let root = read_index_node_without_judging(
            reader,
            inode_root,
            PositionOfASecondaryRead::TheCentralMappingOfTheVersion(central_mapping),
            cache,
        )?;
        if root.level == 0 {
            if !root.entries.is_empty() {
                return None;
            }
        } else {
            collect_inode_tree_units_below(reader, &root, central_mapping, cache, &mut units)?;
        }
    }
    if !extent_root.all_zero {
        let (node_pointers, data_pointers) =
            extent_tree_without_judging(reader, extent_root, central_mapping, cache)?;
        for node_pointer in &node_pointers {
            units.insert(central_mapping_key_of_a_node_pointer(
                node_pointer,
                MAPPING_KEY_UNIT_CLASS_OF_AN_INDEX_NODE,
            ));
        }
        for data_pointer in data_pointers.iter().filter(|pointer| !pointer.all_zero) {
            units.insert(central_mapping_key_of_a_data_pointer(data_pointer));
        }
    }
    Some(units)
}

/// inode 树一个内部节点下面每个单元的映射 key 收进 `units`：类型段 2 的叶容器（码 3），类型段 0 的内部节点（码 2，层级 = 父层级 − 1 且 ≥ 1）再往下。
/// 递归的上界：每下一层层级减一。
fn collect_inode_tree_units_below(
    reader: &dyn ImageReader,
    node: &crate::IndexNodeView,
    central_mapping: &CentralMappingLocationsOfAVersion,
    cache: &mut IndexNodeCache,
    units: &mut BTreeSet<CentralMappingKey>,
) -> Option<()> {
    if node.entry_width < inode_internal_entry_bytes() {
        return None;
    }
    for entry in &node.entries {
        let child_pointer = parse_node_pointer(&entry[34..120]);
        if child_pointer.all_zero {
            return None;
        }
        match InodeInternalEntryType::of(read_u16(entry, 16)) {
            InodeInternalEntryType::LeafContainer => {
                units.insert(central_mapping_key_of_a_node_pointer(
                    &child_pointer,
                    MAPPING_KEY_UNIT_CLASS_OF_A_PACKED_RECORD_UNIT,
                ));
            }
            InodeInternalEntryType::InternalNode => {
                units.insert(central_mapping_key_of_a_node_pointer(
                    &child_pointer,
                    MAPPING_KEY_UNIT_CLASS_OF_AN_INDEX_NODE,
                ));
                let child = read_index_node_without_judging(
                    reader,
                    &child_pointer,
                    PositionOfASecondaryRead::TheCentralMappingOfTheVersion(central_mapping),
                    cache,
                )?;
                if node.level.checked_sub(1) != Some(child.level) || child.level == 0 {
                    return None;
                }
                collect_inode_tree_units_below(reader, &child, central_mapping, cache, units)?;
            }
            InodeInternalEntryType::Unregistered(_) => return None,
        }
    }
    Some(())
}

/// 一棵多层码 2 树（这里是 deadlist 树）的全部叶条目，只读不判：key 宽不是 `schema` 的宽、叶条目窄于 `leaf_entry_bytes`、内部条目窄于
/// key + 86、孩子层级不是父层级减一、子指针全零 ⇒ None。递归的上界：每下一层层级减一，层级 0 停。
#[allow(
    clippy::too_many_arguments,
    reason = "节点指针、key 布局、叶条目宽、父节点点名的层级、这一版的中央映射、缓存与收集表各有名字与类型，不为凑短签名收成结构体"
)]
pub(super) fn leaf_entries_of_a_multi_level_tree_without_judging(
    reader: &dyn ImageReader,
    node_pointer: &PointerView,
    schema: crate::KeySchema,
    leaf_entry_bytes: usize,
    level_the_parent_names: Option<u8>,
    central_mapping: &CentralMappingLocationsOfAVersion,
    cache: &mut IndexNodeCache,
    leaf_entries: &mut Vec<Vec<u8>>,
) -> Option<()> {
    let node = read_index_node_without_judging(
        reader,
        node_pointer,
        PositionOfASecondaryRead::TheCentralMappingOfTheVersion(central_mapping),
        cache,
    )?;
    let key_width = schema.width();
    if node.key_width != key_width
        || level_the_parent_names.is_some_and(|level| level != node.level)
    {
        return None;
    }
    let Some(child_level) = node.level.checked_sub(1) else {
        if node.entry_width < leaf_entry_bytes {
            return None;
        }
        leaf_entries.extend(node.entries.iter().cloned());
        return Some(());
    };
    if node.entry_width < internal_entry_bytes(schema) {
        return None;
    }
    for entry in &node.entries {
        let child = parse_node_pointer(&entry[key_width..key_width + node_pointer_bytes()]);
        if child.all_zero {
            return None;
        }
        leaf_entries_of_a_multi_level_tree_without_judging(
            reader,
            &child,
            schema,
            leaf_entry_bytes,
            Some(child_level),
            central_mapping,
            cache,
            leaf_entries,
        )?;
    }
    Some(())
}

/// 一版上 I-3.5 ③ ④ 与逆向、I-3.6 要的输入，只读读出来：快照（`snapshots`，调用方从同一张树表解好）、活头两棵树、deadlist 树、
/// 这一版中央映射叶里的 key（从 `mapping_root_pointer` 读）；`checkpoint_txg_of_the_version` 是被判这一版的 txg，原样记下。
/// 这一版一个快照都没有、deadlist 树根指针全零（或树表里没有 deadlist 树条目，mkfs 种的第 0 版树表就是这样）⇒ `Ok(None)`：没有被钉的块，
/// 也没有 deadlist 条目。调用方保证每条条目都不窄于字段表。
#[allow(
    clippy::too_many_arguments,
    reason = "读盘接口、树表条目、快照、这一版的 txg、中央映射的位置表与根指针、缓存各有名字与类型，不为凑短签名收成结构体"
)]
fn deadlist_and_snapshot_reach_of_a_version(
    reader: &dyn ImageReader,
    tree_table_entries: &[Vec<u8>],
    snapshots: &[SnapshotEntryOfTheScan],
    checkpoint_txg_of_the_version: u64,
    central_mapping: &CentralMappingLocationsOfAVersion,
    mapping_root_pointer: &PointerView,
    cache: &mut IndexNodeCache,
) -> Result<Option<DeadlistAndSnapshotReachOfAVersion>, SnapshotAndDeadlistScanGap> {
    let kind_of = |entry: &&Vec<u8>| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET);
    let deadlist_tree_entries: Vec<&Vec<u8>> = tree_table_entries
        .iter()
        .filter(|entry| kind_of(entry) == TREE_KIND_DEADLIST)
        .collect();
    let deadlist_root = match deadlist_tree_entries.as_slice() {
        [] => None,
        [only] => Some(parse_node_pointer(&only[14..100])),
        [_, _, ..] => return Err(SnapshotAndDeadlistScanGap::DeadlistTreeEntryNotSingle),
    }
    .filter(|root| !root.all_zero);
    if snapshots.is_empty() && deadlist_root.is_none() {
        return Ok(None);
    }
    let head_inode_tree_entries: Vec<&Vec<u8>> = tree_table_entries
        .iter()
        .filter(|entry| kind_of(entry) == TREE_KIND_INODE)
        .collect();
    let [head_inode_tree_entry] = head_inode_tree_entries.as_slice() else {
        return Err(SnapshotAndDeadlistScanGap::HeadInodeTreeNotSingle);
    };
    let head_inode_tree = read_u64(head_inode_tree_entry, 0);
    let head_extent_tree_entries: Vec<&Vec<u8>> = tree_table_entries
        .iter()
        .filter(|entry| {
            kind_of(entry) == TREE_KIND_EXTENT
                && read_u64(entry, TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET) == head_inode_tree
        })
        .collect();
    let [head_extent_tree_entry] = head_extent_tree_entries.as_slice() else {
        return Err(SnapshotAndDeadlistScanGap::HeadExtentTreeNotSingle);
    };
    let mut deadlist_leaf_entries = Vec::new();
    if let Some(root) = deadlist_root {
        leaf_entries_of_a_multi_level_tree_without_judging(
            reader,
            &root,
            KEY_SCHEMA_DEADLIST,
            deadlist_entry_bytes(),
            None,
            central_mapping,
            cache,
            &mut deadlist_leaf_entries,
        )
        .ok_or(SnapshotAndDeadlistScanGap::DeadlistTreeUnreadable)?;
    }
    let units_of_the_head = units_below_the_user_visible_roots_without_judging(
        reader,
        &parse_node_pointer(&head_inode_tree_entry[14..100]),
        &parse_node_pointer(&head_extent_tree_entry[14..100]),
        central_mapping,
        cache,
    )
    .ok_or(SnapshotAndDeadlistScanGap::HeadTreesUnreadable)?;
    let mut live_snapshots = Vec::new();
    let mut snapshots_being_destroyed = Vec::new();
    for snapshot in snapshots {
        match snapshot.standing {
            SnapshotStanding::Live => {
                let extent_root = snapshot
                    .extent_root
                    .ok_or(SnapshotAndDeadlistScanGap::SnapshotExtentRootNotPaired)?;
                let units = units_below_the_user_visible_roots_without_judging(
                    reader,
                    &snapshot.inode_root,
                    &extent_root,
                    central_mapping,
                    cache,
                )
                .ok_or(SnapshotAndDeadlistScanGap::SnapshotTreesUnreadable)?;
                live_snapshots.push(LiveSnapshotReach {
                    snapshot_inode_root_tree: snapshot.snapshot_inode_root_tree,
                    snapshot_txg: snapshot.snapshot_txg,
                    previous_snapshot_txg: snapshot.previous_snapshot_txg,
                    units,
                });
            }
            SnapshotStanding::BeingDestroyed => {
                snapshots_being_destroyed.push(SnapshotBeingDestroyedOfTheScan {
                    snapshot_inode_root_tree: snapshot.snapshot_inode_root_tree,
                    snapshot_txg: snapshot.snapshot_txg,
                    previous_snapshot_txg: snapshot.previous_snapshot_txg,
                });
            }
        }
    }
    Ok(Some(DeadlistAndSnapshotReachOfAVersion {
        head_inode_tree,
        head_previous_snapshot_txg: read_u64(
            head_inode_tree_entry,
            TREE_TABLE_ENTRY_PREVIOUS_SNAPSHOT_TXG_OFFSET,
        ),
        units_of_the_head,
        live_snapshots,
        snapshots_being_destroyed,
        deadlist_entries: deadlist_leaf_entries
            .iter()
            .map(|entry| DeadlistEntryOfTheScan::of_entry(entry))
            .collect(),
        checkpoint_txg_of_the_version,
        keys_of_the_central_mapping: central_mapping_keys_of_a_version_without_judging(
            reader,
            mapping_root_pointer,
            cache,
        ),
    }))
}

/// 一版中央映射叶里出现过的每一把 key，只读不判（C300 那一格的「查得到」）：根指针全零 ⇒ 读全了、空集（那一版没有映射）；
/// 读不全 ⇒ [`CentralMappingKeysOfAVersion::NotReadInFull`]。读法同 [`CentralMappingLocationsOfAVersion::read_without_judging`]
/// （[`central_mapping_leaf_entries_without_judging`]），只是 key 在叶里出现两次的也收、读不全时分得出来。
fn central_mapping_keys_of_a_version_without_judging(
    reader: &dyn ImageReader,
    mapping_root_pointer: &PointerView,
    cache: &mut IndexNodeCache,
) -> CentralMappingKeysOfAVersion {
    if mapping_root_pointer.all_zero {
        return CentralMappingKeysOfAVersion::ReadInFull(BTreeSet::new());
    }
    let mut leaf_entries = Vec::new();
    if central_mapping_leaf_entries_without_judging(
        reader,
        mapping_root_pointer,
        None,
        cache,
        &mut leaf_entries,
    )
    .is_none()
    {
        return CentralMappingKeysOfAVersion::NotReadInFull;
    }
    CentralMappingKeysOfAVersion::ReadInFull(
        leaf_entries
            .iter()
            .map(|entry| CentralMappingKey(entry[..mapping_key_bytes()].to_vec()))
            .collect(),
    )
}

/// 只读快照与 deadlist 那一遍：版本清单里每一版重读一遍树表（只读、不进 `walk_failures`），
/// 判 I-3.7 每一版那一格（[`judge_live_snapshots_are_captures_before_their_version`]），记下这一版的快照集合；
/// 树表单元相同的几版内容相同，I-3.5 ③ ④ 与逆向、I-3.6（[`judge_the_deadlist_against_the_head_and_the_live_snapshots`]）只判一次；
/// 走完再判 I-3.7 跨版本那一格（[`judge_live_snapshots_of_consecutive_versions`]）。
/// 主走读按 `visited_units` 只走一遍共享子树，这几条要的是「这一版自己的」快照、活头与 deadlist，所以另走一遍（与删除态扫描同一个缘故）。
/// 树表读不了的一版：I-3.6、I-3.7 报射程里判不了（I-3.5 的出现规则那一半另有它自己的判法，这里不动它）。
pub(super) fn judge_snapshots_and_deadlists_of_the_versions(
    reader: &dyn ImageReader,
    versions: &[VersionToScanTheSnapshotsAndTheDeadlistOf<'_>],
    cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    let mut snapshots_of_the_versions: Vec<SnapshotsOfAVersion> = Vec::new();
    let mut tree_tables_whose_deadlist_was_judged: BTreeSet<(u32, u64, u32)> = BTreeSet::new();
    for version in versions {
        let tree_table_pointer = parse_node_pointer(version.tree_table_pointer);
        let tree_table = if tree_table_pointer.all_zero {
            Err(SnapshotAndDeadlistScanGap::TreeTablePointerAllZero)
        } else {
            match read_index_node_without_judging(
                reader,
                &tree_table_pointer,
                PositionOfASecondaryRead::ThePointerItself,
                cache,
            ) {
                None => Err(SnapshotAndDeadlistScanGap::TreeTableUnreadable),
                Some(tree_table)
                    if tree_table
                        .entries
                        .iter()
                        .any(|entry| entry.len() < tree_table_entry_bytes()) =>
                {
                    Err(SnapshotAndDeadlistScanGap::TreeTableEntryShorterThanTheFieldTable)
                }
                Some(tree_table) => Ok(tree_table),
            }
        };
        let tree_table = match tree_table {
            Ok(tree_table) => tree_table,
            Err(gap) => {
                for invariant in [
                    DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE,
                    LIVE_SNAPSHOTS_ARE_ENUMERABLE,
                ] {
                    gap.note_as_a_part_of_the_range_left_unjudged(judgements, invariant);
                }
                continue;
            }
        };
        let snapshots = snapshot_entries_of_the_tree_table(&tree_table.entries);
        judge_live_snapshots_are_captures_before_their_version(
            &version.name,
            version.checkpoint_txg,
            &snapshots,
            judgements,
        );
        snapshots_of_the_versions.push(SnapshotsOfAVersion {
            name: version.name.clone(),
            checkpoint_txg: version.checkpoint_txg,
            snapshots: snapshots
                .iter()
                .map(|snapshot| {
                    (
                        snapshot.snapshot_inode_root_tree,
                        (snapshot.snapshot_txg, snapshot.standing),
                    )
                })
                .collect(),
        });
        let first_location = tree_table_pointer.locations[0];
        if !tree_tables_whose_deadlist_was_judged.insert((
            first_location.device,
            first_location.slot,
            first_location.checksum,
        )) {
            continue;
        }
        // 这一版自己的中央映射：活头、快照与 deadlist 几棵树里豁免三类之外的单元按它认位置（读不出时是空表，照提示）；
        // deadlist 的 key 也在它的叶里查（C300）。树表单元相同的几版只判第一版：用的是第一版的映射与 txg。
        let mapping_root_pointer = parse_node_pointer(version.mapping_root_pointer);
        let central_mapping = CentralMappingLocationsOfAVersion::read_without_judging(
            reader,
            &mapping_root_pointer,
            cache,
        );
        match deadlist_and_snapshot_reach_of_a_version(
            reader,
            &tree_table.entries,
            &snapshots,
            version.checkpoint_txg,
            &central_mapping,
            &mapping_root_pointer,
            cache,
        ) {
            // 一个快照都没有、deadlist 一条都没有：I-3.6 判一次、成立（空 deadlist，同 I-9.24 空队列报成立）；I-3.5 ③ ④ 没有对象。
            Ok(None) => judgements.judge(
                DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE,
                true,
                String::new,
            ),
            Ok(Some(facts)) => judge_the_deadlist_against_the_head_and_the_live_snapshots(
                &version.name,
                &facts,
                judgements,
            ),
            Err(gap) => {
                for invariant in [
                    REFERENCE_INTERVALS_ARE_EXACT,
                    DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE,
                ] {
                    gap.note_as_a_part_of_the_range_left_unjudged(judgements, invariant);
                }
            }
        }
    }
    judge_live_snapshots_of_consecutive_versions(&snapshots_of_the_versions, judgements);
    judgements.not_applicable(
        DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE,
        "回退候选集里一版树表都没读到：没有 deadlist 可判",
    );
    judgements.not_applicable(
        LIVE_SNAPSHOTS_ARE_ENUMERABLE,
        "回退候选集里一版树表都没读到：没有快照集合可判",
    );
}

#[cfg(test)]
mod tests;
