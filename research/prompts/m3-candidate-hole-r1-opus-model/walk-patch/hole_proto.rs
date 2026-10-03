// ===== m3-candidate-hole-r1 攻方原型（只在攻方副本里，经 include! 并进 walk.rs）：洞表与 H1 / H2 的候选判法 =====
// 开关（环境变量，每次 check_pool_image 现读）：
//   HOLE_SKIP = rows（默认，今天的 gap-row）| union（行证据 ∪ journal 里已落未施加的记录）| naive（同 union，但「施加过」只认实例表行）
//             | counter（攻方提的变体，零轮：journal 里那条记录说它存在过就压过行证据）
//   HOLE_I31  = 空（今天）| na（洞表里有「存在过却走不到」就整条不适用）| e1（区间形，字面：只改丁-defer 门槛）
//             | e1w（区间形，宽读法：上沿并进最新根账里释放代高于下沿、遍历没走到的全部已释放槽）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtoHoleStatus {
    Present,
    Abandoned,
    AppliedByRecords,
    Skipped,
    ExistedUnreachable,
}

thread_local! {
    static PROTO_HOLE_TABLE: std::cell::RefCell<BTreeMap<u64, ProtoHoleStatus>> = const { std::cell::RefCell::new(BTreeMap::new()) };
    static PROTO_HOLE_REPORT: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
}

fn proto_env(name: &str) -> String {
    std::env::var(name).unwrap_or_default()
}

/// 上一次 check_pool_image 留下的洞表与 I-3.1 的区间（原型打印用）。
#[must_use]
pub fn proto_take_hole_report() -> String {
    PROTO_HOLE_REPORT.with(|report| std::mem::take(&mut *report.borrow_mut()))
}

fn proto_note(line: String) {
    PROTO_HOLE_REPORT.with(|report| {
        let mut report = report.borrow_mut();
        report.push_str(&line);
        report.push('\n');
    });
}

