//! 按段登记表：登记过的长段放量到哪、超过 u64 的段切片逐趟判（[`registered_segments_of_the_flow`]、
//! [`segment_slices_of_the_recording`]），每一片判到哪的进度与从哪接着判。

use super::*;

// ============================== 按段登记 ==============================

/// 按段登记表的路径从这个环境变量交给引擎（入口 `scripts/crash/crash-point-replay.sh` 开跑时送；表是
/// `.claude/gate.d/crash-point-replay-segments.tsv`，门禁与表归工具那一路）。没设就是一段都没登记。
pub const SEGMENTS_REGISTRY_ENVIRONMENT_VARIABLE: &str =
    "SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY";

/// 登记了的段怎么展开：写数不超过这一趟的展开上限（`expand_up_to`）的全部真子集（全展开罩着甲二），超过的取甲二
/// （展开上限盖不住的段至少跑甲二）。
#[must_use]
pub fn expansion_of_a_registered_segment(
    segment: &[usize],
    expand_up_to: ExpandUpToWritesPerSegment,
) -> CrashPointReplaySegmentExpansion {
    if segment.len() <= expand_up_to.0 {
        CrashPointReplaySegmentExpansion::EveryProperSubset
    } else {
        CrashPointReplaySegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll
    }
}

/// 段写表摘要的十六进制位数（sha256）。
const SEGMENT_WRITE_TABLE_DIGEST_HEXADECIMAL_DIGITS: usize = 2 * DIGEST_BYTES;

/// 按段登记表里一条流登记了的一段：段号、段写数、段写表摘要（那一段带屏障的写表，`crate::crash_identity::step_write_table_with_barriers`
/// 只给这一段算的那一份，64 位小写十六进制）。整域按放量的展开上限展不开的段登记在这里，引擎照这一趟的展开上限展开、超过的取甲二
/// （[`expansion_of_a_registered_segment`]）、计入跳过量。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisteredSegment {
    pub segment_index: usize,
    pub write_count: usize,
    pub write_table_digest_text: String,
}

/// 读按段登记表的正文，取流名是 `flow_name` 的那几行登记的段，按段号排。一行 `<合表项名>\t<流名>\t<段号>\t<段写数>\t<段写表摘要>`，
/// 可再加一段 `#` 起头的说明；`#` 起头的行与空行跳过。第一段（合表项名）这里只核非空，在不在合表里归入口的核法
/// （`scripts/crash/lib_crash_point_replay_items.py`）；引擎按第二段（流名）取：一项有几条流，各条各登各的段。
///
/// # Errors
/// 有一行写坏了（不是五段、合表项名是空的、段号或段写数不是十进制非负整数或带前导 0、段写数是 0、摘要不是 64 位小写十六进制、
/// 同一条流同一段两行），错误里写第几行、哪一处不对与那一行原文：写坏的表不当成「没登记」，整趟拒跑。
pub fn registered_segments_of_the_flow(
    registry_text: &str,
    flow_name: &str,
) -> Result<Vec<RegisteredSegment>, String> {
    let decimal = |text: &str| -> Option<usize> {
        let is_decimal = !text.is_empty()
            && text.bytes().all(|byte| byte.is_ascii_digit())
            && (text == "0" || !text.starts_with('0'));
        if is_decimal {
            text.parse::<usize>().ok()
        } else {
            None
        }
    };
    let mut registered: Vec<RegisteredSegment> = Vec::new();
    let mut every_flow_and_segment: BTreeSet<(String, usize)> = BTreeSet::new();
    for (line_index, line) in registry_text.lines().enumerate() {
        let line_number = line_index + 1;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let refused = |reason: &str| {
            Err(format!(
                "按段登记表第 {line_number} 行写坏了（{reason}）：「{line}」"
            ))
        };
        let mut fields: Vec<&str> = line.split('\t').collect();
        if fields.len() == 6 && fields[5].starts_with('#') {
            fields.pop();
        }
        let [item_field, flow_field, segment_field, write_count_field, digest_field] =
            fields.as_slice()
        else {
            return refused(
                "一行五段（合表项名、流名、段号、段写数、段写表摘要），制表符隔开，可再加一段 # 说明",
            );
        };
        if item_field.is_empty() {
            return refused("第一段合表项名不许空");
        }
        let Some(segment_index) = decimal(segment_field) else {
            return refused("段号要是十进制非负整数、不带前导 0");
        };
        let Some(write_count) = decimal(write_count_field).filter(|count| *count > 0) else {
            return refused("段写数要是大于 0 的十进制整数、不带前导 0");
        };
        let digest_is_lowercase_hexadecimal = digest_field.len()
            == SEGMENT_WRITE_TABLE_DIGEST_HEXADECIMAL_DIGITS
            && digest_field
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
        if !digest_is_lowercase_hexadecimal {
            return refused("段写表摘要要是 64 位小写十六进制（sha256）");
        }
        if !every_flow_and_segment.insert(((*flow_field).to_string(), segment_index)) {
            return refused("同一条流同一段只许一行");
        }
        if *flow_field == flow_name {
            registered.push(RegisteredSegment {
                segment_index,
                write_count,
                write_table_digest_text: (*digest_field).to_string(),
            });
        }
    }
    registered.sort_by_key(|segment| segment.segment_index);
    Ok(registered)
}

/// 这条流第 `segment_index` 段带屏障的写表摘要（按段登记表第五段的取法：只给这一段算 `step_write_table_with_barriers`）；
/// 这一段的写下标不连续时交回 `None`（录制流切出的段都是连续的，不连续说明这条流不是录出来的那种，登记对不上）。
fn segment_write_table_digest_text(flow: &CrashFlow<'_>, segment_index: usize) -> Option<String> {
    let segment = flow.segments.get(segment_index)?;
    let first = *segment.first()?;
    let last = *segment.last()?;
    let is_contiguous = segment.windows(2).all(|pair| pair[1] == pair[0] + 1);
    is_contiguous.then(|| {
        hexadecimal_text(&step_write_table_with_barriers(
            flow.writes,
            std::slice::from_ref(segment),
            first..last + 1,
        ))
    })
}

