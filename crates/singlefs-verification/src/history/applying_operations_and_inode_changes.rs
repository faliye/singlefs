//! 把一步操作分派给各自的施加函数，连同施加删文件、截断、建 inode 与按 inode、按偏移写（逐版比模型）。

use super::*;

/// unlink 一个号（`HistoryOperation::Unlink`）：经挂着的会话发（意图那次 + 走空，D8（核心索引结构） 已定项 16）。删哪个号按号选
/// （[`inode_the_unlink_names`]，判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M4：第一个文件、带数据的别的文件、
/// size 0 的别的文件都抽得到；树里只有第一个文件时就是它）。意图那次带这一步的写入时间（与覆盖写、建 inode 同一个取法 Q-F 乙），
/// 各批照抄。前提：现行那一版带文件、第一个文件那条记录在或已删完而还有带数据单元的别的文件活着（[`some_live_inode_other_than_the_first_holds_data_units`]；
/// 第一个文件删完之后也按号删别的文件，代码轮判决 `research/prompts/m3-close-w5-code-r1-main-verification.md` 第三节第 3 条；别的几组比重不写别的文件的
/// 数据，生成的历史与执行照旧）、inode 树只有一片叶容器（[`MissingPrecondition::InodeTreeSpansMoreThanOneLeafContainer`]）。删除中的那一版照调：会话先走空、再删选中的号；
/// 选中的是第一个文件而它正在删（树里没有别的活着的号）时把它走成有树而文件集合为空的一版、再拒成 `UserChangeRefused::NoFileVersionToChange`，
/// 走空的那几步逐版交模型比（代码轮 U11）。
fn apply_unlink(pool: &mut HistoryPool, write_time_seconds: u64, step_index: usize) -> AppliedStep {
    let HistoryPool {
        devices,
        session: session_slot,
        model,
        stream,
        copies_quarantined_in_this_step,
        ..
    } = pool;
    let Some(open_session) = session_slot.as_ref() else {
        return AppliedStep::not_applicable(MissingPrecondition::NoWritableSession);
    };
    let Some(current_file_version) = open_session.current.file_version() else {
        return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutFile);
    };
    // 第一个文件已删完的一版上，只在还有删之前就写过数据的别的文件活着时按号删（代码轮判决
    // `research/prompts/m3-close-w5-code-r1-main-verification.md` 第三节第 3 条）：带数据的别的文件只由按 inode 号写那一组比重写出，
    // 别的几组比重在删完的一版上只有删完之后建的 size 0 的 inode，这一形照旧不调，它们生成的历史执行照旧。
    if current_file_version.inode_record.is_none()
        && !some_live_inode_other_than_the_first_holds_data_units(current_file_version)
    {
        return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutTheFirstFile);
    }
    let records_before = open_session.allocator.records().to_vec();
    if current_file_version.inode_leaf_containers.len() > 1 {
        return AppliedStep::not_applicable(
            MissingPrecondition::InodeTreeSpansMoreThanOneLeafContainer,
        );
    }
    let inode = inode_the_unlink_names(
        current_file_version,
        u64::try_from(step_index).expect("步号装得进 u64"),
    );
    let stream_length_before = stream.operation_count();
    let WritableSession {
        allocator,
        current,
        instance,
        shadow_ledger,
        parameters_and_device_table,
        publishes_in_this_mount,
        carried_to_zero_unit_publishes: _,
        compaction,
    } = session_slot.take().expect("上面判过这个进程的会话开着");
    let mut mounted_session = MountedSession {
        allocator,
        current,
        instance,
        shadow_ledger,
        parameters_and_device_table,
        compaction,
    };
    let change_result = mounted_session.publish_user_change(
        devices,
        UserChange::Unlink {
            inode: singlefs_core::address::InodeNumber(inode),
            write_time_seconds,
        },
    );
    copies_quarantined_in_this_step.extend(copies_quarantined_by_a_user_change(&change_result));
    let session_after_the_change = session_slot.insert(WritableSession {
        allocator: mounted_session.allocator,
        current: mounted_session.current,
        instance: mounted_session.instance,
        shadow_ledger: mounted_session.shadow_ledger,
        parameters_and_device_table: mounted_session.parameters_and_device_table,
        compaction: mounted_session.compaction,
        publishes_in_this_mount,
        // 零单元发布只接在树表 0 条的一版后面，unlink 之后这一版有树。
        carried_to_zero_unit_publishes: None,
    });
    settle_user_change_comparing_every_version_published(
        session_after_the_change,
        model,
        &|ideal_model: &IdealModel| ideal_model.answer_unlink_intent_of_an_inode(inode),
        &records_before,
        &change_result,
        stream,
        stream_length_before,
    )
}

