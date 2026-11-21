//! 随机历史这一批与五段取样的种子、步数与跑法；照环境变量取这一批与工作线程数，读不懂的变量值怎么报。

use super::*;

/// 默认批的种子区间：不设变量时每次跑同一批。
const DEFAULT_BATCH_FIRST_SEED: u64 = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;

const DEFAULT_BATCH_SEEDS: u64 = 96;

/// 第 121 行那一类的专门取样点：种子区间与每段步数，写死（判出率与窗口大小见那条用例的注释）。
pub(super) const REUSE_SAMPLING_FIRST_SEED: u64 = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;

pub(super) const REUSE_SAMPLING_SEEDS: u64 = 48;

const REUSE_SAMPLING_OPERATIONS_PER_HISTORY: usize = 30;

/// 理想模型 B2 那一格的专门取样点：种子区间与每段步数，写死（判出率见那条用例的注释）。
pub(super) const ROLLBACK_SAMPLING_FIRST_SEED: u64 = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;

pub(super) const ROLLBACK_SAMPLING_SEEDS: u64 = 48;

const ROLLBACK_SAMPLING_OPERATIONS_PER_HISTORY: usize = 30;

/// 原分配记录墙那一格的取样点：种子区间与每段步数，写死（判出率见那条用例的注释）。
pub(super) const WALL_SAMPLING_FIRST_SEED: u64 = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;

pub(super) const WALL_SAMPLING_SEEDS: u64 = 32;

const WALL_SAMPLING_OPERATIONS_PER_HISTORY: usize = 150;

/// 单元区墙那一格的取样点（两块小盘）：种子区间与每段步数，写死（判出率见那条用例的注释）。
pub(super) const UNIT_AREA_WALL_SAMPLING_FIRST_SEED: u64 = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;

pub(super) const UNIT_AREA_WALL_SAMPLING_SEEDS: u64 = 32;

const UNIT_AREA_WALL_SAMPLING_OPERATIONS_PER_HISTORY: usize = 150;

/// 默认批的每段步数、比重与跑法：与「收缩一个种子」那条入口（example）没设变量时取的默认值是同一份（`history` 里的定义）。
pub(super) const DEFAULT_BATCH_SAMPLING: HistorySampling = RANDOM_HISTORY_DEFAULT_SAMPLING;

pub(super) const REUSE_SAMPLING: HistorySampling = HistorySampling {
    operations_per_history: REUSE_SAMPLING_OPERATIONS_PER_HISTORY,
    weights: GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
    execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
};

pub(super) const ROLLBACK_SAMPLING: HistorySampling = HistorySampling {
    operations_per_history: ROLLBACK_SAMPLING_OPERATIONS_PER_HISTORY,
    weights: GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR,
    execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
};

pub(super) const WALL_SAMPLING: HistorySampling = HistorySampling {
    operations_per_history: WALL_SAMPLING_OPERATIONS_PER_HISTORY,
    weights: GenerationWeights::TOWARD_THE_ALLOCATION_RECORD_WALL,
    execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
};

/// 单元区墙那一格空间准入关掉的那一段：两块单元区 256 槽的小盘。
pub(super) const UNIT_AREA_WALL_SAMPLING_WITH_THE_SPACE_ADMISSION_SKIPPED: HistorySampling =
    HistorySampling {
        operations_per_history: UNIT_AREA_WALL_SAMPLING_OPERATIONS_PER_HISTORY,
        weights: GenerationWeights::TOWARD_THE_UNIT_AREA_WALL,
        execution: HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: HistoryDeviceWidth::UnitAreaOf256Slots,
            space_admission: SpaceAdmission::SkippedByTheTestOnlySwitch,
        },
    };

/// 单元区墙那一格空间准入判着的那一段（产品路径）：同样两块单元区 256 槽的小盘。
pub(super) const UNIT_AREA_WALL_SAMPLING_WITH_THE_SPACE_ADMISSION_JUDGED: HistorySampling =
    HistorySampling {
        operations_per_history: UNIT_AREA_WALL_SAMPLING_OPERATIONS_PER_HISTORY,
        weights: GenerationWeights::TOWARD_THE_UNIT_AREA_WALL,
        execution: HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: HistoryDeviceWidth::UnitAreaOf256Slots,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        },
    };

