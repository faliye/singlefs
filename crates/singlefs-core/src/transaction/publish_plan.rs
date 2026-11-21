//! 一次发布的计划 `PublishPlan`：第一个文件与别的文件的版本计划、实例表计划、删除、谱系、日志化操作与快照这几步。

use super::*;

/// 接在 `version_to_build_on` 后面那一次发布的 checkpoint_txg：它加一（D16（发布语义） 已定项 6）。
///
/// # Errors
/// 加一越过 `u64::MAX` ⇒ `NextCheckpointTxgPastTheTopOfItsRange`（代码审阅第 36 条：不 panic）。
pub fn checkpoint_txg_of_the_publish_after(
    version_to_build_on: CheckpointTxg,
) -> Result<CheckpointTxg, PublishError> {
    version_to_build_on
        .0
        .checked_add(1)
        .map(CheckpointTxg)
        .ok_or(PublishError::NextCheckpointTxgPastTheTopOfItsRange {
            version_to_build_on,
        })
}

/// 第一个文件：内容与写入时间（时间是参数，不取系统时钟，同参数两次发布逐字节相同）。
#[derive(Clone, Copy, Debug)]
pub struct FirstFile<'content> {
    pub content: &'content [u8],
    pub write_time_seconds: u64,
}

/// 这次发布要写的文件版本：数据单元与 extent 树根重写，这个对象的 inode 记录跟着换一版（它落在 inode 树的哪一片叶、
/// 要不要分裂由 `crate::inode_tree` 算）。没有它的发布（写行、暖机）这两类角色照抄上一版。
///
/// `content` 是整份文件内容（顺序写从偏移 0 写到末尾）：按切分纪律切成几个一单元事务就写几个数据单元
/// （`crate::write_request_split`，D16（发布语义） 已定项 5 末段），事务号从 [`PublishPlan::transaction`] 起连号。
///
/// 写的是哪个对象由 `inode` 说：第一个文件（inode 1）的数据单元与下段是 [`TransactionUnit::Data`] 与 [`TransactionUnit::ExtentLowerNode`]
/// 那两族，第一个文件之外的文件是 [`TransactionUnit::DataOfAFileOtherThanTheFirst`] 与
/// [`TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst`]，别的文件的数据单元与下段照抄、只在上段叶条目里重写它们那一条的所在叶。
#[derive(Clone, Copy, Debug)]
pub struct FileVersionPlan<'content> {
    /// 写的那个对象（数据单元头的五元组、extent 叶记录 key、inode 记录里都是它）。
    pub inode: InodeNumber,
    pub content: &'content [u8],
    pub write_time_seconds: u64,
    /// 对象出生代 = 创建那次发布的 checkpoint_txg，覆盖写不改（D8（核心索引结构） 已定项 6；I-9.10（对象出生代与 inode 记录相符））。
    pub inode_object_birth: CheckpointTxg,
    /// 改动计数 = 最后一次改动所在发布的 checkpoint_txg（D8（核心索引结构） 已定项 6 偏移 88）。
    pub change_count: u64,
}

/// 实例表单元这次发布怎么处理：根记录照抄上一版的指针，或者整条链重写（D18（块里携带什么信息） 已定项 11「每次写行 COW 重写整条链」）。
#[derive(Clone, Debug)]
pub enum InstanceTablePlan {
    Carry(NodePointer),
    Rewrite(InstanceTableRewrite),
}

/// 整条实例表链重写：这次之后整张表的行，与被这次换下的那条旧链。
#[derive(Clone, Debug)]
pub struct InstanceTableRewrite {
    /// 这次之后整张表的行，按链上的次序（上一版那张表的行在前、这次写的接在后面）。怎么分到各片由
    /// `instance_table_rows_of_each_page` 定（一片写满 369 行再开下一片）。
    pub rows: Vec<InstanceRow>,
    /// 被这次重写换下的那条旧链每一片的指针，按片序号：第 0 片是上一版根记录里那一条，第 k 片是第 k − 1 片链指针记录里那一条
    /// （`recovery::instance_table_chain_of_root` 读出来的 `page_pointers`）。发布逐片释放它们。
    /// 第 1 片起的指针不在上一版的内存态（`TransactionOutput`）里，所以由拼这张计划的调用方带进来：它拼行读的就是这同一条链，
    /// 接行与释放读的是同一份输入。
    pub replaced_chain: Vec<NodePointer>,
}

