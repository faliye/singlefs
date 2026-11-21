//! oracle：恢复到的那一版对不对，按版本表判（同一 txg 两个实例算两版、根不在表里按更早的那一项判）。

use super::*;

/// oracle（E77（发布的持久顺序） 判据 1）：读回的内容要对；根槽已持久就不许恢复到旧态；走读不许失败。
/// 单版本形态：整条流只有一次带文件的发布（新池新建文件）。
#[must_use]
pub fn oracle_violation(
    outcome: &RecoveryOutcome,
    root_persisted: bool,
    expected_content: &[u8],
) -> Option<String> {
    match outcome {
        RecoveryOutcome::FileRead { content, .. } => {
            (content != expected_content).then(|| "读回的内容不对".to_string())
        }
        RecoveryOutcome::NoFile { .. } => {
            root_persisted.then(|| "根槽已持久而恢复到旧态".to_string())
        }
        RecoveryOutcome::Failed { failure, .. } => Some(format!("走读失败：{failure:?}")),
    }
}

/// 多版本形态的 oracle：实际走的根是哪一代，读回的就得是那一代写出的内容；根下面没有文件的那几代（mkfs、暖机）只许报没有文件；
/// 盘上已持久的最新根槽是 (T, 实例 i)，恢复就不许落到按 (txg, 实例) 字典序比它旧的根上（D22（单元原子性怎么合成） 已定项 7：
/// 择新 txg 为主、平局按实例代号高者赢）；走读不许失败。交回给人看的原因；哪一类见 [`classified_oracle_violation_for_versions`]。
#[must_use]
pub fn oracle_violation_for_versions(
    outcome: &RecoveryOutcome,
    effective_root: Option<(InstanceGeneration, CheckpointTxg)>,
    newest_persisted_root: Option<(CheckpointTxg, InstanceGeneration)>,
    versions: &[PublishedVersion],
) -> Option<String> {
    classified_oracle_violation_for_versions(
        outcome,
        effective_root,
        newest_persisted_root,
        versions,
    )
    .map(|violation| violation.reason)
}

/// 崩溃点重放一条流的版本表：带文件的那几版（[`PublishedVersion`]）与文件从那一版起没有了的那几版（[`PublishedVersionWithoutAFile`]，
/// D8（核心索引结构） 已定项 16 第 4 条：删除意图那次发布起冷走读答没有文件）。表按根（实例，txg）认；落到的根不在表里时，
/// 照比它旧的最新那一项判（按 (txg, 实例) 字典序，同 D22（单元原子性怎么合成） 已定项 7 的择根序）。
#[derive(Clone, Copy, Debug)]
pub struct CrashPointReplayVersionTable<'table> {
    versions_with_a_file: &'table [PublishedVersion],
    versions_without_a_file: &'table [PublishedVersionWithoutAFile],
}

/// 恢复落到的根在版本表里是哪一项。
#[derive(Clone, Copy, Debug)]
enum CrashPointReplayVersionAtTheRoot<'table> {
    /// 这一版有文件，读回的要是它的内容。
    WithAFile(&'table PublishedVersion),
    /// 文件从这一版起没有了。
    WithoutAFile,
    /// 表里没有这个根。
    NotInTheTable,
}

