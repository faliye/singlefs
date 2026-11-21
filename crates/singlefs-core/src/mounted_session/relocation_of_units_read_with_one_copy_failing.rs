//! 一次用户改动发成之后，把这次改动里读回时只坏一份的旧数据单元另发一批搬迁走
//! （[`RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing`]）：被空间拒时推抬 F 再发，连同只供测试的强制拒绝。

use super::*;

/// 一次用户改动发成之后，读到只坏一份的单元另发一批换落点（D4（校验和位置） 已定项 4「经已有的搬迁路径把这个单元整个搬到新槽、两份新写，旧落点照释放前核隔离；
/// 走同一个事务层」；用户 2026-11-04 定有活快照时「另发一批搬迁修好」，另发那批被拒时「推抬 F 再试一遍」）做成了什么。封闭集合。用户改动在每一种结局下都已发成。
/// 这一批管三处读到的：这次改动读—改—写读回的、改动之后仍被活快照钉着的旧单元；可写挂载读盘建上一版时读到的（挂载记下、挂载之后头一次改动发成时取走）；
/// 可写挂载期间别的读读到的（[`crate::allocator::PoolAllocator::reading_and_counting_the_other_unit_reads_with_one_copy_failing`] 文档串里那几类，其中写系统配置槽之前算 F 生效值每次发布都读最新那条根的实例表，
/// 这次改动那次发布读到的片就由这一批换）。读的是别的版本、这一批在现行那一版里认不出的只留下计数，不搬。
/// 豁免映射的三类（映射树节点、树表、实例表片）认不出映射 key，同一批按它们自己的重写路子换落点（`transaction::RepairOfTheExemptUnits`）。
/// 这一组记下的这一次只试一次，结局报在这里，之后的用户改动不接着搬；同一组之后再被读到就再记、再试一次（账只记等下一次改动的那几组，试过的不留）。
/// 没有活快照钉着时读回的单元在改动那次发布里就整个重写、旧落点照释放之前读盘核隔离，不另发。
/// 那一批被空间拒（[`refusal_is_short_of_space`] 那两种）时与会话走空同一套（`publish_the_queue_until_it_is_empty_raising_the_floor_for_space`）：
/// 推一串抬 F、回收之后再发同一批，预算按 `mount::push_one_floor_raise_within_the_admission_budget` 数，这一批算一次准入，推的数从 0 起。
#[derive(Debug)]
pub enum RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing {
    /// 不另发：没有记着读到只坏一份的单元，或记着的那几个在这次改动发成之前已经换了落点（读回的旧单元没有活快照钉着、挂载记下的被这次改动整个重写、
    /// 树表与映射树根这类每次发布都换下的），认位置时沿链读实例表也读通、没读到只坏一份。
    NotNeeded,
    /// 另发了一批（`transaction::publish_the_relocations_and_the_repair_of_the_exempt_units`，接在改动发成的那一版后面，被空间拒过的接在推的最后一串后面）：
    /// 计划搬这几把映射 key 的单元，按计划的次序（计划里是这一批本来就重写的节点的，不另搬也在里面）；同一次发布换下这几个豁免映射的单元。
    /// 那一版排在 `versions_published` 末尾，也是会话的 `current`。
    /// `instance_table_chain_unreadable`：账上还有认不出的组、沿链读实例表读不出时的那个失败，实例表这一批没换（别的照发，D4（校验和位置） 已定项 4 不让一处读不出连带丢掉好 key）；
    /// 读得出或用不着读是 `None`。
    Relocated {
        mapping_keys: Vec<Vec<u8>>,
        units_exempt_from_the_mapping_replaced: Vec<TransactionUnit>,
        instance_table_chain_unreadable: Option<RecoveryFailure>,
        floor_raises: FloorRaisesPushedForTheRelocation,
    },
    /// 被空间拒、推满仍放不下（`stop`：预算用完、F 已在上限、抬 F 那一串自己也推不出空间）：会话的 `current` 停在推的最后一串的最后一次空发布上
    /// （一串都没推成时停在改动发成的那一版上），被钉着的单元坏的那一份留在旧落点上，这一批一个字节都没写（空间准入与落点那两拒都在任何写之前）。
    /// `last_refusal` 是最后一次被拒的原样。装箱：拒绝原因带着发布路径的错。
    NoSpaceAfterRaisingTheFloor {
        last_refusal: Box<RelocationBatchRefused>,
        stop: FloorRaiseStop,
        floor_raises: FloorRaisesPushedForTheRelocation,
    },
    /// 被拒、不是空间不够（计划对不上那一版、发布路径别的错；落盘中途块设备错照发布路径冻结在分配器上，不重试；或一样要发的都没有而沿链读实例表读不出，
    /// `RelocationBatchRefused::InstanceTableOfTheVersionUnreadable`）：会话的 `current` 停在最后一次落盘的
    /// 那一版上（改动发成的那一版，或推过抬 F 时最后一次空发布）。只供测试的开关 `compaction_plan::RelocationBatchAfterTheChangeRefusedNotForSpace`
    /// 强制进这一支（计划对不上那一形）。
    RefusedNotForSpace {
        refusal: Box<RelocationBatchRefused>,
        floor_raises: FloorRaisesPushedForTheRelocation,
    },
    /// 为空间推抬 F 时抬 F 自己报了错（`mount::push_one_floor_raise_within_the_admission_budget` 的 `Err`，原样）：这一串是因为
    /// `refusal_that_pushed_the_failed_raise` 那一次被拒才推的；之前推成的几串在 `floor_raises`。会话的 `current` 停在最后一次落盘的那一版上。
    /// 只供测试的开关 `compaction_plan::FloorRaiseForTheRelocationBatchAfterTheChangeFails`（连同被空间拒的那个开关）强制进这一支。
    FloorRaiseFailedWhilePushingForSpace {
        cause: Box<MountError>,
        refusal_that_pushed_the_failed_raise: Box<RelocationBatchRefused>,
        floor_raises: FloorRaisesPushedForTheRelocation,
    },
}