impl InstanceTableRewrite {
    /// 这次之后这条链有几片（D28（挂载期承诺量） 已定项 3 的 max(1, ⌈行数 ÷ 369⌉)）。
    #[must_use]
    pub fn pages_after_this_publish(&self) -> usize {
        instance_table_pages_for_rows(self.rows.len())
    }
}

/// 重写一条 `pages` 片的实例表链时各片的角色，按 bump 次序：尾片先（D3（空间分配） 已定项 10 ⑤，用户 2026-09-24 定案：
/// 第 k 片当第 k + 1 片的父，照先叶后根），第 0 片最后。这个次序同时是各片取落点与发出生序号的次序（D19（块指针的结构与宽度预算） 已定项 9）。
#[must_use]
pub fn instance_table_page_roles_in_bump_order(pages: usize) -> Vec<TransactionUnit> {
    (0..pages)
        .rev()
        .map(|position| {
            TransactionUnit::of_instance_table_page(InstanceTablePageIndex(
                u64::try_from(position).expect("片序号"),
            ))
        })
        .collect()
}

/// 一次发布的全部参数：哪些角色重写、身份字段取什么。新池新建文件、覆盖写、写行、暖机都是它的一种取值，走同一条发布路径
/// （`.claude/rules/fs-design.md`「一个事务层，所有结构共用」）。
#[derive(Clone, Debug)]
pub struct PublishPlan<'content> {
    pub txg: CheckpointTxg,
    /// 这次发布第一条记录的 jsn 计数器（记录落在环里的槽位），全池接着走、换实例不归零（D23（journal 的角色与格式） 已定项 14 第 3 条）。
    /// 一次发布切成 N 条记录时（文件版本有 N 个数据单元）它们连号：`counter .. counter + N`。
    pub counter: u64,
    /// 这次发布新池新建文件的事务号，按实例计数从 1 起，空发布写 0（D23（journal 的角色与格式） 已定项 7 / 已定项 19 ①）。
    /// 文件版本有 N 个数据单元时这次发布是 N 个事务（切分纪律一事务一单元），事务号 `transaction .. transaction + N` 连号，
    /// 一条记录一个事务（C310（事务切分纪律与记录数口径打架） 2026-09-16 用户定案）。
    pub transaction: u64,
    /// 这个实例在这次发布之前用过的最大事务号。`publish_version` 拿它与 `transaction` 取大的存进
    /// `TransactionOutput`，下一次发布从那里加一。
    ///
    /// 为什么不直接取上一条记录的事务号加一：空发布在记录上写 0（已定项 19 ①），取「上一条记录 + 1」会让计数
    /// 退回 1，同一个实例的事务号重号，而 D23（journal 的角色与格式） 已定项 7 逐字要求「事务号按实例计数、从 1 起」，
    /// 并且实例表行的 W 能当精确前缀正是靠这条纪律。
    pub highest_transaction_number_before_this_publish: u64,
    pub instance: InstanceGeneration,
    /// 反向链：上一条记录头的 CRC；本实例的第一条恒 0（D23（journal 的角色与格式） 已定项 19 ②）。
    pub back_chain: u32,
    pub file: Option<FileVersionPlan<'content>>,
    /// 这次发布往 inode 树里新建的 inode，按 inode 号严格升序（建 N 个文件那一路，里程碑「覆盖写、释放、回退与复用」并行线三）。
    /// 号由调用方按记账里的水位发（D5（快照 / 空间记账机制） 已定项 4 第 12 项），落在哪一片叶、要不要在末尾分裂
    /// 由 `crate::inode_tree` 在任何落盘动作之前算。别的发布给空的：文件版本那次只换它自己那条记录，写行与暖机不碰 inode 树。
    pub new_inode_records: &'content [InodeRecord],
    pub instance_table: InstanceTablePlan,
    /// 树表条目的诞生 txg：树建起来那次发布，之后每一版重写都不改。
    pub tree_birth_txg: CheckpointTxg,
    /// 这次发布写进根记录与记录新根段的树 ID 水位（D8（核心索引结构） 已定项 8 ②：max(根环里全部根记录的该字段, 本次发出的最高树 ID + 1)）。
    pub tree_identifier_watermark: u64,
    pub rollback_floor: CheckpointTxg,
}

