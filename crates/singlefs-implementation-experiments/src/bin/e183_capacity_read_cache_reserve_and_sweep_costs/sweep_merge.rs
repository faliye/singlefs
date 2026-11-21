use super::*;

// ============================================================================================
// H5 的合并：片齐了按片的次序接成产物，再算 E37 里 H5 的判定（R5b 甲、R5c、G5、几何敏感性、判别力自证 ②、PC-不扫、V35、T4）。
// ============================================================================================

/// 一格阈值臂的汇总（读回 `h5_cell_summary` 行）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ThresholdCellSummary {
    pub(super) cell: SweepHistoryCell,
    pub(super) threshold_rows: u64,
    writable_mounts: u64,
    pub(super) total_write_bytes: u64,
    pub(super) peak_chain_pages: u64,
    differential_control_holds: bool,
}

fn threshold_cell_summary(
    cell: SweepHistoryCell,
    line: &ResultLine,
) -> Option<ThresholdCellSummary> {
    Some(ThresholdCellSummary {
        cell,
        threshold_rows: cell.arm.threshold_rows()?,
        writable_mounts: line.number("writable_mounts")?,
        total_write_bytes: line.number("total_write_bytes")?,
        peak_chain_pages: line.number("peak_chain_pages")?,
        differential_control_holds: line
            .truth("positive_control_differential_below_the_threshold")?
            && line.truth("positive_control_differential_at_the_first_sweep")?,
    })
}

/// R5b 甲：链重写那一项的片数峰值 = 1 ⇒ 不越过读法甲的上界。
fn chain_pages_stay_on_one_page(peak_chain_pages: u64) -> bool {
    peak_chain_pages == 1
}

/// R5c：W_θ ÷ W_最小 的千分比（W = 整段历史写字节 ÷ 可写挂载次数，交叉相乘免得先除丢精度）。
fn write_amplification_ratio_permille(
    cell: &ThresholdCellSummary,
    smallest: &ThresholdCellSummary,
) -> u128 {
    u128::from(cell.total_write_bytes) * u128::from(smallest.writable_mounts) * 1000
        / (u128::from(cell.writable_mounts) * u128::from(smallest.total_write_bytes)).max(1)
}

fn write_amplification_acceptable(ratio_permille: u128, boundary_permille: u128) -> bool {
    ratio_permille <= boundary_permille
}

/// 同一 (S, 卸载方式) 里 W 最小的那一格（PC-差分 没过的格不参加，V35）。
fn smallest_write_per_mount(cells: &[ThresholdCellSummary]) -> Option<ThresholdCellSummary> {
    cells
        .iter()
        .filter(|cell| cell.differential_control_holds && cell.writable_mounts > 0)
        .min_by(|left, right| {
            (u128::from(left.total_write_bytes) * u128::from(right.writable_mounts))
                .cmp(&(u128::from(right.total_write_bytes) * u128::from(left.writable_mounts)))
        })
        .copied()
}

/// 判别力自证 ②（登记第八节）：把 1.10 挪到真产物两个相邻比值之间（取中点，按两倍千分比算免得取整），至少一个 θ 的判定必须转。
fn acceptance_boundary_moved_between_real_ratios(ratios: &[u128]) -> Option<(u128, usize)> {
    let mut sorted = ratios.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    sorted
        .windows(2)
        .map(|pair| pair[0] + pair[1])
        .map(|doubled_boundary| {
            let flipped = ratios
                .iter()
                .filter(|ratio| {
                    write_amplification_acceptable(**ratio, WRITE_AMPLIFICATION_ACCEPTABLE_PERMILLE)
                        != (2 * **ratio <= doubled_boundary)
                })
                .count();
            (doubled_boundary, flipped)
        })
        .filter(|(_, flipped)| *flipped > 0)
        .min_by_key(|(doubled_boundary, _)| {
            doubled_boundary.abs_diff(2 * WRITE_AMPLIFICATION_ACCEPTABLE_PERMILLE)
        })
}

