//! 用户改动之后的搬迁批：会话推的抬 F、实现隔离的副本与模型看到的那一批。

use super::*;

/// 实现交回的一份「释放之前读盘核出对不上、隔离了」的单元副本（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1；
/// 判决 `research/prompts/m4-i315-r3-main-verification.md` 第三节 C3「实现的隔离清单收齐发布、删快照、整理、回退四条路并跨重挂累积」）：
/// 哪一步交回的、交回的那一份。故障归类器拿它认「插回去那张盘上仍红的那一份是说谎的设备逼出来的隔离」。
///
/// 收的是执行器拿得到的每一处：经会话的用户改动（覆盖写、顺序写、按 inode 号写、unlink、截断、建 inode、建与删快照）交回的每一版、
/// 会话为它推的抬 F 的空发布与改动之后那一趟整理；挂着时整理一趟发成的每一版；新池新建文件那一版；管理员回退交回的
/// `quarantined_user_visible_copies` 与回退那次发布自己那一版；抬 F 的空发布；可写挂载的写行、暖机、删除续做、续做推的抬 F、
/// 挂载里那一趟整理与挂载交回的现行那一版。零单元发布不释放单元、没有这一栏；掉电那一截的发布结局实现自己丢了，收不到。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CopyQuarantinedByTheImplementation {
    pub step: StepPosition,
    pub copy: CopyQuarantinedAfterReleaseChecksumMismatch,
}

/// 一批发成的版里实现交回的隔离（每一版的 `quarantined_after_release_checksum_mismatch`，按版的先后）。
pub(super) fn copies_quarantined_by_the_versions<'version>(
    versions: impl IntoIterator<Item = &'version TransactionOutput>,
) -> Vec<CopyQuarantinedAfterReleaseChecksumMismatch> {
    versions
        .into_iter()
        .flat_map(|version| {
            version
                .quarantined_after_release_checksum_mismatch
                .iter()
                .copied()
        })
        .collect()
}

/// 一次经会话的用户改动发成的每一版里实现交回的隔离：推的抬 F 的空发布、为另发的那一批搬迁推的抬 F 的空发布、推的抬 F 之外发成的每一版（改动那一版与另发的那一批在里面，
/// `mounted_session::VersionsPublishedInThisChange`）；被拒的几种照带着的那几版收。整理不在用户改动里跑（挪出了调用线程，
/// `mounted_session::MountedSession::run_compaction_in_the_background`）。
pub(super) fn copies_quarantined_by_a_user_change(
    change_result: &Result<UserChangePublished, UserChangeRefused>,
) -> Vec<CopyQuarantinedAfterReleaseChecksumMismatch> {
    let (floor_raises, versions_published) =
        floor_raises_and_versions_published_of_a_user_change(change_result);
    let floor_raises_for_the_relocation_batch: &[singlefs_core::mount::RaisedFloor] =
        match change_result {
            Ok(published) => floor_raises_pushed_for_the_relocation_batch(published),
            Err(_) => &[],
        };
    copies_quarantined_by_the_versions(
        floor_raises
            .iter()
            .chain(floor_raises_for_the_relocation_batch)
            .flat_map(|raised| raised.publishes.iter())
            .chain(versions_published.iter().map(Arc::as_ref)),
    )
}

/// 一次发成的用户改动为另发的那一批搬迁推的几串抬 F（按先后，都接在改动那一版后面）。那一批没被空间拒过时是空的。
fn floor_raises_pushed_for_the_relocation_batch(
    published: &UserChangePublished,
) -> &[singlefs_core::mount::RaisedFloor] {
    match &published.relocation_after_the_change_of_the_units_read_with_one_copy_failing {
        RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::NotNeeded => &[],
        RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::Relocated {
            floor_raises, ..
        }
        | RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::NoSpaceAfterRaisingTheFloor {
            floor_raises,
            ..
        }
        | RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::RefusedNotForSpace {
            floor_raises,
            ..
        }
        | RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::FloorRaiseFailedWhilePushingForSpace {
            floor_raises,
            ..
        } => &floor_raises.floor_raises,
    }
}