/// 一次发布在 unlink 里是哪一步（D8（核心索引结构） 已定项 16 第 1、2 条）；不是删除的发布给 [`DeletionStepOfThePublish::NotADeletion`]。
///
/// 不并进 [`PublishPlan`]：验证包几份装置按字段逐个构造 `PublishPlan`，加字段就要动它们；删除这一步由删除的入口
/// （`publish_unlink_intent`、`publish_unlink_step`）单独交给发布路径，别的入口一律给 `NotADeletion`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeletionStepOfThePublish {
    NotADeletion,
    /// 意图：那条 inode 记录换成 nlink = 0、size 不变的一版，`deleted_inodes` 加它的号，墓碑树开放容器追加 `tombstone`
    /// （第 1 条）。数据单元与 extent 树照抄。
    Intent {
        record_with_nlink_zero: InodeRecord,
        tombstone: TombstoneRecord,
    },
    /// 不是最后一批的一批：`inode` 那个文件只留下前 `data_units_kept` 个数据单元（尾部的经映射释放），记录换成 size 改小的那一版
    /// （nlink 仍 0），`deleted_inodes` 与墓碑树照抄（第 2 条；对任一 inode 同一条，判决
    /// `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M1）。别的文件一个单元都不动。
    Batch {
        inode: InodeNumber,
        record_after_the_batch: InodeRecord,
        data_units_kept: u64,
    },
    /// 最后一批：删 `inode` 那条记录、释放它剩下的全部数据单元、删 `deleted_inodes` 里它的号，同一次发布（第 2 条）。
    FinalBatch {
        inode: InodeNumber,
    },
    /// 截断的意图那次（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K2「先落目标」）：`record_after_the_intent.inode` 那条记录
    /// 换成 size = N、改动计数与时间照 unlink 意图的写法的一版（任意活 inode，判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md`
    /// 第二节 S3）；尾单元要重写的（K3）按 `tail_data_unit_rewrite` 重写那个文件的那一个单元，别的数据单元连同 size 之外还挂着的照抄。
    /// 截断条目由同一次发布的 [`LoggedOperationsStepOfThePublish::IssueTheTruncationIntent`] 签。删除类（K10）。
    TruncationIntent {
        record_after_the_intent: InodeRecord,
        tail_data_unit_rewrite: Option<TailDataUnitRewrite>,
    },
    /// 截断的一批（不是最后一批）：`inode` 那个文件（截断条目点名的那一个）只留下前 `data_units_kept` 个数据单元（尾部的经映射释放），
    /// 不碰 inode 记录（K2）。删除类。
    TruncationBatch {
        inode: InodeNumber,
        data_units_kept: u64,
    },
    /// 截断的最后一批：`inode` 那个文件留下前 `data_units_kept` 个（可以一个都不放），截断条目由同一次发布的
    /// [`LoggedOperationsStepOfThePublish::DeleteTheTruncationIntent`] 删（D8（核心索引结构） 已定项 5「删除意图必须与最后一批工作在同一个事务里提交」）。删除类。
    TruncationFinalBatch {
        inode: InodeNumber,
        data_units_kept: u64,
    },
    /// 不是删除：变长（K5，一次发布、不立意图）与 N = S（K12，只改记录）那一次：`record_after_the_change.inode` 那条记录换成它，
    /// 变长时那个文件最后一个存在的单元按 `tail_data_unit_rewrite` 补零重写。准入照普通写判（K10）。
    SizeChangeWithoutAnIntent {
        record_after_the_change: InodeRecord,
        tail_data_unit_rewrite: Option<TailDataUnitRewrite>,
    },
    /// 不是删除：按偏移的部分写（判决 `research/prompts/m4-partial-write-r1-main-verification.md` 第二节 R2–R6，[`publish_partial_write`]）：
    /// `record_after_the_write.inode` 那条记录换成它（size 取写之前与写的终点里大的，改动计数与时间照写，截断字段照抄），那个文件
    /// `rewritten_data_units` 那一段按读回的旧单元（或零）连同这次的字节重写——从偏移 o 所在的单元起到写之后最后一个单元（R4），段里单元号
    /// 不小于写之前单元数的是这次新加的（写跨末尾、写进洞里、空隙补零，R5）。准入照普通写判。
    PartialWrite {
        record_after_the_write: InodeRecord,
        rewritten_data_units: DataUnitsRewrittenInOnePublish,
    },
}

