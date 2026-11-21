//! 判器内核源码集的编译线程数、按片判的单元，与往内核入口里种违例位、判不了原因之后经两个 checker 内核到判定的用例。

use super::*;

/// 按片判的单位跑得超过它的，每过它一次每片至少打一行进度（`.claude/rules/test-script-discipline.md`「编译与算法都要快」；
/// 与崩溃注入读故障用例同一个间隔：5 分钟）。
pub(super) const PROGRESS_INTERVAL_OF_THE_SLICES: Duration = Duration::from_secs(5 * 60);

/// 几套内核源码各建判器、判同一条流时起几个工作线程：编内核一套一套编的变量（[`GPU_KERNELS_ONE_AT_A_TIME_ENVIRONMENT_VARIABLE`]，
/// 与判器自己读的是同一个；值 `one_at_a_time_value`，`None` 是没设）写了 `1`
/// 只起 1 个（几套一起冷编，主机内存跟着翻倍）；否则取 CPU 判器线程数（[`cpu_threads_from_the_value_of_the_variable`]）与套数里小的那个，
/// 至少 1。认法同 `compile_the_judge_kernels`：只有恰好 `1` 算写了，别的值照没写。
fn worker_threads_for_the_kernel_source_sets(
    cpu_threads: std::num::NonZeroUsize,
    one_at_a_time_value: Option<&str>,
    kernel_source_sets: usize,
) -> usize {
    match one_at_a_time_value {
        Some("1") => 1,
        Some(_) | None => cpu_threads.get().min(kernel_source_sets).max(1),
    }
}

/// 起几个工作线程：CPU 判器线程数照它自己的变量取（没设取每台默认重活线程数）、取到 3 条，两套源码起 2 个；
/// 编内核一套一套编的变量写了 `1` 时只起 1 个；CPU 判器线程数取到 1 条时也只起 1 个；那个变量写了别的值照没写。不用卡。
/// 判别力：`crates/mutations.tsv` 里「GPU 对拍 种违例位那条用例不认一套一套编的变量」一行让这一条红。
#[test]
fn kernel_source_sets_take_as_many_worker_threads_as_the_cpu_judge_threads_allow_and_one_when_compiled_one_at_a_time(
) {
    let three = std::num::NonZeroUsize::new(3).expect("3 不是 0");
    let one = std::num::NonZeroUsize::new(1).expect("1 不是 0");
    assert_eq!(
        worker_threads_for_the_kernel_source_sets(three, None, 2),
        2,
        "CPU 判器线程数 3、两套源码：起 2 个"
    );
    assert_eq!(
        worker_threads_for_the_kernel_source_sets(three, Some("1"), 2),
        1,
        "编内核一套一套编的变量写了 1：只起 1 个"
    );
    assert_eq!(
        worker_threads_for_the_kernel_source_sets(one, None, 2),
        1,
        "CPU 判器线程数 1：只起 1 个"
    );
    assert_eq!(
        worker_threads_for_the_kernel_source_sets(three, Some("0"), 2),
        2,
        "那个变量写了 1 之外的值照没写"
    );
}

