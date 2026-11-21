//! 量的轨迹、随机数与删除次序、各种历史的规格与回放。

use super::*;

impl Simulation {
    pub(super) fn scan_inputs<'scan>(&'scan self, overlay: &'scan Overlay, scan_publish: u64) -> ScanInputs<'scan> {
        ScanInputs {
            disk: &self.disk,
            overlay,
            scan_publish,
            scanned_txg: self.version.txg,
            current_instance: self.version.instance,
            current_transaction: self.version.transaction,
            instance_rows: &self.instance_rows,
            chain: ParentChain::of(self.version.txg, &self.version.lineage),
        }
    }

    /// 这条写路径的读者（第二、三段 20 个）各读一遍现行那一版（植入、抹掉的照 `overlay` 改读者输入）。
    pub(super) fn read_every_reader(&self, overlay: &Overlay, scan_publish: u64, path: SelectionPath) -> Vec<ReaderCounts> {
        self.read_readers(&self.readers, overlay, scan_publish, path)
    }

    /// 给定的读者各读一遍现行那一版；B-stub-lin-off 用类型 2 不看谱系的那一份择新（PC-Bs2）。
    pub(super) fn read_readers(&self, readers: &[Reader], overlay: &Overlay, scan_publish: u64, path: SelectionPath) -> Vec<ReaderCounts> {
        let context = ScanContext {
            inputs: self.scan_inputs(overlay, scan_publish),
            truth: truth_of(&self.version),
            objects_with_published_intent: &self.version.objects_with_published_intent,
        };
        let select = |window: ReadableWindow, lineage: InodeLeafLineage| match path {
            SelectionPath::GroupIndex => select_by_the_group_index(&context.inputs, window, lineage),
            SelectionPath::FullScan => select_by_a_full_scan(&context.inputs, window, lineage),
        };
        let forever = select(ReadableWindow::Forever, InodeLeafLineage::Required);
        let ring = select(ReadableWindow::RingWindow, InodeLeafLineage::Required);
        let lineage_ignored = readers.iter().any(|reader| reader.base == BaseArm::ZeroRecordStubsIgnoringLineage).then(|| {
            (select(ReadableWindow::Forever, InodeLeafLineage::Ignored), select(ReadableWindow::RingWindow, InodeLeafLineage::Ignored))
        });
        readers
            .iter()
            .map(|reader| {
                let ignores_lineage = reader.base == BaseArm::ZeroRecordStubsIgnoringLineage;
                let selected = match (reader.window, ignores_lineage, &lineage_ignored) {
                    (ReadableWindow::Forever, true, Some((ignored_forever, _))) => ignored_forever,
                    (ReadableWindow::RingWindow, true, Some((_, ignored_ring))) => ignored_ring,
                    (ReadableWindow::Forever, _, _) => &forever,
                    (ReadableWindow::RingWindow, _, _) => &ring,
                };
                judge_reader(selected, *reader, &context)
            })
            .collect()
    }

    pub(super) fn live_shrunk_object(&self) -> Option<u64> {
        self.largest_size_before_a_shrink.keys().copied().find(|object| self.version.is_live(*object))
    }

    pub(super) fn scan_the_current_version(&self, scan_publish: u64) -> ScanRecord {
        ScanRecord {
            publish_sequence: scan_publish,
            txg: self.version.txg,
            inside_an_unlink: self.version.is_inside_an_unlink(),
            after_a_rollback: !self.version.lineage.is_empty(),
            truth_inode_keys: length_as_u64(self.version.leaves.iter().map(|leaf| leaf.records.len()).sum()),
            live_shrunk_object: self.live_shrunk_object(),
            counts: self.read_every_reader(&Overlay::default(), scan_publish, SelectionPath::GroupIndex),
        }
    }
}

/// 一个量的轨迹（第八节）：峰值、峰值所在的发布序号、为正的扫描次数、期末值（这一类扫描的最后一次）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct QuantityTrajectory {
    pub(super) scans: u64,
    pub(super) peak: u64,
    pub(super) peak_at_publish: u64,
    pub(super) positive_scans: u64,
    pub(super) last: u64,
}

impl QuantityTrajectory {
    pub(super) fn observe(&mut self, value: u64, publish_sequence: u64) {
        self.scans += 1;
        if value > self.peak {
            self.peak = value;
            self.peak_at_publish = publish_sequence;
        }
        self.positive_scans += u64::from(value > 0);
        self.last = value;
    }

    pub(super) fn absorb(&mut self, other: QuantityTrajectory) {
        self.scans += other.scans;
        if other.peak > self.peak {
            self.peak = other.peak;
            self.peak_at_publish = other.peak_at_publish;
        }
        self.positive_scans += other.positive_scans;
        self.last = self.last.max(other.last);
    }