/// 一步按读回的旧单元（或零）重写的那一段数据单元（[`DeletionStepOfThePublish::data_units_rewritten_from_their_payloads`]）：
/// 哪个 inode、哪一段、这一步之后那条记录的 size（每个重写单元的声明长度 = min(size − 起点, 净荷)）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataUnitsRewrittenByTheStep {
    pub inode: InodeNumber,
    pub units: DataUnitsRewrittenInOnePublish,
    pub size_after_in_bytes: u64,
}

impl DeletionStepOfThePublish {
    /// 这一步之后第一个文件还剩几个数据单元：删 inode 1 的一批与截 inode 1 的两种批留下 `data_units_kept` 个；删 inode 1 的最后一批一个不剩；
    /// 删、截别的号的批与最后一批、别的步照旧（判决 m3-multi-file-unlink-r1 第三节 M1、m4-multi-inode-size-r1 第二节 S3：别的号的批不当成
    /// 第一个文件的）。
    #[must_use]
    pub fn data_units_after_this_publish(self, data_units_before_this_publish: usize) -> usize {
        match self {
            DeletionStepOfThePublish::Batch {
                inode,
                data_units_kept,
                ..
            }
            | DeletionStepOfThePublish::TruncationBatch {
                inode,
                data_units_kept,
            }
            | DeletionStepOfThePublish::TruncationFinalBatch {
                inode,
                data_units_kept,
            } if inode.0 == FIRST_INODE_NUMBER => {
                usize::try_from(data_units_kept).expect("一批留下的单元数不多于上一版的单元数")
            }
            DeletionStepOfThePublish::FinalBatch { inode } if inode.0 == FIRST_INODE_NUMBER => 0,
            // 部分写第一个文件：写之后的单元数 = 写之前的与重写的那一段的终点里大的（加单元时多于上一版，判决 m4-partial-write-r1 第二节 R4、R5）。
            DeletionStepOfThePublish::PartialWrite {
                record_after_the_write,
                rewritten_data_units,
            } if record_after_the_write.inode == FIRST_INODE_NUMBER => {
                usize::try_from(rewritten_data_units.end_exclusive)
                    .expect("写之后的单元数装得进 usize：每个单元都要在内存里装一份")
                    .max(data_units_before_this_publish)
            }
            DeletionStepOfThePublish::Batch { .. }
            | DeletionStepOfThePublish::FinalBatch { .. }
            | DeletionStepOfThePublish::TruncationBatch { .. }
            | DeletionStepOfThePublish::TruncationFinalBatch { .. }
            | DeletionStepOfThePublish::Intent { .. }
            | DeletionStepOfThePublish::TruncationIntent { .. }
            | DeletionStepOfThePublish::SizeChangeWithoutAnIntent { .. }
            | DeletionStepOfThePublish::PartialWrite { .. }
            | DeletionStepOfThePublish::NotADeletion => data_units_before_this_publish,
        }
    }

    /// 截断三步与变长、只改记录那一次改的是哪个 inode 的 size（判决 m4-multi-inode-size-r1 第二节 S3：任意活 inode）；别的步没有。
    #[must_use]
    pub const fn inode_whose_size_changes(self) -> Option<InodeNumber> {
        match self {
            DeletionStepOfThePublish::TruncationIntent {
                record_after_the_intent: record,
                ..
            }
            | DeletionStepOfThePublish::SizeChangeWithoutAnIntent {
                record_after_the_change: record,
                ..
            }
            | DeletionStepOfThePublish::PartialWrite {
                record_after_the_write: record,
                ..
            } => Some(InodeNumber(record.inode)),
            DeletionStepOfThePublish::TruncationBatch { inode, .. }
            | DeletionStepOfThePublish::TruncationFinalBatch { inode, .. } => Some(inode),
            DeletionStepOfThePublish::NotADeletion
            | DeletionStepOfThePublish::Intent { .. }
            | DeletionStepOfThePublish::Batch { .. }
            | DeletionStepOfThePublish::FinalBatch { .. } => None,
        }
    }

