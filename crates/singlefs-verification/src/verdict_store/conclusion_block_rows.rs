//! 结论那几张表（`0A`–`0E`、`18`）的行怎么编、怎么读回：结论块的记录、恢复字、计数向量、在哪判的与块属性、
//! 点名的被抛弃根，换上一份结论时删哪几行，两份结论比出哪里不同。

use super::*;

/// 恢复字旁表里一个状态占几个字节：[`RECOVERY_WORDS_PER_STATE`] 个字，每字 4 字节小端。
const RECOVERY_WORD_BYTES_PER_STATE: usize = RECOVERY_WORDS_PER_STATE * std::mem::size_of::<u32>();

pub(crate) fn recovery_words_value_bytes(words_of_each_state: &[RecoveryWordsOfState]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(words_of_each_state.len() * RECOVERY_WORD_BYTES_PER_STATE);
    for words in words_of_each_state {
        for word in words.0 {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
    }
    bytes
}

/// [`recovery_words_value_bytes`] 的逆；长度不是 `state_count` 个状态的整份交回 None。
pub(super) fn recovery_words_from_value_bytes(
    bytes: &[u8],
    state_count: usize,
) -> Option<Vec<RecoveryWordsOfState>> {
    if bytes.len() != state_count * RECOVERY_WORD_BYTES_PER_STATE {
        return None;
    }
    let (states, _no_remainder) = bytes.as_chunks::<RECOVERY_WORD_BYTES_PER_STATE>();
    Some(
        states
            .iter()
            .map(|state_bytes| {
                let (word_bytes, _no_word_remainder) = state_bytes.as_chunks::<4>();
                let mut words = [0u32; RECOVERY_WORDS_PER_STATE];
                for (word, bytes_of_word) in words.iter_mut().zip(word_bytes) {
                    *word = u32::from_le_bytes(*bytes_of_word);
                }
                RecoveryWordsOfState(words)
            })
            .collect(),
    )
}

/// 点名的被抛弃根旁表里一条根占几个字节：实例代号 4 字节、checkpoint_txg 8 字节，都是小端。
const ABANDONED_ROOT_NAMED_BYTES: usize = 4 + 8;

/// 点名的被抛弃根旁表的值：每个状态先写它点名了几条（4 字节小端），再逐条写实例代号与 checkpoint_txg。
pub(crate) fn abandoned_roots_named_value_bytes(
    roots_of_each_state: &[Vec<AbandonedRootNamedByThePoolChecker>],
) -> Vec<u8> {
    let mut bytes = Vec::new();
    for roots in roots_of_each_state {
        let root_count = u32::try_from(roots.len()).expect("一个状态点名的被抛弃根装得进 u32");
        bytes.extend_from_slice(&root_count.to_le_bytes());
        for root in roots {
            bytes.extend_from_slice(&root.instance.0.to_le_bytes());
            bytes.extend_from_slice(&root.checkpoint_txg.0.to_le_bytes());
        }
    }
    bytes
}

/// [`abandoned_roots_named_value_bytes`] 的逆；不是恰好 `state_count` 个状态的整份（字节不够、多出）交回 None。
pub(super) fn abandoned_roots_named_from_value_bytes(
    bytes: &[u8],
    state_count: usize,
) -> Option<Vec<Vec<AbandonedRootNamedByThePoolChecker>>> {
    let mut position = 0usize;
    let mut take = |length: usize| -> Option<&[u8]> {
        let taken = bytes.get(position..position.checked_add(length)?)?;
        position += length;
        Some(taken)
    };
    let mut roots_of_each_state = Vec::with_capacity(state_count);
    for _ in 0..state_count {
        let root_count =
            usize::try_from(u32::from_le_bytes(<[u8; 4]>::try_from(take(4)?).ok()?)).ok()?;
        let mut roots = Vec::new();
        for _ in 0..root_count {
            let root_bytes = take(ABANDONED_ROOT_NAMED_BYTES)?;
            roots.push(AbandonedRootNamedByThePoolChecker {
                instance: InstanceGeneration(u32::from_le_bytes(
                    <[u8; 4]>::try_from(&root_bytes[..4]).ok()?,
                )),
                checkpoint_txg: CheckpointTxg(u64::from_le_bytes(
                    <[u8; 8]>::try_from(&root_bytes[4..]).ok()?,
                )),
            });
        }
        roots_of_each_state.push(roots);
    }
    (position == bytes.len()).then_some(roots_of_each_state)
}

fn push_section_entry_count(bytes: &mut Vec<u8>, entry_count: usize) {
    let entry_count = u32::try_from(entry_count).expect("一块的计数向量一节不会超过 u32::MAX 条");
    bytes.extend_from_slice(&entry_count.to_be_bytes());
}

/// 读一节：条数，再逐条「名字、值」；名字要严格升序（写的时候按 `BTreeMap` 的次序写，重名或乱序就是坏字节）。
fn read_count_section<Count>(
    cursor: &mut ByteCursor<'_>,
    read_count: impl Fn(&mut ByteCursor<'_>) -> Option<Count>,
) -> Option<BTreeMap<String, Count>> {
    let entry_count = cursor.big_endian_u32()?;
    let mut section = BTreeMap::new();
    for _entry in 0..entry_count {
        let name = cursor.length_prefixed_text()?;
        if section
            .last_key_value()
            .is_some_and(|(previous_name, _count): (&String, &Count)| *previous_name >= name)
        {
            return None;
        }
        let count = read_count(cursor)?;
        section.insert(name, count);
    }
    Some(section)
}

/// 计数向量（[`BlockCountVector`]，类型在判器一侧 `crate::crash_judge_conclusions`）存进计数向量表的字节：头一个字节起就是三节
/// （逐条不变量、观察者、按发布分，没有别的头），每节 4 字节大端条数、逐条名字（4 字节大端长度 + UTF-8）与计数（每个 8 字节大端；不变量一条三个：
/// 判到、违例、不适用）。字节布局只在这里，计数向量里每一格是什么由判器一侧定。
#[must_use]
pub fn count_vector_bytes(count_vector: &BlockCountVector) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_section_entry_count(&mut bytes, count_vector.per_invariant.len());
    for (invariant_name, counts) in &count_vector.per_invariant {
        push_length_prefixed_text(&mut bytes, invariant_name);
        for count in [counts.judged, counts.violated, counts.not_applicable] {
            bytes.extend_from_slice(&count.to_be_bytes());
        }
    }
    for section in [&count_vector.per_observer, &count_vector.states_by_publish] {
        push_section_entry_count(&mut bytes, section.len());
        for (name, count) in section {
            push_length_prefixed_text(&mut bytes, name);
            bytes.extend_from_slice(&count.to_be_bytes());
        }
    }
    bytes
}

/// [`count_vector_bytes`] 的逆；读不够、名字不是严格升序、后面多出字节，交回 None。
#[must_use]
pub fn count_vector_from_bytes(bytes: &[u8]) -> Option<BlockCountVector> {
    let mut cursor = ByteCursor { bytes, position: 0 };
    let per_invariant = read_count_section(&mut cursor, |cursor| {
        Some(InvariantStateCounts {
            judged: cursor.big_endian_u64()?,
            violated: cursor.big_endian_u64()?,
            not_applicable: cursor.big_endian_u64()?,
        })
    })?;
    let per_observer = read_count_section(&mut cursor, |cursor| cursor.big_endian_u64())?;
    let states_by_publish = read_count_section(&mut cursor, |cursor| cursor.big_endian_u64())?;
    cursor.is_at_the_end().then_some(BlockCountVector {
        per_invariant,
        per_observer,
        states_by_publish,
    })
}

/// 这一块在哪判的（块属性）：CPU，或 GPU 连同卡型与驱动版本。只是出处，不进种类、不进版本（数据不与硬件、不与 CPU 还是 GPU 绑定，
/// 复用不看设备）；GPU 判器是哪一版不记（判器的代码不进键，判法改了对不对靠对拍与复用哨兵抓）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JudgingDevice {
    Cpu,
    Gpu {
        card_type: String,
        driver_version: String,
    },
}

