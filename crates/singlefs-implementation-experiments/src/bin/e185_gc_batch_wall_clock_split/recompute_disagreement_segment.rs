use super::host_cpu_per_submission_segment::check_string_holds;
use super::resident_gpu_kernels::{untimed_results, ResidentKernels};
use super::*;
use singlefs_verification::gpu_unit_checks::gpu_cards_by_priority;

/// 一批的内容（计时以外造）：16384 个 32 KiB 槽连放（奇数号单元只用前 16 KiB，其余是 0）、长度表、CPU 参照。
struct BatchContent {
    slots: Vec<u8>,
    lengths: Vec<u32>,
    cpu_crcs: Vec<u32>,
}

fn slot_bytes() -> usize {
    usize::try_from(LOCAL_GPU_SLOT_BYTES).expect("32768")
}

/// 造一批（`threads` 条线程按单元号切片并行，每片各写自己的槽与 CRC；结果与单线程逐字节相同，内容只由种子定）。
fn batch_content(card_seed: u64, form: InputForm, batch: u64, threads: usize) -> BatchContent {
    let units = usize::try_from(UNITS_PER_DISAGREEMENT_BATCH).expect("16384");
    let mut slots = vec![0_u8; units * slot_bytes()];
    let mut cpu_crcs = vec![0_u32; units];
    let units_per_thread = units.div_ceil(threads.max(1));
    std::thread::scope(|scope| {
        for (chunk_index, (slot_chunk, crc_chunk)) in slots
            .chunks_mut(units_per_thread * slot_bytes())
            .zip(cpu_crcs.chunks_mut(units_per_thread))
            .enumerate()
        {
            scope.spawn(move || {
                for (offset, (slot, crc)) in slot_chunk
                    .chunks_mut(slot_bytes())
                    .zip(crc_chunk.iter_mut())
                    .enumerate()
                {
                    let unit =
                        u64::try_from(chunk_index * units_per_thread + offset).expect("单元号");
                    let length = disagreement_unit_length(unit);
                    disagreement_unit_content(card_seed, form, batch, unit, &mut slot[..length]);
                    *crc = crc32_castagnoli(&slot[..length]);
                }
            });
        }
    });
    let lengths = (0..UNITS_PER_DISAGREEMENT_BATCH)
        .map(|unit| u32::try_from(disagreement_unit_length(unit)).expect("长度"))
        .collect();
    BatchContent {
        slots,
        lengths,
        cpu_crcs,
    }
}

/// 一个单元的槽（32 KiB，奇数号单元后半是 0）。
fn slot_of(content: &BatchContent, unit: u64) -> &[u8] {
    let start = usize::try_from(unit).expect("单元号") * slot_bytes();
    &content.slots[start..start + slot_bytes()]
}

/// P5 在分开输入时改核二那一份输入：那个单元的槽复制一份、第 100 字节异或 1，交回（在这一次写入里的偏移、改过的槽）。
fn kernel_two_patch(
    content: &BatchContent,
    form: InputForm,
    batch: u64,
    unit: u64,
    offset_in_the_write: u64,
) -> Vec<(u64, Vec<u8>)> {
    kernel_two_input_flip(form, batch, unit)
        .map(|byte| {
            let mut patched = slot_of(content, unit).to_vec();
            patched[byte] ^= 1;
            (offset_in_the_write, patched)
        })
        .into_iter()
        .collect()
}

/// 一批第一次计算的原样结果：写入（P5 的补丁跟着写）、两核各派一次、阻塞等、读回；序号对不上的那一批全记读不到。
fn first_results_of_the_batch(
    kernels: &ResidentKernels,
    content: &BatchContent,
    form: InputForm,
    batch: u64,
) -> (Vec<KernelResultsOfAUnit>, bool) {
    let patches = kernel_two_patch(
        content,
        form,
        batch,
        DETERMINISTIC_INJECTION_UNIT,
        DETERMINISTIC_INJECTION_UNIT * LOCAL_GPU_SLOT_BYTES,
    );
    let (results, are_the_sequence_numbers_holding) = untimed_results(
        kernels,
        form,
        &content.slots,
        &content.lengths,
        &patches,
        KernelsToDispatch::Both,
    );
    if are_the_sequence_numbers_holding {
        (results, true)
    } else {
        (vec![(None, None); results.len()], false)
    }
}