/// unlink 删哪个号（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M4「随机历史 unlink 按号选」）：这一步的步号当选择子
/// （[`HistoryOperation::Unlink`] 不带参数：给它加参数要多抽一个随机数，别的几组比重生成的历史就变了；步号在生成时不知道那一刻树里有哪几个号，
/// 收缩删掉前面几步之后照样有意义）。在 `version` 里活着的号（nlink 不是 0，删除中的那一条会话先走掉）里取：步号模 3 是 0 取第一个文件，
/// 是 1 取带数据单元的别的文件（再按步号一半取其中最大的号——多半罩在上段第 0 片叶之外，删它上段降层——一半按步号取模），是 2 取没有数据单元的
/// 别的文件；那一类一个都没有时取第一个文件（树里只有第一个文件的历史因此恒删它，与按号选之前相同），第一个文件也不活着时取最小的活着的号，
/// 一个活着的号都没有时取第一个文件（会话走空之后拒）。
fn inode_the_unlink_names(version: &TransactionOutput, step_index: u64) -> u64 {
    let first_file = singlefs_core::transaction::FIRST_INODE_NUMBER;
    let live_inodes = live_inode_numbers(version);
    let inodes_with_an_entry = version.inodes_with_an_extent_upper_leaf_entry();
    let (others_holding_data, others_without_data): (Vec<u64>, Vec<u64>) = live_inodes
        .iter()
        .copied()
        .filter(|number| *number != first_file)
        .partition(|number| inodes_with_an_entry.contains(number));
    let nth_of = |numbers: &[u64], selector: u64| {
        let count = u64::try_from(numbers.len())
            .ok()
            .filter(|count| *count > 0)?;
        numbers
            .get(usize::try_from(selector % count).expect("小于条数"))
            .copied()
    };
    let selector_within_the_category = step_index / 3;
    let chosen = match step_index % 3 {
        0 => None,
        1 if selector_within_the_category.is_multiple_of(2) => others_holding_data.last().copied(),
        1 => nth_of(&others_holding_data, selector_within_the_category / 2),
        _ => nth_of(&others_without_data, selector_within_the_category),
    };
    chosen
        .or_else(|| live_inodes.contains(&first_file).then_some(first_file))
        .or_else(|| live_inodes.first().copied())
        .unwrap_or(first_file)
}

/// 模型对一次经会话的用户改动本身的答法（走空的那几步另按 [`IdealModel::answer_next_step_of_the_walk`] 答）。
type ModelAnswerOfAUserChange = Box<dyn Fn(&IdealModel) -> Result<ModelAnswer, ModelDisagreement>>;

