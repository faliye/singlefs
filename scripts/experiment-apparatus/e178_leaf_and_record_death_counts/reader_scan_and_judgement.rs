//! 读者的扫描：读数、真值、父链与覆盖层、扫描输入、候选与择新、单读者的判定。

use super::*;

// ---------------------------------------------------------------------------------------------
// 读者（登记 5.4）：候选 → 择新 → 底座 → 重建 inode 树与整对象墓碑 → 数据单元 → 计数
// ---------------------------------------------------------------------------------------------

/// 一次扫描一个读者的计数（登记第一节读法 1、6）：r = |R − T|、m = |T − R|，三个名字空间各报；O1、O2 崩溃点用。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ReaderCounts {
    pub(super) inode_resurrected: u64,
    pub(super) inode_missed: u64,
    pub(super) extent_resurrected: u64,
    pub(super) extent_missed: u64,
    pub(super) container_resurrected: u64,
    pub(super) container_missed: u64,
    /// O1：重建出的记录 nlink > 0、而真值里 nlink > 0 的对象集合没有它。
    pub(super) live_records_beyond_the_truth: u64,
    /// O2：重建视图里有记录（nlink 不论）、而这一版里它的意图已发布。
    pub(super) records_whose_intent_is_published: u64,
    /// 读出的 inode 记录条数（第十一节 V8 用）。
    pub(super) inode_records_read: u64,
    /// 择中的零条目类型 2 版本个数（修订 2 之五「读到的桩数」，V-Bs3 用；原写路径恒 0）。
    pub(super) stubs_read: u64,
    /// B-tsz：P-C 与 P-W 选出的记录不同的 inode 号个数（修订 2 之七 Q-Bt-2；别的读者恒 0）。
    pub(super) provenance_differences: u64,
}

pub(super) const COUNTED_QUANTITIES: [&str; 6] =
    ["inode_r", "inode_m", "extent_r", "extent_m", "container_r", "container_m"];

impl ReaderCounts {
    pub(super) const fn quantities(self) -> [u64; 6] {
        [
            self.inode_resurrected,
            self.inode_missed,
            self.extent_resurrected,
            self.extent_missed,
            self.container_resurrected,
            self.container_missed,
        ]
    }
}

/// 真值：被扫那一版里的 key 集合。
#[derive(Clone, Debug, Default)]
pub(super) struct Truth {
    pub(super) inode_keys: HashSet<(u64, u64)>,
    live_inode_keys: HashSet<(u64, u64)>,
    pub(super) extent_keys: HashSet<(u64, u64, u64)>,
    container_identities: HashSet<ContainerIdentity>,
}

pub(super) fn truth_of(version: &Version) -> Truth {
    let mut truth = Truth::default();
    // 真值只收记录数 ≥ 1 的容器身份，桩不算活容器（修订 2 之五容器名字空间的口径）。
    for leaf in version.leaves.iter().filter(|leaf| !leaf.records.is_empty()) {
        truth.container_identities.insert(leaf.identity);
        for record in leaf.records.iter() {
            truth.inode_keys.insert((record.inode, record.object_birth));
            if record.nlink > 0 {
                truth.live_inode_keys.insert((record.inode, record.object_birth));
            }
        }
    }
    for (object, file) in &version.files {
        for unit_index_in_file in 0..length_as_u64(file.units.len()) {
            truth.extent_keys.insert((*object, file.object_birth, unit_index_in_file));
        }
    }
    truth
}

/// 父链（I-9.18）：从被扫那一版的 txg 往回走，有谱系记录 (T_rb, T_old) 的 T_rb 的父是 T_old，其余 txg 的父是 txg − 1。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ParentChain {
    inclusive_ranges: Vec<(u64, u64)>,
}

