//! 分片进度表（`13`）：按段登记的段整域装不进 u64 时，每一段判到哪一块。

use super::*;

/// 分片进度记在哪一个判器、哪一把版本下（进度记录键的最后一截）：与它推进时那几块结论存在哪一处是同一处。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JudgeOfTheProgress {
    /// 判法单元指纹那一套（结论按（块键，判法单元指纹）存）。
    TheJudgingUnitsFingerprint(JudgingUnitsFingerprint),
    /// 三种结论里的一种，连这个段所在的点那一把版本。
    TheConclusionKind {
        kind: ConclusionKind,
        version: ConclusionVersion,
    },
}

/// 一条分片进度记录的键：这一段的内容摘要（它所在的点的段写表、起步镜像与计划哈希，连同它在点内的段序；不带流名）、判器
/// （[`JudgeOfTheProgress`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlicedSegmentProgressKey {
    pub segment_content_digest: [u8; SHA256_DIGEST_BYTES],
    pub judge: JudgeOfTheProgress,
}

/// 分片进度键里段的内容摘要之后那一个字节：判器是判法单元指纹那一套。
pub(super) const SLICED_PROGRESS_JUDGE_MARKER_OF_THE_JUDGING_UNITS_FINGERPRINT: u8 = 0x01;
/// 分片进度键里段的内容摘要之后那一个字节：判器是三种结论里的一种（后面接结论种类与版本）。
pub(super) const SLICED_PROGRESS_JUDGE_MARKER_OF_A_CONCLUSION_KIND: u8 = 0x02;

impl SlicedSegmentProgressKey {
    pub(super) fn to_key_bytes_after_the_tag(&self) -> Vec<u8> {
        let mut key_bytes = self.segment_content_digest.to_vec();
        match &self.judge {
            JudgeOfTheProgress::TheJudgingUnitsFingerprint(judging_units_fingerprint) => {
                key_bytes.push(SLICED_PROGRESS_JUDGE_MARKER_OF_THE_JUDGING_UNITS_FINGERPRINT);
                key_bytes.extend_from_slice(judging_units_fingerprint.0.as_slice());
            }
            JudgeOfTheProgress::TheConclusionKind { kind, version } => {
                key_bytes.push(SLICED_PROGRESS_JUDGE_MARKER_OF_A_CONCLUSION_KIND);
                key_bytes.extend_from_slice(&conclusion_kind_key_bytes(kind));
                key_bytes.extend_from_slice(version.0.as_slice());
            }
        }
        key_bytes
    }
}

/// 一段分片的进度：段内编号 [0, `judged_through`) 的块都判完了；`last_block_start` 是最后判完的那一块的起点
/// （`judged_through` 是 0 时也写 0）。读回的人照它核「这一块在、`judged_through` 那一块不在」，核不过就不信它。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlicedSegmentProgress {
    pub judged_through: StateCount1280Bits,
    pub last_block_start: StateCount1280Bits,
}

impl SlicedSegmentProgress {
    /// 存进库的字节：已判到的编号、最后判完的那一块的起点，各 [`STATE_COUNT_BYTES`] 字节小端，没有别的头。
    fn to_value_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(2 * STATE_COUNT_BYTES);
        bytes.extend_from_slice(&self.judged_through.to_le_bytes());
        bytes.extend_from_slice(&self.last_block_start.to_le_bytes());
        bytes
    }

    /// [`Self::to_value_bytes`] 的逆；长度不是两个编号交回 `None`。
    fn from_value_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 2 * STATE_COUNT_BYTES {
            return None;
        }
        let (judged_through, last_block_start) = bytes.split_at(STATE_COUNT_BYTES);
        Some(Self {
            judged_through: StateCount1280Bits::from_le_bytes(judged_through.try_into().ok()?),
            last_block_start: StateCount1280Bits::from_le_bytes(last_block_start.try_into().ok()?),
        })
    }
}

impl VerdictStore {
    /// 写一条分片进度记录（同步落盘）：调用方在推进到的那几块的判定落盘之后才写它。
    ///
    /// # Errors
    /// 服务写不成。
    pub fn store_sliced_segment_progress(
        &mut self,
        key: &SlicedSegmentProgressKey,
        progress: SlicedSegmentProgress,
    ) -> Result<(), VerdictStoreError> {
        self.put_one_synced(
            &VerdictStoreKey::SlicedSegmentProgress(key),
            &progress.to_value_bytes(),
        )
    }

