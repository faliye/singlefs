//! 存结论块与读结论块：先读后写、照新结论换上、两个写者撞同一把键、读的时候违例行在变，
//! 以及按块成批问存没存、红不红、缺哪几种结论。

use super::*;

/// 一块一种结论在一帧里要写的行（[`VerdictStore::conclusion_block_rows_for_one_frame_with_the_slots_the_judge_handed_back`]）：
/// GPU 批处理服务把它连同删待判行、强制现判凭据放进自己的写结论那一帧，这一帧原子地落。帧里的键都是编它的那一份库接上键空间之后的整把键
/// （服务连那一份库的客户端原样写；一次性库与登记的临时库接名字空间，全仓那一份原样）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConclusionBlockRowsForOneFrame {
    /// 这一帧写下去之后的结局；[`VerdictBlockStoring::AlreadyStoredIdentically`] 时结论行一行都没有（判器交回的逐盘槽数那一行可能有）。
    pub storing: VerdictBlockStoring,
    /// 要写的行（键、值，键是库里的整把键字节），守卫要看的结论块那一行在的话排最后。
    pub rows_to_put: Vec<(Vec<u8>, Vec<u8>)>,
    /// 一份红而整份换上时，旧的那一份有、这一份没有的行（违例、旁表）的键。
    pub keys_to_delete: Vec<Vec<u8>>,
    /// 守卫要看的那一行：这一块这一种结论这一把版本的结论块那一行的键。
    pub guard_key: Vec<u8>,
    /// 编这一帧之前读到的守卫那一行的值（不在是 `None`）：写的那一帧照它核，此刻库里那一行与它不同就整帧拒。
    pub guard_row_as_read: Option<Vec<u8>>,
    /// 要存的那一份结论（判定向量是这个库的编号）：结论摘要照它算（`crash_amplification::ConclusionSpelledOutInVerdictVectors`）。
    pub conclusion_as_requested: StoredConclusionBlock,
}

/// 一块一种结论在库里的全部行（结论块、违例、计数向量、块属性，给了恢复字的连同恢复字）放进一批。存结论块（[`VerdictStore::put_a_conclusion_block_into_the_batch`]）经它，行的写法只在这一处。
fn put_conclusion_rows_into_the_batch(
    batch: &mut LibraryWriteBatch,
    block_key: &VerdictBlockKey,
    kind: &ConclusionKind,
    conclusion_version: &ConclusionVersion,
    conclusion: &StoredConclusionBlock,
) {
    batch.put_owned(
        VerdictStoreKey::ConclusionBlock(block_key, kind, conclusion_version)
            .to_key_bytes()
            .0,
        conclusion.block.to_block_value_bytes(),
    );
    for violation in &conclusion.violations {
        batch.put(
            VerdictStoreKey::ConclusionViolation(
                block_key,
                kind,
                conclusion_version,
                violation.offset_in_block,
            )
            .to_key_bytes()
            .0,
            violation.verdict_vector_number.to_stored_bytes(),
        );
    }
    batch.put_owned(
        VerdictStoreKey::ConclusionCountVector(block_key, kind, conclusion_version)
            .to_key_bytes()
            .0,
        count_vector_bytes(&conclusion.count_vector),
    );
    batch.put_owned(
        VerdictStoreKey::ConclusionBlockAttributes(block_key, kind, conclusion_version)
            .to_key_bytes()
            .0,
        conclusion_block_attribute_bytes(&conclusion.judged_on, &conclusion.circumstances),
    );
    if let Some(recovery_words) = &conclusion.recovery_words {
        batch.put_owned(
            VerdictStoreKey::RecoveryWords(block_key, kind, conclusion_version)
                .to_key_bytes()
                .0,
            recovery_words_value_bytes(recovery_words),
        );
    }
    if let Some(roots_of_each_state) = &conclusion.abandoned_roots_named_by_the_pool_checker {
        batch.put_owned(
            VerdictStoreKey::AbandonedRootsNamedByThePoolChecker(
                block_key,
                kind,
                conclusion_version,
            )
            .to_key_bytes()
            .0,
            abandoned_roots_named_value_bytes(roots_of_each_state),
        );
    }
}

/// 要存的结论与库里同键那一份比出来的结局（结论不同是错误，见 [`VerdictStore::stored_conclusion_compared_with`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StoredConclusionComparison {
    NotStored,
    StoredIdentically,
    /// 块、违例、计数向量、恢复字逐项相同，只有在哪判的（[`JudgingDevice`]）不同：同一张表先后分到两张卡、两台机器上判就是这样，
    /// 结论算已存、留库里先存的那一份块属性，什么都不写。
    StoredIdenticallyOnAnotherJudgingDevice,
}

/// 一块一种结论在一张旁表里的那一行：结论块表没有这一块时，报错要点名的表、键与说明。
struct SideRowOfAConclusionBlock {
    table: VerdictStoreTable,
    key_bytes: Vec<u8>,
    reason: &'static str,
}

/// 读回一块一种结论时，结论违例行在读结论块那一帧前后两次扫出来都不同，最多这样读几遍（[`VerdictStore::read_conclusion_block`]）。
/// 每一遍对不上，都要有一批改了这把键违例行的写恰好落在这一遍的两次扫之间。
const CONCLUSION_READ_ATTEMPTS_WHILE_THE_VIOLATION_ROWS_KEEP_CHANGING: usize = 8;

/// 一块一种结论那五行一帧成批点读回来的结局（[`VerdictStore::read_the_conclusion_rows_in_one_frame`]）：结论块在，交五行；不在，交同一帧里
/// 两张旁表（恢复字、点名的被抛弃根）那两行（存块之前照它分没存过与旁表孤儿）。
enum ConclusionRowsReadInOneFrameOrTheSideRows {
    ConclusionBlockStored(ConclusionRowsReadInOneFrame),
    ConclusionBlockNotStored {
        recovery_words_value: Option<Vec<u8>>,
        abandoned_roots_value: Option<Vec<u8>>,
    },
}

/// 一块一种结论在结论块表这一侧的五行，一帧成批点读回来、还没解码（结论块在才有这一份）。
struct ConclusionRowsReadInOneFrame {
    conclusion_key: VerdictStoreKeyBytes,
    count_key: VerdictStoreKeyBytes,
    attribute_key: VerdictStoreKeyBytes,
    recovery_words_key: VerdictStoreKeyBytes,
    abandoned_roots_key: VerdictStoreKeyBytes,
    conclusion_value_bytes: Vec<u8>,
    count_value: Option<Vec<u8>>,
    attribute_value: Option<Vec<u8>>,
    recovery_words_value: Option<Vec<u8>>,
    abandoned_roots_value: Option<Vec<u8>>,
}

/// 读窗口上登记的测试动作（用例在这里插进另一个写者）。
#[cfg(test)]
type ActionInAReadWindow = std::cell::RefCell<Option<Box<dyn FnMut()>>>;

