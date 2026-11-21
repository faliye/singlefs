//! 复用哨兵：照种子从复用的块里抽几块当哨兵、再抽一块强制现判，判完与库里那一份比（[`plan_the_reuse_sentinels`]、
//! [`ReuseSentinelsOfTheRun`]）。

use super::*;

// ============================== 复用哨兵：块级（`.claude/rules/test-script-discipline.md`「判定可复用：按输入存，按前缀剪」） ==============================

/// 复用哨兵的种子（与 `scripts/gate/lib_reuse_checks.py` 的 `SENTINEL_SEED_VARIABLE` 同名）：十进制非负整数照用，没设或认不出就现取一个；
/// 种子打进 [`REUSE_SENTINELS_LINE_PREFIX`] 那一行，凭它复现这一趟抽中的是哪几块。
pub const REUSE_SENTINEL_SEED_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_REUSE_SENTINEL_SEED";

/// 抽几块当哨兵（与 `scripts/gate/lib_reuse_checks.py` 的 `SENTINEL_COUNT_VARIABLE` 同名）：正整数照用，没设或认不出取 [`DEFAULT_REUSE_SENTINEL_COUNT`]。
pub const REUSE_SENTINEL_COUNT_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_REUSE_SENTINEL_COUNT";

/// 没设抽几块时抽 3 块（与 `scripts/gate/lib_reuse_checks.py` 的 `DEFAULT_SENTINEL_COUNT` 相同）。
pub const DEFAULT_REUSE_SENTINEL_COUNT: usize = 3;

/// 每一趟抽哨兵打的那一行的头：`flow=`、`seed=`、`candidates=`（键相同、库里有不红的结论、归这台机器的块数）、`sentinels=`、`forced=`（块序号，没有是 `-`）。
pub const REUSE_SENTINELS_LINE_PREFIX: &str = "CRASH_AMPLIFICATION_REUSE_SENTINELS";

/// 哨兵抽中的一块：块序号（全部点的块按次序编号）、点、点内的序号段，与它在库里的键（块键、种类、版本）；
/// `observer_keys` 是挂在这个点上的每个观察者在这一块上的那一份结论的键（与 [`ObserverForTheReuseSentinels`] 给的次序相同，不挂这个点的不在里面）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReuseSentinelBlock {
    pub block_sequence: u64,
    pub point_index: usize,
    pub local: Range<u64>,
    pub key: crate::verdict_store::ConclusionOfABlockUnderAKey,
    pub observer_keys: Vec<crate::verdict_store::ConclusionOfABlockUnderAKey>,
}

/// 复用哨兵要核的一个观察者：计划里每个点它挂没挂（与计划的点同序）、它的结论存在哪一把版本下、判段重拼它那一份结论读观察者块表的哪一行。
/// 候选块要连同挂在那个点上的每个观察者都有这一把版本下不红的结论（判段照 `observer_conclusion_is_reused` 复用的那一种）；哨兵核这几份结论
/// 也没被现判，强制现判那一块连这几份一起现判、一起比。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObserverForTheReuseSentinels {
    pub holds_each_point: Vec<bool>,
    pub conclusion: ConclusionVersion,
    /// 判段重拼这一份结论读的那一行（观察者块表的种类与版本）：录入段录的观察者是（观察者，观察者版本）；从判器交回的逐盘自证槽数看的
    /// 是（池级 checker 的种类，它那一把版本）。录入段见观察者结论在、不红就不录那一块（`observer_block_is_recorded_or_concluded`），
    /// 这一行不在库里的块判段重拼不出这一份结论：强制现判抽中它，回录入段只给它重录（[`record_the_observers_of_the_forced_block_again`]）。
    pub input_row: (ConclusionKind, ConclusionVersion),
}