/// 一个单元单独重算（n = 1，同一种输入；P5 的补丁照写）。
fn recompute_one_unit(
    kernels: &ResidentKernels,
    content: &BatchContent,
    form: InputForm,
    batch: u64,
    unit: u64,
    dispatched: KernelsToDispatch,
) -> KernelResultsOfAUnit {
    let unit_index = usize::try_from(unit).expect("单元号");
    let patches = kernel_two_patch(content, form, batch, unit, 0);
    let (results, are_the_sequence_numbers_holding) = untimed_results(
        kernels,
        form,
        slot_of(content, unit),
        &content.lengths[unit_index..=unit_index],
        &patches,
        dispatched,
    );
    match (are_the_sequence_numbers_holding, results.first()) {
        (true, Some(result)) => *result,
        (false, _) | (_, None) => (None, None),
    }
}

fn comparison_text(comparison: Option<UnitComparison>) -> String {
    comparison.map_or_else(
        || "unreadable".to_string(),
        |seen| {
            format!(
                "{}{}{}",
                u8::from(seen.kernels_differ),
                u8::from(seen.kernel_one_differs_from_the_cpu),
                u8::from(seen.kernel_two_differs_from_the_cpu)
            )
        },
    )
}

/// 一张卡一种输入的合计（`name=disagreement_summary` 那一行）与 P5、P6 的结局。
#[derive(Default)]
struct CombinationTotals {
    natural_units: u64,
    first_round_unreadable: u64,
    first_round_kernel_disagreements: u64,
    first_round_any_difference: u64,
    both_kernels_agree_and_differ_from_the_cpu: u64,
    kernel_one_differs_from_the_cpu: u64,
    kernel_two_differs_from_the_cpu: u64,
    recomputed: Vec<RecomputedUnit>,
    first_round_disagreements_per_batch: Vec<u64>,
    deterministic_injections: Vec<Option<RecomputeOutcome>>,
    transient_injections: Vec<Option<RecomputeOutcome>>,
}

/// 一张卡一种输入：64 批，每批造内容、第一次计算、比三样、重算（[`process_the_disagreement_batch`]），每批一行、每个重算的单元一行；
/// 64 批之后用同样的种子把最先那一批整批再算一遍比原样结果（R8「同一输入同一结果」）。交回合计。
fn measure_the_combination(
    kernels: &ResidentKernels,
    card_index: u32,
    form: InputForm,
    threads: usize,
    emitter: &mut Emitter,
) -> CombinationTotals {
    let card_seed = u64::from(card_index);
    let mut totals = CombinationTotals::default();
    let mut first_batch_results: Vec<KernelResultsOfAUnit> = Vec::new();
    for batch in 1..=DISAGREEMENT_BATCHES {
        let content = batch_content(card_seed, form, batch, threads);
        let started = Instant::now();
        let (first_results, are_the_sequence_numbers_holding) =
            first_results_of_the_batch(kernels, &content, form, batch);
        let outcome = process_the_disagreement_batch(
            form,
            batch,
            &content.cpu_crcs,
            &first_results,
            |unit, dispatched| recompute_one_unit(kernels, &content, form, batch, unit, dispatched),
        );
        let wall_seconds = started.elapsed().as_secs_f64();
        if batch == 1 {
            first_batch_results = first_results;
        }
        emitter.line(&format!(
            "name=disagreement_batch index={card_index} form={} batch={batch} natural_units={} first_round_kernel_disagreements={} first_round_any_difference={} both_kernels_agree_and_differ_from_the_cpu={} kernel_one_differs_from_the_cpu={} kernel_two_differs_from_the_cpu={} first_round_unreadable={} recomputed_natural_units={} sequence_numbers_hold={are_the_sequence_numbers_holding} wall_seconds={wall_seconds:.6}",
            form.label(),
            outcome.natural_units,
            outcome.first_round_kernel_disagreements,
            outcome.first_round_any_difference,
            outcome.both_kernels_agree_and_differ_from_the_cpu,
            outcome.kernel_one_differs_from_the_cpu,
            outcome.kernel_two_differs_from_the_cpu,
            outcome.first_round_unreadable,
            outcome.natural_recomputes.len(),
        ));
        for recomputed in &outcome.natural_recomputes {
            emitter.line(&format!(
                "name=recompute index={card_index} form={} batch={batch} unit={} seed_card={card_seed} seed_form={} first_round_kernels_and_cpu={} recomputes={} outcome={}",
                form.label(),
                recomputed.unit,
                form.seed_bits(),
                comparison_text(Some(recomputed.first_round)),
                recomputed.recomputes.iter().map(|comparison| comparison_text(*comparison)).collect::<Vec<_>>().join(","),
                recomputed.outcome.label(),
            ));
        }
        totals.natural_units += outcome.natural_units;
        totals.first_round_unreadable += outcome.first_round_unreadable;
        totals.first_round_kernel_disagreements += outcome.first_round_kernel_disagreements;
        totals.first_round_any_difference += outcome.first_round_any_difference;
        totals.both_kernels_agree_and_differ_from_the_cpu +=
            outcome.both_kernels_agree_and_differ_from_the_cpu;
        totals.kernel_one_differs_from_the_cpu += outcome.kernel_one_differs_from_the_cpu;
        totals.kernel_two_differs_from_the_cpu += outcome.kernel_two_differs_from_the_cpu;
        totals.recomputed.extend(outcome.natural_recomputes);
        totals
            .first_round_disagreements_per_batch
            .push(outcome.first_round_kernel_disagreements);
        if INJECTED_BATCHES.contains(&batch) {
            totals
                .deterministic_injections
                .push(outcome.deterministic_injection.flatten());
            totals
                .transient_injections
                .push(outcome.transient_injection.flatten());
        }
    }
    let replay_content = batch_content(card_seed, form, 1, threads);
    let (replayed, _sequence) = first_results_of_the_batch(kernels, &replay_content, form, 1);
    emitter.line(&format!(
        "name=determinism_replay index={card_index} form={} batch=1 units={} identical={} all_identical={}",
        form.label(),
        replayed.len(),
        identical_entries(&first_batch_results, &replayed),
        identical_entries(&first_batch_results, &replayed) == u64::try_from(replayed.len()).expect("条数")
            && !replayed.is_empty(),
    ));
    totals
}