/// 一次发成的用户改动之后另发的那一批搬迁发成的那一版（排在 `versions_published` 末尾）；不另发、没发成的是 `None`。
fn relocation_batch_published_after_the_change(
    published: &UserChangePublished,
) -> Option<&TransactionOutput> {
    match &published.relocation_after_the_change_of_the_units_read_with_one_copy_failing {
        RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::Relocated { .. } => {
            Some(Arc::as_ref(published.versions_published.last().expect(
                "另发的那一批发成时排在 versions_published 末尾",
            )))
        }
        RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::NotNeeded
        | RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::NoSpaceAfterRaisingTheFloor {
            ..
        }
        | RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::RefusedNotForSpace { .. }
        | RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::FloorRaiseFailedWhilePushingForSpace {
            ..
        } => None,
    }
}

/// 一次发成的用户改动推的抬 F 之外发成的那几版，去掉另发的那一批：走空的那几步与改动自己。
pub(super) fn versions_of_the_change_without_the_relocation_batch(
    published: &UserChangePublished,
) -> &[Arc<TransactionOutput>] {
    let versions = published.versions_published.as_slice();
    match relocation_batch_published_after_the_change(published) {
        Some(_) => &versions[..versions.len() - 1],
        None => versions,
    }
}

/// 另发的那一批在模型眼里看得见的几样：搬了几个单元、搬的里头用户可见树的根单元、这一版写没写实例表片。
fn relocation_batch_seen_by_the_model(
    batch: &TransactionOutput,
) -> ModelRelocationBatchAfterAUserChangeSeen {
    ModelRelocationBatchAfterAUserChangeSeen {
        units_relocated: u64::try_from(batch.units_relocated_by_this_publish.len())
            .expect("一批的单元数装得进 u64"),
        user_visible_root_units_relocated: user_visible_root_units_relocated_by_this_publish(batch),
        instance_table: if batch
            .rewritten
            .iter()
            .any(|unit| unit.is_a_page_of_the_instance_table())
        {
            ModelInstanceTableOfTheBatch::RewrittenWithTheSameRows
        } else {
            ModelInstanceTableOfTheBatch::Carried
        },
    }
}

/// 一次用户改动发成、它自己那几版比过之后，比会话接着为修只坏一份的单元发的东西（D4（校验和位置） 已定项 4「可写挂载里读到的，再经已有的搬迁路径把这个单元整个搬到新槽」）：
/// 那几串抬 F 照模型的抬 F 比，发成的那一批照 [`IdealModel::answer_relocation_batch_after_a_user_change`] 比；分配代接着 `records_after_the_change` 逐次比，
/// 那一批按搬迁的判法（[`allocation_generation_judgement_of_a_relocation_batch`]：目的地写单元的诞生代）。前面已经对不上就不再问模型（分配代照判）。
/// 交回这几样一共写了几条根。这几条根都带着这次改动（内容照抄改动那一版）。
pub(super) fn settle_what_the_session_published_for_the_relocation_batch(
    session: &mut WritableSession,
    model: &mut IdealModel,
    published: &UserChangePublished,
    records_after_the_change: &[AllocationRecord],
    verdict: &mut ModelVerdict,
    harness_judgement: &mut Option<HarnessJudgement>,
) -> usize {
    let mut records_of_the_previous_publish = records_after_the_change;
    let mut roots_written = 0_usize;
    for raised in floor_raises_pushed_for_the_relocation_batch(published) {
        if verdict.disagreement.is_none() {
            let raise_verdict = judge_one_floor_raise_pushed_by_the_session(model, raised);
            verdict.counts.add(&raise_verdict.counts);
            verdict.disagreement = raise_verdict.disagreement;
        }
        for publish in &raised.publishes {
            let publish_txg = publish.root.checkpoint_txg;
            *harness_judgement = harness_judgement.take().or_else(|| {
                allocation_generation_judgement(
                    records_of_the_previous_publish,
                    &publish.allocation_records,
                    publish_txg,
                    publish_txg,
                    &publish.quarantined_after_release_checksum_mismatch,
                )
            });
            records_of_the_previous_publish = &publish.allocation_records;
            roots_written += 1;
        }
    }
    if let Some(batch) = relocation_batch_published_after_the_change(published) {
        if verdict.disagreement.is_none() {
            let answer = model.answer_relocation_batch_after_a_user_change(
                &relocation_batch_seen_by_the_model(batch),
            );
            let batch_verdict = judge_by_model(
                model,
                answer,
                &ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                    roots: vec![observed_root_of_file_version(batch)],
                    reported_ceiling: None,
                }),
            );
            verdict.counts.add(&batch_verdict.counts);
            verdict.disagreement = batch_verdict.disagreement;
        }
        *harness_judgement = harness_judgement.take().or_else(|| {
            allocation_generation_judgement_of_a_relocation_batch(
                records_of_the_previous_publish,
                batch,
            )
        });
        roots_written += 1;
    }
    session.publishes_in_this_mount += roots_written;
    roots_written
}