impl JudgingDevice {
    /// 块属性表的值：`01` 是 CPU；`02` 之后是卡型与驱动版本（各 4 字节大端长度 + UTF-8）。
    /// 别的头一个字节认不回来。观察者交回的镜像上池级 checker 的结论记在哪判的，用同一种写法。
    pub(crate) fn to_attribute_bytes(&self) -> Vec<u8> {
        match self {
            Self::Cpu => vec![0x01],
            Self::Gpu {
                card_type,
                driver_version,
            } => {
                let mut bytes = vec![0x02];
                push_length_prefixed_text(&mut bytes, card_type);
                push_length_prefixed_text(&mut bytes, driver_version);
                bytes
            }
        }
    }

    pub(crate) fn from_attribute_bytes(bytes: &[u8]) -> Option<Self> {
        let mut cursor = ByteCursor { bytes, position: 0 };
        let device = match cursor.take(1)? {
            [0x01] => Self::Cpu,
            [0x02] => Self::Gpu {
                card_type: cursor.length_prefixed_text()?,
                driver_version: cursor.length_prefixed_text()?,
            },
            _ => return None,
        };
        cursor.is_at_the_end().then_some(device)
    }
}

/// 存一块一种结论要给的整份：结论块、违例状态、计数向量、块属性，连同恢复字（判器给；观察者没有）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConclusionBlockRecord {
    pub block: VerdictBlock,
    pub violating_states: Vec<StateOffsetInBlock>,
    pub count_vector: BlockCountVector,
    pub judged_on: JudgingDevice,
    pub recovery_words: Option<Vec<RecoveryWordsOfState>>,
}

