//! 被宽护卡住时拷下的这次改动与它剩下的那一段（[`UserChangeKeptByTheRefusal`]）：要出路时读回成交给预演的改动。

use super::*;

/// 被拒的那一次发布之后，这次用户改动还剩什么要发（「被宽护卡住」那一报记一份拷贝，要出路时推完环之后照它再预演一次，
/// [`rehearse_the_push_then_the_remainder_of_the_change`]）。
#[derive(Clone, Copy, Debug)]
pub(super) enum RemainderOfTheUserChange<'content> {
    /// 先把现行那一版的 `deleted_inodes` 走空、再发这次改动（unlink 连同它之后的走空）：被拒的是走空的一步或这次改动本身。
    WalkTheQueueThenPublishTheChange(UserChange<'content>),
    /// 只剩走空：unlink 的意图已发成，被拒的是它之后走空的一步（别的改动之后队列本来就空）。
    WalkTheQueue,
}

/// [`RemainderOfTheUserChange`] 记进「被宽护卡住」那一份报里的拷贝：错误活得比调用方借给发布的内容长，内容拷一份。
#[derive(Debug)]
pub(super) enum RemainderOfTheUserChangeKeptByTheRefusal {
    WalkTheQueueThenPublishTheChange(UserChangeKeptByTheRefusal),
    WalkTheQueue,
}

/// [`UserChange`] 的一份拷贝，内容自有。
#[derive(Debug)]
pub(super) enum UserChangeKeptByTheRefusal {
    Overwrite {
        content: Vec<u8>,
        write_time_seconds: u64,
    },
    SequentialWrite {
        content: Vec<u8>,
        write_time_seconds: u64,
    },
    NewInodes {
        count: u64,
        write_time_seconds: u64,
    },
    Unlink {
        inode: InodeNumber,
        write_time_seconds: u64,
    },
    CreateSnapshot,
    DestroySnapshot {
        snapshot: TreeIdentifier,
    },
    Truncate {
        inode: InodeNumber,
        new_size: u64,
        write_time_seconds: u64,
    },
    OverwriteOfAnInode {
        inode: InodeNumber,
        content: Vec<u8>,
        write_time_seconds: u64,
    },
    SequentialWriteOfAnInode {
        inode: InodeNumber,
        content: Vec<u8>,
        write_time_seconds: u64,
    },
    /// 字节至少一个（从 [`PartialWriteBytes`] 拷来的）。
    WriteAtOffset {
        inode: InodeNumber,
        offset: FileOffsetInBytes,
        bytes: Vec<u8>,
        write_time_seconds: u64,
    },
    /// 整份写的那个文件（号、内容、时间秒）与几次建 inode，内容自有。
    DataChangesPublishedTogether {
        whole_write: Option<(InodeNumber, Vec<u8>, u64)>,
        new_inodes: Vec<NewInodesOfOneChange>,
    },
}

impl UserChangeKeptByTheRefusal {
    fn kept_from(change: UserChange<'_>) -> Self {
        match change {
            UserChange::Overwrite(file) => UserChangeKeptByTheRefusal::Overwrite {
                content: file.content.to_vec(),
                write_time_seconds: file.write_time_seconds,
            },
            UserChange::SequentialWrite(file) => UserChangeKeptByTheRefusal::SequentialWrite {
                content: file.content.to_vec(),
                write_time_seconds: file.write_time_seconds,
            },
            UserChange::NewInodes {
                count,
                write_time_seconds,
            } => UserChangeKeptByTheRefusal::NewInodes {
                count,
                write_time_seconds,
            },
            UserChange::Unlink {
                inode,
                write_time_seconds,
            } => UserChangeKeptByTheRefusal::Unlink {
                inode,
                write_time_seconds,
            },
            UserChange::CreateSnapshot => UserChangeKeptByTheRefusal::CreateSnapshot,
            UserChange::DestroySnapshot { snapshot } => {
                UserChangeKeptByTheRefusal::DestroySnapshot { snapshot }
            }
            UserChange::Truncate {
                inode,
                new_size,
                write_time_seconds,
            } => UserChangeKeptByTheRefusal::Truncate {
                inode,
                new_size,
                write_time_seconds,
            },
            UserChange::OverwriteOfAnInode {
                inode,
                content,
                write_time_seconds,
            } => UserChangeKeptByTheRefusal::OverwriteOfAnInode {
                inode,
                content: content.to_vec(),
                write_time_seconds,
            },
            UserChange::SequentialWriteOfAnInode {
                inode,
                content,
                write_time_seconds,
            } => UserChangeKeptByTheRefusal::SequentialWriteOfAnInode {
                inode,
                content: content.to_vec(),
                write_time_seconds,
            },
            UserChange::WriteAtOffset {
                inode,
                offset,
                bytes,
                write_time_seconds,
            } => UserChangeKeptByTheRefusal::WriteAtOffset {
                inode,
                offset,
                bytes: bytes.bytes().to_vec(),
                write_time_seconds,
            },
            UserChange::DataChangesPublishedTogether(batch) => {
                UserChangeKeptByTheRefusal::DataChangesPublishedTogether {
                    whole_write: batch.whole_write.map(|whole_write| {
                        (
                            whole_write.inode,
                            whole_write.content.to_vec(),
                            whole_write.write_time_seconds,
                        )
                    }),
                    new_inodes: batch.new_inodes.to_vec(),
                }
            }
        }
    }