/// 随机历史这一批跑什么：种子区间、取样参数（每段步数、比重、跑法）与新发现收不收缩。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RandomHistoryBatch {
    pub(super) first_seed: u64,
    pub(super) seed_count: u64,
    pub(super) sampling: HistorySampling,
    pub(super) shrinking: FindingShrinking,
}

/// 不设任何变量时跑的那一批：这个测试周期的种子基起 96 段、每段 30 步、broad 比重、只报种子不收缩。
pub(super) const RANDOM_HISTORY_DEFAULT_BATCH: RandomHistoryBatch = RandomHistoryBatch {
    first_seed: DEFAULT_BATCH_FIRST_SEED,
    seed_count: DEFAULT_BATCH_SEEDS,
    sampling: DEFAULT_BATCH_SAMPLING,
    shrinking: FindingShrinking::ReportSeedsOnly,
};

/// 收不收缩的变量（`every` 每一类收缩第一个种子、`none` 只报种子）。
const RANDOM_HISTORY_SHRINK_VARIABLE: &str = "SINGLEFS_RANDOM_HISTORY_SHRINK";

/// 照 `lookup` 读这一批：种子起点（`SINGLEFS_RANDOM_HISTORY_FIRST_SEED`）、种子数（`_SEEDS`）、收不收缩（`_SHRINK`），
/// 每段步数、比重、跑法经 `HistorySampling::from_environment` 读（`_OPERATIONS`、`_WEIGHTS`、`_CHECKER`、`_DEVICES`、`_SPACE_ADMISSION`，
/// 与判红时打的复现行同一张表）；哪一个没设就取 [`RANDOM_HISTORY_DEFAULT_BATCH`] 的那一项，一个都没设就是默认批。
///
/// # Panics
/// 某个变量设了而认不得（数值不是装得进的非负整数、种子数是 0、收缩不是 `every` 或 `none`、取样参数认不得）。
pub(super) fn random_history_batch_from(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> RandomHistoryBatch {
    let first_seed_variable = "SINGLEFS_RANDOM_HISTORY_FIRST_SEED";
    let seed_count_variable = "SINGLEFS_RANDOM_HISTORY_SEEDS";
    let first_seed = number_from_the_value_or_default(
        first_seed_variable,
        lookup(first_seed_variable).as_deref(),
        RANDOM_HISTORY_DEFAULT_BATCH.first_seed,
    );
    let seed_count = count_from_the_value_or_default(
        seed_count_variable,
        lookup(seed_count_variable).as_deref(),
        RANDOM_HISTORY_DEFAULT_BATCH.seed_count,
    );
    let sampling = HistorySampling::from_environment(
        RANDOM_HISTORY_VARIABLE_PREFIX,
        lookup,
        &RANDOM_HISTORY_DEFAULT_BATCH.sampling,
    )
    .unwrap_or_else(|error| panic!("{}", error.render()));
    let shrinking = match lookup(RANDOM_HISTORY_SHRINK_VARIABLE).as_deref() {
        None => RANDOM_HISTORY_DEFAULT_BATCH.shrinking,
        Some("every") => FindingShrinking::EveryFindingClass,
        Some("none") => FindingShrinking::ReportSeedsOnly,
        Some(other) => panic!("{RANDOM_HISTORY_SHRINK_VARIABLE}={other}：只认 every 与 none"),
    };
    RandomHistoryBatch {
        first_seed,
        seed_count,
        sampling,
        shrinking,
    }
}

/// 一个变量都没设读回默认批；设了的每一项照变量读回、没设的那几项仍取默认批的（设了变量也读回默认批时这一条红）。
#[test]
fn the_random_history_batch_is_the_default_batch_without_variables_and_follows_the_variables_that_are_set(
) {
    assert_eq!(
        random_history_batch_from(&|_variable: &str| None),
        RANDOM_HISTORY_DEFAULT_BATCH,
        "一个 SINGLEFS_RANDOM_HISTORY_* 都没设，跑的该是默认批"
    );
    let given = |variable: &str| {
        match variable {
            "SINGLEFS_RANDOM_HISTORY_FIRST_SEED" => Some("11"),
            "SINGLEFS_RANDOM_HISTORY_SEEDS" => Some("1000"),
            "SINGLEFS_RANDOM_HISTORY_OPERATIONS" => Some("60"),
            "SINGLEFS_RANDOM_HISTORY_WEIGHTS" => Some("reuse"),
            "SINGLEFS_RANDOM_HISTORY_SHRINK" => Some("every"),
            _ => None,
        }
        .map(str::to_string)
    };
    assert_eq!(
        random_history_batch_from(&given),
        RandomHistoryBatch {
            first_seed: 11,
            seed_count: 1000,
            sampling: HistorySampling {
                operations_per_history: 60,
                weights: GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
                execution: RANDOM_HISTORY_DEFAULT_BATCH.sampling.execution,
            },
            shrinking: FindingShrinking::EveryFindingClass,
        },
        "设了的种子起点、种子数、步数、比重、收缩照变量读回，没设的跑法取默认批的"
    );
}

/// 五段的种子基都是这个测试周期的常量：两个二进制（随机历史与崩溃注入）同基，
/// `crates/mutations.tsv` 里名字以「增补 3 第 2 件」起头的那些「这条变异必须红」与里程碑的验收才是可判的命题——
/// 哪一段被改回写死的别的数，那一段跑的就是另一批历史，那些判红不再是同一个命题。
/// 常量本身是不是这个周期抽出来的那个数，由崩溃注入那个二进制的
/// `the_test_cycle_seed_base_is_the_number_drawn_for_this_cycle` 钉。
#[test]
fn the_five_samplings_start_from_the_test_cycle_seed_base() {
    for (sampling_name, first_seed) in [
        ("随机历史的默认批", DEFAULT_BATCH_FIRST_SEED),
        ("偏向抬 F 之后复用的取样点", REUSE_SAMPLING_FIRST_SEED),
        ("偏向抬 F 之后回退的取样点", ROLLBACK_SAMPLING_FIRST_SEED),
        ("越过原分配记录墙的取样点", WALL_SAMPLING_FIRST_SEED),
        (
            "小盘上逼近单元区墙的取样点",
            UNIT_AREA_WALL_SAMPLING_FIRST_SEED,
        ),
    ] {
        assert_eq!(
            first_seed, SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
            "{sampling_name}的种子基是 {first_seed}，不是这个测试周期的种子基：这一段跑的是另一批历史，变异表与验收在它上面量过的判红都不算数"
        );
    }
}

/// 数值变量的值（`None` 是没设）：没设取 `default`；设了而不是非负整数就 panic、消息带变量名与原值
/// （经数值环境变量共用的那一份解析，写错的规模不悄悄退回默认）。
///
/// # Panics
/// 设了而不是非负整数。
fn number_from_the_value_or_default(name: &str, value: Option<&str>, default: u64) -> u64 {
    number_from_the_variable_value_panicking_when_unreadable::<u64>(name, value).unwrap_or(default)
}

/// 「跑几个」的计数变量的值（`None` 是没设，种子数这一类）：没设取 `default`；设了而不是正整数（0 也算）就 panic、消息带变量名与原值。
/// 0 段的一批没有「已知红」清单外的失败，只是因为一段都没跑，照样判成绿（上游 `show-me-test.md`「扫到 0 项也不是通过」）。
///
/// # Panics
/// 设了而不是正整数。
fn count_from_the_value_or_default(name: &str, value: Option<&str>, default: u64) -> u64 {
    number_from_the_variable_value_panicking_when_unreadable::<std::num::NonZeroU64>(name, value)
        .map_or(default, std::num::NonZeroU64::get)
}

const WORKER_THREADS_ENVIRONMENT_VARIABLE: &str = RANDOM_HISTORY_WORKER_THREADS_VARIABLE;

/// 线程数变量的值（`None` 是没设）：设了照它，没设取每台默认重活线程数（`thread_cap_value` 是 `SINGLEFS_THREAD_CAP` 的值，
/// 设了照它，再没设取 `.claude/heavy-thread-default` 的登记值，判法在 `singlefs_verification::heavy_threads`）；默认批、设了变量的那一批与各取样点同一个取法。
/// 设了而不是正整数（0 也算）就 panic、消息带变量名与原值，与仓里别的线程数变量同一口径（经数值环境变量共用的那一份解析，取 `NonZeroUsize`）。
///
/// # Panics
/// 设了而不是正整数；落到默认时 `SINGLEFS_THREAD_CAP` 设了而不是正整数。
fn worker_threads_from_the_value(value: Option<&str>, thread_cap_value: Option<&str>) -> usize {
    own_variable_or_default_heavy_threads(
        WORKER_THREADS_ENVIRONMENT_VARIABLE,
        value,
        thread_cap_value,
    )
    .get()
}

/// 进程环境里的线程数变量与 `SINGLEFS_THREAD_CAP`，判法同 [`worker_threads_from_the_value`]。
///
/// # Panics
/// 设了而不是正整数，或值不是 UTF-8。
pub(super) fn worker_threads_from_environment() -> usize {
    worker_threads_from_the_value(
        process_environment_variable(WORKER_THREADS_ENVIRONMENT_VARIABLE).as_deref(),
        process_environment_variable(THREAD_CAP_ENVIRONMENT_VARIABLE).as_deref(),
    )
}

/// 线程数变量没设：取登记的每台默认重活线程数（不取本机核数，也不再夹到 16）；设了照它。
#[test]
fn random_history_worker_threads_take_the_registered_heavy_thread_default_when_unset() {
    assert_eq!(
        worker_threads_from_the_value(None, None),
        registered_heavy_threads().get(),
        "SINGLEFS_RANDOM_HISTORY_THREADS 与 SINGLEFS_THREAD_CAP 都没设取 .claude/heavy-thread-default 的登记值"
    );
    assert_eq!(
        worker_threads_from_the_value(None, Some("5")),
        5,
        "没设自己的变量读 SINGLEFS_THREAD_CAP"
    );
    assert_eq!(
        worker_threads_from_the_value(Some("3"), Some("5")),
        3,
        "自己的变量先读"
    );
}

#[test]
#[should_panic(expected = "SINGLEFS_RANDOM_HISTORY_SEEDS=\"ten\" 认不出")]
fn an_unreadable_random_history_seed_count_panics_naming_the_variable_and_the_value() {
    let _seed_count =
        count_from_the_value_or_default("SINGLEFS_RANDOM_HISTORY_SEEDS", Some("ten"), 1000);
}

#[test]
#[should_panic(expected = "SINGLEFS_RANDOM_HISTORY_FIRST_SEED=\"ten\" 认不出")]
fn an_unreadable_random_history_first_seed_panics_naming_the_variable_and_the_value() {
    let _first_seed =
        number_from_the_value_or_default("SINGLEFS_RANDOM_HISTORY_FIRST_SEED", Some("ten"), 7);
}

#[test]
#[should_panic(expected = "SINGLEFS_RANDOM_HISTORY_SEEDS=\"0\" 认不出")]
fn a_random_history_seed_count_of_zero_panics_naming_the_variable_and_the_value() {
    let _seed_count =
        count_from_the_value_or_default("SINGLEFS_RANDOM_HISTORY_SEEDS", Some("0"), 1000);
}

#[test]
#[should_panic(expected = "SINGLEFS_RANDOM_HISTORY_THREADS=\"0\" 认不出")]
fn a_random_history_worker_thread_count_of_zero_panics_naming_the_variable_and_the_value() {
    let _threads = worker_threads_from_the_value(Some("0"), None);
}