/// 登记的每一段与这条流对得上：段在、段写数相同、段写表摘要相同（M4 第 15 条：写表变了、段号挪位之后对不上就拒跑，不按写数重找、不照段号跳过）。
///
/// # Errors
/// 第一处对不上的段：项名、段号、登记的与这条流的。
pub fn check_the_registered_segments_against_the_flow(
    flow: &CrashFlow<'_>,
    flow_name: &str,
    registered_segments: &[RegisteredSegment],
) -> Result<(), String> {
    for registered in registered_segments {
        let Some(segment) = flow.segments.get(registered.segment_index) else {
            return Err(format!(
                "按段登记表登记了项 {flow_name} 的第 {} 段，这条流只有 {} 段：写表变了，登记对不上，拒跑（改登记表或改回写表）",
                registered.segment_index,
                flow.segments.len()
            ));
        };
        if segment.len() != registered.write_count {
            return Err(format!(
                "按段登记表登记项 {flow_name} 第 {} 段 {} 次写，这条流这一段 {} 次写：写表变了，登记对不上，拒跑",
                registered.segment_index,
                registered.write_count,
                segment.len()
            ));
        }
        let digest = segment_write_table_digest_text(flow, registered.segment_index);
        if digest.as_deref() != Some(registered.write_table_digest_text.as_str()) {
            return Err(format!(
                "按段登记表登记项 {flow_name} 第 {} 段的写表摘要 {}，这条流这一段是 {}：写表变了，登记对不上，拒跑",
                registered.segment_index,
                registered.write_table_digest_text,
                digest.as_deref().unwrap_or("（这一段的写下标不连续，算不出）")
            ));
        }
    }
    Ok(())
}

/// 这一趟这条流登记了的段：[`SEGMENTS_REGISTRY_ENVIRONMENT_VARIABLE`] 没设是一段都没有；设了就读那份表、取流名是 `flow_name` 的那几行
/// （[`registered_segments_of_the_flow`]），再逐段与这条流核（[`check_the_registered_segments_against_the_flow`]）。
///
/// # Errors
/// 表读不了、写坏了、登记的段对不上。
pub(super) fn registered_segments_of_the_flow_from_the_environment(
    flow: &CrashFlow<'_>,
    flow_name: &str,
) -> Result<Vec<RegisteredSegment>, String> {
    let Some(registry_path) = std::env::var_os("SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY")
    else {
        return Ok(Vec::new());
    };
    let registry_path = PathBuf::from(registry_path);
    let registry_text = std::fs::read_to_string(&registry_path).map_err(|error| {
        format!(
            "{SEGMENTS_REGISTRY_ENVIRONMENT_VARIABLE} 指的按段登记表读不了：{}（{error}）",
            registry_path.display()
        )
    })?;
    let registered = registered_segments_of_the_flow(&registry_text, flow_name)?;
    check_the_registered_segments_against_the_flow(flow, flow_name, &registered)?;
    Ok(registered)
}

/// 录入段里第 `segment_index` 段怎么展开：登记了的段照这一趟的展开上限（`expand_up_to`），超过的取甲二
/// （[`expansion_of_a_registered_segment`]）；没登记的照调用方给的（`flow.expansion`）。
pub(super) fn expansion_in_the_recording(
    registered_segments: &[RegisteredSegment],
    flow: &CrashFlow<'_>,
    expand_up_to: ExpandUpToWritesPerSegment,
    segment_index: usize,
    segment: &[usize],
) -> CrashPointReplaySegmentExpansion {
    if registered_segments
        .iter()
        .any(|registered| registered.segment_index == segment_index)
    {
        expansion_of_a_registered_segment(segment, expand_up_to)
    } else {
        (flow.expansion)(segment_index, segment)
    }
}

/// 按段登记的段整域装不进 u64 时（C790「先段内取样……每次走 2^32，然后通过剪枝慢慢的剪开」），一趟判几个态：
/// 这个环境变量给（要是正整数、不超过 [`LARGEST_REGISTERED_SEGMENT_SLICE_STATES`]），没设取 [`DEFAULT_REGISTERED_SEGMENT_SLICE_STATES`]。
pub const REGISTERED_SEGMENT_SLICE_STATES_ENVIRONMENT_VARIABLE: &str =
    "SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES";

/// 没设片长时一趟判 2^32 个态。
pub const DEFAULT_REGISTERED_SEGMENT_SLICE_STATES: u64 = 1 << 32;

/// 片长的上界 2^32：两个 GPU 内核把片内序号当 u32 加到片起点的各位上（序号 − 段起点的高 32 位要是 0）。
pub const LARGEST_REGISTERED_SEGMENT_SLICE_STATES: u64 = 1 << 32;

/// 片长变量的值（`None` 是没设）换成一趟判几个态。片长要是 [`STATES_PER_BLOCK`] 的整数倍：块从片起点按它切，片起点（续点）因此总落在它的倍数上，
/// 同一段内编号的块几趟之间块长相同（片长不是它的倍数时片末那一块短，下一趟换一种片长时同一块键录成另一个块长，
/// 录入表拒、每一趟都卡在同一处：`VerdictStore::record_block_of_a_slice`）。
///
/// # Errors
/// 设了而不是十进制正整数、超过 [`LARGEST_REGISTERED_SEGMENT_SLICE_STATES`]，或不是 [`STATES_PER_BLOCK`] 的整数倍：错误带变量名与原值
/// （写错的值不悄悄变成默认那一档）。
pub fn registered_segment_slice_states_of_the_variable_value(
    value: Option<&str>,
) -> Result<u64, String> {
    let Some(text) = value else {
        return Ok(DEFAULT_REGISTERED_SEGMENT_SLICE_STATES);
    };
    let is_decimal = !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit());
    match text.parse::<u64>() {
        Ok(states)
            if is_decimal
                && states > 0
                && states <= LARGEST_REGISTERED_SEGMENT_SLICE_STATES
                && states % STATES_PER_BLOCK == 0 =>
        {
            Ok(states)
        }
        Ok(_) | Err(_) => Err(format!(
            "{REGISTERED_SEGMENT_SLICE_STATES_ENVIRONMENT_VARIABLE}={text}：一趟判几个态要是十进制正整数、不超过 {LARGEST_REGISTERED_SEGMENT_SLICE_STATES}（GPU 内核按 32 位加片内序号）、是一块的态数 {STATES_PER_BLOCK} 的整数倍（块从片起点按它切，块长几趟之间才对得上）"
        )),
    }
}