/// 改一个 inode 的 size（`HistoryOperation::Truncate`，判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K8）：经挂着的会话发
/// （`UserChange::Truncate`）。一次调用 = 先走空 + 改 size 那一次 + 缩短时之后的全部批；会话推的每一串抬 F 与发成的每一版交模型比（走空的一步与
/// 截断的各批按「走空的一步」，改 size 那一次按 [`IdealModel::answer_truncation`]），照 unlink 的比法。
/// 点名谁：现行那一版有第一个文件那条记录（活的、删除中的、截断途中的）时改第一个文件，目标按那条记录的 size 与这一版的数据单元数取
/// （[`TruncationChoice::new_size_in_bytes`]；删除中的那一版会话先走空、再拒成 `NoFileVersionToChange`），树里另有别的 inode 时选择子最高位是 1 的
/// 那一半改其中一个（带数据单元的先挑，目标按它自己的 size 与单元数取，判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S5）；没有而树里有别的 inode（删完之后建过）
/// 时点名按选择子取的那一个，目标取 [1, 3 × 净荷]（[`IdealModel::answer_size_change_of_an_inode_other_than_the_first`]；任意活 inode 都能缩短与变长，
/// 判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S6）；
/// 都没有（有树而文件集合为空）照样点名第一个文件，会话拒成 `NoFileVersionToChange`。写入时间取这一步的（同覆盖写、unlink）。
/// 前提：会话开着、现行那一版有树。
fn apply_truncate(
    pool: &mut HistoryPool,
    choice: TruncationChoice,
    step_index: usize,
    write_time_seconds: u64,
) -> AppliedStep {
    let HistoryPool {
        devices,
        session: session_slot,
        model,
        stream,
        sizes_requested_by_the_truncations,
        inodes_named_by_the_truncations,
        copies_quarantined_in_this_step,
        ..
    } = pool;
    let Some(open_session) = session_slot.as_ref() else {
        return AppliedStep::not_applicable(MissingPrecondition::NoWritableSession);
    };
    let version_of_trees = match &open_session.current {
        PoolVersion::WithoutFile(_) => {
            return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutFile);
        }
        PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => version,
    };
    let other_inodes: Vec<singlefs_core::records::InodeRecord> = version_of_trees
        .inode_leaf_containers
        .iter()
        .flat_map(|container| container.contents.records.iter())
        .filter(|record| record.inode != singlefs_core::transaction::FIRST_INODE_NUMBER)
        .copied()
        .collect();
    let (inode, new_size, answer_of_the_change): (u64, u64, ModelAnswerOfAUserChange) =
        match (version_of_trees.inode_record, other_inodes.as_slice()) {
            // 第一个文件活着、树里还有别的 inode（只有在第一个文件旁边建过 inode 的那几组比重造得出这一形）：选择子最高位是 1 的那一半点名别的 inode
            // （按选择子的高 32 位取），目标按它自己的 size 与单元数取类（判决 m4-multi-inode-size-r1 第二节 S5「随机历史点名别的 inode 截断、变长」）。
            (Some(_), [_, ..]) if choice.selector >> 63 == 1 => {
                // 带数据单元的别的 inode 先挑（这一组要罩的是它们的截断与变长）；一个都没有时在全部别的 inode 里挑。
                let holding_data_units: Vec<singlefs_core::records::InodeRecord> = other_inodes
                    .iter()
                    .filter(|record| {
                        version_of_trees
                            .file_other_than_the_first(singlefs_core::address::InodeNumber(
                                record.inode,
                            ))
                            .is_some_and(|file| !file.data_pointers.is_empty())
                    })
                    .copied()
                    .collect();
                let candidates = if holding_data_units.is_empty() {
                    other_inodes.as_slice()
                } else {
                    holding_data_units.as_slice()
                };
                let index = usize::try_from(
                    (choice.selector >> 32)
                        % u64::try_from(candidates.len()).expect("条数装得进 u64"),
                )
                .expect("小于条数");
                let record = candidates[index];
                let data_units_present = u64::try_from(
                    version_of_trees
                        .file_other_than_the_first(singlefs_core::address::InodeNumber(
                            record.inode,
                        ))
                        .map_or(0, |file| file.data_pointers.len()),
                )
                .expect("单元数装得进 u64");
                let new_size = choice.new_size_in_bytes(record.size, data_units_present);
                let changed_inode = record.inode;
                (
                    changed_inode,
                    new_size,
                    Box::new(move |ideal_model: &IdealModel| {
                        ideal_model.answer_size_change_of_an_inode_other_than_the_first(
                            changed_inode,
                            new_size,
                        )
                    }),
                )
            }
            (Some(record), _) => {
                let data_units_present =
                    u64::try_from(version_of_trees.data_pointers.len()).expect("单元数装得进 u64");
                let new_size = choice.new_size_in_bytes(record.size, data_units_present);
                (
                    singlefs_core::transaction::FIRST_INODE_NUMBER,
                    new_size,
                    Box::new(move |ideal_model: &IdealModel| {
                        ideal_model.answer_truncation(new_size)
                    }),
                )
            }
            (None, []) => {
                let new_size = choice.new_size_in_bytes(0, 0);
                (
                    singlefs_core::transaction::FIRST_INODE_NUMBER,
                    new_size,
                    Box::new(move |ideal_model: &IdealModel| {
                        ideal_model.answer_truncation(new_size)
                    }),
                )
            }
            (None, [_, ..]) => {
                let index = usize::try_from(
                    choice.selector % u64::try_from(other_inodes.len()).expect("条数装得进 u64"),
                )
                .expect("小于条数");
                let record = other_inodes[index];
                let new_size =
                    new_size_of_an_inode_created_after_the_first_file_was_deleted(choice.selector);
                (
                    record.inode,
                    new_size,
                    Box::new(move |ideal_model: &IdealModel| {
                        ideal_model.answer_size_change_of_an_inode_other_than_the_first(
                            record.inode,
                            new_size,
                        )
                    }),
                )
            }
        };
    sizes_requested_by_the_truncations.insert(step_index, new_size);
    inodes_named_by_the_truncations.insert(step_index, inode);
    let records_before = open_session.allocator.records().to_vec();
    let stream_length_before = stream.operation_count();
    let WritableSession {
        allocator,
        current,
        instance,
        shadow_ledger,
        parameters_and_device_table,
        publishes_in_this_mount,
        carried_to_zero_unit_publishes: _,
        compaction,
    } = session_slot.take().expect("上面判过这个进程的会话开着");
    let mut mounted_session = MountedSession {
        allocator,
        current,
        instance,
        shadow_ledger,
        parameters_and_device_table,
        compaction,
    };
    let change_result = mounted_session.publish_user_change(
        devices,
        UserChange::Truncate {
            inode: singlefs_core::address::InodeNumber(inode),
            new_size,
            write_time_seconds,
        },
    );
    copies_quarantined_in_this_step.extend(copies_quarantined_by_a_user_change(&change_result));
    let session_after_the_change = session_slot.insert(WritableSession {
        allocator: mounted_session.allocator,
        current: mounted_session.current,
        instance: mounted_session.instance,
        shadow_ledger: mounted_session.shadow_ledger,
        parameters_and_device_table: mounted_session.parameters_and_device_table,
        compaction: mounted_session.compaction,
        publishes_in_this_mount,
        // 零单元发布只接在树表 0 条的一版后面，改 size 只接在有树的一版后面。
        carried_to_zero_unit_publishes: None,
    });
    settle_user_change_comparing_every_version_published(
        session_after_the_change,
        model,
        answer_of_the_change.as_ref(),
        &records_before,
        &change_result,
        stream,
        stream_length_before,
    )
}