/// 这一趟挂着的观察者交给复用哨兵的样子（[`ObserverForTheReuseSentinels`]），与 `observers` 同序。
///
/// # Panics
/// `versions` 与 `observers` 不一样长。
#[must_use]
pub fn observers_for_the_reuse_sentinels(
    plan: &CrashPlan,
    observers: &[ObserverOnTheNodes<'_>],
    versions: &[ObserverVersions],
) -> Vec<ObserverForTheReuseSentinels> {
    assert_eq!(
        observers.len(),
        versions.len(),
        "每个观察者各一组版本、同序"
    );
    observers
        .iter()
        .zip(versions)
        .map(
            |(observer, observer_versions)| ObserverForTheReuseSentinels {
                holds_each_point: plan
                    .points
                    .iter()
                    .map(|point| {
                        observer
                            .nodes
                            .iter()
                            .any(|node| node.holds_the_point(point))
                    })
                    .collect(),
                conclusion: observer_versions.conclusion,
                input_row: match observer.observe {
                    ObserverCode::CountsOnly(_) | ObserverCode::HandsBackAnImage { .. } => {
                        (ConclusionKind::Observer, observer_versions.observer)
                    }
                    ObserverCode::FromTheSelfVerifiedSystemConfigurationSlotsTheJudgeHandsBack(
                        _,
                    ) => (
                        observer_versions.pool_checker.conclusion_kind(),
                        observer_versions.pool_checker_version,
                    ),
                },
            },
        )
        .collect()
}

/// 挂在计划第 `point_index` 个点上的每个观察者在 `block_key` 那一块上那一份结论的键，与 `observers` 同序（不挂这个点的跳过）。
fn observer_conclusion_keys_of_the_block(
    observers: &[ObserverForTheReuseSentinels],
    point_index: usize,
    block_key: &VerdictBlockKey,
) -> Vec<crate::verdict_store::ConclusionOfABlockUnderAKey> {
    observers
        .iter()
        .filter(|observer| observer.holds_each_point[point_index])
        .map(|observer| {
            (
                block_key.clone(),
                ConclusionKind::Observer,
                observer.conclusion,
            )
        })
        .collect()
}

/// 判段重拼这一块上每个挂着的观察者的结论要读的那一行都在库里（[`ObserverForTheReuseSentinels::input_row`]）；没有观察者挂这个点答在。
///
/// # Errors
/// 库读失败。
pub(super) fn observer_inputs_of_the_block_are_stored(
    store: &VerdictStore,
    observers: &[ObserverForTheReuseSentinels],
    point_index: usize,
    block_key: &VerdictBlockKey,
) -> Result<bool, VerdictStoreError> {
    for observer in observers
        .iter()
        .filter(|observer| observer.holds_each_point[point_index])
    {
        let (input_kind, input_version) = &observer.input_row;
        if store
            .read_observer_block(block_key, input_kind, input_version)?
            .is_none()
        {
            return Ok(false);
        }
    }
    Ok(true)
}

/// 报告里点名一块的块键：库里那几个字节的十六进制（[`VerdictBlockKey::to_block_key_bytes`]，读库程序按它报块）。
pub(super) fn block_key_named_in_the_sentinel_report(block_key: &VerdictBlockKey) -> String {
    hexadecimal_text(&block_key.to_block_key_bytes())
}

/// 一份结论块里 [`StoredConclusionBlock::is_the_same_conclusion_as`] 比的那几样，判定向量编号换成判定向量本身
/// （[`Self::of_the_stored_block`]）；在哪判的、判定日期与跑法是块属性，不在里面。
/// 判定向量编号由 prune-server 统一给（一份库里一种一个号）；跨库（全仓那一份与一次性库各有一套编号）比两份结论照它比，不照编号比。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConclusionSpelledOutInVerdictVectors {
    pub verdict_vector_of_each_state: Vec<EncodedVerdictVector>,
    pub violations: Vec<(StateOffsetInBlock, EncodedVerdictVector)>,
    pub count_vector: BlockCountVector,
    pub recovery_words: Option<Vec<RecoveryWordsOfState>>,
    pub abandoned_roots_named_by_the_pool_checker:
        Option<Vec<Vec<AbandonedRootNamedByThePoolChecker>>>,
}

impl ConclusionSpelledOutInVerdictVectors {
    /// `store` 读回的一份结论块读成判定向量本身：块里与违例里的判定向量编号换成它此刻在 `store` 里登记的那一份
    /// （调用方拿着读回这一块的同一把锁）。
    ///
    /// # Panics
    /// 块里有 `store` 没登记过的编号：存块那一处断言过编号是这个库登记过的，读回来却没有是判定库的 bug。
    #[must_use]
    pub fn of_the_stored_block(store: &VerdictStore, stored: &StoredConclusionBlock) -> Self {
        let StoredConclusionBlock {
            block,
            violations,
            count_vector,
            judged_on: _,
            recovery_words,
            abandoned_roots_named_by_the_pool_checker,
            circumstances: _,
        } = stored;
        let verdict_vector_of = |number: VerdictVectorNumber| {
            store
                .verdict_vector_of_number(number)
                .expect("结论块里的判定向量编号是这个库登记过的（存块那一处断言过）")
                .clone()
        };
        Self {
            verdict_vector_of_each_state: block
                .verdict_vector_number_of_each_state()
                .iter()
                .map(|number| verdict_vector_of(*number))
                .collect(),
            violations: violations
                .iter()
                .map(|violation| {
                    (
                        violation.offset_in_block,
                        verdict_vector_of(violation.verdict_vector_number),
                    )
                })
                .collect(),
            count_vector: count_vector.clone(),
            recovery_words: recovery_words.clone(),
            abandoned_roots_named_by_the_pool_checker: abandoned_roots_named_by_the_pool_checker
                .clone(),
        }
    }