/// 一次可写挂载交回的每一版里实现交回的隔离：写行那一版、暖机、删除续做、续做推的抬 F、挂载里那一趟整理、挂载交回的现行那一版
/// （几样里同一版会重复，并进来时去重）。
pub(super) fn copies_quarantined_by_a_mount(
    mounted: &singlefs_core::mount::Mounted,
) -> Vec<CopyQuarantinedAfterReleaseChecksumMismatch> {
    let output = &mounted.output;
    let versions_of_the_compaction_continuation: &[TransactionOutput] = output
        .compaction_continuation
        .run
        .as_ref()
        .map(|run| run.versions_published.as_slice())
        .unwrap_or_default();
    copies_quarantined_by_the_versions(
        std::iter::once(&output.row_publish)
            .chain(&output.warm_up_publishes)
            .chain(std::iter::once(&mounted.current))
            .filter_map(|version| version.version_of_trees())
            .chain(output.deletion_queue_continuation.iter().map(Arc::as_ref))
            .chain(
                output
                    .floor_raises_pushed_by_the_continuation
                    .iter()
                    .flat_map(|raised| raised.publishes.iter()),
            )
            .chain(versions_of_the_compaction_continuation),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model_comparison::observed_node_mapping_key;
    use singlefs_core::allocator::Placement;
    use singlefs_core::block_device::{BlockDevice, WriteDurability};
    use singlefs_core::compaction_plan::RelocationBatchAfterTheChangeRefusedForSpace;
    use singlefs_core::mount::RaisedFloor;
    use singlefs_core::mounted_session::{
        DiscardsAfterThePublishes, FloorRaisesPushedForTheRelocation,
    };
    use singlefs_core::pointer::LocationEntry;
    use singlefs_core::records::TREE_KIND_EXTENT;
    use singlefs_core::transaction::{
        QuarantinedCopyReading, RelocatedUnitHolder, TransactionUnit,
    };
    use singlefs_core::unit::UNIT_CLASS_INDEX_NODE;
    use std::num::NonZeroU64;

    /// 用户改动发成之后另发的那一批（D4（校验和位置） 已定项 4）的用例共用：可写会话开着、第一个文件已发成的一池（两块 4 GiB 的盘，不注入故障）。
    fn pool_after_the_first_file() -> HistoryPool {
        let device_width = HistoryDeviceWidth::FourGibibytes;
        let Ok((pool, _verdict_of_the_first_file)) = HistoryPool::start(
            HistoryStartingPoint::AfterFirstFile,
            device_width,
            SpaceAdmission::JudgedByTheFormula,
            &SharedStream::new(),
            &SharedFaultPlan::unarmed(device_width.fixed_geometry()),
        ) else {
            panic!("不注入故障时起点段的入口不报错");
        };
        pool
    }

    fn current_version_of_the_session(pool: &HistoryPool) -> &TransactionOutput {
        pool.session
            .as_ref()
            .expect("会话开着")
            .current
            .version_of_trees()
            .expect("第一个文件发成之后现行那一版有树")
    }

    /// 把 `location` 那一份头一槽改一个字节，另一份不动：之后读这个单元只坏一份。
    fn damage_one_copy_at(pool: &mut HistoryPool, location: LocationEntry) {
        let (_, device) = pool
            .devices
            .iter_mut()
            .find(|(identity, _)| *identity == location.device)
            .expect("位置条目点名的是池里的盘");
        let offset = location.slot.to_device_offset();
        let mut bytes_of_the_slot =
            vec![0_u8; usize::try_from(SLOT_BYTES).expect("一槽装得进 usize")];
        device
            .read_at(offset, &mut bytes_of_the_slot)
            .expect("不注入故障时读得出");
        bytes_of_the_slot[100] ^= 0xFF;
        device
            .write_at(offset, &bytes_of_the_slot, WriteDurability::Plain)
            .expect("不注入故障时写得进");
    }

    /// 一个数据单元装得下的一份覆盖写内容。
    const CONTENT_OF_AN_OVERWRITE: ContentChoice = ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector: 0 },
        fill_seed: 1,
    };

    /// 这一步执行器判的失败与模型的对不上都没有，交回模型比过的计数。
    fn counts_of_a_step_without_a_failure(step: &AppliedStep) -> &ModelJudgementCounts {
        assert_eq!(step.harness_judgement, None, "执行器不判失败");
        let verdict = step.model_verdict.as_ref().expect("入口调了：模型答过");
        assert!(
            verdict.disagreement.is_none(),
            "模型对得上：{:?}",
            verdict.disagreement
        );
        &verdict.counts
    }

    fn roots_carrying_the_change_of(step: &AppliedStep) -> usize {
        let StepOutcome::Applied(AppliedEffect::Published {
            roots_carrying_the_change,
            ..
        }) = &step.outcome
        else {
            panic!("这一步发成了改动：{:?}", step.outcome);
        };
        *roots_carrying_the_change
    }

    /// 模型的现行版本就是会话的现行那一版：会话写的根模型一条都没漏。
    fn assert_the_model_stands_on_the_root_of_the_session(pool: &HistoryPool) {
        let root = pool.session.as_ref().expect("会话开着").current.root();
        assert_eq!(
            pool.model
                .current_version_in_the_session()
                .expect("模型里会话开着")
                .key,
            model_root_key(root.instance, root.checkpoint_txg)
        );
    }

    /// 覆盖写那次发布读到实例表片只坏一份，会话接着另发一批换实例表：改动自己那一版照覆盖写比（分配代是它自己的 txg），那一批照另发的那一批比（模型记实例表这次重写），
    /// 这一步写两条根、都带着这次改动，模型停在那一批上。判别力：执行器拿现行那一版当改动那一版比时（模型答 txg N、实现是 N + 1）这一条红。
    #[test]
    fn an_overwrite_reading_the_instance_table_with_one_copy_failing_is_followed_by_a_batch_judged_on_its_own(
    ) {
        let mut pool = pool_after_the_first_file();
        let version_before = current_version_of_the_session(&pool);
        let txg_before = version_before.root.checkpoint_txg.0;
        let table_location = version_before.root.instance_table.locations[0];
        damage_one_copy_at(&mut pool, table_location);
        let publishes_before = pool
            .session
            .as_ref()
            .expect("会话开着")
            .publishes_in_this_mount;
        let step = apply_publish_overwrite(
            &mut pool,
            &CONTENT_OF_AN_OVERWRITE,
            FIXED_WRITE_TIME_SECONDS + 1,
        );
        assert_eq!(
            counts_of_a_step_without_a_failure(&step)
                .relocation_batches_after_a_user_change_matched,
            1,
            "另发的那一批交模型比过一次"
        );
        assert_eq!(
            roots_carrying_the_change_of(&step),
            2,
            "改动那一版与另发的那一批"
        );
        assert_the_model_stands_on_the_root_of_the_session(&pool);
        let batch = current_version_of_the_session(&pool);
        assert_eq!(batch.root.checkpoint_txg.0, txg_before + 2);
        assert_eq!(
            pool.session
                .as_ref()
                .expect("会话开着")
                .publishes_in_this_mount,
            publishes_before + 2,
            "这次挂载里写出的根连那一批一起数"
        );
        assert_ne!(
            batch.root.instance_table.locations[0].slot, table_location.slot,
            "那一批把实例表换到了新落点"
        );
    }

    /// 可写挂载建上一版时读到数据单元只坏一份，下一次用户改动（建 inode，不重写这个单元）之后另发一批搬它：目的地的分配代是单元的诞生代，
    /// 那一批按搬迁的判法比过（按「代要是这次的 txg」比就判红）；走的是逐版比那一支（`settle_user_change_comparing_every_version_published`），
    /// 那一批不当成又一次改动问模型。
    #[test]
    fn data_unit_read_with_one_copy_failing_while_mounting_moves_in_a_batch_after_the_next_change_that_is_judged_as_a_relocation(
    ) {
        let mut pool = pool_after_the_first_file();
        let data_location = current_version_of_the_session(&pool).data_pointers[0].locations[0];
        damage_one_copy_at(&mut pool, data_location);
        let parameters = pool.device_width.parameters();
        let mount = apply_mount_writable(&mut pool, &parameters);
        let _counts_of_the_mount = counts_of_a_step_without_a_failure(&mount);
        let step = apply_create_inodes(
            &mut pool,
            InodeCountChoice { count_drawn: 1 },
            FIXED_WRITE_TIME_SECONDS + 1,
        );
        assert_eq!(
            counts_of_a_step_without_a_failure(&step)
                .relocation_batches_after_a_user_change_matched,
            1,
            "另发的那一批交模型比过一次"
        );
        assert_eq!(
            roots_carrying_the_change_of(&step),
            2,
            "建 inode 那一版与另发的那一批"
        );
        assert_the_model_stands_on_the_root_of_the_session(&pool);
        let batch = current_version_of_the_session(&pool);
        let [relocated] = batch.units_relocated_by_this_publish.as_slice() else {
            panic!(
                "那一批搬一个单元：{:?}",
                batch.units_relocated_by_this_publish
            );
        };
        assert_eq!(
            relocated.source[0].slot, data_location.slot,
            "搬的是挂载时读到只坏一份的那个单元"
        );
    }

    /// 现行那一版上再覆盖写两次（抬 F 有得抬），弄坏实例表那一份，头 `attempts` 次发那一批当作被空间拒（只供测试的开关）。
    fn pool_whose_next_overwrite_reads_the_damaged_table_with_the_batch_refused_for_space(
        forcing: RelocationBatchAfterTheChangeRefusedForSpace,
    ) -> HistoryPool {
        let mut pool = pool_after_the_first_file();
        for write_time_offset in 1..=2 {
            let step = apply_publish_overwrite(
                &mut pool,
                &CONTENT_OF_AN_OVERWRITE,
                FIXED_WRITE_TIME_SECONDS + write_time_offset,
            );
            let _counts = counts_of_a_step_without_a_failure(&step);
        }
        let table_location = current_version_of_the_session(&pool)
            .root
            .instance_table
            .locations[0];
        damage_one_copy_at(&mut pool, table_location);
        pool.session
            .as_mut()
            .expect("会话开着")
            .compaction
            .force_the_relocation_batch_after_the_change_refused_for_space_for_tests(forcing);
        pool
    }

    /// 那一批头一次被空间拒、推一串抬 F 之后发成：那一串接在改动那一版后面照模型的抬 F 比，分配代逐次接着比，那一批接在那一串后面比。
    /// 判别力：那一串不交模型（模型少走几条根）或分配代不接着比时这一条红。
    #[test]
    fn floor_raises_pushed_for_the_batch_after_a_change_are_judged_between_the_change_and_the_batch(
    ) {
        let mut pool =
            pool_whose_next_overwrite_reads_the_damaged_table_with_the_batch_refused_for_space(
                RelocationBatchAfterTheChangeRefusedForSpace::ForcedForTheNextAttemptsForTests {
                    attempts: NonZeroU64::new(1).expect("拒一次：1 不是 0"),
                },
            );
        let step = apply_publish_overwrite(
            &mut pool,
            &CONTENT_OF_AN_OVERWRITE,
            FIXED_WRITE_TIME_SECONDS + 3,
        );
        assert_eq!(
            counts_of_a_step_without_a_failure(&step)
                .relocation_batches_after_a_user_change_matched,
            1,
            "推过抬 F 之后那一批发成"
        );
        assert!(
            roots_carrying_the_change_of(&step) > 2,
            "改动那一版、至少一条抬 F 的空发布、那一批：{}",
            roots_carrying_the_change_of(&step)
        );
        assert_the_model_stands_on_the_root_of_the_session(&pool);
    }

    /// 那一批每一次都被空间拒：一批都没发成，推的那几串照样交模型比；模型停在推的最后一条空发布上，不算比过那一批。
    #[test]
    fn floor_raises_pushed_for_a_batch_that_never_got_published_are_still_judged() {
        let mut pool =
            pool_whose_next_overwrite_reads_the_damaged_table_with_the_batch_refused_for_space(
                RelocationBatchAfterTheChangeRefusedForSpace::ForcedOnEveryAttemptForTests,
            );
        let step = apply_publish_overwrite(
            &mut pool,
            &CONTENT_OF_AN_OVERWRITE,
            FIXED_WRITE_TIME_SECONDS + 3,
        );
        assert_eq!(
            counts_of_a_step_without_a_failure(&step)
                .relocation_batches_after_a_user_change_matched,
            0,
            "那一批没发成"
        );
        assert!(
            roots_carrying_the_change_of(&step) > 1,
            "改动那一版之后至少推了一条抬 F 的空发布：{}",
            roots_carrying_the_change_of(&step)
        );
        assert_the_model_stands_on_the_root_of_the_session(&pool);
    }

    /// 为另发的那一批推的抬 F 的空发布交回的隔离也收进这一步（故障归类器拿隔离清单认说谎的设备留下的）。隔离那一份合成：
    /// 一串空发布真核出对不上要坏一份旧固定点单元，这里只核收不收。
    #[test]
    fn copies_quarantined_by_a_floor_raise_pushed_for_the_batch_after_a_change_are_collected() {
        let pool = pool_after_the_first_file();
        let quarantined = CopyQuarantinedAfterReleaseChecksumMismatch {
            unit: TransactionUnit::InodeRoot,
            device: DeviceIdentity(0),
            placement: Placement {
                slot: SlotNumber(777),
                span: 1,
            },
            reading: QuarantinedCopyReading::ChecksumMismatch,
        };
        let mut publish = current_version_of_the_session(&pool).clone();
        publish.quarantined_after_release_checksum_mismatch = vec![quarantined];
        let change_result = Ok(UserChangePublished {
            floor_raises: Vec::new(),
            refusals_that_pushed_the_floor_raises: Vec::new(),
            versions_published: Vec::new(),
            discards_after_the_change: DiscardsAfterThePublishes::default(),
            relocation_after_the_change_of_the_units_read_with_one_copy_failing:
                RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::RefusedNotForSpace {
                    refusal: Box::new(RelocationBatchRefused::NoLiveCompactionIntent),
                    floor_raises: FloorRaisesPushedForTheRelocation {
                        floor_raises: vec![RaisedFloor {
                            ceiling: publish.root.checkpoint_txg,
                            system_configuration_writes_before_the_first_publish: publish
                                .writes
                                .clone(),
                            publishes: vec![publish],
                            floor_of_the_publishes: FloorOfThePushedPublishes::RaisedToTheCeiling,
                            reclaimed: Vec::new(),
                            abandoned_roots_unreadable: 0,
                        }],
                        refusals_that_pushed_the_floor_raises: Vec::new(),
                    },
                },
        });
        assert_eq!(
            copies_quarantined_by_a_user_change(&change_result),
            vec![quarantined]
        );
    }

    /// 另发的那一批交给模型的几样照这一版交回的算：搬了几个、搬的里头有没有用户可见树的根；这一版没写实例表片就是照抄。
    #[test]
    fn the_batch_seen_by_the_model_names_what_the_version_relocated() {
        let pool = pool_after_the_first_file();
        let mut batch = current_version_of_the_session(&pool).clone();
        let extent_root = batch
            .tree_table_entries
            .iter()
            .find(|entry| entry.kind == TREE_KIND_EXTENT)
            .expect("第一个文件那一版的树表里有 extent 树那一条")
            .root;
        batch.units_relocated_by_this_publish = vec![UnitRelocatedByThisPublish {
            mapping_key: observed_node_mapping_key(UNIT_CLASS_INDEX_NODE, extent_root),
            holder: RelocatedUnitHolder::KeptForTheSnapshots,
            source: extent_root.locations,
            destination: extent_root.locations,
        }];
        let seen = relocation_batch_seen_by_the_model(&batch);
        assert_eq!(seen.units_relocated, 1);
        assert_eq!(
            seen.user_visible_root_units_relocated.len(),
            1,
            "搬的是 extent 树的根：{:?}",
            seen.user_visible_root_units_relocated
        );
        assert_eq!(seen.instance_table, ModelInstanceTableOfTheBatch::Carried);
    }
}
