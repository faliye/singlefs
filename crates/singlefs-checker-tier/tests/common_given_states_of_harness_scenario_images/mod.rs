//! harness 档场景交出来的镜像经给定状态引擎两段判、再逐张核场景写下的池级 checker 该判成什么（用户 2026-10-14 弹窗定「搬进 checker 档两段」）：
//! 判法只这一份，checker 档里收 harness 场景镜像的测试文件经 `mod` 引它，并经 `#[path]` 引 harness 档的 `common_images_for_the_checker_tier`。
//! 块键只由表的内容定（设点文件、项名、场景名都不进键，场景名只进行里报的项名）。
//!
//! 每张镜像建成一张给定状态表（基线就是这张镜像、写表为空、一个空的持久集合：池级 checker 判的就是这张镜像本身，记录核对器在空写表上
//! 没有发布可判；交的是一批崩溃态时基线、写表、切段与持久集合照交的），交崩溃点重放的给定状态引擎（`crash_amplification::GivenStatesEngine`）：
//! 先写进给定状态表那一类的判定库、判段从库里读回来判，配了 GPU 只由 GPU 判器判、没配只由 CPU 判器判，结论按表存进库、判过的复用。
//!
//! 判到位与违例细节库里不存（复用来的结论没有）：场景里有一张镜像要核判成立、没判到、违例细节（连同带场景这一刻算出的一个数的细节判法）
//! 或与先交的别的镜像比的，这条用例在自己那一份一次性的库里判（[`LibraryOfTheTest`]：`${TMPDIR:-/tmp}` 下一份这个进程独占的一次性目录，
//! 名字带进程号与进程内序号，录入段起跑先清空、用完连同库一起删；长期库不碰），结论都是这一趟现判的；违例细节只有 CPU 判器给，GPU 判的结论上那几样只核违例位。别的用例走那一份长期库
//! （入口脚本按趟交的 `SINGLEFS_CRASH_AMPLIFICATION_LIBRARY`，没交就是家目录下的）。

use crate::common_images_for_the_checker_tier::{
    ComparisonWithAnEarlierImage, PoolCheckerImage, PoolCheckerImages,
};
use singlefs_checker::image::IMPLEMENTED_INVARIANTS;
use singlefs_checker::walk::ContainerCapacityKnobs;
use singlefs_checker_tier::crash_amplification::{
    one_test_changes_the_environment_at_a_time, GivenStatesEngine, GivenStatesItem, GivenStatesRun,
};
use singlefs_checker_tier::crash_given_states::{
    ClaimedVersionsOfTheStates, GivenStateConclusion, GivenStatesConclusionsSource,
    GivenStatesJudge, GivenStatesOfATable, GivenStatesOfATableWithItsColumns,
};
use singlefs_checker_tier::crash_library::{
    ExclusiveTemporaryDirectory, RunStage, CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE,
};

/// 写明这份库是一次性的那个变量（`crash_library::refuse_a_library_that_a_reboot_would_erase`）：只在起引擎那一刻设。
const LIBRARY_IS_TEMPORARY_VARIABLE: &str = "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY_IS_TEMPORARY";
use singlefs_checker_tier::crash_point_replay_progress::CrashPointReplayToolchainIdentity;
use singlefs_harness::gpu_memory_ledger::gpu_not_run_line;

/// 给定状态引擎里这一项的名字（进块键与库里的清单名；每个场景的种子与规模那一格写场景名）。
pub const ITEM_NAME: &str = "harness-scenario-images";

/// 要核判到位、违例细节或与别的镜像比的用例各判在自己的一份一次性的库里（[`LibraryOfTheTest`]）：那份一次性目录的标签以这一段起头、再接用例名
/// （不放进长期库的家目录，用户 2026-10-15 定：流水线单测起跑不清长期库）。
const FRESH_LIBRARY_LABEL_PREFIX: &str = "harness-scenario-images-judged-afresh";

