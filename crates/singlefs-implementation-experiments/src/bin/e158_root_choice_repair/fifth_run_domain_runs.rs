//! 第 5 次跑按域、按分片的整趟，常量回比与模式入口。

use super::*;

/// 跑一个域：列单位、并行处理、按单位序合并、汇总、阳性对照、判决行与摘要。`cut_depth` 给了就用它（单测用），没给按线程数选。
fn run_fifth_run_domain(
    domain: &FifthRunDomain,
    arm: &str,
    threads: usize,
    cut_depth: Option<u64>,
    judgements: &ReusableJudgements,
) -> Result<FifthRunReport, String> {
    let (cut_depth, units) = units_of_the_run(domain, threads, cut_depth)?;
    let process = |unit: &EnumerationUnit| process_unit(unit, domain, arm, judgements);
    let outputs = run_units_in_parallel(&units, threads, &process)?;
    let pairs: Vec<PairRecord> = outputs
        .iter()
        .flat_map(|output| output.pairs.iter().cloned())
        .collect();
    let nodes: u64 = outputs.iter().map(|output| output.nodes_visited).sum();
    let transition_failures: u64 = outputs
        .iter()
        .map(|output| output.transition_failures)
        .sum();
    let bound = node_count_upper_bound(domain);
    if nodes > bound {
        return Err(format!(
            "stop reason=S4 detail=node_count_above_bound nodes={nodes} bound={bound}"
        ));
    }
    let cells = root_cells(&pairs);
    let construction = tally_construction(&pairs);
    let account = tally_account_quantities(&cells, &domain.geometry);
    let root_slot = tally_root_slot_quantities(&cells);
    let tallies = [&construction, &account, &root_slot];
    let mut lines = pair_rows_in_key_order(arm, &domain.geometry, &pairs);
    lines.extend(quantity_lines(
        arm,
        domain,
        nodes,
        transition_failures,
        tallies,
    ));
    let controls = [
        positive_control_on_the_account(
            arm,
            domain,
            &cells,
            "PC1-a",
            LedgerAspect::AllocationRecordTree,
        ),
        positive_control_on_the_account(arm, domain, &cells, "PC1-b", LedgerAspect::TreeTable),
        positive_control_on_the_root_slot(arm, domain, &cells),
        positive_control_on_the_twin(arm, domain, &cells),
        positive_control_on_the_reuse_detection(arm, domain, &cells),
        positive_control_on_the_empty_domain(
            arm,
            domain,
            construction.count_of("constructed_nodes"),
        ),
    ];
    let controls_not_pass = u64::try_from(
        controls
            .iter()
            .filter(|(_, verdict)| *verdict != "pass")
            .count(),
    )
    .expect("个数装得进 u64");
    lines.extend(controls.into_iter().map(|(line, _)| line));
    lines.push(verdict_line(arm, domain, tallies, controls_not_pass));
    let digest = lines_digest(&lines);
    lines.push(format!(
        "name=r5_digest arm={arm} slots_per_region={} ring={} sha256={digest}",
        domain.geometry.slots_per_region, domain.geometry.journal_ring_bytes
    ));
    Ok(FifthRunReport {
        lines,
        cut_depth,
        units: units.len(),
        fresh: pairs.iter().map(|pair| pair.fresh).sum(),
        reused: pairs.iter().map(|pair| pair.reused).sum(),
    })
}

/// 一片枚举交回的东西：产物行（片参数行以外）、单位数、切深、现判与复用的个数。
struct FifthRunShardReport {
    lines: Vec<String>,
    cut_depth: u64,
    units_total: usize,
    units_in_shard: usize,
    fresh: u64,
    reused: u64,
}

impl FifthRunShardReport {
    /// 片产物的参数行：这一片是整张单位列表的哪一片，切深与单位总数供各片之间对账。
    fn parameters_line(
        &self,
        mode: &str,
        arm: &str,
        domain: &FifthRunDomain,
        threads: usize,
        shard: UnitShard,
        description: &str,
    ) -> String {
        format!(
            "name=r5_shard_parameters arm={arm} mode={mode} slots_per_region={} ring={} step_limit={} starting_overwrites={:?} threads={threads} cut_depth={} units_total={} shard_index={} shard_count={} units_in_shard={} {description} judgements_fresh={} judgements_reused={}",
            domain.geometry.slots_per_region,
            domain.geometry.journal_ring_bytes,
            domain.step_limit,
            domain.starting_overwrites,
            self.cut_depth,
            self.units_total,
            shard.index,
            shard.count,
            self.units_in_shard,
            self.fresh,
            self.reused
        )
    }
}

