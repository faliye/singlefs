//! 重放幂等（I-8.4 的 R-换）：恢复加写行那一整段做两次与做一次，读出的内容比一比。

use super::*;

/// I-8.4（重放幂等）在这里判的那一条的编号：R-换（`research/prompts/m3-inv-a-r1-main-verification.md` 第四节）——
/// 在二次崩溃上判「恢复加写行那一整段」幂等，等价关系是读出的内容相同。池级 checker 不判它（单份镜像上恒真），判红时记在这一截的违例里。
pub const REPLAY_IDEMPOTENCE_INVARIANT: &str = "I-8.4";

/// 只读恢复（看 journal）读出的内容：I-8.4 的等价关系只比这个，不比扇区、实例代号、实例表行、恢复落到的那条根
/// （攻方 `research/prompts/m3-inv-a-r1-opus-output.md` 一·2：挂一次与崩在取号之后再挂一次，这几样都不同，读出的内容相同）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContentReadByRecovery {
    File(Vec<u8>),
    NoFile,
    /// 恢复失败，什么都没读出来。
    NothingRead,
}

impl ContentReadByRecovery {
    /// 给人看：几个字节、没有文件、没读出来（不把整份内容打出来）。
    #[must_use]
    pub fn render(&self) -> String {
        match self {
            ContentReadByRecovery::File(content) => format!("文件 {} 字节", content.len()),
            ContentReadByRecovery::NoFile => "没有文件".to_string(),
            ContentReadByRecovery::NothingRead => "恢复失败、没读出内容".to_string(),
        }
    }
}

/// 对 `reader` 跑一次只读恢复（看 journal），交回读出的内容。要恢复报告与挂载透明的字段的，走
/// `singlefs_verification::mount_transparency::read_recovery_fields`，再用 [`content_read_by_the_recovery_report`] 取内容。
#[must_use]
pub fn content_read_by_recovery(reader: &dyn PoolReader) -> ContentReadByRecovery {
    content_read_by_the_recovery_report(&recover(reader, JournalPolicy::Consult))
}

/// 一份只读恢复报告读出的内容。
#[must_use]
pub fn content_read_by_the_recovery_report(report: &RecoveryReport) -> ContentReadByRecovery {
    match &report.outcome {
        RecoveryOutcome::FileRead { content, .. } => ContentReadByRecovery::File(content.clone()),
        RecoveryOutcome::NoFile { .. } => ContentReadByRecovery::NoFile,
        RecoveryOutcome::Failed { .. } => ContentReadByRecovery::NothingRead,
    }
}

/// 二次崩溃之后再挂这一截为什么没判：这几种情形条款没写「做一次」或「做两次」的结果是什么，不替它定红绿，只计数（交主 agent）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RemountIdempotenceNotJudged {
    /// 没崩过的那一次可写挂载被拒：没有「做一次」的结果。
    TheUninterruptedMountWasRefused,
    /// 没崩过的那一次挂载之后只读恢复失败：「做一次」没读出内容，没有可比的。
    NothingWasReadAfterTheUninterruptedMount,
    /// 二次崩溃镜像上再挂被拒：「做两次」那一趟没做完。
    TheRemountWasRefused,
}

/// 「恢复加写行那一整段」做一次与做两次读出的内容比下来。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RemountIdempotence {
    SameContent,
    /// 读出的内容不同：判红。
    DifferentContent {
        once: ContentReadByRecovery,
        twice: ContentReadByRecovery,
    },
    NotJudged(RemountIdempotenceNotJudged),
}

/// I-8.4 的等价关系：两边读出的内容逐字节相同（都没有文件也算相同）。做一次那一边恢复失败时没有可比的，不判；
/// 做一次读出了东西、做两次恢复失败，算内容不同。
#[must_use]
pub fn compare_content_read_once_and_twice(
    once: &ContentReadByRecovery,
    twice: &ContentReadByRecovery,
) -> RemountIdempotence {
    match once {
        ContentReadByRecovery::NothingRead => RemountIdempotence::NotJudged(
            RemountIdempotenceNotJudged::NothingWasReadAfterTheUninterruptedMount,
        ),
        ContentReadByRecovery::File(_) | ContentReadByRecovery::NoFile => {
            if once == twice {
                RemountIdempotence::SameContent
            } else {
                RemountIdempotence::DifferentContent {
                    once: once.clone(),
                    twice: twice.clone(),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// I-8.4 的等价关系：读出的内容逐字节相同才算相同；一边读出 3000 字节、另一边差一个字节、少一截、没有文件或恢复失败，都判内容不同；
    /// 做一次那一边恢复失败时没有可比的，不判。
    #[test]
    fn content_read_twice_that_differs_from_content_read_once_is_red_and_the_same_content_is_not() {
        let once = ContentReadByRecovery::File(vec![7u8; 3000]);
        let mut one_byte_changed = vec![7u8; 3000];
        one_byte_changed[1234] = 8;
        assert_eq!(
            compare_content_read_once_and_twice(&once, &once.clone()),
            RemountIdempotence::SameContent,
            "逐字节相同的内容判相同"
        );
        assert_eq!(
            compare_content_read_once_and_twice(
                &ContentReadByRecovery::NoFile,
                &ContentReadByRecovery::NoFile
            ),
            RemountIdempotence::SameContent,
            "两边都没有文件判相同"
        );
        for twice in [
            ContentReadByRecovery::File(one_byte_changed),
            ContentReadByRecovery::File(vec![7u8; 2999]),
            ContentReadByRecovery::NoFile,
            ContentReadByRecovery::NothingRead,
        ] {
            assert_eq!(
                compare_content_read_once_and_twice(&once, &twice),
                RemountIdempotence::DifferentContent {
                    once: once.clone(),
                    twice: twice.clone()
                },
                "做两次读出 {} 与做一次读出的 3000 字节不同，要判红",
                twice.render()
            );
        }
        assert_eq!(
            compare_content_read_once_and_twice(
                &ContentReadByRecovery::NothingRead,
                &ContentReadByRecovery::NothingRead
            ),
            RemountIdempotence::NotJudged(
                RemountIdempotenceNotJudged::NothingWasReadAfterTheUninterruptedMount
            ),
            "做一次那一边没读出内容，没有可比的，不判"
        );
    }
}