    /// 结论摘要（强制现判凭据里的那一个，`crate::verdict_store::GpuRejudgedRow`）：这几样逐样编成字节、带长度接起来取 SHA-256——
    /// 每个状态的判定向量、违例（块内序号与判定向量）、计数向量（`crate::verdict_store::count_vector_bytes`）、恢复字与点名的被抛弃根
    /// （各带一个在不在的字节，在的照库里那一份的写法）。判定向量是它本身，不是一份库里的编号：服务与判段各读各的库也算得出同一个。
    #[must_use]
    pub fn conclusion_digest(&self) -> [u8; DIGEST_BYTES] {
        let mut message = b"singlefs gpu-rejudged conclusion digest\n".to_vec();
        let put_length = |bytes_to_digest: &mut Vec<u8>, length: usize| {
            bytes_to_digest.extend_from_slice(
                &u64::try_from(length)
                    .expect("一份结论里的长度装得进 u64")
                    .to_be_bytes(),
            );
        };
        put_length(&mut message, self.verdict_vector_of_each_state.len());
        for verdict_vector in &self.verdict_vector_of_each_state {
            put_length(&mut message, verdict_vector.0.len());
            message.extend_from_slice(&verdict_vector.0);
        }
        put_length(&mut message, self.violations.len());
        for (offset, verdict_vector) in &self.violations {
            message.extend_from_slice(&offset.0.to_be_bytes());
            put_length(&mut message, verdict_vector.0.len());
            message.extend_from_slice(&verdict_vector.0);
        }
        let count_vector = crate::verdict_store::count_vector_bytes(&self.count_vector);
        put_length(&mut message, count_vector.len());
        message.extend_from_slice(&count_vector);
        for side_row in [
            self.recovery_words
                .as_deref()
                .map(crate::verdict_store::recovery_words_value_bytes),
            self.abandoned_roots_named_by_the_pool_checker
                .as_deref()
                .map(crate::verdict_store::abandoned_roots_named_value_bytes),
        ] {
            match side_row {
                Some(bytes) => {
                    message.push(0x01);
                    put_length(&mut message, bytes.len());
                    message.extend_from_slice(&bytes);
                }
                None => message.push(0x00),
            }
        }
        sha256_digest(&message)
    }
}

/// 一趟真跑的复用哨兵：开跑前从键相同、库里有不红结论的块里抽几块当哨兵、再从剩下的里抽一块强制现判（抽中的那一块这个进程里问复用一律答不上，
/// `VerdictStore::also_force_these_conclusion_blocks_to_be_judged_again`）；判完照 [`Self::problems_after_judging`] 判：哨兵有一块这一趟交来存过（被现判了）、
/// 强制现判那一块这一趟没交来存（还是复用了）、或它现判出的与库里那一份不是同一个结论（按判定向量本身比，[`ConclusionSpelledOutInVerdictVectors`]），整趟红。
/// 强制现判那一块按复用算（它键相同、只是这一趟被抽去复判：`scripts/gate/lib_reuse_checks.py` 文件头复用三问那一段）。
/// 抽与判可以分在两次拿库的锁里：存的是判定向量本身，不是只在一次拿锁里有效的编号。
#[derive(Clone, Debug)]
pub struct ReuseSentinelsOfTheRun {
    pub seed: u64,
    pub candidate_count: usize,
    pub sentinels: Vec<ReuseSentinelBlock>,
    /// 强制现判的那一块与开跑前库里它那一份结论（判定向量本身）。
    pub forced: Option<(ReuseSentinelBlock, ConclusionSpelledOutInVerdictVectors)>,
    /// 强制现判那一块上每个挂着的观察者那一份结论的键与开跑前库里那一份（判定向量本身），与 `forced` 那一块的 `observer_keys` 同序；
    /// 没抽到强制现判、或那个点上没有观察者是空的。
    pub forced_observer_conclusions: Vec<(
        crate::verdict_store::ConclusionOfABlockUnderAKey,
        ConclusionSpelledOutInVerdictVectors,
    )>,
    /// 开跑前库里交来存过几块（`VerdictStore::conclusion_blocks_offered_so_far`）：判完只看这之后交来的。
    pub(super) offered_before_judging: usize,
}