/// 整趟与合并的参数行（不进两趟之间的比较）：末尾 `assembly=` 说这份产物是整趟一次跑出的还是片都跑完之后从库里合并的。
fn run_parameters_line(
    mode: &str,
    arm: &str,
    domain: &FifthRunDomain,
    threads: usize,
    assembly: FifthRunAssembly,
    report: &FifthRunReport,
    description: &str,
) -> String {
    format!(
        "name=r5_run_parameters arm={arm} mode={mode} slots_per_region={} ring={} step_limit={} starting_overwrites={:?} threads={threads} cut_depth={} units={} {description} judgements_fresh={} judgements_reused={} assembly={}",
        domain.geometry.slots_per_region,
        domain.geometry.journal_ring_bytes,
        domain.step_limit,
        domain.starting_overwrites,
        report.cut_depth,
        report.units,
        report.fresh,
        report.reused,
        assembly.name()
    )
}

/// 只跑单位列表里归这一片的单位：判定照常写进库，不汇总、不做阳性对照、不打判决行；
/// 交回一行摘要（节点数、对数、失败分支数、这一片全部对的产物行的 sha256），供读的人与合并那一趟的结果对账。
fn run_fifth_run_shard(
    domain: &FifthRunDomain,
    arm: &str,
    threads: usize,
    cut_depth: Option<u64>,
    shard: UnitShard,
    judgements: &ReusableJudgements,
) -> Result<FifthRunShardReport, String> {
    let (cut_depth, units) = units_of_the_run(domain, threads, cut_depth)?;
    let units_total = units.len();
    let units_of_the_shard: Vec<EnumerationUnit> = units
        .into_iter()
        .enumerate()
        .filter(|(unit_index, _)| shard.holds(*unit_index))
        .map(|(_, unit)| unit)
        .collect();
    let process = |unit: &EnumerationUnit| process_unit(unit, domain, arm, judgements);
    let outputs = run_units_in_parallel(&units_of_the_shard, threads, &process)?;
    let pairs: Vec<PairRecord> = outputs
        .iter()
        .flat_map(|output| output.pairs.iter().cloned())
        .collect();
    let nodes: u64 = outputs.iter().map(|output| output.nodes_visited).sum();
    let transition_failures: u64 = outputs
        .iter()
        .map(|output| output.transition_failures)
        .sum();
    let rows_digest = lines_digest(&pair_rows_in_key_order(arm, &domain.geometry, &pairs));
    let summary = format!(
        "name=r5_shard_summary arm={arm} slots_per_region={} ring={} shard_index={} shard_count={} units_in_shard={} nodes={nodes} pairs={} transition_failures={transition_failures} pair_rows_sha256={rows_digest}",
        domain.geometry.slots_per_region,
        domain.geometry.journal_ring_bytes,
        shard.index,
        shard.count,
        units_of_the_shard.len(),
        pairs.len()
    );
    Ok(FifthRunShardReport {
        lines: vec![summary],
        cut_depth,
        units_total,
        units_in_shard: units_of_the_shard.len(),
        fresh: pairs.iter().map(|pair| pair.fresh).sum(),
        reused: pairs.iter().map(|pair| pair.reused).sum(),
    })
}

/// 开跑检查（停机 S1）：本地常量与 `crates/` 回比；续写终点的槽序回比（锚点 A1：同一根槽的下一次发布在 txg + 3S，更早的都不碰）。
fn run_fifth_run_constants_and_anchors(arm: &str) -> bool {
    let mut all_passed = true;
    let mut checks = fourth_run_local_constants_checks();
    for (label, geometry) in [
        ("G0", &GEOMETRY_PRIMARY),
        ("S16", &GEOMETRY_LARGER_ROOT_RING),
        ("S4", &GEOMETRY_SMALLER_ROOT_RING),
    ] {
        let sizes = parameters_for(geometry).geometry;
        let (expected_slots, expected_ring_bytes) = match label {
            "G0" => (8, 3 * 1024 * 1024),
            "S16" => (16, 3 * 1024 * 1024),
            _ => (4, 3 * 1024 * 1024),
        };
        checks.push(ConstantCheck {
            name: "各档每区槽数 S（本地档表对 parameters_for 交回的系统配置）",
            local_value: expected_slots,
            crates_value: sizes.root_ring_slots_per_region.count(),
        });
        checks.push(ConstantCheck {
            name: "各档 journal 环字节数（本地档表对 parameters_for 交回的系统配置）",
            local_value: expected_ring_bytes,
            crates_value: sizes.journal_ring_bytes,
        });
    }
    for check in checks {
        let passed = check.matches();
        all_passed &= passed;
        emit_result(&format!(
            "name=r5_local_constant_check arm={arm} item={:?} verdict={} local_value={} crates_value={}",
            check.name,
            verdict_word(passed),
            check.local_value,
            check.crates_value
        ));
    }
    for slots_per_region in [4u64, 8, 16] {
        let (passed, first_collision_offsets) = continuation_anchor_of(slots_per_region);
        all_passed &= passed;
        emit_result(&format!(
            "name=r5_continuation_anchor arm={arm} slots_per_region={slots_per_region} verdict={} first_collision_offsets={first_collision_offsets:?} expected={}",
            verdict_word(passed),
            ROOT_RING_REGIONS_LOCAL * slots_per_region
        ));
    }
    all_passed
}

