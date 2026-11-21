#![cfg_attr(
    not(feature = "gpu"),
    allow(
        dead_code,
        reason = "第 3 次重跑的两段 GPU 子命令（gpu 特性）才用；没编 gpu 特性时只在单测里用"
    )
)]

use super::*;

mod recompute_disagreement_arithmetic;
mod submission_cells_and_point_lines;

#[cfg_attr(
    not(any(feature = "gpu", test)),
    allow(
        unused_imports,
        reason = "第 3 次重跑第二段（gpu 特性）与单测才从这一份之外用它；两样都没有时没有人用"
    )
)]
pub(super) use recompute_disagreement_arithmetic::*;
pub(super) use submission_cells_and_point_lines::*;

/// 一次提交的单元数 n 的八档（每单元 32768 字节：128 KiB 到 16 MiB；登记第一节「提交大小 n」）。
pub(super) const SUBMISSION_UNIT_COUNTS: [u64; 8] = [4, 8, 12, 16, 20, 24, 32, 512];
/// 最大那一档（常驻缓冲按它开，登记第一节「常驻写法」）。
pub(super) const LARGEST_SUBMISSION_UNITS: u64 = 512;

/// 岔路单第 1 行的四个候选（登记第一节「候选门槛」）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum GpuCandidate {
    /// A：只放扫描里的校验和。
    ScanChecksumOnly,
    /// B：A 加选段打分。
    ScanChecksumAndChoice,
    /// C：甲、乙、丙全放（g1 的门槛）。
    EveryStage,
    /// D：只换整理发布里释放前读盘核那一段。
    ReleaseCheckOnly,
}

impl GpuCandidate {
    pub(super) const ALL: [GpuCandidate; 4] = [
        GpuCandidate::ScanChecksumOnly,
        GpuCandidate::ScanChecksumAndChoice,
        GpuCandidate::EveryStage,
        GpuCandidate::ReleaseCheckOnly,
    ];

    pub(super) const fn label(self) -> &'static str {
        match self {
            GpuCandidate::ScanChecksumOnly => "scan_checksum_only",
            GpuCandidate::ScanChecksumAndChoice => "scan_checksum_and_choice",
            GpuCandidate::EveryStage => "every_stage",
            GpuCandidate::ReleaseCheckOnly => "release_check_only",
        }
    }
}

/// 一个点（登记第一节「12 个点」）：(f, P, K)，与 `out/g1.out` 的 12 行一一对应。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct SubmissionPoint {
    pub(super) fraction: LiveSlotFraction,
    pub(super) pool_data_units: u64,
    pub(super) batch_units: u64,
}

impl SubmissionPoint {
    pub(super) fn label(self) -> String {
        format!(
            "fraction={} pool_data_units={} batch_units={}",
            self.fraction.label(),
            self.pool_data_units,
            self.batch_units
        )
    }
}

/// 12 个点，按 (f, P, K) 升序（同 `g1_cpu_saved_ledger.py` 打行的次序）。
pub(super) fn every_submission_point() -> Vec<SubmissionPoint> {
    every_sampling_point()
        .into_iter()
        .flat_map(|point| {
            BATCH_SIZES
                .into_iter()
                .map(move |batch_units| SubmissionPoint {
                    fraction: point.fraction,
                    pool_data_units: point.pool_data_units,
                    batch_units,
                })
        })
        .collect()
}

/// 一条意图里各次提交交的单元数（登记第一节「一个点的提交序列」）。A、B、C：签发一次加每批各一次，每次交那一刻 R 里还剩的单元
/// （每批之前重扫整个 R）；D：每批一次，交这一批搬的单元两块盘各一份。与 P 无关。
pub(super) fn submission_units_of_the_intent(
    candidate: GpuCandidate,
    fraction: LiveSlotFraction,
    batch_units: u64,
) -> Vec<u64> {
    let data_units = fraction.data_units_per_segment();
    let mut units_left = data_units;
    match candidate {
        GpuCandidate::ScanChecksumOnly
        | GpuCandidate::ScanChecksumAndChoice
        | GpuCandidate::EveryStage => {
            let mut submissions = vec![data_units];
            // 迭代上界：每圈 units_left 减 min(K, units_left) ≥ 1，⌈m ÷ K⌉ 圈出去
            while units_left > 0 {
                submissions.push(units_left);
                units_left -= units_left.min(batch_units);
            }
            submissions
        }
        GpuCandidate::ReleaseCheckOnly => {
            let mut submissions = Vec::new();
            // 迭代上界同上
            while units_left > 0 {
                let relocated_in_this_batch = units_left.min(batch_units);
                submissions.push(DEVICES_HOLDING_A_COPY * relocated_in_this_batch);
                units_left -= relocated_in_this_batch;
            }
            submissions
        }
    }
}

/// 乙-1（意图内一次交）那一次交的单元数：签发那一次交的 m（登记第五节「第 3 行的臂」乙-1，`c_rt(m)` 直接取 n = m 那一档）。
pub(super) fn units_of_one_submission_per_intent(submission_units: &[u64]) -> u64 {
    submission_units
        .first()
        .copied()
        .expect("A、B、C 的提交序列头一次是签发，至少一次")
}

/// 第七节乙类：每个点 A、B、C 的提交次数、平均每次提交的字节（四舍五入到 0.1 字节写成整十分之一）、D 的提交次数。键 (f, K)。
pub(super) const SUBMISSION_SEQUENCE_ANCHORS: [(LiveSlotFraction, u64, u64, u64, u64); 6] = [
    (LiveSlotFraction::OneQuarter, 4, 3, 2_184_533, 2),
    (LiveSlotFraction::OneQuarter, 8, 2, 2_621_440, 1),
    (LiveSlotFraction::OneQuarter, 16, 2, 2_621_440, 1),
    (LiveSlotFraction::ThreeQuarters, 4, 7, 5_055_634, 6),
    (LiveSlotFraction::ThreeQuarters, 8, 4, 5_898_240, 3),
    (LiveSlotFraction::ThreeQuarters, 16, 3, 6_116_693, 2),
];

/// 平均每次提交的字节 × 10，四舍五入到整数（第七节乙类锚点 218453.3 写成 2184533）。
pub(super) fn mean_submission_tenths_of_bytes(submission_units: &[u64]) -> u64 {
    let total_bytes: u64 = submission_units
        .iter()
        .map(|units| units * LOCAL_GPU_SLOT_BYTES)
        .sum();
    let submissions = u64::try_from(submission_units.len()).expect("提交次数装得进 u64");
    (total_bytes * 10 * 2 + submissions).div_euclid(2 * submissions)
}