/// 种子推下一个数（SplitMix64：只用来从候选里挑，不求密码学强度；同一个种子同一串数）。
pub(super) fn next_number_of_the_reuse_sentinel_seed(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut mixed = *state;
    mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    mixed ^ (mixed >> 31)
}

/// 这一趟的种子：[`REUSE_SENTINEL_SEED_ENVIRONMENT_VARIABLE`] 是十进制非负整数照用，否则按此刻的时刻与进程号现取一个。
#[must_use]
pub fn reuse_sentinel_seed_of_this_process() -> u64 {
    if let Some(seed) = std::env::var("SINGLEFS_REUSE_SENTINEL_SEED")
        .ok()
        .and_then(|text| text.parse::<u64>().ok())
    {
        return seed;
    }
    let since_the_epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| since.as_nanos());
    let digest = sha256_digest(
        format!(
            "singlefs reuse sentinel seed\nprocess={}\nnanoseconds={since_the_epoch}\n",
            std::process::id()
        )
        .as_bytes(),
    );
    u64::from_le_bytes(digest[..8].try_into().expect("SHA-256 摘要至少 8 个字节"))
}

/// 这一趟抽几块当哨兵：[`REUSE_SENTINEL_COUNT_ENVIRONMENT_VARIABLE`] 是正整数照用，否则 [`DEFAULT_REUSE_SENTINEL_COUNT`]。
#[must_use]
pub fn reuse_sentinel_count_of_this_process() -> usize {
    std::env::var("SINGLEFS_REUSE_SENTINEL_COUNT")
        .ok()
        .and_then(|text| text.parse::<usize>().ok())
        .filter(|count| *count >= 1)
        .unwrap_or(DEFAULT_REUSE_SENTINEL_COUNT)
}

/// 开跑前抽这一趟的复用哨兵：候选是归这台机器（`is_mine`）、这一处键相同、库里有不红结论的块（[`ConclusionsStoredUnder::is_stored_for_reuse`]
/// 照判段同一个谓词），挂在那个点上的每个观察者（`observers`）在这一块上也都有这一把版本下不红的结论；照 `seed` 抽 `count` 块当哨兵（不足就全抽）、
/// 剩下的里照同一串数抽一块强制现判，把那一块的判器结论与观察者结论都登进库的强制现判表、记下它们此刻在库里的那一份。强制现判那一块上
/// 观察者结论要读的录入段那一行（[`ObserverForTheReuseSentinels::input_row`]）不在库里的照样抽，调用方抽完调
/// [`record_the_observers_of_the_forced_block_again`] 回录入段给它重录。
/// 判法单元指纹那一套（流水线各段单测，不是一趟真跑）不抽，交回 `None`。
///
/// # Errors
/// 库读失败。
pub fn plan_the_reuse_sentinels(
    store: &mut VerdictStore,
    plan: &CrashPlan,
    conclusions: &ConclusionsStoredUnder<'_>,
    is_mine: &dyn Fn(u64) -> bool,
    count: usize,
    seed: u64,
    observers: &[ObserverForTheReuseSentinels],
) -> Result<Option<ReuseSentinelsOfTheRun>, VerdictStoreError> {
    if let ConclusionsStoredUnder::TheJudgingUnitsFingerprint(_) = conclusions {
        return Ok(None);
    }
    let reuse_read = conclusions.reuse_read_of_the_plan(store, plan, is_mine)?;
    plan_the_reuse_sentinels_from_the_reuse_read(
        store,
        plan,
        conclusions,
        &reuse_read,
        is_mine,
        count,
        seed,
        observers,
    )
}

