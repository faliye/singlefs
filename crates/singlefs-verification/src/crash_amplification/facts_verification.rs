//! 按事实核对：核对器的行键、按事实核一块（[`verify_blocks_from_facts`]）、判器与核对器对比，连同读还没判的块与读违例。

use super::*;

// ============================== 三段流的第 ②、③ 段：读事实核对，与判器逐状态比 ==============================

/// 第 ② 段在哪算：CPU 参照（`crash_facts::verify_states_from_facts`）或一张卡（`crash_verify_gpu`）。
pub enum FactsVerifierOn {
    Cpu,
    #[cfg(feature = "gpu")]
    Gpu(std::sync::Arc<crate::gpu_unit_checks::GpuCard>),
}

impl FactsVerifierOn {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Cpu => "cpu",
            #[cfg(feature = "gpu")]
            Self::Gpu(_) => "gpu",
        }
    }
}

/// 第 ② 段的结局：在哪核的（`cpu` / `gpu`）与核了多少。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VerificationOutcome {
    pub verifier: &'static str,
    pub tally: VerificationTally,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VerificationTally {
    pub blocks_verified: u64,
    pub blocks_already_verified: u64,
    pub states_verified: u64,
    pub verifier_red_states: u64,
}

/// 核对行的键（核对表里块键之后那 32 字节，L8）：这一份事实表的摘要带标签取 SHA-256，只由核的输入数据定。
/// 事实表改一个字节，键就换、那一块现核；核对代码（CPU 参照与 GPU 内核都是）不进键，CPU 核的与 GPU 核的同一份事实表同一把键，
/// 改核对代码不作废已核的块，核对对不对靠两边对拍与复用哨兵抓。
#[must_use]
pub fn facts_verifier_row_key(facts_digest: &[u8; DIGEST_BYTES]) -> [u8; DIGEST_BYTES] {
    let mut message = b"singlefs facts verifier row\n".to_vec();
    message.extend_from_slice(facts_digest);
    sha256_digest(&message)
}

/// 一条流照这份计划抽的事实表的核对键（[`facts_verifier_row_key`]；单测读核对行、比判器与核对时用）。
#[must_use]
pub fn facts_verifier_row_key_of_the_plan(
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
) -> [u8; DIGEST_BYTES] {
    let (facts, _segment_expansions) = facts_and_segment_expansions_of_the_plan(flow, plan);
    facts_verifier_row_key(&facts.digest())
}

/// 第 ① 段的事实进库、第 ② 段按块核：`is_mine` 选这台机器领的块（与 [`check_blocks_where`] 同一套分工），
/// 库里同一把核对键（[`facts_verifier_row_key`]）下核过的块不再核。事实表按流抽一次、上卡一次。
///
/// # Errors
/// 库读写失败；GPU 内核起不来。
pub fn verify_blocks_from_facts(
    store: &mut VerdictStore,
    flow: &FlowOfTheLibrary<'_>,
    plan: &CrashPlan,
    is_mine: &dyn Fn(u64) -> bool,
    on: &FactsVerifierOn,
) -> Result<VerificationTally, String> {
    let (facts, segment_expansions) = facts_and_segment_expansions_of_the_plan(flow, plan);
    // 事实表先入库、再读库，核对只拿读回的那一份
    let facts = crate::crash_library::record_the_facts_and_read_them_back(store, &facts)?;
    verify_blocks_from_these_facts(
        &LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(store),
        plan,
        &facts,
        &segment_expansions,
        is_mine,
        on,
    )
}

/// 每一段照这份计划的拆法（第 ② 段两种核对都照它拆序号）：判段在库里读回的流程上照计划现算。
pub(super) fn segment_expansions_of_the_plan(
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
) -> Vec<SegmentExpansionFact> {
    let expansion = |segment_index: usize, segment: &[usize]| {
        plan.segment_expansion(flow, segment_index, segment)
    };
    segment_expansion_facts(flow.writes, flow.segments, &expansion)
}

/// 一条流照这份计划的展开抽的事实表，与每一段的拆法（第 ② 段两种核对都照它拆序号；G 那一路交给引擎的「事实表带上段的展开方式」）：
/// 展开取 [`CrashPlan::segment_expansion`]，ignore 的点的段与计划一样不展开，事实表的序号区间与计划的块对得上。
pub(super) fn facts_and_segment_expansions_of_the_plan(
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
) -> (FlowFacts, Vec<SegmentExpansionFact>) {
    let expansion = |segment_index: usize, segment: &[usize]| {
        plan.segment_expansion(flow, segment_index, segment)
    };
    (
        extract_flow_facts_with_slices(
            flow.base,
            flow.writes,
            flow.segments,
            &expansion,
            plan.segment_slices(),
        ),
        segment_expansion_facts(flow.writes, flow.segments, &expansion),
    )
}