/// 这个进程的片长变量（[`REGISTERED_SEGMENT_SLICE_STATES_ENVIRONMENT_VARIABLE`]）换成一趟判几个态（[`registered_segment_slice_states_of_the_variable_value`]）。
///
/// # Errors
/// 值不是 UTF-8；值认不出（同 [`registered_segment_slice_states_of_the_variable_value`]）。
pub fn registered_segment_slice_states_from_the_environment() -> Result<u64, String> {
    let slice_states_text = std::env::var_os(REGISTERED_SEGMENT_SLICE_STATES_ENVIRONMENT_VARIABLE)
        .map(|value| {
            value.into_string().map_err(|value| {
                format!(
                    "{REGISTERED_SEGMENT_SLICE_STATES_ENVIRONMENT_VARIABLE}={value:?}：不是 UTF-8"
                )
            })
        })
        .transpose()?;
    registered_segment_slice_states_of_the_variable_value(slice_states_text.as_deref())
}

/// 按段登记的一段在这一趟怎么判：整域装得进 u64 就全展开（`full`）；装不进就分片（`slice`），每趟从续点起判一片。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DomainKindOfARegisteredSegment {
    Full,
    Slice,
}

impl DomainKindOfARegisteredSegment {
    /// 驱动行 `registered_segment_domains=` 里的写法。
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Slice => "slice",
        }
    }
}

/// 按段登记的一段：段号、整域的态数（[`StateCount1280Bits`]，按 [`expansion_of_a_registered_segment`] 的展开）与它在这一趟怎么判。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DomainOfARegisteredSegment {
    pub segment_index: usize,
    pub domain_state_count: StateCount1280Bits,
    pub kind: DomainKindOfARegisteredSegment,
}

/// 录入段：按段登记的每一段整域的态数与它这一趟怎么判（`flow` 里登记的段已换成 [`expansion_in_the_recording`] 的那一份；ignore 的点的段
/// 不展开，同 [`plan_crash_points`]）。整域装得进 u64 是 `full`，装不进是 `slice`。
///
/// # Errors
/// 整域越过 1280 位（[`STATE_COUNT_LIMBS`] 个 u128位宽）：拒跑，都在写库之前交回。
pub fn domains_of_the_registered_segments(
    flow: &CrashFlow<'_>,
    registered_segments: &[RegisteredSegment],
) -> Result<Vec<DomainOfARegisteredSegment>, String> {
    let owner_of_segment = owner_of_each_segment(flow);
    let tearable = TearableInPlaceOverwrites::of(flow.base, flow.writes);
    registered_segments
        .iter()
        .map(|registered| {
            let segment_index = registered.segment_index;
            let segment = &flow.segments[segment_index];
            let expansion = if flow.crash_points[owner_of_segment[segment_index]].ignored {
                CrashPointReplaySegmentExpansion::NotExpanded
            } else {
                (flow.expansion)(segment_index, segment)
            };
            let domain_state_count = expansion
                .checked_state_count(flow.writes, segment, &tearable)
                .ok_or_else(|| {
                    format!(
                        "按段登记表登记的第 {segment_index} 段（{} 次写）按登记段的展开（{}）展开的态数越过 {} 位（{STATE_COUNT_LIMBS} 个 u128，\
                         用户 2026-10-13 定的位宽）：装不下，拒跑（没写库）",
                        segment.len(),
                        expansion.name(),
                        STATE_COUNT_LIMBS * 128
                    )
                })?;
            let kind = match domain_state_count.to_u64() {
                Some(_) => DomainKindOfARegisteredSegment::Full,
                None => DomainKindOfARegisteredSegment::Slice,
            };
            Ok(DomainOfARegisteredSegment {
                segment_index,
                domain_state_count,
                kind,
            })
        })
        .collect()
}

/// 录入段排计划之前核：`flow`（登记了的段已换成 [`expansion_in_the_recording`] 的那一份）照计划的展开（ignore 的点的段不展开，
/// 同 [`plan_crash_points`]）、分片的段照 `segment_slices` 的片长，整条流的态数每段与合起来（再加全部持久那一个）都装得进 u64 才交回，
/// 排计划时不会在算态数那一步 panic。
///
/// # Errors
/// 不分片的段态数装不进 u64（登记了的段整域装不进 u64 的已经分片，到这里的是没登记的段）、整条流合起来装不下：拒跑，都在写库之前交回。
pub fn states_of_the_flow_to_record(
    flow: &CrashFlow<'_>,
    segment_slices: &[Option<CrashPointReplaySegmentSlice>],
) -> Result<u64, String> {
    let owner_of_segment = owner_of_each_segment(flow);
    let tearable = TearableInPlaceOverwrites::of(flow.base, flow.writes);
    let mut states = 1u64;
    for (segment_index, segment) in flow.segments.iter().enumerate() {
        let expansion = if flow.crash_points[owner_of_segment[segment_index]].ignored {
            CrashPointReplaySegmentExpansion::NotExpanded
        } else {
            (flow.expansion)(segment_index, segment)
        };
        let states_of_the_segment = match &segment_slices[segment_index] {
            Some(slice) => slice.state_count,
            None => expansion
                .checked_state_count(flow.writes, segment, &tearable)
                .and_then(|state_count| state_count.to_u64())
                .ok_or_else(|| {
                    format!(
                        "第 {segment_index} 段（{} 次写）按 {} 展开的态数装不进 u64，拒跑（没写库）",
                        segment.len(),
                        expansion.name()
                    )
                })?,
        };
        states = states.checked_add(states_of_the_segment).ok_or_else(|| {
            format!("整条流的态数到第 {segment_index} 段装不进 u64，拒跑（没写库）")
        })?;
    }
    Ok(states)
}