#[cfg(test)]
thread_local! {
    /// 测试开关：这个线程上存块之前那一帧读到结论块不在、旁表有行，重读结论块之前跑的动作。
    static ACTION_BETWEEN_THE_CONCLUSION_READ_AND_THE_SIDE_ROW_READ: ActionInAReadWindow =
        const { std::cell::RefCell::new(None) };
    /// 测试开关：这个线程上读结论块时，先扫的结论违例行扫完、结论块那一帧还没读时跑的动作。
    static ACTION_BETWEEN_THE_EARLIER_VIOLATION_SCAN_AND_THE_CONCLUSION_ROWS_READ: ActionInAReadWindow =
        const { std::cell::RefCell::new(None) };
    /// 测试开关：这个线程上读结论块时，结论块那一帧读完、后扫的结论违例行还没扫时跑的动作。
    static ACTION_BETWEEN_THE_CONCLUSION_ROWS_READ_AND_THE_LATER_VIOLATION_SCAN: ActionInAReadWindow =
        const { std::cell::RefCell::new(None) };
}

/// 跑 `read_window` 上登记的动作。动作跑的时候格子是空的：它里面另一个写者读库走到这一处时不再套一层。
#[cfg(test)]
fn run_the_action_registered_in(read_window: &'static std::thread::LocalKey<ActionInAReadWindow>) {
    if let Some(mut action) = read_window.take() {
        action();
        read_window.set(Some(action));
    }
}

impl VerdictStore {
    /// 这一块这一种结论在库里存着的每一把版本（按版本字节排）：键对不上时照它找库里那一把，点名变了的单元。扫的是块属性表
    /// （与结论块同一批写、键的后缀相同、值只有几十字节），不把几把版本的整块结论读回来。
    ///
    /// # Errors
    /// 服务读不成。
    pub fn conclusion_versions_stored_for(
        &self,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
    ) -> Result<Vec<ConclusionVersion>, VerdictStoreError> {
        let mut prefix_after_the_tag = block_key.to_block_key_bytes();
        prefix_after_the_tag.extend_from_slice(&conclusion_kind_key_bytes(kind));
        let mut prefix = vec![VerdictStoreTable::ConclusionBlockAttributes.key_tag()];
        prefix.extend_from_slice(&prefix_after_the_tag);
        let prefix_length = prefix.len();
        let mut found = Vec::new();
        for entry in self.entries_with_key_prefix_kept_in_or_out(VerdictStoreKeyBytes(prefix)) {
            let (key_bytes, _value_bytes) = entry?;
            if let Ok(version) = <[u8; SHA256_DIGEST_BYTES]>::try_from(&key_bytes[prefix_length..])
            {
                found.push(ConclusionVersion(version));
            }
        }
        Ok(found)
    }

    /// 几块各自在库里存着的这一种结论的每一把版本（[`Self::conclusion_versions_stored_for`] 的成批版），交回的次序与 `block_keys` 相同、每块里按版本字节排。
    /// 块键头两个摘要（输入摘要与枚举计划哈希）相同的几块一起扫：块属性表里键以这 64 字节起头的行按键序逐帧扫一遍
    /// （[`Self::visit_entries_with_key_prefix_kept_in_or_out`]），只留 `block_keys` 里的块、种类是 `kind`、种类之后正好 32 字节版本的那几行；
    /// 逐块扫读到的就是这几行，同一块的行在扫回的次序里按版本字节排，所以每块交回的与逐块扫的逐项相同。
    /// 一帧扫回的行可以多出别的种类与不在 `block_keys` 里的块（同一个点上块长不同的那一块另成一组），只丢不留。
    ///
    /// # Errors
    /// 服务读不成。
    pub fn conclusion_versions_stored_for_blocks(
        &self,
        block_keys: &[VerdictBlockKey],
        kind: &ConclusionKind,
    ) -> Result<Vec<Vec<ConclusionVersion>>, VerdictStoreError> {
        let kind_bytes = conclusion_kind_key_bytes(kind);
        let mut found: Vec<Vec<ConclusionVersion>> = vec![Vec::new(); block_keys.len()];
        // 键是块键的字节，值是这一块在 `block_keys` 里的下标（同一块给了几次就有几个）
        let mut positions_of_each_block: HashMap<Vec<u8>, Vec<usize>> = HashMap::new();
        let mut prefixes_in_the_order_seen: Vec<Vec<u8>> = Vec::new();
        for (position, block_key) in block_keys.iter().enumerate() {
            let block_key_bytes = block_key.to_block_key_bytes();
            let prefix_of_the_group = block_key_bytes[..2 * SHA256_DIGEST_BYTES].to_vec();
            if !prefixes_in_the_order_seen.contains(&prefix_of_the_group) {
                prefixes_in_the_order_seen.push(prefix_of_the_group);
            }
            positions_of_each_block
                .entry(block_key_bytes)
                .or_default()
                .push(position);
        }
        // 迭代上界：组数（块键头 64 字节不同的有几种）；每组逐行看扫回的行
        for prefix_of_the_group in prefixes_in_the_order_seen {
            let mut prefix = vec![VerdictStoreTable::ConclusionBlockAttributes.key_tag()];
            prefix.extend_from_slice(&prefix_of_the_group);
            self.visit_entries_with_key_prefix_kept_in_or_out(
                VerdictStoreKeyBytes(prefix),
                &mut |(key_bytes, _value_bytes)| {
                    let Some(after_the_tag) = key_bytes.get(KEY_TAG_BYTES..) else {
                        return Ok(());
                    };
                    if after_the_tag.len() < BLOCK_KEY_BYTES {
                        return Ok(());
                    }
                    let (block_key_bytes, suffix) = after_the_tag.split_at(BLOCK_KEY_BYTES);
                    let Some(positions) = positions_of_each_block.get(block_key_bytes) else {
                        return Ok(());
                    };
                    let Some(version_bytes) = suffix.strip_prefix(kind_bytes.as_slice()) else {
                        return Ok(());
                    };
                    if let Ok(version) = <[u8; SHA256_DIGEST_BYTES]>::try_from(version_bytes) {
                        for position in positions {
                            found[*position].push(ConclusionVersion(version));
                        }
                    }
                    Ok(())
                },
            )?;
        }
        Ok(found)
    }