/// 洞表：窗口里 [max(F, N − 3S + 1), N] 每个 txg 的状态（调查员第十节第二张表，一格只取「那一版存不存在」那一列）。
#[allow(clippy::too_many_arguments)]
fn proto_hole_table_store(
    roots: &[(u64, u64, crate::RootView)],
    newest_instance: u32,
    newest_txg: u64,
    rows: &[InstanceTableRow],
    applied: &[VersionAppliedOnlyByRecords],
    records_by_device: &[(u32, BTreeMap<u64, ScannedJournalRecord>)],
    window: OneRevolutionWindow,
    effective_rollback_floor: u64,
) {
    PROTO_HOLE_TABLE.with(|table| table.borrow_mut().clear());
    PROTO_HOLE_REPORT.with(|report| report.borrow_mut().clear());
    let evidence = proto_env("HOLE_SKIP");
    let abandoned = |instance: u32, txg: u64| {
        rows.iter().any(|row| row.instance == instance && txg > row.published_checkpoint_txg)
    };
    let low = effective_rollback_floor.max((newest_txg + 1).saturating_sub(window.root_ring_slot_count));
    // 锚：窗口里没被抛弃的根与由记录施加出来的版本（候选集里那几版）。
    let mut anchors: BTreeMap<u64, u32> = BTreeMap::new();
    for (_, _, root) in roots {
        if root.checkpoint_txg >= low && root.checkpoint_txg <= newest_txg && !abandoned(root.instance, root.checkpoint_txg) {
            anchors.insert(root.checkpoint_txg, root.instance);
        }
    }
    for version in applied {
        anchors.insert(version.checkpoint_txg, version.instance);
    }
    let mut table: BTreeMap<u64, ProtoHoleStatus> = BTreeMap::new();
    for t in low..=newest_txg {
        let root_here: Vec<&crate::RootView> = roots.iter().map(|(_, _, r)| r).filter(|r| r.checkpoint_txg == t).collect();
        let status = if root_here.iter().any(|r| !abandoned(r.instance, t)) {
            ProtoHoleStatus::Present
        } else if !root_here.is_empty() {
            ProtoHoleStatus::Abandoned
        } else if applied.iter().any(|v| v.checkpoint_txg == t) {
            ProtoHoleStatus::AppliedByRecords
        } else {
            let previous = anchors.range(..t).next_back().map(|(txg, inst)| (*txg, *inst));
            let next = anchors.range(t + 1..).next().map(|(txg, inst)| (*txg, *inst));
            let rows_skip = match (previous, next) {
                (Some((pt, pi)), Some((nt, ni))) => {
                    proto_hole_by_rows(pt, &BTreeSet::from([pi]), nt, ni, rows)
                        == HoleInTheCandidateSet::HoleBridgedByTheInstanceTableRows
                }
                _ => false,
            };
            let committed: Vec<&ScannedJournalRecord> = records_by_device
                .iter()
                .flat_map(|(_, records)| records.values())
                .filter(|record| record.checkpoint_txg == t && record.commit_marker == CommitMarker::Present)
                .collect();
            let covered_by_a_row = |record: &ScannedJournalRecord| {
                rows.iter().any(|row| row.instance == record.instance && record.checkpoint_txg <= row.published_checkpoint_txg)
            };
            let existed_by_journal = committed.iter().any(|record| {
                covered_by_a_row(record) || (evidence != "naive" && record.instance == newest_instance)
            });
            let unapplied_by_journal = committed.iter().any(|record| {
                !(covered_by_a_row(record) || (evidence != "naive" && record.instance == newest_instance))
            });
            let skipped = match evidence.as_str() {
                "union" | "naive" => rows_skip || unapplied_by_journal,
                "counter" => !existed_by_journal && (rows_skip || unapplied_by_journal),
                _ => rows_skip,
            };
            if skipped { ProtoHoleStatus::Skipped } else { ProtoHoleStatus::ExistedUnreachable }
        };
        table.insert(t, status);
    }
    let shown: Vec<String> = table
        .iter()
        .filter(|(_, s)| **s != ProtoHoleStatus::Present)
        .map(|(t, s)| format!("{t}:{s:?}"))
        .collect();
    proto_note(format!("HOLE_TABLE evidence={evidence:?} window=[{low},{newest_txg}] non_present={shown:?}"));
    PROTO_HOLE_TABLE.with(|stored| *stored.borrow_mut() = table);
}

/// I-3.5 缺口那一判在原型开关下怎么答：HOLE_SKIP 不是 rows 时，(R⁻, R) 之间每个 txg 在洞表里都是「取号跳过」或「被抛弃」才算接得上。
fn proto_bridge_override(previous_txg: u64, version_txg: u64, by_rows: HoleInTheCandidateSet) -> HoleInTheCandidateSet {
    let evidence = proto_env("HOLE_SKIP");
    if !matches!(evidence.as_str(), "union" | "naive" | "counter") {
        return by_rows;
    }
    let bridged = PROTO_HOLE_TABLE.with(|table| {
        let table = table.borrow();
        (previous_txg + 1..version_txg).all(|t| {
            matches!(table.get(&t), Some(ProtoHoleStatus::Skipped | ProtoHoleStatus::Abandoned))
        })
    });
    if bridged {
        HoleInTheCandidateSet::HoleBridgedByTheInstanceTableRows
    } else {
        HoleInTheCandidateSet::HoleNotBridgedByTheInstanceTableRows
    }
}