/// 为另发的那一批搬迁推的几串抬 F（按先后）与推它们的那几次被拒（一一对应）。一次都没被空间拒是两个空表。
#[derive(Debug)]
pub struct FloorRaisesPushedForTheRelocation {
    pub floor_raises: Vec<RaisedFloor>,
    pub refusals_that_pushed_the_floor_raises: Vec<RelocationBatchRefused>,
}

/// 用户改动发成之后另发的那一批（[`RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing`]）：`mapping_keys` 是这次改动里读回时只坏一份的旧单元，
/// 其中到 `current` 这一版仍被活快照钉着的（`transaction::relocations_of_old_data_units_kept_for_the_snapshots`）一个都没有就不发；有就在新开的
/// 写入口上照整理的搬迁入口发一批（[`publish_the_relocation_batch_raising_the_floor_for_space`]，意图照抄：`transaction::publish_relocation_batch` 那一支），
/// 同一个事务层、同一条发布路径：原样字节写到新槽两份、留存的映射条目换到新槽、源落点照释放之前读盘核（只坏一份 ⇒ 两份一起隔离）。
/// 被空间拒就推抬 F 再发同一批（用户 2026-11-04 定「推抬 F 再试一遍」）。推成的抬 F 已落盘：`current` 先换成推到的那一版；
/// 那一批发成就把它追加进 `versions_published`、换成 `current`。计划只算一次：每次发之前发布路径照它接在后面的那一版核计划
/// （计划对不上在任何写之前拒，`RelocationBatchRefused::PlannedSourceIsNotTheMappingPosition` 这一类）。
///
/// `mapping_keys_read_while_mounting` 是可写挂载读盘建上一版时读到只坏一份的那几个单元（`compaction_plan::CompactionOfThisMount` 记下的映射 key，
/// 用户 2026-11-05 定「可写挂载时搬，冷恢复只计数」）：其中到 `current` 这一版仍在中央映射里或留存条目里的那几个
/// （`transaction::relocations_of_the_units_read_with_one_copy_failing`）接在上面那几个后面、进同一批，同一条路径、同一套被拒推抬 F；
/// 结局报在同一个成员里（[`RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing`]）。挂载那一串里不另发：另发就是挂载的第七段写，挂载分段与二次崩溃、
/// 取号之前的续做预演都要跟着改，下一次用户改动之后这一处已有搬迁路径与它的崩溃点（判决 `research/prompts/m4-close-code-r3-main-verification.md`
/// 第三节第 7 条「挑走已有路径改动最小的那一处」）。
///
/// 可写挂载期间别的读读到只坏一份的那几组位置条目（分配器上那份账）按 `current` 这一版认：认出映射 key 的走上面同一条路，
/// 此刻是豁免映射的单元的同一次发布换下（`transaction::repair_of_the_exempt_units_at_these_locations`），别的（这次改动已经换下的）不管。
/// 认的时候沿链读实例表读到只坏一份的那几片也进账、同一批接着换。实例表链读不出时实例表那一半跳过，别的照发，结局报明实例表没换
/// （[`RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::Relocated`] 的 `instance_table_chain_unreadable`；一样要发的都没有时报在 `RefusedNotForSpace` 里）。
/// 这几组与这一批要搬的源落点上记下的，发完（发成或被拒）都从账上去掉：这一组记下的这一次搬没搬成都只试这一次。这一批自己推抬 F 时新读到的不去掉，归下一次改动。
///
/// # Panics
/// `current` 不是有树的一版：调用方只在一次用户改动发成之后调，发成的都是有树的一版（`publish_user_change` 开头拒了没树的那一形）。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：参数、盘表、分配器、现行那一版、实例代号、影子账那一臂、会话这一侧的整理状态、读回只坏一份的 key、挂载时读到只坏一份的 key、这次调用发成的各版"
)]
pub(super) fn relocate_the_units_read_with_one_copy_failing_after_the_change<
    Device: BlockDevice,