    /// 几块几种结论几把版本各自存没存、红不红（交回的次序与 `conclusions` 相同）：块属性那几行一起成批点读（与结论块同一批写、值只有几十字节，
    /// 同 [`Self::conclusions_missing_for_block`]），存着的再各问一帧违例表这一块的前缀有没有行（[`Self::conclusion_block_has_violations`]）。
    /// 与逐块照这两样问答的相同；不把结论块读回来。
    ///
    /// # Errors
    /// 服务读不成；块属性那一行在而不合布局（同 [`Self::conclusions_missing_for_block`]，按 `conclusions` 的次序报头一个）。
    pub fn conclusion_blocks_stored_and_red(
        &self,
        conclusions: &[ConclusionOfABlockUnderAKey],
    ) -> Result<Vec<ConclusionBlockInTheLibrary>, VerdictStoreError> {
        let attribute_keys: Vec<VerdictStoreKeyBytes> = conclusions
            .iter()
            .map(|(block_key, kind, conclusion_version)| {
                VerdictStoreKey::ConclusionBlockAttributes(block_key, kind, conclusion_version)
                    .to_key_bytes()
            })
            .collect();
        let key_slices: Vec<&[u8]> = attribute_keys.iter().map(|key| key.0.as_slice()).collect();
        let attribute_values = self.read_values_kept_in_or_out(&key_slices)?;
        let mut answers = Vec::with_capacity(conclusions.len());
        for (((block_key, kind, conclusion_version), attribute_value), attribute_key) in conclusions
            .iter()
            .zip(attribute_values)
            .zip(&attribute_keys)
        {
            let Some(attribute_bytes) = attribute_value else {
                answers.push(ConclusionBlockInTheLibrary::NotStored);
                continue;
            };
            if conclusion_block_attributes_from_bytes(&attribute_bytes).is_none() {
                return Err(stored_bytes_mismatch(
                    VerdictStoreTable::ConclusionBlockAttributes,
                    &attribute_key.0,
                    "块属性的字节不合布局",
                ));
            }
            answers.push(
                if self.conclusion_block_has_violations(block_key, kind, conclusion_version)? {
                    ConclusionBlockInTheLibrary::StoredRed
                } else {
                    ConclusionBlockInTheLibrary::StoredGreen
                },
            );
        }
        Ok(answers)
    }

    /// 观察者录入块那张表里这几行各自在不在（[`Self::read_observer_block`] 读回的是不是 `Some`，交回的次序与 `rows` 相同）：
    /// 每 [`OBSERVER_BLOCK_ROWS_READ_TOGETHER`] 行一帧成批点读，读回的值判完在不在就丢。
    ///
    /// # Errors
    /// 服务读不成。
    pub fn observer_block_rows_stored(
        &self,
        rows: &[ConclusionOfABlockUnderAKey],
    ) -> Result<Vec<bool>, VerdictStoreError> {
        let mut stored = Vec::with_capacity(rows.len());
        for rows_read_together in rows.chunks(OBSERVER_BLOCK_ROWS_READ_TOGETHER) {
            let keys: Vec<VerdictStoreKeyBytes> = rows_read_together
                .iter()
                .map(|(block_key, kind, observer_version)| {
                    VerdictStoreKey::ObserverBlock(block_key, kind, observer_version).to_key_bytes()
                })
                .collect();
            let key_slices: Vec<&[u8]> = keys.iter().map(|key| key.0.as_slice()).collect();
            stored.extend(
                self.read_values_kept_in_or_out(&key_slices)?
                    .into_iter()
                    .map(|value| value.is_some()),
            );
        }
        Ok(stored)
    }

    /// 存一块一种结论：结论块、它的违例、计数向量、块属性与恢复字同一批同步写下去之后才返回；
    /// 违例状态的判定向量编号取块里那个状态的。同一块在别的种类、别的版本下的行不受影响。这个（块键，种类，版本）下已经存着不同的结论、
    /// 两份里有一份红的：照这一份换上（[`VerdictBlockStoring::ReplacedBecauseOneOfTheTwoIsRed`]）。交来的每一块都记进这个进程的账
    /// （[`Self::conclusion_blocks_offered_since`]，复用哨兵读它）。
    ///
    /// # Errors
    /// 这个（块键，种类，版本）下已经存着不同的结论、两份都不红
    /// （`DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion`，什么都没写）；已存的行读回来不合布局；服务读写不成。
    /// 结论相同、只有在哪判的不同（同一张表先后分到两张卡上判）交回 `AlreadyStoredIdentically`、留先存的那一份块属性。
    ///
    /// # Panics
    /// 块里有这个库没登记过的编号、违例状态的块内序号不在块里、恢复字条数与块长不等：都是调用方的 bug。
    pub fn store_conclusion_block(
        &mut self,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
        conclusion_version: &ConclusionVersion,
        record: &ConclusionBlockRecord,
    ) -> Result<VerdictBlockStoring, VerdictStoreError> {
        self.store_conclusion_block_with_the_rows_written_along(
            block_key,
            kind,
            conclusion_version,
            record.clone(),
            None,
            LibraryWriteBatch::default(),
        )
    }

    /// 同 [`Self::store_conclusion_block`]，收下整份记录（块与恢复字不另拷一份），判器交回了逐盘自证槽数的（`slots_bytes_the_judge_handed_back`，
    /// 存在观察者录入块那张表、与结论块同一把键）连同结论那几行同一批、同一帧写：结论块在库里，槽数那一行就在。
    /// 结论已存着、逐项相同（[`VerdictBlockStoring::AlreadyStoredIdentically`]）时只写槽数那一行（结论在而槽数不在的块照判一遍补上的就是这一路）。
    ///
    /// # Errors
    /// 同 [`Self::store_conclusion_block`]（两份都不红而不同时槽数那一行也不写）。
    ///
    /// # Panics
    /// 同 [`Self::store_conclusion_block`]。
    pub fn store_conclusion_block_with_the_slots_the_judge_handed_back(
        &mut self,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
        conclusion_version: &ConclusionVersion,
        record: ConclusionBlockRecord,
        slots_bytes_the_judge_handed_back: Option<Vec<u8>>,
    ) -> Result<VerdictBlockStoring, VerdictStoreError> {
        let mut written_along = LibraryWriteBatch::default();
        if let Some(slots_bytes) = slots_bytes_the_judge_handed_back {
            written_along.put_owned(
                VerdictStoreKey::ObserverBlock(block_key, kind, conclusion_version)
                    .to_key_bytes()
                    .0,
                slots_bytes,
            );
        }
        self.store_conclusion_block_with_the_rows_written_along(
            block_key,
            kind,
            conclusion_version,
            record,
            None,
            written_along,
        )
    }

    /// 同 [`Self::store_conclusion_block`]，连同每个状态上池级 checker 判 I-7.4 时违例细节里点名的被抛弃根（点名的被抛弃根旁表）同一批写；
    /// 库里这个（块键，种类，版本）下已经存着的结论连同点名的根逐项相同才算已存，点名的根不同照「结论不同」报错。给定状态表那一路经它存。
    ///
    /// # Errors
    /// 同 [`Self::store_conclusion_block`]。
    ///
    /// # Panics
    /// 同 [`Self::store_conclusion_block`]；点名的根不是块里每个状态一份。
    pub fn store_conclusion_block_naming_the_abandoned_roots(
        &mut self,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
        conclusion_version: &ConclusionVersion,
        record: &ConclusionBlockRecord,
        abandoned_roots_named_of_each_state: &[Vec<AbandonedRootNamedByThePoolChecker>],
    ) -> Result<VerdictBlockStoring, VerdictStoreError> {
        assert_eq!(
            abandoned_roots_named_of_each_state.len(),
            record.block.verdict_vector_number_of_each_state().len(),
            "点名的被抛弃根每个状态一份：调用方给的条数与块长不等"
        );
        self.store_conclusion_block_with_the_rows_written_along(
            block_key,
            kind,
            conclusion_version,
            record.clone(),
            Some(abandoned_roots_named_of_each_state.to_vec()),
            LibraryWriteBatch::default(),
        )
    }