/// 一个点的 c_rt（登记第一节「一个点的 c_rt」）：那个点的提交序列里每次提交取它那一档 n 的值，取平均（一条意图的总 CPU ÷ 提交次数，
/// 与 g1 的门槛同一个分母）。有一档没有读数交回 `None`。
pub(super) fn point_milliseconds_of_the_sequence(
    submission_units: &[u64],
    milliseconds_by_units: &BTreeMap<u64, f64>,
) -> Option<f64> {
    let per_submission: Vec<f64> = submission_units
        .iter()
        .map(|units| milliseconds_by_units.get(units).copied())
        .collect::<Option<Vec<f64>>>()?;
    let count = per_submission.len() as f64;
    Some(per_submission.iter().sum::<f64>() / count)
}

/// R1、R2、R3 的判定（登记第六节）：c_rt < 门槛判不赔，≥ 判赔；B 的不赔写成下界（丙的输入是分配记录，展平它的 CPU 没量）。
pub(super) fn judgement_against_the_threshold(
    host_cpu_milliseconds: f64,
    threshold_milliseconds: f64,
    candidate: GpuCandidate,
) -> &'static str {
    if host_cpu_milliseconds < threshold_milliseconds {
        match candidate {
            GpuCandidate::ScanChecksumAndChoice => "not_losing_lower_bound",
            GpuCandidate::ScanChecksumOnly
            | GpuCandidate::EveryStage
            | GpuCandidate::ReleaseCheckOnly => "not_losing",
        }
    } else {
        "losing"
    }
}

/// 5 轮各判一次之后的判定（登记第六节）：有一轮没有读数（三次都作废，第十条 J9）写「轮数不足」，不下判定、不当阴性；5 轮都有读数时
/// 全相同写那个判定，不全相同写 `unstable`。
pub(super) fn judgement_of_the_valid_rounds(
    judgements_per_round: &[Option<&'static str>],
) -> &'static str {
    if judgements_per_round.len() < ROUNDS || judgements_per_round.iter().any(Option::is_none) {
        return "rounds_insufficient";
    }
    let judged: Vec<&'static str> = judgements_per_round.iter().flatten().copied().collect();
    judgement_of_the_rounds(&judged)
}

/// 两份输入文件的 sha256（登记第十一节停机 J-S4）：`out/g1.out` 与 r2 第二段产物。
pub(super) const ATTACK_LEDGER_SHA256: &str =
    "61b210296634f2f20fa0465a4e4f09859cd9236522c54d4fb18301e75f243025";
pub(super) const STAGE_SPLIT_PRODUCT_SHA256: &str =
    "f70eb21b8a7081489dd8fc3209080f3ab83f16ac40beee3042c03f4756f1e07a";

/// 停机 J-S4 的核法：两份输入各算 sha256，与登记的比；交回（项、算出的、登记的、对不对得上）。
pub(super) fn input_file_checks(
    attack_ledger_bytes: &[u8],
    stage_split_bytes: &[u8],
) -> Vec<(&'static str, String, &'static str, bool)> {
    [
        ("attack_ledger", attack_ledger_bytes, ATTACK_LEDGER_SHA256),
        (
            "stage_split_product",
            stage_split_bytes,
            STAGE_SPLIT_PRODUCT_SHA256,
        ),
    ]
    .into_iter()
    .map(|(item, bytes, registered)| {
        let digest = hexadecimal_of(&sha256_digest(bytes));
        let holds = digest == registered;
        (item, digest, registered, holds)
    })
    .collect()
}

/// r2 第二段产物里一条 `name=stage` 行要的几个数（`g1_cpu_saved_ledger.py` 第 13–37 行读的那几样）。
#[derive(Clone, Debug, PartialEq)]
pub(super) struct StageLedgerRow {
    pub(super) intent_key: LedgerIntentKey,
    pub(super) is_the_issue: bool,
    pub(super) call_cpu_nanoseconds: f64,
    pub(super) crc_cpu_nanoseconds: f64,
    pub(super) scan_increment_cpu_nanoseconds: f64,
    pub(super) choose_cpu_nanoseconds: Option<f64>,
    pub(super) release_check_crc_cpu_nanoseconds: f64,
}

/// 照 `g1_cpu_saved_ledger.py` 第 13 行读：只收 `name=stage` 的行，不滤 `valid`；键 (f, P, K, 轮)。缺数的行交回 `None`（整份读不成）。
pub(super) fn stage_ledger_rows_of(
    lines: &[BTreeMap<String, String>],
) -> Option<Vec<StageLedgerRow>> {
    lines
        .iter()
        .filter(|fields| fields.get("name").map(String::as_str) == Some("stage"))
        .map(|fields| {
            let number = |key: &str| fields.get(key)?.parse::<f64>().ok();
            Some(StageLedgerRow {
                intent_key: (
                    fields.get("fraction")?.clone(),
                    fields.get("pool_data_units")?.clone(),
                    fields.get("batch_units")?.clone(),
                    fields.get("round")?.clone(),
                ),
                is_the_issue: fields.get("kind")? == "issue",
                call_cpu_nanoseconds: number("call_cpu_nanoseconds")?,
                crc_cpu_nanoseconds: number("crc_cpu_nanoseconds")?,
                scan_increment_cpu_nanoseconds: number("scan_increment_cpu_nanoseconds")?,
                choose_cpu_nanoseconds: number("choose_cpu_nanoseconds"),
                release_check_crc_cpu_nanoseconds: number("release_check_crc_cpu_nanoseconds")?,
            })
        })
        .collect()
}

/// 一个候选在一个点上的门槛（登记第一节「候选门槛」）：一条意图省下的毫秒（各轮中位）、提交次数、门槛 = 省下 ÷ 提交次数。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct CandidateThreshold {
    pub(super) saved_per_intent_milliseconds: f64,
    pub(super) submissions_per_intent: u64,
    pub(super) threshold_milliseconds: f64,
}

/// 一条意图一个候选省下的纳秒与提交次数（照 `g1_cpu_saved_ledger.py` 第 30–36 行的式子与加法次序；A、B、D 是换候选的同一个算法）。
pub(super) fn saved_nanoseconds_of_the_intent(
    candidate: GpuCandidate,
    issue: &StageLedgerRow,
    batches: &[&StageLedgerRow],
) -> Option<(f64, u64)> {
    let batch_count = u64::try_from(batches.len()).expect("批数");
    let sum_over_batches = |value_of: fn(&StageLedgerRow) -> f64| {
        batches
            .iter()
            .fold(0.0_f64, |total, batch| total + value_of(batch))
    };
    Some(match candidate {
        GpuCandidate::ScanChecksumOnly => (
            sum_over_batches(|batch| batch.crc_cpu_nanoseconds) + issue.crc_cpu_nanoseconds,
            batch_count + 1,
        ),
        GpuCandidate::ScanChecksumAndChoice => (
            sum_over_batches(|batch| batch.crc_cpu_nanoseconds)
                + issue.crc_cpu_nanoseconds
                + issue.choose_cpu_nanoseconds?,
            batch_count + 1,
        ),
        GpuCandidate::EveryStage => (
            sum_over_batches(|batch| {
                batch.crc_cpu_nanoseconds + batch.scan_increment_cpu_nanoseconds
            }) + (issue.crc_cpu_nanoseconds
                + issue.scan_increment_cpu_nanoseconds
                + issue.choose_cpu_nanoseconds?),
            batch_count + 1,
        ),
        GpuCandidate::ReleaseCheckOnly => (
            sum_over_batches(|batch| batch.release_check_crc_cpu_nanoseconds),
            batch_count,
        ),
    })
}

