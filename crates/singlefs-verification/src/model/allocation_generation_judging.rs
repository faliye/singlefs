//! 判实现交回的每个单元那几条分配记录：每块盘一条、仍分配、分配代等于写它的那次发布。

use super::*;

impl IdealModel {
    /// 这一版每个单元的分配记录：每块盘各一条、仍分配着、分配代等于写它的那次发布的 txg（D3（空间分配） 已定项 3 / 7）。
    /// 两个方向都比（代码审阅第 12 条）：实现交回的每个角色这一版都要有，模型这一版的每个角色实现都要交回。
    /// 零单元发布（树表 0 条的一版上）输出不带分配记录，比的是这次发布重写了哪几个角色：模型那一侧是分配代等于这次 txg 的那几个，
    /// 零单元发布一个都没有；模型说这次重写了角色的那一版（写行、带文件的各种发布）只交回重写的角色，就是对不上。
    pub(super) fn judge_allocation_generations(
        &self,
        expected: &ModelRoot,
        observed: &ObservedRoot,
        counts: &mut ModelJudgementCounts,
    ) -> Result<(), ModelDisagreement> {
        let observed_records = match &observed.unit_allocation_records {
            ObservedUnitAllocationRecords::EveryRoleOfTheVersion(records) => records,
            ObservedUnitAllocationRecords::RewrittenRolesOnly(rewritten) => {
                let rewritten_in_the_model: BTreeSet<ModelUnitRole> = expected
                    .role_written_at
                    .iter()
                    .filter(|(_, written_at)| **written_at == expected.key.checkpoint_txg)
                    .map(|(role, _)| *role)
                    .chain(
                        expected
                            .data_unit_written_at
                            .contains(&expected.key.checkpoint_txg)
                            .then_some(ModelUnitRole::Data),
                    )
                    .collect();
                counts.rewritten_role_sets_compared += 1;
                if !rewritten_in_the_model.is_empty() {
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::AllocationGeneration,
                        format!(
                            "{:?} 这次重写了 {rewritten_in_the_model:?}：实现要交回这一版每个角色的分配记录",
                            expected.key
                        ),
                        format!("只交回了重写的角色 {rewritten:?}"),
                    ));
                }
                if rewritten_in_the_model == *rewritten {
                    return Ok(());
                }
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::AllocationGeneration,
                    format!(
                        "{:?} 这次重写（分配代等于这次 txg）的角色 {rewritten_in_the_model:?}",
                        expected.key
                    ),
                    format!("重写了 {rewritten:?}"),
                ));
            }
            ObservedUnitAllocationRecords::AllocationRecordTreeRootChangedByAZeroUnitPublish {
                what,
            } => {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::AllocationGeneration,
                    format!(
                        "{:?} 的分配记录就是同一次挂载里前一版那一份（零单元发布照抄分配记录树根指针）",
                        expected.key
                    ),
                    what.clone(),
                ));
            }
        };
        let ObservedAllocationRecordsOfEveryUnit {
            roles: observed_records,
            data_units: observed_data_units,
            data_units_of_the_files_other_than_the_first: observed_data_units_of_the_other_files,
        } = observed_records;
        let handed_in: BTreeSet<ModelUnitRole> =
            observed_records.iter().map(|(role, _)| *role).collect();
        if let Some(role) = expected
            .role_written_at
            .keys()
            .find(|role| !handed_in.contains(role))
        {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::AllocationGeneration,
                format!(
                    "{:?} 这一版有 {role:?}（{:?} 写的），实现要交回它的分配记录",
                    expected.key,
                    expected.role_written_at.get(role)
                ),
                format!("只交回了 {handed_in:?}"),
            ));
        }
        for (role, records) in observed_records {
            let written_at = match role {
                // 数据单元按单元号交（`data_units`），混在角色里交的说不上是哪一个。
                ModelUnitRole::Data => {
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::AllocationGeneration,
                        format!(
                            "{:?} 的数据单元按单元号逐个交（{} 个）",
                            expected.key,
                            expected.data_unit_written_at.len()
                        ),
                        format!("在角色里交了一个不带单元号的数据单元：{records:?}"),
                    ));
                }
                ModelUnitRole::ExtentRoot
                | ModelUnitRole::ExtentUpperNodeBelowTheRoot(_)
                | ModelUnitRole::InodeLeaf
                | ModelUnitRole::InodeLeafRightOfTheLeftmost(_)
                | ModelUnitRole::InodeRoot
                | ModelUnitRole::AllocationTree
                | ModelUnitRole::AccountingTree
                | ModelUnitRole::MappingTree
                | ModelUnitRole::TreeTable
                | ModelUnitRole::InstanceTable
                | ModelUnitRole::DeletedInodesRoot
                | ModelUnitRole::LoggedOperationsRoot
                | ModelUnitRole::TombstoneLeaf
                | ModelUnitRole::TombstoneRoot
                | ModelUnitRole::RetirementLeaf
                | ModelUnitRole::RetirementRoot => {
                    let Some(written_at) = expected.role_written_at.get(role) else {
                        return Err(ModelDisagreement::new(
                            ModelDisagreementAspect::AllocationGeneration,
                            format!("{:?} 这一版没有 {role:?} 这个角色", expected.key),
                            format!("实现交回了 {role:?} 的单元"),
                        ));
                    };
                    *written_at
                }
            };
            counts.allocation_records_compared += u64::try_from(records.len()).expect("条数");
            if !self.allocation_records_are_one_per_device_allocated_at(records, written_at) {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::AllocationGeneration,
                    format!(
                        "{:?} 的 {role:?}：每块盘一条、仍分配、分配代 {}（写它的那次发布）",
                        expected.key, written_at.0
                    ),
                    format!("{records:?}"),
                ));
            }
        }
        // 数据单元按单元号逐个比（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K4：截断只重写尾单元之后各单元的分配代不同）：
        // 交回的单元号恰好是 0 .. 模型这一版的单元数、各一项，单元 i 的分配代等于模型按单元号记的第 i 个。
        let mut data_units_handed_in: Vec<u64> = observed_data_units
            .iter()
            .map(|(index, _)| *index)
            .collect();
        data_units_handed_in.sort_unstable();
        let data_units_in_the_model: Vec<u64> =
            (0..u64::try_from(expected.data_unit_written_at.len()).expect("单元数")).collect();
        if data_units_handed_in != data_units_in_the_model {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::AllocationGeneration,
                format!(
                    "{:?} 这一版有 {} 个数据单元（单元号 0 起连着、各一项），实现要交回它们的分配记录",
                    expected.key,
                    expected.data_unit_written_at.len()
                ),
                format!("交回的单元号（排过序）{data_units_handed_in:?}"),
            ));
        }
        for (index, records) in observed_data_units {
            let written_at = *expected
                .data_unit_written_at
                .get(usize::try_from(*index).expect("单元号装得进 usize"))
                .expect("上面判过交回的单元号恰好是 0 .. 模型这一版的单元数");
            counts.allocation_records_compared += u64::try_from(records.len()).expect("条数");
            let data_unit_records_match =
                self.allocation_records_are_one_per_device_allocated_at(records, written_at);
            if !data_unit_records_match {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::AllocationGeneration,
                    format!(
                        "{:?} 的数据单元 {index}：每块盘一条、仍分配、分配代 {}（写它的那次发布）",
                        expected.key, written_at.0
                    ),
                    format!("{records:?}"),
                ));
            }
        }
        // 第一个文件之外的文件的数据单元按 (inode 号, 单元号) 逐个比（里程碑「删除与多次 COW」一·1）：交回的恰好是模型这一版每个有数据单元的文件的
        // 0 .. 单元数、各一项，单元 i 的分配代等于模型按单元号记的第 i 个（整份写那次全是它，改 size 重写的尾单元是那一次的，判决 m4-multi-inode-size-r1
        // 第二节 S3）。
        let mut other_data_units_handed_in: Vec<(u64, u64)> =
            observed_data_units_of_the_other_files
                .iter()
                .map(|(inode_and_index, _)| *inode_and_index)
                .collect();
        other_data_units_handed_in.sort_unstable();
        let other_data_units_in_the_model: Vec<(u64, u64)> = expected
            .inodes_other_than_the_first
            .iter()
            .flat_map(|(number, inode)| {
                (0..inode.data_units_present()).map(move |index| (*number, index))
            })
            .collect();
        if other_data_units_handed_in != other_data_units_in_the_model {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::AllocationGeneration,
                format!(
                    "{:?} 第一个文件之外的文件的数据单元 (inode 号, 单元号) {other_data_units_in_the_model:?}，实现要交回它们的分配记录",
                    expected.key
                ),
                format!("交回的（排过序）{other_data_units_handed_in:?}"),
            ));
        }
        for ((number, index), records) in observed_data_units_of_the_other_files {
            let written_at = expected
                .inodes_other_than_the_first
                .get(number)
                .and_then(|inode| {
                    inode
                        .data_unit_written_at
                        .get(usize::try_from(*index).expect("这个文件的单元号装得进 usize"))
                        .copied()
                })
                .expect(
                    "上面判过交回的 (inode 号, 单元号) 恰好是模型这一版有数据单元的文件的那几个",
                );
            counts.allocation_records_compared += u64::try_from(records.len()).expect("条数");
            if !self.allocation_records_are_one_per_device_allocated_at(records, written_at) {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::AllocationGeneration,
                    format!(
                        "{:?} 的 inode {number} 数据单元 {index}：每块盘一条、仍分配、分配代 {}（整份写它的那次发布）",
                        expected.key, written_at.0
                    ),
                    format!("{records:?}"),
                ));
            }
        }
        Ok(())
    }

    /// 一个单元的那几条分配记录：每块盘恰好一条、仍分配着、分配代等于写它的那次发布的 txg（D3（空间分配） 已定项 3 / 7）。
    fn allocation_records_are_one_per_device_allocated_at(
        &self,
        records: &[ObservedAllocationRecord],
        written_at: ModelCheckpointTxg,
    ) -> bool {
        let devices_expected: BTreeSet<ModelDeviceIdentity> =
            self.geometry.devices.iter().copied().collect();
        let devices_seen: BTreeSet<ModelDeviceIdentity> =
            records.iter().map(|record| record.device).collect();
        records.len() == self.geometry.devices.len()
            && devices_seen == devices_expected
            && records
                .iter()
                .all(|record| !record.is_released && record.generation == written_at)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 单元的分配代是写它的那次发布的 txg（D3（空间分配） 已定项 3 / 7）：写行那次（txg 4）照抄的数据单元分配代 3，写成 4 就对不上；
    /// 这次重写的实例表分配代 4。
    #[test]
    fn carried_unit_keeps_the_generation_of_the_publish_that_wrote_it() {
        let mut model = two_device_model();
        model.acquire_and_warm_up_in_the_make_filesystem_process();
        let first = model.answer_publish_first_file(&[3]).expect("会话开着");
        succeed(&mut model, &first);
        model.close_session();
        let mount = model.answer_mount_writable();
        let row_publish = &mount.expected_roots[0];
        let records = |generation: u64| {
            [ModelDeviceIdentity(0), ModelDeviceIdentity(1)]
                .into_iter()
                .map(|device| ObservedAllocationRecord {
                    device,
                    generation: ModelCheckpointTxg(generation),
                    is_released: false,
                })
                .collect::<Vec<_>>()
        };
        let observed = |data_generation: u64| ObservedRoot {
            key: row_publish.key,
            journal_counter: row_publish.journal_counter,
            rollback_floor: row_publish.rollback_floor,
            file: ObservedFile::Decoded(vec![3]),
            instance_table: ObservedInstanceTable::Rows(
                row_publish.instance_table_rows.as_ref().clone(),
            ),
            unit_allocation_records: ObservedUnitAllocationRecords::EveryRoleOfTheVersion(
                ObservedAllocationRecordsOfEveryUnit {
                    roles: row_publish
                        .role_written_at
                        .iter()
                        .map(|(role, written_at)| (*role, records(written_at.0)))
                        .collect(),
                    data_units: (0_u64..)
                        .zip(row_publish.data_unit_written_at.iter())
                        .map(|(index, _)| (index, records(data_generation)))
                        .collect(),
                    data_units_of_the_files_other_than_the_first: Vec::new(),
                },
            ),
            snapshot_state: snapshot_state_as_the_model_wrote(row_publish),
            logged_compaction_intent: row_publish.logged_compaction_intent,
            first_file_record_truncation_field: truncation_field_as_the_model_wrote(row_publish),
            inodes_other_than_the_first: BTreeMap::new(),
            truncation_fields_of_the_inodes_other_than_the_first: BTreeMap::new(),
        };
        let mut counts = ModelJudgementCounts::default();
        assert_eq!(
            model.judge_allocation_generations(row_publish, &observed(3), &mut counts),
            Ok(())
        );
        assert_eq!(
            counts.allocation_records_compared,
            2 * 9,
            "写行那一版九个角色，每个角色每块盘一条"
        );
        let wrong = model
            .judge_allocation_generations(row_publish, &observed(4), &mut counts)
            .expect_err("照抄的数据单元分配代写成了这次的 txg");
        assert_eq!(wrong.aspect, ModelDisagreementAspect::AllocationGeneration);
    }
}