/// 同 [`plan_the_reuse_sentinels`]，候选块判器那一份结论照调用方读回的那一份答（`reuse_read` 读过 `is_mine` 选的每一块；
/// 一趟里录入时数复用的那一份，同一个进程里不再逐块读库），强制现判表现问。候选块上挂着的观察者那几份结论一次成批问
/// （[`VerdictStore::conclusion_blocks_stored_and_red`]，存着不红、不在强制现判表里的才算复用得上，与判段照 `observer_conclusion_is_reused`
/// 复用的同一个谓词：块属性与结论块同一批写），不逐块读结论块。
///
/// # Errors
/// 库读失败。
#[allow(
    clippy::too_many_arguments,
    reason = "与 plan_the_reuse_sentinels 同一组参数，多一份读回的复用"
)]
pub(crate) fn plan_the_reuse_sentinels_from_the_reuse_read(
    store: &mut VerdictStore,
    plan: &CrashPlan,
    conclusions: &ConclusionsStoredUnder<'_>,
    reuse_read: &ReuseOfTheBlocksReadOnce,
    is_mine: &dyn Fn(u64) -> bool,
    count: usize,
    seed: u64,
    observers: &[ObserverForTheReuseSentinels],
) -> Result<Option<ReuseSentinelsOfTheRun>, VerdictStoreError> {
    let (kind, version_of_each_point) = match conclusions {
        ConclusionsStoredUnder::TheJudgingUnitsFingerprint(_) => return Ok(None),
        ConclusionsStoredUnder::TheConclusionKind {
            kind,
            version_of_each_point,
        } => (kind.clone(), *version_of_each_point),
    };
    // 候选照判段的谓词数：同一条连接上别的趟这时抽中强制现判的块问复用答不上，不当候选（强制现判表现问）
    let mut blocks_whose_judge_conclusion_is_reused: Vec<ReuseSentinelBlock> = Vec::new();
    let mut block_sequence = 0u64;
    for (point_index, point) in plan.points.iter().enumerate() {
        for local in point.blocks() {
            let this_block = block_sequence;
            block_sequence += 1;
            if !is_mine(this_block) {
                continue;
            }
            let block_key = point.block_key(&local);
            if reuse_read.is_stored_for_reuse(
                store,
                conclusions,
                point_index,
                &block_key,
                this_block,
            ) {
                let observer_keys =
                    observer_conclusion_keys_of_the_block(observers, point_index, &block_key);
                blocks_whose_judge_conclusion_is_reused.push(ReuseSentinelBlock {
                    block_sequence: this_block,
                    point_index,
                    local,
                    key: (
                        block_key,
                        kind.clone(),
                        ConclusionsStoredUnder::conclusion_version_of_the_point(
                            version_of_each_point,
                            point_index,
                        ),
                    ),
                    observer_keys,
                });
            }
        }
    }
    let observer_conclusions_reused =
        observer_conclusions_reused_of_the_blocks(store, &blocks_whose_judge_conclusion_is_reused)?;
    let candidates: Vec<ReuseSentinelBlock> = blocks_whose_judge_conclusion_is_reused
        .into_iter()
        .zip(observer_conclusions_reused)
        .filter(|(_block, every_observer_conclusion_is_reused)| {
            *every_observer_conclusion_is_reused
        })
        .map(|(block, _every_observer_conclusion_is_reused)| block)
        .collect();
    let candidate_count = candidates.len();
    // 不放回地抽：每抽一块，从还没抽的里照种子推出的数取一个下标
    let mut state = seed;
    let mut remaining = candidates;
    let mut sentinels: Vec<ReuseSentinelBlock> = Vec::new();
    // 迭代上界：count 次，且不超过候选数
    while sentinels.len() < count && !remaining.is_empty() {
        let picked = index_drawn_by_the_reuse_sentinel_seed(&mut state, remaining.len());
        sentinels.push(remaining.swap_remove(picked));
    }
    sentinels.sort_by_key(|sentinel| sentinel.block_sequence);
    // 强制现判照同一串数接着从剩下的里抽一块。判段重拼不出挂着的某个观察者那一份结论的块（录入段那一行不在库里）照样抽：
    // 抽完由调用方回录入段只给它重录（`record_the_observers_of_the_forced_block_again`，不丢掉再抽）
    let forced_block: Option<ReuseSentinelBlock> = if remaining.is_empty() {
        None
    } else {
        let picked = index_drawn_by_the_reuse_sentinel_seed(&mut state, remaining.len());
        Some(remaining.swap_remove(picked))
    };
    let (forced, forced_observer_conclusions) = match forced_block {
        Some(block) => {
            let (block_key, block_kind, block_version) = &block.key;
            let stored = store
                .read_conclusion_block(block_key, block_kind, block_version)?
                .expect("候选都是库里有不红结论的块：is_stored_for_reuse 刚答过");
            let spelled_out =
                ConclusionSpelledOutInVerdictVectors::of_the_stored_block(store, &stored);
            let mut forced_observer_conclusions = Vec::with_capacity(block.observer_keys.len());
            for observer_key in &block.observer_keys {
                let (observer_block_key, observer_kind, observer_version) = observer_key;
                let stored_observer_conclusion = store
                    .read_conclusion_block(observer_block_key, observer_kind, observer_version)?
                    .expect(
                        "候选块上挂着的观察者都有不红的结论：observer_conclusions_reused_of_the_blocks 刚照块属性答过，块属性与结论块同一批写",
                    );
                forced_observer_conclusions.push((
                    observer_key.clone(),
                    ConclusionSpelledOutInVerdictVectors::of_the_stored_block(
                        store,
                        &stored_observer_conclusion,
                    ),
                ));
            }
            store.also_force_these_conclusion_blocks_to_be_judged_again(
                std::iter::once(block.key.clone()).chain(block.observer_keys.iter().cloned()),
            );
            (Some((block, spelled_out)), forced_observer_conclusions)
        }
        None => (None, Vec::new()),
    };
    Ok(Some(ReuseSentinelsOfTheRun {
        seed,
        candidate_count,
        sentinels,
        forced,
        forced_observer_conclusions,
        offered_before_judging: store.conclusion_blocks_offered_so_far(),
    }))
}