/// 录入段排计划之前定每段的片：按段登记的段整域装得进 u64 的全展开、不分片，装不进的分片，片长照
/// `SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES`、片起点是库里的续点（每个分片的段往 stderr 打一行 `CRASH_AMPLIFICATION_SLICE`，
/// [`segment_slices_of_the_recording`]）。`flow` 里登记了的段已换成 [`expansion_in_the_recording`] 的那一份。只读库，不写。
///
/// # Errors
/// 都在写库之前交回：登记段整域越过 1280 位；片长为 0 的那份计划（找续点用）算态数装不下，即没登记的段态数装不进 u64、
/// 或整条流合起来装不下；片长变量认不出；库读失败。
pub fn segment_slices_of_the_recording_from_the_environment_and_the_library(
    store: &VerdictStore,
    flow: &CrashFlow<'_>,
    flow_name: &str,
    registered_segments: &[RegisteredSegment],
    crash_judge: ComputedOn,
) -> Result<Vec<Option<CrashPointReplaySegmentSlice>>, String> {
    let domains = domains_of_the_registered_segments(flow, registered_segments)?;
    let slices_of_the_plan_to_find_the_resume_points =
        placeholder_slices_of_the_sliced_segments(flow.segments.len(), &domains);
    // 找续点那份计划同样要算态数（分片的段片长为 0）：先核装得进 u64，装不进在写库之前拒跑，不在排那份计划时 panic
    states_of_the_flow_to_record(flow, &slices_of_the_plan_to_find_the_resume_points)?;
    if !domains
        .iter()
        .any(|domain| domain.kind == DomainKindOfARegisteredSegment::Slice)
    {
        return Ok(no_segment_slices(flow.segments.len()));
    }
    let slice_states = registered_segment_slice_states_from_the_environment()?;
    // 续点从库里读（进度记录核过才信，核不过从 0 起逐块问库）：块键只由计划哈希与段内编号定，先排一份片长为 0 的计划拿块键
    let plan_to_find_the_resume_points =
        plan_crash_points_with_slices(flow, &slices_of_the_plan_to_find_the_resume_points);
    // 续点照镜像判定那一把版本找（由结论种类与判器引到的被检测产品代码定，两条路线同一把）
    let judged_by_in_the_recording = match crash_judge {
        ComputedOn::Gpu => JudgedBy::GpuJudge,
        ComputedOn::Cpu => JudgedBy::CpuJudge,
    };
    let conclusion_versions =
        ConclusionVersionsOfThePlan::of_the_plan(&plan_to_find_the_resume_points);
    let (segment_slices, resume_points) = segment_slices_of_the_recording(
        store,
        flow.segments.len(),
        &domains,
        &plan_to_find_the_resume_points,
        &conclusion_versions.stored_under(judged_by_in_the_recording),
        slice_states,
    )
    .map_err(|refused| format!("定分片的段的片失败：{refused:?}"))?;
    for (segment_index, resume_point) in &resume_points {
        eprintln!(
            "CRASH_AMPLIFICATION_SLICE flow={flow_name} segment={segment_index} slice_start={} slice_states={} resume_point_found_by={} judged_through_in_the_library={}",
            segment_slices[*segment_index].map_or_else(
                || resume_point.judged_through.decimal_text(),
                |slice| slice.start_within_the_domain.decimal_text()
            ),
            segment_slices[*segment_index].map_or(0, |slice| slice.state_count),
            match resume_point.found_by {
                ResumePointFoundBy::TheProgressRecordChecked => "progress_record",
                ResumePointFoundBy::AWalkFromZeroThroughTheLibrary => "walk_from_zero",
            },
            resume_point.judged_through.decimal_text()
        );
    }
    Ok(segment_slices)
}

/// 分片进度记在哪一个判器、哪一把版本下：这一段所在的点那一把（与它那几块的结论存在同一处）。
fn judge_of_the_progress(
    conclusions: &ConclusionsStoredUnder<'_>,
    point_index: usize,
) -> JudgeOfTheProgress {
    match conclusions {
        ConclusionsStoredUnder::TheJudgingUnitsFingerprint(version) => {
            JudgeOfTheProgress::TheJudgingUnitsFingerprint(**version)
        }
        ConclusionsStoredUnder::TheConclusionKind {
            kind,
            version_of_each_point,
        } => JudgeOfTheProgress::TheConclusionKind {
            kind: kind.clone(),
            version: ConclusionsStoredUnder::conclusion_version_of_the_point(
                version_of_each_point,
                point_index,
            ),
        },
    }
}

/// 计划里第 `segment_index` 段（分片的段）在 `conclusions` 那一处的分片进度记录的键：按这一段的内容摘要
/// （[`PlannedCrashPoint::sliced_segment_content_digest`]，不按流名、段号）与判器取，几趟之间片起点、片长怎么变都是同一把。
///
/// # Panics
/// `plan` 里第 `segment_index` 段不是分片的段。
#[must_use]
pub fn progress_key_of_the_sliced_segment(
    plan: &CrashPlan,
    segment_index: usize,
    conclusions: &ConclusionsStoredUnder<'_>,
) -> SlicedSegmentProgressKey {
    let (point_index, segment_in_point) =
        point_and_segment_in_point_of_the_sliced_segment(plan, segment_index)
            .expect("计划里这一段是分片的段");
    SlicedSegmentProgressKey {
        segment_content_digest: plan.points[point_index]
            .sliced_segment_content_digest(segment_in_point),
        judge: judge_of_the_progress(conclusions, point_index),
    }
}

/// 一段分片的段的续点是怎么找到的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResumePointFoundBy {
    /// 进度记录核过了：它指的那一块判完了、它之后那一块没判。
    TheProgressRecordChecked,
    /// 没有进度记录、或进度记录核不过：从编号 0 起逐块问库，走到第一块没判的（真相以块为准）。
    AWalkFromZeroThroughTheLibrary,
}

/// 一段分片的段从哪接着判：段内编号 [0, `judged_through`) 的块都判完了。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResumePointOfASlicedSegment {
    pub judged_through: StateCount1280Bits,
    pub found_by: ResumePointFoundBy,
}