/// 今天的 gap-row 判法原样一份（[`hole_in_the_candidate_set_between`] 在原型里经 [`proto_bridge_override`] 收尾，洞表要不经它的那一份）。
fn proto_hole_by_rows(
    previous_txg: u64,
    previous_instances: &BTreeSet<u32>,
    version_txg: u64,
    version_instance: u32,
    rows: &[InstanceTableRow],
) -> HoleInTheCandidateSet {
    if previous_txg.checked_add(1) == Some(version_txg) {
        return HoleInTheCandidateSet::NoHole;
    }
    let bridged_from = |previous_instance: u32| {
        if version_instance <= previous_instance {
            return false;
        }
        let row_of_the_previous_version = rows
            .iter()
            .any(|row| row.instance == previous_instance && row.published_checkpoint_txg == previous_txg);
        let rows_between: Vec<&InstanceTableRow> = rows
            .iter()
            .filter(|row| row.instance > previous_instance && row.instance < version_instance)
            .collect();
        let instances_between: BTreeSet<u32> = rows_between.iter().map(|row| row.instance).collect();
        let every_instance_between_has_a_row =
            instances_between.len() as u64 == u64::from(version_instance - previous_instance - 1);
        let every_row_between_published_nothing = rows_between
            .iter()
            .all(|row| row.published_checkpoint_txg == 0 && row.applied_transaction_high_water_mark == 0);
        row_of_the_previous_version && every_instance_between_has_a_row && every_row_between_published_nothing
    };
    if previous_instances.iter().any(|previous_instance| bridged_from(*previous_instance)) {
        HoleInTheCandidateSet::HoleBridgedByTheInstanceTableRows
    } else {
        HoleInTheCandidateSet::HoleNotBridgedByTheInstanceTableRows
    }
}

/// I-3.1 在原型开关下每块盘怎么判。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProtoI31 {
    /// 今天的等式。
    Exact,
    NotApplicable,
    /// 区间 [今天的遍历和, 上沿]（字节）。
    Interval { upper: u64 },
    /// e1ws 的逐槽那一判红：遍历走到的槽里有 `missing` 个在最新根的账里一条记录都没有（已分配、已释放都没有）。
    Red { missing: u64 },
    /// rc（攻方提的变体，零轮）：记账那一行对最新根账里的记录比——[E(θ_high), E(θ_low)]，E(θ) = 已分配记录的槽 + 释放代高于 θ 的已释放记录的槽；
    /// 另判遍历走到的每个槽在 E(θ_high) 那一份里（`uncounted` 是不在的槽数）。
    Records { low: u64, high: u64, uncounted: u64 },
}

/// rc 用：最新根账里按门槛 θ 算「记账该算已分配」的槽集合（已分配的记录，加释放代高于 θ 的已释放记录）。
fn proto_counted_slots(
    reader: &dyn ImageReader,
    newest_root: &crate::RootView,
    threshold: u64,
    cache: &mut IndexNodeCache,
) -> BTreeSet<(u32, u64)> {
    let mut slots: BTreeSet<(u32, u64)> = BTreeSet::new();
    let tree_table_pointer = parse_node_pointer(&newest_root.record_bytes[36..122]);
    if tree_table_pointer.all_zero {
        return slots;
    }
    let Some(tree_table) = read_index_node_without_judging(reader, &tree_table_pointer, cache) else {
        return slots;
    };
    for entry in &tree_table.entries {
        if entry.len() < tree_table_entry_bytes() || read_u16(entry, 10) != TREE_KIND_ALLOCATION {
            continue;
        }
        let allocation_root = parse_node_pointer(&entry[14..100]);
        if allocation_root.all_zero {
            continue;
        }
        let Some((_, records)) = allocation_record_tree_without_judging(reader, &allocation_root, cache) else {
            continue;
        };
        for record in records {
            if !record.is_released || record.generation > threshold {
                slots.extend((record.slot..record.slot + record.span_slots).map(|slot| (record.device, slot)));
            }
        }
    }
    slots
}

/// 最新根账里每一条记录（不论已分配、已释放）罩住的槽（e1ws 的逐槽那一判用）。
fn proto_record_slots(
    reader: &dyn ImageReader,
    newest_root: &crate::RootView,
    cache: &mut IndexNodeCache,
) -> BTreeSet<(u32, u64)> {
    let mut slots: BTreeSet<(u32, u64)> = BTreeSet::new();
    let tree_table_pointer = parse_node_pointer(&newest_root.record_bytes[36..122]);
    if tree_table_pointer.all_zero {
        return slots;
    }
    let Some(tree_table) = read_index_node_without_judging(reader, &tree_table_pointer, cache) else {
        return slots;
    };
    for entry in &tree_table.entries {
        if entry.len() < tree_table_entry_bytes() || read_u16(entry, 10) != TREE_KIND_ALLOCATION {
            continue;
        }
        let allocation_root = parse_node_pointer(&entry[14..100]);
        if allocation_root.all_zero {
            continue;
        }
        let Some((_, records)) = allocation_record_tree_without_judging(reader, &allocation_root, cache) else {
            continue;
        };
        for record in records {
            slots.extend((record.slot..record.slot + record.span_slots).map(|slot| (record.device, slot)));
        }
    }
    slots
}