    /// 读一条分片进度记录；没写过交回 None。
    ///
    /// # Errors
    /// 服务读不成；值不合 [`SlicedSegmentProgress`] 的布局。
    pub fn read_sliced_segment_progress(
        &self,
        key: &SlicedSegmentProgressKey,
    ) -> Result<Option<SlicedSegmentProgress>, VerdictStoreError> {
        let key_bytes = VerdictStoreKey::SlicedSegmentProgress(key).to_key_bytes();
        let Some(stored) = self.read_value_kept_in_or_out(&key_bytes.0)? else {
            return Ok(None);
        };
        SlicedSegmentProgress::from_value_bytes(&stored)
            .map(Some)
            .ok_or_else(|| {
                stored_bytes_mismatch(
                    VerdictStoreTable::SlicedSegmentProgress,
                    &key_bytes.0,
                    "分片进度的值不是两个 160 字节的编号",
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 分片进度（C790）：一条记录按（流、段、判器）存、读回逐字相同，换了判器或段读不到；值的字节不合布局时报布局对不上。
    /// 判别力（`crates/mutations.tsv`）：写值时两个编号次序对调，这条红。
    #[test]
    fn a_sliced_segment_progress_reads_back_under_its_flow_segment_and_judge_only() {
        let library = TestLibrary::new("sliced-progress");
        let mut store = library.open();
        let key_of = |segment_index: u8, version_byte: u8| SlicedSegmentProgressKey {
            segment_content_digest: [segment_index; SHA256_DIGEST_BYTES],
            judge: JudgeOfTheProgress::TheConclusionKind {
                kind: ConclusionKind::ImageJudgement,
                version: ConclusionVersion([version_byte; SHA256_DIGEST_BYTES]),
            },
        };
        assert_eq!(
            store
                .read_sliced_segment_progress(&key_of(1, 7))
                .expect("读"),
            None
        );
        let judged_through = StateCount1280Bits::from_u64(u64::MAX)
            .checked_mul_u64(1 << 40)
            .expect("装得进 1280 位");
        let progress = SlicedSegmentProgress {
            judged_through,
            last_block_start: StateCount1280Bits::from_u64(300),
        };
        store
            .store_sliced_segment_progress(&key_of(1, 7), progress)
            .expect("写");
        assert_eq!(
            store
                .read_sliced_segment_progress(&key_of(1, 7))
                .expect("读"),
            Some(progress)
        );
        assert_eq!(
            store
                .read_sliced_segment_progress(&key_of(1, 8))
                .expect("读"),
            None,
            "换了判法单元指纹读不到"
        );
        assert_eq!(
            store
                .read_sliced_segment_progress(&key_of(2, 7))
                .expect("读"),
            None,
            "换了段读不到"
        );
        store
            .put_one_synced(
                &VerdictStoreKey::SlicedSegmentProgress(&key_of(3, 7)),
                &[1, 2, 3],
            )
            .expect("写");
        assert!(matches!(
            store.read_sliced_segment_progress(&key_of(3, 7)),
            Err(VerdictStoreError::StoredBytesDoNotMatchTheLayout { .. })
        ));
    }

    /// 分片的段的一块（C790）：录过、没判过的块换一个状态数照新的改写（两趟之间片大小改了）；判过的块换状态数照旧拒、录入表不动。
    #[test]
    fn a_slice_block_recorded_but_not_judged_is_recorded_again_with_its_new_length_and_a_judged_one_is_refused(
    ) {
        let library = TestLibrary::new("slice-block-rerecord");
        let mut store = library.open();
        let unjudged = block_key_named("点", 300);
        store.record_block(&unjudged, 300).expect("录");
        store
            .record_block_of_a_slice(&unjudged, 500)
            .expect("没判过的照新长度改写");
        assert_eq!(
            store.recorded_state_count_of_block(&unjudged).expect("读"),
            Some(500)
        );
        let judged = block_key_named("点", 0);
        store.record_block(&judged, 2).expect("录");
        let clean = store
            .register_verdict_vector(&verdict_vector(b"clean"))
            .expect("登记");
        store
            .store_verdict_block(
                &judged,
                &TEST_JUDGING_UNITS_FINGERPRINT,
                &block_of(&[clean, clean]),
                &[],
            )
            .expect("存块");
        assert!(matches!(
            store.record_block_of_a_slice(&judged, 3),
            Err(VerdictStoreError::DifferentVerdictsUnderAStoredBlockKeyNotDecidedInTheFirstVersion { .. })
        ));
        assert_eq!(
            store.recorded_state_count_of_block(&judged).expect("读"),
            Some(2)
        );
        assert_eq!(
            store
                .recorded_state_count_of_block(&block_key_named("点", 9))
                .expect("读"),
            None
        );
    }
}