/// 锚点 A1：对每个起点 txg，同一根槽的下一次发布恰在 txg + 3S，更早的 txg + 1 … txg + 3S − 1 都不在这一槽（按 `crates/` 的 `target_for_publish`）。
fn continuation_anchor_of(slots_per_region: u64) -> (bool, Vec<u64>) {
    let ring = RootRingSlotsPerRegion::from_system_configuration_field(slots_per_region)
        .expect("S 在 [4,16] 区间内");
    let mut offsets = BTreeSet::new();
    for start in 0..(6 * slots_per_region) {
        let slot = singlefs_core::root_ring::target_for_publish(CheckpointTxg(start), ring);
        let collision = (1..=6 * slots_per_region).find(|offset| {
            singlefs_core::root_ring::target_for_publish(CheckpointTxg(start + offset), ring)
                == slot
        });
        match collision {
            Some(offset) => {
                offsets.insert(offset);
            }
            None => {
                offsets.insert(0);
            }
        }
    }
    let expected = BTreeSet::from([ROOT_RING_REGIONS_LOCAL * slots_per_region]);
    (offsets == expected, offsets.into_iter().collect())
}

/// 模式对应的域。小环档先搜小环字节数（与 `q3-1-small-ring` 同一个搜法）并打搜索结果行。
fn fifth_run_domain_of_mode(mode: &str) -> Option<FifthRunDomain> {
    let starting_overwrites = FIFTH_RUN_STARTING_OVERWRITES.to_vec();
    match mode {
        "r5-enum-g0" | "r5-enum-g0-one-thread" => Some(FifthRunDomain {
            geometry: GEOMETRY_PRIMARY,
            step_limit: 3,
            starting_overwrites,
        }),
        "r5-enum-s16" => Some(FifthRunDomain {
            geometry: GEOMETRY_LARGER_ROOT_RING,
            step_limit: 3,
            starting_overwrites,
        }),
        "r5-enum-small-ring" => {
            let small_ring_bytes = find_small_ring();
            emit_result(&format!(
                "name=small_ring_search found_ring_bytes={small_ring_bytes} search_started_at_bytes=49152 step_bytes=16384"
            ));
            Some(FifthRunDomain {
                geometry: Geometry {
                    label: "小环",
                    slots_per_region: 8,
                    journal_ring_bytes: small_ring_bytes,
                },
                step_limit: 3,
                starting_overwrites,
            })
        }
        "r5-enum-s4" => Some(FifthRunDomain {
            geometry: GEOMETRY_SMALLER_ROOT_RING,
            step_limit: 4,
            starting_overwrites,
        }),
        _ => None,
    }
}