/// 照种子推出的下一个数在 `remaining_count` 块里取一个下标。
///
/// # Panics
/// `remaining_count` 是 0（调用方只在还有块时抽）。
fn index_drawn_by_the_reuse_sentinel_seed(state: &mut u64, remaining_count: usize) -> usize {
    assert!(remaining_count > 0, "只在还有块时抽");
    let remaining_count = u64::try_from(remaining_count).expect("块数装得进 u64");
    usize::try_from(next_number_of_the_reuse_sentinel_seed(state) % remaining_count)
        .expect("下标小于候选数，装得进 usize")
}

/// 每一块上挂着的观察者那几份结论（[`ReuseSentinelBlock::observer_keys`]）是不是都在库里、都不红、都不在强制现判表里
/// （判段照 `observer_conclusion_is_reused` 复用的那一种）；一份都没有答是。几块的几份一次成批问存没存、红不红
/// （[`VerdictStore::conclusion_blocks_stored_and_red`]：块属性那一行与违例表；块属性与结论块同一批写，与读回结论块看在不在、有没有违例答的相同），
/// 交回的次序与 `blocks` 相同。
///
/// # Errors
/// 库读失败；块属性那一行不合布局。
fn observer_conclusions_reused_of_the_blocks(
    store: &VerdictStore,
    blocks: &[ReuseSentinelBlock],
) -> Result<Vec<bool>, VerdictStoreError> {
    let observer_conclusions_not_forced: Vec<crate::verdict_store::ConclusionOfABlockUnderAKey> =
        blocks
            .iter()
            .flat_map(|block| block.observer_keys.iter())
            .filter(|(block_key, observer_kind, observer_version)| {
                !store.conclusion_block_is_forced_to_be_judged_again(
                    block_key,
                    observer_kind,
                    observer_version,
                )
            })
            .cloned()
            .collect();
    let mut in_the_library = store
        .conclusion_blocks_stored_and_red(&observer_conclusions_not_forced)?
        .into_iter();
    let mut reused_of_each_block = Vec::with_capacity(blocks.len());
    for block in blocks {
        let mut every_observer_conclusion_is_reused = true;
        for (block_key, observer_kind, observer_version) in &block.observer_keys {
            if store.conclusion_block_is_forced_to_be_judged_again(
                block_key,
                observer_kind,
                observer_version,
            ) {
                every_observer_conclusion_is_reused = false;
                continue;
            }
            let observer_conclusion = in_the_library
                .next()
                .expect("不在强制现判表里的每一份观察者结论各问了一次");
            if observer_conclusion != ConclusionBlockInTheLibrary::StoredGreen {
                every_observer_conclusion_is_reused = false;
            }
        }
        reused_of_each_block.push(every_observer_conclusion_is_reused);
    }
    Ok(reused_of_each_block)
}