/// 经会话的 unlink（或建 inode）返回之后：会话推的每一串抬 F 与推的抬 F 之外发成的每一版按 txg 排好，逐样交模型比——抬 F 照模型的抬 F，
/// 发成的一版在模型的现行版本有删除中的文件时照「走空的一步」、否则照这次改动本身（`answer_of_the_change`：意图那次、建 inode 那次）；
/// 分配代逐次比（上一版是前一样落盘的）。发成的改动之后另发的那一批与为它推的抬 F 不算这次改动本身，
/// 最后接着比（[`settle_what_the_session_published_for_the_relocation_batch`]）。
/// 被拒的比理由：被拒的是模型此刻的下一次发布（有删除中的文件就是走空的一步，否则是这次改动本身），「拒之前写没写盘」只看那几串抬 F
/// 与发成的那几版之外多出来的写。
pub(super) fn settle_user_change_comparing_every_version_published(
    session: &mut WritableSession,
    model: &mut IdealModel,
    answer_of_the_change: &dyn Fn(&IdealModel) -> Result<ModelAnswer, ModelDisagreement>,
    records_before: &[AllocationRecord],
    change_result: &Result<UserChangePublished, UserChangeRefused>,
    stream: &SharedStream,
    stream_length_before: usize,
) -> AppliedStep {
    let (floor_raises, versions_published) =
        floor_raises_and_versions_published_of_a_user_change(change_result);
    let versions_of_the_change = match change_result {
        Ok(published) => versions_of_the_change_without_the_relocation_batch(published),
        Err(_) => versions_published,
    };
    let published_in_txg_order =
        floor_raises_and_versions_in_txg_order(floor_raises, versions_of_the_change);
    let mut verdict = ModelVerdict::default();
    let mut harness_judgement = None;
    let mut records_of_the_previous_publish: &[AllocationRecord] = records_before;
    // 这次改动自己（意图那次、建 inode 那次、覆盖写）发成之前，会话先走空的那几步与走空时为空间推的抬 F（N3）。
    let mut the_change_itself_was_published = false;
    let mut steps_walked_before_the_change = 0_u64;
    let mut floor_raises_while_walking_before_the_change = 0_u64;
    // 这次改动自己那一次发布起写的根（`AppliedEffect::Published::roots_carrying_the_change`）。
    let mut roots_carrying_the_change = 0_usize;
    // 迭代上界是这次落盘的样数；跨轮带的是上一样落盘之后的分配记录与合起来的判定；模型对不上之后不再问模型（分配代照判）。
    for published in &published_in_txg_order {
        let outputs: Vec<&singlefs_core::transaction::TransactionOutput> = match published {
            FloorRaiseOrVersion::FloorRaise(raised) => {
                if !the_change_itself_was_published && model.current_version_has_a_step_to_walk() {
                    floor_raises_while_walking_before_the_change += 1;
                }
                if verdict.disagreement.is_none() {
                    let raise_verdict = judge_one_floor_raise_pushed_by_the_session(model, raised);
                    verdict.counts.add(&raise_verdict.counts);
                    verdict.disagreement = raise_verdict.disagreement;
                }
                raised.publishes.iter().collect()
            }
            FloorRaiseOrVersion::Version(version) => {
                // 走空的一步：删除中的文件的一批，或销毁中的快照的一批（`deleted_inodes` 先走空，判决 S4 删快照照 unlink 的过程做）。
                let walking = model.current_version_has_a_step_to_walk();
                if walking && !the_change_itself_was_published {
                    steps_walked_before_the_change += 1;
                }
                if !walking {
                    the_change_itself_was_published = true;
                }
                if verdict.disagreement.is_none() {
                    let answer = if walking {
                        model.answer_next_step_of_the_walk()
                    } else {
                        answer_of_the_change(model)
                    };
                    let version_verdict = judge_by_model(
                        model,
                        answer,
                        &ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                            roots: vec![observed_root_of_file_version(version)],
                            reported_ceiling: None,
                        }),
                    );
                    verdict.counts.add(&version_verdict.counts);
                    verdict.disagreement = version_verdict.disagreement;
                }
                vec![*version]
            }
        };
        if the_change_itself_was_published {
            roots_carrying_the_change += outputs.len();
        }
        for output in outputs {
            let publish_txg = output.root.checkpoint_txg;
            harness_judgement = harness_judgement.or_else(|| {
                allocation_generation_judgement(
                    records_of_the_previous_publish,
                    &output.allocation_records,
                    publish_txg,
                    publish_txg,
                    &output.quarantined_after_release_checksum_mismatch,
                )
            });
            records_of_the_previous_publish = &output.allocation_records;
            session.publishes_in_this_mount += 1;
        }
    }
    if let Ok(published) = change_result {
        roots_carrying_the_change += settle_what_the_session_published_for_the_relocation_batch(
            session,
            model,
            published,
            records_of_the_previous_publish,
            &mut verdict,
            &mut harness_judgement,
        );
    }
    if verdict.disagreement.is_none() {
        if steps_walked_before_the_change >= 2 {
            verdict
                .counts
                .walks_of_several_steps_before_a_user_change_matched += 1;
        }
        if floor_raises_while_walking_before_the_change > 0 {
            verdict
                .counts
                .walks_before_a_user_change_refused_for_space_matched += 1;
        }
    }
    let reuse = RecordReuse::between(records_before, session.allocator.records());
    let refusal = match change_result {
        Ok(_) => {
            return AppliedStep {
                outcome: StepOutcome::Applied(AppliedEffect::Published {
                    reuse,
                    floor_raises_pushed_by_the_session: floor_raises.len(),
                    roots_carrying_the_change,
                }),
                harness_judgement,
                model_verdict: Some(verdict),
            };
        }
        Err(refused) => refused,
    };
    let (member, reason) = member_and_reason_of_a_refused_user_change(refusal);
    if verdict.disagreement.is_none() {
        let answer = if model.current_version_has_a_step_to_walk() {
            model.answer_next_step_of_the_walk()
        } else {
            answer_of_the_change(model)
        };
        let writes_of_the_versions_published: u64 = versions_published
            .iter()
            .map(|version| version.writes.total().write_calls)
            .sum();
        let wrote_anything_besides_what_was_published =
            if floor_raises.is_empty() && versions_published.is_empty() {
                stream.operation_count() != stream_length_before
            } else {
                writes_in_the_stream_since(stream, stream_length_before)
                    > writes_of_the_floor_raises(floor_raises) + writes_of_the_versions_published
            };
        let refusal_verdict = judge_by_model(
            model,
            answer,
            &ObservedOutcome::Refused {
                member: member.clone(),
                reason,
                publishes_completed: 0,
                wrote_anything: wrote_anything_besides_what_was_published,
                reported_ceiling: None,
                reported_root_ring_slot_still_bad_after_one_reread: None,
            },
        );
        verdict.counts.add(&refusal_verdict.counts);
        verdict.disagreement = refusal_verdict.disagreement;
    }
    AppliedStep {
        outcome: StepOutcome::Refused { member },
        harness_judgement,
        model_verdict: Some(verdict),
    }
}