/// 门槛账的键：一条意图 (f, P, K, 轮) 与一个点 (f, P, K)，都按产物里的文字。
pub(super) type LedgerIntentKey = (String, String, String, String);
pub(super) type LedgerPointKey = (String, String, String);
/// 每个点四个候选的门槛。
pub(super) type CandidateThresholdsByPoint =
    BTreeMap<LedgerPointKey, BTreeMap<GpuCandidate, CandidateThreshold>>;

/// 照 `g1_cpu_saved_ledger.py` 的取法算四个候选在每个点上的门槛：按 (f, P, K, 轮) 分组（`kind=issue` 那一行算签发、后来的签发行盖掉先来的，
/// 其余算批），每个点各轮省下的取中位 ÷ 1e6 得毫秒，提交次数取轮号最小那一轮的。键 (f, P, K)。读不成交回 `None`。
pub(super) fn candidate_thresholds_of_the_ledger(
    rows: &[StageLedgerRow],
) -> Option<CandidateThresholdsByPoint> {
    let mut intents: BTreeMap<LedgerIntentKey, (Option<&StageLedgerRow>, Vec<&StageLedgerRow>)> =
        BTreeMap::new();
    for row in rows {
        let intent = intents.entry(row.intent_key.clone()).or_default();
        if row.is_the_issue {
            intent.0 = Some(row);
        } else {
            intent.1.push(row);
        }
    }
    let mut per_point: BTreeMap<LedgerPointKey, BTreeMap<GpuCandidate, Vec<(f64, u64)>>> =
        BTreeMap::new();
    for ((fraction, pool, batch, _round), (issue, batches)) in &intents {
        let issue = (*issue)?;
        for candidate in GpuCandidate::ALL {
            per_point
                .entry((fraction.clone(), pool.clone(), batch.clone()))
                .or_default()
                .entry(candidate)
                .or_default()
                .push(saved_nanoseconds_of_the_intent(candidate, issue, batches)?);
        }
    }
    per_point
        .into_iter()
        .map(|(point_key, by_candidate)| {
            let thresholds = by_candidate
                .into_iter()
                .map(|(candidate, intents_of_the_point)| {
                    let saved: Vec<f64> = intents_of_the_point
                        .iter()
                        .map(|(saved, _)| *saved)
                        .collect();
                    let saved_per_intent_milliseconds = median_of(&saved)? / 1e6;
                    let submissions_per_intent = intents_of_the_point.first()?.1;
                    Some((
                        candidate,
                        CandidateThreshold {
                            saved_per_intent_milliseconds,
                            submissions_per_intent,
                            threshold_milliseconds: saved_per_intent_milliseconds
                                / submissions_per_intent as f64,
                        },
                    ))
                })
                .collect::<Option<BTreeMap<_, _>>>()?;
            Some((point_key, thresholds))
        })
        .collect()
}

/// `out/g1.out` 的一行 `name=g1_ledger`：门槛原样的文字（小数第 4 位，J-S3 逐字比）与它的数、省下的上界、提交次数。
#[derive(Clone, Debug, PartialEq)]
pub(super) struct AttackLedgerPoint {
    pub(super) break_even_text: String,
    pub(super) break_even_milliseconds: f64,
    pub(super) saved_upper_milliseconds: f64,
    pub(super) submissions_per_intent: u64,
}

/// 读 `out/g1.out`（行首没有 `E7RESULT`）：键 (f, P, K) 的文字。缺数的行交回 `None`。
pub(super) fn attack_ledger_points_of(
    text: &str,
) -> Option<BTreeMap<(String, String, String), AttackLedgerPoint>> {
    text.lines()
        .filter(|line| line.starts_with("name=g1_ledger "))
        .map(|line| {
            let fields: BTreeMap<&str, &str> = line
                .split(' ')
                .filter_map(|field| field.split_once('='))
                .collect();
            let break_even_text =
                (*fields.get("break_even_host_cpu_per_submission_ms")?).to_string();
            Some((
                (
                    (*fields.get("f")?).to_string(),
                    (*fields.get("P")?).to_string(),
                    (*fields.get("K")?).to_string(),
                ),
                AttackLedgerPoint {
                    break_even_milliseconds: break_even_text.parse().ok()?,
                    break_even_text,
                    saved_upper_milliseconds: fields.get("saved_upper_ms")?.parse().ok()?,
                    submissions_per_intent: fields.get("submissions_per_intent")?.parse().ok()?,
                },
            ))
        })
        .collect()
}

/// 停机 J-S3：装置算的候选 C 门槛到小数第 4 位与 `g1.out` 不同，或提交次数与 `g1.out`、与第七节乙类锚点不同，交回一句；都对上交 `None`。
pub(super) fn ledger_finding_of_the_point(
    point: SubmissionPoint,
    computed_every_stage: &CandidateThreshold,
    attack_ledger: &AttackLedgerPoint,
) -> Option<String> {
    let computed_text = format!("{:.4}", computed_every_stage.threshold_milliseconds);
    let sequence_submissions = u64::try_from(
        submission_units_of_the_intent(GpuCandidate::EveryStage, point.fraction, point.batch_units)
            .len(),
    )
    .expect("提交次数");
    if computed_text != attack_ledger.break_even_text {
        return Some(format!(
            "clause_j_s3_threshold_{computed_text}_differs_from_the_attack_ledger_{}",
            attack_ledger.break_even_text
        ));
    }
    if computed_every_stage.submissions_per_intent != attack_ledger.submissions_per_intent
        || sequence_submissions != attack_ledger.submissions_per_intent
    {
        return Some(format!(
        "clause_j_s3_submissions_{}_or_sequence_{sequence_submissions}_differ_from_the_attack_ledger_{}",
        computed_every_stage.submissions_per_intent, attack_ledger.submissions_per_intent
    ));
    }
    None
}