    pub(super) fn fields(self, prefix: &str) -> String {
        format!(
            "{prefix}_peak={} {prefix}_peak_at={} {prefix}_positive_scans={} {prefix}_last={}",
            self.peak, self.peak_at_publish, self.positive_scans, self.last
        )
    }
}

/// 扫描按什么分类（第一节读法 2、4，Q-B2「回退之后」）。
pub(super) const SCAN_SCOPES: [&str; 5] = ["all", "inside_unlink", "outside_unlink", "after_rollback", "after_rollback_outside_unlink"];

pub(super) const fn scan_scopes_of(record: &ScanRecord) -> [bool; 5] {
    [
        true,
        record.inside_an_unlink,
        !record.inside_an_unlink,
        record.after_a_rollback,
        record.after_a_rollback && !record.inside_an_unlink,
    ]
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ReaderTrajectory {
    pub(super) by_scope: [[QuantityTrajectory; 6]; 5],
    /// 读出的 inode 记录集合为空而真值不空的扫描数（第十一节 V8 的触发计数）。
    pub(super) read_out_empty_scans: u64,
}

pub(super) fn trajectories_of(scans: &[ScanRecord], reader_count: usize) -> Vec<ReaderTrajectory> {
    let mut trajectories = vec![ReaderTrajectory::default(); reader_count];
    for record in scans {
        let scopes = scan_scopes_of(record);
        for (reader_position, counts) in record.counts.iter().enumerate() {
            let trajectory = &mut trajectories[reader_position];
            for (scope_position, in_scope) in scopes.iter().enumerate() {
                if *in_scope {
                    for (quantity_position, value) in counts.quantities().iter().enumerate() {
                        trajectory.by_scope[scope_position][quantity_position].observe(*value, record.publish_sequence);
                    }
                }
            }
            trajectory.read_out_empty_scans += u64::from(counts.inode_records_read == 0 && record.truth_inode_keys > 0);
        }
    }
    trajectories
}

// ---------------------------------------------------------------------------------------------
// 历史族（登记 5.2）：H51 照 E177 装置逐段重放，F10、F11、F12 是本实验加的
// ---------------------------------------------------------------------------------------------

/// 带种子的确定性随机源（splitmix64，E177 装置第 4566–4586 行同一写法）。
#[derive(Clone, Debug)]
pub(super) struct SeededRandom {
    state: u64,
}

impl SeededRandom {
    pub(super) const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_value(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut mixed = self.state;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        mixed ^ (mixed >> 31)
    }

    pub(super) fn below(&mut self, bound: u64) -> u64 {
        self.next_value() % bound
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DeletionOrder {
    Ascending,
    EvenThenOdd,
    Random(u64),
}

impl DeletionOrder {
    pub(super) fn label(self) -> String {
        match self {
            DeletionOrder::Ascending => "ascending".to_string(),
            DeletionOrder::EvenThenOdd => "even_then_odd".to_string(),
            DeletionOrder::Random(seed) => format!("random_seed_{seed}"),
        }
    }
}

pub(super) fn deletion_orders() -> Vec<DeletionOrder> {
    let mut orders = vec![DeletionOrder::Ascending, DeletionOrder::EvenThenOdd];
    orders.extend((1..=5).map(DeletionOrder::Random));
    orders
}

/// 删序：inode 2..n 的一个排列（E177 装置 `deletion_order_of`，第 4709–4730 行）。
pub(super) fn deletion_order_of(inode_count: u64, order: DeletionOrder) -> Vec<u64> {
    let inodes: Vec<u64> = (FIRST_FILE_INODE + 1..=inode_count).collect();
    match order {
        DeletionOrder::Ascending => inodes,
        DeletionOrder::EvenThenOdd => inodes
            .iter()
            .copied()
            .filter(|inode| inode % 2 == 0)
            .chain(inodes.iter().copied().filter(|inode| inode % 2 == 1))
            .collect(),
        DeletionOrder::Random(seed) => {
            let mut shuffled = inodes;
            let mut random = SeededRandom::new(seed);
            for position in (1..shuffled.len()).rev() {
                let other = u64_as_index(random.below(length_as_u64(position) + 1));
                shuffled.swap(position, other);
            }
            shuffled
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum History {
    /// H51 的 F1：建删空 inode。
    BuildAndDeleteEmptyInodes { inode_count: u64, order: DeletionOrder },
    /// H51 的 F8：大文件改写删。
    LargeFileRewriteAndDelete { data_units: u64, rewrites: u64, overwrite_before_unlink: bool, batch: u64 },
    /// H51 的 F3：混合随机。
    MixedRandom { seed: u64 },
    /// F10：多对象带数据、借与吸收。
    ManyObjectsWithData { inode_count: u64, order: DeletionOrder, batch: u64 },
    /// F11：回退越过一个缺口事件。
    RollBackOverAGap { event: GapEvent },
}

impl History {
    pub(super) const fn family(self) -> &'static str {
        match self {
            History::BuildAndDeleteEmptyInodes { .. } => "F1",
            History::LargeFileRewriteAndDelete { .. } => "F8",
            History::MixedRandom { .. } => "F3",
            History::ManyObjectsWithData { .. } => "F10",
            History::RollBackOverAGap { .. } => "F11",
        }
    }

    pub(super) fn label(self) -> String {
        match self {
            History::BuildAndDeleteEmptyInodes { inode_count, order } => {
                format!("family=F1 n={inode_count} order={}", order.label())
            }
            History::LargeFileRewriteAndDelete { data_units, rewrites, overwrite_before_unlink, batch } => format!(
                "family=F8 n_u={data_units} k={rewrites} e={} batch={batch}",
                if overwrite_before_unlink { "OU" } else { "U" }
            ),
            History::MixedRandom { seed } => format!("family=F3 seed={seed}"),
            History::ManyObjectsWithData { inode_count, order, batch } => {
                format!("family=F10 n={inode_count} order={} batch={batch}", order.label())
            }
            History::RollBackOverAGap { event } => format!("family=F11 n={ROLLBACK_OVER_A_GAP_INODE_COUNT} gap_event={} batch={PRODUCT_UNLINK_BATCH_DATA_UNITS}", event.label()),
        }
    }

    pub(super) const fn batch(self) -> u64 {
        match self {
            History::LargeFileRewriteAndDelete { batch, .. } | History::ManyObjectsWithData { batch, .. } => batch,
            History::BuildAndDeleteEmptyInodes { .. } | History::MixedRandom { .. } | History::RollBackOverAGap { .. } => {
                PRODUCT_UNLINK_BATCH_DATA_UNITS
            }
        }
    }
}

/// H51：F1 14 段、F8 32 段、F3 5 段（E177 装置 `segment_one_histories`，第 4670–4697 行）。
pub(super) fn histories_replayed_from_the_e177_segment_one() -> Vec<History> {
    let mut histories = Vec::new();
    for inode_count in [300, 900] {
        for order in deletion_orders() {
            histories.push(History::BuildAndDeleteEmptyInodes { inode_count, order });
        }
    }
    for data_units in [1, 3, 145, 300] {
        for rewrites in [0, 5] {
            for overwrite_before_unlink in [false, true] {
                for batch in [1, PRODUCT_UNLINK_BATCH_DATA_UNITS] {
                    histories.push(History::LargeFileRewriteAndDelete { data_units, rewrites, overwrite_before_unlink, batch });
                }
            }
        }
    }
    for seed in 1..=5 {
        histories.push(History::MixedRandom { seed });
    }
    histories
}

/// F10：n ∈ {300, 900} × 删序 7 种 × 批 ∈ {1, 144}，28 段。
pub(super) fn many_objects_with_data_histories() -> Vec<History> {
    let mut histories = Vec::new();
    for inode_count in [300, 900] {
        for order in deletion_orders() {
            for batch in [1, PRODUCT_UNLINK_BATCH_DATA_UNITS] {
                histories.push(History::ManyObjectsWithData { inode_count, order, batch });
            }
        }
    }
    histories
}

/// E177 产物 `research/results/e177-per-tree-resurrection-without-tombstones-segment-one.out` 里 F3 五段、`geometry=wide` 那两条
/// `name=simulation` 行的 `publishes=` 与 `refused_steps=`（登记第七节 A1；两条写路径的两行相同，只读这两段）。
pub(super) const E177_MIXED_RANDOM_PUBLISHES_AND_REFUSALS: [(u64, u64, u64); 5] =
    [(1, 414, 0), (2, 414, 0), (3, 328, 95), (4, 334, 89), (5, 347, 85)];

/// F10、F11 的建与写：每次建 50 个空 inode 到共 n 个；对 i = 2..n 各顺序写 1 + (i mod 5) 个单元（各一次发布）。
pub(super) fn build_and_write_many_objects(simulation: &mut Simulation, inode_count: u64) {
    let mut created = 1;
    while created < inode_count {
        let count = NEW_INODES_PER_STEP.min(inode_count - created);
        simulation.apply(UserChange::NewInodes { count });
        created += count;
    }
    for object in FIRST_FILE_INODE + 1..=inode_count {
        simulation.apply(UserChange::WriteFile { object, data_units: 1 + object % 5 });
    }
}

#[cfg(test)]
#[path = "trajectories_and_histories_tests.rs"]
mod tests;