    /// 存结论块那几个入口的本体：点名的根给了（`Some`）就同一批写旁表；`written_along` 里的行（判器交回的逐盘槽数）排在结论那几行前面、
    /// 同一批写，结论已存着、逐项相同时只写它们（它是空的就什么都不写）。这一批怎么编在 [`Self::conclusion_block_batch_with_the_rows_written_along`]。
    fn store_conclusion_block_with_the_rows_written_along(
        &mut self,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
        conclusion_version: &ConclusionVersion,
        record: ConclusionBlockRecord,
        abandoned_roots_named_by_the_pool_checker: Option<
            Vec<Vec<AbandonedRootNamedByThePoolChecker>>,
        >,
        written_along: LibraryWriteBatch,
    ) -> Result<VerdictBlockStoring, VerdictStoreError> {
        self.conclusion_blocks_offered_in_this_process.push((
            block_key.clone(),
            kind.clone(),
            *conclusion_version,
        ));
        let (storing, batch, _requested) = self
            .conclusion_block_batch_with_the_rows_written_along(
                block_key,
                kind,
                conclusion_version,
                record,
                abandoned_roots_named_by_the_pool_checker,
                written_along,
            )?;
        match storing {
            VerdictBlockStoring::AlreadyStoredIdentically if batch.is_empty() => {
                return Ok(storing)
            }
            VerdictBlockStoring::AlreadyStoredIdentically
            | VerdictBlockStoring::WrittenAndSynced
            | VerdictBlockStoring::ReplacedBecauseOneOfTheTwoIsRed => {}
        }
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch)?;
        Ok(storing)
    }

    /// 一块一种结论连同判器交回的逐盘自证槽数（`slots_bytes_the_judge_handed_back`）要在一帧里写的行，照
    /// [`Self::store_conclusion_block_with_the_slots_the_judge_handed_back`] 同一套规矩编（结论相同不写结论行、一份红就整份换上、两份都不红而不同拒），
    /// 不写库、不记这个进程交来存过：GPU 批处理服务拿它在自己的写结论那一帧里写。守卫要看的那一行（结论块那一行，
    /// [`ConclusionBlockRowsForOneFrame::guard_key`]）在编之前先读（[`ConclusionBlockRowsForOneFrame::guard_row_as_read`]），之后才读库里那一份结论来比：
    /// 两次读之间别人写进来的，守卫照先读的那一份核、整帧拒，不会盖掉别人刚写的那一份。判定向量编号照块里给的（调用方已登记进这个库，登记表只追加）。
    ///
    /// # Errors
    /// 同 [`Self::store_conclusion_block`]。
    ///
    /// # Panics
    /// 同 [`Self::store_conclusion_block`]。
    pub fn conclusion_block_rows_for_one_frame_with_the_slots_the_judge_handed_back(
        &self,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
        conclusion_version: &ConclusionVersion,
        record: ConclusionBlockRecord,
        slots_bytes_the_judge_handed_back: Option<Vec<u8>>,
    ) -> Result<ConclusionBlockRowsForOneFrame, VerdictStoreError> {
        let guard_key = VerdictStoreKey::ConclusionBlock(block_key, kind, conclusion_version)
            .to_key_bytes()
            .0;
        let guard_row_as_read = self
            .read_values_kept_in_or_out(&[guard_key.as_slice()])?
            .pop()
            .expect("点读一把键交回一项");
        let mut written_along = LibraryWriteBatch::default();
        if let Some(slots_bytes) = slots_bytes_the_judge_handed_back {
            written_along.put_owned(
                VerdictStoreKey::ObserverBlock(block_key, kind, conclusion_version)
                    .to_key_bytes()
                    .0,
                slots_bytes,
            );
        }
        let (storing, batch, conclusion_as_requested) = self
            .conclusion_block_batch_with_the_rows_written_along(
                block_key,
                kind,
                conclusion_version,
                record,
                None,
                written_along,
            )?;
        let mut rows_to_put = Vec::new();
        let mut guard_row_to_put = None;
        let mut keys_to_delete = Vec::new();
        for row in batch.rows {
            match row {
                LibraryWriteOfARow::Put { key, value } if key == guard_key => {
                    guard_row_to_put = Some((self.service.namespaced_key(&key), value));
                }
                LibraryWriteOfARow::Put { key, value } => {
                    rows_to_put.push((self.service.namespaced_key(&key), value));
                }
                LibraryWriteOfARow::Delete { key } => {
                    keys_to_delete.push(self.service.namespaced_key(&key));
                }
            }
        }
        rows_to_put.extend(guard_row_to_put);
        Ok(ConclusionBlockRowsForOneFrame {
            storing,
            rows_to_put,
            keys_to_delete,
            guard_key: self.service.namespaced_key(&guard_key),
            guard_row_as_read,
            conclusion_as_requested,
        })
    }

    /// 单测替 GPU 批处理服务把一帧的结论行写进这个库（要写的、要删的同一批），不核守卫：服务那一侧的写结论帧核守卫与认领。
    /// 帧里的键是接上键空间之后的整把键，经这一份写时先去掉键空间那一段（这一份写的时候再接上）。
    ///
    /// # Errors
    /// 服务写不成。
    #[cfg(all(test, feature = "gpu"))]
    pub(crate) fn write_the_conclusion_rows_of_one_frame(
        &self,
        frame: &ConclusionBlockRowsForOneFrame,
    ) -> Result<(), VerdictStoreError> {
        let mut batch = LibraryWriteBatch::default();
        for (key, value) in &frame.rows_to_put {
            batch.put(self.service.key_without_the_key_space(key), value);
        }
        for key in &frame.keys_to_delete {
            batch.delete(self.service.key_without_the_key_space(key));
        }
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch)
    }

    /// 存结论块那几个入口要写的那一批（写不写由调用方定）与要存的那一份结论：断言块里的编号是这个库登记过的、恢复字每状态一份，
    /// 拼出要存的那一份（在哪判的、判定日期与跑法是这个进程此刻的），照 [`Self::put_a_conclusion_block_into_the_batch`] 放进 `written_along` 之后。
    #[allow(
        clippy::type_complexity,
        reason = "存的结局、那一批与要存的那一份结论三样一起交回，各是各的东西"
    )]
    fn conclusion_block_batch_with_the_rows_written_along(
        &self,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
        conclusion_version: &ConclusionVersion,
        record: ConclusionBlockRecord,
        abandoned_roots_named_by_the_pool_checker: Option<
            Vec<Vec<AbandonedRootNamedByThePoolChecker>>,
        >,
        written_along: LibraryWriteBatch,
    ) -> Result<
        (
            VerdictBlockStoring,
            LibraryWriteBatch,
            StoredConclusionBlock,
        ),
        VerdictStoreError,
    > {
        let registered_count = self.known_verdict_vector_count();
        let numbers_of_each_state = record.block.verdict_vector_number_of_each_state();
        assert!(
            numbers_of_each_state.iter().all(|state_number| state_number
                .position_in_the_registration_order()
                < registered_count),
            "结论块里的判定向量编号必须是这个库登记过的（这个库登记了 {registered_count} 种）"
        );
        if let Some(recovery_words) = &record.recovery_words {
            assert_eq!(
                recovery_words.len(),
                numbers_of_each_state.len(),
                "恢复字每个状态一份：调用方给的条数与块长不等"
            );
        }
        let ConclusionBlockRecord {
            block,
            violating_states,
            count_vector,
            judged_on,
            recovery_words,
        } = record;
        let violations = violating_states
            .iter()
            .copied()
            .collect::<BTreeSet<StateOffsetInBlock>>()
            .into_iter()
            .map(|offset_in_block| StoredViolation {
                offset_in_block,
                verdict_vector_number: block
                    .verdict_vector_number_at(offset_in_block)
                    .expect("结论违例的块内序号必须在块里（调用方给的序号超出了块长）"),
            })
            .collect();
        let requested = StoredConclusionBlock {
            block,
            violations,
            count_vector,
            judged_on,
            recovery_words,
            abandoned_roots_named_by_the_pool_checker,
            circumstances: JudgementCircumstances::of_this_process_now(),
        };
        let mut batch = written_along;
        let storing = self.put_a_conclusion_block_into_the_batch(
            block_key,
            kind,
            conclusion_version,
            &requested,
            &mut batch,
        )?;
        Ok((storing, batch, requested))
    }

    /// 一块一种结论（编号已是这个库的）放进 `batch`（规矩只在这一处）：这个（块键，种类，版本）下结论相同的什么都不放、交回
    /// [`VerdictBlockStoring::AlreadyStoredIdentically`]（只有在哪判的不同也算相同，留先存的那一份块属性）；不同而两份里有一份红的，照这一份换上、
    /// 旧的有而这一份没有的行同一批删掉（[`VerdictBlockStoring::ReplacedBecauseOneOfTheTwoIsRed`]）；没存过的交回 [`VerdictBlockStoring::WrittenAndSynced`]。
    /// 交回的是这一批写下去之后的结局，写不写由调用方定。库里那一份照 [`Self::read_the_stored_conclusion_block_rereading_after_a_side_row`] 读。
    ///
    /// # Errors
    /// 同 [`Self::read_the_stored_conclusion_block_rereading_after_a_side_row`]；同 [`Self::stored_conclusion_compared_with`]（两份都不红而不同时 `batch` 里没放这一块的行）。
    fn put_a_conclusion_block_into_the_batch(
        &self,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
        conclusion_version: &ConclusionVersion,
        requested: &StoredConclusionBlock,
        batch: &mut LibraryWriteBatch,
    ) -> Result<VerdictBlockStoring, VerdictStoreError> {
        // 记红、撤绿：库里那一份与这一份不同、两份里有一份红的，照这一份整份换上（判红写红记录、撤同一把键的绿；
        // 记着红的块下一趟现判，判完照新结论写回）；旧的那一份有、这一份没有的行同一批删掉。两份都不红的照下面的比法拒
        let stored = self.read_the_stored_conclusion_block_rereading_after_a_side_row(
            block_key,
            kind,
            conclusion_version,
        )?;
        let replaced_stored = match &stored {
            Some(stored)
                if !stored.is_the_same_conclusion_as(requested)
                    && (!stored.violations.is_empty() || !requested.violations.is_empty()) =>
            {
                delete_the_rows_the_replacing_conclusion_drops(
                    batch,
                    block_key,
                    kind,
                    conclusion_version,
                    stored,
                    requested,
                );
                true
            }
            Some(_) | None => false,
        };
        if !replaced_stored {
            match self.stored_conclusion_compared_with(block_key, kind, stored, requested)? {
                StoredConclusionComparison::StoredIdentically
                | StoredConclusionComparison::StoredIdenticallyOnAnotherJudgingDevice => {
                    return Ok(VerdictBlockStoring::AlreadyStoredIdentically)
                }
                StoredConclusionComparison::NotStored => {}
            }
        }
        put_conclusion_rows_into_the_batch(batch, block_key, kind, conclusion_version, requested);
        if replaced_stored {
            Ok(VerdictBlockStoring::ReplacedBecauseOneOfTheTwoIsRed)
        } else {
            Ok(VerdictBlockStoring::WrittenAndSynced)
        }
    }

    /// 这一块这一种结论这一把版本在库里有没有违例（只扫违例表这一块的前缀、见到一行就停）；没存过的块答没有。
    ///
    /// # Errors
    /// 服务读不成。
    pub fn conclusion_block_has_violations(
        &self,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
        conclusion_version: &ConclusionVersion,
    ) -> Result<bool, VerdictStoreError> {
        let prefix =
            VerdictStoreKey::ConclusionViolationsOfBlockPrefix(block_key, kind, conclusion_version)
                .to_key_bytes();
        self.has_an_entry_with_key_prefix_kept_in_or_out(prefix)
    }

    /// 存块之前读库里这个（块键，种类，版本）下那一份。先一帧成批点读结论块那五行（[`Self::read_the_conclusion_rows_in_one_frame`]，没存过的块——
    /// 存块那一处的常态——这一帧就答完）：结论块在，照 [`Self::read_conclusion_block`] 整遍读（前后两次扫违例行）；结论块与两张旁表都不在，就是没存过；
    /// 结论块不在而旁表有行时重读一次结论块：一帧成批点读里各键不是同一刻读的，结论块那一键读过之后别的进程可以把整批写进去，
    /// 这时旁表的行不是孤儿，重读读到的那一份照常往下比。一批里结论块排在旁表前面（[`put_conclusion_rows_into_the_batch`]），一帧里原子、切成几帧也是结论块先落，
    /// 读得到旁表的行之后结论块已经落了，所以只重读这一次：重读仍没有，结论块就是没写过，旁表那一行是孤儿。
    /// 结论块那一帧读到在、整遍读却不在（两帧之间被换掉的那一份不会删结论块，只有直接改行的会），照旁表再点读一次、同上判。
    ///
    /// # Errors
    /// 重读仍没有（[`VerdictStoreError::StoredBytesDoNotMatchTheLayout`]，报那一张旁表与那一把键）；同 [`Self::read_conclusion_block`]。
    fn read_the_stored_conclusion_block_rereading_after_a_side_row(
        &self,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
        conclusion_version: &ConclusionVersion,
    ) -> Result<Option<StoredConclusionBlock>, VerdictStoreError> {
        let side_row = match self.read_the_conclusion_rows_in_one_frame(
            block_key,
            kind,
            conclusion_version,
        )? {
            ConclusionRowsReadInOneFrameOrTheSideRows::ConclusionBlockStored(_rows_read) => {
                if let Some(stored) =
                    self.read_conclusion_block(block_key, kind, conclusion_version)?
                {
                    return Ok(Some(stored));
                }
                let side_rows_read_again = self.read_values_kept_in_or_out(&[
                    VerdictStoreKey::RecoveryWords(block_key, kind, conclusion_version)
                        .to_key_bytes()
                        .0
                        .as_slice(),
                    VerdictStoreKey::AbandonedRootsNamedByThePoolChecker(
                        block_key,
                        kind,
                        conclusion_version,
                    )
                    .to_key_bytes()
                    .0
                    .as_slice(),
                ])?;
                Self::side_row_of_the_conclusion_block(
                    block_key,
                    kind,
                    conclusion_version,
                    side_rows_read_again,
                )
            }
            ConclusionRowsReadInOneFrameOrTheSideRows::ConclusionBlockNotStored {
                recovery_words_value,
                abandoned_roots_value,
            } => Self::side_row_of_the_conclusion_block(
                block_key,
                kind,
                conclusion_version,
                vec![recovery_words_value, abandoned_roots_value],
            ),
        };
        let Some(side_row) = side_row else {
            return Ok(None);
        };
        #[cfg(test)]
        run_the_action_registered_in(&ACTION_BETWEEN_THE_CONCLUSION_READ_AND_THE_SIDE_ROW_READ);
        match self.read_conclusion_block(block_key, kind, conclusion_version)? {
            Some(stored_by_the_other_writer) => Ok(Some(stored_by_the_other_writer)),
            None => Err(stored_bytes_mismatch(
                side_row.table,
                &side_row.key_bytes,
                side_row.reason,
            )),
        }
    }

    /// 这个（块键，种类，版本）在旁表（恢复字、点名的被抛弃根）里有没有行，照调用方读回的那两行的值（`stored_values`，恢复字在前）：
    /// 两张都有时交回排在前面的那一张（恢复字），都没有交回 `None`。
    fn side_row_of_the_conclusion_block(
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
        conclusion_version: &ConclusionVersion,
        stored_values: Vec<Option<Vec<u8>>>,
    ) -> Option<SideRowOfAConclusionBlock> {
        let side_rows_to_look_for = [
            SideRowOfAConclusionBlock {
                table: VerdictStoreTable::RecoveryWords,
                key_bytes: VerdictStoreKey::RecoveryWords(block_key, kind, conclusion_version)
                    .to_key_bytes()
                    .0,
                reason: "恢复字旁表有这一块这一种结论这一版本，结论块表没有",
            },
            SideRowOfAConclusionBlock {
                table: VerdictStoreTable::AbandonedRootsNamedByThePoolChecker,
                key_bytes: VerdictStoreKey::AbandonedRootsNamedByThePoolChecker(
                    block_key,
                    kind,
                    conclusion_version,
                )
                .to_key_bytes()
                .0,
                reason: "点名的被抛弃根旁表有这一块这一种结论这一版本，结论块表没有",
            },
        ];
        side_rows_to_look_for
            .into_iter()
            .zip(stored_values)
            .find_map(|(side_row, stored_value)| stored_value.map(|_stored_bytes| side_row))
    }

    /// 要存的一块一种结论（编号已是这个库的）与库里这个（块键，种类，版本）下存着的那一份（`stored`，调用方刚读回的）比：没存过交回
    /// `NotStored`；块、违例、计数向量、恢复字与块属性都相同交回 `StoredIdentically`；只有块属性（在哪判的）不同交回
    /// `StoredIdenticallyOnAnotherJudgingDevice`。规矩只在这一处。
    ///
    /// # Errors
    /// 结论不同（`DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion`，带着两份差在哪）。
    fn stored_conclusion_compared_with(
        &self,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
        stored: Option<StoredConclusionBlock>,
        requested: &StoredConclusionBlock,
    ) -> Result<StoredConclusionComparison, VerdictStoreError> {
        if let Some(stored) = stored {
            if !stored.is_the_same_conclusion_as(requested) {
                let verdict_vector_of_number =
                    |number: VerdictVectorNumber| self.verdict_vector_of_number(number);
                return Err(
                    VerdictStoreError::DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion {
                        block_key: block_key.clone(),
                        kind: kind.clone(),
                        differences: Box::new(DifferencesOfTwoConclusionsUnderOneKey {
                            differences: differences_between_two_conclusions(
                                &stored,
                                requested,
                                verdict_vector_of_number,
                            ),
                            stored_judged_on: stored.judged_on.clone(),
                            requested_judged_on: requested.judged_on.clone(),
                        }),
                    },
                );
            }
            if stored.judged_on != requested.judged_on {
                return Ok(StoredConclusionComparison::StoredIdenticallyOnAnotherJudgingDevice);
            }
            return Ok(StoredConclusionComparison::StoredIdentically);
        }
        Ok(StoredConclusionComparison::NotStored)
    }

    /// 读回一块一种结论，连同违例、计数向量、块属性与恢复字；这个（块键，种类，版本）没存过时是 `None`（别的种类、别的版本存过也是 `None`）。
    ///
    /// 服务没有把点读与前缀扫放进一帧的读法，结论块那几行与结论违例行分两帧读，其间别的进程可以换掉这把键的结论。所以读结论块那一帧前后各扫一次违例行：
    /// 两次相同，这把键的违例行在读结论块那一刻就是这一份，交回的结论块与违例行是同一批写下的；不同就整遍重读，
    /// 最多 `CONCLUSION_READ_ATTEMPTS_WHILE_THE_VIOLATION_ROWS_KEEP_CHANGING` 遍。
    /// 一批写落在一帧里（`LibraryWriteBatch` 的文档），所以这两次扫看到的是这批写之前或之后的违例行，不会是半批。
    ///
    /// # Errors
    /// 存着的结论块是空的或点名了没登记的编号、缺计数向量或块属性、计数向量或块属性或恢复字不合布局（块属性的跑法码不是这一版认得的也是）；
    /// 每一遍两次扫到的违例行都不同（[`VerdictStoreError::StoredBytesDoNotMatchTheLayout`]，报结论违例表与这一块的前缀）；
    /// 服务读不成。
    pub fn read_conclusion_block(
        &self,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
        conclusion_version: &ConclusionVersion,
    ) -> Result<Option<StoredConclusionBlock>, VerdictStoreError> {
        // 迭代上界：常量。跨轮不带状态，每一遍从头读；提前出口是结论块不在，与两次扫相同
        for _read_attempt in 0..CONCLUSION_READ_ATTEMPTS_WHILE_THE_VIOLATION_ROWS_KEEP_CHANGING {
            let violation_rows_before_the_block =
                self.conclusion_violation_rows_as_stored(block_key, kind, conclusion_version)?;
            #[cfg(test)]
            run_the_action_registered_in(
                &ACTION_BETWEEN_THE_EARLIER_VIOLATION_SCAN_AND_THE_CONCLUSION_ROWS_READ,
            );
            let ConclusionRowsReadInOneFrameOrTheSideRows::ConclusionBlockStored(
                rows_read_in_one_frame,
            ) = self.read_the_conclusion_rows_in_one_frame(block_key, kind, conclusion_version)?
            else {
                return Ok(None);
            };
            #[cfg(test)]
            run_the_action_registered_in(
                &ACTION_BETWEEN_THE_CONCLUSION_ROWS_READ_AND_THE_LATER_VIOLATION_SCAN,
            );
            let violation_rows_after_the_block =
                self.conclusion_violation_rows_as_stored(block_key, kind, conclusion_version)?;
            if violation_rows_before_the_block == violation_rows_after_the_block {
                return self.conclusion_block_from_the_rows_read(
                    block_key,
                    kind,
                    conclusion_version,
                    rows_read_in_one_frame,
                    violation_rows_after_the_block,
                );
            }
        }
        Err(stored_bytes_mismatch(
            VerdictStoreTable::ConclusionViolations,
            &VerdictStoreKey::ConclusionViolationsOfBlockPrefix(
                block_key,
                kind,
                conclusion_version,
            )
            .to_key_bytes()
            .0,
            "读结论块那一帧前后两次扫到的结论违例行每一遍都不同：别的进程一直在换这把键的结论",
        ))
    }

    /// 这一块一种结论的五行（结论块、计数向量、块属性、恢复字、点名的被抛弃根）一帧成批点读回来，不解码；结论块不在交回同一帧读到的两张旁表那两行。
    ///
    /// # Errors
    /// 服务读不成。
    fn read_the_conclusion_rows_in_one_frame(
        &self,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
        conclusion_version: &ConclusionVersion,
    ) -> Result<ConclusionRowsReadInOneFrameOrTheSideRows, VerdictStoreError> {
        let conclusion_key =
            VerdictStoreKey::ConclusionBlock(block_key, kind, conclusion_version).to_key_bytes();
        let count_key = VerdictStoreKey::ConclusionCountVector(block_key, kind, conclusion_version)
            .to_key_bytes();
        let attribute_key =
            VerdictStoreKey::ConclusionBlockAttributes(block_key, kind, conclusion_version)
                .to_key_bytes();
        let recovery_words_key =
            VerdictStoreKey::RecoveryWords(block_key, kind, conclusion_version).to_key_bytes();
        let abandoned_roots_key = VerdictStoreKey::AbandonedRootsNamedByThePoolChecker(
            block_key,
            kind,
            conclusion_version,
        )
        .to_key_bytes();
        // 一块一种结论的几行一起读：垫在 prune-server 上时是一帧成批点读，不逐键一帧（违例另一帧按前缀扫，对得上对不上由 `read_conclusion_block` 判）
        let [conclusion_value, count_value, attribute_value, recovery_words_value, abandoned_roots_value]: [Option<Vec<u8>>; 5] = self
            .read_values_kept_in_or_out(&[
                conclusion_key.0.as_slice(),
                count_key.0.as_slice(),
                attribute_key.0.as_slice(),
                recovery_words_key.0.as_slice(),
                abandoned_roots_key.0.as_slice(),
            ])?
            .try_into()
            .expect("问了五行：交回的个数与次序与问的相同");
        let Some(conclusion_value_bytes) = conclusion_value else {
            return Ok(
                ConclusionRowsReadInOneFrameOrTheSideRows::ConclusionBlockNotStored {
                    recovery_words_value,
                    abandoned_roots_value,
                },
            );
        };
        Ok(
            ConclusionRowsReadInOneFrameOrTheSideRows::ConclusionBlockStored(
                ConclusionRowsReadInOneFrame {
                    conclusion_key,
                    count_key,
                    attribute_key,
                    recovery_words_key,
                    abandoned_roots_key,
                    conclusion_value_bytes,
                    count_value,
                    attribute_value,
                    recovery_words_value,
                    abandoned_roots_value,
                },
            ),
        )
    }

    /// 一帧读回的五行与照它核过的结论违例行解码成一块一种结论。
    ///
    /// # Errors
    /// 同 [`Self::read_conclusion_block`] 里不合布局的那几种；结论违例行不合布局（[`Self::conclusion_violations_from_the_rows_as_stored`]）。
    fn conclusion_block_from_the_rows_read(
        &self,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
        conclusion_version: &ConclusionVersion,
        rows_read_in_one_frame: ConclusionRowsReadInOneFrame,
        violation_rows_as_stored: Vec<StoredEntry>,
    ) -> Result<Option<StoredConclusionBlock>, VerdictStoreError> {
        let ConclusionRowsReadInOneFrame {
            conclusion_key,
            count_key,
            attribute_key,
            recovery_words_key,
            abandoned_roots_key,
            conclusion_value_bytes,
            count_value,
            attribute_value,
            recovery_words_value,
            abandoned_roots_value,
        } = rows_read_in_one_frame;
        let numbers_of_each_state = self.registered_verdict_vector_numbers_of_a_stored_block(
            VerdictStoreTable::ConclusionBlocks,
            &conclusion_key.0,
            &conclusion_value_bytes,
        )?;
        let state_count = numbers_of_each_state.len();
        let block =
            VerdictBlock::new(numbers_of_each_state).map_err(|EmptyVerdictBlockRejected| {
                stored_bytes_mismatch(
                    VerdictStoreTable::ConclusionBlocks,
                    &conclusion_key.0,
                    "存着的结论块值是空的",
                )
            })?;
        let count_bytes = count_value.ok_or_else(|| {
            stored_bytes_mismatch(
                VerdictStoreTable::ConclusionCountVectors,
                &count_key.0,
                "结论块在而这一行缺着",
            )
        })?;
        let count_vector = count_vector_from_bytes(&count_bytes).ok_or_else(|| {
            stored_bytes_mismatch(
                VerdictStoreTable::ConclusionCountVectors,
                &count_key.0,
                "计数向量的字节不合布局",
            )
        })?;
        let attribute_bytes = attribute_value.ok_or_else(|| {
            stored_bytes_mismatch(
                VerdictStoreTable::ConclusionBlockAttributes,
                &attribute_key.0,
                "结论块在而这一行缺着",
            )
        })?;
        let (judged_on, circumstances) = conclusion_block_attributes_from_bytes(&attribute_bytes)
            .ok_or_else(|| {
            stored_bytes_mismatch(
                VerdictStoreTable::ConclusionBlockAttributes,
                &attribute_key.0,
                "块属性的字节不合布局",
            )
        })?;
        let recovery_words = match recovery_words_value {
            None => None,
            Some(words_bytes) => Some(
                recovery_words_from_value_bytes(&words_bytes, state_count).ok_or_else(|| {
                    stored_bytes_mismatch(
                        VerdictStoreTable::RecoveryWords,
                        &recovery_words_key.0,
                        "恢复字不是块里每个状态 RECOVERY_WORDS_PER_STATE 个字",
                    )
                })?,
            ),
        };
        let abandoned_roots_named_by_the_pool_checker = match abandoned_roots_value {
            None => None,
            Some(roots_bytes) => Some(
                abandoned_roots_named_from_value_bytes(&roots_bytes, state_count).ok_or_else(
                    || {
                        stored_bytes_mismatch(
                            VerdictStoreTable::AbandonedRootsNamedByThePoolChecker,
                            &abandoned_roots_key.0,
                            "点名的被抛弃根不是块里每个状态一份",
                        )
                    },
                )?,
            ),
        };
        Ok(Some(StoredConclusionBlock {
            block,
            violations: self.conclusion_violations_from_the_rows_as_stored(
                block_key,
                kind,
                conclusion_version,
                violation_rows_as_stored,
            )?,
            count_vector,
            judged_on,
            recovery_words,
            abandoned_roots_named_by_the_pool_checker,
            circumstances,
        }))
    }

    /// 这一块一种结论的结论违例行，按键序、不解码：两次扫比的是存着的字节，读到结论块不在时也不解码它们。
    ///
    /// # Errors
    /// 服务读不成。
    fn conclusion_violation_rows_as_stored(
        &self,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
        conclusion_version: &ConclusionVersion,
    ) -> Result<Vec<StoredEntry>, VerdictStoreError> {
        let prefix =
            VerdictStoreKey::ConclusionViolationsOfBlockPrefix(block_key, kind, conclusion_version)
                .to_key_bytes();
        self.entries_with_key_prefix_kept_in_or_out(prefix)
            .collect()
    }

    /// 这一块一种结论的结论违例行（[`Self::conclusion_violation_rows_as_stored`]）解码成违例，按块内序号排。
    ///
    /// # Errors
    /// 版本之后不是 8 字节的块内序号、值不是 4 字节的编号、编号没登记。
    fn conclusion_violations_from_the_rows_as_stored(
        &self,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
        conclusion_version: &ConclusionVersion,
        violation_rows_as_stored: Vec<StoredEntry>,
    ) -> Result<Vec<StoredViolation>, VerdictStoreError> {
        let prefix =
            VerdictStoreKey::ConclusionViolationsOfBlockPrefix(block_key, kind, conclusion_version)
                .to_key_bytes();
        let prefix_length = prefix.0.len();
        let mut violations = Vec::new();
        for (key_bytes, number_bytes) in violation_rows_as_stored {
            let offset_bytes =
                <[u8; STATE_OFFSET_IN_BLOCK_BYTES]>::try_from(&key_bytes[prefix_length..]);
            let (Ok(offset_bytes), Some(stored_number)) =
                (offset_bytes, stored_verdict_vector_number(&number_bytes))
            else {
                return Err(stored_bytes_mismatch(
                    VerdictStoreTable::ConclusionViolations,
                    &key_bytes,
                    "结论违例表里版本之后不是 8 字节的块内序号，或值不是 4 字节的编号",
                ));
            };
            violations.push(StoredViolation {
                offset_in_block: StateOffsetInBlock(u64::from_be_bytes(offset_bytes)),
                verdict_vector_number: self.registered_verdict_vector_number(
                    VerdictStoreTable::ConclusionViolations,
                    &key_bytes,
                    stored_number,
                )?,
            });
        }
        Ok(violations)
    }

    /// 一块要的几种结论里哪几种还没有（按给的次序）：「哪一种没有就只排那一种」。每一项的版本由调用方给：
    /// GPU 判器一把、这一块的设点文件那一把 CPU、挂在这个节点上的每个观察者各一把。
    ///
    /// # Errors
    /// 服务读不成；块属性那一行在而不合布局（跑法码不是这一版认得的也是：报错，不当没判）。
    pub fn conclusions_missing_for_block(
        &self,
        block_key: &VerdictBlockKey,
        required: &[(ConclusionKind, ConclusionVersion)],
    ) -> Result<Vec<(ConclusionKind, ConclusionVersion)>, VerdictStoreError> {
        // 一块要的几种结论一起问（一帧成批点读，不逐键一帧）；问的是块属性那一行（与结论块同一批写、值只有几十字节），不把整块结论读回来
        let key_bytes_of_each: Vec<VerdictStoreKeyBytes> = required
            .iter()
            .map(|(kind, conclusion_version)| {
                VerdictStoreKey::ConclusionBlockAttributes(block_key, kind, conclusion_version)
                    .to_key_bytes()
            })
            .collect();
        let keys: Vec<&[u8]> = key_bytes_of_each
            .iter()
            .map(|key_bytes| key_bytes.0.as_slice())
            .collect();
        let stored = self.read_values_kept_in_or_out(&keys)?;
        let mut missing = Vec::new();
        for (((kind, conclusion_version), attribute_value), attribute_key) in
            required.iter().zip(stored).zip(&key_bytes_of_each)
        {
            let Some(attribute_bytes) = attribute_value else {
                missing.push((kind.clone(), *conclusion_version));
                continue;
            };
            if conclusion_block_attributes_from_bytes(&attribute_bytes).is_none() {
                return Err(stored_bytes_mismatch(
                    VerdictStoreTable::ConclusionBlockAttributes,
                    &attribute_key.0,
                    "块属性的字节不合布局",
                ));
            }
        }
        Ok(missing)
    }

    /// 结论块表里的全部（块键，种类，版本），按键序：同一块几种结论、几把版本就有几行。扫的是块属性表（与结论块同一批写、键的后缀相同、
    /// 值只有几十字节），整表逐帧扫、只留键（[`Self::visit_stored_conclusion_keys`]）。
    ///
    /// # Errors
    /// 服务读不成；键字节认不回来。
    pub fn stored_conclusion_keys(
        &self,
    ) -> Result<Vec<(VerdictBlockKey, ConclusionKind, ConclusionVersion)>, VerdictStoreError> {
        let mut found = Vec::new();
        self.visit_stored_conclusion_keys(&mut |conclusion| {
            found.push(conclusion);
            Ok(())
        })?;
        Ok(found)
    }

    /// 库里存着的每一块一种结论一把版本（块属性表的键），按键序逐个交给 `visit_conclusion`，不攒成一份：读库程序数整份库的块用。
    ///
    /// # Errors
    /// 服务读不成；键字节认不回来；`visit_conclusion` 的错。
    pub fn visit_stored_conclusion_keys(
        &self,
        visit_conclusion: &mut dyn FnMut(
            ConclusionOfABlockUnderAKey,
        ) -> Result<(), VerdictStoreError>,
    ) -> Result<(), VerdictStoreError> {
        self.visit_rows_of_a_block_table(
            VerdictStoreTable::ConclusionBlockAttributes,
            &mut |block_key, suffix, _value| {
                let unreadable = || {
                    stored_bytes_mismatch(
                        VerdictStoreTable::ConclusionBlockAttributes,
                        suffix,
                        "块属性的键在块键之后要是结论种类加 32 字节的版本",
                    )
                };
                let (kind, version_bytes) =
                    conclusion_kind_from_key_bytes(suffix).ok_or_else(unreadable)?;
                let conclusion_version = <[u8; SHA256_DIGEST_BYTES]>::try_from(version_bytes)
                    .map(ConclusionVersion)
                    .map_err(|_| unreadable())?;
                visit_conclusion((block_key, kind, conclusion_version))
            },
        )
    }
}

#[cfg(test)]
mod tests;