/// 一条用例自己那一份一次性的库：`${TMPDIR:-/tmp}` 下一份这个进程独占的一次性目录（`ExclusiveTemporaryDirectory`，名字带进程号与进程内序号，
/// 两个进程同跑一个测试二进制各是各的），库建在它底下（库目录本身由引擎建）；丢掉时连同库一起删，只删这一份。
/// 同一条用例前后几趟要读同一份库的（[`LibraryOfTheTestAtTheStart::KeptFromTheEarlierRunOfTheTest`]），起引擎时交同一份，不按用例名重算路径。
pub struct LibraryOfTheTest {
    directory: ExclusiveTemporaryDirectory,
}

impl LibraryOfTheTest {
    /// 建这条用例自己那一份库。录入段与判段分进程跑（`SINGLEFS_CRASH_AMPLIFICATION_STAGE=record` / `judge`）时不建：这一份只在这个进程里，
    /// 判段那个进程拿不到录入段那个进程的这一份（跨进程怎么交这份库没定），打「本次未跑」那一行、交 `None`。
    #[must_use]
    pub fn create_or_report_not_run(test_name: &str) -> Option<Self> {
        match RunStage::of_this_process() {
            RunStage::RecordThenJudge => Some(Self {
                directory: ExclusiveTemporaryDirectory::create(&format!(
                    "{FRESH_LIBRARY_LABEL_PREFIX}-{test_name}"
                ))
                .unwrap_or_else(|reason| {
                    panic!("{test_name} 自己那一份一次性的库建不了：{reason}")
                }),
            }),
            stage @ (RunStage::RecordOnly | RunStage::JudgeOnly) => {
                println!(
                    "{}",
                    gpu_not_run_line(
                        test_name,
                        &format!(
                            "这条用例在自己那一份一次性的库里判，录入段与判段分进程跑（这一趟只跑 {}）时两段拿不到同一份库：跨进程怎么交这份库没定",
                            stage.name()
                        )
                    )
                );
                None
            }
        }
    }

    /// 库目录（第一次起引擎、录入段跑时由引擎建）。
    #[must_use]
    pub fn library(&self) -> std::path::PathBuf {
        self.directory.join("library")
    }
}

/// 一张镜像（一批崩溃态）在这一趟的结论（逐状态，与持久集合同序）与它从哪来。
pub struct ConclusionsOfAnImage {
    pub states: Vec<GivenStateConclusion>,
    pub source: GivenStatesConclusionsSource,
}

/// `IMPLEMENTED_INVARIANTS` 里这一条的位置（对拍字里的位号）。
fn position_of(invariant: &str) -> usize {
    IMPLEMENTED_INVARIANTS
        .iter()
        .position(|implemented| *implemented == invariant)
        .unwrap_or_else(|| panic!("{invariant} 不在池级 checker 已实现的不变量里"))
}

pub fn is_violated(conclusion: &GivenStateConclusion, invariant: &str) -> bool {
    conclusion.words.pool_checker_violated_bits & (1u128 << position_of(invariant)) != 0
}

pub fn is_judged(conclusion: &GivenStateConclusion, invariant: &str) -> bool {
    conclusion.words.pool_checker_judged_bits & (1u128 << position_of(invariant)) != 0
}

/// 这一条的违例细节（只有这一趟 CPU 判器判的结论带）。
fn violation_detail_of<'conclusion>(
    conclusion: &'conclusion GivenStateConclusion,
    invariant: &str,
) -> Option<&'conclusion str> {
    conclusion
        .pool_checker_violation_details
        .iter()
        .find(|(violated, _)| *violated == invariant)
        .map(|(_, detail)| detail.as_str())
}

/// 这张镜像上要不要核判到位、违例细节或与别的镜像比（要就得这一趟现判：库里不存它们）。
fn needs_a_fresh_judgement(image: &PoolCheckerImage) -> bool {
    !image.expected.holds.is_empty()
        || !image.expected.not_judged.is_empty()
        || !image.expected.violation_detail_contains.is_empty()
        || !image
            .expected
            .violation_detail_if_violated_satisfies
            .is_empty()
        || !image
            .expected
            .violation_detail_if_violated_satisfies_given_a_value
            .is_empty()
        || !image.expected.compared_with_an_earlier_image.is_empty()
}

