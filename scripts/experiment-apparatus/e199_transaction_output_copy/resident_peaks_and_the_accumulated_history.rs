//! 乙 2 的额外跳数、各式的常驻峰值、累计 unlink 与丁的整段历史。

use super::*;

// ── 读路径跳数与常驻峰值（第一份第五节「读路径与常驻峰值」；第二份加现算重建与残余式） ──

/// 乙2 读一条墓碑记录、读一个单元比甲多走几跳。
pub(super) fn yi2_extra_hops(geometry: &Geometry) -> (u64, u64) {
    let fanout = geometry.knobs.persistent_fanout;
    let block = geometry.knobs.persistent_leaf_block;
    let tombstones = &geometry.tombstones;
    let record = persistent_depth(tombstones.containers.div_ceil(fanout), fanout) + 1 + persistent_depth(tombstones.fill.div_ceil(block), fanout) + 1;
    let unit = persistent_depth(geometry.units.div_ceil(fanout), fanout) + 1;
    (record, unit)
}

/// 甲的常驻峰值 = |V| + |V'| + 一轮准入前照抄（O + Rl）+ 一轮现算重建（准入前那一份 ÷ s）+ 准入后现算重建。
pub(super) fn jia_resident_peak(geometry: &Geometry) -> u64 {
    let rounds = jia_after_patch_copied_before_admission(geometry);
    let (recomputed_before, recomputed_after) = recomputed_in_full(geometry);
    version_bytes(geometry)
        + version_bytes_after(geometry)
        + rounds.ordinary_allocation_pass
        + rounds.settle_round
        + recomputed_before / geometry.knobs.settle_rounds
        + recomputed_after
}

/// 丙的常驻峰值 = |V| + 新装 + 撤回日志 + 被换下的 k 个容器与 ρ 个节点的旧字节 + 残余式（准入前 ÷ s 与准入后）。
pub(super) fn bing_resident_peak(geometry: &Geometry) -> u64 {
    let replaced_old_bytes: u64 =
        geometry.container_trees().iter().map(|tree| tree.open_container_rewritten * UNIT_BYTES + tree.rewritten_index_nodes * NODE_BYTES).sum();
    let (residual_before, residual_after) = recomputed_residual(geometry);
    version_bytes(geometry)
        + installed_bytes(geometry).1
        + bing_copied_after_admission(geometry)
        + replaced_old_bytes
        + residual_before / geometry.knobs.settle_rounds
        + residual_after
}

// ── 两段历史：丁的 256 次连发与 10⁶ 次 unlink 累计（第六节；第十四节：串内不切，串与串之间独立） ──

/// 丁的压实周期取样点与主值（第八节逐一扫；G0 取 p = 64）。
pub(super) const DING_PERIODS: [u64; 7] = [1, 2, 3, 4, 8, 64, 1024];
pub(super) const DING_MAIN_PERIOD: u64 = 64;
/// 256 次连发（G0 起，a_T = 1、u 逐次加一）。
pub(super) const DING_HISTORY_PUBLISHES: u64 = 256;
/// 10⁶ 次 unlink（每次 unlink 发两次：意图 a_T = 1、最后一批 a_T = 0）。
pub(super) const CUMULATIVE_UNLINKS: u64 = 1_000_000;

fn add_arm_bytes(totals: &mut ArmBytes, bytes: ArmBytes) {
    totals.copied_before += bytes.copied_before;
    totals.copied_after += bytes.copied_after;
    totals.recomputed_before += bytes.recomputed_before;
    totals.recomputed_after += bytes.recomputed_after;
}

/// 丁这一次发布：压实那次照压实方式照抄，现算重建准入后取全量；非压实那次照抄 0、残余式。
fn ding_bytes(geometry: &Geometry, compacts_now: bool, compaction: Compaction) -> ArmBytes {
    let residual = recomputed_residual(geometry);
    ArmBytes {
        copied_before: 0,
        copied_after: ding_copied_after_admission(geometry, compacts_now, compaction),
        recomputed_before: residual.0,
        recomputed_after: if compacts_now { recomputed_in_full(geometry).1 } else { residual.1 },
    }
}

/// 一段历史上每臂的合计。
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(super) struct HistoryTotals {
    pub(super) publishes: u64,
    pub(super) jia: ArmBytes,
    yi2: ArmBytes,
    bing: ArmBytes,
    pub(super) ding: ArmBytes,
    pub(super) installed_before: u64,
    pub(super) installed_after: u64,
}

impl HistoryTotals {
    pub(super) fn add_publish(&mut self, geometry: &Geometry, ding: ArmBytes) {
        self.publishes += 1;
        add_arm_bytes(&mut self.jia, arm_bytes(geometry, Arm::JiaAfterPatch));
        add_arm_bytes(&mut self.yi2, arm_bytes(geometry, Arm::Yi2));
        add_arm_bytes(&mut self.bing, arm_bytes(geometry, Arm::Bing));
        add_arm_bytes(&mut self.ding, ding);
        let (installed_before, installed_after) = installed_bytes(geometry);
        self.installed_before += installed_before;
        self.installed_after += installed_after;
    }

    fn merge(&mut self, other: &HistoryTotals) {
        self.publishes += other.publishes;
        add_arm_bytes(&mut self.jia, other.jia);
        add_arm_bytes(&mut self.yi2, other.yi2);
        add_arm_bytes(&mut self.bing, other.bing);
        add_arm_bytes(&mut self.ding, other.ding);
        self.installed_before += other.installed_before;
        self.installed_after += other.installed_after;
    }

    pub(super) fn arm(&self, arm: Arm) -> ArmBytes {
        match arm {
            Arm::Yi2 => self.yi2,
            Arm::Bing => self.bing,
            Arm::DingWithoutCompaction | Arm::DingWholeNewBaseCompaction => self.ding,
            Arm::JiaAfterPatch | Arm::JiaToday => self.jia,
        }
    }
}