impl ParentChain {
    pub(super) fn of(scanned_txg: u64, lineage: &[(u64, u64)]) -> Self {
        let mut inclusive_ranges = Vec::new();
        let mut upper = scanned_txg;
        loop {
            let jump = lineage
                .iter()
                .copied()
                .filter(|(rollback_txg, _)| *rollback_txg <= upper)
                .max_by_key(|(rollback_txg, _)| *rollback_txg);
            match jump {
                Some((rollback_txg, old_txg)) => {
                    assert!(old_txg < rollback_txg, "回退回到更早的一版");
                    inclusive_ranges.push((rollback_txg, upper));
                    upper = old_txg;
                }
                None => {
                    inclusive_ranges.push((1, upper));
                    return Self { inclusive_ranges };
                }
            }
        }
    }

    pub(super) fn contains(&self, txg: u64) -> bool {
        self.inclusive_ranges.iter().any(|(lowest, highest)| *lowest <= txg && txg <= *highest)
    }
}

/// 阳性对照对读者输入的改动：植入的单元（全新身份）、抹掉的单元、拿掉的一条墓碑记录（各条墓碑写流里同字段的那条一起拿掉）。
#[derive(Clone, Debug, Default)]
pub(super) struct Overlay {
    planted: Vec<Unit>,
    erased: Option<UnitIndex>,
    removed_tombstone: Option<TombstoneRecord>,
    /// PC-Bt2：供出规则倒过来（取写序最小的那一条）。
    reverse_provenance: bool,
    /// PC-Bt3：整对象墓碑不按谱系滤。
    whole_object_tombstones_ignore_lineage: bool,
}

impl Overlay {
    pub(super) fn planting(planted: Vec<Unit>) -> Self {
        Self { planted, erased: None, removed_tombstone: None, reverse_provenance: false, whole_object_tombstones_ignore_lineage: false }
    }

    pub(super) fn erasing(erased: UnitIndex) -> Self {
        Self { planted: Vec::new(), erased: Some(erased), removed_tombstone: None, reverse_provenance: false, whole_object_tombstones_ignore_lineage: false }
    }

    pub(super) fn removing_a_tombstone(removed: TombstoneRecord) -> Self {
        Self { planted: Vec::new(), erased: None, removed_tombstone: Some(removed), reverse_provenance: false, whole_object_tombstones_ignore_lineage: false }
    }

    pub(super) fn reversing_the_provenance() -> Self {
        Self { planted: Vec::new(), erased: None, removed_tombstone: None, reverse_provenance: true, whole_object_tombstones_ignore_lineage: false }
    }

    pub(super) fn ignoring_the_lineage_of_whole_object_tombstones() -> Self {
        Self { planted: Vec::new(), erased: None, removed_tombstone: None, reverse_provenance: false, whole_object_tombstones_ignore_lineage: true }
    }
}

pub(super) struct ScanInputs<'scan> {
    pub(super) disk: &'scan Disk,
    pub(super) overlay: &'scan Overlay,
    pub(super) scan_publish: u64,
    pub(super) scanned_txg: u64,
    pub(super) current_instance: u64,
    pub(super) current_transaction: u64,
    pub(super) instance_rows: &'scan BTreeMap<u64, u64>,
    pub(super) chain: ParentChain,
}

impl ScanInputs<'_> {
    pub(super) fn unit(&self, unit_index: UnitIndex) -> &Unit {
        if unit_index < self.disk.units.len() {
            &self.disk.units[unit_index]
        } else {
            &self.overlay.planted[unit_index - self.disk.units.len()]
        }
    }

    fn unit_count_with_the_planted(&self) -> usize {
        self.disk.units.len() + self.overlay.planted.len()
    }
}

fn is_readable(unit: &Unit, scan_publish: u64, window: ReadableWindow) -> bool {
    match unit.replaced_at_publish {
        None => true,
        Some(replaced_at) => window.allows(scan_publish.saturating_sub(replaced_at)),
    }
}

/// 已发布谓词（D18 已定项 11 的实例表、C113 P2）：现行实例按现行那一版的事务号判，旧实例按实例表那一行判，没有行的旧实例算已发布。
fn is_published_by_the_instance_table(unit: &Unit, inputs: &ScanInputs<'_>) -> bool {
    let instance = unit.write_order.instance;
    if instance == inputs.current_instance {
        return unit.write_order.transaction <= inputs.current_transaction;
    }
    match inputs.instance_rows.get(&instance) {
        Some(applied_transaction) => unit.write_order.transaction <= *applied_transaction,
        None => instance < inputs.current_instance,
    }
}