/// 最新根账里已释放、释放代高于 `threshold`、遍历没走到的槽，逐盘计数（e1w 的上沿用）。
fn proto_released_unwalked_above(
    reader: &dyn ImageReader,
    newest_root: &crate::RootView,
    walked_slots: &BTreeSet<(u32, u64)>,
    threshold: u64,
    cache: &mut IndexNodeCache,
) -> BTreeMap<u32, u64> {
    let mut per_device: BTreeMap<u32, u64> = BTreeMap::new();
    let tree_table_pointer = parse_node_pointer(&newest_root.record_bytes[36..122]);
    if tree_table_pointer.all_zero {
        return per_device;
    }
    let Some(tree_table) = read_index_node_without_judging(reader, &tree_table_pointer, cache) else {
        return per_device;
    };
    let mut slots: BTreeSet<(u32, u64)> = BTreeSet::new();
    for entry in &tree_table.entries {
        if entry.len() < tree_table_entry_bytes() || read_u16(entry, 10) != TREE_KIND_ALLOCATION {
            continue;
        }
        let allocation_root = parse_node_pointer(&entry[14..100]);
        if allocation_root.all_zero {
            continue;
        }
        let Some((_, records)) = allocation_record_tree_without_judging(reader, &allocation_root, cache) else {
            continue;
        };
        for record in records {
            if allocation_record_is_released_above_the_reclaim_threshold(&record, threshold) {
                slots.extend((record.slot..record.slot + record.span_slots).map(|slot| (record.device, slot)));
            }
        }
    }
    for (device, slot) in slots {
        if !walked_slots.contains(&(device, slot)) {
            *per_device.entry(device).or_insert(0) += 1;
        }
    }
    per_device
}