/// 读回的一块一种结论：结论块、按块内序号排的违例、计数向量、块属性、恢复字（存的时候没给就是 None）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredConclusionBlock {
    pub block: VerdictBlock,
    pub violations: Vec<StoredViolation>,
    pub count_vector: BlockCountVector,
    pub judged_on: JudgingDevice,
    pub recovery_words: Option<Vec<RecoveryWordsOfState>>,
    /// 每个状态上池级 checker 判 I-7.4 时违例细节里点名的被抛弃根（存的时候给了才有：
    /// [`VerdictStore::store_conclusion_block_naming_the_abandoned_roots`]；给定状态表那一路每块都给）。
    pub abandoned_roots_named_by_the_pool_checker:
        Option<Vec<Vec<AbandonedRootNamedByThePoolChecker>>>,
    /// 判定日期与跑法（块属性，存的时候由库按这个进程此刻填；不进键、不进结论的比对）。
    pub circumstances: JudgementCircumstances,
}

impl StoredConclusionBlock {
    /// 两份是不是同一个结论：块、违例、计数向量、恢复字、点名的被抛弃根逐项相同（在哪判的、判定日期与跑法是块属性，不比）。
    /// 存块照它比；跨两次拿锁、跨库比结论（复用哨兵的强制现判）照 `crate::crash_amplification::ConclusionSpelledOutInVerdictVectors` 比同一组字段的判定向量版本。
    /// 两处都整份解构、不写 `..`：这个结构体加了字段，两处不各自写明比不比就编不过。
    #[must_use]
    pub fn is_the_same_conclusion_as(&self, other: &Self) -> bool {
        let Self {
            block: this_block,
            violations: this_violations,
            count_vector: this_count_vector,
            // 在哪判的是块属性、只记出处：同一个结论 CPU 判的与 GPU 判的算同一个（复用不看设备，`JudgingDevice` 的文档）。
            judged_on: _,
            recovery_words: this_recovery_words,
            abandoned_roots_named_by_the_pool_checker:
                this_abandoned_roots_named_by_the_pool_checker,
            // 判定日期与跑法是块属性，存的时候由库按那个进程此刻填：同一个结论换一天、换一种跑法判出来照样是同一个。
            circumstances: _,
        } = self;
        let Self {
            block: other_block,
            violations: other_violations,
            count_vector: other_count_vector,
            // 不比的理由见 `self` 那份解构里 `judged_on` 那一行。
            judged_on: _,
            recovery_words: other_recovery_words,
            abandoned_roots_named_by_the_pool_checker:
                other_abandoned_roots_named_by_the_pool_checker,
            // 不比的理由见 `self` 那份解构里 `circumstances` 那一行。
            circumstances: _,
        } = other;
        this_block == other_block
            && this_violations == other_violations
            && this_count_vector == other_count_vector
            && this_recovery_words == other_recovery_words
            && this_abandoned_roots_named_by_the_pool_checker
                == other_abandoned_roots_named_by_the_pool_checker
    }
}