/// 一个状态的结论与场景交它时写的池级 checker 该判成什么对不上的地方，一条一句（空就是对得上；不带镜像名，[`mismatches_of_the_images`] 接上）。
/// `judged_afresh`：结论是这一趟现判的（判到位作数）。与别的镜像比的那几样另在 [`mismatches_against_the_earlier_images`] 里核。
fn mismatches_of_one_state(
    image: &PoolCheckerImage,
    conclusion: &GivenStateConclusion,
    judged_afresh: bool,
) -> Vec<String> {
    let expected = &image.expected;
    let violated_now: Vec<&'static str> = IMPLEMENTED_INVARIANTS
        .iter()
        .copied()
        .filter(|invariant| is_violated(conclusion, invariant))
        .collect();
    let mut mismatches = Vec::new();
    if let Some(exactly) = &expected.exactly_violated {
        let mut wanted: Vec<&'static str> = exactly.clone();
        wanted.sort_by_key(|invariant| position_of(invariant));
        wanted.dedup();
        if wanted != violated_now {
            mismatches.push(format!(
                "判红的该恰好是 {wanted:?}，实际 {violated_now:?}（细节：{:?}）",
                conclusion.pool_checker_violations()
            ));
        }
    }
    for invariant in &expected.violated {
        if !is_violated(conclusion, invariant) {
            mismatches.push(format!(
                "{invariant} 该判红，没红（判红的是 {violated_now:?}）"
            ));
        }
    }
    for invariant in &expected.not_violated {
        if is_violated(conclusion, invariant) {
            mismatches.push(format!(
                "{invariant} 不该红，红了（细节：{:?}）",
                conclusion.pool_checker_violations()
            ));
        }
    }
    for invariant in &expected.holds {
        assert!(
            judged_afresh,
            "要核判成立的镜像只在现判的结论上核（判到位库里不存）"
        );
        if is_violated(conclusion, invariant) || !is_judged(conclusion, invariant) {
            mismatches.push(format!(
                "{invariant} 该判成立：违例 {}、判到 {}",
                is_violated(conclusion, invariant),
                is_judged(conclusion, invariant)
            ));
        }
    }
    for invariant in &expected.not_judged {
        assert!(
            judged_afresh,
            "要核没判到的镜像只在现判的结论上核（判到位库里不存）"
        );
        if is_violated(conclusion, invariant) || is_judged(conclusion, invariant) {
            mismatches.push(format!(
                "{invariant} 该没判到（不适用或部分没判）：违例 {}、判到 {}",
                is_violated(conclusion, invariant),
                is_judged(conclusion, invariant)
            ));
        }
    }
    for (invariant, text) in &expected.violation_detail_contains {
        if !is_violated(conclusion, invariant) {
            mismatches.push(format!("{invariant} 该判红（细节里有「{text}」），没红"));
            continue;
        }
        // 细节只有 CPU 判器给：GPU 判的结论上这一样只核违例位
        if let Some(detail) = violation_detail_of(conclusion, invariant) {
            if !detail.contains(text.as_str()) {
                mismatches.push(format!("{invariant} 的违例细节里该有「{text}」：{detail}"));
            }
        }
    }
    for condition in &expected.violation_detail_if_violated_satisfies {
        // 细节只有 CPU 判器给：GPU 判的结论上这一样不核
        if let Some(detail) = violation_detail_of(conclusion, condition.invariant) {
            if !(condition.holds_for)(detail) {
                mismatches.push(format!(
                    "{} 红了，违例细节该满足「{}」：{detail}",
                    condition.invariant, condition.described_as
                ));
            }
        }
    }
    for condition in &expected.violation_detail_if_violated_satisfies_given_a_value {
        // 同上：细节只有 CPU 判器给
        if let Some(detail) = violation_detail_of(conclusion, condition.invariant) {
            if !(condition.holds_for)(detail, condition.value) {
                mismatches.push(format!(
                    "{} 红了，违例细节该满足「{}」（那个数是 {}）：{detail}",
                    condition.invariant, condition.described_as, condition.value
                ));
            }
        }
    }
    mismatches
}