/// 计划里第 `segment_index` 段（分片的段）落在哪个点、点内第几段：（点号，点内段序）。
fn point_and_segment_in_point_of_the_sliced_segment(
    plan: &CrashPlan,
    segment_index: usize,
) -> Option<(usize, usize)> {
    plan.sliced_segments()
        .into_iter()
        .find_map(|(point_index, piece)| match piece.kind {
            KindOfAPiece::SlicedSegment {
                segment_index: piece_segment,
                segment_in_point,
                ..
            } if piece_segment == segment_index => Some((point_index, segment_in_point)),
            KindOfAPiece::SlicedSegment { .. } | KindOfAPiece::SegmentsNotSliced { .. } => None,
        })
}

/// 录入段：分片的段从哪接着判（C790）。先读进度记录（[`VerdictStore::read_sliced_segment_progress`]）再核它——它记的最后判完的那一块
/// （段内编号 [`last_block_start`, `judged_through`)）在 `conclusions` 那一处有结论、录入表里的块长对得上，`judged_through` 那一块没有结论
/// （走到整域末尾的不用核这一半）；核过了就从 `judged_through` 接着。没有记录、或核不过，从编号 0 起逐块问库：一块有结论就照录入表里
/// 它的块长跳到下一块，走到第一块没有结论的停下，从那里接着（真相以块为准，进度记录只是加速）。
/// `plan` 是排出了这一段为分片的段的计划（块键只由计划哈希与段内编号定，不看这一份计划的片起点与片长）。
///
/// # Errors
/// 库读失败。
///
/// # Panics
/// `plan` 里第 `segment_index` 段不是分片的段。
pub fn resume_point_of_the_sliced_segment(
    store: &VerdictStore,
    plan: &CrashPlan,
    segment_index: usize,
    domain_state_count: &StateCount1280Bits,
    conclusions: &ConclusionsStoredUnder<'_>,
) -> Result<ResumePointOfASlicedSegment, VerdictStoreError> {
    let (point_index, segment_in_point) =
        point_and_segment_in_point_of_the_sliced_segment(plan, segment_index)
            .expect("计划里这一段是分片的段");
    let point = &plan.points[point_index];
    // 判过的就算（红的也算）：续点只管片从哪起，块要不要现判由判段的复用那一步定
    let is_judged = |start_within_the_domain: &StateCount1280Bits| {
        conclusions.is_judged_with_the_slots(
            store,
            point_index,
            &point.block_key_of_a_slice_block(segment_in_point, start_within_the_domain),
        )
    };
    let key = progress_key_of_the_sliced_segment(plan, segment_index, conclusions);
    if let Some(progress) = store.read_sliced_segment_progress(&key)? {
        let last_block_is_judged_with_its_length = if progress.judged_through.is_zero() {
            true
        } else {
            let length_of_the_last_block = progress
                .judged_through
                .checked_sub(&progress.last_block_start)
                .and_then(|length| length.to_u64());
            let recorded_length = store.recorded_state_count_of_block(
                &point.block_key_of_a_slice_block(segment_in_point, &progress.last_block_start),
            )?;
            length_of_the_last_block.is_some()
                && length_of_the_last_block == recorded_length
                && is_judged(&progress.last_block_start)?
        };
        let next_block_is_not_judged =
            progress.judged_through == *domain_state_count || !is_judged(&progress.judged_through)?;
        if progress.judged_through <= *domain_state_count
            && last_block_is_judged_with_its_length
            && next_block_is_not_judged
        {
            return Ok(ResumePointOfASlicedSegment {
                judged_through: progress.judged_through,
                found_by: ResumePointFoundBy::TheProgressRecordChecked,
            });
        }
    }
    let mut judged_through = StateCount1280Bits::ZERO;
    while judged_through < *domain_state_count && is_judged(&judged_through)? {
        let Some(length) = store.recorded_state_count_of_block(
            &point.block_key_of_a_slice_block(segment_in_point, &judged_through),
        )?
        else {
            break;
        };
        if length == 0 {
            break;
        }
        judged_through = judged_through
            .checked_add_u64(length)
            .expect("录入表里的块不越过段的域，装得进 1280 位");
    }
    Ok(ResumePointOfASlicedSegment {
        judged_through,
        found_by: ResumePointFoundBy::AWalkFromZeroThroughTheLibrary,
    })
}

/// 这一趟分片的段各判到了哪：从这一趟的片起点起，`conclusions` 那一处有结论的块连着数，数到第一块没有结论的为止
/// （交回（段号，判到的段内编号，最后判完的那一块的起点））；这一趟一块都没判完的交回片起点、最后那一块的起点记 `None`。只读库。
///
/// # Errors
/// 库读失败。
pub fn judged_through_of_the_sliced_segments(
    store: &VerdictStore,
    plan: &CrashPlan,
    conclusions: &ConclusionsStoredUnder<'_>,
) -> Result<Vec<(usize, StateCount1280Bits, Option<StateCount1280Bits>)>, VerdictStoreError> {
    let mut judged = Vec::new();
    for (point_index, piece) in plan.sliced_segments() {
        let KindOfAPiece::SlicedSegment {
            segment_index,
            start_within_the_domain,
            ..
        } = piece.kind
        else {
            continue;
        };
        let point = &plan.points[point_index];
        let mut judged_through = start_within_the_domain;
        let mut last_block_start = None;
        for local in point.blocks() {
            if !piece.local_states.contains(&local.start) {
                continue;
            }
            if !conclusions.is_stored(store, point_index, &point.block_key(&local))? {
                break;
            }
            let block_start = start_within_the_domain
                .checked_add_u64(local.start - piece.local_states.start)
                .expect("片里的编号装得进 1280 位");
            last_block_start = Some(block_start);
            judged_through = block_start
                .checked_add_u64(local.end - local.start)
                .expect("片里的编号装得进 1280 位");
        }
        judged.push((segment_index, judged_through, last_block_start));
    }
    Ok(judged)
}