/// 每块盘的判法：`deferred_high_bytes` 是今天并进去的丁-defer 字节，`walked` 是今天的遍历和（含它）。
#[allow(clippy::too_many_arguments)]
fn proto_i31_per_device(
    reader: &dyn ImageReader,
    filesystem_identifier_low: u64,
    newest_root: &crate::RootView,
    roots: &[(u64, u64, crate::RootView)],
    roots_below_the_floor: &[&crate::RootView],
    walked_slots: &BTreeSet<(u32, u64)>,
    effective_rollback_floor: u64,
    deferred_threshold: u64,
    cache: &mut IndexNodeCache,
) -> Box<dyn Fn(u32, u64, u64) -> ProtoI31> {
    let mode = proto_env("HOLE_I31");
    let table = PROTO_HOLE_TABLE.with(|table| table.borrow().clone());
    let unreachable: Vec<u64> =
        table.iter().filter(|(_, s)| **s == ProtoHoleStatus::ExistedUnreachable).map(|(t, _)| *t).collect();
    let hole_positions: Vec<u64> = table
        .iter()
        .filter(|(_, s)| !matches!(s, ProtoHoleStatus::Present | ProtoHoleStatus::Abandoned))
        .map(|(t, _)| *t)
        .collect();
    let _ = roots;
    let trigger: Vec<u64> = if mode == "e1wu" || mode == "na" { unreachable.clone() } else { hole_positions };
    // e1ws（攻方提的变体，零轮）：逐槽那一判不论有没有洞都判——遍历走到的每个槽在最新根的账里都要有一条记录。
    let missing_per_device: BTreeMap<u32, u64> = if mode == "e1ws" {
        let records = proto_record_slots(reader, newest_root, cache);
        let mut missing: BTreeMap<u32, u64> = BTreeMap::new();
        for (device, slot) in walked_slots {
            if !records.contains(&(*device, *slot)) {
                *missing.entry(*device).or_insert(0) += 1;
            }
        }
        proto_note(format!("I31_PROTO_SLOTS walked_slots_without_a_record={missing:?}"));
        missing
    } else {
        BTreeMap::new()
    };
    let theta_low = trigger
        .iter()
        .min()
        .map_or(deferred_threshold, |min_position| deferred_threshold.min(effective_rollback_floor.max(*min_position)));
    proto_note(format!(
        "I31_PROTO mode={mode:?} unreachable={unreachable:?} trigger={trigger:?} theta_high={deferred_threshold} theta_low={theta_low}"
    ));
    if mode == "rc" || mode == "rcw" {
        let counted_high = proto_counted_slots(reader, newest_root, deferred_threshold, cache);
        let counted_low = proto_counted_slots(reader, newest_root, theta_low, cache);
        let per_device = |set: &BTreeSet<(u32, u64)>| {
            let mut counts: BTreeMap<u32, u64> = BTreeMap::new();
            for (device, _) in set {
                *counts.entry(*device).or_insert(0) += 1;
            }
            counts
        };
        let (high_counts, low_counts) = (per_device(&counted_high), per_device(&counted_low));
        let mut uncounted: BTreeMap<u32, u64> = BTreeMap::new();
        for key in walked_slots {
            if !counted_high.contains(key) {
                *uncounted.entry(key.0).or_insert(0) += 1;
            }
        }
        proto_note(format!("I31_PROTO_RC counted_high={high_counts:?} counted_low={low_counts:?} walked_uncounted={uncounted:?}"));
        // rcw（攻方提的变体，零轮）：rc 的下沿与逐槽，上沿再与 e1w 的上沿取小（遍历 + 释放代高于 θ_low、遍历没走到的已释放槽）——
        // 记账多于遍历（泄漏方向）那一半照旧按遍历判，只在洞里那几版的已释放槽上放宽。
        let released_low = if mode == "rcw" {
            proto_released_unwalked_above(reader, newest_root, walked_slots, theta_low, cache)
        } else {
            BTreeMap::new()
        };
        let with_walk = mode == "rcw";
        return Box::new(move |device, walked, deferred_high_bytes| {
            let high_by_records = low_counts.get(&device).copied().unwrap_or(0) * SLOT_BYTES;
            let high = if with_walk {
                let by_walk = walked - deferred_high_bytes + released_low.get(&device).copied().unwrap_or(0) * SLOT_BYTES;
                high_by_records.min(by_walk.max(walked))
            } else {
                high_by_records
            };
            ProtoI31::Records {
                low: high_counts.get(&device).copied().unwrap_or(0) * SLOT_BYTES,
                high,
                uncounted: uncounted.get(&device).copied().unwrap_or(0),
            }
        });
    }
    if mode.is_empty() || (trigger.is_empty() && missing_per_device.is_empty()) {
        return Box::new(|_, _, _| ProtoI31::Exact);
    }
    if mode == "e1ws" && trigger.is_empty() {
        return Box::new(move |device, _, _| match missing_per_device.get(&device) {
            Some(missing) => ProtoI31::Red { missing: *missing },
            None => ProtoI31::Exact,
        });
    }
    if mode == "na" {
        return Box::new(|_, _, _| ProtoI31::NotApplicable);
    }
    let deferred_low = deferred_slots_referenced_only_below_the_floor_per_device(
        reader, filesystem_identifier_low, newest_root, roots_below_the_floor, walked_slots, theta_low, cache,
    );
    let released_low = proto_released_unwalked_above(reader, newest_root, walked_slots, theta_low, cache);
    Box::new(move |device, walked, deferred_high_bytes| {
        if let Some(missing) = missing_per_device.get(&device) {
            return ProtoI31::Red { missing: *missing };
        }
        let base = walked - deferred_high_bytes;
        let upper = if mode == "e1" {
            base + deferred_low.get(&device).copied().unwrap_or(0) * SLOT_BYTES
        } else {
            base + released_low.get(&device).copied().unwrap_or(0) * SLOT_BYTES
        };
        ProtoI31::Interval { upper: upper.max(walked) }
    })
}