>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut PoolVersion,
    instance: InstanceGeneration,
    shadow_ledger: ShadowLedger,
    compaction: &mut CompactionOfThisMount,
    mapping_keys: &[Vec<u8>],
    mapping_keys_read_while_mounting: &[Vec<u8>],
    versions_published: &mut VersionsPublishedInThisChange,
) -> RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing {
    let version_after_the_change = current
        .version_of_trees()
        .expect("一次用户改动发成之后现行那一版是有树的一版");
    let mut planned = relocations_of_old_data_units_kept_for_the_snapshots(
        version_after_the_change,
        mapping_keys,
    );
    let locations_read_with_one_copy_failing =
        allocator.locations_read_with_one_copy_failing_waiting_for_the_next_user_change();
    let units_read_with_one_copy_failing = units_at_these_locations(
        version_after_the_change,
        &locations_read_with_one_copy_failing,
    );
    let mapping_keys_read_while_mounting_and_by_the_other_reads: Vec<Vec<u8>> =
        mapping_keys_read_while_mounting
            .iter()
            .chain(&units_read_with_one_copy_failing.mapping_keys)
            .cloned()
            .collect();
    let planned_for_the_units_read_while_mounting =
        relocations_of_the_units_read_with_one_copy_failing(
            version_after_the_change,
            &mapping_keys_read_while_mounting_and_by_the_other_reads,
            &planned,
        );
    planned.extend(planned_for_the_units_read_while_mounting);
    let repair = repair_of_the_exempt_units_at_these_locations(
        &*devices,
        version_after_the_change,
        &units_read_with_one_copy_failing.locations_without_a_mapping_key,
    );
    allocator.count_the_other_unit_reads_with_one_copy_failing(
        repair
            .locations_read_with_one_copy_failing_while_reading_the_chain
            .clone(),
    );
    let locations_tried_by_this_batch: Vec<[LocationEntry; 2]> =
        locations_read_with_one_copy_failing
            .iter()
            .chain(&repair.locations_read_with_one_copy_failing_while_reading_the_chain)
            .copied()
            .collect();
    if planned.is_empty() && repair.is_empty() {
        allocator.forget_the_locations_read_with_one_copy_failing(|locations| {
            locations_tried_by_this_batch.contains(locations)
        });
        return match repair.instance_table_chain_unreadable {
            None => RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::NotNeeded,
            Some(failure) => {
                RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::RefusedNotForSpace {
                    refusal: Box::new(RelocationBatchRefused::InstanceTableOfTheVersionUnreadable(
                        failure,
                    )),
                    floor_raises: FloorRaisesPushedForTheRelocation {
                        floor_raises: Vec::new(),
                        refusals_that_pushed_the_floor_raises: Vec::new(),
                    },
                }
            }
        };
    }
    let mut version_to_relocate_on = version_after_the_change.clone();
    let batch = publish_the_relocation_batch_raising_the_floor_for_space(
        parameters,
        devices,
        allocator,
        &mut version_to_relocate_on,
        instance,
        shadow_ledger,
        compaction,
        &planned,
        &repair,
    );
    allocator.forget_the_locations_read_with_one_copy_failing(|locations| {
        locations_tried_by_this_batch.contains(locations)
            || locations.iter().any(|location| {
                planned
                    .iter()
                    .any(|relocation| relocation.source.slot == location.slot)
            })
    });
    // 推成的抬 F 已落盘（被拒的结局也是）：现行那一版至少是推到的那一版。
    *current = PoolVersion::of_a_version_of_trees(version_to_relocate_on);
    match batch {
        RelocationBatchAfterTheFloorRaises::Published {
            relocated,
            floor_raises,
        } => {
            let relocated = Arc::new(*relocated);
            versions_published.push(Arc::clone(&relocated));
            *current = PoolVersion::of_a_shared_version_of_trees(relocated);
            RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::Relocated {
                mapping_keys: planned
                    .into_iter()
                    .map(|relocation| relocation.mapping_key)
                    .collect(),
                units_exempt_from_the_mapping_replaced: repair.units,
                instance_table_chain_unreadable: repair.instance_table_chain_unreadable,
                floor_raises,
            }
        }
        RelocationBatchAfterTheFloorRaises::NotPublished(not_published) => not_published,
    }
}