/// 建一个 size 0 的 inode（`HistoryOperation::CreateInode`）：经挂着的会话发（`UserChange::NewInodes`，count 1）。
/// 前提：第一个文件不再活着——现行那一版有树而文件集合为空，或是在它上面建过 inode 的一版（带文件、没有第一个文件那条记录、队列空），
/// 或第一个文件删除中的那一版（会话先走空，代码轮 U11）。
fn apply_create_inode(pool: &mut HistoryPool, write_time_seconds: u64) -> AppliedStep {
    let HistoryPool {
        devices,
        session: session_slot,
        model,
        stream,
        copies_quarantined_in_this_step,
        ..
    } = pool;
    let Some(open_session) = session_slot.as_ref() else {
        return AppliedStep::not_applicable(MissingPrecondition::NoWritableSession);
    };
    match &open_session.current {
        PoolVersion::WithoutFile(_) => {
            return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutFile);
        }
        PoolVersion::WithFile(version) => {
            // 删除中的那一版（队列不空）照调：会话先走空、把它走成删完的一版再建（代码轮 U11）。
            if version.inode_record.is_some() && version.deleted_inodes.is_empty() {
                return AppliedStep::not_applicable(
                    MissingPrecondition::CurrentVersionStillHasTheFirstFile,
                );
            }
        }
        PoolVersion::WithTreesAndNoFile(_) => {}
    }
    let records_before = open_session.allocator.records().to_vec();
    let stream_length_before = stream.operation_count();
    let WritableSession {
        allocator,
        current,
        instance,
        shadow_ledger,
        parameters_and_device_table,
        publishes_in_this_mount,
        carried_to_zero_unit_publishes: _,
        compaction,
    } = session_slot.take().expect("上面判过这个进程的会话开着");
    let mut mounted_session = MountedSession {
        allocator,
        current,
        instance,
        shadow_ledger,
        parameters_and_device_table,
        compaction,
    };
    let change_result = mounted_session.publish_user_change(
        devices,
        UserChange::NewInodes {
            count: 1,
            write_time_seconds,
        },
    );
    copies_quarantined_in_this_step.extend(copies_quarantined_by_a_user_change(&change_result));
    let session_after_the_change = session_slot.insert(WritableSession {
        allocator: mounted_session.allocator,
        current: mounted_session.current,
        instance: mounted_session.instance,
        shadow_ledger: mounted_session.shadow_ledger,
        parameters_and_device_table: mounted_session.parameters_and_device_table,
        compaction: mounted_session.compaction,
        publishes_in_this_mount,
        // 零单元发布只接在树表 0 条的一版后面，建 inode 之后这一版有树。
        carried_to_zero_unit_publishes: None,
    });
    settle_user_change_comparing_every_version_published(
        session_after_the_change,
        model,
        &IdealModel::answer_create_inode,
        &records_before,
        &change_result,
        stream,
        stream_length_before,
    )
}

/// 一版 inode 树里活着的记录（nlink 不是 0）的号，升序：删除中的那一条（unlink 的意图之后、最后一批之前）不算，会话改动之前先把它走空。
fn live_inode_numbers(version: &TransactionOutput) -> Vec<u64> {
    let mut numbers: Vec<u64> = version
        .inode_leaf_containers
        .iter()
        .flat_map(|container| container.contents.records.iter())
        .filter(|record| record.nlink != NLINK_OF_AN_INODE_BEING_DELETED)
        .map(|record| record.inode)
        .collect();
    numbers.sort_unstable();
    numbers
}

/// 一版里有没有第一个文件之外、活着（nlink 不是 0）而有数据单元的 inode（[`apply_unlink`] 在第一个文件已删完的一版上按它判调不调）。
fn some_live_inode_other_than_the_first_holds_data_units(version: &TransactionOutput) -> bool {
    let inodes_with_an_entry = version.inodes_with_an_extent_upper_leaf_entry();
    live_inode_numbers(version).iter().any(|number| {
        *number != singlefs_core::transaction::FIRST_INODE_NUMBER
            && inodes_with_an_entry.contains(number)
    })
}