    /// 这一步重写的那一个尾单元与它属于哪个 inode（[`Self::tail_data_unit_rewrite`] 配上 [`Self::inode_whose_size_changes`]）。
    #[must_use]
    pub fn tail_data_unit_rewrite_of_the_inode(self) -> Option<(InodeNumber, TailDataUnitRewrite)> {
        self.inode_whose_size_changes()
            .zip(self.tail_data_unit_rewrite())
    }

    /// 这一步重写的那一个尾单元（截断意图那次、变长那次，K3、K5）；别的发布没有。
    #[must_use]
    pub const fn tail_data_unit_rewrite(self) -> Option<TailDataUnitRewrite> {
        match self {
            DeletionStepOfThePublish::TruncationIntent {
                tail_data_unit_rewrite,
                ..
            }
            | DeletionStepOfThePublish::SizeChangeWithoutAnIntent {
                tail_data_unit_rewrite,
                ..
            } => tail_data_unit_rewrite,
            DeletionStepOfThePublish::NotADeletion
            | DeletionStepOfThePublish::Intent { .. }
            | DeletionStepOfThePublish::Batch { .. }
            | DeletionStepOfThePublish::FinalBatch { .. }
            | DeletionStepOfThePublish::TruncationBatch { .. }
            | DeletionStepOfThePublish::TruncationFinalBatch { .. }
            | DeletionStepOfThePublish::PartialWrite { .. } => None,
        }
    }

    /// 这一步按读回的旧单元（或零）重写的那一段数据单元（[`DataUnitsRewrittenByTheStep`]）：截断意图与变长重写的那一个尾单元（K3、K5），
    /// 按偏移部分写的那一段（判决 m4-partial-write-r1 第二节 R4）；别的步没有。计划的数据角色、切出的一单元事务、映射 key 与装单元都读它。
    #[must_use]
    pub fn data_units_rewritten_from_their_payloads(self) -> Option<DataUnitsRewrittenByTheStep> {
        match self {
            DeletionStepOfThePublish::TruncationIntent {
                record_after_the_intent: record,
                tail_data_unit_rewrite: Some(rewrite),
            }
            | DeletionStepOfThePublish::SizeChangeWithoutAnIntent {
                record_after_the_change: record,
                tail_data_unit_rewrite: Some(rewrite),
            } => Some(DataUnitsRewrittenByTheStep {
                inode: InodeNumber(record.inode),
                units: DataUnitsRewrittenInOnePublish::of_one_unit(rewrite.unit),
                size_after_in_bytes: record.size,
            }),
            DeletionStepOfThePublish::PartialWrite {
                record_after_the_write: record,
                rewritten_data_units,
            } => Some(DataUnitsRewrittenByTheStep {
                inode: InodeNumber(record.inode),
                units: rewritten_data_units,
                size_after_in_bytes: record.size,
            }),
            DeletionStepOfThePublish::TruncationIntent {
                tail_data_unit_rewrite: None,
                ..
            }
            | DeletionStepOfThePublish::SizeChangeWithoutAnIntent {
                tail_data_unit_rewrite: None,
                ..
            }
            | DeletionStepOfThePublish::NotADeletion
            | DeletionStepOfThePublish::Intent { .. }
            | DeletionStepOfThePublish::Batch { .. }
            | DeletionStepOfThePublish::FinalBatch { .. }
            | DeletionStepOfThePublish::TruncationBatch { .. }
            | DeletionStepOfThePublish::TruncationFinalBatch { .. } => None,
        }
    }

    /// 准入归删除类（D28（挂载期承诺量） 已定项 1：不判式子那一判、落点那一判照判）的那几步：unlink 的三种与截断的三种（K10）。
    /// 变长、只改记录与别的发布不是。
    #[must_use]
    pub const fn is_a_deletion_class_publish(self) -> bool {
        match self {
            DeletionStepOfThePublish::Intent { .. }
            | DeletionStepOfThePublish::Batch { .. }
            | DeletionStepOfThePublish::FinalBatch { .. }
            | DeletionStepOfThePublish::TruncationIntent { .. }
            | DeletionStepOfThePublish::TruncationBatch { .. }
            | DeletionStepOfThePublish::TruncationFinalBatch { .. } => true,
            DeletionStepOfThePublish::NotADeletion
            | DeletionStepOfThePublish::SizeChangeWithoutAnIntent { .. } => false,
            // 部分写当普通写（判决 m4-partial-write-r1：准入照普通写判）。
            DeletionStepOfThePublish::PartialWrite { .. } => false,
        }
    }

