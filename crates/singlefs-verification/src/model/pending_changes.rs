//! 攒批会话的语义模型：收下的改动先进待发，发布点才施加。

use super::*;

/// 攒批的会话的语义模型（D16（发布语义） 已定项 2、10；判决 `research/prompts/m4-publish-trigger-r1-main-verification.md` 第二节「返回即落盘」「RefFS」
/// 两格：随机历史与模型对拍「返回即落盘」的假设改成按会话报告的发布点判）：收下的改动先进待发、不改「盘上读得到」的那一份，只有会话报告一次发布时
/// 待发才按收下的次序施加到那一份上（同一个文件的几次整份写，后到的盖掉先到的；建 inode 按收下的次序从水位起发号）。
///
/// 触发照条款另写一份：待发非空 ∧（now − 上次发布的读数 ≥ `T_time` ∨ 待发 journal 字节 ≥ 有效 `T_dirty`），读数倒退当作没过时间、上次发布的读数不回拨，
/// 第一次见到的读数当基线。待发要写多少 journal 字节模型算不出（要走实现的固定点），取实现报的每一次改动单独发布要写的字节，只核它的下界：
/// 每一次改动至少一条记录（元数据改动也计入，D16 已定项 5），批内同一个文件写 K 次照 K 次加、不折叠。
///
/// 只 `use singlefs_format`（同 `model.rs` 开头那一条，D13（验证路线） 已定项 5）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelOfThePendingChanges {
    time_threshold_in_milliseconds: u64,
    dirty_threshold_in_journal_bytes: u64,
    reading_of_the_last_publish_in_milliseconds: Option<u64>,
    published: ModelFilesReadableAfterThePublishes,
    pending: Vec<ModelPendingChange>,
    journal_bytes_of_the_pending_changes: u64,
}

/// 最近一次发布之后盘上读得到的文件：每个 inode 号的整份内容（建出来还没写过的是空的）与下一个要发的 inode 号。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelFilesReadableAfterThePublishes {
    pub contents_by_inode: BTreeMap<u64, Vec<u8>>,
    pub inode_number_watermark: u64,
}

/// 收进待发的一次改动。封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelPendingChange {
    WholeWrite { inode: u64, content: Vec<u8> },
    NewInodes { count: u64 },
}

/// 模型判的这一次触发是哪一支先到。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelPublishTriggerCause {
    TimeThreshold,
    DirtyThreshold,
    BothThresholds,
}

/// 实现报的一次改动单独发布要写的 journal 字节低于一条记录：元数据改动也要写记录（D16（发布语义） 已定项 5「元数据改动也计入」），一次发布至少一条。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelJournalBytesOfOneChangeBelowOneRecord {
    pub reported: u64,
    pub at_least: u64,
}

impl ModelOfThePendingChanges {
    /// 一次发布之后起步：盘上读得到 `published`，待发空，还没见过读数。两个阈值是这次挂载的有效值（时间毫秒、脏量 journal 字节）。
    #[must_use]
    pub fn after_a_publish(
        published: ModelFilesReadableAfterThePublishes,
        time_threshold_in_milliseconds: u64,
        dirty_threshold_in_journal_bytes: u64,
    ) -> Self {
        Self {
            time_threshold_in_milliseconds,
            dirty_threshold_in_journal_bytes,
            reading_of_the_last_publish_in_milliseconds: None,
            published,
            pending: Vec::new(),
            journal_bytes_of_the_pending_changes: 0,
        }
    }

    /// 最近一次发布之后盘上读得到的文件。
    #[must_use]
    pub fn published(&self) -> &ModelFilesReadableAfterThePublishes {
        &self.published
    }

    #[must_use]
    pub fn pending_is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    /// 待发里那几次改动要写的 journal 字节之和（实现报的逐次相加）。
    #[must_use]
    pub fn journal_bytes_of_the_pending_changes(&self) -> u64 {
        self.journal_bytes_of_the_pending_changes
    }

    /// 一次调用交进来的读数：还没见过读数时它是基线。
    pub fn note_a_reading(&mut self, now_in_milliseconds: u64) {
        if self.reading_of_the_last_publish_in_milliseconds.is_none() {
            self.reading_of_the_last_publish_in_milliseconds = Some(now_in_milliseconds);
        }
    }

    /// 收下一次改动，脏量加上实现报的这一次单独发布要写的 journal 字节。
    ///
    /// # Errors
    /// 报的字节低于一条记录（[`ModelJournalBytesOfOneChangeBelowOneRecord`]）：模型不动。
    pub fn accept(
        &mut self,
        change: ModelPendingChange,
        journal_bytes_of_this_change_reported: u64,
    ) -> Result<(), ModelJournalBytesOfOneChangeBelowOneRecord> {
        if journal_bytes_of_this_change_reported < JOURNAL_RECORD_BYTES {
            return Err(ModelJournalBytesOfOneChangeBelowOneRecord {
                reported: journal_bytes_of_this_change_reported,
                at_least: JOURNAL_RECORD_BYTES,
            });
        }
        self.pending.push(change);
        self.journal_bytes_of_the_pending_changes = self
            .journal_bytes_of_the_pending_changes
            .saturating_add(journal_bytes_of_this_change_reported);
        Ok(())
    }

    /// 此刻（`now_in_milliseconds`）该不该触发、哪一支先到。待发空时不触发（空闲时不推空发布）。
    #[must_use]
    pub fn expected_trigger(&self, now_in_milliseconds: u64) -> Option<ModelPublishTriggerCause> {
        if self.pending.is_empty() {
            return None;
        }
        let milliseconds_since_the_last_publish = self
            .reading_of_the_last_publish_in_milliseconds
            .map_or(0, |last| now_in_milliseconds.saturating_sub(last));
        let is_time_reached =
            milliseconds_since_the_last_publish >= self.time_threshold_in_milliseconds;
        let is_dirty_reached =
            self.journal_bytes_of_the_pending_changes >= self.dirty_threshold_in_journal_bytes;
        match (is_time_reached, is_dirty_reached) {
            (true, true) => Some(ModelPublishTriggerCause::BothThresholds),
            (true, false) => Some(ModelPublishTriggerCause::TimeThreshold),
            (false, true) => Some(ModelPublishTriggerCause::DirtyThreshold),
            (false, false) => None,
        }
    }

    /// 会话报了一次排空待发的发布：待发按收下的次序施加到盘上读得到的那一份上，清空；上次发布的读数取这一次的（不回拨）。
    pub fn publish_the_pending_changes(&mut self, now_in_milliseconds: u64) {
        for change in std::mem::take(&mut self.pending) {
            match change {
                ModelPendingChange::WholeWrite { inode, content } => {
                    self.published.contents_by_inode.insert(inode, content);
                }
                ModelPendingChange::NewInodes { count } => {
                    for _created in 0..count {
                        self.published
                            .contents_by_inode
                            .insert(self.published.inode_number_watermark, Vec::new());
                        self.published.inode_number_watermark += 1;
                    }
                }
            }
        }
        self.journal_bytes_of_the_pending_changes = 0;
        self.note_a_publish(now_in_milliseconds);
    }

    /// 会话报了一次不排空待发的发布（待发本来就空时的单发）：只记读数。
    pub fn note_a_publish(&mut self, now_in_milliseconds: u64) {
        self.reading_of_the_last_publish_in_milliseconds = Some(
            self.reading_of_the_last_publish_in_milliseconds
                .map_or(now_in_milliseconds, |last| last.max(now_in_milliseconds)),
        );
    }
}