/// ρ_机器（登记第一节「机器换算」）：本机 16 MiB 校准基线的中位 ÷ 第二台（r2 第二段产物 `position=start` 那一行）的中位。
/// 门槛 × ρ 是「换到本机」的门槛：本机算同一份活更快（ρ < 1），省下的 CPU 也少。
pub(super) fn machine_ratio(
    local_baseline_nanoseconds: f64,
    peer_baseline_nanoseconds: f64,
) -> f64 {
    local_baseline_nanoseconds / peer_baseline_nanoseconds
}

/// r2 第二段产物里 `name=calibration_baseline segment=stage_split position=start` 那一行的中位；取不到交回 `None`（机器换算那几行写 `not_run`）。
pub(super) fn peer_calibration_baseline_of(lines: &[BTreeMap<String, String>]) -> Option<f64> {
    lines
        .iter()
        .find(|fields| {
            fields.get("name").map(String::as_str) == Some("calibration_baseline")
                && fields.get("segment").map(String::as_str) == Some("stage_split")
                && fields.get("position").map(String::as_str) == Some("start")
        })
        .and_then(|fields| fields.get("median_nanoseconds")?.parse().ok())
}

/// 第十条 J10 的两个触发：ρ 不在 [0.5, 2]；本机同一线程算同一份字节的 CPU ÷ T_A 与 ρ 差一倍以上。
pub(super) const MACHINE_RATIO_LOWER: f64 = 0.5;
pub(super) const MACHINE_RATIO_UPPER: f64 = 2.0;

pub(super) fn is_the_machine_ratio_far_from_one(ratio: f64) -> bool {
    !(MACHINE_RATIO_LOWER..=MACHINE_RATIO_UPPER).contains(&ratio)
}

pub(super) fn is_the_cross_check_off_by_more_than_double(
    local_crc_over_the_scan_checksum_threshold: f64,
    ratio: f64,
) -> bool {
    is_the_machine_ratio_far_from_one(local_crc_over_the_scan_checksum_threshold / ratio)
}

/// R4 的拟合不合（第十条 J6）：最大相对残差超过 20%。
pub(super) const FIT_RELATIVE_RESIDUAL_ABOVE_FOR_NOT_LINEAR: f64 = 0.20;
pub(super) const BYTES_PER_MEBIBYTE: f64 = 1_048_576.0;

/// R4：c_rt 拟合成「固定部分 + 每字节部分」（毫秒；每 MiB 的斜率另报一份），与最大相对残差。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct FixedAndPerByteFit {
    pub(super) fixed_milliseconds: f64,
    pub(super) per_byte_milliseconds: f64,
    pub(super) per_mebibyte_milliseconds: f64,
    pub(super) maximum_relative_residual: f64,
}

impl FixedAndPerByteFit {
    pub(super) fn is_linear(&self) -> bool {
        self.maximum_relative_residual <= FIT_RELATIVE_RESIDUAL_ABOVE_FOR_NOT_LINEAR
    }
}

/// 最小二乘：各档 n 的 c_rt（毫秒）对字节（n × 32768）。少于两档、或有一档读数不是正数交回 `None`。
pub(super) fn fixed_and_per_byte_fit(
    milliseconds_by_units: &BTreeMap<u64, f64>,
) -> Option<FixedAndPerByteFit> {
    let points: Vec<(f64, f64)> = milliseconds_by_units
        .iter()
        .map(|(units, milliseconds)| {
            let bytes = (units * LOCAL_GPU_SLOT_BYTES) as f64;
            (bytes, *milliseconds)
        })
        .collect();
    if points
        .iter()
        .any(|(_bytes, milliseconds)| *milliseconds <= 0.0)
    {
        return None;
    }
    let (fixed_milliseconds, per_byte_milliseconds) = least_squares_intercept_and_slope(&points)?;
    let maximum_relative_residual = points
        .iter()
        .map(|(bytes, milliseconds)| {
            ((fixed_milliseconds + per_byte_milliseconds * bytes - milliseconds) / milliseconds)
                .abs()
        })
        .fold(0.0_f64, f64::max);
    Some(FixedAndPerByteFit {
        fixed_milliseconds,
        per_byte_milliseconds,
        per_mebibyte_milliseconds: per_byte_milliseconds * BYTES_PER_MEBIBYTE,
        maximum_relative_residual,
    })
}

/// R5 一个点一轮的攒批类别（登记第六节 R5，按次序判）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum MergingShape {
    /// 甲：c_rt < T。
    OneSubmissionPerBatch,
    /// 乙-1：c_rt(m) < 省下的上界。
    OneSubmissionPerIntent,
    /// 乙-2 或丙：每字节那一部分 c1 × m × 32768 < 省下的上界，至少攒这么多条意图（W*）。
    MergeAcrossIntents { intents_to_merge_at_least: f64 },
    /// 每字节那一部分已经吃掉省下的。
    LosesHoweverMerged,
    /// 后两类要 R4 不是 `fit_not_linear`（第十条 J6）。
    FitNotLinear,
}

impl MergingShape {
    pub(super) const fn label(self) -> &'static str {
        match self {
            MergingShape::OneSubmissionPerBatch => "one_submission_per_batch",
            MergingShape::OneSubmissionPerIntent => "one_submission_per_intent",
            MergingShape::MergeAcrossIntents { .. } => "merge_across_intents",
            MergingShape::LosesHoweverMerged => "loses_however_merged",
            MergingShape::FitNotLinear => "fit_not_linear",
        }
    }
}

/// R5 一个点一轮：`point_milliseconds` 是这个点的 c_rt，`one_submission_per_intent_milliseconds` 是 n = m 那一档量到的 c_rt(m)，
/// `saved_upper_milliseconds` 是一条意图省下的上界；后两类用 R4 的拟合（`fit` 读不成时交回 `None`，那一轮写 not_run）。
pub(super) fn merging_shape(
    point_milliseconds: f64,
    threshold_milliseconds: f64,
    one_submission_per_intent_milliseconds: f64,
    saved_upper_milliseconds: f64,
    data_units: u64,
    fit: Option<FixedAndPerByteFit>,
    is_the_fit_linear: bool,
) -> Option<MergingShape> {
    if point_milliseconds < threshold_milliseconds {
        return Some(MergingShape::OneSubmissionPerBatch);
    }
    if one_submission_per_intent_milliseconds < saved_upper_milliseconds {
        return Some(MergingShape::OneSubmissionPerIntent);
    }
    if !is_the_fit_linear {
        return Some(MergingShape::FitNotLinear);
    }
    let fit = fit?;
    let per_byte_part_milliseconds =
        fit.per_byte_milliseconds * (data_units * LOCAL_GPU_SLOT_BYTES) as f64;
    if per_byte_part_milliseconds < saved_upper_milliseconds {
        Some(MergingShape::MergeAcrossIntents {
            intents_to_merge_at_least: (fit.fixed_milliseconds
                / (saved_upper_milliseconds - per_byte_part_milliseconds))
                .ceil(),
        })
    } else {
        Some(MergingShape::LosesHoweverMerged)
    }
}