    /// 这一步之后 `deleted_inodes` 的号：意图加一个、最后一批删一个、别的照抄（升序）。
    #[must_use]
    pub fn deleted_inodes_after_this_publish(self, before: &[InodeNumber]) -> Vec<InodeNumber> {
        let mut keys = before.to_vec();
        match self {
            DeletionStepOfThePublish::Intent {
                record_with_nlink_zero,
                ..
            } => {
                keys.push(InodeNumber(record_with_nlink_zero.inode));
                keys.sort_unstable();
                keys.dedup();
            }
            DeletionStepOfThePublish::FinalBatch { inode } => keys.retain(|key| *key != inode),
            DeletionStepOfThePublish::Batch { .. }
            | DeletionStepOfThePublish::TruncationIntent { .. }
            | DeletionStepOfThePublish::TruncationBatch { .. }
            | DeletionStepOfThePublish::TruncationFinalBatch { .. }
            | DeletionStepOfThePublish::SizeChangeWithoutAnIntent { .. }
            | DeletionStepOfThePublish::PartialWrite { .. }
            | DeletionStepOfThePublish::NotADeletion => {}
        }
        keys
    }
}

/// 一次发布往谱系树里追加什么（D18（块里携带什么信息） 已定项 24 第 5 条、D23（journal 的角色与格式） 已定项 14）：
/// 只有管理员回退那次发布追加一条，别的发布照抄谱系树。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineageStepOfThePublish {
    NoLineageRecord,
    /// 管理员回退那次发布：谱系树开放容器追加 `record`（T_rb = 这次发布的 txg，T_old = R_old 的 txg，H_cur = cur 的 inode 号水位）；
    /// 这次发布的 inode 号水位写 `inode_number_watermark_after_the_rollback`（D23（journal 的角色与格式） 已定项 14「水位」一格的 max，
    /// 回退那一判算好交进来），不从上一版的记账现算。
    AdministratorRollback {
        record: LineageRecord,
        inode_number_watermark_after_the_rollback: u64,
    },
}

/// 删除这一步连同只供测试的墓碑容器容量（D18（块里携带什么信息） 已定项 24 第 3 条：关闭路径加只供测试的容量旋钮，旋钮不落盘），
/// 与这次往谱系树里追加什么。不并进 [`PublishPlan`] 的理由同 [`DeletionStepOfThePublish`]。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeletionAndLineageOfThePublish {
    pub step: DeletionStepOfThePublish,
    /// 墓碑树与谱系树追加时的容量（写入口上装的只供测试的开关，产品路径按格式）。
    pub container_tree_capacities: ContainerTreeCapacities,
    pub lineage: LineageStepOfThePublish,
    /// 这次往树表里加不加一个只读快照（只有建快照那次发布加，[`publish_create_snapshot`]）。并在这里而不并进 [`PublishPlan`]，
    /// 理由同 [`DeletionStepOfThePublish`]。
    pub snapshot: SnapshotStepOfThePublish,
    /// 这次往 logged_ops 树里签发还是删一条意图（只有整理的发布做，[`publish_compaction_step`]）。并在这里而不并进 [`PublishPlan`]，
    /// 理由同 [`DeletionStepOfThePublish`]。
    pub logged_operations: LoggedOperationsStepOfThePublish,
}