impl<'table> CrashPointReplayVersionTable<'table> {
    /// 有文件的那几版（崩溃点重放过库时抄进录下的流）。
    #[must_use]
    pub(crate) fn versions_with_a_file(&self) -> &'table [PublishedVersion] {
        self.versions_with_a_file
    }

    /// 文件从那一版起没有了的那几版（崩溃点重放过库时抄进录下的流）。
    #[must_use]
    pub(crate) fn versions_without_a_file(&self) -> &'table [PublishedVersionWithoutAFile] {
        self.versions_without_a_file
    }

    /// 没有一版是「文件从这一版起没有了」：文件一旦写出就不消失的流（删除之前的每一条流）。
    #[must_use]
    pub fn with_a_file_only(versions_with_a_file: &'table [PublishedVersion]) -> Self {
        Self {
            versions_with_a_file,
            versions_without_a_file: &[],
        }
    }

    /// # Panics
    /// 同一个根（实例，txg）既在带文件的那几版里、又在没有文件的那几版里：版本表自相矛盾，是用例写错了。
    #[must_use]
    pub fn new(
        versions_with_a_file: &'table [PublishedVersion],
        versions_without_a_file: &'table [PublishedVersionWithoutAFile],
    ) -> Self {
        for without_a_file in versions_without_a_file {
            assert!(
                !versions_with_a_file.iter().any(|with_a_file| {
                    (with_a_file.instance, with_a_file.checkpoint_txg)
                        == (without_a_file.instance, without_a_file.checkpoint_txg)
                }),
                "实例 {} 第 {} 代既写成有文件、又写成文件从这一版起没有了",
                without_a_file.instance.0,
                without_a_file.checkpoint_txg.0
            );
        }
        Self {
            versions_with_a_file,
            versions_without_a_file,
        }
    }

    fn version_at_the_root(
        &self,
        instance: InstanceGeneration,
        checkpoint_txg: CheckpointTxg,
    ) -> CrashPointReplayVersionAtTheRoot<'table> {
        if let Some(with_a_file) = self.versions_with_a_file.iter().find(|version| {
            version.checkpoint_txg == checkpoint_txg && version.instance == instance
        }) {
            return CrashPointReplayVersionAtTheRoot::WithAFile(with_a_file);
        }
        if self
            .versions_without_a_file
            .iter()
            .any(|version| version.checkpoint_txg == checkpoint_txg && version.instance == instance)
        {
            return CrashPointReplayVersionAtTheRoot::WithoutAFile;
        }
        CrashPointReplayVersionAtTheRoot::NotInTheTable
    }

    /// 不在表里的根照比它旧的最新那一项：那一项有文件就交回它（报没有文件是违例），那一项是没有文件、或没有更旧的项就交回 `None`。
    /// 文件从不消失的表里（[`Self::with_a_file_only`]）这与「任一更旧版有文件」相同。
    fn newest_older_version_if_it_has_a_file(
        &self,
        instance: InstanceGeneration,
        checkpoint_txg: CheckpointTxg,
    ) -> Option<&'table PublishedVersion> {
        let newest_older_with_a_file = self
            .versions_with_a_file
            .iter()
            .filter(|version| {
                (version.checkpoint_txg, version.instance) < (checkpoint_txg, instance)
            })
            .max_by_key(|version| (version.checkpoint_txg, version.instance));
        let newest_older_without_a_file = self
            .versions_without_a_file
            .iter()
            .map(|version| (version.checkpoint_txg, version.instance))
            .filter(|root| *root < (checkpoint_txg, instance))
            .max();
        match (newest_older_with_a_file, newest_older_without_a_file) {
            (Some(with_a_file), Some(without_a_file)) => {
                ((with_a_file.checkpoint_txg, with_a_file.instance) > without_a_file)
                    .then_some(with_a_file)
            }
            (Some(with_a_file), None) => Some(with_a_file),
            (None, Some(_) | None) => None,
        }
    }
}

/// 同 [`oracle_violation_for_versions`]，另交回违例是哪一类（[`CrashPointReplayOracleViolationKind`]，崩溃点重放发现表按它分签名）。
/// 文件从不消失的流：版本表是 [`CrashPointReplayVersionTable::with_a_file_only`]。
#[must_use]
pub fn classified_oracle_violation_for_versions(
    outcome: &RecoveryOutcome,
    effective_root: Option<(InstanceGeneration, CheckpointTxg)>,
    newest_persisted_root: Option<(CheckpointTxg, InstanceGeneration)>,
    versions: &[PublishedVersion],
) -> Option<CrashPointReplayOracleViolation> {
    classified_oracle_violation_for_the_version_table(
        outcome,
        effective_root,
        newest_persisted_root,
        CrashPointReplayVersionTable::with_a_file_only(versions),
    )
}