/// 第 5 次跑的模式：开跑检查不过就停（S1）；过了跑点名的那一档。交回 true = 这个模式归第 5 次跑。
pub(super) fn run_fifth_run_mode(mode: &str, arguments: &[String]) -> bool {
    let known = [
        "r5-constants",
        "r5-enum-g0",
        "r5-enum-g0-one-thread",
        "r5-enum-s16",
        "r5-enum-small-ring",
        "r5-enum-s4",
    ];
    if !known.contains(&mode) {
        return false;
    }
    emit_input_header();
    let arm = second_run_arm_label();
    let finish = |exit_code: i32| {
        emit_result(&format!(
            "name=done emitted={}",
            emitted_result_line_count() + 1
        ));
        if exit_code != 0 {
            std::process::exit(exit_code);
        }
    };
    if arm != "today" && arm != "skip" {
        emit_result(&format!(
            "name=stop arm={arm} reason=arm_unknown detail=\"第 5 次跑只认 today 与 skip 两条臂\""
        ));
        finish(1);
        return true;
    }
    let assembly = match fifth_run_assembly_of(mode, arguments) {
        Ok(assembly) => assembly,
        Err(error) => {
            emit_result(&format!(
                "name=stop arm={arm} mode={mode} reason=assembly_arguments detail=\"{error}\""
            ));
            finish(2);
            return true;
        }
    };
    if !run_fifth_run_constants_and_anchors(&arm) {
        emit_result(&format!(
            "name=stop arm={arm} reason=S1 detail=\"第 5 次跑的常量回比或槽序回比有不过的，整轮不开跑\""
        ));
        finish(1);
        return true;
    }
    if mode == "r5-constants" {
        finish(0);
        return true;
    }
    let domain = fifth_run_domain_of_mode(mode).expect("known 里的 r5 枚举模式都有域");
    let threads = if mode == "r5-enum-g0-one-thread" {
        1
    } else {
        singlefs_verification::heavy_threads::default_heavy_threads().get()
    };
    // 单线程对拍趟（Q8）要自己现判每一个单位，读库复用会让它与正式趟比的是同一份判定：不连库、全现判。别的模式连库，连不上不退回本地。
    let judging = if mode == "r5-enum-g0-one-thread" {
        FifthRunJudging {
            library: Box::new(DisconnectedJudgementLibrary),
            code_keys: JudgementCodeKeys::unconnected(),
            description: "judgement_library=none_fresh_for_parity_run".to_string(),
        }
    } else {
        match connect_judgements(arguments, DeviceLibrary::from_environment()) {
            Ok(connected) => connected,
            Err(error) => {
                emit_result(&format!(
                    "name=stop arm={arm} mode={mode} reason=judgement_library_not_connected detail=\"{error}\""
                ));
                finish(2);
                return true;
            }
        }
    };
    finish(run_fifth_run_enumeration(
        mode, &arm, &domain, threads, assembly, &judging,
    ));
    true
}

/// 连上判定库之后的一趟：整趟、一片或合并（`assembly`），打产物行；交回退出码（0 成，1 停）。
/// 判定库读写花的线程时间打在标准错误一行，不进产物。
fn run_fifth_run_enumeration(
    mode: &str,
    arm: &str,
    domain: &FifthRunDomain,
    threads: usize,
    assembly: FifthRunAssembly,
    judging: &FifthRunJudging,
) -> i32 {
    let started_at = std::time::Instant::now();
    let reuse_only = ReuseOnlyJudgementLibrary::over(
        judging.library.as_ref(),
        REUSE_ONLY_FRESH_JUDGEMENTS_ALLOWED,
    );
    let library: &dyn JudgementLibrary = match assembly {
        FifthRunAssembly::ReuseOnlyFromTheLibrary => &reuse_only,
        FifthRunAssembly::Whole | FifthRunAssembly::Shard(_) => judging.library.as_ref(),
    };
    let judgements = ReusableJudgements {
        library,
        code_keys: judging.code_keys.clone(),
    };
    let exit_code = match assembly {
        FifthRunAssembly::Shard(shard) => {
            match run_fifth_run_shard(domain, arm, threads, None, shard, &judgements) {
                Ok(report) => {
                    emit_result(&report.parameters_line(
                        mode,
                        arm,
                        domain,
                        threads,
                        shard,
                        &judging.description,
                    ));
                    for line in &report.lines {
                        emit_result(line);
                    }
                    0
                }
                Err(error) => {
                    emit_result(&format!(
                        "name=stop arm={arm} mode={mode} detail=\"{error}\""
                    ));
                    1
                }
            }
        }
        FifthRunAssembly::Whole | FifthRunAssembly::ReuseOnlyFromTheLibrary => {
            match run_fifth_run_domain(domain, arm, threads, None, &judgements) {
                Ok(report) => {
                    emit_result(&run_parameters_line(
                        mode,
                        arm,
                        domain,
                        threads,
                        assembly,
                        &report,
                        &judging.description,
                    ));
                    for line in &report.lines {
                        emit_result(line);
                    }
                    0
                }
                Err(error) => {
                    emit_result(&format!(
                        "name=stop arm={arm} mode={mode} detail=\"{error}\""
                    ));
                    1
                }
            }
        }
    };
    if let Some(summary) = judging.library.timing_summary() {
        eprintln!(
            "r5 library_timing assembly={} wall_milliseconds={} threads={threads} {summary}",
            assembly.name(),
            started_at.elapsed().as_millis()
        );
    }
    exit_code
}

#[cfg(test)]
mod fifth_run_tests;