/// 另发那一批走完推抬 F 再试之后：发成（发出来的那一版与推的几串），或会话要报的那一种没发成
/// （[`RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing`] 的三种被拒）。
enum RelocationBatchAfterTheFloorRaises {
    /// 装箱：一版的输出远大过另一个成员（clippy `large_enum_variant`）。
    Published {
        relocated: Box<TransactionOutput>,
        floor_raises: FloorRaisesPushedForTheRelocation,
    },
    NotPublished(RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing),
}

/// 在新开的写入口上照 `planned` 与 `repair` 发一批（[`publish_the_relocation_batch_once`]，意图照抄），接在 `version` 后面；被空间拒
/// （[`compaction_publish_refused_short_of_space`]：发布路径的空间准入不够、或准入放行而落点取不到）就推一串抬 F
/// （`mount::push_one_floor_raise_within_the_admission_budget`，就地推进 `version`）、回收之后再发同一批——与会话走空被空间拒同一套
/// （[`publish_the_queue_until_it_is_empty_raising_the_floor_for_space`]）：这一批算一次准入，推的数从 0 起，预算照那个函数数（推成的每一串的空发布数相加）。
/// 推满仍放不下、抬 F 自己报错、被拒不是空间不够，交回会话要报的那一种；被拒不是空间不够的不再试（落盘途中块设备错照发布路径冻结在分配器上）。
/// 只供测试的开关（`compaction_plan::RelocationBatchAfterTheChangeRefusedForSpace`）打开时这一次不建写入口，交回一个空间准入不够的拒绝
/// （[`space_refusal_of_the_relocation_forced_for_tests`]），照被空间拒那一支往下走；另两个只供测试的开关同一种进法：
/// `compaction_plan::RelocationBatchAfterTheChangeRefusedNotForSpace` 打开时交回一个计划对不上的拒绝
/// （[`refusal_not_for_space_of_the_relocation_forced_for_tests`]），照被拒不是空间不够那一支往下走；
/// `compaction_plan::FloorRaiseForTheRelocationBatchAfterTheChangeFails` 打开时为空间推抬 F 那一步不推、交回一个抬 F 的错
/// （[`floor_raise_failure_of_the_relocation_forced_for_tests`]），照抬 F 自己报错那一支往下走。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：参数、盘表、分配器、接在后面的那一版、实例代号、影子账那一臂、会话这一侧的整理状态、这一批的计划、这一批换下的豁免映射的单元"
)]
fn publish_the_relocation_batch_raising_the_floor_for_space<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    version: &mut TransactionOutput,
    instance: InstanceGeneration,
    shadow_ledger: ShadowLedger,
    compaction: &mut CompactionOfThisMount,
    planned: &[PlannedRelocation],
    repair: &RepairOfTheExemptUnits,
) -> RelocationBatchAfterTheFloorRaises {
    let mut floor_raises = FloorRaisesPushedForTheRelocation {
        floor_raises: Vec::new(),
        refusals_that_pushed_the_floor_raises: Vec::new(),
    };
    let mut publishes_pushed_in_this_admission = 0;
    // 迭代上界：每一轮要么交回（发成、被拒不是空间不够、推满仍放不下、抬 F 报错），要么推成一串（至少一次空发布，推的数加进这次准入，
    // 预算在 `mount::PUBLISHES_PER_ADMISSION_AT_MOST` − 1 次空发布之内停）⇒ 至多 8 轮。跨轮携带的是推到的那一版、已推的数与推成的几串。
    loop {
        let refusal_of_the_relocation = if compaction
            .take_a_forced_space_refusal_of_the_relocation_batch_after_the_change()
        {
            space_refusal_of_the_relocation_forced_for_tests()
        } else if compaction.is_the_relocation_batch_after_the_change_forced_refused_not_for_space()
        {
            refusal_not_for_space_of_the_relocation_forced_for_tests(planned)
        } else {
            match publish_the_relocation_batch_once(
                parameters,
                devices.as_mut_slice(),
                allocator,
                version,
                planned,
                repair,
                instance,
            ) {
                Ok(relocated) => {
                    return RelocationBatchAfterTheFloorRaises::Published {
                        relocated: Box::new(relocated),
                        floor_raises,
                    }
                }
                Err(refusal) => refusal,
            }
        };
        if !compaction_publish_refused_short_of_space(&refusal_of_the_relocation) {
            return RelocationBatchAfterTheFloorRaises::NotPublished(
                RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::RefusedNotForSpace {
                    refusal: Box::new(refusal_of_the_relocation),
                    floor_raises,
                },
            );
        }
        let floor_raise = if compaction
            .is_the_floor_raise_for_the_relocation_batch_after_the_change_forced_to_fail()
        {
            Err(floor_raise_failure_of_the_relocation_forced_for_tests())
        } else {
            push_one_floor_raise_within_the_admission_budget(
                parameters,
                devices,
                allocator,
                version,
                shadow_ledger,
                publishes_pushed_in_this_admission,
            )
        };
        match floor_raise {
            Ok(FloorRaisePushedWithinTheAdmissionBudget::Raised(raised)) => {
                publishes_pushed_in_this_admission += raised.publishes.len();
                floor_raises.floor_raises.push(raised);
                floor_raises
                    .refusals_that_pushed_the_floor_raises
                    .push(refusal_of_the_relocation);
            }
            Ok(FloorRaisePushedWithinTheAdmissionBudget::Stopped(stop)) => {
                return RelocationBatchAfterTheFloorRaises::NotPublished(
                    RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::NoSpaceAfterRaisingTheFloor {
                        last_refusal: Box::new(refusal_of_the_relocation),
                        stop,
                        floor_raises,
                    },
                );
            }
            Err(cause) => {
                return RelocationBatchAfterTheFloorRaises::NotPublished(
                    RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::FloorRaiseFailedWhilePushingForSpace {
                        cause: Box::new(cause),
                        refusal_that_pushed_the_failed_raise: Box::new(refusal_of_the_relocation),
                        floor_raises,
                    },
                );
            }
        }
    }
}