/// 第八节轨迹：一格一轮的计时提交逐次的线程钟读数里，峰值、第 90 百分位、≥ 最小 / 最大门槛的次数、头一次与最后一次。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SubmissionTrajectory {
    pub(super) maximum_nanoseconds: f64,
    pub(super) percentile_90_nanoseconds: f64,
    pub(super) at_or_above_the_smallest_threshold: usize,
    pub(super) at_or_above_the_largest_threshold: usize,
    pub(super) first_nanoseconds: f64,
    pub(super) last_nanoseconds: f64,
}

pub(super) fn submission_trajectory(
    thread_cpu_nanoseconds_in_order: &[f64],
    smallest_threshold_milliseconds: f64,
    largest_threshold_milliseconds: f64,
) -> Option<SubmissionTrajectory> {
    let count_at_or_above = |milliseconds: f64| {
        thread_cpu_nanoseconds_in_order
            .iter()
            .filter(|nanoseconds| **nanoseconds >= milliseconds * 1e6)
            .count()
    };
    Some(SubmissionTrajectory {
        maximum_nanoseconds: thread_cpu_nanoseconds_in_order
            .iter()
            .copied()
            .reduce(f64::max)?,
        percentile_90_nanoseconds: percentile_of(thread_cpu_nanoseconds_in_order, 90)?,
        at_or_above_the_smallest_threshold: count_at_or_above(smallest_threshold_milliseconds),
        at_or_above_the_largest_threshold: count_at_or_above(largest_threshold_milliseconds),
        first_nanoseconds: *thread_cpu_nanoseconds_in_order.first()?,
        last_nanoseconds: *thread_cpu_nanoseconds_in_order.last()?,
    })
}

/// P1 插在计时区间开头（① 之前）的那一段（登记第五节阳性对照 P1）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum IntervalInsertion {
    Nothing,
    /// 甲：按线程钟自旋 2 ms。
    SpinTwoMilliseconds,
    /// 乙：睡 2 ms。
    SleepTwoMilliseconds,
}

/// P1 插的那 2 ms（纳秒）。
pub(super) const INSERTED_NANOSECONDS: u64 = 2_000_000;
/// P1 甲：c_rt 增量中位在 2 ms 的 [0.9, 1.1] 倍；乙：c_rt 增量中位 ≤ 2 ms × 0.1、挂钟增量中位 ≥ 2 ms × 0.9。
pub(super) const SPIN_CONTROL_LOWER: f64 = 0.9;
pub(super) const SPIN_CONTROL_UPPER: f64 = 1.1;
pub(super) const SLEEP_CONTROL_CPU_AT_MOST: f64 = 0.1;
pub(super) const SLEEP_CONTROL_WALL_AT_LEAST: f64 = 0.9;

pub(super) fn spin_control_holds(cpu_increment_nanoseconds: f64) -> bool {
    let inserted = INSERTED_NANOSECONDS as f64;
    (SPIN_CONTROL_LOWER * inserted..=SPIN_CONTROL_UPPER * inserted)
        .contains(&cpu_increment_nanoseconds)
}

pub(super) fn sleep_control_holds(
    cpu_increment_nanoseconds: f64,
    wall_increment_nanoseconds: f64,
) -> bool {
    let inserted = INSERTED_NANOSECONDS as f64;
    cpu_increment_nanoseconds <= SLEEP_CONTROL_CPU_AT_MOST * inserted
        && wall_increment_nanoseconds >= SLEEP_CONTROL_WALL_AT_LEAST * inserted
}

/// P1 每轮的增量（插了的中位 − 基线的中位），5 轮再取中位。有一轮读不到交回 `None`。
pub(super) fn median_increment_of_the_rounds(
    baseline_medians: &[Option<f64>],
    inserted_medians: &[Option<f64>],
) -> Option<f64> {
    let increments: Vec<f64> = baseline_medians
        .iter()
        .zip(inserted_medians)
        .map(|(baseline, inserted)| Some((*inserted)? - (*baseline)?))
        .collect::<Option<Vec<f64>>>()?;
    median_of(&increments)
}

/// P3：比对报恰好 1 条「不确定」、是第 3 条（下标 2）。
pub(super) const READBACK_ENTRY_FLIPPED_BY_THE_COMPARISON_CONTROL: usize = 2;

pub(super) fn comparison_control_holds(uncertain_entries: &[usize]) -> bool {
    uncertain_entries == [READBACK_ENTRY_FLIPPED_BY_THE_COMPARISON_CONTROL]
}

/// 第十条 J5：进程钟比线程钟多出线程钟的 10% 以上（驱动与 wgpu 在本进程里起的线程的 CPU 没算进 c_rt）。
pub(super) const PROCESS_CLOCK_EXCESS_ABOVE_FOR_UNDERESTIMATE: f64 = 0.10;

pub(super) fn is_the_thread_clock_underestimating(
    process_cpu_median_nanoseconds: f64,
    thread_cpu_median_nanoseconds: f64,
) -> bool {
    process_cpu_median_nanoseconds - thread_cpu_median_nanoseconds
        > thread_cpu_median_nanoseconds * PROCESS_CLOCK_EXCESS_ABOVE_FOR_UNDERESTIMATE
}

/// 停机 J-S5：一次提交的线程钟增量大过包着它的挂钟增量（计时口径错）。
pub(super) fn is_the_thread_cpu_inside_the_wall(
    thread_cpu_nanoseconds: u64,
    wall_nanoseconds: u64,
) -> bool {
    thread_cpu_nanoseconds <= wall_nanoseconds
}

/// 停机 J-S1：核一与 crates 的 `GpuCard::crc32c_of_ranges` 对同一份输入有一条不同（或条数不同），交回一句；逐条相同交 `None`。
pub(super) fn kernel_one_finding_against_the_crates(
    units: u64,
    kernel_one_results: &[u32],
    crates_results: &[u32],
) -> Option<String> {
    let differing = kernel_one_results
        .iter()
        .zip(crates_results)
        .filter(|(kernel_one, crates)| kernel_one != crates)
        .count();
    (differing > 0 || kernel_one_results.len() != crates_results.len()).then(|| {
    format!(
        "clause_j_s1_kernel_one_differs_from_the_crates_at_{differing}_of_{units}_units_lengths_{}_{}",
        kernel_one_results.len(),
        crates_results.len()
    )
})
}