/// 一份结论换掉库里同键那一份时（[`VerdictBlockStoring::ReplacedBecauseOneOfTheTwoIsRed`]），旧的那一份有、新的那一份没有的行放进同一批删掉：
/// 新的没有的违例状态、新的没给而旧的有的恢复字与点名的被抛弃根。结论块、计数向量与块属性随新的那一份照写（[`put_conclusion_rows_into_the_batch`]）。
pub(super) fn delete_the_rows_the_replacing_conclusion_drops(
    batch: &mut LibraryWriteBatch,
    block_key: &VerdictBlockKey,
    kind: &ConclusionKind,
    conclusion_version: &ConclusionVersion,
    stored: &StoredConclusionBlock,
    replacing: &StoredConclusionBlock,
) {
    for stale in stored.violations.iter().filter(|stored_violation| {
        !replacing
            .violations
            .iter()
            .any(|violation| violation.offset_in_block == stored_violation.offset_in_block)
    }) {
        batch.delete(
            VerdictStoreKey::ConclusionViolation(
                block_key,
                kind,
                conclusion_version,
                stale.offset_in_block,
            )
            .to_key_bytes()
            .0,
        );
    }
    if stored.recovery_words.is_some() && replacing.recovery_words.is_none() {
        batch.delete(
            VerdictStoreKey::RecoveryWords(block_key, kind, conclusion_version)
                .to_key_bytes()
                .0,
        );
    }
    if stored.abandoned_roots_named_by_the_pool_checker.is_some()
        && replacing
            .abandoned_roots_named_by_the_pool_checker
            .is_none()
    {
        batch.delete(
            VerdictStoreKey::AbandonedRootsNamedByThePoolChecker(
                block_key,
                kind,
                conclusion_version,
            )
            .to_key_bytes()
            .0,
        );
    }
}

/// 一个红项写进差异说明的样子：池级 checker 那几项（`pool_checker:<不变量名>`）后面接它在违例位里的下标（位 i ↔
/// `singlefs_checker::image::IMPLEMENTED_INVARIANTS[i]`），别的原样。
fn red_item_naming_its_violated_bit(item: &str) -> String {
    let invariant = item
        .strip_prefix(crate::crash::CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER)
        .and_then(|rest| rest.strip_prefix(':'));
    match invariant.and_then(|invariant| {
        singlefs_checker::image::IMPLEMENTED_INVARIANTS
            .iter()
            .position(|candidate| *candidate == invariant)
    }) {
        Some(position) => format!("{item}（违例位下标 {position}）"),
        None => item.to_string(),
    }
}

/// 一条不变量名写进差异说明的样子：在池级 checker 的清单里的，后面接它在违例位里的下标。
fn invariant_naming_its_violated_bit(invariant: &str) -> String {
    match singlefs_checker::image::IMPLEMENTED_INVARIANTS
        .iter()
        .position(|candidate| *candidate == invariant)
    {
        Some(position) => format!("{invariant}（违例位下标 {position}）"),
        None => invariant.to_string(),
    }
}