/// 一次建几个 size 0 的 inode（`HistoryOperation::CreateInodes`）：经挂着的会话发（`UserChange::NewInodes`），第一个文件活着或已删完的一版上都调
/// （删除中的那一版会话先走空、把它走成删完的一版再建）。建几个压到最右那片叶容器装得下的条数（[`InodeCountChoice::count_fitting_the_rightmost_leaf_container`]，
/// 按活着的记录数算：删除中的那一条会话先走掉）。会话交回的每一版交模型比（走空的一步按「走空的一步」，建 inode 那次按 [`IdealModel::answer_create_inodes`]）。
/// 前提：会话开着、现行那一版有树。
pub(super) fn apply_create_inodes(
    pool: &mut HistoryPool,
    choice: InodeCountChoice,
    write_time_seconds: u64,
) -> AppliedStep {
    let HistoryPool {
        devices,
        session: session_slot,
        model,
        stream,
        copies_quarantined_in_this_step,
        ..
    } = pool;
    let Some(open_session) = session_slot.as_ref() else {
        return AppliedStep::not_applicable(MissingPrecondition::NoWritableSession);
    };
    let version_of_trees = match &open_session.current {
        PoolVersion::WithoutFile(_) => {
            return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutFile);
        }
        PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => version,
    };
    let live_inode_records =
        u64::try_from(live_inode_numbers(version_of_trees).len()).expect("条数装得进 u64");
    let count = choice.count_fitting_the_rightmost_leaf_container(live_inode_records);
    let records_before = open_session.allocator.records().to_vec();
    let stream_length_before = stream.operation_count();
    let WritableSession {
        allocator,
        current,
        instance,
        shadow_ledger,
        parameters_and_device_table,
        publishes_in_this_mount,
        carried_to_zero_unit_publishes: _,
        compaction,
    } = session_slot.take().expect("上面判过这个进程的会话开着");
    let mut mounted_session = MountedSession {
        allocator,
        current,
        instance,
        shadow_ledger,
        parameters_and_device_table,
        compaction,
    };
    let change_result = mounted_session.publish_user_change(
        devices,
        UserChange::NewInodes {
            count,
            write_time_seconds,
        },
    );
    copies_quarantined_in_this_step.extend(copies_quarantined_by_a_user_change(&change_result));
    let session_after_the_change = session_slot.insert(WritableSession {
        allocator: mounted_session.allocator,
        current: mounted_session.current,
        instance: mounted_session.instance,
        shadow_ledger: mounted_session.shadow_ledger,
        parameters_and_device_table: mounted_session.parameters_and_device_table,
        compaction: mounted_session.compaction,
        publishes_in_this_mount,
        // 零单元发布只接在树表 0 条的一版后面，建 inode 之后这一版有树。
        carried_to_zero_unit_publishes: None,
    });
    settle_user_change_comparing_every_version_published(
        session_after_the_change,
        model,
        &|ideal_model: &IdealModel| ideal_model.answer_create_inodes(count),
        &records_before,
        &change_result,
        stream,
        stream_length_before,
    )
}

/// 按 inode 号整份写一个文件（`HistoryOperation::WriteOfAnInode`，里程碑「删除与多次 COW」一·1）：经挂着的会话发
/// （`UserChange::OverwriteOfAnInode` 或 `UserChange::SequentialWriteOfAnInode`，按内容长度分，同覆盖写）。号在现行那一版树里活着的号里取
/// （[`WriteOfAnInodeChoice::inode_among`]，第一个文件活着时连 inode 1）；第一个文件已删完、之后建过 inode 的一版上照调（必拒，K-U5）。
/// 会话交回的每一版交模型比（[`IdealModel::answer_overwrite_of_an_inode`]、[`IdealModel::answer_sequential_write_of_an_inode`]）。
/// 前提：会话开着、现行那一版树里有活着的号。
fn apply_write_of_an_inode(
    pool: &mut HistoryPool,
    choice: WriteOfAnInodeChoice,
    write_time_seconds: u64,
) -> AppliedStep {
    let HistoryPool {
        devices,
        session: session_slot,
        model,
        stream,
        copies_quarantined_in_this_step,
        ..
    } = pool;
    let Some(open_session) = session_slot.as_ref() else {
        return AppliedStep::not_applicable(MissingPrecondition::NoWritableSession);
    };
    let version_of_trees = match &open_session.current {
        PoolVersion::WithoutFile(_) => {
            return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutFile);
        }
        PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => version,
    };
    let Some(inode) = choice.inode_among(&live_inode_numbers(version_of_trees)) else {
        return AppliedStep::not_applicable(MissingPrecondition::NoInodeToWriteInTheCurrentVersion);
    };
    let content = choice.content.bytes();
    let records_before = open_session.allocator.records().to_vec();
    let stream_length_before = stream.operation_count();
    let WritableSession {
        allocator,
        current,
        instance,
        shadow_ledger,
        parameters_and_device_table,
        publishes_in_this_mount,
        carried_to_zero_unit_publishes: _,
        compaction,
    } = session_slot.take().expect("上面判过这个进程的会话开着");
    let mut mounted_session = MountedSession {
        allocator,
        current,
        instance,
        shadow_ledger,
        parameters_and_device_table,
        compaction,
    };
    let entry = choice.content.length.whole_file_write_entry();
    let inode_number = singlefs_core::address::InodeNumber(inode);
    let change_result = mounted_session.publish_user_change(
        devices,
        match entry {
            WholeFileWriteEntry::Overwrite => UserChange::OverwriteOfAnInode {
                inode: inode_number,
                content: &content,
                write_time_seconds,
            },
            WholeFileWriteEntry::SequentialWrite => UserChange::SequentialWriteOfAnInode {
                inode: inode_number,
                content: &content,
                write_time_seconds,
            },
        },
    );
    copies_quarantined_in_this_step.extend(copies_quarantined_by_a_user_change(&change_result));
    let session_after_the_change = session_slot.insert(WritableSession {
        allocator: mounted_session.allocator,
        current: mounted_session.current,
        instance: mounted_session.instance,
        shadow_ledger: mounted_session.shadow_ledger,
        parameters_and_device_table: mounted_session.parameters_and_device_table,
        compaction: mounted_session.compaction,
        publishes_in_this_mount,
        // 零单元发布只接在树表 0 条的一版后面，按 inode 号写只接在有树的一版后面。
        carried_to_zero_unit_publishes: None,
    });
    let answer_of_the_change = |ideal_model: &IdealModel| match entry {
        WholeFileWriteEntry::Overwrite => ideal_model.answer_overwrite_of_an_inode(inode, &content),
        WholeFileWriteEntry::SequentialWrite => {
            ideal_model.answer_sequential_write_of_an_inode(inode, &content)
        }
    };
    settle_user_change_comparing_every_version_published(
        session_after_the_change,
        model,
        &answer_of_the_change,
        &records_before,
        &change_result,
        stream,
        stream_length_before,
    )
}