/// `nvidia-smi --query-gpu=index,pci.bus_id --format=csv,noheader` 的输出：序号 → 总线号（小写）。
pub(super) fn bus_identifiers_by_index_of(query_text: &str) -> BTreeMap<u32, String> {
    query_text
        .lines()
        .filter_map(|line| {
            let (index, bus) = line.split_once(',')?;
            Some((index.trim().parse().ok()?, bus.trim().to_ascii_lowercase()))
        })
        .collect()
}

/// `nvidia-smi --query-compute-apps=pid,gpu_bus_id --format=csv,noheader` 的输出里，在这张卡上、不是本进程的计算进程（登记第十一节「机器被别的活挤」）。
pub(super) fn other_compute_processes_on_the_card(
    query_text: &str,
    bus_identifier: &str,
    own_process: u32,
) -> Vec<u32> {
    query_text
        .lines()
        .filter_map(|line| {
            let (process, bus) = line.split_once(',')?;
            let process: u32 = process.trim().parse().ok()?;
            (bus.trim().eq_ignore_ascii_case(bus_identifier) && process != own_process)
                .then_some(process)
        })
        .collect()
}

/// 两核的输入形态（登记第一节「输入形态」，岔路单第 5 行甲、乙）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum InputForm {
    /// 两核读同一块输入缓冲。
    Shared,
    /// 每核一块、各写一次。
    Split,
}

impl InputForm {
    pub(super) const ALL: [InputForm; 2] = [InputForm::Shared, InputForm::Split];

    pub(super) const fn label(self) -> &'static str {
        match self {
            InputForm::Shared => "shared",
            InputForm::Split => "split",
        }
    }

    /// 这一种形态一次提交往卡上写几份单元字节。
    pub(super) const fn input_copies(self) -> u64 {
        match self {
            InputForm::Shared => 1,
            InputForm::Split => 2,
        }
    }

    /// 第二段内容种子里的那一位（共用 0、分开 1，登记第五节「第二段怎么量」）。
    pub(super) const fn seed_bits(self) -> u64 {
        match self {
            InputForm::Shared => 0,
            InputForm::Split => 1,
        }
    }
}

/// 睡眠等的两档间隔（纳秒；登记第八节，第一节问题单第 3 行主 agent 认下）。
pub(super) const SHORT_SLEEP_INTERVAL_NANOSECONDS: u64 = 100_000;
pub(super) const LONG_SLEEP_INTERVAL_NANOSECONDS: u64 = 1_000_000;

/// 等法（登记第一节「等法」）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum WaitMethod {
    /// `device.poll(Wait { submission_index: Some(这一次), timeout: None })`。
    Blocking,
    /// 「poll 一次；回读回调到了就出；没到睡 100 µs」。
    SleepingShortly,
    /// 同上，睡 1 ms。
    SleepingLonger,
}

impl WaitMethod {
    pub(super) const ALL: [WaitMethod; 3] = [
        WaitMethod::Blocking,
        WaitMethod::SleepingShortly,
        WaitMethod::SleepingLonger,
    ];

    pub(super) const fn label(self) -> &'static str {
        match self {
            WaitMethod::Blocking => "blocking",
            WaitMethod::SleepingShortly => "sleep_100_microseconds",
            WaitMethod::SleepingLonger => "sleep_1_millisecond",
        }
    }

    /// 睡眠等的间隔；阻塞等交回 `None`。
    pub(super) const fn sleep_interval(self) -> Option<Duration> {
        match self {
            WaitMethod::Blocking => None,
            WaitMethod::SleepingShortly => {
                Some(Duration::from_nanos(SHORT_SLEEP_INTERVAL_NANOSECONDS))
            }
            WaitMethod::SleepingLonger => {
                Some(Duration::from_nanos(LONG_SLEEP_INTERVAL_NANOSECONDS))
            }
        }
    }
}

/// 配置行里的睡眠间隔（第九节 V98 盯的那一段）。
pub(super) fn sleep_interval_configuration_text() -> String {
    format!(
    "sleep_interval_nanoseconds={SHORT_SLEEP_INTERVAL_NANOSECONDS},{LONG_SLEEP_INTERVAL_NANOSECONDS}"
)
}

/// 睡眠等（登记第一节「等法」）：循环「poll 一次（`poll_once` 交回回读回调到没到）；到了就出；没到睡 `interval`」，交回 poll 了几次。
/// poll 与睡都从参数进来，单测换成假的（前 5 次没好、第 6 次好，应睡 5 次）。
pub(super) fn wait_by_sleeping(
    mut poll_once: impl FnMut() -> bool,
    mut sleep: impl FnMut(Duration),
    interval: Duration,
) -> u64 {
    let mut polls = 0_u64;
    // 迭代上界：回读回调在提交做完之后一定到（设备丢了另由 poll 的错误报出），每圈 poll 一次
    loop {
        polls += 1;
        if poll_once() {
            return polls;
        }
        sleep(interval);
    }
}

/// 一次提交往卡上写的那一块（① 写哪几块输入缓冲）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum InputBufferOfTheKernel {
    /// 共用形态两核都读它；分开形态只有核一读它。
    KernelOne,
    /// 分开形态核二读它。
    KernelTwo,
}

/// ① 把这一次的单元字节写进卡上的输入缓冲（`write` 是 `queue.write_buffer` 或单测里记账的假的），交回写进卡的单元字节数
/// （第七节乙类 `input_bytes_written`：共用 n × 32768，分开 2 × n × 32768）。长度表与派活参数另写，不计在内。
pub(super) fn write_the_unit_bytes(
    form: InputForm,
    unit_bytes: &[u8],
    mut write: impl FnMut(InputBufferOfTheKernel, &[u8]),
) -> u64 {
    let length = u64::try_from(unit_bytes.len()).expect("一次提交的字节装得进 u64");
    match form {
        InputForm::Shared => {
            write(InputBufferOfTheKernel::KernelOne, unit_bytes);
            length
        }
        InputForm::Split => {
            write(InputBufferOfTheKernel::KernelOne, unit_bytes);
            write(InputBufferOfTheKernel::KernelTwo, unit_bytes);
            2 * length
        }
    }
}

/// 一种形态一次提交该写进卡的单元字节（第七节乙类锚点、P4 物理下限的分子）。
pub(super) const fn expected_input_bytes(form: InputForm, units: u64) -> u64 {
    units * LOCAL_GPU_SLOT_BYTES * form.input_copies()
}

/// P4：一次提交的挂钟没快过「这一次写进卡的字节 ÷ 链路峰值」（分开输入按两份字节算）。
pub(super) fn respects_the_floor_of_the_form(
    wall_seconds: f64,
    form: InputForm,
    units: u64,
    link_peak_bytes_per_second: f64,
) -> bool {
    respects_the_physical_floor(
        wall_seconds,
        units * LOCAL_GPU_SLOT_BYTES * form.input_copies(),
        link_peak_bytes_per_second,
    )
}