/// 一次发布对 logged_ops 树做什么（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1、G3；D26（后台整理与放置回收） 已定项 2；
/// D8（核心索引结构） 已定项 5「删除意图必须与最后一批工作在同一个事务里提交」）：只有整理的发布签发或删意图，别的发布照抄上一版的条目。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoggedOperationsStepOfThePublish {
    CarryTheLoggedOperations,
    /// 签发：追加这一条整理意图，签发 txg 写这次发布的 txg。上一版已有一条活整理意图时调用方不许走到这里（全池至多一条，判决 G1 e1）。
    IssueTheCompactionIntent(CompactionIntent),
    /// 删签发 txg 为 `issued_txg` 的那一条整理意图（意图达成时与最后一批同一次发布，或丢意图时单独一次发布）。
    DeleteTheCompactionIntent {
        issued_txg: CheckpointTxg,
    },
    /// 签发截断意图（判决 m4-trunc-r1 K1、K2：截断意图那次发布）：追加这一条，签发 txg 写这次发布的 txg。上一版已有截断条目时
    /// 调用方不许走到这里（全池至多一条；发意图的入口断言三样队列都空）。
    IssueTheTruncationIntent(TruncationIntent),
    /// 删签发 txg 为 `issued_txg` 的那一条截断意图（截断的最后一批，同一次发布）。
    DeleteTheTruncationIntent {
        issued_txg: CheckpointTxg,
    },
    /// 管理员回退那次发布（判决 m4-trunc-r1 K1「logged_ops 按种类分来源」）：整理条目照抄上一版（现行那一版），截断条目换成回退目标
    /// R_old 那一版的（`None` = 它没有）。
    TakeTheTruncationEntryOfTheRollbackTarget(Option<LoggedOperationEntry>),
}

impl LoggedOperationsStepOfThePublish {
    /// 这一步之后 logged_ops 的条目（按 key 升序）：签发加一条（签发 txg = `txg`）、删掉一条、别的照抄。
    ///
    /// # Panics
    /// 签发时上一版已有一条同种意图、删的那一条不在上一版里：整理那两样由整理的入口（`publish_compaction_step`）在任何读写之前拒过，
    /// 截断那两样由发截断意图的入口断言、截断的最后一批从上一版取条目保证。
    #[must_use]
    pub fn logged_operations_after_this_publish(
        self,
        before: &[LoggedOperationEntry],
        txg: CheckpointTxg,
    ) -> Vec<LoggedOperationEntry> {
        let mut entries = before.to_vec();
        match self {
            LoggedOperationsStepOfThePublish::CarryTheLoggedOperations => {}
            LoggedOperationsStepOfThePublish::IssueTheCompactionIntent(intent) => {
                assert!(
                    live_compaction_intent_of_the_entries(before).is_none(),
                    "全池活整理意图至多一条（判决 G1 e1）：签发入口判过上一版没有活意图"
                );
                entries.push(LoggedOperationEntry {
                    issued_txg: txg,
                    operation: LoggedOperation::Compaction(intent),
                });
                entries.sort_by_key(|entry| entry.key().sort_key());
            }
            LoggedOperationsStepOfThePublish::DeleteTheCompactionIntent { issued_txg } => {
                let entries_before = entries.len();
                entries.retain(|entry| {
                    !(entry.issued_txg == issued_txg
                        && entry.operation.kind() == LoggedOperationKind::Compaction)
                });
                assert_eq!(
                    entries.len() + 1,
                    entries_before,
                    "删的那一条整理意图在上一版里：删意图入口判过"
                );
            }
            LoggedOperationsStepOfThePublish::IssueTheTruncationIntent(intent) => {
                assert!(
                    live_truncation_intent_of_the_entries(before).is_none(),
                    "全池活截断意图至多一条（判决 m4-trunc-r1 K1）：发意图的入口断言上一版没有截断条目"
                );
                entries.push(LoggedOperationEntry {
                    issued_txg: txg,
                    operation: LoggedOperation::Truncation(intent),
                });
                entries.sort_by_key(|entry| entry.key().sort_key());
            }
            LoggedOperationsStepOfThePublish::DeleteTheTruncationIntent { issued_txg } => {
                let entries_before = entries.len();
                entries.retain(|entry| {
                    !(entry.issued_txg == issued_txg
                        && entry.operation.kind() == LoggedOperationKind::Truncation)
                });
                assert_eq!(
                    entries.len() + 1,
                    entries_before,
                    "删的那一条截断意图在上一版里：截断的最后一批从上一版取的它"
                );
            }
            LoggedOperationsStepOfThePublish::TakeTheTruncationEntryOfTheRollbackTarget(
                entry_of_the_target,
            ) => {
                entries.retain(|entry| entry.operation.kind() != LoggedOperationKind::Truncation);
                if let Some(entry) = entry_of_the_target {
                    assert_eq!(
                        entry.operation.kind(),
                        LoggedOperationKind::Truncation,
                        "回退目标那一版交来的是它的截断条目"
                    );
                    entries.push(entry);
                }
                entries.sort_by_key(|entry| entry.key().sort_key());
            }
        }
        entries
    }
}