/// 判段：这一趟分片的段判完的块落盘之后推进进度记录（[`judged_through_of_the_sliced_segments`]）；这一趟一块都没判完的段不写。
/// 被叫停提早收尾的那一趟同样推进到判完的地方。片从续点起（[`segment_slices_of_the_recording`]），判完的地方不在续点前面。
///
/// # Errors
/// 库读写失败。
pub fn advance_the_progress_of_the_sliced_segments(
    store: &mut VerdictStore,
    plan: &CrashPlan,
    conclusions: &ConclusionsStoredUnder<'_>,
) -> Result<(), VerdictStoreError> {
    for (segment_index, judged_through, last_block_start) in
        judged_through_of_the_sliced_segments(store, plan, conclusions)?
    {
        let Some(last_block_start) = last_block_start else {
            continue;
        };
        let progress_key = progress_key_of_the_sliced_segment(plan, segment_index, conclusions);
        store.store_sliced_segment_progress(
            &progress_key,
            SlicedSegmentProgress {
                judged_through,
                last_block_start,
            },
        )?;
    }
    Ok(())
}

/// 录入段每一段这一趟的片（与段同序）与每个分片的段（段号）的续点（[`segment_slices_of_the_recording`] 交回的）。
pub type SegmentSlicesAndResumePoints = (
    Vec<Option<CrashPointReplaySegmentSlice>>,
    Vec<(usize, ResumePointOfASlicedSegment)>,
);

/// 录入段定不出这一趟分片的段的片。
#[derive(Debug)]
pub enum SegmentSlicesOfTheRecordingRefused {
    /// 读库失败。
    Library(VerdictStoreError),
}

/// 录入段：这一趟每一段的片。不分片的段是 `None`；分片的段从续点（[`resume_point_of_the_sliced_segment`]）起判 `slice_states` 个态，
/// 到整域末尾为止（走完了的段片长是 0）。`plan_to_find_the_resume_points` 是这几段已排成分片的段的一份计划（片起点与片长随便给，
/// 块键不看它们）。交回每段的片与每个分片的段的续点（库里判到的地方）。
///
/// # Errors
/// 库读失败。
pub fn segment_slices_of_the_recording(
    store: &VerdictStore,
    segment_count: usize,
    domains: &[DomainOfARegisteredSegment],
    plan_to_find_the_resume_points: &CrashPlan,
    conclusions: &ConclusionsStoredUnder<'_>,
    slice_states: u64,
) -> Result<SegmentSlicesAndResumePoints, SegmentSlicesOfTheRecordingRefused> {
    let mut segment_slices = no_segment_slices(segment_count);
    let mut resume_points = Vec::new();
    for domain in domains {
        match domain.kind {
            DomainKindOfARegisteredSegment::Full => {}
            DomainKindOfARegisteredSegment::Slice => {
                let resume_point = resume_point_of_the_sliced_segment(
                    store,
                    plan_to_find_the_resume_points,
                    domain.segment_index,
                    &domain.domain_state_count,
                    conclusions,
                )
                .map_err(SegmentSlicesOfTheRecordingRefused::Library)?;
                let slice_start = resume_point.judged_through;
                let states_left = domain
                    .domain_state_count
                    .checked_sub(&slice_start)
                    .expect("片起点不越过整域");
                let state_count = states_left
                    .to_u64()
                    .map_or(slice_states, |left| left.min(slice_states));
                segment_slices[domain.segment_index] = Some(CrashPointReplaySegmentSlice {
                    start_within_the_domain: slice_start,
                    state_count,
                });
                resume_points.push((domain.segment_index, resume_point));
            }
        }
    }
    Ok((segment_slices, resume_points))
}

/// 排出续点之前那一份计划要的片：分片的段从 0 起、片长 0（块键不看片起点与片长，只用它找续点）。
#[must_use]
pub fn placeholder_slices_of_the_sliced_segments(
    segment_count: usize,
    domains: &[DomainOfARegisteredSegment],
) -> Vec<Option<CrashPointReplaySegmentSlice>> {
    let mut segment_slices = no_segment_slices(segment_count);
    for domain in domains {
        match domain.kind {
            DomainKindOfARegisteredSegment::Full => {}
            DomainKindOfARegisteredSegment::Slice => {
                segment_slices[domain.segment_index] = Some(CrashPointReplaySegmentSlice {
                    start_within_the_domain: StateCount1280Bits::ZERO,
                    state_count: 0,
                });
            }
        }
    }
    segment_slices
}

/// 驱动行里一段登记了的段的域与进度（[`segment_fields_of_the_line`]）：`full` 的段只报段号；`slice` 的段报到这一趟末判到的段内编号与整域的态数，
/// 另按片记账（[`slice_account_of_a_sliced_segment`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DomainAndProgressOfARegisteredSegment {
    pub segment_index: usize,
    pub kind: DomainKindOfARegisteredSegment,
    /// `slice` 的段判到的段内编号（[0, 它) 的块都判完了）；`full` 的段是整域。
    pub judged_through: StateCount1280Bits,
    pub domain_state_count: StateCount1280Bits,
    /// `slice` 的段一片几个态（这个进程的片长，[`registered_segment_slice_states_from_the_environment`]）：按片记账照它数；`full` 的段不用它。
    pub states_per_slice: u64,
}