/// 一张镜像与同一个场景里先交的那几张比（「改坏之后别的判决逐条不变」这一类）：对不上的地方一条一句。
/// 比的两张都只许是一张镜像（一个状态），先交的那一张在 `images` 里排在它前面。
fn mismatches_against_the_earlier_images(
    position_of_the_image: usize,
    images: &[PoolCheckerImage],
    conclusions: &[ConclusionsOfAnImage],
    judged_afresh: bool,
) -> Vec<String> {
    let image = &images[position_of_the_image];
    let mut mismatches = Vec::new();
    for compared in &image.expected.compared_with_an_earlier_image {
        assert!(
            judged_afresh,
            "与别的镜像比的镜像只在现判的结论上核（判到位与细节库里不存）"
        );
        let position_of_the_earlier = images[..position_of_the_image]
            .iter()
            .position(|earlier| earlier.label == compared.label)
            .unwrap_or_else(|| {
                panic!(
                    "{}：要比的「{}」不是同一个场景里先交的镜像",
                    image.label, compared.label
                )
            });
        assert!(
            image.persisted_sets.len() == 1
                && images[position_of_the_earlier].persisted_sets.len() == 1,
            "{}：与别的镜像比的两张都只许是一张镜像（一个状态）",
            image.label
        );
        let here = &conclusions[position_of_the_image].states[0];
        let there = &conclusions[position_of_the_earlier].states[0];
        match &compared.comparison {
            ComparisonWithAnEarlierImage::JudgedAlikeExcept(except) => {
                let differing: Vec<&'static str> = IMPLEMENTED_INVARIANTS
                    .iter()
                    .copied()
                    .filter(|invariant| !except.contains(invariant))
                    .filter(|invariant| {
                        is_violated(here, invariant) != is_violated(there, invariant)
                            || is_judged(here, invariant) != is_judged(there, invariant)
                    })
                    .collect();
                if !differing.is_empty() {
                    mismatches.push(format!(
                        "除了 {except:?}，判到与违例该与「{}」逐条相同，这几条不同：{differing:?}",
                        compared.label
                    ));
                }
            }
            ComparisonWithAnEarlierImage::AtMostThisManyJudgedThereAreNotJudgedHere(at_most) => {
                let judged_there_not_here: Vec<&'static str> = IMPLEMENTED_INVARIANTS
                    .iter()
                    .copied()
                    .filter(|invariant| is_judged(there, invariant) && !is_judged(here, invariant))
                    .collect();
                if judged_there_not_here.len() > *at_most {
                    mismatches.push(format!(
                        "「{}」上判到、这一张上没判到的至多 {at_most} 条，实际 {judged_there_not_here:?}",
                        compared.label
                    ));
                }
                if here.words.pool_checker_judged_bits == 0 {
                    mismatches.push("这一张上一条都没判到（整片不适用）".to_string());
                }
            }
            ComparisonWithAnEarlierImage::ViolatedAlike => {
                if here.words.pool_checker_violated_bits != there.words.pool_checker_violated_bits {
                    mismatches.push(format!(
                        "判红的该与「{}」逐条相同：这一张 {:?}，那一张 {:?}",
                        compared.label,
                        here.pool_checker_violations(),
                        there.pool_checker_violations()
                    ));
                }
                for invariant in IMPLEMENTED_INVARIANTS {
                    if let (Some(detail_here), Some(detail_there)) = (
                        violation_detail_of(here, invariant),
                        violation_detail_of(there, invariant),
                    ) {
                        if detail_here != detail_there {
                            mismatches.push(format!(
                                "{invariant} 的违例细节该与「{}」逐字相同：这一张「{detail_here}」，那一张「{detail_there}」",
                                compared.label
                            ));
                        }
                    }
                }
            }
        }
    }
    mismatches
}