/// 候选：读得出、已发布、诞生 txg 在被扫那一版的父链上且不大于它的 txg（登记 5.4 第 1 条）。
pub(super) fn is_candidate(unit: &Unit, inputs: &ScanInputs<'_>, window: ReadableWindow) -> bool {
    let birth_txg = unit.write_order.birth_txg;
    is_readable(unit, inputs.scan_publish, window)
        && is_published_by_the_instance_table(unit, inputs)
        && birth_txg <= inputs.scanned_txg
        && inputs.chain.contains(birth_txg)
}

/// 类型 2 版本的候选要不要看「诞生 txg 在父链上」：只有 B-stub-lin-off（PC-Bs2）不看。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum InodeLeafLineage {
    Required,
    Ignored,
}

/// 进择新的单元：候选（登记 5.4 第 1 条）；零条目版本照样进，B-stub 的桩靠写序最大被择中（修订 2 之五）。
fn enters_the_selection(unit: &Unit, inputs: &ScanInputs<'_>, window: ReadableWindow, lineage: InodeLeafLineage) -> bool {
    match (lineage, &unit.body) {
        (InodeLeafLineage::Ignored, UnitBody::InodeLeaf { .. }) => {
            is_published_by_the_instance_table(unit, inputs)
                && is_readable(unit, inputs.scan_publish, window)
                && unit.write_order.birth_txg <= inputs.scanned_txg
        }
        (InodeLeafLineage::Required, _)
        | (InodeLeafLineage::Ignored, UnitBody::Data { .. } | UnitBody::Tombstones { .. } | UnitBody::Retirements { .. }) => {
            is_candidate(unit, inputs, window)
        }
    }
}

/// 择新之后每组留下的那一个单元，按类分开、按单元序号升序。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SelectedUnits {
    pub(super) inode_leaves: Vec<UnitIndex>,
    tombstones: Vec<Vec<UnitIndex>>,
    retirements: Vec<UnitIndex>,
    pub(super) data: Vec<UnitIndex>,
}

impl SelectedUnits {
    pub(super) fn empty() -> Self {
        Self {
            inode_leaves: Vec::new(),
            tombstones: vec![Vec::new(); TOMBSTONE_STREAM_COUNT],
            retirements: Vec::new(),
            data: Vec::new(),
        }
    }

    pub(super) fn push(&mut self, category: GroupCategory, unit_index: UnitIndex) {
        match category {
            GroupCategory::Data => self.data.push(unit_index),
            GroupCategory::InodeLeaf => self.inode_leaves.push(unit_index),
            GroupCategory::Tombstones(stream) => self.tombstones[stream.position()].push(unit_index),
            GroupCategory::Retirements => self.retirements.push(unit_index),
        }
    }

    pub(super) fn sort(&mut self) {
        self.inode_leaves.sort_unstable();
        for stream_units in &mut self.tombstones {
            stream_units.sort_unstable();
        }
        self.retirements.sort_unstable();
        self.data.sort_unstable();
    }
}

/// 主路径：按组的版本清单从新往旧找第一个候选（版本清单按写入次序，写序随写入次序单调）。
pub(super) fn select_by_the_group_index(inputs: &ScanInputs<'_>, window: ReadableWindow, lineage: InodeLeafLineage) -> SelectedUnits {
    let mut selected = SelectedUnits::empty();
    for (category, groups) in &inputs.disk.groups_by_category {
        for group in groups {
            let newest_candidate = inputs.disk.versions_of_group[*group].iter().rev().copied().find(|unit_index| {
                inputs.overlay.erased != Some(*unit_index) && enters_the_selection(inputs.unit(*unit_index), inputs, window, lineage)
            });
            if let Some(unit_index) = newest_candidate {
                selected.push(*category, unit_index);
            }
        }
    }
    for (offset, planted) in inputs.overlay.planted.iter().enumerate() {
        if enters_the_selection(planted, inputs, window, lineage) {
            selected.push(category_of(planted.body.group_key()), inputs.disk.units.len() + offset);
        }
    }
    selected.sort();
    selected
}

