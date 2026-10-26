//! 攻方腿（m4-publish-lookup-code-r3）差分那一组历史的覆盖：只在改后那棵拷贝里编（读 `lookup_work` 计数），每段历史打一行
//! 各处按身份查了几次、解了几次映射叶，说明差分那一组历史真走到了整理发批核源落点、释放三处与快照判定那一圈。
//! 环境变量：R3_DIFF_SEEDS（每组比重、每种盘宽几段，默认 8）、R3_DIFF_STEPS（每段几步，默认 40）、
//! SINGLEFS_THREAD_CAP（并行几段，默认 1）、R3_DIFF_SEED_OFFSET（种子起点，默认 0）。
use singlefs_core::admission::SpaceAdmission;
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, HistoryDeviceWidth, HistoryEnding,
    HistoryExecution, HistorySeed, NamedGenerationWeights, PerStepChecker,
};
use singlefs_harness::SharedStream;
use singlefs_core::transaction::lookup_work::{lookup_work_while, LookupSite};

fn environment_number(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|text| text.parse().ok())
        .unwrap_or(default)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn digest(bytes: &[u8]) -> String {
    hex(&singlefs_harness::sha256::sha256_digest(bytes))
}

fn ending_summary(ending: &HistoryEnding) -> String {
    match ending {
        HistoryEnding::Completed => "completed".to_string(),
        HistoryEnding::KnownRed { form, .. } => format!("known_red_form_{form}"),
        HistoryEnding::NewFinding { signature, .. } => {
            format!("new_finding {}", digest(format!("{signature:?}").as_bytes()))
        }
    }
}

fn one_history(
    named: NamedGenerationWeights,
    width: HistoryDeviceWidth,
    seed: u64,
    steps: usize,
) -> String {
    let weights = named.weights();
    let history = generate_history_with_weights(HistorySeed(seed), steps, &weights);
    let stream = SharedStream::new();
    let mut step_digests = Vec::new();
    let (run, work) = lookup_work_while(|| execute_history_with(
        &history,
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: width,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        },
        &stream,
        &mut |observation| {
            let mut written = Vec::new();
            for (identity, device) in &observation.image.devices {
                written.extend_from_slice(&identity.0.to_le_bytes());
                for (sector, bytes) in device.written_sectors() {
                    written.extend_from_slice(&sector.to_le_bytes());
                    written.extend_from_slice(bytes);
                }
            }
            step_digests.extend_from_slice(&singlefs_harness::sha256::sha256_digest(&written));
        },
    ));
    let _ = &step_digests;
    let sites: Vec<String> = [LookupSite::SnapshotJudgement, LookupSite::MappingEntriesOfReleasedUnits, LookupSite::CompactionReadsTheCentralMapping, LookupSite::RelocationSources]
        .iter()
        .map(|site| { let w = work.of_the_site(*site); format!("{site:?}:lookups={},builds={},leaf_decodes={}", w.index_lookups, w.index_builds, w.mapping_leaf_decodes) })
        .collect();
    format!(
        "name=coverage weights={} width={} seed={seed} ending={} {}",
        named.environment_value(),
        width.name(),
        ending_summary(&run.ending),
        sites.join(" ")
    )
}

#[test]
fn r3_opus_coverage_of_the_differential_histories() {
    let seeds = environment_number("R3_DIFF_SEEDS", 8);
    let offset = environment_number("R3_DIFF_SEED_OFFSET", 0);
    let steps = usize::try_from(environment_number("R3_DIFF_STEPS", 40)).expect("步数");
    let threads = usize::try_from(environment_number("SINGLEFS_THREAD_CAP", 1).max(1)).expect("线程");
    let mut jobs = Vec::new();
    for named in [
        NamedGenerationWeights::SnapshotOperations,
        NamedGenerationWeights::CompactionWhileMounted,
        NamedGenerationWeights::CompactionWhileMountedWithRollback,
        NamedGenerationWeights::BroadWithCompactionWhileMounted,
        NamedGenerationWeights::Truncation,
        NamedGenerationWeights::WritesByInodeNumber,
    ] {
        for width in [HistoryDeviceWidth::FourGibibytes, HistoryDeviceWidth::UnitAreaOf240Slots] {
            for seed in offset..offset + seeds {
                jobs.push((named, width, seed));
            }
        }
    }
    let next = std::sync::atomic::AtomicUsize::new(0);
    let mut lines: Vec<(usize, String)> = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|_| {
                scope.spawn(|| {
                    let mut mine = Vec::new();
                    loop {
                        let job = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        let Some((named, width, seed)) = jobs.get(job).copied() else {
                            break;
                        };
                        mine.push((job, one_history(named, width, seed, steps)));
                    }
                    mine
                })
            })
            .collect();
        workers
            .into_iter()
            .flat_map(|worker| worker.join().expect("差分线程"))
            .collect()
    });
    lines.sort();
    for (_, line) in &lines {
        println!("{line}");
    }
}