impl ReuseSentinelsOfTheRun {
    /// 这一趟用完强制现判那一块：它的判器结论与它上面挂着的观察者结论从库的强制现判表里撤掉，同一条连接上别的趟抽中的照留
    /// （判完由 [`Self::problems_after_judging`] 撤；叫停没判完的那一趟照「没跑到不判」只撤这一份）。
    pub fn stop_forcing_the_forced_block(&self, store: &mut VerdictStore) {
        if let Some((forced, _stored_before_judging)) = &self.forced {
            store.stop_forcing_these_conclusion_blocks_to_be_judged_again(
                std::iter::once(forced.key.clone()).chain(forced.observer_keys.iter().cloned()),
            );
        }
    }

    /// 打进日志的那一行（[`REUSE_SENTINELS_LINE_PREFIX`]）：种子、候选数、哨兵与强制现判那一块的块序号；不足 `count` 块时另说一句、不红。
    #[must_use]
    pub fn line(&self, flow_name: &str, count: usize) -> String {
        let sentinels = if self.sentinels.is_empty() {
            "-".to_string()
        } else {
            self.sentinels
                .iter()
                .map(|sentinel| sentinel.block_sequence.to_string())
                .collect::<Vec<String>>()
                .join(",")
        };
        let forced = self.forced.as_ref().map_or_else(
            || "-".to_string(),
            |(block, _stored)| block.block_sequence.to_string(),
        );
        let short = if self.sentinels.len() < count {
            format!(
                " note=哨兵不足{count}块（键相同、库里有不红结论的只有{}块），这一趟不因此判红",
                self.candidate_count
            )
        } else {
            String::new()
        };
        format!(
            "{REUSE_SENTINELS_LINE_PREFIX} flow={flow_name} seed={} candidates={} sentinels={sentinels} forced={forced} reproduce={REUSE_SENTINEL_SEED_ENVIRONMENT_VARIABLE}={}{short}",
            self.seed, self.candidate_count, self.seed
        )
    }

