//! 判法单元与结论版本：每个点的结论存在哪一把版本键下（[`ConclusionVersionsOfThePlan`]：由结论种类与判器引到的被检测产品代码定，判器的代码不进），
//! 流水线这一趟挂着的观察者里没登记或拒算的记在占位键下（[`refused_judging_units_of_the_run`]）。

use super::*;

// ============================== 结论的版本、对照与「根槽已持久」 ==============================

/// 这一趟判第 ① 段的是哪一个判器（配了 GPU 是 GPU 判器，没配是 CPU 判器）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JudgedBy {
    GpuJudge,
    CpuJudge,
}

impl JudgedBy {
    /// 两种判器判出的都是同一种结论（数据不与设备绑定）：在哪判的只进块属性。
    #[must_use]
    pub fn conclusion_kind(self) -> ConclusionKind {
        match self {
            Self::GpuJudge | Self::CpuJudge => ConclusionKind::ImageJudgement,
        }
    }
}

/// 这个进程的标记：判法单元拒算的那几组的占位按它取，几个进程、几趟之间撞不上（进程号、此刻的时刻、它自己的地址拼起来取 SHA-256）。
pub(super) fn marker_of_this_process() -> &'static [u8; DIGEST_BYTES] {
    static MARKER: std::sync::OnceLock<[u8; DIGEST_BYTES]> = std::sync::OnceLock::new();
    MARKER.get_or_init(|| {
        let since_the_epoch = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| since.as_nanos());
        sha256_digest(
            format!(
                "singlefs process marker\nprocess={}\nnanoseconds={since_the_epoch}\naddress={:p}\n",
                std::process::id(),
                &MARKER
            )
            .as_bytes(),
        )
    })
}

/// 拒算的那一组（没登记或编译时拒算的观察者、拒算的判器引到的被检测产品代码那一组）这一趟用的占位：指纹按组名与这个进程的标记取（几组各一把、几个进程各一把，撞不上登记了的观察者的判定项标识，
/// 也撞不上早先哪一趟的），组成清单是空的。照它存的行照判、不入库：调用方把它与从它拼出来的版本登记成照判不入库
/// （`VerdictStore::keep_the_rows_of_these_versions_out_of_the_library`），别的组照键入库同一份库。
#[must_use]
pub fn placeholder_judging_units_key_of_the_refused(units: &JudgingUnits) -> JudgingUnitsKey {
    placeholder_judging_units_key_of_the_refused_under_the_process_marker(
        units,
        marker_of_this_process(),
    )
}

/// 同 [`placeholder_judging_units_key_of_the_refused`]，进程的标记由调用方给（单测拿两个标记造录入与判段各在一个进程里的样子）。
pub(super) fn placeholder_judging_units_key_of_the_refused_under_the_process_marker(
    units: &JudgingUnits,
    process_marker: &[u8; DIGEST_BYTES],
) -> JudgingUnitsKey {
    let mut message = b"singlefs judging units refused in this process\n".to_vec();
    message.extend_from_slice(process_marker);
    message.extend_from_slice(format!("\ngroup={}\n", units.group_name()).as_bytes());
    JudgingUnitsKey::of_the_parts(sha256_digest(&message), Vec::new())
}

/// 流水线那一趟判法单元拒算的那两行的头（[`refused_judging_units_of_the_run`]）：一个没登记或拒算的观察者一行，点名它那一组（组名）与形态，再一行
/// `refused_blocks=<这条流的块数>`。打了这一行的那一趟照做、那几个照判不入库，别的照键入库同一份库。
pub const JUDGING_UNITS_REFUSED_LINE_PREFIX: &str = "CRASH_AMPLIFICATION_JUDGING_UNITS_REFUSED";