impl DeletionAndLineageOfThePublish {
    /// 既不是删除、也不是管理员回退、也不是建快照的发布：墓碑树与谱系树照抄，容量不会被读到；树表照上一版的快照条目续上。
    pub const NEITHER_A_DELETION_NOR_A_ROLLBACK: DeletionAndLineageOfThePublish =
        DeletionAndLineageOfThePublish {
            step: DeletionStepOfThePublish::NotADeletion,
            container_tree_capacities: ContainerTreeCapacities::FROM_THE_FORMAT,
            lineage: LineageStepOfThePublish::NoLineageRecord,
            snapshot: SnapshotStepOfThePublish::NoNewSnapshot,
            logged_operations: LoggedOperationsStepOfThePublish::CarryTheLoggedOperations,
        };
}

/// 一次发布往树表里加不加一个只读快照（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1、S2）。
/// 每一次带树表的发布都把上一版树表里的快照条目（码 14、15）原样续在十二条之后（判决 `research/prompts/m4-batch2-code-r1-main-verification.md`
/// 第二节 S2），只有建快照那次独立发布再追加一对。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapshotStepOfThePublish {
    NoNewSnapshot,
    /// 建快照那次发布（[`publish_create_snapshot`]）：续上的快照条目之后追加一对——码 14 树号 `snapshot_inode_root_tree`、码 15 树号
    /// `snapshot_extent_root_tree`（这次从水位连发的两个号，D8（核心索引结构） 已定项 8 ②），字段取值见 [`TREE_KIND_SNAPSHOT_INODE_ROOT`] 与
    /// [`TREE_KIND_SNAPSHOT_EXTENT_ROOT`]；活头 inode 树条目的 `previous_snapshot_txg` 换成这次的 checkpoint_txg（S.txg，D5（快照 / 空间记账机制）
    /// 已定项 9、已定项 14）。
    CreateSnapshot {
        snapshot_inode_root_tree: TreeIdentifier,
        snapshot_extent_root_tree: TreeIdentifier,
    },
    /// 删快照的意图那次发布（[`publish_snapshot_destruction_intent`]，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S4）：
    /// 码 14 条目 `snapshot` 的 flags 位 0 写成「销毁中」，它的前驱 P 传给那一刻的下一侧（活头 inode 条目或下一条码 14 的 `previous_snapshot_txg`，
    /// D5（快照 / 空间记账机制） 已定项 12「prev(S) 在写意图的那个事务里传给那一刻的下一侧」）。一条 deadlist 条目都不动。
    DestructionIntent {
        snapshot: TreeIdentifier,
    },
    /// 删快照意图之后、不是最后一批的一批（[`publish_snapshot_destruction_step`]）：放掉 `snapshot` 那一段里能放的条目
    /// （纪元落在 [lo, hi) 里、birth > P，[`deadlist_entries_the_destruction_of_a_snapshot_releases`]）按 key 升序的前 `entries_released` 条。
    DestructionBatch {
        snapshot: TreeIdentifier,
        entries_released: usize,
    },
    /// 删快照的最后一批：放掉剩下能放的全部条目，同一次发布删掉 `snapshot` 那一对码 14、15 条目。
    DestructionFinalBatch {
        snapshot: TreeIdentifier,
    },
}

impl SnapshotStepOfThePublish {
    /// 这一步是不是删快照的三种发布之一（与 unlink 的三种同准入：式子那一判不判、落点那一判照判，判决第七节岔路 4）。
    #[must_use]
    pub const fn is_a_snapshot_destruction(self) -> bool {
        match self {
            SnapshotStepOfThePublish::NoNewSnapshot
            | SnapshotStepOfThePublish::CreateSnapshot { .. } => false,
            SnapshotStepOfThePublish::DestructionIntent { .. }
            | SnapshotStepOfThePublish::DestructionBatch { .. }
            | SnapshotStepOfThePublish::DestructionFinalBatch { .. } => true,
        }
    }
}