    fn as_user_change(&self) -> UserChange<'_> {
        match self {
            UserChangeKeptByTheRefusal::Overwrite {
                content,
                write_time_seconds,
            } => UserChange::Overwrite(FirstFile {
                content,
                write_time_seconds: *write_time_seconds,
            }),
            UserChangeKeptByTheRefusal::SequentialWrite {
                content,
                write_time_seconds,
            } => UserChange::SequentialWrite(FirstFile {
                content,
                write_time_seconds: *write_time_seconds,
            }),
            UserChangeKeptByTheRefusal::NewInodes {
                count,
                write_time_seconds,
            } => UserChange::NewInodes {
                count: *count,
                write_time_seconds: *write_time_seconds,
            },
            UserChangeKeptByTheRefusal::Unlink {
                inode,
                write_time_seconds,
            } => UserChange::Unlink {
                inode: *inode,
                write_time_seconds: *write_time_seconds,
            },
            UserChangeKeptByTheRefusal::CreateSnapshot => UserChange::CreateSnapshot,
            UserChangeKeptByTheRefusal::DestroySnapshot { snapshot } => {
                UserChange::DestroySnapshot {
                    snapshot: *snapshot,
                }
            }
            UserChangeKeptByTheRefusal::Truncate {
                inode,
                new_size,
                write_time_seconds,
            } => UserChange::Truncate {
                inode: *inode,
                new_size: *new_size,
                write_time_seconds: *write_time_seconds,
            },
            UserChangeKeptByTheRefusal::OverwriteOfAnInode {
                inode,
                content,
                write_time_seconds,
            } => UserChange::OverwriteOfAnInode {
                inode: *inode,
                content,
                write_time_seconds: *write_time_seconds,
            },
            UserChangeKeptByTheRefusal::SequentialWriteOfAnInode {
                inode,
                content,
                write_time_seconds,
            } => UserChange::SequentialWriteOfAnInode {
                inode: *inode,
                content,
                write_time_seconds: *write_time_seconds,
            },
            UserChangeKeptByTheRefusal::WriteAtOffset {
                inode,
                offset,
                bytes,
                write_time_seconds,
            } => UserChange::WriteAtOffset {
                inode: *inode,
                offset: *offset,
                bytes: PartialWriteBytes::new(bytes)
                    .expect("拷的是一段 PartialWriteBytes：至少一个字节"),
                write_time_seconds: *write_time_seconds,
            },
            UserChangeKeptByTheRefusal::DataChangesPublishedTogether {
                whole_write,
                new_inodes,
            } => {
                UserChange::DataChangesPublishedTogether(DataChangesPublishedTogether {
                    whole_write: whole_write.as_ref().map(
                        |(inode, content, write_time_seconds)| WholeWriteOfOneInode {
                            inode: *inode,
                            content,
                            write_time_seconds: *write_time_seconds,
                        },
                    ),
                    new_inodes,
                })
            }
        }
    }
}

impl RemainderOfTheUserChangeKeptByTheRefusal {
    pub(super) fn kept_from(remainder: RemainderOfTheUserChange<'_>) -> Self {
        match remainder {
            RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change) => {
                RemainderOfTheUserChangeKeptByTheRefusal::WalkTheQueueThenPublishTheChange(
                    UserChangeKeptByTheRefusal::kept_from(change),
                )
            }
            RemainderOfTheUserChange::WalkTheQueue => {
                RemainderOfTheUserChangeKeptByTheRefusal::WalkTheQueue
            }
        }
    }

    pub(super) fn as_remainder(&self) -> RemainderOfTheUserChange<'_> {
        match self {
            RemainderOfTheUserChangeKeptByTheRefusal::WalkTheQueueThenPublishTheChange(change) => {
                RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change.as_user_change())
            }
            RemainderOfTheUserChangeKeptByTheRefusal::WalkTheQueue => {
                RemainderOfTheUserChange::WalkTheQueue
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 「被宽护卡住」那一份报拷下的这次改动（[`UserChangeKeptByTheRefusal`]），要出路时读回成交给预演的改动，与被拒那一次交进来的改动
    /// 逐项相同：种类、内容全长、写入时间、inode、个数（正推 `research/prompts/m3-batch10z-vw-code-r1-sonnet-output.md` K3-3：
    /// 记错这几项的 8 种改坏原来照绿，其中读回时把覆盖写当顺序写那一种只在读回这一步，报的 `{:?}` 看不出来）。
    #[test]
    fn each_kind_of_change_kept_by_the_refusal_reads_back_as_the_change_handed_in() {
        let overwrite_content: Vec<u8> = (0..2999_u32)
            .map(|position| u8::try_from(position % 241).expect("小于 256"))
            .collect();
        let sequential_write_content: Vec<u8> = (0..5003_u32)
            .map(|position| u8::try_from((position * 7) % 239).expect("小于 256"))
            .collect();
        let changes_handed_in = [
            UserChange::Overwrite(FirstFile {
                content: &overwrite_content,
                write_time_seconds: 1_000_003,
            }),
            UserChange::SequentialWrite(FirstFile {
                content: &sequential_write_content,
                write_time_seconds: 1_000_007,
            }),
            UserChange::NewInodes {
                count: 37,
                write_time_seconds: 1_000_011,
            },
            UserChange::Unlink {
                inode: InodeNumber(4_099),
                write_time_seconds: 1_000_013,
            },
            UserChange::CreateSnapshot,
            UserChange::DestroySnapshot {
                snapshot: TreeIdentifier(65_551),
            },
        ];
        for change_handed_in in changes_handed_in {
            let kept = UserChangeKeptByTheRefusal::kept_from(change_handed_in);
            assert_eq!(
                format!("{:?}", kept.as_user_change()),
                format!("{change_handed_in:?}"),
                "拷下的改动读回成被拒那一次交进来的改动（种类、内容全长、写入时间、inode、个数逐项相同）"
            );
        }
    }
}