/// ⑥ 的比对结果：确认（两核相同）与不确定的条数、不确定的下标、两核各与期望值（计时窗口外 CPU 算好的）不同的条数、两份序号都对不对。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SubmissionComparison {
    pub(super) confirmed: u64,
    pub(super) uncertain_entries: Vec<usize>,
    pub(super) kernel_one_differs_from_the_expected: u64,
    pub(super) kernel_two_differs_from_the_expected: u64,
    pub(super) are_the_sequence_numbers_holding: bool,
}

/// 回读缓冲里一条结果的字节（[`LOCAL_GPU_RESULT_ENTRY_BYTES`]，换成下标用的 usize）。
pub(super) fn result_entry_bytes() -> usize {
    usize::try_from(LOCAL_GPU_RESULT_ENTRY_BYTES).expect("一条结果 8 字节装得进 usize")
}

/// 回读缓冲里一条结果：CRC 与序号，小端。
pub(super) fn readback_entry(readback: &[u8], entry: usize) -> (u32, u32) {
    let start = entry * result_entry_bytes();
    let word = |offset: usize| {
        u32::from_le_bytes(
            readback[start + offset..start + offset + 4]
                .try_into()
                .expect("4 字节"),
        )
    };
    (word(0), word(4))
}

/// ⑥ 读回两份结果（前 n 条核一、后 n 条核二）、核每条结果的序号、逐条比两份（相同记确认、不同记不确定），再逐条与期望值比。
/// `kernel_two_flip` 给了就把核二那一条的 CRC 异或 1（P3 在读回之后、比对之前注入）。
pub(super) fn compare_the_readback(
    readback: &[u8],
    expected: &[u32],
    kernel_two_flip: Option<usize>,
) -> SubmissionComparison {
    let units = expected.len();
    let mut comparison = SubmissionComparison {
        confirmed: 0,
        uncertain_entries: Vec::new(),
        kernel_one_differs_from_the_expected: 0,
        kernel_two_differs_from_the_expected: 0,
        are_the_sequence_numbers_holding: readback.len() >= 2 * units * result_entry_bytes(),
    };
    if !comparison.are_the_sequence_numbers_holding {
        return comparison;
    }
    for (entry, expected_crc) in expected.iter().enumerate() {
        let (kernel_one_crc, kernel_one_sequence) = readback_entry(readback, entry);
        let (mut kernel_two_crc, kernel_two_sequence) = readback_entry(readback, units + entry);
        if kernel_two_flip == Some(entry) {
            kernel_two_crc ^= 1;
        }
        let entry_number = u32::try_from(entry).expect("一次提交的条数装得进 u32");
        comparison.are_the_sequence_numbers_holding &= kernel_one_sequence == entry_number;
        comparison.are_the_sequence_numbers_holding &= kernel_two_sequence == entry_number;
        if kernel_one_crc == kernel_two_crc {
            comparison.confirmed += 1;
        } else {
            comparison.uncertain_entries.push(entry);
        }
        comparison.kernel_one_differs_from_the_expected +=
            u64::from(kernel_one_crc != *expected_crc);
        comparison.kernel_two_differs_from_the_expected +=
            u64::from(kernel_two_crc != *expected_crc);
    }
    comparison
}

/// 一次 GPU 提交的七步（登记第一节「一次 GPU 提交」①–⑦）。真卡上的常驻写法、今天的写法与单测里按假钟走的步骤各实现一份。
pub(super) trait SubmissionSteps {
    /// ① 写输入（单元字节、长度表、派活参数），交回写进卡的单元字节。
    fn prepare(&mut self) -> u64;
    /// ② 编命令缓冲、③ 提交一次。
    fn encode_and_submit(&mut self);
    /// ④ 回读映射登记。
    fn register_the_readback(&mut self);
    /// ⑤ 等：交回睡眠等 poll 了几次（阻塞等交 0）；等那一段分不出来的写法（今天的写法，等在 ① 那一次调用里）交 `None`。
    fn wait(&mut self) -> Option<u64>;
    /// ⑥ 读回、核序号、逐条比两份、与期望值比。
    fn read_and_compare(&mut self) -> SubmissionComparison;
    /// ⑦ 解映射。
    fn unmap(&mut self);
}

/// 计时用的三个钟（登记第一节「c_rt」：线程钟；「c_rt 的进程钟版」；挂钟 `CLOCK_MONOTONIC_RAW` 包着两个 CPU 钟）。真钟与单测的假钟各一份。
pub(super) trait SubmissionClocks {
    fn thread_cpu(&mut self) -> Option<u64>;
    fn process_cpu(&mut self) -> Option<u64>;
    fn wall(&mut self) -> u64;
}

/// 真钟：本线程的线程钟、本进程的进程钟、`CLOCK_MONOTONIC_RAW`。
pub(super) struct ClocksOfThisThread;

impl SubmissionClocks for ClocksOfThisThread {
    fn thread_cpu(&mut self) -> Option<u64> {
        thread_cpu_nanoseconds()
    }

    fn process_cpu(&mut self) -> Option<u64> {
        process_cpu_nanoseconds()
    }

    fn wall(&mut self) -> u64 {
        RawWallClockReading::now().0
    }
}

/// 一次计时提交的读数（纳秒）。`wait_*` 在等那一段分不出来时是 `None`。
#[derive(Clone, Debug, PartialEq)]
pub(super) struct SubmissionTiming {
    pub(super) thread_cpu_nanoseconds: u64,
    pub(super) process_cpu_nanoseconds: u64,
    pub(super) wall_nanoseconds: u64,
    pub(super) wait_thread_cpu_nanoseconds: Option<u64>,
    pub(super) wait_wall_nanoseconds: Option<u64>,
    pub(super) polls: Option<u64>,
    pub(super) input_bytes_written: u64,
    pub(super) comparison: SubmissionComparison,
}

impl SubmissionTiming {
    /// c_floor：c_rt − c_wait（与等法、核快慢无关的那一部分；登记第一节「等那一段」）。
    pub(super) fn floor_thread_cpu_nanoseconds(&self) -> Option<u64> {
        Some(
            self.thread_cpu_nanoseconds
                .saturating_sub(self.wait_thread_cpu_nanoseconds?),
        )
    }
}