/// 收下的每张镜像的结论与场景写下的该判成什么对不上的地方，一条一句、前面接镜像名（一批崩溃态的再接持久集合下标）：
/// 逐状态的那几样（[`mismatches_of_one_state`]）与跟先交的镜像比的那几样（[`mismatches_against_the_earlier_images`]）。
/// `conclusions` 与 `images` 同序；`judged_afresh`：结论是这一趟现判的（判到位与细节作数）。
pub fn mismatches_of_the_images(
    images: &[PoolCheckerImage],
    conclusions: &[ConclusionsOfAnImage],
    judged_afresh: bool,
) -> Vec<String> {
    assert_eq!(
        images.len(),
        conclusions.len(),
        "每张镜像一份结论，与交镜像的次序相同"
    );
    let mut mismatches: Vec<String> = Vec::new();
    for (position_of_the_image, (image, judged)) in images.iter().zip(conclusions).enumerate() {
        for (state_index, conclusion) in judged.states.iter().enumerate() {
            for mismatch in mismatches_of_one_state(image, conclusion, judged_afresh) {
                if image.persisted_sets.len() == 1 {
                    mismatches.push(format!("{}：{mismatch}", image.label));
                } else {
                    mismatches.push(format!(
                        "{}：崩溃态（持久集合下标 {state_index}）{mismatch}",
                        image.label
                    ));
                }
            }
        }
        for mismatch in mismatches_against_the_earlier_images(
            position_of_the_image,
            images,
            conclusions,
            judged_afresh,
        ) {
            mismatches.push(format!("{}：{mismatch}", image.label));
        }
    }
    mismatches
}

/// 起引擎：照此刻的环境（多机配置、显卡先后、选段、分片）。`library_of_the_test` 给了，库换成这条用例自己的那一份
/// （录入段起跑先清空），引擎起完把库目录那个变量还原；拿着改环境的那把锁起，不读到别的用例设的值。
pub fn start_the_engine(
    scenario_name: &str,
    library_of_the_test: Option<&LibraryOfTheTest>,
) -> GivenStatesEngine {
    start_the_engine_on_the_library_of_the_test(
        scenario_name,
        library_of_the_test,
        LibraryOfTheTestAtTheStart::EmptiedWhenTheRecordStageRuns,
    )
}

/// 用例自己那一份库在引擎起步时怎么办。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LibraryOfTheTestAtTheStart {
    /// 录入段起跑先清空（这一趟的结论都是现判的）。
    EmptiedWhenTheRecordStageRuns,
    /// 留着同一条用例前一趟写下的（钉「从库里复用回来」那一判）。
    KeptFromTheEarlierRunOfTheTest,
}

/// 同 [`start_the_engine`]，`library_at_the_start` 定用例自己那一份库起步时清不清空。
pub fn start_the_engine_on_the_library_of_the_test(
    scenario_name: &str,
    library_of_the_test: Option<&LibraryOfTheTest>,
    library_at_the_start: LibraryOfTheTestAtTheStart,
) -> GivenStatesEngine {
    let _one_test_at_a_time = one_test_changes_the_environment_at_a_time();
    let item = GivenStatesItem {
        item_name: format!("{}::scenario={scenario_name}", ITEM_NAME),
    };
    let toolchain = CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test();
    let Some(library_of_the_test) = library_of_the_test else {
        return GivenStatesEngine::start_from_environment(item, &toolchain)
            .expect("崩溃点重放的引擎照本机配置起得来（配了 GPU 卡用不上、结论版本与编译时对不上都在这里红，不退回 CPU）");
    };
    // 库目录本身由引擎建（建库时目录已在会拒，verdict_store 的 LibraryDirectoryAlreadyExists）；上一层是这条用例独占的那一份一次性目录
    let directory = library_of_the_test.library();
    if library_at_the_start == LibraryOfTheTestAtTheStart::EmptiedWhenTheRecordStageRuns
        && RunStage::of_this_process().records()
        && directory.exists()
    {
        std::fs::remove_dir_all(&directory)
            .unwrap_or_else(|error| panic!("清不掉 {}：{error}", directory.display()));
    }
    // 一次性的库落在 TMPDIR 下：起引擎那一刻写明它是一次性的（引擎拒开机会清空的地方的长期库），起完连同库目录变量一起还原
    let previous = std::env::var_os(CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE);
    let previous_temporary_mark = std::env::var_os(LIBRARY_IS_TEMPORARY_VARIABLE);
    std::env::set_var(CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE, &directory);
    std::env::set_var(LIBRARY_IS_TEMPORARY_VARIABLE, "1");
    let started = GivenStatesEngine::start_from_environment(item, &toolchain);
    match previous {
        Some(value) => std::env::set_var(CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE, value),
        None => std::env::remove_var(CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE),
    }
    match previous_temporary_mark {
        Some(value) => std::env::set_var(LIBRARY_IS_TEMPORARY_VARIABLE, value),
        None => std::env::remove_var(LIBRARY_IS_TEMPORARY_VARIABLE),
    }
    started.expect("崩溃点重放的引擎照本机配置起得来（配了 GPU 卡用不上、结论版本与编译时对不上都在这里红，不退回 CPU）")
}