/// `name=disagreement_summary` 一行（R6、R7、R8 与 P5、P6、J8）；交回 P5、P6 过没过。
fn emit_the_summary(
    card_index: u32,
    form: InputForm,
    totals: &CombinationTotals,
    emitter: &mut Emitter,
) -> bool {
    let expected = natural_unit_times(
        DISAGREEMENT_BATCHES,
        UNITS_PER_DISAGREEMENT_BATCH,
        injected_units_per_combination(),
    );
    let sufficient = are_the_natural_unit_times_sufficient(totals.natural_units);
    let deterministic_holds = injected_units_hold(
        &totals.deterministic_injections,
        RecomputeOutcome::Deterministic,
    );
    let transient_holds =
        injected_units_hold(&totals.transient_injections, RecomputeOutcome::Transient);
    let outcomes: Vec<RecomputeOutcome> =
        totals.recomputed.iter().map(|unit| unit.outcome).collect();
    let category = match (deterministic_holds && transient_holds, sufficient) {
        (false, _) => "void_positive_control",
        (true, false) => "unit_times_insufficient",
        (true, true) => recompute_category(totals.first_round_any_difference, &outcomes),
    };
    emitter.line(&format!(
        "name=disagreement_summary index={card_index} form={} natural_unit_times={} expected_natural_unit_times={expected} natural_unit_times_sufficient={sufficient} first_round_unreadable={} first_round_kernel_disagreements={} first_round_any_difference={} first_round_rate={:.9e} upper_bound_at_95_percent_when_zero={} both_kernels_agree_and_differ_from_the_cpu={} kernel_one_differs_from_the_cpu={} kernel_two_differs_from_the_cpu={} still_disagreeing_after_one_recompute={} still_disagreeing_after_two_recomputes={} still_disagreeing_after_three_recomputes={} unreadable_recomputes={} deterministic_injection_outcomes={} transient_injection_outcomes={} deterministic_injection_holds={deterministic_holds} transient_injection_holds={transient_holds} {} category={category}",
        form.label(),
        totals.natural_units,
        totals.first_round_unreadable,
        totals.first_round_kernel_disagreements,
        totals.first_round_any_difference,
        totals.first_round_kernel_disagreements as f64 / totals.natural_units.max(1) as f64,
        if totals.first_round_kernel_disagreements == 0 {
            format!("{:.6e}", upper_bound_at_95_percent_when_zero(totals.natural_units))
        } else {
            "none".to_string()
        },
        totals.both_kernels_agree_and_differ_from_the_cpu,
        totals.kernel_one_differs_from_the_cpu,
        totals.kernel_two_differs_from_the_cpu,
        still_disagreeing_after(&totals.recomputed, 1),
        still_disagreeing_after(&totals.recomputed, 2),
        still_disagreeing_after(&totals.recomputed, 3),
        outcomes.iter().filter(|outcome| **outcome == RecomputeOutcome::Unreadable).count(),
        totals.deterministic_injections.iter().map(|outcome| outcome.map_or("no_first_round_difference", RecomputeOutcome::label)).collect::<Vec<_>>().join(","),
        totals.transient_injections.iter().map(|outcome| outcome.map_or("no_first_round_difference", RecomputeOutcome::label)).collect::<Vec<_>>().join(","),
        disagreement_trajectory_text(&totals.first_round_disagreements_per_batch),
    ));
    deterministic_holds && transient_holds
}