/// 读回片目录里的一片：做完的交回不含末行的那些行；不在、没做完交回 `None`。
pub(super) fn finished_shard_lines(directory: &Path, shard: &str) -> Option<Vec<String>> {
    let text = std::fs::read_to_string(directory.join(shard)).ok()?;
    lines_of_a_finished_shard(&text, shard)
}

/// 读回的一片：片名、格、行。
type HistoryShard = (String, SweepHistoryCell, Vec<ResultLine>);

/// E37 里 H5 的判定，接在全部片的行之后。
fn emit_sweep_summary(emitter: &mut Emitter, shards: &[HistoryShard]) -> MergeEnd {
    let summaries: Vec<ThresholdCellSummary> = shards
        .iter()
        .filter(|(shard, _, _)| !shard.starts_with(SWEEP_REPEAT_SHARD_PREFIX))
        .filter_map(|(_, cell, lines)| {
            threshold_cell_summary(
                *cell,
                lines_named(lines, "h5_cell_summary").into_iter().next()?,
            )
        })
        .collect();
    let mut ratios: Vec<u128> = Vec::new();
    let mut verdicts_by_threshold: BTreeMap<u64, Vec<(bool, Option<bool>)>> = BTreeMap::new();
    let mut points_without_a_candidate = 0;
    let mut differential_control_failed_cells = 0;
    for slots_per_region in SLOTS_PER_REGION_SAMPLING_POINTS {
        for unmount_kind in [UnmountKind::Normal, UnmountKind::Crash] {
            let point: Vec<ThresholdCellSummary> = summaries
                .iter()
                .filter(|summary| {
                    summary.cell.slots_per_region == slots_per_region
                        && summary.cell.unmount_kind == unmount_kind
                })
                .copied()
                .collect();
            let smallest = smallest_write_per_mount(&point);
            let mut passing_both = Vec::new();
            for summary in &point {
                let one_page = chain_pages_stay_on_one_page(summary.peak_chain_pages);
                differential_control_failed_cells +=
                    usize::from(!summary.differential_control_holds);
                let ratio = smallest
                    .filter(|_| summary.differential_control_holds)
                    .map(|smallest| write_amplification_ratio_permille(summary, &smallest));
                let acceptable = ratio.map(|ratio| {
                    write_amplification_acceptable(ratio, WRITE_AMPLIFICATION_ACCEPTABLE_PERMILLE)
                });
                ratios.extend(ratio);
                if one_page && acceptable == Some(true) {
                    passing_both.push(summary.threshold_rows);
                }
                verdicts_by_threshold
                    .entry(summary.threshold_rows)
                    .or_default()
                    .push((one_page, acceptable));
                emitter.emit(&format!(
                    "name=h5_threshold_verdicts cell={} slots_per_region={slots_per_region} unmount_kind={} threshold_rows={} peak_chain_pages={} r5b_reading_one_page={} writable_mounts={} total_write_bytes={} ratio_to_the_smallest_write_per_mount_permille={} r5c={}",
                    summary.cell.name(),
                    unmount_kind.name(),
                    summary.threshold_rows,
                    summary.peak_chain_pages,
                    if one_page { "not_crossing" } else { "crossing" },
                    summary.writable_mounts,
                    summary.total_write_bytes,
                    ratio.map_or_else(|| "none".to_string(), |ratio| ratio.to_string()),
                    match acceptable {
                        Some(true) => "acceptable",
                        Some(false) => "not_acceptable",
                        None => "void_differential_control_failed",
                    },
                ));
            }
            points_without_a_candidate += usize::from(passing_both.is_empty());
            emitter.emit(&format!(
                "name=h5_point slots_per_region={slots_per_region} unmount_kind={} cells={} thresholds_passing_both={} no_candidate_passes_both={}",
                unmount_kind.name(),
                point.len(),
                joined_or_none(&passing_both),
                passing_both.is_empty(),
            ));
        }
    }
    let mut geometry_unstable = Vec::new();
    for (threshold, verdicts) in &verdicts_by_threshold {
        let page_verdicts: BTreeSet<bool> =
            verdicts.iter().map(|(one_page, _)| *one_page).collect();
        let write_amplification_verdicts: BTreeSet<Option<bool>> =
            verdicts.iter().map(|(_, acceptable)| *acceptable).collect();
        if page_verdicts.len() > 1 || write_amplification_verdicts.len() > 1 {
            geometry_unstable.push(*threshold);
        }
        emitter.emit(&format!(
            "name=h5_geometry_sensitivity threshold_rows={threshold} points={} r5b_stable_across_points={} r5c_stable_across_points={}",
            verdicts.len(),
            page_verdicts.len() == 1,
            write_amplification_verdicts.len() == 1,
        ));
    }
    let moved = acceptance_boundary_moved_between_real_ratios(&ratios);
    emitter.emit(&format!(
        "name=discrimination_self_proof judgement=h5_r5c_write_amplification real_ratios={} moved_doubled_boundary_permille={} flipped={} holds={}",
        ratios.len(),
        moved.map_or_else(|| "none".to_string(), |(boundary, _)| boundary.to_string()),
        moved.map_or(0, |(_, flipped)| flipped),
        moved.is_some(),
    ));
    emit_sweep_verdict(
        emitter,
        shards,
        SweepVerdictInputs {
            differential_control_failed_cells,
            chain_read_differential_failed_cells: chain_read_differential_failed_cells(shards),
            points_without_a_candidate,
            geometry_unstable,
            self_proof_holds: moved.is_some(),
        },
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SweepVerdictInputs {
    differential_control_failed_cells: usize,
    chain_read_differential_failed_cells: usize,
    points_without_a_candidate: usize,
    geometry_unstable: Vec<u64>,
    pub(super) self_proof_holds: bool,
}

/// PC-差分新臂没过的格数：阈值臂的每一格（段首重跑的那一片不算）要有一条 `h5_chain_read_differential` 行，行里的判定是真；缺行按没过。
fn chain_read_differential_failed_cells(shards: &[HistoryShard]) -> usize {
    shards
        .iter()
        .filter(|(shard, cell, _)| {
            !shard.starts_with(SWEEP_REPEAT_SHARD_PREFIX) && cell.arm.threshold_rows().is_some()
        })
        .filter(|(_, _, lines)| {
            lines_named(lines, "h5_chain_read_differential")
                .into_iter()
                .next()
                .and_then(|line| line.truth("read_bytes_difference_equals_the_chain_read"))
                != Some(true)
        })
        .count()
}

/// H5 的判决行：PC-不扫（V34）、PC-差分（V35）、G5、几何敏感性、判别力自证 ②、T4（段首那一格的重跑与原片逐字段比）各报各的。
fn emit_sweep_verdict(
    emitter: &mut Emitter,
    shards: &[HistoryShard],
    inputs: SweepVerdictInputs,
) -> MergeEnd {
    let control = shards
        .iter()
        .find(|(_, cell, _)| cell.arm == SweepArm::HandedToTheBackground)
        .and_then(|(_, _, lines)| {
            lines_named(lines, "h5_cell_summary")
                .into_iter()
                .next()
                .cloned()
        });
    let control_field = |key: &str| {
        control
            .as_ref()
            .and_then(|line| line.field(key))
            .unwrap_or("not_run")
            .to_string()
    };
    let first = shards
        .iter()
        .find(|(shard, _, _)| !shard.starts_with(SWEEP_REPEAT_SHARD_PREFIX));
    let repeat = shards
        .iter()
        .find(|(shard, _, _)| shard.starts_with(SWEEP_REPEAT_SHARD_PREFIX));
    let repeat_difference = match (first, repeat) {
        (Some((_, _, first_lines)), Some((_, _, repeat_lines))) => {
            first_line_differing_outside_the_timing_fields(first_lines, repeat_lines)
        }
        (Some(_), None) | (None, Some(_)) | (None, None) => Some(0),
    };
    let crossing: BTreeSet<u64> = shards
        .iter()
        .filter(|(shard, _, _)| !shard.starts_with(SWEEP_REPEAT_SHARD_PREFIX))
        .filter_map(|(_, cell, lines)| {
            let summary = threshold_cell_summary(
                *cell,
                lines_named(lines, "h5_cell_summary").into_iter().next()?,
            )?;
            (!chain_pages_stay_on_one_page(summary.peak_chain_pages))
                .then_some(summary.threshold_rows)
        })
        .collect();
    emitter.emit(&format!(
        "name=verdict item=h5_summary positive_control_no_sweep_publish={} positive_control_rows0_plus_one_each_mount={} control_final_rows0={} control_final_rows0_equals_the_revised_registered_value={} differential_control_failed_cells={} chain_read_differential_failed_cells={} r5b_crossing_thresholds={} points_without_a_candidate={} geometry_unstable_thresholds={} discrimination_self_proof_holds={} repeat_of_the_first_shard_equal_outside_timing={} repeat_first_differing_line={}",
        control_field("positive_control_no_sweep_publish"),
        control_field("positive_control_rows0_plus_one_each_mount"),
        control_field("final_rows0"),
        control_field("final_rows0_equals_the_registered_value"),
        inputs.differential_control_failed_cells,
        inputs.chain_read_differential_failed_cells,
        joined_or_none(&crossing.into_iter().collect::<Vec<_>>()),
        inputs.points_without_a_candidate,
        joined_or_none(&inputs.geometry_unstable),
        inputs.self_proof_holds,
        repeat_difference.is_none(),
        optional_number_text(repeat_difference.map(|line| u64::try_from(line).expect("行号"))),
    ));
    match repeat_difference {
        None => MergeEnd::Complete,
        Some(_) => MergeEnd::Stopped {
            reason: "t4_repeat_of_the_first_shard_differs".to_string(),
        },
    }
}

/// `h5 merge`：片齐了（每一片的程序文件 sha256 与合并给的相同）按片的次序接成产物打到标准输出，接着打判定；缺片或程序文件不同 ⇒ 停机行、退 3。
pub(super) fn run_sweep_merge(directory: &Path, binary_sha256: &str) -> i32 {
    let mut emitter = Emitter::to_standard_output();
    emitter.emit(&format!(
        "name=config experiment=e183 segment=3 scope=h5 shards={} binary_sha256={binary_sha256}",
        sweep_shards().len()
    ));
    let mut shards: Vec<HistoryShard> = Vec::new();
    for (shard, cell) in sweep_shards() {
        let Some(lines) = finished_shard_lines(directory, &shard) else {
            emitter.emit(&format!(
                "name=stop reason=shard_missing_or_unfinished shard={shard}"
            ));
            emitter.finish();
            return 3;
        };
        let parsed: Vec<ResultLine> = lines
            .iter()
            .filter_map(|line| ResultLine::parse(line))
            .collect();
        let sha256 = lines_named(&parsed, "h5_config")
            .into_iter()
            .next()
            .and_then(|line| line.field("binary_sha256"))
            .unwrap_or("none")
            .to_string();
        if sha256 != binary_sha256 {
            emitter.emit(&format!(
                "name=stop reason=binary_sha256_differs shard={shard} shard_binary_sha256={sha256}"
            ));
            emitter.finish();
            return 3;
        }
        for line in &lines {
            let body = line.strip_prefix("E7RESULT ").unwrap_or(line);
            emitter.emit(body);
        }
        shards.push((shard, cell, parsed));
    }
    let end = emit_sweep_summary(&mut emitter, &shards);
    finish_a_merge(&mut emitter, end)
}

#[cfg(test)]
mod tests;