/// 一张镜像建成一张给定状态表（基线是它、写表为空、一个空的持久集合；交的是一批崩溃态时基线、写表、切段与持久集合照交的），
/// 旋钮栏填 `container_capacity_knobs`（场景写这批镜像时容器树压没压容量；没压是 [`ContainerCapacityKnobs::PRODUCT`]），交引擎判。
/// 判不了就 panic：判定库读写失败、配了 GPU 卡用不上都判红。
pub fn judge_one_image(
    engine: &GivenStatesEngine,
    image: &PoolCheckerImage,
    container_capacity_knobs: ContainerCapacityKnobs,
) -> ConclusionsOfAnImage {
    let table = GivenStatesOfATable {
        table_name: image.label.clone(),
        base: &image.image,
        writes: &image.writes,
        segments: &image.segments,
        seams: &[],
        persisted_sets: image.persisted_sets.clone(),
    };
    let judged = engine
        .judge_the_given_states_with_its_columns(GivenStatesOfATableWithItsColumns {
            table: &table,
            container_capacity_knobs,
            claimed_versions: ClaimedVersionsOfTheStates::EveryStateLandedOnByTheConsultedRecovery,
        })
        .unwrap_or_else(|reason| panic!("镜像 {} 判不了（不退回别处判）：{reason}", image.label));
    assert_eq!(
        judged.states.len(),
        image.persisted_sets.len(),
        "{}：判器逐状态交回结论，与表里的状态同序",
        image.label
    );
    ConclusionsOfAnImage {
        states: judged.states,
        source: judged.source,
    }
}

/// 这一趟作不作数：叫停了、只跑了录入段、两台分片而有表归别的机器判，打「本次未跑」那一行、交 `false`。
pub fn the_run_counts(
    run: &GivenStatesRun,
    conclusions: &[ConclusionsOfAnImage],
    scenario_name: &str,
) -> bool {
    let not_judged_here = |wanted: GivenStatesConclusionsSource| {
        conclusions
            .iter()
            .filter(|judged| judged.source == wanted)
            .count()
    };
    if run.stopped_early {
        println!(
            "{}",
            gpu_not_run_line(
                scenario_name,
                "叫停了（SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE 指的文件在）：判过的表在库里，下一趟接着判"
            )
        );
        return false;
    }
    let recorded_only =
        not_judged_here(GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly);
    if run.recorded_only() || recorded_only > 0 {
        println!(
            "{}",
            gpu_not_run_line(
                scenario_name,
                &format!(
                    "只跑了录入段（SINGLEFS_CRASH_AMPLIFICATION_STAGE=record）：{recorded_only} 张镜像写进了库、没判，判段那一趟从库里读回判才作数"
                )
            )
        );
        return false;
    }
    let left = not_judged_here(GivenStatesConclusionsSource::PlaceholderOfATableLeftToAnotherHost);
    if run.left_tables_to_the_other_hosts() || left > 0 {
        println!(
            "{}",
            gpu_not_run_line(
                scenario_name,
                &format!(
                    "两台分片：这一份是 {}/{}，{left} 张镜像归别的机器判、还没导回来（导入那一台的库之后再跑一趟才作数）",
                    run.share.index, run.share.count
                )
            )
        );
        return false;
    }
    true
}

