//! 把发布计划解成 `ResolvedPublish`：文件内容的事务、普通分配那一部分与多层树的计划（`impl PublishPlan`）。

use super::*;

impl PublishPlan<'_> {
    /// 这次发布写的文件内容按切分纪律切出来的一单元事务，按单元序号升序（= extent key 升序，D16（发布语义） 已定项 5 末段）；
    /// 没有文件版本的发布（写行、暖机、建 inode）为空。事务号从 `transaction` 起连号。
    /// 角色清单、数据单元的装法、记录的切法都读这一份，不各算一遍（`crate::write_request_split` 是切分唯一的实现）。
    #[must_use]
    pub fn file_content_transactions(&self) -> Vec<OneUnitTransaction> {
        match &self.file {
            Some(file) => split_sequential_write_into_one_unit_transactions(
                u64::try_from(file.content.len()).expect("内容长度"),
                self.transaction,
            ),
            None => Vec::new(),
        }
    }

    /// 这次发布写的数据单元按一单元事务排：有文件版本就是 [`Self::file_content_transactions`]；没有而这一步按读回的旧单元重写一段单元
    /// （截断意图、变长的那一个尾单元，判决 m4-trunc-r1 K3、K5；部分写从偏移所在的单元起的那一段，判决 m4-partial-write-r1 第二节 R4）
    /// 就是那一段里每个单元一个事务，按单元号升序、事务号从这次发布的 `transaction` 起连号（切分纪律一事务一单元，D16（发布语义） 已定项 5），
    /// 载荷区间是它在文件里那一段（声明长度 = min(这一步之后的 size − 起点, 净荷) 那么长）；都没有为空。
    #[must_use]
    pub fn file_content_transactions_and_the_rewritten_data_units(
        &self,
        data_units_rewritten_from_their_payloads: Option<DataUnitsRewrittenByTheStep>,
    ) -> Vec<OneUnitTransaction> {
        match (&self.file, data_units_rewritten_from_their_payloads) {
            (Some(_), _) => self.file_content_transactions(),
            (None, Some(rewritten)) => {
                let payload_capacity = u64::try_from(data_unit_payload_capacity()).expect("32634");
                // 迭代次数的上界是重写的单元数；不跨轮携带状态、没有提前出口。
                rewritten
                    .units
                    .units()
                    .zip(self.transaction..)
                    .map(|(unit, transaction_number)| {
                        let unit = DataUnitIndexInFile(unit);
                        let payload_start = unit.first_file_byte(payload_capacity);
                        OneUnitTransaction {
                            transaction_number,
                            unit_index_in_file: unit,
                            payload_start,
                            payload_end_exclusive: FileOffsetInBytes(
                                payload_start.0
                                    + declared_length_within_a_size(
                                        unit,
                                        rewritten.size_after_in_bytes,
                                    ),
                            ),
                        }
                    })
                    .collect()
            }
            (None, None) => Vec::new(),
        }
    }

    /// 这次发布写的文件版本里写第一个文件（inode 1）的那一份；写第一个文件之外的文件、或没有文件版本的发布没有。
    pub(super) fn first_file_version(&self) -> Option<&FileVersionPlan<'_>> {
        self.file
            .as_ref()
            .filter(|file| file.inode.0 == FIRST_INODE_NUMBER)
    }

    /// 这次发布写的第一个文件之外那个文件的 inode 号与它这次切出的单元数（[`Self::file_content_transactions`] 的条数）；
    /// 写第一个文件、或没有文件版本的发布没有。
    pub(super) fn file_other_than_the_first_written(&self) -> Option<(InodeNumber, u64)> {
        self.inode_of_the_file_other_than_the_first_written()
            .map(|inode| {
                (
                    inode,
                    u64::try_from(self.file_content_transactions().len()).expect("这次写的单元数"),
                )
            })
    }

    /// 这次发布写的第一个文件之外那个文件的 inode 号（不切内容）。
    fn inode_of_the_file_other_than_the_first_written(&self) -> Option<InodeNumber> {
        self.file
            .as_ref()
            .map(|file| file.inode)
            .filter(|inode| inode.0 != FIRST_INODE_NUMBER)
    }

    /// 这次发布重写的一个数据单元的角色：写第一个文件之外的文件、或按读回的旧单元重写第一个文件之外那个文件的一段单元
    /// （`inode_whose_tail_is_rewritten`：截断意图与变长的尾单元，判决 m4-multi-inode-size-r1 第二节 S3；部分写，判决 m4-partial-write-r1
    /// 第二节 R6）时是那个文件的那一族，别的（写第一个文件、重写第一个文件的单元）是 [`TransactionUnit::Data`]。
    fn role_of_a_data_unit_written(
        &self,
        unit: DataUnitIndexInFile,
        inode_whose_tail_is_rewritten: Option<InodeNumber>,
    ) -> TransactionUnit {
        match self
            .inode_of_the_file_other_than_the_first_written()
            .or(inode_whose_tail_is_rewritten.filter(|inode| inode.0 != FIRST_INODE_NUMBER))
        {
            Some(inode) => TransactionUnit::DataOfAFileOtherThanTheFirst { inode, unit },
            None => TransactionUnit::Data(unit),
        }
    }

    /// 这次发布的 inode 记录写入：文件版本那条（有文件版本时）在前，新建的那些在后。
    /// 号天然严格升序——文件版本只与「不建 inode」一起出现（写文件的入口新 inode 记录恒空），新建的号从记账水位起发、都大于树里每一个 key。
    fn inode_record_writes(&self) -> Vec<InodeRecord> {
        let mut records = Vec::new();
        if let Some(file) = &self.file {
            records.push(inode_record_of_file_version(file));
        }
        records.extend_from_slice(self.new_inode_records);
        records
    }

    /// 这次发布里普通分配那一半的样子（[`OrdinaryAllocationPartOfThePublish`]）：inode 树与 extent 树这次之后怎样、写的文件内容切成哪几个
    /// 一单元事务。只由计划与上一版定，与分配记录树这次重写哪几个节点无关——[`Self::resolve`] 每一轮都从它起，
    /// [`prepare_the_version_publish`] 在走分配记录树的固定点之前判「这次的单元落得下」也读它。
    ///
    /// # Errors
    /// inode 记录的落法要的条款仓里还没定 ⇒ `InodeTreeWriteRefused`（`crate::inode_tree` 的三格）；
    /// 墓碑树或退役树要开新容器而最右那片的号已到顶 ⇒ `NextContainerNumberPastTheTopOfItsRange`（C841）。
    pub(super) fn ordinary_allocation_part(
        &self,
        previous: Option<&TransactionOutput>,
        trees: &FileVersionTreeIdentifiers,
        deletion_and_lineage: &DeletionAndLineageOfThePublish,
    ) -> Result<OrdinaryAllocationPartOfThePublish, PublishError> {
        let containers_before = previous
            .map(TransactionOutput::inode_leaf_container_contents)
            .unwrap_or_default();
        let mut record_writes = self.inode_record_writes();
        match deletion_and_lineage.step {
            DeletionStepOfThePublish::Intent {
                record_with_nlink_zero: record,
                ..
            }
            | DeletionStepOfThePublish::Batch {
                record_after_the_batch: record,
                ..
            }
            | DeletionStepOfThePublish::TruncationIntent {
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
            } => record_writes.push(record),
            DeletionStepOfThePublish::FinalBatch { .. }
            | DeletionStepOfThePublish::TruncationBatch { .. }
            | DeletionStepOfThePublish::TruncationFinalBatch { .. }
            | DeletionStepOfThePublish::NotADeletion => {}
        }
        let inode_tree = match deletion_and_lineage.step {
            DeletionStepOfThePublish::FinalBatch { inode } => {
                assert!(
                    record_writes.is_empty(),
                    "最后一批只删一条记录，不与别的记录写同一次发布：删除的入口不带文件版本、不建 inode"
                );
                remove_record_from_leaf_containers(&containers_before, inode)?
            }
            DeletionStepOfThePublish::Intent { .. }
            | DeletionStepOfThePublish::Batch { .. }
            | DeletionStepOfThePublish::TruncationIntent { .. }
            | DeletionStepOfThePublish::TruncationBatch { .. }
            | DeletionStepOfThePublish::TruncationFinalBatch { .. }
            | DeletionStepOfThePublish::SizeChangeWithoutAnIntent { .. }
            | DeletionStepOfThePublish::PartialWrite { .. }
            | DeletionStepOfThePublish::NotADeletion => write_records_into_leaf_containers(
                &containers_before,
                &record_writes,
                self.txg,
                trees.inode,
            )?,
        };
        // n_live 与 n_记录随一版带着、按这次写进与删掉的那几条改（准入读它，`.claude/rules/fs-design.md`「空间统计与配额落在第一格」）：
        // 上一版里那个号的记录按叶序折半找，不遍历 inode 树。
        let inode_record_counts_before = previous
            .map_or(InodeRecordCounts::OF_AN_EMPTY_INODE_TREE, |version| {
                version.inode_record_counts
            });
        let record_before_this_publish = |inode: InodeNumber| {
            previous.and_then(|version| version.inode_record_found_by_its_leaf(inode))
        };
        let inode_record_counts = match deletion_and_lineage.step {
            DeletionStepOfThePublish::FinalBatch { inode } => inode_record_counts_before.after_removing(
                &record_before_this_publish(inode)
                    .expect("最后一批删的那条记录在上一版的 inode 树里：不在的那一种 remove_record_from_leaf_containers 已经拒了"),
            ),
            DeletionStepOfThePublish::Intent { .. }
            | DeletionStepOfThePublish::Batch { .. }
            | DeletionStepOfThePublish::TruncationIntent { .. }
            | DeletionStepOfThePublish::TruncationBatch { .. }
            | DeletionStepOfThePublish::TruncationFinalBatch { .. }
            | DeletionStepOfThePublish::SizeChangeWithoutAnIntent { .. }
            | DeletionStepOfThePublish::PartialWrite { .. }
            | DeletionStepOfThePublish::NotADeletion => {
                inode_record_counts_before.after_writing(&record_writes, record_before_this_publish)
            }
        };
        let file_content_transactions = self
            .file_content_transactions_and_the_rewritten_data_units(
                deletion_and_lineage
                    .step
                    .data_units_rewritten_from_their_payloads(),
            );
        let data_units_before = previous.map_or(0, |version| version.data_pointers.len());
        let carried_data_units = deletion_and_lineage
            .step
            .data_units_after_this_publish(data_units_before);
        // 截断三步与变长、只改记录那一次改的那个文件（任意活 inode，判决 m4-multi-inode-size-r1 第二节 S3）改 size 之前与之后的样子。
        let size_change_of_a_file = size_change_of_a_file(deletion_and_lineage.step, previous);
        let size_change_of_the_first_file = size_change_of_a_file
            .filter(|size_change| size_change.inode.0 == FIRST_INODE_NUMBER)
            .map(|size_change| size_change.change);
        // 第一个文件之外的文件这次改哪一个：按 inode 号写的那一个，删除的一批与最后一批删的那一个（判决
        // `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M1：批与最后一批按 `inode` 分，不把别的号的批当第一个文件的），
        // 或改 size 的那一个（判决 m4-multi-inode-size-r1 第二节 S3：截断两种批带号，不当成第一个文件的）。
        let change_of_a_file_other_than_the_first = match self.file_other_than_the_first_written() {
            Some((inode, data_units)) => {
                ChangeOfAFileOtherThanTheFirst::WholeWrite { inode, data_units }
            }
            None => match deletion_and_lineage.step {
                DeletionStepOfThePublish::TruncationIntent { .. }
                | DeletionStepOfThePublish::TruncationBatch { .. }
                | DeletionStepOfThePublish::TruncationFinalBatch { .. }
                | DeletionStepOfThePublish::SizeChangeWithoutAnIntent { .. }
                | DeletionStepOfThePublish::PartialWrite { .. }
                    if size_change_of_a_file
                        .is_some_and(|size_change| size_change.inode.0 != FIRST_INODE_NUMBER) =>
                {
                    let size_change =
                        size_change_of_a_file.expect("上一句判过：改 size 的是第一个文件之外的号");
                    ChangeOfAFileOtherThanTheFirst::SizeChange {
                        inode: size_change.inode,
                        change: size_change.change,
                        size_after_in_bytes: size_change.size_after_in_bytes,
                    }
                }
                DeletionStepOfThePublish::Batch {
                    inode,
                    data_units_kept,
                    ..
                } if inode.0 != FIRST_INODE_NUMBER => {
                    ChangeOfAFileOtherThanTheFirst::DeletionBatch {
                        inode,
                        data_units_kept,
                    }
                }
                DeletionStepOfThePublish::FinalBatch { inode } if inode.0 != FIRST_INODE_NUMBER => {
                    ChangeOfAFileOtherThanTheFirst::FinalDeletionBatch { inode }
                }
                DeletionStepOfThePublish::Batch { .. }
                | DeletionStepOfThePublish::FinalBatch { .. }
                | DeletionStepOfThePublish::Intent { .. }
                | DeletionStepOfThePublish::TruncationIntent { .. }
                | DeletionStepOfThePublish::TruncationBatch { .. }
                | DeletionStepOfThePublish::TruncationFinalBatch { .. }
                | DeletionStepOfThePublish::SizeChangeWithoutAnIntent { .. }
                | DeletionStepOfThePublish::PartialWrite { .. }
                | DeletionStepOfThePublish::NotADeletion => {
                    ChangeOfAFileOtherThanTheFirst::NoFileChanges
                }
            },
        };
        let files_other_than_the_first = plan_the_files_other_than_the_first_after_this_publish(
            previous,
            change_of_a_file_other_than_the_first,
        );
        let extent_tree_of_the_step = match (deletion_and_lineage.step, previous) {
            (
                DeletionStepOfThePublish::TruncationIntent { .. }
                | DeletionStepOfThePublish::TruncationBatch { .. }
                | DeletionStepOfThePublish::TruncationFinalBatch { .. }
                | DeletionStepOfThePublish::SizeChangeWithoutAnIntent { .. }
                | DeletionStepOfThePublish::PartialWrite { .. },
                Some(previous_version),
            ) if size_change_of_the_first_file.is_some() => {
                plan_the_extent_tree_after_a_size_change(
                    &previous_version.extent_tree,
                    FIRST_INODE_NUMBER,
                    size_change_of_the_first_file.expect("上一句判过是 Some"),
                )
            }
            // 第一个文件之外的文件改 size：第一个文件的下段整段照抄，上段走统一计划（同写别的文件那一支）——这次之后有条目的是上一版的那几个，
            // 加这次第一次写上标签 0 的它（粘住的标签 0，判决 m4-multi-inode-size-r1 第二节 S2）；它那一条变了才重写它那一串。
            (
                DeletionStepOfThePublish::TruncationIntent { .. }
                | DeletionStepOfThePublish::TruncationBatch { .. }
                | DeletionStepOfThePublish::TruncationFinalBatch { .. }
                | DeletionStepOfThePublish::SizeChangeWithoutAnIntent { .. }
                | DeletionStepOfThePublish::PartialWrite { .. },
                Some(previous_version),
            ) if size_change_of_a_file.is_some() => {
                let size_change = size_change_of_a_file.expect("上一句判过是 Some");
                plan_the_extent_tree_after_the_size_change_of_a_file_other_than_the_first(
                    previous_version,
                    &files_other_than_the_first,
                    size_change,
                )
            }
            // 删除的一批与最后一批按 `inode` 分（判决 m3-multi-file-unlink-r1 第三节 M1、M3），上段都走统一计划（M2）。
            (
                DeletionStepOfThePublish::Batch {
                    inode,
                    data_units_kept,
                    ..
                },
                Some(previous_version),
            ) => {
                let data_units_before_the_batch = if inode.0 == FIRST_INODE_NUMBER {
                    previous_version.data_pointers.len()
                } else {
                    previous_version
                        .file_other_than_the_first(inode)
                        .map_or(0, |file| file.data_pointers.len())
                };
                plan_the_extent_tree_after_a_deletion_batch(
                    &previous_version.extent_tree,
                    &previous_version.inodes_with_an_extent_upper_leaf_entry(),
                    inode.0,
                    u64::try_from(data_units_before_the_batch).expect("单元数"),
                    data_units_kept,
                )
            }
            (DeletionStepOfThePublish::FinalBatch { inode }, Some(previous_version)) => {
                plan_the_extent_tree_after_the_final_batch(
                    &previous_version.extent_tree,
                    &previous_version.inodes_with_an_extent_upper_leaf_entry(),
                    inode.0,
                )
            }
            (
                DeletionStepOfThePublish::Batch { .. }
                | DeletionStepOfThePublish::FinalBatch { .. }
                | DeletionStepOfThePublish::Intent { .. }
                | DeletionStepOfThePublish::TruncationIntent { .. }
                | DeletionStepOfThePublish::TruncationBatch { .. }
                | DeletionStepOfThePublish::TruncationFinalBatch { .. }
                | DeletionStepOfThePublish::SizeChangeWithoutAnIntent { .. }
                | DeletionStepOfThePublish::PartialWrite { .. }
                | DeletionStepOfThePublish::NotADeletion,
                _,
            ) => plan_the_extent_tree_after_this_publish(
                previous,
                self.first_file_version().map(|_| {
                    u64::try_from(file_content_transactions.len()).expect("这次写的单元数")
                }),
            ),
        };
        // 写第一个文件之外的文件：第一个文件的下段照抄，上段按这次之后有条目的 inode 号重排、那个文件那一串重写
        // （[`plan_the_extent_tree_after_writing_a_file_other_than_the_first`]）。
        let extent_tree = match (self.inode_of_the_file_other_than_the_first_written(), previous) {
            (Some(inode), Some(previous_version)) => {
                plan_the_extent_tree_after_writing_a_file_other_than_the_first(
                    previous_version,
                    inode,
                )
            }
            (Some(inode), None) => unreachable!(
                "写第一个文件之外的文件（inode {}）的入口恒接在上一版之后（`publish_sequential_write_of_an_inode` 收 `previous: &TransactionOutput`）",
                inode.0
            ),
            (None, _) => extent_tree_of_the_step,
        };
        let data_roles_rewritten: Vec<TransactionUnit> = file_content_transactions
            .iter()
            .map(|transaction| {
                self.role_of_a_data_unit_written(
                    transaction.unit_index_in_file,
                    deletion_and_lineage
                        .step
                        .data_units_rewritten_from_their_payloads()
                        .map(|rewritten| rewritten.inode),
                )
            })
            .collect();
        let deleted_inodes_before: &[InodeNumber] = match previous {
            Some(version) => &version.deleted_inodes,
            None => &[],
        };
        let deleted_inodes = deletion_and_lineage
            .step
            .deleted_inodes_after_this_publish(deleted_inodes_before);
        deleted_inodes_root_entries(&deleted_inodes)
            .map_err(PublishError::DeletedInodesQueueRefused)?;
        let deleted_inodes_root = if deleted_inodes.is_empty() {
            DeletedInodesRootOrigin::NoRoot
        } else if deleted_inodes == deleted_inodes_before {
            DeletedInodesRootOrigin::CarriedFromThePreviousVersion
        } else {
            DeletedInodesRootOrigin::RewrittenThisPublish
        };
        let tombstone_containers_before: Vec<TombstoneContainer> = previous
            .map(|version| {
                version
                    .tombstone_containers
                    .iter()
                    .map(|container| container.contents.clone())
                    .collect()
            })
            .unwrap_or_default();
        // 上一版的形状借过来用，不先照抄一份：追加那一支只拿它的引用（新一版的形状在追加里从它造），照抄那一支才要一份自己的。
        let zero_root_shape = ContainerTreeShape::zero_root();
        let tombstone_shape_before: &ContainerTreeShape = previous
            .map_or(&zero_root_shape, |version| {
                &version.tombstone_tree_index_nodes.shape
            });
        let tombstone_tree = match deletion_and_lineage.step {
            DeletionStepOfThePublish::Intent { tombstone, .. } => {
                append_tombstone_to_the_open_container(
                    &tombstone_containers_before,
                    tombstone_shape_before,
                    tombstone,
                    self.txg,
                    trees.tombstone,
                    deletion_and_lineage
                        .container_tree_capacities
                        .tombstone_container,
                    deletion_and_lineage.container_tree_capacities.node_entries,
                )
                .map_err(PublishError::NextContainerNumberPastTheTopOfItsRange)?
            }
            DeletionStepOfThePublish::Batch { .. }
            | DeletionStepOfThePublish::FinalBatch { .. }
            | DeletionStepOfThePublish::TruncationIntent { .. }
            | DeletionStepOfThePublish::TruncationBatch { .. }
            | DeletionStepOfThePublish::TruncationFinalBatch { .. }
            | DeletionStepOfThePublish::SizeChangeWithoutAnIntent { .. }
            | DeletionStepOfThePublish::PartialWrite { .. }
            | DeletionStepOfThePublish::NotADeletion => TombstoneTreeAfterThisPublish::carried(
                tombstone_containers_before,
                tombstone_shape_before.clone(),
            ),
        };
        // 退役树（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 3–5 条）：这次身份消失的每一片 inode 叶容器
        // （删空摘掉、被左邻吸收）追加一条，死亡 txg = 这次发布的 checkpoint_txg，按它们在上一版里的叶序；写者只追加，不去重、不覆盖。
        // 借不写（右片保留身份，不在 `retired` 里）。没有身份消失的发布整棵照抄。
        let retirement_containers_before: Vec<RetirementContainer> = previous
            .map(|version| {
                version
                    .retirement_containers
                    .iter()
                    .map(|container| container.contents.clone())
                    .collect()
            })
            .unwrap_or_default();
        let retirement_shape_before: &ContainerTreeShape = previous
            .map_or(&zero_root_shape, |version| {
                &version.retirement_tree_index_nodes.shape
            });
        let retirement_tree = append_retirement_records_to_the_open_container(
            &retirement_containers_before,
            retirement_shape_before,
            &retirement_records_of_this_publish(&inode_tree.retired_identities(), self.txg),
            self.txg,
            trees.retirement,
            deletion_and_lineage
                .container_tree_capacities
                .retirement_container,
            deletion_and_lineage.container_tree_capacities.node_entries,
        )
        .map_err(PublishError::NextContainerNumberPastTheTopOfItsRange)?;
        // logged_ops（判决 G1）：整理的发布签发一条或删一条，别的照抄；装不下一个根走不到（全池活整理意图至多一条）。
        let logged_operations_before: &[LoggedOperationEntry] = match previous {
            Some(version) => &version.logged_operations,
            None => &[],
        };
        let logged_operations = deletion_and_lineage
            .logged_operations
            .logged_operations_after_this_publish(logged_operations_before, self.txg);
        let logged_operations_root = if logged_operations.is_empty() {
            LoggedOperationsRootOrigin::NoRoot
        } else if logged_operations == logged_operations_before {
            LoggedOperationsRootOrigin::CarriedFromThePreviousVersion
        } else {
            LoggedOperationsRootOrigin::RewrittenThisPublish
        };
        Ok(OrdinaryAllocationPartOfThePublish {
            inode_tree,
            inode_record_counts,
            extent_tree,
            files_other_than_the_first,
            data_roles_rewritten,
            carried_data_units,
            deleted_inodes,
            deleted_inodes_root,
            tombstone_tree,
            retirement_tree,
            logged_operations,
            logged_operations_root,
        })
    }

    /// 把这张计划算成「这次之后 extent 树、inode 树、记账树、中央映射树各长什么样、这次重写哪些角色」。`trees` 是这一版那十二棵树的号
    /// （接在上一版之后就是上一版的，第一个文件版本是这次发出来的），新分裂出来的叶容器的出生树取其中的 inode 树。
    /// `device_identities` 是记账行按盘展开的那几块盘（与装记账行时的分配器同一批），`capacities` 是两棵多层码 2 树的节点容量
    /// （写入口上那个只供测试的开关，产品路径按节点格式算）。`allocation_record_tree` 是这次之后分配记录树的样子：它重写哪几个节点
    /// 要在分配器的拷贝上走到固定点才知道（取落点会改记录），由调用方定下来交进来（`settle_the_allocation_record_tree`），
    /// 这里按它排角色、发映射 key。`mapping_keys_moved_by_this_publish` 是整理的一批搬走的单元的映射 key（别的发布给空的）：
    /// 它们 key 不变、条目的位置换了，中央映射树里装着它们的叶连同到根的路这次重写（[`plan_rewriting_the_paths_to_the_moved_entries`]）。
    ///
    /// # Errors
    /// inode 记录的落法要的条款仓里还没定 ⇒ `InodeTreeWriteRefused`（`crate::inode_tree` 的三格）；
    /// 记账树或中央映射树这次之后的形状算不出来 ⇒ `MultiLevelCodeTwoTreeRefused`；
    /// 墓碑树、谱系树或退役树要开新容器而最右那片的号已到顶 ⇒ `NextContainerNumberPastTheTopOfItsRange`（C841）。
    #[allow(
        clippy::too_many_arguments,
        reason = "参数各是一样东西：计划、上一版、树号、记账行的盘、节点容量、分配记录树、删除这一步、这次搬走的映射 key、释放判定那一臂"
    )]
    pub fn resolve(
        &self,
        previous: Option<&TransactionOutput>,
        trees: &FileVersionTreeIdentifiers,
        device_identities: &[DeviceIdentity],
        capacities: CodeTwoTreeNodeCapacities,
        allocation_record_tree: AllocationRecordTreePlan,
        deletion_and_lineage: &DeletionAndLineageOfThePublish,
        mapping_keys_moved_by_this_publish: &[Vec<u8>],
        snapshot_lifetime_judgement: SnapshotLifetimeJudgement,
    ) -> Result<ResolvedPublish, PublishError> {
        let ordinary_part = self.ordinary_allocation_part(previous, trees, deletion_and_lineage)?;
        let deletion_roles_rewritten = ordinary_part.deletion_roles_rewritten();
        let logged_operations_roles_rewritten = ordinary_part.logged_operations_roles_rewritten();
        let retirement_roles_rewritten = ordinary_part.retirement_roles_rewritten();
        let lineage_tree =
            self.lineage_tree_after_this_publish(previous, trees, deletion_and_lineage)?;
        // 没有同名后继的上一版角色：第一个文件那几样（[`OrdinaryAllocationPartOfThePublish::previous_roles_without_a_successor`]）加
        // 删除的一批与最后一批截掉、摘掉的第一个文件之外那个文件的数据单元与下段节点（判决 m3-multi-file-unlink-r1 第三节 M1）。
        let mut previous_roles_without_a_successor: Vec<TransactionUnit> = previous
            .map(|previous_version| {
                ordinary_part.previous_roles_without_a_successor(
                    previous_version,
                    self.first_file_version().is_some(),
                )
            })
            .unwrap_or_default();
        previous_roles_without_a_successor.extend(
            ordinary_part
                .files_other_than_the_first
                .previous_roles_without_a_successor
                .iter()
                .copied(),
        );
        let OrdinaryAllocationPartOfThePublish {
            inode_tree,
            inode_record_counts,
            extent_tree,
            files_other_than_the_first,
            data_roles_rewritten,
            carried_data_units,
            deleted_inodes,
            deleted_inodes_root,
            tombstone_tree,
            retirement_tree,
            logged_operations,
            logged_operations_root,
        } = ordinary_part;
        let mut rewritten_roles: Vec<TransactionUnit> = data_roles_rewritten;
        if let InstanceTablePlan::Rewrite(rewrite) = &self.instance_table {
            rewritten_roles.extend(instance_table_page_roles_in_bump_order(
                rewrite.pages_after_this_publish(),
            ));
        }
        rewritten_roles.extend(extent_tree_roles_rewritten(
            &extent_tree,
            &files_other_than_the_first,
        ));
        for index in &inode_tree.rewritten {
            rewritten_roles.push(TransactionUnit::InodeLeafContainer(*index));
        }
        if inode_tree.is_the_root_rewritten() {
            rewritten_roles.push(TransactionUnit::InodeRoot);
        }
        // 释放判定分流（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3）：这次换下的用户可见单元——这次重写的数据单元、
        // inode 叶容器与根，没有同名后继的那几个，extent 树按计划换下的节点——逐个问 D5（快照 / 空间记账机制） 已定项 19 的判定，
        // 被活快照引用的进 deadlist、不释放、映射条目留存；删快照这一批放掉的条目也在这里定。要在中央映射树与 deadlist 树规划之前定：
        // 两棵树的 key 都要它。释放判定路径（`placements_released_by_this_publish`）核它与按全部换下角色走出来的那一份相同。
        let user_visible_roles_rewritten_or_without_a_successor: Vec<TransactionUnit> =
            rewritten_roles
                .iter()
                .copied()
                .filter(|identity| {
                    matches!(
                        identity,
                        TransactionUnit::Data(_)
                            | TransactionUnit::InodeLeafContainer(_)
                            | TransactionUnit::InodeRoot
                    )
                })
                .chain(
                    previous_roles_without_a_successor
                        .iter()
                        .copied()
                        .filter(|identity| asks_the_snapshot_lifetime_judgement(*identity)),
                )
                .chain(extent_tree.replaced_previous_roles.iter().copied())
                // 写第一个文件之外的文件：它上一版的每个数据单元与下段节点整份换下（重写出来的同名角色不在上面那一段里，免得点两次）。
                .chain(files_other_than_the_first.replaced_previous_roles())
                .collect();
        let deadlist = deadlist_changes_of_this_publish(
            previous,
            trees,
            &user_visible_roles_rewritten_or_without_a_successor,
            deletion_and_lineage.snapshot,
            self.txg,
            snapshot_lifetime_judgement,
            FileContentOfThePublish::of_the_plan(self),
        )?;
        rewritten_roles.extend(
            allocation_record_tree
                .rewritten_nodes()
                .into_iter()
                .map(role_of_allocation_record_tree_node),
        );
        let MultiLevelTreesOfThisPublish {
            accounting_tree,
            deadlist_tree,
            central_mapping_tree,
            mapped_units,
        } = self.plan_the_multi_level_trees(
            previous,
            trees,
            &MappedPositionAddressedTrees {
                extent: &extent_tree,
                files_other_than_the_first: &files_other_than_the_first,
                allocation_records: &allocation_record_tree,
            },
            &inode_tree,
            &DeletionTreesOfThisPublish {
                carried_data_units,
                data_units_rewritten_from_their_payloads: deletion_and_lineage
                    .step
                    .data_units_rewritten_from_their_payloads()
                    .map(|rewritten| {
                        (
                            rewritten.inode,
                            self.file_content_transactions_and_the_rewritten_data_units(Some(
                                rewritten,
                            )),
                        )
                    }),
                deleted_inodes_root,
                tombstone_tree: &tombstone_tree,
                lineage_tree: &lineage_tree,
                logged_operations_root,
                retirement_tree: &retirement_tree,
            },
            &deadlist,
            device_identities,
            capacities,
            mapping_keys_moved_by_this_publish,
        )?;
        // 树内先叶后根、同层按 key 升序（D3（空间分配） 已定项 10 ⑤），记账树（树 65539）在前、删除的两棵（65544、65545）接着、
        // 谱系树（65546）再接着、中央映射树倒数第二、树表最末。谱系树的节点归固定点（D28（挂载期承诺量） 已定项 1 接线「谱系容器归固定点、
        // 进 ckpt_cost 的 Σ 名单」），不在普通分配那一半里。
        rewritten_roles.extend(
            accounting_tree
                .rewritten_positions()
                .into_iter()
                .map(|position| {
                    MultiLevelCodeTwoTree::Accounting.role_of_node(position, &accounting_tree.shape)
                }),
        );
        // deadlist 树（树 65543）按树 ID 排在记账树之后、删除那两棵树之前。
        rewritten_roles.extend(
            deadlist_tree
                .rewritten_positions()
                .into_iter()
                .map(|position| {
                    MultiLevelCodeTwoTree::Deadlist.role_of_node(position, &deadlist_tree.shape)
                }),
        );
        rewritten_roles.extend(deletion_roles_rewritten);
        rewritten_roles.extend(container_tree_roles_rewritten(
            &lineage_tree,
            ContainerTreeRoles::OF_THE_LINEAGE_TREE,
        ));
        // logged_ops 根（树 ID 接在谱系树之后，判决 G1）：普通分配，按树 ID 排在谱系树之后、退役树之前。
        rewritten_roles.extend(logged_operations_roles_rewritten);
        // 退役树（树 ID 接在 logged_ops 之后，判决 m3-retire-r1 第三节第 1 条）：普通分配，按树 ID 排在谱系树之后。
        rewritten_roles.extend(retirement_roles_rewritten);
        rewritten_roles.extend(central_mapping_tree.rewritten_positions().into_iter().map(
            |position| {
                MultiLevelCodeTwoTree::CentralMapping
                    .role_of_node(position, &central_mapping_tree.shape)
            },
        ));
        rewritten_roles.push(TransactionUnit::TreeTable);
        Ok(ResolvedPublish {
            inode_tree,
            inode_record_counts,
            extent_tree,
            files_other_than_the_first,
            allocation_record_tree,
            accounting_tree,
            central_mapping_tree,
            deadlist_tree,
            deadlist,
            mapped_units,
            rewritten_roles,
            carried_data_units,
            deleted_inodes,
            deleted_inodes_root,
            tombstone_tree,
            lineage_tree,
            retirement_tree,
            logged_operations,
            logged_operations_root,
            previous_roles_without_a_successor,
            data_units_rewritten_from_their_payloads: deletion_and_lineage
                .step
                .data_units_rewritten_from_their_payloads(),
        })
    }

    /// 这次之后谱系树的样子：管理员回退那次往开放容器追加那一条（容器号同墓碑规则，根满时长高），别的发布照抄上一版。
    ///
    /// # Errors
    /// 要开新容器而最右那片的号已到顶 ⇒ `NextContainerNumberPastTheTopOfItsRange`（C841）。
    fn lineage_tree_after_this_publish(
        &self,
        previous: Option<&TransactionOutput>,
        trees: &FileVersionTreeIdentifiers,
        deletion_and_lineage: &DeletionAndLineageOfThePublish,
    ) -> Result<LineageTreeAfterThisPublish, PublishError> {
        let containers_before: Vec<LineageContainer> = previous
            .map(|version| {
                version
                    .lineage_containers
                    .iter()
                    .map(|container| container.contents.clone())
                    .collect()
            })
            .unwrap_or_default();
        // 上一版的形状借过来用（同墓碑树那一处）：追加那一支只拿引用，照抄那一支才要一份自己的。
        let zero_root_shape = ContainerTreeShape::zero_root();
        let shape_before: &ContainerTreeShape = previous.map_or(&zero_root_shape, |version| {
            &version.lineage_tree_index_nodes.shape
        });
        match deletion_and_lineage.lineage {
            LineageStepOfThePublish::AdministratorRollback { record, .. } => {
                append_lineage_record_to_the_open_container(
                    &containers_before,
                    shape_before,
                    record,
                    self.txg,
                    trees.lineage,
                    deletion_and_lineage
                        .container_tree_capacities
                        .lineage_container,
                    deletion_and_lineage.container_tree_capacities.node_entries,
                )
                .map_err(PublishError::NextContainerNumberPastTheTopOfItsRange)
            }
            LineageStepOfThePublish::NoLineageRecord => Ok(LineageTreeAfterThisPublish::carried(
                containers_before,
                shape_before.clone(),
            )),
        }
    }

    /// 记账树与中央映射树这次之后的形状，与这一版每个进映射单元的映射 key（`resolve` 的后一半）。
    #[allow(
        clippy::too_many_arguments,
        reason = "参数各是一样东西：计划、上一版、树号、两棵按位置寻址的树、inode 树、删除那两棵树、deadlist 的改动、记账行的盘、节点容量、\
                  这次搬走的映射 key"
    )]
    fn plan_the_multi_level_trees(
        &self,
        previous: Option<&TransactionOutput>,
        trees: &FileVersionTreeIdentifiers,
        position_addressed_trees: &MappedPositionAddressedTrees<'_>,
        inode_tree: &InodeLeafContainersAfterThisPublish,
        deletion_trees: &DeletionTreesOfThisPublish<'_>,
        deadlist: &DeadlistChangesOfThisPublish,
        device_identities: &[DeviceIdentity],
        capacities: CodeTwoTreeNodeCapacities,
        mapping_keys_moved_by_this_publish: &[Vec<u8>],
    ) -> Result<MultiLevelTreesOfThisPublish, PublishError> {
        let txg = self.txg;
        let instance = self.instance;
        let empty_shape = CodeTwoTreeShape::default();
        let previous_shape_of = |tree: MultiLevelCodeTwoTree| match previous {
            Some(previous_version) => &previous_version.multi_level_tree(tree).shape,
            None => &empty_shape,
        };
        let refused = |tree: MultiLevelCodeTwoTree| {
            move |refusal: CodeTwoTreeRefusal| PublishError::MultiLevelCodeTwoTreeRefused {
                tree,
                refusal,
            }
        };
        // 记账树：每次发布整批重写（代 = 这次的 txg），key 只看统计量、树、盘与代，不看值；inode 号水位的台阶行照抄上一版的。
        let inode_number_watermark_steps_carried =
            previous.map_or_else(Vec::new, TransactionOutput::inode_number_watermark_steps);
        let accounting_keys: BTreeSet<CodeTwoTreeKey> = accounting_entries_of_this_publish(
            txg,
            trees.inode,
            device_identities,
            &inode_number_watermark_steps_carried,
            &|_| 0,
            &|_, _| 0,
        )
        .iter()
        .map(|entry| CodeTwoTreeKey::new(&entry.key_bytes(), CodeTwoKeyFieldWidths::ACCOUNTING))
        .collect();
        let accounting_tree = plan_the_tree_after_this_publish(
            previous_shape_of(MultiLevelCodeTwoTree::Accounting),
            &accounting_keys,
            capacities.of_tree(MultiLevelCodeTwoTree::Accounting),
        )
        .map_err(refused(MultiLevelCodeTwoTree::Accounting))?;
        // deadlist 树（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3）：按这次之后的条目规划，一条都不剩时零根。
        let deadlist_keys: BTreeSet<CodeTwoTreeKey> = deadlist
            .entries_after_this_publish
            .iter()
            .map(|entry| CodeTwoTreeKey::new(&entry.key_bytes(), CodeTwoKeyFieldWidths::DEADLIST))
            .collect();
        let deadlist_tree = plan_the_tree_after_this_publish_down_to_a_zero_root(
            previous_shape_of(MultiLevelCodeTwoTree::Deadlist),
            &deadlist_keys,
            capacities.of_tree(MultiLevelCodeTwoTree::Deadlist),
        )
        .map_err(refused(MultiLevelCodeTwoTree::Deadlist))?;

        // 这一版每个进映射的单元的 key：重写的现算，照抄的取上一版的。出生序号在各自那棵树里按 bump 次序从 0 发（数据单元不发，
        // 码 1 的 key 带写序）：extent 树下段先叶后根、再上段先叶后根，inode 树先叶后根，分配记录树按位置先叶后根。
        let node_key = |unit_class: u8, tree: TreeIdentifier, birth_sequence: usize| {
            mapping_key_of_a_node_born_in_this_publish(
                unit_class,
                tree,
                txg,
                instance,
                BirthSequence(
                    u32::try_from(birth_sequence).expect("一棵树一次发布的单元数装得进 u32"),
                ),
            )
        };
        // 照抄的角色约有上一版那么多个，每个按角色取上一版的 key：先建上一版 `mapped_units` 的索引（七·6），不逐个在表上找。
        let mapped_units_of_the_previous_version =
            previous.map(TransactionOutput::mapped_units_by_role);
        let carried = |identity: TransactionUnit| {
            mapped_units_of_the_previous_version
                .as_ref()
                .expect("照抄的角色只出现在接着上一版的发布里")
                .mapping_key_of(identity)
                .to_vec()
        };
        let mut mapped_units: Vec<(TransactionUnit, Vec<u8>)> = Vec::new();
        match self.first_file_version() {
            Some(_) => {
                for transaction in self.file_content_transactions() {
                    mapped_units.push((
                        TransactionUnit::Data(transaction.unit_index_in_file),
                        mapping_key_for_data(
                            PointerHead {
                                birth_tree: trees.extent,
                                birth_txg: txg,
                            },
                            WriteOrder {
                                instance,
                                transaction: transaction.transaction_number,
                            },
                        ),
                    ));
                }
            }
            None => {
                let data_units_of_the_carried_version = previous
                    .expect("没有文件版本的发布要接在上一版之后：文件角色从它照抄（删除的一批只照抄前几个）")
                    .data_pointers
                    .len();
                let transactions_rewritten_in_the_first_file: &[OneUnitTransaction] =
                    deletion_trees
                        .data_units_rewritten_from_their_payloads
                        .as_ref()
                        .filter(|(inode, _)| inode.0 == FIRST_INODE_NUMBER)
                        .map_or(&[], |(_, transactions)| transactions.as_slice());
                for position in 0..deletion_trees.carried_data_units {
                    let unit = DataUnitIndexInFile(u64::try_from(position).expect("单元序号"));
                    let identity = TransactionUnit::Data(unit);
                    match transaction_of_a_rewritten_data_unit(
                        transactions_rewritten_in_the_first_file,
                        unit,
                    ) {
                        // 按读回的旧单元（或零）重写的那一段：各一个事务，事务号是切给它的那一个
                        // （`file_content_transactions_and_the_rewritten_data_units`）。
                        Some(transaction) => mapped_units.push((
                            identity,
                            mapping_key_for_data(
                                PointerHead {
                                    birth_tree: trees.extent,
                                    birth_txg: txg,
                                },
                                WriteOrder {
                                    instance,
                                    transaction: transaction.transaction_number,
                                },
                            ),
                        )),
                        None => {
                            assert!(
                                position < data_units_of_the_carried_version,
                                "照抄的数据单元在上一版里：删除的一批只照抄前几个，加单元的只有部分写、它重写到最后一个"
                            );
                            mapped_units.push((identity, carried(identity)));
                        }
                    }
                }
            }
        }
        // 第一个文件之外各文件的数据单元（按 inode 号、单元序号）：这次写的那一个按这次的事务现算，别的照抄上一版的 key。
        let transactions_of_the_file_other_than_the_first_written: Vec<OneUnitTransaction> =
            match self.inode_of_the_file_other_than_the_first_written() {
                Some(_) => self.file_content_transactions(),
                None => Vec::new(),
            };
        for file in &position_addressed_trees.files_other_than_the_first.files {
            for position in 0..file.data_units {
                let identity = file.data_role_at(position);
                match file.origin {
                    // 改 size 那次重写的尾单元（截断意图、变长，判决 m4-multi-inode-size-r1 第二节 S3）与部分写重写的那一段（判决 m4-partial-write-r1
                    // 第二节 R6）：各一个事务，事务号是切给它的那一个（同第一个文件，`file_content_transactions_and_the_rewritten_data_units`）。
                    FileOtherThanTheFirstOrigin::SizeChangedThisPublish { .. }
                        if file
                            .origin
                            .rewrites_the_data_unit(DataUnitIndexInFile(position)) =>
                    {
                        let transaction = deletion_trees
                            .data_units_rewritten_from_their_payloads
                            .as_ref()
                            .filter(|(inode, _)| *inode == file.inode)
                            .and_then(|(_, transactions)| {
                                transaction_of_a_rewritten_data_unit(
                                    transactions,
                                    DataUnitIndexInFile(position),
                                )
                            })
                            .expect("计划里重写的单元就是这一步重写的那一段里的（同一份计划拼的）");
                        mapped_units.push((
                            identity,
                            mapping_key_for_data(
                                PointerHead {
                                    birth_tree: trees.extent,
                                    birth_txg: txg,
                                },
                                WriteOrder {
                                    instance,
                                    transaction: transaction.transaction_number,
                                },
                            ),
                        ));
                    }
                    FileOtherThanTheFirstOrigin::RewrittenThisPublish => {
                        let transaction = transactions_of_the_file_other_than_the_first_written
                            .get(usize::try_from(position).expect("单元序号"))
                            .expect("这次写的那个文件的单元数就是这次切出的事务数");
                        mapped_units.push((
                            identity,
                            mapping_key_for_data(
                                PointerHead {
                                    birth_tree: trees.extent,
                                    birth_txg: txg,
                                },
                                WriteOrder {
                                    instance,
                                    transaction: transaction.transaction_number,
                                },
                            ),
                        ));
                    }
                    FileOtherThanTheFirstOrigin::CarriedFromThePreviousVersion
                    | FileOtherThanTheFirstOrigin::CutByADeletionBatch
                    | FileOtherThanTheFirstOrigin::SizeChangedThisPublish { .. } => {
                        mapped_units.push((identity, carried(identity)));
                    }
                }
            }
        }
        // extent 树的每个节点（下段与上段）都进映射（码 2 节点，D19（块指针的结构与宽度预算） 已定项 8）：重写的按 bump 次序发出生序号，
        // 照抄的取上一版那个角色的 key（上段的根层级这次没变时角色名照旧；变了的只有这次重写的那一串）。
        let extent_plan = position_addressed_trees.extent;
        let previous_upper_root_level =
            previous.map(|previous_version| previous_version.extent_tree.upper_root().level);
        let mut extent_birth_sequence = 0;
        let upper_root_level = extent_plan.upper_root_level();
        // 第一个文件的下段、第一个文件之外各文件的下段（按 inode 号）、上段：与装树时发出生序号的次序相同（`build_extent_tree`）。
        let mut extent_nodes: Vec<(TransactionUnit, TransactionUnit, ExtentTreeNodeOrigin)> =
            extent_plan
                .lower_nodes
                .iter()
                .map(|(position, origin)| {
                    (
                        TransactionUnit::ExtentLowerNode(*position),
                        TransactionUnit::ExtentLowerNode(*position),
                        *origin,
                    )
                })
                .collect();
        for file in &position_addressed_trees.files_other_than_the_first.files {
            extent_nodes.extend(file.lower_nodes.iter().map(|(position, origin)| {
                let role = file.lower_node_role(*position);
                (role, role, *origin)
            }));
        }
        extent_nodes.extend(extent_plan.upper_nodes.iter().map(|(position, origin)| {
            (
                role_of_extent_upper_node(*position, upper_root_level),
                role_of_extent_upper_node(
                    *position,
                    previous_upper_root_level.unwrap_or(upper_root_level),
                ),
                *origin,
            )
        }));
        for (identity, identity_in_the_previous_version, origin) in extent_nodes {
            match origin {
                ExtentTreeNodeOrigin::RewrittenThisPublish => {
                    mapped_units.push((
                        identity,
                        node_key(UNIT_CLASS_INDEX_NODE, trees.extent, extent_birth_sequence),
                    ));
                    extent_birth_sequence += 1;
                }
                ExtentTreeNodeOrigin::CarriedFromThePreviousVersion => {
                    mapped_units.push((identity, carried(identity_in_the_previous_version)));
                }
            }
        }
        let mut inode_tree_birth_sequence = 0;
        for (position, origin) in inode_tree.origins.iter().enumerate() {
            let identity = TransactionUnit::InodeLeafContainer(
                InodeLeafContainerIndexInTree::of_position(position),
            );
            match origin {
                InodeLeafContainerOrigin::RewrittenThisPublish => {
                    mapped_units.push((
                        identity,
                        node_key(UNIT_CLASS_PACKED, trees.inode, inode_tree_birth_sequence),
                    ));
                    inode_tree_birth_sequence += 1;
                }
                // 照抄的一片取它在上一版里那个叶序的 key（左边有片身份消失时它在上一版里更靠右）。
                InodeLeafContainerOrigin::CarriedFrom(previous_index) => {
                    mapped_units.push((
                        identity,
                        carried(TransactionUnit::InodeLeafContainer(*previous_index)),
                    ));
                }
            }
        }
        mapped_units.push(if !inode_tree.is_the_root_rewritten() {
            (
                TransactionUnit::InodeRoot,
                carried(TransactionUnit::InodeRoot),
            )
        } else {
            (
                TransactionUnit::InodeRoot,
                node_key(
                    UNIT_CLASS_INDEX_NODE,
                    trees.inode,
                    inode_tree_birth_sequence,
                ),
            )
        });
        // 分配记录树的每个节点都进映射：重写的按位置先叶后根发出生序号，照抄的取上一版同一个位置那个角色的 key（位置不随版本挪动）。
        let mut allocation_birth_sequence = 0;
        for (node, origin) in &position_addressed_trees.allocation_records.nodes {
            let identity = role_of_allocation_record_tree_node(*node);
            match origin {
                AllocationRecordTreeNodeOrigin::RewrittenThisPublish => {
                    mapped_units.push((
                        identity,
                        node_key(
                            UNIT_CLASS_INDEX_NODE,
                            trees.allocation_records,
                            allocation_birth_sequence,
                        ),
                    ));
                    allocation_birth_sequence += 1;
                }
                AllocationRecordTreeNodeOrigin::CarriedFromThePreviousVersion => {
                    mapped_units.push((identity, carried(identity)));
                }
            }
        }
        let previous_accounting_shape = previous_shape_of(MultiLevelCodeTwoTree::Accounting);
        let mut accounting_birth_sequence = 0;
        for (node, origin) in accounting_tree
            .shape
            .nodes()
            .iter()
            .zip(&accounting_tree.origins)
        {
            let identity = MultiLevelCodeTwoTree::Accounting
                .role_of_node(node.position, &accounting_tree.shape);
            match origin {
                CodeTwoTreeNodeOrigin::RewrittenThisPublish => {
                    mapped_units.push((
                        identity,
                        node_key(
                            UNIT_CLASS_INDEX_NODE,
                            trees.accounting,
                            accounting_birth_sequence,
                        ),
                    ));
                    accounting_birth_sequence += 1;
                }
                CodeTwoTreeNodeOrigin::CarriedFrom(previous_position) => {
                    let previous_identity = MultiLevelCodeTwoTree::Accounting
                        .role_of_node(*previous_position, previous_accounting_shape);
                    mapped_units.push((identity, carried(previous_identity)));
                }
            }
        }
        // deadlist 树的每个节点都进映射（码 2 树节点，同记账树）：按 bump 次序，记账树之后、删除那两棵树之前（按树 ID）。
        let previous_deadlist_shape = previous_shape_of(MultiLevelCodeTwoTree::Deadlist);
        let mut deadlist_birth_sequence = 0;
        for (node, origin) in deadlist_tree
            .shape
            .nodes()
            .iter()
            .zip(&deadlist_tree.origins)
        {
            let identity =
                MultiLevelCodeTwoTree::Deadlist.role_of_node(node.position, &deadlist_tree.shape);
            match origin {
                CodeTwoTreeNodeOrigin::RewrittenThisPublish => {
                    mapped_units.push((
                        identity,
                        node_key(
                            UNIT_CLASS_INDEX_NODE,
                            trees.deadlist,
                            deadlist_birth_sequence,
                        ),
                    ));
                    deadlist_birth_sequence += 1;
                }
                CodeTwoTreeNodeOrigin::CarriedFrom(previous_position) => {
                    let previous_identity = MultiLevelCodeTwoTree::Deadlist
                        .role_of_node(*previous_position, previous_deadlist_shape);
                    mapped_units.push((identity, carried(previous_identity)));
                }
            }
        }
        // 删除那两棵树（记账树之后、按树 ID）：`deleted_inodes` 根（队列非空时），墓碑树的每片容器先、根后。
        match deletion_trees.deleted_inodes_root {
            DeletedInodesRootOrigin::NoRoot => {}
            DeletedInodesRootOrigin::CarriedFromThePreviousVersion => mapped_units.push((
                TransactionUnit::DeletedInodesRoot,
                carried(TransactionUnit::DeletedInodesRoot),
            )),
            DeletedInodesRootOrigin::RewrittenThisPublish => mapped_units.push((
                TransactionUnit::DeletedInodesRoot,
                node_key(UNIT_CLASS_INDEX_NODE, trees.deleted_inodes, 0),
            )),
        }
        // 墓碑树、谱系树（按树 ID）：每片容器先、根之下的节点按层级升序、根最末。
        mapped_units.extend(mapped_units_of_a_container_tree(
            deletion_trees.tombstone_tree,
            ContainerTreeRoles::OF_THE_TOMBSTONE_TREE,
            &|unit_class, birth_sequence| node_key(unit_class, trees.tombstone, birth_sequence),
            &carried,
        ));
        mapped_units.extend(mapped_units_of_a_container_tree(
            deletion_trees.lineage_tree,
            ContainerTreeRoles::OF_THE_LINEAGE_TREE,
            &|unit_class, birth_sequence| node_key(unit_class, trees.lineage, birth_sequence),
            &carried,
        ));
        // logged_ops 根（谱系树之后、退役树之前，按树 ID）：有条目时一个码 2 根。
        match deletion_trees.logged_operations_root {
            LoggedOperationsRootOrigin::NoRoot => {}
            LoggedOperationsRootOrigin::CarriedFromThePreviousVersion => mapped_units.push((
                TransactionUnit::LoggedOperationsRoot,
                carried(TransactionUnit::LoggedOperationsRoot),
            )),
            LoggedOperationsRootOrigin::RewrittenThisPublish => mapped_units.push((
                TransactionUnit::LoggedOperationsRoot,
                node_key(UNIT_CLASS_INDEX_NODE, trees.logged_operations, 0),
            )),
        }
        mapped_units.extend(mapped_units_of_a_container_tree(
            deletion_trees.retirement_tree,
            ContainerTreeRoles::OF_THE_RETIREMENT_TREE,
            &|unit_class, birth_sequence| node_key(unit_class, trees.retirement, birth_sequence),
            &carried,
        ));
        // 被活快照钉住的单元的映射条目留在树里（判决 S3 的 α）：它们的 key 与这一版的单元的 key 一起排。
        let mapping_keys: BTreeSet<CodeTwoTreeKey> = mapped_units
            .iter()
            .map(|(_, key)| key)
            .chain(
                deadlist
                    .kept_mapping_entries_after_this_publish
                    .iter()
                    .map(|(key, _)| key),
            )
            .map(|key| CodeTwoTreeKey::new(key, CodeTwoKeyFieldWidths::CENTRAL_MAPPING))
            .collect();
        assert_eq!(
            mapping_keys.len(),
            mapped_units.len() + deadlist.kept_mapping_entries_after_this_publish.len(),
            "进映射的单元与留存的映射条目各一把 key，互不相撞（D19（块指针的结构与宽度预算） 已定项 6：出生身份各不相同；留存的是上一版里\
             被杀的单元的 key，这一版的单元不再引用它们）"
        );
        let central_mapping_tree = plan_rewriting_the_paths_to_the_moved_entries(
            plan_the_tree_after_this_publish(
                previous_shape_of(MultiLevelCodeTwoTree::CentralMapping),
                &mapping_keys,
                capacities.of_tree(MultiLevelCodeTwoTree::CentralMapping),
            )
            .map_err(refused(MultiLevelCodeTwoTree::CentralMapping))?,
            previous_shape_of(MultiLevelCodeTwoTree::CentralMapping),
            mapping_keys_moved_by_this_publish
                .iter()
                .map(|key| CodeTwoTreeKey::new(key, CodeTwoKeyFieldWidths::CENTRAL_MAPPING))
                .filter(|key| mapping_keys.contains(key)),
        );
        Ok(MultiLevelTreesOfThisPublish {
            accounting_tree,
            deadlist_tree,
            central_mapping_tree,
            mapped_units,
        })
    }
}