/// 崩溃点重放的 oracle 对一张版本表（[`CrashPointReplayVersionTable`]）判：落到的根是带文件的一版，读回的得是它的内容；是「文件从这一版起没有了」
/// 的一版，只许报没有文件；不在表里，报没有文件时看比它旧的最新那一项——那一项有文件就是违例（文件从不消失的流上与「任一更旧版有文件」
/// 相同），不在表里却读出了内容一律是违例。根槽已持久而恢复到旧态、走读失败、没择到根同 [`oracle_violation_for_versions`]。
#[must_use]
pub fn classified_oracle_violation_for_the_version_table(
    outcome: &RecoveryOutcome,
    effective_root: Option<(InstanceGeneration, CheckpointTxg)>,
    newest_persisted_root: Option<(CheckpointTxg, InstanceGeneration)>,
    versions: CrashPointReplayVersionTable<'_>,
) -> Option<CrashPointReplayOracleViolation> {
    let violation = |kind: CrashPointReplayOracleViolationKind, reason: String| {
        CrashPointReplayOracleViolation { kind, reason }
    };
    let Some((effective_instance, effective_txg)) = effective_root else {
        return Some(violation(
            CrashPointReplayOracleViolationKind::NoRootChosen,
            "没择到根".to_string(),
        ));
    };
    if let Some((newest_txg, newest_instance)) = newest_persisted_root {
        if (effective_txg, effective_instance) < (newest_txg, newest_instance) {
            return Some(violation(
                CrashPointReplayOracleViolationKind::RecoveredToAnOlderRoot,
                format!(
                "根槽已持久而恢复到旧态（盘上最新的根槽是实例 {} 第 {} 代，走的是实例 {} 第 {} 代）",
                newest_instance.0, newest_txg.0, effective_instance.0, effective_txg.0
            ),
            ));
        }
    }
    match (
        outcome,
        versions.version_at_the_root(effective_instance, effective_txg),
    ) {
        (RecoveryOutcome::Failed { failure, .. }, _) => Some(violation(
            CrashPointReplayOracleViolationKind::ReadFailed,
            format!("走读失败：{failure:?}"),
        )),
        // 落在一个表里没有的更新的根上报「没有文件」：比它旧的最新一项下面有文件时是违例（一律放过会漏判）；
        // 那一项是「文件从这一版起没有了」（删除那几批落在意图之后）时不是。
        (RecoveryOutcome::NoFile { .. }, CrashPointReplayVersionAtTheRoot::NotInTheTable) => versions
            .newest_older_version_if_it_has_a_file(effective_instance, effective_txg)
            .map(|older_version| {
                violation(
                    CrashPointReplayOracleViolationKind::NoFileWhereAnOlderRootHasOne,
                    format!(
                    "走到实例 {} 第 {} 代根报没有文件，而没有这一代的版本、实例 {} 第 {} 代下面有文件",
                    effective_instance.0,
                    effective_txg.0,
                    older_version.instance.0,
                    older_version.checkpoint_txg.0
                ),
                )
            }),
        (RecoveryOutcome::NoFile { .. }, CrashPointReplayVersionAtTheRoot::WithoutAFile) => None,
        (RecoveryOutcome::NoFile { .. }, CrashPointReplayVersionAtTheRoot::WithAFile(_)) => Some(violation(
            CrashPointReplayOracleViolationKind::NoFileWhereThisRootHasOne,
            format!("第 {} 代根下面有文件却报没有", effective_txg.0),
        )),
        (
            RecoveryOutcome::FileRead { .. },
            CrashPointReplayVersionAtTheRoot::NotInTheTable | CrashPointReplayVersionAtTheRoot::WithoutAFile,
        ) => Some(violation(
            CrashPointReplayOracleViolationKind::FileReadWhereThisRootHasNone,
            format!("第 {} 代根下面没有文件却读出了内容", effective_txg.0),
        )),
        (RecoveryOutcome::FileRead { content, .. }, CrashPointReplayVersionAtTheRoot::WithAFile(version)) => {
            (*content != version.content).then(|| {
                violation(
                    CrashPointReplayOracleViolationKind::WrongContent,
                    format!("读回的内容不对（走的是第 {} 代根）", effective_txg.0),
                )
            })
        }
    }
}

#[cfg(test)]
mod oracle_instance_tests {
    use super::{
        classified_oracle_violation_for_the_version_table,
        classified_oracle_violation_for_versions, oracle_violation_for_versions,
        CrashPointReplayOracleViolationKind, CrashPointReplayVersionTable, PublishedVersion,
        PublishedVersionWithoutAFile,
    };
    use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
    use singlefs_core::recovery::RecoveryOutcome;