/// 判段：这一趟登记了的段各自的域与进度。分片的段照计划里的片（录入段定的、从库里读回的流带着）与库里已判的块数：
/// 从片起点起连着判完的块（[`judged_through_of_the_sliced_segments`]）；不分片的登记段是 `full`。`states_per_slice` 是这个进程的片长
/// （分片的段按片记账照它数）。
///
/// # Errors
/// 库读失败。
pub fn domains_and_progress_of_the_registered_segments(
    store: &VerdictStore,
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    registered_segments: &[RegisteredSegment],
    conclusions: &ConclusionsStoredUnder<'_>,
    states_per_slice: u64,
) -> Result<Vec<DomainAndProgressOfARegisteredSegment>, VerdictStoreError> {
    let judged_through_of_the_slices =
        judged_through_of_the_sliced_segments(store, plan, conclusions)?;
    let tearable = TearableInPlaceOverwrites::of(flow.base, flow.writes);
    Ok(registered_segments
        .iter()
        .map(|registered| {
            let segment_index = registered.segment_index;
            let sliced =
                plan.sliced_segments()
                    .into_iter()
                    .find_map(|(_point_index, piece)| match piece.kind {
                        KindOfAPiece::SlicedSegment {
                            segment_index: piece_segment,
                            domain_state_count,
                            ..
                        } if piece_segment == segment_index => Some(domain_state_count),
                        KindOfAPiece::SlicedSegment { .. }
                        | KindOfAPiece::SegmentsNotSliced { .. } => None,
                    });
            match sliced {
                Some(domain_state_count) => DomainAndProgressOfARegisteredSegment {
                    segment_index,
                    kind: DomainKindOfARegisteredSegment::Slice,
                    judged_through: judged_through_of_the_slices
                        .iter()
                        .find(|(judged_segment, _, _)| *judged_segment == segment_index)
                        .map_or(StateCount1280Bits::ZERO, |(_, judged_through, _)| {
                            *judged_through
                        }),
                    domain_state_count,
                    states_per_slice,
                },
                None => {
                    let segment = &flow.segments[segment_index];
                    let domain_state_count = plan
                        .segment_expansion(flow, segment_index, segment)
                        .checked_state_count(flow.writes, segment, &tearable)
                        .expect("不分片的登记段整域装得进 u64：录入段核过");
                    DomainAndProgressOfARegisteredSegment {
                        segment_index,
                        kind: DomainKindOfARegisteredSegment::Full,
                        judged_through: domain_state_count,
                        domain_state_count,
                        states_per_slice,
                    }
                }
            }
        })
        .collect())
}
/// 同 [`domains_and_progress_of_the_registered_segments`]；登了分片段的流照区间判的那一趟（`part_of_the_run` 不是 `None`）片起点由区间定，
/// 从片起点数的不是「从编号 0 起连着判完的」，分片的段判到哪换成库里的续点（[`resume_point_of_the_sliced_segment`]，只读库）。
///
/// # Errors
/// 库读失败。
pub(super) fn domains_and_progress_of_the_registered_segments_of_the_run(
    store: &VerdictStore,
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    registered_segments: &[RegisteredSegment],
    conclusions: &ConclusionsStoredUnder<'_>,
    states_per_slice: u64,
    part_of_the_run: Option<&PartOfASlicedFlowSelected>,
) -> Result<Vec<DomainAndProgressOfARegisteredSegment>, VerdictStoreError> {
    let mut domains_and_progress = domains_and_progress_of_the_registered_segments(
        store,
        flow,
        plan,
        registered_segments,
        conclusions,
        states_per_slice,
    )?;
    if part_of_the_run.is_none() {
        return Ok(domains_and_progress);
    }
    for domain_and_progress in &mut domains_and_progress {
        match domain_and_progress.kind {
            DomainKindOfARegisteredSegment::Full => {}
            DomainKindOfARegisteredSegment::Slice => {
                domain_and_progress.judged_through = resume_point_of_the_sliced_segment(
                    store,
                    plan,
                    domain_and_progress.segment_index,
                    &domain_and_progress.domain_state_count,
                    conclusions,
                )?
                .judged_through;
            }
        }
    }
    Ok(domains_and_progress)
}

/// 按片记账里的片数：装得进 u64 写精确的十进制；装不进写下界 `at_least_<2^64 的十进制>`（D13（验证路线） 已定项 9 那一段：
/// 片数装不进 u64 的写成下界）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SliceCountOfASlicedSegment {
    Exactly(u64),
    AtLeastTwoToTheSixtyFour,
}

impl SliceCountOfASlicedSegment {
    /// `states` 个态按一片 `states_per_slice` 个切成几片：`rounding` 定末尾不满一片的那一截算不算一片。
    fn of_the_states(
        states: &StateCount1280Bits,
        states_per_slice: u64,
        rounding: RoundingOfThePartialSlice,
    ) -> Self {
        let (whole_slices, states_left_over) = states.divided_by_u64(states_per_slice);
        let slices = match rounding {
            RoundingOfThePartialSlice::CountedAsASlice if states_left_over > 0 => {
                whole_slices.checked_add_u64(1)
            }
            RoundingOfThePartialSlice::CountedAsASlice | RoundingOfThePartialSlice::LeftOut => {
                Some(whole_slices)
            }
        };
        match slices.and_then(|slices| slices.to_u64()) {
            Some(slices) => Self::Exactly(slices),
            None => Self::AtLeastTwoToTheSixtyFour,
        }
    }

    /// 驱动行里的写法。
    #[must_use]
    pub fn text(self) -> String {
        match self {
            Self::Exactly(slices) => slices.to_string(),
            Self::AtLeastTwoToTheSixtyFour => format!("at_least_{}", u128::from(u64::MAX) + 1),
        }
    }
}

/// 数片的时候末尾不满一片的那一截：共几片要算上它（整域的末片可以短），判完几片不算（没判完的片不算判过）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RoundingOfThePartialSlice {
    CountedAsASlice,
    LeftOut,
}

/// 分片的段到这一趟末还剩什么：整域判完了，或余下的这一趟没跑（这个节点不算判过）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RestOfASlicedSegment {
    EveryStateJudged,
    NotRunThisTime,
}

impl RestOfASlicedSegment {
    /// 驱动行里的写法。
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::EveryStateJudged => "every_state_judged",
            Self::NotRunThisTime => "rest_not_run_this_time",
        }
    }
}

/// 分片的段按片记的账（驱动行 `registered_segment_slices=` 的一项，D13（验证路线） 已定项 9「枚举不完靠精确剪枝，一时跑不了的节点标 ignore 慢慢开」
/// 那一段：照旧分片、每趟接着判下一片，如实报共几片、到这一趟累计判完几片，余下的报本次未跑，这个节点不算判过）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SliceAccountOfASlicedSegment {
    pub judged_slices: SliceCountOfASlicedSegment,
    pub slices: SliceCountOfASlicedSegment,
    pub rest: RestOfASlicedSegment,
}

impl SliceAccountOfASlicedSegment {
    /// 驱动行里的写法：`<段号>:<累计判完几片>/<共几片>:<every_state_judged|rest_not_run_this_time>`。
    #[must_use]
    pub fn field_text(&self, segment_index: usize) -> String {
        format!(
            "{segment_index}:{}/{}:{}",
            self.judged_slices.text(),
            self.slices.text(),
            self.rest.name()
        )
    }
}