/// 场景写下的该判成什么里没有要现判的那几样（判成立、没判到、违例细节、与别的镜像比）时，还要不要在这条用例自己那一份清空过的库里现判
/// （用例要在几张镜像合起来的结论上核判到位：随机历史「某一步上判成立」那一判）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JudgedAfreshRegardlessOfTheExpectations {
    Yes,
    OnlyIfAnImageExpectsIt,
}

/// 收下的镜像逐张建表交引擎判（旋钮栏都填 `container_capacity_knobs`；场景里有一张要核判到位、违例细节或与别的镜像比，
/// 或 `judged_afresh_regardless`，就在这条用例自己那一份清空过的库里判），收尾，再核 GPU 判不了的只许是场景许它建不出判表的那几张。
/// 交回逐张的结论、引擎那一趟与结论是不是这一趟现判的；这一趟不作数（叫停、只跑录入段、两台分片而有表归别的机器判）打「本次未跑」那一行、交 `None`。
pub fn judge_the_collected_images(
    scenario_name: &str,
    images: &[PoolCheckerImage],
    container_capacity_knobs: ContainerCapacityKnobs,
    judged_afresh_regardless: JudgedAfreshRegardlessOfTheExpectations,
) -> Option<(Vec<ConclusionsOfAnImage>, GivenStatesRun, bool)> {
    let judged_afresh = images.iter().any(needs_a_fresh_judgement)
        || judged_afresh_regardless == JudgedAfreshRegardlessOfTheExpectations::Yes;
    let library_of_the_test = if judged_afresh {
        Some(LibraryOfTheTest::create_or_report_not_run(scenario_name)?)
    } else {
        None
    };
    let engine = start_the_engine(scenario_name, library_of_the_test.as_ref());
    let conclusions: Vec<ConclusionsOfAnImage> = images
        .iter()
        .map(|image| judge_one_image(&engine, image, container_capacity_knobs))
        .collect();
    let run = engine.finish();
    for (image, judged) in images.iter().zip(&conclusions) {
        let violated: Vec<Vec<(&'static str, String)>> = judged
            .states
            .iter()
            .map(GivenStateConclusion::pool_checker_violations)
            .collect();
        eprintln!(
            "HARNESS_SCENARIO_IMAGE scenario={scenario_name} image={} states={} pool_checker_violations={violated:?}",
            image.label,
            judged.states.len()
        );
    }
    if !the_run_counts(&run, &conclusions, scenario_name) {
        return None;
    }
    assert!(
        !judged_afresh || run.tally.tables_reused == 0,
        "在自己那一份清空过的库里判的用例，结论都该是这一趟现判的：复用了 {} 张",
        run.tally.tables_reused
    );
    let refused_beyond_what_the_scenario_allows: Vec<&String> = run
        .tally
        .states_the_gpu_could_not_judge
        .iter()
        .filter(|state| {
            !images.iter().any(|image| {
                image.the_gpu_may_refuse_its_tables_because.is_some()
                    && state.starts_with(&format!("{} ", image.label))
            })
        })
        .collect();
    assert!(
        refused_beyond_what_the_scenario_allows.is_empty(),
        "场景 {scenario_name}：GPU 判不了的镜像（场景没许它建不出判表）：{refused_beyond_what_the_scenario_allows:?}"
    );
    Some((conclusions, run, judged_afresh))
}

/// 场景一张镜像都不交时怎么办。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WhenTheScenarioHandsOutNoImage {
    /// 判红：这个场景每一趟都该交镜像（harness 档那一侧 `run_a_harness_scenario_leaving_its_images_to_the_checker_tier` 判同一条）。
    Reddens,
    /// 照常：这个场景只在某一条路上交镜像（例如可写挂载做成了才判写完之后那一张；拒了就没有镜像可判），打一行写明，不判红。
    IsExpectedOnTheRefusingPath,
}