/// 一个状态的判定向量编号认回红项（编号认不回判定向量时写明）。
fn red_items_of_the_number<'vectors>(
    number: VerdictVectorNumber,
    verdict_vector_of_number: &impl Fn(VerdictVectorNumber) -> Option<&'vectors EncodedVerdictVector>,
) -> Result<BTreeSet<String>, String> {
    verdict_vector_of_number(number)
        .map(|vector| {
            crate::crash_judge_conclusions::red_items_of_the_encoded_verdict(vector)
                .into_iter()
                .collect()
        })
        .ok_or_else(|| format!("编号 {}（认不回判定向量）", number.0))
}

/// 同一个（块键，结论种类，版本）下已存的与要存的两份结论差在哪，一处一行（[`VerdictStoreError::DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion`]
/// 带着它）：块内状态数；块内每个判定向量编号不同的状态，两份各自独有的红项（池级 checker 的连同违例位下标）；违例的状态；计数向量里逐条不变量、
/// 逐个观察者、逐次发布的态数不同的那几行；恢复字不同的状态；点名的被抛弃根不同的状态。只读两份结论与判定向量，不进任何判定。
pub(super) fn differences_between_two_conclusions<'vectors>(
    stored: &StoredConclusionBlock,
    requested: &StoredConclusionBlock,
    verdict_vector_of_number: impl Fn(VerdictVectorNumber) -> Option<&'vectors EncodedVerdictVector>,
) -> Vec<String> {
    let mut differences = Vec::new();
    let stored_numbers = stored.block.verdict_vector_number_of_each_state();
    let requested_numbers = requested.block.verdict_vector_number_of_each_state();
    if stored_numbers.len() != requested_numbers.len() {
        differences.push(format!(
            "块内状态数：已存那一份 {}、要存那一份 {}",
            stored_numbers.len(),
            requested_numbers.len()
        ));
    }
    let describe_items = |items: Vec<&String>| -> String {
        let named: Vec<String> = items
            .into_iter()
            .map(|item| red_item_naming_its_violated_bit(item))
            .collect();
        format!("[{}]", named.join("、"))
    };
    for (state, (stored_number, requested_number)) in
        stored_numbers.iter().zip(requested_numbers).enumerate()
    {
        if stored_number == requested_number {
            continue;
        }
        match (
            red_items_of_the_number(*stored_number, &verdict_vector_of_number),
            red_items_of_the_number(*requested_number, &verdict_vector_of_number),
        ) {
            (Ok(stored_items), Ok(requested_items)) => differences.push(format!(
                "块内第 {state} 个状态的红项：只在已存那一份里的 {}，只在要存那一份里的 {}",
                describe_items(stored_items.difference(&requested_items).collect()),
                describe_items(requested_items.difference(&stored_items).collect())
            )),
            (stored_items, requested_items) => differences.push(format!(
                "块内第 {state} 个状态的判定向量：已存那一份 {stored_items:?}、要存那一份 {requested_items:?}"
            )),
        }
    }
    let violating_states = |conclusion: &StoredConclusionBlock| -> BTreeSet<u64> {
        conclusion
            .violations
            .iter()
            .map(|violation| violation.offset_in_block.0)
            .collect()
    };
    let (stored_violating, requested_violating) =
        (violating_states(stored), violating_states(requested));
    if stored_violating != requested_violating {
        differences.push(format!(
            "违例的状态（块内序号）：只在已存那一份里的 {:?}，只在要存那一份里的 {:?}",
            stored_violating
                .difference(&requested_violating)
                .collect::<Vec<_>>(),
            requested_violating
                .difference(&stored_violating)
                .collect::<Vec<_>>()
        ));
    }
    let invariants: BTreeSet<&String> = stored
        .count_vector
        .per_invariant
        .keys()
        .chain(requested.count_vector.per_invariant.keys())
        .collect();
    for invariant in invariants {
        let stored_counts = stored.count_vector.per_invariant.get(invariant);
        let requested_counts = requested.count_vector.per_invariant.get(invariant);
        if stored_counts != requested_counts {
            differences.push(format!(
                "计数向量里 {}：已存那一份 {stored_counts:?}、要存那一份 {requested_counts:?}",
                invariant_naming_its_violated_bit(invariant)
            ));
        }
    }
    for (what, stored_counts, requested_counts) in [
        (
            "观察者",
            &stored.count_vector.per_observer,
            &requested.count_vector.per_observer,
        ),
        (
            "按发布分的态数",
            &stored.count_vector.states_by_publish,
            &requested.count_vector.states_by_publish,
        ),
    ] {
        let names: BTreeSet<&String> = stored_counts
            .keys()
            .chain(requested_counts.keys())
            .collect();
        for name in names {
            let (stored_count, requested_count) =
                (stored_counts.get(name), requested_counts.get(name));
            if stored_count != requested_count {
                differences.push(format!(
                    "计数向量里{what} {name}：已存那一份 {stored_count:?}、要存那一份 {requested_count:?}"
                ));
            }
        }
    }
    match (&stored.recovery_words, &requested.recovery_words) {
        (Some(stored_words), Some(requested_words)) => {
            for (state, (stored_state_words, requested_state_words)) in
                stored_words.iter().zip(requested_words).enumerate()
            {
                if stored_state_words != requested_state_words {
                    differences.push(format!(
                        "块内第 {state} 个状态的恢复字：已存那一份 {:?}、要存那一份 {:?}",
                        stored_state_words.0, requested_state_words.0
                    ));
                }
            }
            if stored_words.len() != requested_words.len() {
                differences.push(format!(
                    "恢复字的状态数：已存那一份 {}、要存那一份 {}",
                    stored_words.len(),
                    requested_words.len()
                ));
            }
        }
        (stored_words, requested_words) => {
            if stored_words.is_some() != requested_words.is_some() {
                differences.push(format!(
                    "恢复字：已存那一份{}、要存那一份{}",
                    if stored_words.is_some() {
                        "有"
                    } else {
                        "没有"
                    },
                    if requested_words.is_some() {
                        "有"
                    } else {
                        "没有"
                    }
                ));
            }
        }
    }
    match (
        &stored.abandoned_roots_named_by_the_pool_checker,
        &requested.abandoned_roots_named_by_the_pool_checker,
    ) {
        (Some(stored_roots), Some(requested_roots)) => {
            for (state, (stored_state_roots, requested_state_roots)) in
                stored_roots.iter().zip(requested_roots).enumerate()
            {
                if stored_state_roots != requested_state_roots {
                    differences.push(format!(
                        "块内第 {state} 个状态点名的被抛弃根：已存那一份 {stored_state_roots:?}、要存那一份 {requested_state_roots:?}"
                    ));
                }
            }
            if stored_roots.len() != requested_roots.len() {
                differences.push(format!(
                    "点名的被抛弃根的状态数：已存那一份 {}、要存那一份 {}",
                    stored_roots.len(),
                    requested_roots.len()
                ));
            }
        }
        (stored_roots, requested_roots) => {
            if stored_roots.is_some() != requested_roots.is_some() {
                differences.push(format!(
                    "点名的被抛弃根：已存那一份{}、要存那一份{}",
                    if stored_roots.is_some() {
                        "有"
                    } else {
                        "没有"
                    },
                    if requested_roots.is_some() {
                        "有"
                    } else {
                        "没有"
                    }
                ));
            }
        }
    }
    differences
}