/// 按偏移写一个活 inode（`HistoryOperation::WriteAtOffset`，判决 `research/prompts/m4-partial-write-r1-main-verification.md` 第二节）：经挂着的会话发
/// （`UserChange::WriteAtOffset`）。号在现行那一版树里活着的号里取（[`PartialWriteChoice::inode_among`]），偏移与长度按那个文件此刻的 size 与单元数取
/// （[`PartialWriteChoice::offset_and_length_in_bytes`]；单元数取它与 size 罩的单元数里小的那个——截断途中 size 之外还挂着的单元会话先走空放掉），
/// 交给会话的那一次按步号记下（RefFS 翻译与计数用）。会话交回的每一版交模型比（走空的一步按「走空的一步」，这一次按 [`IdealModel::answer_partial_write`]）。
/// 前提：会话开着、现行那一版树里有活着的号。
fn apply_write_at_offset(
    pool: &mut HistoryPool,
    choice: PartialWriteChoice,
    step_index: usize,
    write_time_seconds: u64,
) -> AppliedStep {
    let HistoryPool {
        devices,
        session: session_slot,
        model,
        stream,
        partial_writes_requested,
        copies_quarantined_in_this_step,
        ..
    } = pool;
    let Some(open_session) = session_slot.as_ref() else {
        return AppliedStep::not_applicable(MissingPrecondition::NoWritableSession);
    };
    let version_of_trees = match &open_session.current {
        PoolVersion::WithoutFile(_) => {
            return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutFile);
        }
        PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => version,
    };
    let Some(inode) = choice.inode_among(&live_inode_numbers(version_of_trees)) else {
        return AppliedStep::not_applicable(MissingPrecondition::NoInodeToWriteInTheCurrentVersion);
    };
    let size_in_bytes = version_of_trees
        .inode_leaf_containers
        .iter()
        .flat_map(|container| container.contents.records.iter())
        .find(|record| record.inode == inode)
        .map_or(0, |record| record.size);
    let data_units_existing = if inode == singlefs_core::transaction::FIRST_INODE_NUMBER {
        version_of_trees.data_pointers.len()
    } else {
        version_of_trees
            .file_other_than_the_first(InodeNumber(inode))
            .map_or(0, |file| file.data_pointers.len())
    };
    let payload = u64::try_from(data_unit_payload_capacity()).expect("净荷容量装得进 u64");
    let data_units_present = u64::try_from(data_units_existing)
        .expect("单元数装得进 u64")
        .min(size_in_bytes.div_ceil(payload).max(1));
    let (offset_in_bytes, length_in_bytes) =
        choice.offset_and_length_in_bytes(size_in_bytes, data_units_present);
    partial_writes_requested.insert(
        step_index,
        PartialWriteRequested {
            inode,
            offset_in_bytes,
            length_in_bytes,
            offset_cell: PartialWriteOffsetCell::of_the_write(
                size_in_bytes,
                data_units_present,
                offset_in_bytes,
                length_in_bytes,
            ),
        },
    );
    let bytes = choice.bytes_of_length(length_in_bytes);
    let records_before = open_session.allocator.records().to_vec();
    let stream_length_before = stream.operation_count();
    let WritableSession {
        allocator,
        current,
        instance,
        shadow_ledger,
        parameters_and_device_table,
        publishes_in_this_mount,
        carried_to_zero_unit_publishes: _,
        compaction,
    } = session_slot.take().expect("上面判过这个进程的会话开着");
    let mut mounted_session = MountedSession {
        allocator,
        current,
        instance,
        shadow_ledger,
        parameters_and_device_table,
        compaction,
    };
    let change_result = mounted_session.publish_user_change(
        devices,
        UserChange::WriteAtOffset {
            inode: InodeNumber(inode),
            offset: FileOffsetInBytes(offset_in_bytes),
            bytes: PartialWriteBytes::new(&bytes)
                .expect("长度至少 1（`offset_and_length_in_bytes` 的契约）"),
            write_time_seconds,
        },
    );
    copies_quarantined_in_this_step.extend(copies_quarantined_by_a_user_change(&change_result));
    let session_after_the_change = session_slot.insert(WritableSession {
        allocator: mounted_session.allocator,
        current: mounted_session.current,
        instance: mounted_session.instance,
        shadow_ledger: mounted_session.shadow_ledger,
        parameters_and_device_table: mounted_session.parameters_and_device_table,
        compaction: mounted_session.compaction,
        publishes_in_this_mount,
        // 零单元发布只接在树表 0 条的一版后面，按偏移写只接在有树的一版后面。
        carried_to_zero_unit_publishes: None,
    });
    settle_user_change_comparing_every_version_published(
        session_after_the_change,
        model,
        &|ideal_model: &IdealModel| {
            ideal_model.answer_partial_write(inode, offset_in_bytes, &bytes)
        },
        &records_before,
        &change_result,
        stream,
        stream_length_before,
    )
}