/// 把 `units` 切成首尾相接的片（`crash_injection::seed_slices`：片数取 min(单位数, 4 × 线程数)，各片长度相差至多 1），起
/// min(`worker_threads`, 片数) 个工作线程（`worker_threads` 是 0 时照 1 算）按片号从小到大领片、每片各自判自己那几个单位
/// （同崩溃注入读故障用例的切法）。调用线程收到一片打一行（`line_prefix`_SLICE_FINISHED），每过 [`PROGRESS_INTERVAL_OF_THE_SLICES`]
/// 给每片各打一行（`line_prefix`_PROGRESS：等着 / 在判 / 判完），全部收回之后按片号放回单位的次序：交回与 `units` 逐个对应，
/// 与线程数、调度次序无关。
///
/// # Panics
/// 某个工作线程 panic：原样往外抛（卡上的断言、判器 panic 的原话不换成「工作线程 panic」）；有一片没交回。
fn outcomes_of_units_judged_in_slices_on_worker_threads<Unit: Sync, Outcome: Send>(
    line_prefix: &str,
    units: &[Unit],
    worker_threads: usize,
    judge_a_unit: &(dyn Fn(&Unit) -> Outcome + Sync),
) -> Vec<Outcome> {
    let unit_count = u64::try_from(units.len()).expect("单位数装得进 u64");
    let slices: Vec<std::ops::Range<usize>> = seed_slices(unit_count, worker_threads.max(1))
        .into_iter()
        .map(|slice| {
            usize::try_from(slice.start).expect("片首不超过单位数，装得进 usize")
                ..usize::try_from(slice.end).expect("片尾不超过单位数，装得进 usize")
        })
        .collect();
    let spawned_worker_threads = worker_threads.max(1).min(slices.len());
    let slice_is_claimed: Vec<AtomicBool> = slices.iter().map(|_| AtomicBool::new(false)).collect();
    let next_slice_index = AtomicUsize::new(0);
    let started = Instant::now();
    let finished_slices: BTreeMap<usize, Vec<Outcome>> = std::thread::scope(|scope| {
        let (sender, receiver) = mpsc::channel::<(usize, Vec<Outcome>)>();
        let worker_handles: Vec<std::thread::ScopedJoinHandle<'_, ()>> = (0
            ..spawned_worker_threads)
            .map(|_| {
                let sender = sender.clone();
                let slices = &slices;
                let slice_is_claimed = &slice_is_claimed;
                let next_slice_index = &next_slice_index;
                scope.spawn(move || loop {
                    let slice_index = next_slice_index.fetch_add(1, Ordering::Relaxed);
                    let Some(slice) = slices.get(slice_index) else {
                        break;
                    };
                    slice_is_claimed[slice_index].store(true, Ordering::Relaxed);
                    let outcomes: Vec<Outcome> =
                        units[slice.clone()].iter().map(judge_a_unit).collect();
                    if sender.send((slice_index, outcomes)).is_err() {
                        break;
                    }
                })
            })
            .collect();
        drop(sender);
        let mut finished_slices: BTreeMap<usize, Vec<Outcome>> = BTreeMap::new();
        let mut next_progress_report = started + PROGRESS_INTERVAL_OF_THE_SLICES;
        // 迭代上界：每片收一次，加每个进度间隔醒一次；工作线程都退了（发送端全丢了，panic 的也算）就停
        loop {
            match receiver
                .recv_timeout(next_progress_report.saturating_duration_since(Instant::now()))
            {
                Ok((slice_index, outcomes)) => {
                    let slice = &slices[slice_index];
                    println!(
                        "{line_prefix}_SLICE_FINISHED slice={}/{} units=[{},{}) finished_slices={}/{} elapsed_seconds={:.1}",
                        slice_index + 1,
                        slices.len(),
                        slice.start,
                        slice.end,
                        finished_slices.len() + 1,
                        slices.len(),
                        started.elapsed().as_secs_f64()
                    );
                    finished_slices.insert(slice_index, outcomes);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
            if Instant::now() >= next_progress_report {
                for (slice_index, slice) in slices.iter().enumerate() {
                    let state = if finished_slices.contains_key(&slice_index) {
                        "finished"
                    } else if slice_is_claimed[slice_index].load(Ordering::Relaxed) {
                        "running"
                    } else {
                        "waiting"
                    };
                    println!(
                        "{line_prefix}_PROGRESS slice={}/{} units=[{},{}) state={state} elapsed_seconds={:.1}",
                        slice_index + 1,
                        slices.len(),
                        slice.start,
                        slice.end,
                        started.elapsed().as_secs_f64()
                    );
                }
                next_progress_report = Instant::now() + PROGRESS_INTERVAL_OF_THE_SLICES;
            }
        }
        for worker_handle in worker_handles {
            if let Err(panic) = worker_handle.join() {
                std::panic::resume_unwind(panic);
            }
        }
        finished_slices
    });
    assert_eq!(
        finished_slices.len(),
        slices.len(),
        "每一片都交回了：少了说明有工作线程领了片却没交回"
    );
    println!(
        "{line_prefix}_FINISHED units={} slices={} worker_threads={spawned_worker_threads} configured_worker_threads={worker_threads} elapsed_seconds={:.1}",
        units.len(),
        slices.len(),
        started.elapsed().as_secs_f64()
    );
    let mut outcomes_in_unit_order: Vec<Outcome> = Vec::with_capacity(units.len());
    for (slice_index, outcomes_of_the_slice) in finished_slices {
        assert_eq!(
            outcomes_of_the_slice.len(),
            slices[slice_index].len(),
            "片 {slice_index} 交回的结论与它的单位逐个对应"
        );
        outcomes_in_unit_order.extend(outcomes_of_the_slice);
    }
    outcomes_in_unit_order
}

/// 多线程与单线程的对拍（同切法、小输入，不用卡）：7 个单位，单线程那一趟 1 个工作线程切 4 片、多线程那一趟 3 个工作线程每个单位一片，
/// 两趟的切法不同；每个单位的结论由单位自己定（单位号与它的平方），两趟交回逐个相同、都按单位次序。
/// 两趟的工作线程数写死成 1 与 3（比的就是单线程与多线程两种切法，不取登记的线程数）；两个行首前缀只是进度行的标签，不是环境变量。
/// 判别力：`crates/mutations.tsv` 里「GPU 对拍 按片判合并时漏掉片号最大的那一片」「GPU 对拍 按片判每片只判头一个单位」两行各让这一条红。
#[test]
fn units_judged_in_slices_on_several_worker_threads_and_on_one_come_back_identical_and_in_unit_order(
) {
    const UNIT_COUNT: u64 = 7;
    let units: Vec<u64> = (0..UNIT_COUNT).collect();
    let judge_a_unit = |unit: &u64| (*unit, unit * unit);
    let on_one_worker_thread = outcomes_of_units_judged_in_slices_on_worker_threads(
        "CRASH_JUDGE_SLICES_COMPARISON_ONE_AT_A_TIME",
        &units,
        1,
        &judge_a_unit,
    );
    let on_several_worker_threads = outcomes_of_units_judged_in_slices_on_worker_threads(
        "CRASH_JUDGE_SLICES_COMPARISON_THREE_AT_A_TIME",
        &units,
        3,
        &judge_a_unit,
    );
    let expected: Vec<(u64, u64)> = units.iter().map(judge_a_unit).collect();
    assert_eq!(
        on_one_worker_thread, expected,
        "单线程那一趟每个单位一条结论、按单位次序"
    );
    assert_eq!(
        on_several_worker_threads, on_one_worker_thread,
        "多线程与单线程逐个相同"
    );
}

/// 扫描内核源码里入口 `entry_point` 那个函数体里头一处 `anchor` 之后插一行 `violate(bit);`，交回插过的源码。
///
/// # Panics
/// 同 [`plant_a_line_after_the_anchor_inside_the_entry_point`]。
fn plant_a_violated_bit_after_the_anchor_inside_the_entry_point(
    source: &str,
    entry_point: &str,
    anchor: &str,
    bit: usize,
) -> String {
    plant_a_line_after_the_anchor_inside_the_entry_point(
        source,
        entry_point,
        anchor,
        &format!("violate({bit}u);"),
    )
}

/// 扫描内核源码里入口 `entry_point` 那个函数体里头一处 `anchor` 之后插一行 `planted_line`（缩进四格），交回插过的源码。
///
/// # Panics
/// 源码里 `fn <entry_point>(` 不是恰好一处，或那个函数体（到它之后第一个 `\n}\n` 为止）里没有 `anchor`。
fn plant_a_line_after_the_anchor_inside_the_entry_point(
    source: &str,
    entry_point: &str,
    anchor: &str,
    planted_line: &str,
) -> String {
    let signature = format!("fn {entry_point}(");
    assert_eq!(
        source.matches(signature.as_str()).count(),
        1,
        "扫描内核源码里入口 {entry_point} 的函数恰好一个"
    );
    let entry_start = source.find(signature.as_str()).expect("上面核过恰好一处");
    let body_end = entry_start
        + source[entry_start..]
            .find("\n}\n")
            .expect("入口函数以单独一行的 `}` 收尾");
    let anchor_start = entry_start
        + source[entry_start..body_end]
            .find(anchor)
            .unwrap_or_else(|| panic!("入口 {entry_point} 的函数体里有 {anchor:?}"));
    let anchor_end = anchor_start + anchor.len();
    format!(
        "{}    {planted_line}\n{}",
        &source[..anchor_end],
        &source[anchor_end..]
    )
}

/// 在走读内核清零违例位之后种一个假的违例位（下标取 64 与池级 checker 条数里大的那个，落在高 64 位、不撞真的不变量）、
/// 在扫描内核源码里每个入口读回违例位之后各种一位，从第 128 位（下标 127）往下数：主入口（扫描那三组判的头一组）127，
/// 引用与快照那一截的入口（[`CHECKER_REFERENCES_AND_SNAPSHOTS_ENTRY_POINT`]）126，扫描重建那一截的入口（[`CHECKER_SCAN_REBUILD_ENTRY_POINT`]，
/// 判 I-1.13 之前要读另几截写下的违例位）125，扫描那三组判后两组的入口 124、123，记账头、分片、决断、收尾四趟的入口 122、121、120、119。
/// 两份种过的内核判出的每个状态，违例位等于没种的那一份判出的再并上种的位（第 65 位每个状态都有，扫描源码各入口种的那几位只在
/// 交接段有效、扫描内核判完的状态上有：这几个入口都只在交接段有效时读回违例位），
/// 判到位等于没种的那一份判出的再并上同样种的位（内核存回前把违例位并进判到位），其余的字逐字相同。
/// 违例位的高 64 位在走读内核写出、扫描内核源码里哪一个入口读回又写出、Rust 读出这几处任一处被截掉，这条红；哪一个入口不读回前一趟写的违例位
/// （只从零起记），在它之前的入口种的位就丢，这条也红。扫描内核源码里的入口（`@compute` 那几处）不是这几个时先红：多出来的入口也读回违例位，
/// 就要在这里给它也种一位。
#[test]
fn a_planted_violated_bit_past_the_sixty_fourth_reaches_the_verdict_through_both_checker_kernels() {
    const FIRST_BIT_OF_THE_HIGH_HALF: usize = 64;
    const SCAN_KERNEL_JUDGED_THE_STATE: u32 = 5;
    // 扫描内核源码里每个读回违例位的入口与它种的那一位（扫描那三组判、记账那四趟、引用与快照那一截、扫描重建那一截）
    let planted_by_each_scan_source_entry = [
        (KERNEL_ENTRY_POINT, 127),
        (CHECKER_REFERENCES_AND_SNAPSHOTS_ENTRY_POINT, 126),
        (CHECKER_SCAN_REBUILD_ENTRY_POINT, 125),
        (CHECKER_SCAN_ALLOCATION_RECORDS_DISJOINT_ENTRY_POINT, 124),
        (
            CHECKER_SCAN_ALLOCATION_GENERATIONS_THROUGH_THE_ROLLBACK_FLOOR_RAISES_ENTRY_POINT,
            123,
        ),
        (CHECKER_ACCOUNTING_HEAD_ENTRY_POINT, 122),
        (CHECKER_ACCOUNTING_SLICE_ENTRY_POINT, 121),
        (CHECKER_ACCOUNTING_DECIDE_ENTRY_POINT, 120),
        (CHECKER_ACCOUNTING_FINAL_ENTRY_POINT, 119),
    ];
    let planted_by_the_walk = IMPLEMENTED_INVARIANTS.len().max(FIRST_BIT_OF_THE_HIGH_HALF);
    let planted_bit_indexes: std::collections::BTreeSet<usize> = planted_by_each_scan_source_entry
        .iter()
        .map(|(_, bit)| *bit)
        .chain(std::iter::once(planted_by_the_walk))
        .collect();
    assert!(
        planted_bit_indexes.len() == planted_by_each_scan_source_entry.len() + 1
            && planted_by_each_scan_source_entry
                .iter()
                .all(|(_, bit)| planted_by_the_walk < *bit
                    && *bit < POOL_CHECKER_VIOLATED_BIT_CAPACITY),
        "种的 {} 位都在违例位容量之内、两两不同、不撞真的不变量（池级 checker {} 条）",
        planted_bit_indexes.len(),
        IMPLEMENTED_INVARIANTS.len()
    );
    let judge_sources = JudgeKernelSources::of_the_judge();
    let walk_anchor = "\n    clear_violated();\n";
    let scan_anchor = "\n    load_violated(out);\n";
    assert_eq!(
        judge_sources.checker_walk.matches(walk_anchor).count(),
        1,
        "走读内核里清零违例位的调用恰好一处"
    );
    assert_eq!(
        judge_sources.checker_scan.matches("@compute").count(),
        planted_by_each_scan_source_entry.len(),
        "扫描内核源码里的入口恰好是这几个：扫描那三组判、记账那四趟、引用与快照那一截、扫描重建那一截（多了一个入口就在这里给它种一位）"
    );
    assert_eq!(
        judge_sources.checker_scan.matches(scan_anchor).count(),
        planted_by_each_scan_source_entry.len(),
        "扫描内核源码里读回违例位的调用恰好每个入口各一处"
    );
    let checker_scan_planted_in_every_entry = planted_by_each_scan_source_entry.iter().fold(
        judge_sources.checker_scan.clone(),
        |source, (entry_point, bit)| {
            plant_a_violated_bit_after_the_anchor_inside_the_entry_point(
                &source,
                entry_point,
                scan_anchor,
                *bit,
            )
        },
    );
    let planted_sources = JudgeKernelSources {
        recovery: judge_sources.recovery.clone(),
        checker_walk: judge_sources.checker_walk.replacen(
            walk_anchor,
            &format!("{walk_anchor}    violate({planted_by_the_walk}u);\n"),
            1,
        ),
        checker_scan: checker_scan_planted_in_every_entry,
    };
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "a_planted_violated_bit_past_the_sixty_fourth_reaches_the_verdict_through_both_checker_kernels",
    ) else {
        return;
    };
    // 判的是种的位一路传到结论，不是崩溃域：每段固定展开到 8 次写（53 个状态），不跟展开上限变量走全量域（这条流全量域 1677 万个状态）
    const EXPAND_UP_TO: usize = 8;
    let flow = record_new_pool_file_creation("crash-judge-gpu-planted-bit", "new-pool");
    let expansion = |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= EXPAND_UP_TO {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let judge_flow = JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    };
    let tables = build_judge_tables(&judge_flow).expect("建得出表");
    let state_count = tables.state_count;
    // 判表先入库、再读库，GPU 只拿读回的那一份算
    let Some(packed) = packed_tables_through_the_library("a_planted_violated_bit_past_the_sixty_fourth_reaches_the_verdict_through_both_checker_kernels", &tables) else {
        return;
    };
    // 两个判器先后建；前一个的缓冲什么时候还给驱动看 wgpu 收拾的时机，借两份
    let Some(lease) = cards.lease_card_or_report_not_run(
        "a_planted_violated_bit_past_the_sixty_fourth_reaches_the_verdict_through_both_checker_kernels",
        2 * mebibytes_a_judge_needs(&packed, usize::MAX),
    ) else {
        return;
    };
    let card = lease.card();
    // 两套源码各建一个判器、判这条流，按区间切片分给工作线程：冷编一套要几分钟（驱动编一个内核是单线程的），两套的编叠在一起；
    // 判器在哪条线程上建就在哪条线程上接 Out of Memory（wgpu 的错误范围按线程分），合并按片号放回「没种、种过」的次序
    let kernel_source_sets = [
        ("建没种的判器、判这条流", &judge_sources),
        ("建种过的判器、判这条流", &planted_sources),
    ];
    let worker_threads = worker_threads_for_the_kernel_source_sets(
        cpu_threads_from_the_value_of_the_variable(
            process_environment_variable(CPU_THREADS_ENVIRONMENT_VARIABLE).as_deref(),
            process_environment_variable(THREAD_CAP_ENVIRONMENT_VARIABLE).as_deref(),
        ),
        process_environment_variable(GPU_KERNELS_ONE_AT_A_TIME_ENVIRONMENT_VARIABLE).as_deref(),
        kernel_source_sets.len(),
    );
    let judged_by_each_kernel_source_set = outcomes_of_units_judged_in_slices_on_worker_threads(
        "CRASH_JUDGE_PLANTED_BIT_KERNEL_SOURCE_SETS",
        &kernel_source_sets,
        worker_threads,
        &|(what_ran, kernel_sources): &(&str, &JudgeKernelSources)| {
            on_the_card_or_report_not_run_when_out_of_memory(
                "a_planted_violated_bit_past_the_sixty_fourth_reaches_the_verdict_through_both_checker_kernels",
                card,
                what_ran,
                || {
                    GpuCrashJudge::on_card_compiling_these_kernel_sources(
                        card,
                        &packed,
                        usize::MAX,
                        kernel_sources,
                    )
                    .judge_states(0..state_count)
                },
            )
        },
    );
    let [Some(unplanted), Some(planted)]: [Option<Vec<[u32; VERDICT_WORDS]>>; 2] =
        judged_by_each_kernel_source_set
            .try_into()
            .expect("两套源码各交回一份结论")
    else {
        return;
    };
    assert_eq!(unplanted.len(), planted.len());
    let mut states_the_scan_judged = 0u64;
    let mut mismatches = Vec::new();
    for (ordinal, (unplanted_words, planted_words)) in unplanted.iter().zip(&planted).enumerate() {
        assert_eq!(
            unplanted_words[POOL_CHECKER_UNDECIDABLE_WORD], 0,
            "状态 {ordinal}：没种的那一份池级 checker 判得了"
        );
        let scan_judged =
            unplanted_words[POOL_CHECKER_REACHED_WORD] == SCAN_KERNEL_JUDGED_THE_STATE;
        if scan_judged {
            states_the_scan_judged += 1;
        }
        let planted_bits = (1u128 << planted_by_the_walk)
            | if scan_judged {
                planted_by_each_scan_source_entry
                    .iter()
                    .fold(0u128, |bits, (_, bit)| bits | (1u128 << bit))
            } else {
                0
            };
        let expected_violated_bits = pool_checker_violated_bits(unplanted_words) | planted_bits;
        // 内核存回结论之前把违例位并进判到位（违例的一定判到），种的两位因此也落进判到位
        let expected_judged_bits = pool_checker_judged_bits(unplanted_words) | planted_bits;
        let violated_words = POOL_CHECKER_VIOLATED_FIRST_WORD
            ..POOL_CHECKER_VIOLATED_FIRST_WORD + POOL_CHECKER_VIOLATED_WORDS;
        let judged_words = POOL_CHECKER_JUDGED_FIRST_WORD
            ..POOL_CHECKER_JUDGED_FIRST_WORD + POOL_CHECKER_JUDGED_WORDS;
        let other_words_agree = unplanted_words
            .iter()
            .zip(planted_words)
            .enumerate()
            .filter(|(word_index, _)| {
                !violated_words.contains(word_index) && !judged_words.contains(word_index)
            })
            .all(|(_, (unplanted_word, planted_word))| unplanted_word == planted_word);
        let read_violated_bits = pool_checker_violated_bits(planted_words);
        let read_judged_bits = pool_checker_judged_bits(planted_words);
        if read_violated_bits != expected_violated_bits
            || read_judged_bits != expected_judged_bits
            || !other_words_agree
        {
            mismatches.push(format!(
                "状态 {ordinal}：没种 {unplanted_words:?}；种过 {planted_words:?}；违例位要 {expected_violated_bits:#x}，读出 {read_violated_bits:#x}；判到位要 {expected_judged_bits:#x}，读出 {read_judged_bits:#x}"
            ));
        }
    }
    println!(
        "CRASH_JUDGE_PLANTED_BIT states={state_count} scan_judged={states_the_scan_judged} mismatches={}",
        mismatches.len()
    );
    assert!(
        states_the_scan_judged > 0,
        "这条流上要有扫描内核判完的状态，扫描内核源码里各入口种的那几位才走得到"
    );
    assert!(
        mismatches.is_empty(),
        "{} 个状态上种的位没走到结论（前几个）：\n{}",
        mismatches.len(),
        mismatches
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// 种进扫描内核主入口的那个判不了的原因：「条目太多」一类（扫描内核的 UNDECIDABLE_TOO_MANY_ENTRIES，3）加一个内核里没用的站点号 250。
const PLANTED_UNDECIDABLE_REASON: u32 = 3 | (250 << 8);

/// 扫描内核源码主入口读回违例位之后（扫描那三组判的头一组之前）种一个判不了的原因：它经扫描那后两组、记账头、分片、决断几趟之间的交接段
/// （`C_ACCOUNTING_CARRY_UNDECIDABLE`）一路传到记账收尾那一趟，收尾那一趟把它写进结论里判不了的原因那一字，「走到第几步」那一字不写成 5
/// （结论只在收尾那一趟写一次）。新池新建文件那条流（展开 8）上：没有一个状态走到 5，扫描内核判了的状态都带着种的原因（至少一个）。
/// 哪一趟不读回或不存下交接段里的原因（例：记账头那一趟开头不读回，`crates/mutations.tsv` 里点名这一条的那一行），收尾那一趟就把这个状态记成走完，
/// 这条红。一份种过的内核一个判器：哪个状态扫描内核判了，看原因字是不是种的那一个就分得出（走读内核判不了的状态原因字是它自己的）。
#[test]
fn an_undecidable_reason_planted_before_the_scan_judges_reaches_the_verdict_through_every_later_scan_and_accounting_pass(
) {
    const TEST_NAME: &str = "an_undecidable_reason_planted_before_the_scan_judges_reaches_the_verdict_through_every_later_scan_and_accounting_pass";
    const EXPAND_UP_TO: usize = 8;
    const SCAN_KERNEL_JUDGED_THE_STATE: u32 = 5;
    let judge_sources = JudgeKernelSources::of_the_judge();
    let planted_sources = JudgeKernelSources {
        recovery: judge_sources.recovery.clone(),
        checker_walk: judge_sources.checker_walk.clone(),
        checker_scan: plant_a_line_after_the_anchor_inside_the_entry_point(
            &judge_sources.checker_scan,
            KERNEL_ENTRY_POINT,
            "\n    load_violated(out);\n",
            &format!("undecidable = {PLANTED_UNDECIDABLE_REASON}u;"),
        ),
    };
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let flow = record_new_pool_file_creation("crash-judge-gpu-planted-undecidable", "new-pool");
    let expansion = |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= EXPAND_UP_TO {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let judge_flow = JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    };
    let tables = build_judge_tables(&judge_flow).expect("建得出表");
    let state_count = tables.state_count;
    // 判表先入库、再读库，GPU 只拿读回的那一份算
    let Some(packed) = packed_tables_through_the_library(TEST_NAME, &tables) else {
        return;
    };
    let Some(lease) =
        cards.lease_card_or_report_not_run(TEST_NAME, mebibytes_a_judge_needs(&packed, usize::MAX))
    else {
        return;
    };
    let card = lease.card();
    let Some(planted) = on_the_card_or_report_not_run_when_out_of_memory(
        TEST_NAME,
        card,
        "建种过的判器、判这条流",
        || {
            GpuCrashJudge::on_card_compiling_these_kernel_sources(
                card,
                &packed,
                usize::MAX,
                &planted_sources,
            )
            .judge_states(0..state_count)
        },
    ) else {
        return;
    };
    drop(lease);
    let states_reached_by_the_scan_kernel = planted
        .iter()
        .filter(|words| words[POOL_CHECKER_REACHED_WORD] == SCAN_KERNEL_JUDGED_THE_STATE)
        .count();
    let states_carrying_the_planted_reason = planted
        .iter()
        .filter(|words| words[POOL_CHECKER_UNDECIDABLE_WORD] == PLANTED_UNDECIDABLE_REASON)
        .count();
    println!(
        "CRASH_JUDGE_PLANTED_UNDECIDABLE states={state_count} reached_by_the_scan_kernel={states_reached_by_the_scan_kernel} carrying_the_planted_reason={states_carrying_the_planted_reason}"
    );
    assert_eq!(
        states_reached_by_the_scan_kernel, 0,
        "种了判不了的原因，扫描内核一个状态都不该记成走完（走到 5）：有趟没读回或没存下交接段里的原因"
    );
    assert!(
        states_carrying_the_planted_reason > 0,
        "这条流上要有扫描内核判了的状态（结论里判不了的原因是种的那一个 {PLANTED_UNDECIDABLE_REASON:#x}）"
    );
}