/// 序号落在 `unlinks` 里的那几次 unlink（第 k 次：u = k 上的意图，u = k + 1 上的最后一批）；丁照 p 在第 p、2p、… 次发布压实。
fn cumulative_unlinks_between(unlinks: std::ops::Range<u64>, ding_period: u64) -> HistoryTotals {
    let mut totals = HistoryTotals::default();
    for unlink in unlinks {
        let intent = Knobs { tombstone_records: unlink, tombstones_added: 1, ..Knobs::main_geometry() };
        let final_batch = Knobs { tombstone_records: unlink + 1, tombstones_added: 0, ..Knobs::main_geometry() };
        for (publish_number, knobs) in [(2 * unlink + 1, intent), (2 * unlink + 2, final_batch)] {
            let geometry = Geometry::new(knobs);
            let ding = ding_bytes(&geometry, publish_number % ding_period == 0, Compaction::WholeNewBase);
            totals.add_publish(&geometry, ding);
        }
    }
    totals
}

/// 把 [0, unlinks) 按序号切成 `threads` 片并行算，按片的次序合并（合计是整数加法，与线程数无关）。
pub(super) fn cumulative_unlinks(unlinks: u64, threads: usize) -> HistoryTotals {
    let slices = (threads.max(1) as u64).min(unlinks.max(1));
    let slice_length = unlinks.div_ceil(slices);
    let ranges: Vec<std::ops::Range<u64>> =
        (0..slices).map(|slice| (slice * slice_length).min(unlinks)..((slice + 1) * slice_length).min(unlinks)).collect();
    let partials: Vec<HistoryTotals> = std::thread::scope(|scope| {
        let handles: Vec<_> =
            ranges.iter().cloned().map(|range| scope.spawn(move || cumulative_unlinks_between(range, DING_MAIN_PERIOD))).collect();
        handles.into_iter().map(|handle| handle.join().expect("累计那一片的线程不该 panic")).collect()
    });
    let mut totals = HistoryTotals::default();
    for partial in &partials {
        totals.merge(partial);
    }
    totals
}

/// 丁一个 p、一种压实方式在 256 次连发上的轨迹与合计。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct DingHistory {
    pub(super) period: u64,
    pub(super) compaction: Compaction,
    pub(super) compaction_copied_total: u64,
    pub(super) ding: ArmBytes,
    pub(super) layer_peak: u64,
    pub(super) layer_positive_publishes: u64,
    pub(super) layer_final: u64,
    pub(super) resident_peak: u64,
    pub(super) resident_final: u64,
}

/// 256 次连发里第 n 次（从 1 数）的几何：u = 10⁶ + n − 1、a_T = 1，其余取 G0。
pub(super) fn ding_history_geometries() -> Vec<Geometry> {
    let main = Knobs::main_geometry();
    (0..DING_HISTORY_PUBLISHES)
        .map(|offset| Geometry::new(Knobs { tombstone_records: main.tombstone_records + offset, ..main }))
        .collect()
}

/// 丁第 n 次发布的常驻字节（第四份第五节「常驻峰值」丁那一式）：基版 + 已有各层 + 这次准入后新装 + [压实且整份新基]·|V'|
/// + 残余式准入前 ÷ s + 准入后那一份（压实那次取甲-B2后的准入后现算重建，两种压实方式都取它；不压实取残余式）。
fn ding_resident_bytes(geometry: &Geometry, base_version: u64, layer_bytes: u64, compacts_now: bool, compaction: Compaction) -> u64 {
    let (residual_before, residual_after) = recomputed_residual(geometry);
    let new_base_copy = if compacts_now && compaction == Compaction::WholeNewBase { version_bytes_after(geometry) } else { 0 };
    let recomputed_after_in_the_peak = if compacts_now { recomputed_in_full(geometry).1 } else { residual_after };
    base_version + layer_bytes + installed_bytes(geometry).1 + new_base_copy + residual_before / geometry.knobs.settle_rounds + recomputed_after_in_the_peak
}

pub(super) fn ding_history(geometries: &[Geometry], period: u64, compaction: Compaction) -> DingHistory {
    let mut history = DingHistory {
        period,
        compaction,
        compaction_copied_total: 0,
        ding: ArmBytes::default(),
        layer_peak: 0,
        layer_positive_publishes: 0,
        layer_final: 0,
        resident_peak: 0,
        resident_final: 0,
    };
    let mut base_version = geometries.first().map(version_bytes).unwrap_or(0);
    let mut layer_bytes = 0;
    for (offset, geometry) in geometries.iter().enumerate() {
        let publish_number = offset as u64 + 1;
        let compacts_now = publish_number.is_multiple_of(period);
        let bytes = ding_bytes(geometry, compacts_now, compaction);
        history.compaction_copied_total += bytes.copied_after;
        add_arm_bytes(&mut history.ding, bytes);
        let installed_after = installed_bytes(geometry).1;
        let resident = ding_resident_bytes(geometry, base_version, layer_bytes, compacts_now, compaction);
        history.resident_peak = history.resident_peak.max(resident);
        history.resident_final = resident;
        if compacts_now {
            base_version = version_bytes_after(geometry);
            layer_bytes = 0;
        } else {
            layer_bytes += installed_after;
        }
        let layers = publish_number % period;
        history.layer_peak = history.layer_peak.max(layers);
        history.layer_positive_publishes += u64::from(layers > 0);
        history.layer_final = layers;
    }
    history
}

#[cfg(test)]
#[path = "resident_peaks_and_the_accumulated_history_tests.rs"]
mod tests;