/// 流水线这一趟挂着的观察者里没登记、或编译时它那一组拒算的那几个（编进二进制的那一份表，不读树；[`judging_units_needed_by_the_run`]、
/// [`observer_product_code_key_in`]）。判器、核对器与观察者结论拼块自己的代码不进任何键，不在这里认。
#[must_use]
pub fn refused_judging_units_of_the_run(
    observers: &[ObserverOnTheNodes<'_>],
) -> Vec<JudgingUnitsKeyRefused> {
    refused_judging_units_among(&judging_units_needed_by_the_run(observers), |units| {
        observer_product_code_key_in(
            units,
            ObserverJudgingUnitsKeyTable::OfTheVerificationPackage,
        )
        .map(|_product_code| ())
    })
}

/// `needed` 里哪几组没登记或拒算（由 `registration_of_the_units` 认，单测造它拒算）。
fn refused_judging_units_among(
    needed: &[JudgingUnits],
    registration_of_the_units: impl Fn(&JudgingUnits) -> Result<(), JudgingUnitsKeyRefused>,
) -> Vec<JudgingUnitsKeyRefused> {
    needed
        .iter()
        .filter_map(|units| registration_of_the_units(units).err())
        .collect()
}

/// 流水线这一趟要认登没登记、拒没拒算的几组：挂着的每个观察者那一组（它的判定项标识）。判镜像的定义、GPU 判器、两种核对器与
/// 观察者结论拼块都不进键（结论的版本由结论种类与判器引到的被检测产品代码定，[`ConclusionVersionsOfThePlan`]；核对键只由事实表定，[`facts_verifier_row_key`]），
/// 这里不认。
#[must_use]
fn judging_units_needed_by_the_run(observers: &[ObserverOnTheNodes<'_>]) -> Vec<JudgingUnits> {
    observers
        .iter()
        .map(|observer| JudgingUnits::ObserverCode {
            observer_code_file: observer.observer_code_file.clone(),
            observer_name: observer.observer_name.as_str().to_string(),
        })
        .collect()
}

/// 这一趟每个点的镜像判定记在哪一把版本下：判崩溃镜像那一种结论的版本（[`ConclusionVersion::of_the_crash_image_judgement`]），
/// 由结论种类与判器引到的被检测产品代码定，判器的代码（CPU 判器、GPU 判器、池级 checker、记录核对器）不进：改判器不作废已判的块，判法改了对不对靠对拍与复用哨兵抓。
/// 每个点同一把（设点文件不进键），与计划的点同序（连同末节点）。GPU 判的与 CPU 判的都记在这一把下（数据不与设备绑定）；
/// 在哪判的、卡型与驱动只进块属性当出处。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConclusionVersionsOfThePlan {
    pub judgement_of_each_point: Vec<ConclusionVersion>,
}

impl ConclusionVersionsOfThePlan {
    /// 计划里每个点都记在判崩溃镜像那一种结论的版本下（两条路线同一把，不取任何一组判法单元）。
    #[must_use]
    pub fn of_the_plan(plan: &CrashPlan) -> Self {
        Self {
            judgement_of_each_point: vec![
                ConclusionVersion::of_the_crash_image_judgement();
                plan.points.len()
            ],
        }
    }

    /// 这一种判器判出的结论记在哪一处：都是镜像判定、同一把版本。
    #[must_use]
    pub fn stored_under(&self, judged_by: JudgedBy) -> ConclusionsStoredUnder<'_> {
        ConclusionsStoredUnder::TheConclusionKind {
            kind: judged_by.conclusion_kind(),
            version_of_each_point: self.versions_of_each_point_when_judged_by(judged_by),
        }
    }

    /// 这一种判器判出的结论每个点记在哪一把版本下（与 [`Self::stored_under`] 同一个取法）：两条路线同一把。
    #[must_use]
    pub fn versions_of_each_point_when_judged_by(
        &self,
        _judged_by_either_judge: JudgedBy,
    ) -> &[ConclusionVersion] {
        &self.judgement_of_each_point
    }
}

/// 「根槽已持久」的计数事后数（的 M2-2 甲、J7）：这条流按这份计划展开的状态里，写表第 `root_slot_write` 次写（一条根槽写）
/// 持久的有几个。不进块键、不进库：各项按自己的被判根现算。按段的序号区间数：这一次写所在的段之前一个都不持久；之后的段与最后全部持久
/// 那一个都持久；所在那一段展开了的，组合数是状态数加一（真子集之外的那一个全落），根槽写两态、持久的占一半，去掉全落那一个。
///
/// # Panics
/// `root_slot_write` 不是根槽写（只有根槽写两态、在甲二里也逐位拆，这里的数法只对它成立）；它不在任何一段里。
#[must_use]
pub fn states_with_the_root_slot_write_persisted(
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    root_slot_write: usize,
) -> u64 {
    assert_eq!(
        flow.writes[root_slot_write].kind,
        StepKind::RootRecordFua,
        "第 {root_slot_write} 次写不是根槽写：这里只按根槽写两态数"
    );
    let segment_of_the_write = flow
        .segments
        .iter()
        .position(|segment| segment.contains(&root_slot_write))
        .expect("写表里的每次写都在某一段里");
    // 分片的段里根槽写只占片里的一部分，这里按「组合数的一半」数不成立；崩溃枚举用例（crash_state_enumeration、tests/ 里的调用方）的计划
    // 都不分片，只有 run_from_environment 排出分片的计划，它不调这里
    assert!(
        plan.segment_slices()[segment_of_the_write].is_none(),
        "第 {root_slot_write} 次写所在的第 {segment_of_the_write} 段分了片：按段数「根槽已持久」只对不分片的段成立"
    );
    let expansion = |segment_index: usize, segment: &[usize]| {
        plan.segment_expansion(flow, segment_index, segment)
    };
    let (ranges_by_segment, state_count) = crash_state_ranges_by_segment_with_slices(
        flow.base,
        flow.writes,
        flow.segments,
        &expansion,
        plan.segment_slices(),
    );
    let range = &ranges_by_segment[segment_of_the_write];
    let states_of_the_segment = range.end - range.start;
    let persisted_inside_the_segment = if states_of_the_segment == 0 {
        0
    } else {
        // 组合数 = 状态数 + 1（偶数：根槽写两态）；根槽写持久的占一半，去掉全落那一个
        states_of_the_segment.div_ceil(2) - 1
    };
    // 之后的段与最后全部持久那一个：整条流的状态数减去到这一段为止的
    persisted_inside_the_segment + (state_count - range.end)
}

#[cfg(test)]
mod tests;