/// 在新开的写入口上发一次另发的那一批（写入口的容器容量取分配器上装的开关，同 [`publish_the_compaction_step_once`]）。
///
/// # Errors
/// 同 `transaction::publish_the_relocations_and_the_repair_of_the_exempt_units`。
fn publish_the_relocation_batch_once<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    planned: &[PlannedRelocation],
    repair: &RepairOfTheExemptUnits,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, RelocationBatchRefused> {
    let switches = allocator.deletion_switches();
    let mut writer = PoolWriter::new(parameters, devices);
    writer.set_container_tree_capacities(switches.container_tree_capacities());
    publish_the_relocations_and_the_repair_of_the_exempt_units(
        &mut writer,
        allocator,
        previous,
        planned,
        repair,
        instance,
    )
}

/// 只供测试的开关造的那一拒（`compaction_plan::RelocationBatchAfterTheChangeRefusedForSpace`）：空间准入不够那一种，一块不够的盘都不点名——
/// 没有真算准入，只为把另发那一批强制进「被空间拒」那一支；在这一批的写入口建起之前交回，一个写都没发。
fn space_refusal_of_the_relocation_forced_for_tests() -> RelocationBatchRefused {
    RelocationBatchRefused::Publish(PublishError::SpaceAdmissionRefused(
        AdmissionRefusedOnSomeDevices {
            short_devices: Vec::new(),
        },
    ))
}

/// 只供测试的开关造的那一拒（`compaction_plan::RelocationBatchAfterTheChangeRefusedNotForSpace`）：计划对不上那一版那一种，点名计划里头一把 key——
/// 没有真核计划，只为把另发那一批强制进「被拒、不是空间不够」那一支；在这一批的写入口建起之前交回，一个写都没发。
/// 这一批只换豁免映射的单元、计划是空的时点名的 key 是空的。
fn refusal_not_for_space_of_the_relocation_forced_for_tests(
    planned: &[PlannedRelocation],
) -> RelocationBatchRefused {
    RelocationBatchRefused::PlannedUnitNotInTheVersion {
        mapping_key: planned
            .first()
            .map(|relocation| relocation.mapping_key.clone())
            .unwrap_or_default(),
    }
}

/// 只供测试的开关造的那一错（`compaction_plan::FloorRaiseForTheRelocationBatchAfterTheChangeFails`）：现行那一版的实例表读不出——
/// [`push_one_floor_raise_within_the_admission_budget`] 在任何写之前交得出的一种；没有真读盘，只为把另发那一批强制进「抬 F 自己报错」那一支，一个写都没发。
fn floor_raise_failure_of_the_relocation_forced_for_tests() -> MountError {
    MountError::InstanceTableUnreadable
}