/// 参照路径：整扫一遍盘上每个单元（加植入的、去掉抹掉的），每组按 (写序, 单元序号) 取最大，不读版本清单。
pub(super) fn select_by_a_full_scan(inputs: &ScanInputs<'_>, window: ReadableWindow, lineage: InodeLeafLineage) -> SelectedUnits {
    let mut newest: BTreeMap<GroupKey, (WriteOrder, UnitIndex)> = BTreeMap::new();
    for unit_index in 0..inputs.unit_count_with_the_planted() {
        if inputs.overlay.erased == Some(unit_index) {
            continue;
        }
        let unit = inputs.unit(unit_index);
        if !enters_the_selection(unit, inputs, window, lineage) {
            continue;
        }
        let order = (unit.write_order, unit_index);
        newest
            .entry(unit.body.group_key())
            .and_modify(|best| {
                if order > *best {
                    *best = order;
                }
            })
            .or_insert(order);
    }
    let mut selected = SelectedUnits::empty();
    for (key, (_, unit_index)) in newest {
        selected.push(category_of(key), unit_index);
    }
    selected.sort();
    selected
}

/// 整对象墓碑对 (o, b) 施不施行：E-I、E-L-w 直接施行；E-L-r 只在施行之前的树里没有 (o, b) 时施行（登记 5.3、5.4 第 4 条）。
const fn whole_object_tombstone_applies(effect: UnlinkEffectArm, object_is_in_the_tree_before_tombstones: bool) -> bool {
    match effect {
        UnlinkEffectArm::AtIntent | UnlinkEffectArm::AtLastBatchWritten => true,
        UnlinkEffectArm::AtLastBatchRead => !object_is_in_the_tree_before_tombstones,
    }
}

/// 一次扫描读得到的、与读者无关的东西（真值、两档 W 的择新结果）。
pub(super) struct ScanContext<'scan> {
    pub(super) inputs: ScanInputs<'scan>,
    pub(super) truth: Truth,
    pub(super) objects_with_published_intent: &'scan BTreeSet<u64>,
}