/// 同 [`verify_blocks_from_facts`]，事实表由调用方给（流水线判段给的是从库里读回的那一份）；核对行写进库、带事实表摘要。
/// 库每一次读写现取现放：核一块的时候手里不握锁。
///
/// # Errors
/// 同 [`verify_blocks_from_facts`]。
pub(super) fn verify_blocks_from_these_facts(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    plan: &CrashPlan,
    facts: &crate::crash_facts::FlowFactsReadBack,
    segment_expansions: &[SegmentExpansionFact],
    is_mine: &dyn Fn(u64) -> bool,
    on: &FactsVerifierOn,
) -> Result<VerificationTally, String> {
    let facts_digest = facts.digest();
    let verifier_digest = facts_verifier_row_key(&facts_digest);
    let verifier = BlockVerifier::new(on, facts, segment_expansions);
    let mut tally = VerificationTally::default();
    let mut block_sequence = 0u64;
    for point in &plan.points {
        for local in point.blocks() {
            let sequence = block_sequence;
            block_sequence += 1;
            if !is_mine(sequence) {
                continue;
            }
            let key = point.block_key(&local);
            if library
                .take()
                .read_verifier_block(&key, &verifier_digest)
                .map_err(|error| format!("核对表读失败：{error:?}"))?
                .is_some()
            {
                tally.blocks_already_verified += 1;
                continue;
            }
            let ordinals = point.ordinals.start + local.start..point.ordinals.start + local.end;
            let bits = verifier.verify(facts, segment_expansions, ordinals);
            tally.verifier_red_states += bits.iter().filter(|value| **value != 0).count() as u64;
            tally.states_verified += bits.len() as u64;
            tally.blocks_verified += 1;
            library
                .take()
                .store_verifier_block(&key, &verifier_digest, &facts_digest, &bits)
                .map_err(|error| format!("核对表写不进库：{error:?}"))?;
        }
    }
    Ok(tally)
}

/// 一条流在某一处核对：CPU 参照不用准备；GPU 要先把事实表上卡。
enum BlockVerifier<'card> {
    Cpu,
    #[cfg(feature = "gpu")]
    Gpu(Box<crate::crash_verify_gpu::GpuFactsVerifier<'card>>),
    #[cfg(not(feature = "gpu"))]
    #[expect(dead_code, reason = "没编进 gpu 时只替生命周期参数占位，从不建出")]
    Never(std::marker::PhantomData<&'card ()>),
}

impl<'card> BlockVerifier<'card> {
    fn new(
        on: &'card FactsVerifierOn,
        facts: &crate::crash_facts::FlowFactsReadBack,
        segment_expansions: &[SegmentExpansionFact],
    ) -> Self {
        match on {
            FactsVerifierOn::Cpu => {
                let _ = (facts, segment_expansions);
                Self::Cpu
            }
            #[cfg(feature = "gpu")]
            FactsVerifierOn::Gpu(card) => Self::Gpu(Box::new(
                crate::crash_verify_gpu::GpuFactsVerifier::new_by_the_segment_expansions(
                    card,
                    facts,
                    segment_expansions,
                ),
            )),
        }
    }

    fn verify(
        &self,
        facts: &crate::crash_facts::FlowFactsReadBack,
        segment_expansions: &[SegmentExpansionFact],
        ordinals: Range<u64>,
    ) -> Vec<u32> {
        match self {
            Self::Cpu => verify_states_from_facts_by_the_segment_expansions(
                facts,
                segment_expansions,
                ordinals,
            ),
            #[cfg(feature = "gpu")]
            Self::Gpu(verifier) => verifier.verify_states(ordinals),
            #[cfg(not(feature = "gpu"))]
            Self::Never(_) => unreachable!("没编进 gpu 特性就不会建出这一支"),
        }
    }
}

/// 第 ③ 段：核对表与判定块表逐状态比。核对红 ⇒ 判器也红，不然是不一致（核对那一侧错了或判器漏了，都要查）；
/// 判器红而核对绿是核对没覆盖到的那一类，只计数。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct JudgeVerifierComparison {
    pub states_compared: u64,
    pub both_green: u64,
    pub both_red: u64,
    /// 核对红、判器绿的（路径，点内序号，结论位）——最多留 [`RED_TEXTS_KEPT`] 条，计数在 `disagreements`。
    pub disagreement_samples: Vec<(String, u64, u32)>,
    pub disagreements: u64,
    pub judge_red_only: u64,
    /// 判器判过而这一版核对代码没核的块（第 ② 段还欠的）。
    pub blocks_without_verifier: u64,
    /// 这一版核对代码核过而判器没判的块。
    pub blocks_without_judge: u64,
}