pub(super) fn apply_operation(
    pool: &mut HistoryPool,
    operation: &HistoryOperation,
    step_index: usize,
) -> AppliedStep {
    let parameters = pool.device_width.parameters();
    // 写入时间是参数、不取系统时钟：同一段历史两次跑逐字节相同。
    let write_time_seconds =
        FIXED_WRITE_TIME_SECONDS + u64::try_from(step_index).expect("步号装得进 u64");
    // 零单元发布不写分配记录，这里不比；挂载写出的写行与暖机那几次在 `settle_mount` 里比。
    match operation {
        HistoryOperation::PublishFirstFile(content) => {
            apply_publish_first_file(pool, &parameters, content, write_time_seconds)
        }
        HistoryOperation::PublishOverwrite(content) => {
            apply_publish_overwrite(pool, content, write_time_seconds)
        }
        HistoryOperation::PublishWithoutUnits => apply_publish_without_units(pool, &parameters),
        HistoryOperation::CloseAndMountWritable => apply_mount_writable(pool, &parameters),
        HistoryOperation::RollBackWhileMounted(choice) => {
            let applied = apply_rollback_while_mounted(pool, &parameters, *choice);
            file_the_current_version_by_what_the_rollback_took(pool);
            applied
        }
        HistoryOperation::RaiseRollbackFloor(choice) => {
            apply_raise_rollback_floor(pool, &parameters, *choice)
        }
        HistoryOperation::ColdStartRecover => apply_cold_start_recover(pool),
        HistoryOperation::CrashRecoveryAbandoningTheNewestRoot => {
            apply_crash_recovery_abandoning_the_newest_root(pool, &parameters)
        }
        HistoryOperation::Unlink => apply_unlink(pool, write_time_seconds, step_index),
        HistoryOperation::CreateInode => apply_create_inode(pool, write_time_seconds),
        HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(content) => {
            apply_overwrite_losing_power_before_its_journal_record_then_mount_writable(
                pool,
                &parameters,
                content,
                write_time_seconds,
            )
        }
        HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros => {
            apply_writable_mount_with_the_newest_system_configuration_slot_reading_zeros(
                pool,
                &parameters,
                DevicesReadingTheirNewestSystemConfigurationSlotAsZeros::EveryDevice,
            )
        }
        HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfOneDeviceReadingZeros => {
            apply_writable_mount_with_the_newest_system_configuration_slot_reading_zeros(
                pool,
                &parameters,
                DevicesReadingTheirNewestSystemConfigurationSlotAsZeros::OnlyTheFirstDevice,
            )
        }
        HistoryOperation::CreateSnapshot => apply_create_snapshot(pool),
        HistoryOperation::ReadSnapshot(choice) => apply_read_snapshot(pool, *choice),
        HistoryOperation::DestroySnapshot(choice) => apply_destroy_snapshot(pool, *choice),
        HistoryOperation::CompactWhileMounted(choice) => {
            apply_compact_while_mounted(pool, &parameters, *choice)
        }
        HistoryOperation::Truncate(choice) => {
            apply_truncate(pool, *choice, step_index, write_time_seconds)
        }
        HistoryOperation::CreateInodes(choice) => {
            apply_create_inodes(pool, *choice, write_time_seconds)
        }
        HistoryOperation::WriteOfAnInode(choice) => {
            apply_write_of_an_inode(pool, *choice, write_time_seconds)
        }
        HistoryOperation::WriteAtOffset(choice) => {
            apply_write_at_offset(pool, *choice, step_index, write_time_seconds)
        }
    }
}

/// 回退在会话的带文件那一版上就地发（`mount::roll_back_by_a_forward_publish` 改的是 `PoolVersion::WithFile` 里那一版）：
/// 回退到删完的一版之后这一版有树而文件集合为空，按 `PoolVersion::of_a_version_of_trees` 的判法重新归成员
/// （不归的话之后的覆盖写照「带文件」调进会话、接在没有文件的一版后面）。
fn file_the_current_version_by_what_the_rollback_took(pool: &mut HistoryPool) {
    let Some(session) = pool.session.as_mut() else {
        return;
    };
    let PoolVersion::WithFile(version) = &session.current else {
        return;
    };
    if version.is_a_version_with_trees_and_no_file() {
        session.current = PoolVersion::WithTreesAndNoFile(version.clone());
    }
}