/// 一个读者在择新结果上读出 R、与真值比（登记 5.4 第 3–6 条；两条择新路径共用这一段）。
pub(super) fn judge_reader(selected: &SelectedUnits, reader: Reader, context: &ScanContext<'_>) -> ReaderCounts {
    let inputs = &context.inputs;
    let truth = &context.truth;
    // 第 3 条 底座：一个身份有一条可施行的退役记录、且这一版的诞生 txg ≤ 它的死亡 txg ⇒ 丢掉这一版。
    let mut retirement_deaths: HashMap<ContainerIdentity, u64> = HashMap::new();
    for unit_index in &selected.retirements {
        let retirement_unit = inputs.unit(*unit_index);
        let UnitBody::Retirements { records, .. } = &retirement_unit.body else {
            continue;
        };
        for retirement in records.iter() {
            let applies = match reader.base {
                BaseArm::RetirementOnLineage => inputs.chain.contains(retirement.death_txg),
                BaseArm::RetirementIgnoringLineage => true,
                BaseArm::WithoutRetirement
                | BaseArm::ZeroRecordStubs
                | BaseArm::ZeroRecordStubsIgnoringLineage
                | BaseArm::TombstonesOnly => false,
            };
            if applies {
                let death = retirement_deaths.entry(retirement.identity).or_insert(retirement.death_txg);
                *death = (*death).max(retirement.death_txg);
            }
        }
    }
    // 第 4 条 重建 inode 树：同号多条时按供出规则取一条（原登记 P-W：写序最大的那个容器里的；B-tsz 取 P-C）。
    let provenance = if inputs.overlay.reverse_provenance { Provenance::OldestWriteOrder } else { provenance_of(reader.base) };
    let mut tree_before_tombstones: HashMap<u64, (InodeRecord, (WriteOrder, UnitIndex))> = HashMap::new();
    let mut newest_write_order_choice: HashMap<u64, (WriteOrder, UnitIndex)> = HashMap::new();
    let mut containers_read: Vec<ContainerIdentity> = Vec::new();
    let mut counts = ReaderCounts::default();
    for unit_index in &selected.inode_leaves {
        let leaf_unit = inputs.unit(*unit_index);
        let UnitBody::InodeLeaf { identity, records } = &leaf_unit.body else {
            continue;
        };
        if retirement_deaths.get(identity).is_some_and(|death_txg| leaf_unit.write_order.birth_txg <= *death_txg) {
            continue;
        }
        // 择中的零条目版本（B-stub 的桩）不出记录、不算读出的容器，另数一个「读到的桩数」（修订 2 之五）。
        if records.is_empty() {
            counts.stubs_read += 1;
            continue;
        }
        containers_read.push(*identity);
        let candidate_order = (leaf_unit.write_order, *unit_index);
        for record in records.iter() {
            let keeps_the_existing = tree_before_tombstones.get(&record.inode).is_some_and(|(existing_record, existing_order)| match provenance {
                Provenance::NewestWriteOrder => *existing_order >= candidate_order,
                Provenance::LargestChangeCounter => {
                    (existing_record.change_counter, *existing_order) >= (record.change_counter, candidate_order)
                }
                Provenance::OldestWriteOrder => *existing_order <= candidate_order,
            });
            if !keeps_the_existing {
                tree_before_tombstones.insert(record.inode, (*record, candidate_order));
            }
            if provenance == Provenance::LargestChangeCounter {
                let newest = newest_write_order_choice.entry(record.inode).or_insert(candidate_order);
                *newest = (*newest).max(candidate_order);
            }
        }
    }
    if provenance == Provenance::LargestChangeCounter {
        counts.provenance_differences = length_as_u64(
            tree_before_tombstones
                .iter()
                .filter(|(inode, (_, chosen_order))| newest_write_order_choice.get(*inode).is_some_and(|newest| newest != chosen_order))
                .count(),
        );
    }
    let objects_in_the_tree: HashSet<(u64, u64)> =
        tree_before_tombstones.values().map(|(record, _)| (record.inode, record.object_birth)).collect();
    // 这个读者那条墓碑写流里在谱系上的整对象墓碑与区间墓碑。
    let stream = reader.tombstone_stream();
    let mut whole_object_deaths: HashMap<(u64, u64), u64> = HashMap::new();
    let mut range_tombstones: HashMap<(u64, u64), Vec<(u64, u64)>> = HashMap::new();
    for unit_index in &selected.tombstones[stream.position()] {
        let UnitBody::Tombstones { records, .. } = &inputs.unit(*unit_index).body else {
            continue;
        };
        for tombstone in records.iter() {
            if inputs.overlay.removed_tombstone == Some(*tombstone) {
                continue;
            }
            let object_key = (tombstone.object, tombstone.object_birth);
            if tombstone.range_start_bytes == WHOLE_OBJECT_RANGE_START_BYTES {
                let whole_object_tombstone_is_on_the_chain = inputs.chain.contains(tombstone.death_txg);
                if whole_object_tombstone_is_on_the_chain || inputs.overlay.whole_object_tombstones_ignore_lineage {
                    let death = whole_object_deaths.entry(object_key).or_insert(tombstone.death_txg);
                    *death = (*death).max(tombstone.death_txg);
                }
            } else {
                let range_tombstone_is_on_the_chain = inputs.chain.contains(tombstone.death_txg);
                if range_tombstone_is_on_the_chain {
                    range_tombstones.entry(object_key).or_default().push((tombstone.range_start_bytes, tombstone.death_txg));
                }
            }
        }
    }
    // inode 读出 = 施行整对象墓碑之前的树，去掉被施行的 (o, b)（读法 2）。
    let mut inode_hits: u64 = 0;
    for (inode, (record, _)) in &tree_before_tombstones {
        let object_key = (*inode, record.object_birth);
        if whole_object_deaths.contains_key(&object_key) && whole_object_tombstone_applies(reader.effect, true) {
            continue;
        }
        counts.inode_records_read += 1;
        if truth.inode_keys.contains(&object_key) {
            inode_hits += 1;
        } else {
            counts.inode_resurrected += 1;
        }
        if record.nlink > 0 && !truth.live_inode_keys.contains(&object_key) {
            counts.live_records_beyond_the_truth += 1;
        }
        if context.objects_with_published_intent.contains(inode) {
            counts.records_whose_intent_is_published += 1;
        }
    }
    counts.inode_missed = difference_or_marker(length_as_u64(truth.inode_keys.len()), inode_hits);
    // 第 5 条 数据单元：按 X2 臂判活（用施行墓碑之前的树），再按 X3 臂施行整对象墓碑。
    let mut extent_hits: u64 = 0;
    for unit_index in &selected.data {
        let data_unit = inputs.unit(*unit_index);
        let Some((object, object_birth, unit_index_in_file)) = data_key_of(data_unit) else {
            continue;
        };
        let object_key = (object, object_birth);
        let object_is_in_the_tree = objects_in_the_tree.contains(&object_key);
        let unit_birth_txg = data_unit.write_order.birth_txg;
        let alive_by_the_shrink_arm = match reader.shrink {
            ShrinkArm::RecordSize => tree_before_tombstones.get(&object).is_some_and(|(record, _)| {
                record.object_birth == object_birth && unit_index_in_file < record.data_units()
            }),
            ShrinkArm::RangeTombstone => {
                object_is_in_the_tree
                    && !range_tombstones.get(&object_key).is_some_and(|ranges| {
                        ranges.iter().any(|(range_start, range_death_txg)| {
                            unit_index_in_file * DATA_UNIT_PAYLOAD_BYTES >= *range_start && unit_birth_txg <= *range_death_txg
                        })
                    })
            }
            ShrinkArm::ObjectGateOnly => object_is_in_the_tree,
        };
        if !alive_by_the_shrink_arm {
            continue;
        }
        let killed_by_a_whole_object_tombstone = whole_object_deaths.get(&object_key).is_some_and(|whole_death_txg| {
            whole_object_tombstone_applies(reader.effect, object_is_in_the_tree) && unit_birth_txg <= *whole_death_txg
        });
        if killed_by_a_whole_object_tombstone {
            continue;
        }
        if truth.extent_keys.contains(&(object, object_birth, unit_index_in_file)) {
            extent_hits += 1;
        } else {
            counts.extent_resurrected += 1;
        }
    }
    counts.extent_missed = difference_or_marker(length_as_u64(truth.extent_keys.len()), extent_hits);
    let mut container_hits: u64 = 0;
    for identity in &containers_read {
        if truth.container_identities.contains(identity) {
            container_hits += 1;
        } else {
            counts.container_resurrected += 1;
        }
    }
    counts.container_missed = difference_or_marker(length_as_u64(truth.container_identities.len()), container_hits);
    counts
}

/// 一次扫描留下的东西：每个读者的计数（次序同 `every_reader`）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ScanRecord {
    pub(super) publish_sequence: u64,
    pub(super) txg: u64,
    pub(super) inside_an_unlink: bool,
    pub(super) after_a_rollback: bool,
    pub(super) truth_inode_keys: u64,
    /// 这一版有没有一个变短过的活对象（PC-Z 取最后一次有的那次扫描），有就是号最小的那个。
    pub(super) live_shrunk_object: Option<u64>,
    pub(super) counts: Vec<ReaderCounts>,
}

/// 择新走哪条路（第十一节 V3 的两路）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SelectionPath {
    GroupIndex,
    FullScan,
}

#[cfg(test)]
#[path = "reader_scan_and_judgement_tests.rs"]
mod tests;