/// 结论块属性那一行的值：在哪判的（[`JudgingDevice::to_attribute_bytes`]，前面 4 字节大端记它的长度）、判定日期（4 字节大端长度 + UTF-8）、
/// 跑法码（1 字节）。
pub(super) fn conclusion_block_attribute_bytes(
    judged_on: &JudgingDevice,
    circumstances: &JudgementCircumstances,
) -> Vec<u8> {
    let device_bytes = judged_on.to_attribute_bytes();
    let mut bytes = Vec::new();
    bytes.extend_from_slice(
        &u32::try_from(device_bytes.len())
            .expect("块属性里在哪判的那一截装得进 u32")
            .to_be_bytes(),
    );
    bytes.extend_from_slice(&device_bytes);
    push_length_prefixed_text(&mut bytes, &circumstances.judged_date);
    bytes.push(circumstances.run_kind.code());
    bytes
}

/// [`conclusion_block_attribute_bytes`] 的逆；不合布局交回 None（跑法码不是这一版认得的也是不合布局）。
pub(super) fn conclusion_block_attributes_from_bytes(
    bytes: &[u8],
) -> Option<(JudgingDevice, JudgementCircumstances)> {
    let mut cursor = ByteCursor { bytes, position: 0 };
    let device_length = usize::try_from(cursor.big_endian_u32()?).ok()?;
    let judged_on = JudgingDevice::from_attribute_bytes(cursor.take(device_length)?)?;
    let judged_date = cursor.length_prefixed_text()?;
    let run_kind = RunKindOfTheJudgement::of_code(*cursor.take(1)?.first()?)?;
    cursor.is_at_the_end().then_some((
        judged_on,
        JudgementCircumstances {
            judged_date,
            run_kind,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 块属性里在哪判的：CPU 的字节是 `01`，GPU 的头一个字节是 `02`、后面接卡型与驱动版本（不记 GPU 判器是哪一版）；两种都读得回原样。
    /// 判别力（`crates/mutations.tsv`）：GPU 的头一个字节写成 `03`，这条红。
    #[test]
    fn judging_device_attribute_bytes_lead_with_one_for_the_cpu_and_two_for_the_gpu() {
        let gpu = JudgingDevice::Gpu {
            card_type: "card".to_string(),
            driver_version: "driver".to_string(),
        };
        let cpu_bytes = JudgingDevice::Cpu.to_attribute_bytes();
        let gpu_bytes = gpu.to_attribute_bytes();
        assert_eq!(cpu_bytes, vec![0x01], "CPU 的块属性字节");
        assert_eq!(gpu_bytes.first(), Some(&0x02), "GPU 的块属性头一个字节");
        assert_eq!(
            JudgingDevice::from_attribute_bytes(&cpu_bytes),
            Some(JudgingDevice::Cpu),
            "CPU 读回原样"
        );
        assert_eq!(
            JudgingDevice::from_attribute_bytes(&gpu_bytes),
            Some(gpu),
            "GPU 读回原样"
        );
    }

    /// 计数向量写成字节再读回逐项相同（空的也一样），字节头一个起就是逐条不变量那一节的条数；读不够、后面多出字节、名字不是严格升序的，都读不回。
    /// 判别力（`crates/mutations.tsv`）：读回时不核名字升序，这条红。
    #[test]
    fn count_vector_bytes_round_trip_and_refuse_malformed_bytes() {
        let count_vector = count_vector_of(7);
        let bytes = count_vector_bytes(&count_vector);
        assert_eq!(count_vector_from_bytes(&bytes), Some(count_vector.clone()));
        assert_eq!(
            &bytes[..4],
            &u32::try_from(count_vector.per_invariant.len())
                .expect("测试的计数向量条数装得进 u32")
                .to_be_bytes(),
            "字节头一个起就是逐条不变量那一节的条数"
        );
        assert_eq!(
            count_vector_from_bytes(&count_vector_bytes(&BlockCountVector::default())),
            Some(BlockCountVector::default())
        );
        let mut trailing = bytes.clone();
        trailing.push(0);
        let truncated = &bytes[..bytes.len() - 1];
        for (malformed, what) in [(&trailing[..], "后面多出字节"), (truncated, "读不够")] {
            assert_eq!(count_vector_from_bytes(malformed), None, "{what}");
        }
        let mut unsorted = Vec::new();
        push_section_entry_count(&mut unsorted, 0);
        push_section_entry_count(&mut unsorted, 2);
        for name in ["b", "a"] {
            push_length_prefixed_text(&mut unsorted, name);
            unsorted.extend_from_slice(&1u64.to_be_bytes());
        }
        push_section_entry_count(&mut unsorted, 0);
        assert_eq!(count_vector_from_bytes(&unsorted), None, "名字不是严格升序");
    }
}