/// # Errors
/// 库读失败。
pub fn compare_judge_and_verifier(
    store: &VerdictStore,
    plan: &CrashPlan,
    verifier_digest: &[u8; DIGEST_BYTES],
    version: &JudgingUnitsFingerprint,
) -> Result<JudgeVerifierComparison, VerdictStoreError> {
    compare_the_conclusions_and_the_verifier(
        store,
        plan,
        verifier_digest,
        &ConclusionsStoredUnder::TheJudgingUnitsFingerprint(version),
    )
}

/// 同 [`compare_judge_and_verifier`]，判器那一侧是 `conclusions` 那一处的结论。
///
/// # Errors
/// 库读失败。
pub fn compare_the_conclusions_and_the_verifier(
    store: &VerdictStore,
    plan: &CrashPlan,
    verifier_digest: &[u8; DIGEST_BYTES],
    conclusions: &ConclusionsStoredUnder<'_>,
) -> Result<JudgeVerifierComparison, VerdictStoreError> {
    compare_the_conclusions_and_the_verifier_of_the_blocks(
        store,
        plan,
        verifier_digest,
        conclusions,
        &|_| true,
    )
}

/// 同 [`compare_the_conclusions_and_the_verifier`]，只比 `is_compared` 判为要比的块（块序号 = 全部点的块按次序编号，
/// 与 [`pending_blocks`] 的 `is_mine` 同一种编号）。按块序号区间选块的那一趟只比这一片选中的块：整条流判过的块随片数涨，
/// 每一片都把整条流重比一遍是片数的平方，而且比的时候持着卡、不打日志。
///
/// # Errors
/// 库读失败。
pub fn compare_the_conclusions_and_the_verifier_of_the_blocks(
    store: &VerdictStore,
    plan: &CrashPlan,
    verifier_digest: &[u8; DIGEST_BYTES],
    conclusions: &ConclusionsStoredUnder<'_>,
    is_compared: &dyn Fn(u64) -> bool,
) -> Result<JudgeVerifierComparison, VerdictStoreError> {
    let mut comparison = JudgeVerifierComparison::default();
    let mut block_sequence = 0u64;
    for (point_index, point) in plan.points.iter().enumerate() {
        for local in point.blocks() {
            let this_block = block_sequence;
            block_sequence += 1;
            if !is_compared(this_block) {
                continue;
            }
            let key = point.block_key(&local);
            let judged = conclusions.verdict_numbers_of_each_state(store, point_index, &key)?;
            let verified = store.read_verifier_block(&key, verifier_digest)?;
            compare_a_block_with_the_verifier(
                store,
                point,
                &local,
                judged.as_deref(),
                verified.as_ref(),
                &mut comparison,
            );
        }
    }
    Ok(comparison)
}

/// 第 ③ 段比一块（[`compare_the_conclusions_and_the_verifier_of_the_blocks`] 与判段收尾那一遍读 [`read_the_judged_conclusions_once_at_the_end`] 共用）：
/// 判器那一份每个状态的判定向量编号（没有结论是 `None`）与核对表那一行（没核是 `None`）逐状态比，记进 `comparison`。
pub(super) fn compare_a_block_with_the_verifier(
    store: &VerdictStore,
    point: &PlannedCrashPoint,
    local: &Range<u64>,
    judged: Option<&[VerdictVectorNumber]>,
    verified: Option<&crate::verdict_store::VerifierBlockRow>,
    comparison: &mut JudgeVerifierComparison,
) {
    let (Some(judged), Some(row)) = (judged, verified) else {
        if judged.is_some() {
            comparison.blocks_without_verifier += 1;
        }
        if verified.is_some() {
            comparison.blocks_without_judge += 1;
        }
        return;
    };
    for (offset, (number, bit)) in judged.iter().zip(&row.bits).enumerate() {
        let judge_red = store
            .verdict_vector_of_number(*number)
            .is_some_and(|vector| !vector.0.is_empty());
        let verifier_red = *bit != 0;
        comparison.states_compared += 1;
        match (judge_red, verifier_red) {
            (false, false) => comparison.both_green += 1,
            (true, true) => comparison.both_red += 1,
            (true, false) => comparison.judge_red_only += 1,
            (false, true) => {
                comparison.disagreements += 1;
                if comparison.disagreement_samples.len() < RED_TEXTS_KEPT {
                    comparison.disagreement_samples.push((
                        point.path_number.clone(),
                        local.start + offset as u64,
                        *bit,
                    ));
                }
            }
        }
    }
}