/// 第二段（登记第五节「第二段怎么量」）：段开头核装置 sha256、常量回比（停机 J-S2）、内容锚点（停机 J-S8）；每张卡起卡、P2、两种输入各
/// 64 批，再算「两张卡同一输入」那一批；两张卡都算完逐条比（第十条 J11）。这一段不计时，只报每批挂钟。
/// 取卡在开设备前一刻（`cards_and_pairing_of_the_gpu_segments::card_indices_handed_over_to_the_segment`），「两张卡同一输入」那一批在取卡之前造好；
/// 每张卡每种输入的 64 批（一批 16384 个 32 KiB 槽）在持卡时逐批现造、现算 CPU 参照。
pub(super) fn measure_recompute_disagreement_segment(card_indices: &[u32]) -> i32 {
    let threads = worker_threads_from_the_environment();
    let mut emitter = Emitter::new();
    emitter.line(&format!(
        "name=configuration segment=recompute_disagreement cards={} batches={DISAGREEMENT_BATCHES} units_per_batch={UNITS_PER_DISAGREEMENT_BATCH} recomputes_at_most={RECOMPUTES_AT_MOST} injected_batches={}..={} deterministic_injection_unit={DETERMINISTIC_INJECTION_UNIT} transient_injection_unit={TRANSIENT_INJECTION_UNIT} worker_threads={threads} card_quota_mebibytes={CARD_QUOTA_MEBIBYTES}",
        card_indices.iter().map(u32::to_string).collect::<Vec<_>>().join(","),
        INJECTED_BATCHES.start(),
        INJECTED_BATCHES.end(),
    ));
    if !check_the_apparatus_sha256(&mut emitter) {
        return EXIT_CODE_STOP_CLAUSE;
    }
    // P8：校准分得出两倍的活（第一、二段都做；这一段不计时，校准只报这一行与基线）
    if let Err(exit_code) =
        SegmentCalibration::start("recompute_disagreement", Instant::now(), &mut emitter)
    {
        return exit_code;
    }
    let mismatches = check_the_constants_and_the_anchors(&mut emitter);
    let content_finding = content_anchor_finding();
    emitter.line(&format!(
        "name=content_anchor_check anchors={} holds={}",
        CONTENT_ANCHORS.len(),
        content_finding.is_none()
    ));
    if mismatches > 0 || content_finding.is_some() {
        if let Some(finding) = &content_finding {
            emitter.line(&format!(
                "name=stop_clause segment=recompute_disagreement finding={finding}"
            ));
        }
        emitter.line(&format!(
            "name=verdict segment=recompute_disagreement constant_or_anchor_mismatches={mismatches} stop_clause_findings={}",
            usize::from(content_finding.is_some())
        ));
        emitter.finish();
        return EXIT_CODE_STOP_CLAUSE;
    }
    let both_cards_content = batch_content(
        BOTH_CARDS_SEED_CARD,
        InputForm::Shared,
        BOTH_CARDS_SEED_BATCH,
        threads,
    );
    let mut both_cards_results: Vec<(u32, Vec<KernelResultsOfAUnit>)> = Vec::new();
    let mut cards_measured = 0_usize;
    let mut control_failures: Vec<String> = Vec::new();
    let picked_card_indices =
        match cards_and_pairing_of_the_gpu_segments::card_indices_handed_over_to_the_segment(
            card_indices,
        ) {
            Ok(picked) => picked,
            Err(refusal) => {
                return cards_and_pairing_of_the_gpu_segments::stop_without_the_cards_handed_over(
                    "recompute_disagreement",
                    &refusal,
                    &mut emitter,
                )
            }
        };
    for card_index in &picked_card_indices {
        let Some(card) = gpu_cards_by_priority(&[(*card_index, CARD_QUOTA_MEBIBYTES)])
            .into_iter()
            .next()
        else {
            emitter.line(&format!(
                "name=gpu_card index={card_index} status=not_run reason=card_not_usable"
            ));
            continue;
        };
        let kernels = ResidentKernels::start(&card, UNITS_PER_DISAGREEMENT_BATCH);
        emitter.line(&format!(
            "name=gpu_card index={card_index} card_name={} driver={} free_memory_mebibytes={} resident_capacity_units={}",
            field_token(&card.name),
            field_token(&card.driver_version),
            card.free_memory_mebibytes,
            kernels.capacity_units,
        ));
        cards_measured += 1;
        if !check_string_holds(&kernels, &mut emitter, *card_index) {
            control_failures.push(format!("card_{card_index}_check_string"));
        }
        let mut card_totals = Vec::new();
        for form in InputForm::ALL {
            let totals =
                measure_the_combination(&kernels, *card_index, form, threads, &mut emitter);
            if !emit_the_summary(*card_index, form, &totals, &mut emitter) {
                control_failures.push(format!(
                    "card_{card_index}_form_{}_injections",
                    form.label()
                ));
            }
            card_totals.push(totals);
        }
        let natural: u64 = card_totals.iter().map(|totals| totals.natural_units).sum();
        let disagreements: u64 = card_totals
            .iter()
            .map(|totals| totals.first_round_kernel_disagreements)
            .sum();
        emitter.line(&format!(
            "name=disagreement_card_summary index={card_index} natural_unit_times={natural} first_round_kernel_disagreements={disagreements} upper_bound_at_95_percent_when_zero={}",
            if disagreements == 0 {
                format!("{:.6e}", upper_bound_at_95_percent_when_zero(natural))
            } else {
                "none".to_string()
            },
        ));
        let (results, are_the_sequence_numbers_holding) = untimed_results(
            &kernels,
            InputForm::Shared,
            &both_cards_content.slots,
            &both_cards_content.lengths,
            &[],
            KernelsToDispatch::Both,
        );
        let differing_from_the_cpu = results
            .iter()
            .zip(&both_cards_content.cpu_crcs)
            .filter(|((kernel_one, kernel_two), cpu)| {
                *kernel_one != Some(**cpu) || *kernel_two != Some(**cpu)
            })
            .count();
        emitter.line(&format!(
            "name=both_cards_same_input_card index={card_index} units={} sequence_numbers_hold={are_the_sequence_numbers_holding} differing_from_the_cpu={differing_from_the_cpu}",
            results.len()
        ));
        both_cards_results.push((*card_index, results));
    }
    let mut cards_differ = false;
    if let [(one_card, one_results), (other_card, other_results), ..] =
        both_cards_results.as_slice()
    {
        let identical = identical_entries(one_results, other_results);
        let units = u64::try_from(one_results.len()).expect("条数");
        cards_differ = identical != units;
        emitter.line(&format!(
            "name=both_cards_same_input cards={one_card},{other_card} units={units} identical={identical} cards_differ={cards_differ}"
        ));
    }
    emitter.line(&format!(
        "name=verdict segment=recompute_disagreement constant_or_anchor_mismatches=0 stop_clause_findings=0 cards_measured={cards_measured} positive_control_failures={} positive_control_failure_items={} two_cards_differ_on_the_same_input={cards_differ}",
        control_failures.len(),
        if control_failures.is_empty() { "none".to_string() } else { control_failures.join(",") },
    ));
    emitter.finish();
    if cards_measured == 0 {
        EXIT_CODE_NOT_RUN
    } else if !control_failures.is_empty() {
        EXIT_CODE_POSITIVE_CONTROL_FAILED
    } else {
        EXIT_CODE_OK
    }
}