/// 跑一个 harness 场景（收镜像的那一种），把它交出来的每张镜像经给定状态引擎判，逐张核场景写下的池级 checker 该判成什么：
/// 对不上的、GPU 判不了的（场景没许它建不出判表的）都判红，一张都不许交不出来。
pub fn judge_the_images_a_harness_scenario_hands_out(
    scenario_name: &str,
    scenario: impl FnOnce(&mut PoolCheckerImages),
) {
    judge_the_images_a_harness_scenario_hands_out_when_it_hands_out_none(
        scenario_name,
        scenario,
        WhenTheScenarioHandsOutNoImage::Reddens,
    );
}

/// 同 [`judge_the_images_a_harness_scenario_hands_out`]，场景写这批镜像时把容器树压小了：每张表的旋钮栏填 `container_capacity_knobs`
/// （用户 2026-10-14 弹窗定「给定状态表加旋钮栏」），引擎按它判（CPU 判器调池级 checker 带旋钮值的入口，GPU 判器把旋钮值编进扫描内核）。
pub fn judge_the_images_a_harness_scenario_hands_out_under_the_container_capacity_knobs(
    scenario_name: &str,
    container_capacity_knobs: ContainerCapacityKnobs,
    scenario: impl FnOnce(&mut PoolCheckerImages),
) {
    judge_the_images_a_harness_scenario_hands_out_under_the_knobs_when_it_hands_out_none(
        scenario_name,
        scenario,
        container_capacity_knobs,
        WhenTheScenarioHandsOutNoImage::Reddens,
    );
}

/// 同 [`judge_the_images_a_harness_scenario_hands_out`]，一张镜像都不交时照 `when_none` 办。
pub fn judge_the_images_a_harness_scenario_hands_out_when_it_hands_out_none(
    scenario_name: &str,
    scenario: impl FnOnce(&mut PoolCheckerImages),
    when_none: WhenTheScenarioHandsOutNoImage,
) {
    judge_the_images_a_harness_scenario_hands_out_under_the_knobs_when_it_hands_out_none(
        scenario_name,
        scenario,
        ContainerCapacityKnobs::PRODUCT,
        when_none,
    );
}

/// 跑场景、收镜像、逐张建表（旋钮栏填 `container_capacity_knobs`）交引擎判、逐张核场景写下的该判成什么（[`mismatches_of_the_images`]）；
/// 一张镜像都不交时照 `when_none` 办。
fn judge_the_images_a_harness_scenario_hands_out_under_the_knobs_when_it_hands_out_none(
    scenario_name: &str,
    scenario: impl FnOnce(&mut PoolCheckerImages),
    container_capacity_knobs: ContainerCapacityKnobs,
    when_none: WhenTheScenarioHandsOutNoImage,
) {
    let mut handed_out = PoolCheckerImages::collected_for_the_checker_tier();
    scenario(&mut handed_out);
    let images = handed_out.into_collected();
    if images.is_empty() {
        match when_none {
            WhenTheScenarioHandsOutNoImage::Reddens => {
                panic!("场景 {scenario_name} 一张镜像都没交给池级 checker")
            }
            WhenTheScenarioHandsOutNoImage::IsExpectedOnTheRefusingPath => {
                println!(
                    "HARNESS_SCENARIO_HANDED_OUT_NO_IMAGE scenario={scenario_name}：场景走了不交镜像的那一条路（拒了），没有镜像可判"
                );
                return;
            }
        }
    }
    let Some((conclusions, run, judged_afresh)) = judge_the_collected_images(
        scenario_name,
        &images,
        container_capacity_knobs,
        JudgedAfreshRegardlessOfTheExpectations::OnlyIfAnImageExpectsIt,
    ) else {
        return;
    };
    let mismatches = mismatches_of_the_images(&images, &conclusions, judged_afresh);
    assert!(
        mismatches.is_empty(),
        "场景 {scenario_name} 交出的镜像上池级 checker 判得与场景写下的对不上（引擎 {:?}）：\n{}",
        run.tally,
        mismatches.join("\n")
    );
}