// ============================== 对账与门禁读 ==============================

/// 录了、在这一判法单元指纹下还没有判定块的块（卡掉线、进程被杀留下的，或判法换了版本之后还没重判的）：一轮收尾按它重新分给活着的一方补跑。
///
/// # Errors
/// 库读失败。
pub fn unjudged_blocks(
    store: &VerdictStore,
    version: &JudgingUnitsFingerprint,
) -> Result<Vec<(VerdictBlockKey, u64)>, VerdictStoreError> {
    let judged: std::collections::HashSet<VerdictBlockKey> = store
        .stored_verdict_block_keys()?
        .into_iter()
        .filter(|(_, stored_version)| stored_version == version)
        .map(|(key, _)| key)
        .collect();
    Ok(store
        .recorded_blocks()?
        .into_iter()
        .filter(|(key, _)| !judged.contains(key))
        .collect())
}

/// 这条流自己的块里，在这一判法单元指纹下还没有判定块的（块键、状态数），按点与块序。一份库装几条流时，一条流跑完对的是它自己的账：
/// 库里别的流录了还没判的块不算它的（[`unjudged_blocks`] 是整份库的账，全部分项跑完之后读）。
///
/// # Errors
/// 库读失败。
pub fn unjudged_blocks_of_the_flow(
    store: &VerdictStore,
    plan: &CrashPlan,
    conclusions: &ConclusionsStoredUnder<'_>,
) -> Result<Vec<(VerdictBlockKey, u64)>, VerdictStoreError> {
    let mut unjudged = Vec::new();
    for (point_index, point) in plan.points.iter().enumerate() {
        for local in point.blocks() {
            let key = point.block_key(&local);
            if !conclusions.is_stored(store, point_index, &key)? {
                unjudged.push((key, local.end - local.start));
            }
        }
    }
    Ok(unjudged)
}

/// 同 [`unjudged_blocks_of_the_flow`]，只数这条流里 `is_counted` 选中的块（块序号 = 全部点的块按次序编号）：给了块区间的那一趟
/// （[`BLOCK_RANGE_ENVIRONMENT_VARIABLE`]）只对本片的账，别的片还没判的块不算这一片没判完。
///
/// # Errors
/// 库读失败。
pub fn unjudged_blocks_of_the_flow_among_the_counted_blocks(
    store: &VerdictStore,
    plan: &CrashPlan,
    conclusions: &ConclusionsStoredUnder<'_>,
    is_counted: &dyn Fn(u64) -> bool,
) -> Result<Vec<(VerdictBlockKey, u64)>, VerdictStoreError> {
    let mut unjudged_among_the_counted = Vec::new();
    let mut block_sequence = 0u64;
    for (point_index, point) in plan.points.iter().enumerate() {
        for local in point.blocks() {
            let this_block = block_sequence;
            block_sequence += 1;
            if !is_counted(this_block) {
                continue;
            }
            let key = point.block_key(&local);
            if !conclusions.is_stored(store, point_index, &key)? {
                unjudged_among_the_counted.push((key, local.end - local.start));
            }
        }
    }
    Ok(unjudged_among_the_counted)
}

/// 库里的一个红状态：节点名、点内序号、判定向量的逐项文字。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredRedState {
    pub path: String,
    pub identity: String,
    pub state_in_point: u64,
    pub items: Vec<String>,
}