/// 一段分片的段到这一趟末的按片记账：整域按一片 `states_per_slice` 个态切成几片（末片可以短）；从编号 0 起连着判完的 `judged_through`
/// 个态里有几片整片判完了；`judged_through` 到了整域末尾才是判完了，没到的余下这一趟没跑。
///
/// # Panics
/// `states_per_slice` 是 0。
#[must_use]
pub fn slice_account_of_a_sliced_segment(
    judged_through: &StateCount1280Bits,
    domain_state_count: &StateCount1280Bits,
    states_per_slice: u64,
) -> SliceAccountOfASlicedSegment {
    let slices = SliceCountOfASlicedSegment::of_the_states(
        domain_state_count,
        states_per_slice,
        RoundingOfThePartialSlice::CountedAsASlice,
    );
    let rest = if *judged_through == *domain_state_count {
        RestOfASlicedSegment::EveryStateJudged
    } else {
        RestOfASlicedSegment::NotRunThisTime
    };
    let judged_slices = match rest {
        RestOfASlicedSegment::EveryStateJudged => slices,
        RestOfASlicedSegment::NotRunThisTime => SliceCountOfASlicedSegment::of_the_states(
            judged_through,
            states_per_slice,
            RoundingOfThePartialSlice::LeftOut,
        ),
    };
    SliceAccountOfASlicedSegment {
        judged_slices,
        slices,
        rest,
    }
}

/// 一段的段号与写数：驱动行 `registered_segments=` 与 `unregistered_segments_beyond_expansion=` 两个字段的一项。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SegmentWithItsWriteCount {
    pub segment_index: usize,
    pub write_count: usize,
}

/// 这条流里没登记、而且展开上限盖不住（[`ExpandUpToWritesPerSegment::expansion_of`] 判 [`CrashPointReplaySegmentExpansion::NotExpanded`]）的段，
/// 按段号升序：登了段的流照旧报 `exhaustive=false`，入口靠它分得出「登记段之外还有展不开的段」（C790 阻断二）。
#[must_use]
pub fn unregistered_segments_beyond_the_expansion(
    segments: &[Vec<usize>],
    registered_segments: &[RegisteredSegment],
    expand_up_to: ExpandUpToWritesPerSegment,
) -> Vec<SegmentWithItsWriteCount> {
    segments
        .iter()
        .enumerate()
        .filter(|(segment_index, segment)| {
            !registered_segments
                .iter()
                .any(|registered| registered.segment_index == *segment_index)
                && expand_up_to.expansion_of(segment)
                    == CrashPointReplaySegmentExpansion::NotExpanded
        })
        .map(|(segment_index, segment)| SegmentWithItsWriteCount {
            segment_index,
            write_count: segment.len(),
        })
        .collect()
}

/// 驱动行末尾说段的五个字段：` registered_segments=<段号>:<写数>[,…] unregistered_segments_beyond_expansion=<段号>:<写数>[,…]
/// registered_segment_domains=<段号>:<full|slice>[,…] registered_segment_progress=<段号>:<已判态数>/<域的总态数>[,…]
/// registered_segment_slices=<段号>:<累计判完几片>/<共几片>:<every_state_judged|rest_not_run_this_time>[,…]`，
/// 一段都没有的写 `-`（[`reuse_and_scale_fields_of_the_line`]）。`registered_segment_progress` 只列 `slice` 的段，两个数都是精确的十进制
/// （1280 位的 [`StateCount1280Bits`]，不写浮点、不写近似、不饱和）；`registered_segment_slices` 同样只列 `slice` 的段
/// （[`slice_account_of_a_sliced_segment`]：片数装不进 u64 写下界，余下没判的写 `rest_not_run_this_time`，那一段这一趟不算判过）。
#[must_use]
pub fn segment_fields_of_the_line(
    registered_segments: &[RegisteredSegment],
    unregistered_segments_beyond_the_expansion: &[SegmentWithItsWriteCount],
    registered_segment_domains: &[DomainAndProgressOfARegisteredSegment],
) -> String {
    let field_text = |segments: Vec<SegmentWithItsWriteCount>| {
        if segments.is_empty() {
            "-".to_string()
        } else {
            segments
                .iter()
                .map(|segment| format!("{}:{}", segment.segment_index, segment.write_count))
                .collect::<Vec<String>>()
                .join(",")
        }
    };
    let registered = field_text(
        registered_segments
            .iter()
            .map(|registered| SegmentWithItsWriteCount {
                segment_index: registered.segment_index,
                write_count: registered.write_count,
            })
            .collect(),
    );
    let unregistered_beyond = field_text(unregistered_segments_beyond_the_expansion.to_vec());
    let or_a_dash = |items: Vec<String>| {
        if items.is_empty() {
            "-".to_string()
        } else {
            items.join(",")
        }
    };
    let domains = or_a_dash(
        registered_segment_domains
            .iter()
            .map(|domain| format!("{}:{}", domain.segment_index, domain.kind.name()))
            .collect(),
    );
    let progress = or_a_dash(
        registered_segment_domains
            .iter()
            .filter(|domain| domain.kind == DomainKindOfARegisteredSegment::Slice)
            .map(|domain| {
                format!(
                    "{}:{}/{}",
                    domain.segment_index,
                    domain.judged_through.decimal_text(),
                    domain.domain_state_count.decimal_text()
                )
            })
            .collect(),
    );
    let slices = or_a_dash(
        registered_segment_domains
            .iter()
            .filter(|domain| domain.kind == DomainKindOfARegisteredSegment::Slice)
            .map(|domain| {
                slice_account_of_a_sliced_segment(
                    &domain.judged_through,
                    &domain.domain_state_count,
                    domain.states_per_slice,
                )
                .field_text(domain.segment_index)
            })
            .collect(),
    );
    format!(
        " registered_segments={registered} unregistered_segments_beyond_expansion={unregistered_beyond} registered_segment_domains={domains} registered_segment_progress={progress} registered_segment_slices={slices}"
    )
}

#[cfg(test)]
mod tests;