    /// 判完之后判哨兵（一句一处红，每句点名块键与种子），连同把库的强制现判表清掉（这一趟用完了）：哨兵有一块这一趟交来存过（键相同却现判了），
    /// 或它上面挂着的观察者有一份结论这一趟交来存过；强制现判那一块这一趟没交来存（还是复用了，强制现判没起作用），或它上面挂着的观察者有一份结论
    /// 没交来存；它（或它上面一份观察者结论）现判出的与开跑前库里那一份不是同一个结论（判定不稳；按判定向量本身比，
    /// 抽哨兵之后库换过一套编号也照样比得对）。观察者结论在判器结论之后判（观察者判段读判器交回的那一份），调用方在两段都判完之后才调它。
    ///
    /// # Errors
    /// 库读失败。
    pub fn problems_after_judging(
        &self,
        store: &mut VerdictStore,
    ) -> Result<Vec<String>, VerdictStoreError> {
        self.stop_forcing_the_forced_block(store);
        let offered: BTreeSet<&crate::verdict_store::ConclusionOfABlockUnderAKey> = store
            .conclusion_blocks_offered_since(self.offered_before_judging)
            .iter()
            .collect();
        let mut problems: Vec<String> = self
            .sentinels
            .iter()
            .filter(|sentinel| offered.contains(&sentinel.key))
            .map(|sentinel| {
                format!(
                    "复用失灵：哨兵块 {}（点 {} 的 [{}, {})，块键 {}）键相同、库里有不红的结论，这一趟却现判了（种子 {}，带 {REUSE_SENTINEL_SEED_ENVIRONMENT_VARIABLE}={} 重跑复现）",
                    sentinel.block_sequence,
                    sentinel.point_index,
                    sentinel.local.start,
                    sentinel.local.end,
                    block_key_named_in_the_sentinel_report(&sentinel.key.0),
                    self.seed,
                    self.seed
                )
            })
            .collect();
        for sentinel in &self.sentinels {
            for observer_key in sentinel
                .observer_keys
                .iter()
                .filter(|observer_key| offered.contains(observer_key))
            {
                problems.push(format!(
                    "复用失灵：哨兵块 {}（点 {} 的 [{}, {})，块键 {}）上观察者那一份结论（版本 {}）键相同、库里有不红的，这一趟却现判了（种子 {}，带 {REUSE_SENTINEL_SEED_ENVIRONMENT_VARIABLE}={} 重跑复现）",
                    sentinel.block_sequence,
                    sentinel.point_index,
                    sentinel.local.start,
                    sentinel.local.end,
                    block_key_named_in_the_sentinel_report(&sentinel.key.0),
                    hexadecimal_text(&observer_key.2 .0[..8]),
                    self.seed,
                    self.seed
                ));
            }
        }
        if let Some((block, stored_before_judging)) = &self.forced {
            if offered.contains(&block.key) {
                let (block_key, block_kind, block_version) = &block.key;
                let judged_now =
                    store.read_conclusion_block(block_key, block_kind, block_version)?;
                if !judged_now.as_ref().is_some_and(|judged_now| {
                    ConclusionSpelledOutInVerdictVectors::of_the_stored_block(store, judged_now)
                        == *stored_before_judging
                }) {
                    problems.push(format!(
                        "判定不稳：强制现判的块 {}（点 {} 的 [{}, {})，块键 {}）现判出的与库里那一份不是同一个结论（种子 {}）：判法依赖了键之外的输入",
                        block.block_sequence,
                        block.point_index,
                        block.local.start,
                        block.local.end,
                        block_key_named_in_the_sentinel_report(block_key),
                        self.seed
                    ));
                }
            } else {
                problems.push(format!(
                    "复用哨兵：抽中强制现判的块 {}（点 {} 的 [{}, {})，块键 {}）这一趟还是复用了，强制现判没起作用（种子 {}）",
                    block.block_sequence,
                    block.point_index,
                    block.local.start,
                    block.local.end,
                    block_key_named_in_the_sentinel_report(&block.key.0),
                    self.seed
                ));
            }
            for (observer_key, observer_conclusion_before_judging) in
                &self.forced_observer_conclusions
            {
                let observer_version_text = hexadecimal_text(&observer_key.2 .0[..8]);
                if !offered.contains(observer_key) {
                    problems.push(format!(
                        "复用哨兵：抽中强制现判的块 {}（点 {} 的 [{}, {})，块键 {}）上观察者那一份结论（版本 {observer_version_text}）这一趟还是复用了，强制现判没起作用（种子 {}）",
                        block.block_sequence,
                        block.point_index,
                        block.local.start,
                        block.local.end,
                        block_key_named_in_the_sentinel_report(&block.key.0),
                        self.seed
                    ));
                } else if !observer_conclusion_judged_now_is_the_one_stored_before(
                    store,
                    observer_key,
                    observer_conclusion_before_judging,
                )? {
                    problems.push(format!(
                        "判定不稳：强制现判的块 {}（点 {} 的 [{}, {})，块键 {}）上观察者那一份结论（版本 {observer_version_text}）现判出的与库里那一份不是同一个结论（种子 {}）：判法依赖了键之外的输入",
                        block.block_sequence,
                        block.point_index,
                        block.local.start,
                        block.local.end,
                        block_key_named_in_the_sentinel_report(&block.key.0),
                        self.seed
                    ));
                }
            }
        }
        Ok(problems)
    }

    /// 强制现判那一块这一趟交来存过没有（抽完之后交来的那几块里有它）；没抽到答没有。
    #[must_use]
    pub fn forced_block_was_judged_here(&self, store: &VerdictStore) -> bool {
        self.forced.as_ref().is_some_and(|(block, _stored)| {
            store
                .conclusion_blocks_offered_since(self.offered_before_judging)
                .contains(&block.key)
        })
    }

    /// 强制现判那一块的态数（它按复用算：判完从这一趟现判的账里扣掉）；没抽到是 0。
    #[must_use]
    pub fn states_of_the_forced_block(&self) -> u64 {
        self.forced
            .as_ref()
            .map_or(0, |(block, _stored)| block.local.end - block.local.start)
    }
}

/// 强制现判之后库里那一份观察者结论（`observer_key`）与开跑前那一份（判定向量本身）是同一个结论；这一趟现判之后库里没有了答不是。
///
/// # Errors
/// 库读失败。
fn observer_conclusion_judged_now_is_the_one_stored_before(
    store: &VerdictStore,
    observer_key: &crate::verdict_store::ConclusionOfABlockUnderAKey,
    stored_before_judging: &ConclusionSpelledOutInVerdictVectors,
) -> Result<bool, VerdictStoreError> {
    let (block_key, observer_kind, observer_version) = observer_key;
    Ok(
        match store.read_conclusion_block(block_key, observer_kind, observer_version)? {
            Some(judged_now) => {
                ConclusionSpelledOutInVerdictVectors::of_the_stored_block(store, &judged_now)
                    == *stored_before_judging
            }
            None => false,
        },
    )
}

#[cfg(test)]
mod tests;