/// 这条流每个状态在库里、这一判法单元指纹下的判定（判红的各项；绿的是空的），按整条流的状态序号排；这一判法单元指纹下没判过的块，
/// 它的状态是 `None`。复用与剪枝的对照拿它与不经库现判的结果逐态比。
///
/// # Errors
/// 库读失败。
///
/// # Panics
/// 判定块里的编号没在这个库里登记过，或块里的状态数与计划不符（库坏了）。
pub fn read_verdict_items_of_every_state(
    store: &VerdictStore,
    plan: &CrashPlan,
    version: &JudgingUnitsFingerprint,
) -> Result<Vec<Option<Vec<String>>>, VerdictStoreError> {
    let mut items_of_each_state = Vec::new();
    for point in &plan.points {
        for local in point.blocks() {
            let state_count = usize::try_from(local.end - local.start).expect("一块装得进 usize");
            match store.read_verdict_block(&point.block_key(&local), version)? {
                None => items_of_each_state.extend(std::iter::repeat_n(None, state_count)),
                Some(block) => {
                    let numbers = block.verdict_vector_number_of_each_state();
                    assert_eq!(
                        numbers.len(),
                        state_count,
                        "点 {} 块 {} 存着的状态数与计划不符",
                        point.path_number,
                        local.start
                    );
                    for number in numbers {
                        let vector = store
                            .verdict_vector_of_number(*number)
                            .expect("判定块里的编号在这个库里登记过");
                        items_of_each_state.push(Some(if vector.0.is_empty() {
                            Vec::new()
                        } else {
                            String::from_utf8_lossy(&vector.0)
                                .split('\n')
                                .map(str::to_string)
                                .collect()
                        }));
                    }
                }
            }
        }
    }
    Ok(items_of_each_state)
}

/// 门禁一次读：这些点每一块在这一判法单元指纹下的违例，按点、按序号交回。
///
/// # Errors
/// 库读失败。
pub fn read_violations(
    store: &VerdictStore,
    plan: &CrashPlan,
    version: &JudgingUnitsFingerprint,
) -> Result<Vec<StoredRedState>, VerdictStoreError> {
    read_the_violations_stored_under(
        store,
        plan,
        &ConclusionsStoredUnder::TheJudgingUnitsFingerprint(version),
    )
}

/// 同 [`read_violations`]，读 `conclusions` 那一处的违例。
///
/// # Errors
/// 库读失败。
pub fn read_the_violations_stored_under(
    store: &VerdictStore,
    plan: &CrashPlan,
    conclusions: &ConclusionsStoredUnder<'_>,
) -> Result<Vec<StoredRedState>, VerdictStoreError> {
    read_the_violations_stored_under_among_the_counted_blocks(store, plan, conclusions, &|_| true)
}

/// 同 [`read_the_violations_stored_under`]，只读这条流里 `is_counted` 选中的块（块序号 = 全部点的块按次序编号）：给了块区间的那一趟
/// （[`BLOCK_RANGE_ENVIRONMENT_VARIABLE`]）读本片的违例，本片里复用的块红过照样读到。
///
/// # Errors
/// 库读失败。
pub fn read_the_violations_stored_under_among_the_counted_blocks(
    store: &VerdictStore,
    plan: &CrashPlan,
    conclusions: &ConclusionsStoredUnder<'_>,
    is_counted: &dyn Fn(u64) -> bool,
) -> Result<Vec<StoredRedState>, VerdictStoreError> {
    let mut found = Vec::new();
    let mut block_sequence = 0u64;
    for (point_index, point) in plan.points.iter().enumerate() {
        for local in point.blocks() {
            let this_block = block_sequence;
            block_sequence += 1;
            if !is_counted(this_block) {
                continue;
            }
            let key = point.block_key(&local);
            let violations = conclusions.violations_in_block(store, point_index, &key)?;
            found.extend(red_states_of_a_block(store, point, &local, &violations));
        }
    }
    Ok(found)
}

/// 一块的违例读成库里的红状态（[`read_the_violations_stored_under_among_the_counted_blocks`] 与判段收尾那一遍读
/// [`read_the_judged_conclusions_once_at_the_end`] 共用），按违例的次序。
///
/// # Panics
/// 违例里的编号没在这个库里登记过（库坏了）。
pub(super) fn red_states_of_a_block(
    store: &VerdictStore,
    point: &PlannedCrashPoint,
    local: &Range<u64>,
    violations: &[StoredViolation],
) -> Vec<StoredRedState> {
    violations
        .iter()
        .map(|violation| {
            let vector = store
                .verdict_vector_of_number(violation.verdict_vector_number)
                .expect("违例表里的编号在这个库里登记过");
            StoredRedState {
                path: point.path_number.clone(),
                identity: point.identity.clone(),
                state_in_point: local.start + violation.offset_in_block.0,
                items: String::from_utf8_lossy(&vector.0)
                    .split('\n')
                    .map(str::to_string)
                    .collect(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests;