/// 一次计时提交（登记第五节「每次计时提交」）：读挂钟、线程钟、进程钟 → P1 插的那一段（`insertion`，① 之前）→ ①–⑦（⑤ 前后另读线程钟与
/// 挂钟）→ 读进程钟、线程钟、挂钟（挂钟包着两个 CPU 钟）。任何一次 CPU 钟读不到交回 `None`（那一格那一轮作废，登记第十一节「读不到」）。
pub(super) fn time_one_submission(
    steps: &mut dyn SubmissionSteps,
    clocks: &mut dyn SubmissionClocks,
    insertion: &mut dyn FnMut(),
) -> Option<SubmissionTiming> {
    let wall_before = clocks.wall();
    let thread_before = clocks.thread_cpu()?;
    let process_before = clocks.process_cpu()?;
    insertion();
    let input_bytes_written = steps.prepare();
    steps.encode_and_submit();
    steps.register_the_readback();
    let wait_wall_before = clocks.wall();
    let wait_thread_before = clocks.thread_cpu()?;
    let polls = steps.wait();
    let wait_thread_after = clocks.thread_cpu()?;
    let wait_wall_after = clocks.wall();
    let comparison = steps.read_and_compare();
    steps.unmap();
    let process_after = clocks.process_cpu()?;
    let thread_after = clocks.thread_cpu()?;
    let wall_after = clocks.wall();
    Some(SubmissionTiming {
        thread_cpu_nanoseconds: thread_after.saturating_sub(thread_before),
        process_cpu_nanoseconds: process_after.saturating_sub(process_before),
        wall_nanoseconds: wall_after.saturating_sub(wall_before),
        wait_thread_cpu_nanoseconds: polls
            .map(|_| wait_thread_after.saturating_sub(wait_thread_before)),
        wait_wall_nanoseconds: polls.map(|_| wait_wall_after.saturating_sub(wait_wall_before)),
        polls,
        input_bytes_written,
        comparison,
    })
}

/// 一格一轮的计时提交（登记第五节「每轮」）：`count` 次，每次之前（第一次除外）睡 `gap`，睡在下一次读挂钟之前、不在区间里。
/// 交回逐次的读数（读不到的那一次是 `None`）。热身由调用方另跑同一个函数、丢掉读数。
pub(super) fn time_the_submissions_of_a_cell(
    steps: &mut dyn SubmissionSteps,
    clocks: &mut dyn SubmissionClocks,
    count: usize,
    gap: Duration,
    sleep: &mut dyn FnMut(Duration),
    insertion: &mut dyn FnMut(),
) -> Vec<Option<SubmissionTiming>> {
    let mut timings = Vec::with_capacity(count);
    for submission in 0..count {
        let gap_before_this_submission = if submission == 0 { Duration::ZERO } else { gap };
        sleep(gap_before_this_submission);
        timings.push(time_one_submission(steps, clocks, insertion));
    }
    timings
}

/// 一格一轮的中位与轨迹（`name=submission_cell` 那一行的数）。有一次读不到交回 `None`。
#[derive(Clone, Debug, PartialEq)]
pub(super) struct CellRoundReading {
    pub(super) thread_cpu_median_nanoseconds: f64,
    pub(super) floor_thread_cpu_median_nanoseconds: Option<f64>,
    pub(super) process_cpu_median_nanoseconds: f64,
    pub(super) wait_thread_cpu_median_nanoseconds: Option<f64>,
    pub(super) wait_wall_median_nanoseconds: Option<f64>,
    pub(super) wall_median_nanoseconds: f64,
    pub(super) polls_median: Option<f64>,
    pub(super) input_bytes_written: Vec<u64>,
    pub(super) thread_cpu_in_order: Vec<f64>,
    pub(super) are_the_results_holding: bool,
    pub(super) are_the_sequence_numbers_holding: bool,
    pub(super) uncertain_total: usize,
    pub(super) is_every_thread_cpu_inside_the_wall: bool,
    /// 第七节甲类：每次提交 ⑤ 那一段的线程钟 ≤ 整次的线程钟（等那一段分不出来的写法不判）。
    pub(super) is_every_wait_inside_the_submission: bool,
    /// 第七节甲类：每次提交「确认」+「不确定」= n（两份核各算一次、逐条比了）。
    pub(super) are_the_comparison_counts_holding: bool,
}

pub(super) fn cell_round_reading(
    timings: &[Option<SubmissionTiming>],
    units: u64,
) -> Option<CellRoundReading> {
    let timings: Vec<&SubmissionTiming> = timings
        .iter()
        .map(Option::as_ref)
        .collect::<Option<Vec<_>>>()?;
    let median_of_the = |value_of: &dyn Fn(&SubmissionTiming) -> Option<u64>| -> Option<f64> {
        let values: Vec<f64> = timings
            .iter()
            .map(|timing| value_of(timing).map(|value| value as f64))
            .collect::<Option<Vec<f64>>>()?;
        median_of(&values)
    };
    Some(CellRoundReading {
        thread_cpu_median_nanoseconds: median_of_the(&|timing| {
            Some(timing.thread_cpu_nanoseconds)
        })?,
        floor_thread_cpu_median_nanoseconds: median_of_the(
            &SubmissionTiming::floor_thread_cpu_nanoseconds,
        ),
        process_cpu_median_nanoseconds: median_of_the(&|timing| {
            Some(timing.process_cpu_nanoseconds)
        })?,
        wait_thread_cpu_median_nanoseconds: median_of_the(&|timing| {
            timing.wait_thread_cpu_nanoseconds
        }),
        wait_wall_median_nanoseconds: median_of_the(&|timing| timing.wait_wall_nanoseconds),
        wall_median_nanoseconds: median_of_the(&|timing| Some(timing.wall_nanoseconds))?,
        polls_median: median_of_the(&|timing| timing.polls),
        input_bytes_written: timings
            .iter()
            .map(|timing| timing.input_bytes_written)
            .collect(),
        thread_cpu_in_order: timings
            .iter()
            .map(|timing| timing.thread_cpu_nanoseconds as f64)
            .collect(),
        are_the_results_holding: timings.iter().all(|timing| {
            timing.comparison.kernel_one_differs_from_the_expected == 0
                && timing.comparison.kernel_two_differs_from_the_expected == 0
        }),
        are_the_sequence_numbers_holding: timings
            .iter()
            .all(|timing| timing.comparison.are_the_sequence_numbers_holding),
        uncertain_total: timings
            .iter()
            .map(|timing| timing.comparison.uncertain_entries.len())
            .sum(),
        is_every_thread_cpu_inside_the_wall: timings.iter().all(|timing| {
            is_the_thread_cpu_inside_the_wall(
                timing.thread_cpu_nanoseconds,
                timing.wall_nanoseconds,
            )
        }),
        is_every_wait_inside_the_submission: timings.iter().all(|timing| {
            timing
                .wait_thread_cpu_nanoseconds
                .is_none_or(|wait| wait <= timing.thread_cpu_nanoseconds)
        }),
        are_the_comparison_counts_holding: timings.iter().all(|timing| {
            timing.comparison.confirmed
                + u64::try_from(timing.comparison.uncertain_entries.len()).expect("条数")
                == units
        }),
    })
}

#[cfg(test)]
mod tests;