    /// 两条同 txg 不同实例的版本（设备失而复得、或回退实例与被抛弃实例同时在环里会造出来）。
    fn two_instances_at_txg_seven() -> Vec<PublishedVersion> {
        vec![
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(7),
                content: b"written by instance one".to_vec(),
            },
            PublishedVersion {
                instance: InstanceGeneration(2),
                checkpoint_txg: CheckpointTxg(7),
                content: b"written by instance two".to_vec(),
            },
        ]
    }

    /// D22（单元原子性怎么合成） 已定项 7：txg 平局按实例代号高者赢。实例 2 的第 7 代根已持久而恢复落在实例 1 的第 7 代根上是一次退代。
    #[test]
    fn landing_on_the_lower_instance_of_the_same_txg_is_a_violation() {
        let outcome = RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(7)),
            content: b"written by instance one".to_vec(),
        };
        let violation = oracle_violation_for_versions(
            &outcome,
            Some((InstanceGeneration(1), CheckpointTxg(7))),
            Some((CheckpointTxg(7), InstanceGeneration(2))),
            &two_instances_at_txg_seven(),
        );
        assert!(
            violation.is_some(),
            "只按 txg 比会把实例 1 的第 7 代当成最新的"
        );
    }

    /// 走到一个没有版本的更新的根上报「没有文件」，而更旧的根下面有文件：违例，不许因为查不到版本就放过。
    #[test]
    fn newer_root_without_any_version_reporting_no_file_is_violation() {
        let outcome = RecoveryOutcome::NoFile {
            root: (InstanceGeneration(1), CheckpointTxg(9)),
        };
        assert!(oracle_violation_for_versions(
            &outcome,
            Some((InstanceGeneration(1), CheckpointTxg(9))),
            Some((CheckpointTxg(7), InstanceGeneration(2))),
            &two_instances_at_txg_seven(),
        )
        .is_some());
        // 比每条版本都旧的根（暖机）报没有文件照旧不是违例。
        let warm_up = RecoveryOutcome::NoFile {
            root: (InstanceGeneration(1), CheckpointTxg(2)),
        };
        assert_eq!(
            oracle_violation_for_versions(
                &warm_up,
                Some((InstanceGeneration(1), CheckpointTxg(2))),
                Some((CheckpointTxg(2), InstanceGeneration(1))),
                &two_instances_at_txg_seven(),
            ),
            None
        );
    }

    /// 多记录流（`crates/singlefs-verification/tests/crash_enumeration_multi_record_publish_stream.rs`）的版本表：实例 1 的
    /// A、B、C 三次带文件的发布落在第 3、4、5 代（第 1、2 代是两次暖机，根下面没有文件）。
    fn the_multi_record_publish_stream_versions() -> Vec<PublishedVersion> {
        [
            (3, b"A".as_slice()),
            (4, b"B".as_slice()),
            (5, b"C".as_slice()),
        ]
        .into_iter()
        .map(|(checkpoint_txg, content)| PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(checkpoint_txg),
            content: content.to_vec(),
        })
        .collect()
    }

    /// 改「根不在表里只看比它旧的最新一版」之前的判法（逐字照那一版的分支写）：报没有文件、根不在表里，任一更旧版有文件就是违例。
    fn kind_judged_by_any_older_version_with_a_file(
        outcome: &RecoveryOutcome,
        effective_root: (InstanceGeneration, CheckpointTxg),
        newest_persisted_root: Option<(CheckpointTxg, InstanceGeneration)>,
        versions: &[PublishedVersion],
    ) -> Option<CrashPointReplayOracleViolationKind> {
        let (effective_instance, effective_txg) = effective_root;
        if newest_persisted_root.is_some_and(|newest| (effective_txg, effective_instance) < newest)
        {
            return Some(CrashPointReplayOracleViolationKind::RecoveredToAnOlderRoot);
        }
        let version = versions.iter().find(|version| {
            version.checkpoint_txg == effective_txg && version.instance == effective_instance
        });
        match (outcome, version) {
            (RecoveryOutcome::Failed { .. }, _) => {
                Some(CrashPointReplayOracleViolationKind::ReadFailed)
            }
            (RecoveryOutcome::NoFile { .. }, None) => versions
                .iter()
                .any(|candidate_version| {
                    (candidate_version.checkpoint_txg, candidate_version.instance)
                        < (effective_txg, effective_instance)
                })
                .then_some(CrashPointReplayOracleViolationKind::NoFileWhereAnOlderRootHasOne),
            (RecoveryOutcome::NoFile { .. }, Some(_)) => {
                Some(CrashPointReplayOracleViolationKind::NoFileWhereThisRootHasOne)
            }
            (RecoveryOutcome::FileRead { .. }, None) => {
                Some(CrashPointReplayOracleViolationKind::FileReadWhereThisRootHasNone)
            }
            (RecoveryOutcome::FileRead { content, .. }, Some(version)) => (*content
                != version.content)
                .then_some(CrashPointReplayOracleViolationKind::WrongContent),
        }
    }

    /// 文件从不消失的流上，「根不在表里只看比它旧的最新一版」与「任一更旧版有文件」判得逐项相同：多记录流与两实例同 txg 两张版本表，
    /// 实例 0–2、第 0–8 代的每个根，报没有文件、读出表里每一版的内容、读出表里没有的内容，盘上最新根槽没有或就是这个根，两种判法交回的违例类相同。
    #[test]
    fn table_where_the_file_never_vanishes_judges_a_root_not_in_it_like_any_older_version_with_a_file(
    ) {
        for versions in [
            the_multi_record_publish_stream_versions(),
            two_instances_at_txg_seven(),
        ] {
            let mut judged_pairs = 0usize;
            for instance in 0..=2 {
                for checkpoint_txg in 0..=8 {
                    let root = (InstanceGeneration(instance), CheckpointTxg(checkpoint_txg));
                    let mut outcomes = vec![RecoveryOutcome::NoFile { root }];
                    outcomes.extend(
                        versions
                            .iter()
                            .map(|version| version.content.clone())
                            .chain([b"not written by any version".to_vec()])
                            .map(|content| RecoveryOutcome::FileRead { root, content }),
                    );
                    for outcome in &outcomes {
                        for newest_persisted_root in [None, Some((root.1, root.0))] {
                            assert_eq!(
                                classified_oracle_violation_for_versions(
                                    outcome,
                                    Some(root),
                                    newest_persisted_root,
                                    &versions,
                                )
                                .map(|violation| violation.kind),
                                kind_judged_by_any_older_version_with_a_file(
                                    outcome,
                                    root,
                                    newest_persisted_root,
                                    &versions,
                                ),
                                "根 {root:?}、{outcome:?}、盘上最新根槽 {newest_persisted_root:?}：两种判法应当相同"
                            );
                            judged_pairs += 1;
                        }
                    }
                }
            }
            assert_eq!(
                judged_pairs,
                3 * 9 * (versions.len() + 2) * 2,
                "每个根、每种结局、两种最新根槽都判到了"
            );
        }
    }

    /// 多记录流之后 unlink 的版本表（`crates/singlefs-verification/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs`）：
    /// A、B、C 有文件，第 6 代（删除意图那次）起没有文件，之后两批与最后一批（第 7–9 代）不进表。
    fn versions_without_a_file_from_txg_six() -> Vec<PublishedVersionWithoutAFile> {
        vec![PublishedVersionWithoutAFile {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(6),
        }]
    }

    fn kind_judged_against_the_table(
        outcome: &RecoveryOutcome,
        effective_root: (InstanceGeneration, CheckpointTxg),
        versions: CrashPointReplayVersionTable<'_>,
    ) -> Option<CrashPointReplayOracleViolationKind> {
        classified_oracle_violation_for_the_version_table(
            outcome,
            Some(effective_root),
            Some((effective_root.1, effective_root.0)),
            versions,
        )
        .map(|violation| violation.kind)
    }

    /// D8（核心索引结构） 已定项 16 第 4 条：意图那次起冷走读答没有文件。落在意图那一版、或落在它之后不在表里的根（各批）上报没有文件不是违例；
    /// 在那几个根上读出内容、落在 C 上报没有文件照旧是违例。
    #[test]
    fn no_file_from_the_version_without_a_file_on_is_not_a_violation_and_a_file_read_there_is() {
        let with_a_file = the_multi_record_publish_stream_versions();
        let without_a_file = versions_without_a_file_from_txg_six();
        let table = CrashPointReplayVersionTable::new(&with_a_file, &without_a_file);
        for checkpoint_txg in 6..=9 {
            let root = (InstanceGeneration(1), CheckpointTxg(checkpoint_txg));
            assert_eq!(
                kind_judged_against_the_table(&RecoveryOutcome::NoFile { root }, root, table),
                None,
                "第 {checkpoint_txg} 代（意图那次或之后的一批）报没有文件不是违例"
            );
            assert_eq!(
                kind_judged_against_the_table(
                    &RecoveryOutcome::FileRead {
                        root,
                        content: b"C".to_vec(),
                    },
                    root,
                    table
                ),
                Some(CrashPointReplayOracleViolationKind::FileReadWhereThisRootHasNone),
                "第 {checkpoint_txg} 代起没有文件，读出 C 的内容是违例"
            );
        }
        let root_of_the_last_version_with_a_file = (InstanceGeneration(1), CheckpointTxg(5));
        assert_eq!(
            kind_judged_against_the_table(
                &RecoveryOutcome::NoFile {
                    root: root_of_the_last_version_with_a_file
                },
                root_of_the_last_version_with_a_file,
                table
            ),
            Some(CrashPointReplayOracleViolationKind::NoFileWhereThisRootHasOne),
            "C 那一版有文件，报没有是违例"
        );
    }

    /// 根不在表里只看比它旧的最新那一项：文件没了之后又有一版带文件的（第 8 代），落在它后面的第 9 代报没有文件是违例、点名第 8 代；
    /// 落在没有文件那一版与第 8 代之间的第 7 代报没有文件不是违例。
    #[test]
    fn root_not_in_the_table_is_judged_by_the_newest_older_entry_only() {
        let with_a_file = vec![
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(3),
                content: b"A".to_vec(),
            },
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(8),
                content: b"written after the file vanished".to_vec(),
            },
        ];
        let without_a_file = versions_without_a_file_from_txg_six();
        let table = CrashPointReplayVersionTable::new(&with_a_file, &without_a_file);
        let root_after_the_newer_file = (InstanceGeneration(1), CheckpointTxg(9));
        let violation = classified_oracle_violation_for_the_version_table(
            &RecoveryOutcome::NoFile {
                root: root_after_the_newer_file,
            },
            Some(root_after_the_newer_file),
            None,
            table,
        )
        .expect("比第 9 代旧的最新一项是第 8 代、有文件：报没有文件是违例");
        assert_eq!(
            violation.kind,
            CrashPointReplayOracleViolationKind::NoFileWhereAnOlderRootHasOne
        );
        assert!(
            violation.reason.contains("实例 1 第 8 代下面有文件"),
            "违例点名比它旧的最新那一版：{}",
            violation.reason
        );
        let root_between = (InstanceGeneration(1), CheckpointTxg(7));
        assert_eq!(
            kind_judged_against_the_table(
                &RecoveryOutcome::NoFile { root: root_between },
                root_between,
                table
            ),
            None,
            "比第 7 代旧的最新一项是第 6 代、没有文件"
        );
    }

    /// 同一个根既写成有文件、又写成没有文件：版本表自相矛盾，构造时停下。
    #[test]
    #[should_panic(expected = "既写成有文件、又写成文件从这一版起没有了")]
    fn root_both_with_and_without_a_file_is_refused_when_the_table_is_built() {
        let with_a_file = the_multi_record_publish_stream_versions();
        let without_a_file = vec![PublishedVersionWithoutAFile {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(5),
        }];
        let _table = CrashPointReplayVersionTable::new(&with_a_file, &without_a_file);
    }

    /// 同 txg 的两条版本是两条版本：落在实例 2 上读出实例 2 的内容不是违例。
    #[test]
    fn the_same_txg_from_two_instances_are_two_versions() {
        let outcome = RecoveryOutcome::FileRead {
            root: (InstanceGeneration(2), CheckpointTxg(7)),
            content: b"written by instance two".to_vec(),
        };
        assert_eq!(
            oracle_violation_for_versions(
                &outcome,
                Some((InstanceGeneration(2), CheckpointTxg(7))),
                Some((CheckpointTxg(7), InstanceGeneration(2))),
                &two_instances_at_txg_seven(),
            ),
            None
        );
    }
}
