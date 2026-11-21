//! 三样读者下判退（D23（journal 的角色与格式） 已定项 14 F2 段）：失败归哪一类、所选那一版在三样读者下各读一遍、
//! 退到系统配置见证过的那一版里最新的根。

use super::*;

/// 一处读者读一版没读通，失败是哪一类（C866（F2 退回不分字节丢了与内容拒收） 改法甲，判决
/// `research/prompts/m3-close-c693-r3-main-verification.md` 第二节「㊁ 实现改法甲」那一格：F2 只为字节丢了退）。封闭集合，`match` 不写通配臂。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailureKindForTheFallBack {
    /// 字节丢了：一个单元两条位置条目都读不到校验和相符的那份、经中央映射仍读不出，或这一版要的那条记录环里一条都读不出。
    BytesLost,
    /// 内容拒收：字节读到了（整单元校验和对得上），读者不收它的内容（解不开、flags 未知位、条目宽不对、判不过、形状不合……）。
    ContentRefused,
}

impl RecoveryFailure {
    /// 这一个失败在 F2 判退里算哪一类（[`FailureKindForTheFallBack`]）：只有「两条位置条目都读不到校验和相符的那份」
    /// （[`Self::UnitUnreadable`]）与「经映射仍读不出」（[`Self::MappingStillUnreadable`]）是字节丢了，别的成员都是读到了字节、读者不收。
    /// 系统配置、根环、重放那几个成员走读一版的读者报不出来，照内容拒收算（判退走不到它们）。
    #[must_use]
    pub fn failure_kind_for_the_fall_back(&self) -> FailureKindForTheFallBack {
        match self {
            RecoveryFailure::UnitUnreadable { .. }
            | RecoveryFailure::MappingStillUnreadable { .. } => FailureKindForTheFallBack::BytesLost,
            RecoveryFailure::NoValidSystemConfiguration { .. }
            | RecoveryFailure::SystemConfigurationIncompatBitsNotRecognized { .. }
            | RecoveryFailure::SystemConfigurationsDisagree
            | RecoveryFailure::NoDeviceInThePool
            | RecoveryFailure::RootRingSlotsPerRegionOutOfRange { .. }
            | RecoveryFailure::SystemConfigurationValueRefused { .. }
            | RecoveryFailure::DeviceEndsBeforeTheUnitAreaStart { .. }
            | RecoveryFailure::NoValidRoot
            | RecoveryFailure::UnitMalformed { .. }
            | RecoveryFailure::InvariantViolated { .. }
            | RecoveryFailure::EntryWidthDiffersFromItsFieldTable { .. }
            | RecoveryFailure::InodeNumberWatermarkRowMissingFromTheAccountingTree
            | RecoveryFailure::AllocationRecordOutsideThePoolGeometry { .. }
            | RecoveryFailure::MappingMiss { .. }
            | RecoveryFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
                ..
            } => FailureKindForTheFallBack::ContentRefused,
        }
    }
}

impl RebuildVersionFailure {
    /// 建上一版那一样读者没读通是哪一类：走读同款的失败照 [`RecoveryFailure::failure_kind_for_the_fall_back`]；
    /// 这一版下面有文件、环里一条记录都读不出来顶着它（[`Self::NoRecordStandingForFileVersion`]）算字节丢了——
    /// D23（journal 的角色与格式） 已定项 14「末条记录读不出的那一条也算」。
    #[must_use]
    pub fn failure_kind_for_the_fall_back(&self) -> FailureKindForTheFallBack {
        match self {
            RebuildVersionFailure::Walk(failure) => failure.failure_kind_for_the_fall_back(),
            RebuildVersionFailure::NoRecordStandingForFileVersion => {
                FailureKindForTheFallBack::BytesLost
            }
        }
    }
}

/// 所选那一版在三样读者（恢复的冷走读、只读挂载的打开、可写挂载的建上一版连同读它的实例表）下各读一遍，判退要的那一样
/// （D23（journal 的角色与格式） 已定项 14「三处判退不退看同一个并集」，C755（恢复与两种挂载触发退回的失败不同）；
/// 与 C866（F2 退回不分字节丢了与内容拒收） 改法甲合在一起：并集里有内容拒收就不退）。封闭集合，`match` 不写通配臂。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectedVersionUnderTheThreeReaders {
    /// 三样都读通（树表 0 条的一版打开那一步报的「还没发布过文件版本」也算读通：它不是读坏）。
    EveryReaderGetsThrough,
    /// 有一样没读通，没读通的每一样都是字节丢了：F2 判退的那一个触发。
    SomeReaderLostBytesAndNoReaderRefusedContent,
    /// 有一样拒收了内容：不退（C866 改法甲）。
    SomeReaderRefusedContent,
}

/// 三样读者在 `selected_version` 上各读一遍（只读，不写盘），按 [`SelectedVersionUnderTheThreeReaders`] 归。每样读者各自停在它第一个失败上，
/// 并集看的就是这三个失败。建上一版要的那条记录照可写挂载的取法从 `records` 里取（`mount::record_standing_for_the_version`）。
#[must_use]
pub fn selected_version_under_the_three_readers(
    reader: &dyn PoolReader,
    system_configuration: &SystemConfiguration,
    selected_version: &RootRecord,
    records: &BTreeMap<(InstanceGeneration, u64), JournalRecord>,
) -> SelectedVersionUnderTheThreeReaders {
    let unit_area_start = unit_area_start_of_the_chosen_system_configuration(system_configuration);
    let mut mapping_fallbacks_of_the_judgement = 0;
    let failure_of_the_walk = walk_to_file_in_the_unit_area_starting_at(
        reader,
        unit_area_start,
        selected_version,
        &mut mapping_fallbacks_of_the_judgement,
    )
    .err()
    .map(|failure| failure.failure_kind_for_the_fall_back());
    let failure_of_the_opening = crate::mounted_read::open_pool_for_read(reader, selected_version)
        .err()
        .and_then(|failure| failure.failure_kind_for_the_fall_back());
    let failure_of_the_rebuild = rebuild_version_in_the_unit_area_starting_at(
        reader,
        unit_area_start,
        selected_version,
        crate::mount::record_standing_for_the_version(records, selected_version),
    )
    .err()
    .map(|failure| failure.failure_kind_for_the_fall_back());
    let failure_of_the_instance_table_of_the_rebuild =
        instance_table_chain_of_root(reader, selected_version)
            .err()
            .map(|failure| failure.failure_kind_for_the_fall_back());
    let failures = [
        failure_of_the_walk,
        failure_of_the_opening,
        failure_of_the_rebuild,
        failure_of_the_instance_table_of_the_rebuild,
    ];
    if failures.contains(&Some(FailureKindForTheFallBack::ContentRefused)) {
        return SelectedVersionUnderTheThreeReaders::SomeReaderRefusedContent;
    }
    if failures.contains(&Some(FailureKindForTheFallBack::BytesLost)) {
        return SelectedVersionUnderTheThreeReaders::SomeReaderLostBytesAndNoReaderRefusedContent;
    }
    SelectedVersionUnderTheThreeReaders::EveryReaderGetsThrough
}

/// 三处判退的同一套交回：退到哪一版，与交给判法的那一份记录（触发之后重扫的那一遍）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FallBackJudgedOnTheThreeReaders {
    pub version_fallen_back_to: RootRecord,
    /// 触发之后重扫 journal 环的那一份（D23（journal 的角色与格式） 已定项 14：三处交给判法的记录是同一种来源；可写挂载里 F2、
    /// 建退到那一版要的那条记录与 F1 再判同用这一遍）。
    pub records_rescanned_after_the_trigger: BTreeMap<(InstanceGeneration, u64), JournalRecord>,
}

/// 恢复、只读挂载、可写挂载三处判退共用的这一份（D23（journal 的角色与格式） 已定项 14 F2 段，C755 用户 2026-10-20 定「三句都补」，
/// C866 改法甲）：所选那一版 `selected_version`（一条记录都没施加时就是 `chosen_root`）
/// ① 系统配置没见证它（照 `records_of_the_site`——这一处已经扫过的那一份——问一次 F2 判法，交不出一条可退的根就不必读三样读者）；
/// ② 三样读者的并集里有一样字节丢了、没有一样拒收内容（[`selected_version_under_the_three_readers`]）；
/// ③ 触发之后重扫一遍 journal 环，拿重扫的那一份再问一次 F2 判法
/// （[`newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to`]）——三样都成立才交回退到的那一版。
/// 并集只用来判退、不判成败：退到的那一版各处照自己的读者读，读不通就留在所选的那一版（调用方做）。
/// ①用这一处已经扫过的那一份先挡一下，是为了见证过的一版（平时就是这样）不多读三样读者、不多扫一遍环；
/// 它与③那一遍判出来的不同只在两次扫环之间有一次瞬时读错的时候（条款没写这一格，交主 agent）。
#[must_use]
pub fn fall_back_judged_on_the_three_readers(
    reader: &dyn PoolReader,
    system_configuration: &SystemConfiguration,
    chosen_root: &RootRecord,
    selected_version: &RootRecord,
    records_of_the_site: &BTreeMap<(InstanceGeneration, u64), JournalRecord>,
) -> Option<FallBackJudgedOnTheThreeReaders> {
    newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to(
        reader,
        system_configuration,
        chosen_root,
        selected_version,
        records_of_the_site,
    )?;
    match selected_version_under_the_three_readers(
        reader,
        system_configuration,
        selected_version,
        records_of_the_site,
    ) {
        SelectedVersionUnderTheThreeReaders::SomeReaderLostBytesAndNoReaderRefusedContent => {}
        SelectedVersionUnderTheThreeReaders::EveryReaderGetsThrough
        | SelectedVersionUnderTheThreeReaders::SomeReaderRefusedContent => return None,
    }
    let records_rescanned_after_the_trigger = scan_journal(reader, system_configuration);
    let version_fallen_back_to =
        newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to(
            reader,
            system_configuration,
            chosen_root,
            selected_version,
            &records_rescanned_after_the_trigger,
        )?;
    Some(FallBackJudgedOnTheThreeReaders {
        version_fallen_back_to,
        records_rescanned_after_the_trigger,
    })
}

/// 所选那一版走读失败之后退回哪一版（F2 乙，用户 2026-10-07 定「三处都做」；判决 `research/prompts/m3-c694-r2-main-verification.md`
/// 第三节第 7 条）：系统配置没见证过所选那一版时，退到 [见证到的那一版, 所选根) 里读得出、不被所选根的实例表抛弃的最新一条根——交回的不一定是
/// 见证到的那一版本身，是这一段里最新的那条（`F2_NEWEST`，
/// 判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 H4 那一行与第三节第 1 条，用户 2026-10-08 定「四形都修」）——
/// 见证到的那一版 = 根环里 (checkpoint_txg, 实例代号) 与计数器 = c_见证 的那条记录相等的那一版；不退到比它更旧的根（`F2_EXACT`，
/// 判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T5 那一行与第三节第 1 条：那样已确认的那一版根槽读回全零时会退过它、
/// 记录读不出时会退到被新实例抛弃了的根），也不停在它而跳过它之后一条已确认、读得出的根（P 那次轮换写的较新系统配置槽读回全零时 c_见证 落回 P 之前）。
/// 所选根的实例表读不出时只认与见证到的那一版恰好相等的那条，见证到的那一版（读得出的根）与所选根的实例表指针相同时另收区间里与所选根同实例、
/// 读得出的根（C751，用户 2026-10-20 定「甲」）；c_见证 落在所选那次发布自己的某条记录上时那一段是空的、不退（所选那一版算见证过）。
/// 退到的那一条与所选根之间有一次发布带末条标志的记录读得出、根读不出、不被所选根的实例表抛弃时也不退（攻方
/// `research/prompts/m3-batch37-43-code-r1-opus-output.md` 第二节 Y4，用户 2026-10-08 定「两形都修」）：退过去会越过那次写齐了记录的发布。
/// 见证过、见证判不出、c_见证 = 0（环里读得出计数器 0 的记录也一样）、那条记录读不出、那一段里没有读得出、不被抛弃的根、或退过去会越过上面那样一次发布时
/// `None`（照报原来的失败）。
/// 恢复（[`recover`]）、可写挂载读阶段之后建上一版那一步（`mount`）、只读挂载择根
/// （`mounted_read::mount_read_only`）三处都调这一份，退过去之后不重放、直接走那一版。所选那一版是施加过记录、由记录重建的
/// （`selected_version` 不是 `chosen_root`）时不退：条款写的是「所选根走读失败」，施加过记录的那一版走不通条款没写（函数体第一句）。
///
/// 「见证过」的判法取攻方 `strict`（判决同一条）：c_见证 = 每块盘两槽里自证过的系统配置 journal tail 的最大值（与可写挂载判据 N-配置
/// 同一个读数，`mount::newer_publish_witness`）。所选那一版那次发布带末条标志的记录读得出（任一份）⇒ 它的计数器 ≤ c_见证 即见证过；
/// 读不出而计数器 = c_见证 的那条记录读得出 ⇒ 它的 (checkpoint_txg, 实例代号) 不比所选那一版旧即见证过；两条都读不出当见证过、不退。
/// 退到哪一条：上面那一段里最新的那条。根环里的根按择根同一个序（checkpoint_txg 为主、实例代号破平局）比。
///
/// 系统配置见证判不出（R2：有槽设备报读错、又没有一块盘两槽都自证过，[`system_configuration_witness_is_undecidable`]）时不退：
/// 条款只写了「系统配置没见证它时退」，判不出算不算没见证没写；这一支照报原来的失败（与今天相同），交主 agent 定。
///
/// 代价（判决同一条，用户认下）：已确认的一版在较新的系统配置槽与它的单元同时读回全零时，c_见证 落回它之前，读回的是上一版。
#[must_use]
pub fn newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to<
    Reader: PoolReader + ?Sized,
>(
    reader: &Reader,
    system_configuration: &SystemConfiguration,
    chosen_root: &RootRecord,
    selected_version: &RootRecord,
    records: &BTreeMap<(InstanceGeneration, u64), JournalRecord>,
) -> Option<RootRecord> {
    // 条款写的是「所选根走读失败」：只在走读失败的就是所选根本身（一条记录都没施加）时退。施加过记录、由记录重建的那一版走读失败不退：
    // 施加之前它点名的单元两份都验过（前缀第四条），走不通说明记录或单元的内容坏了，不是被后来的写盖掉；那一支条款没写，照报原来的失败（与今天相同），交主 agent 定。
    if (selected_version.instance, selected_version.checkpoint_txg)
        != (chosen_root.instance, chosen_root.checkpoint_txg)
    {
        return None;
    }
    let slot_spacing_in_bytes = u64::from(
        system_configuration
            .immutable
            .sizes
            .fixed_structure_slot_spacing,
    );
    let slots_of_each_device: Vec<SystemConfigurationSlotsOfOneDevice> = reader
        .device_identities()
        .into_iter()
        .map(|device| {
            system_configuration_slots_of_one_device_read_once(
                reader,
                device,
                slot_spacing_in_bytes,
                &system_configuration.immutable.filesystem_identifier,
            )
        })
        .collect();
    if system_configuration_witness_is_undecidable(&slots_of_each_device) {
        return None;
    }
    let witnessed_journal_counter = slots_of_each_device
        .iter()
        .flat_map(|slots_of_this_device| slots_of_this_device.self_verified.iter())
        .map(|slot| slot.quantities.journal_tail)
        .max()
        .unwrap_or(0);
    // c_见证 = 0 不退（D23（journal 的角色与格式） 已定项 14「c_见证 = 0 … 都不退」）：写者的计数器从 1 起
    // （`journal::FIRST_JOURNAL_COUNTER`），环里读到计数器 0 的记录只能是坏盘或伪造的，不拿它当见证到的那一版。
    if witnessed_journal_counter == 0 {
        return None;
    }
    let order_of_the_root_choice = |root: &RootRecord| (root.checkpoint_txg, root.instance);
    let last_record_counter_of_the_selected_version = records
        .values()
        .filter(|record| {
            record.instance == selected_version.instance
                && record.checkpoint_txg == selected_version.checkpoint_txg
                && record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
        })
        .map(|record| record.counter)
        .max();
    let record_at_the_witnessed_counter = records
        .values()
        .filter(|record| record.counter == witnessed_journal_counter)
        .map(|record| (record.checkpoint_txg, record.instance))
        .max();
    let is_witnessed = match (
        last_record_counter_of_the_selected_version,
        record_at_the_witnessed_counter,
    ) {
        (Some(last_record_counter), _) => last_record_counter <= witnessed_journal_counter,
        (None, Some(witnessed_publish)) => {
            witnessed_publish >= order_of_the_root_choice(selected_version)
        }
        (None, None) => true,
    };
    if is_witnessed {
        return None;
    }
    // 计数器 = c_见证 的那条记录读不出就不退。
    let witnessed_publish = record_at_the_witnessed_counter?;
    let chosen = order_of_the_root_choice(chosen_root);
    // 退到 [见证到的那一版, 所选根) 里读得出、不被所选根的实例表抛弃的最新一条根（`F2_NEWEST`）。c_见证 落在所选那次发布自己的某条记录上时
    // 这一段是空的：所选那一版算见证过、不退。所选根的实例表读不出时滤不了，只认与见证到的那一版恰好相等的那条（`F2_EXACT` 那一取法），
    // 外加 C751 甲那一收（见下）。
    let instance_table_of_the_chosen_root = instance_table_chain_of_root(reader, chosen_root)
        .ok()
        .map(|chain| chain.records);
    // 读坏的槽一律重读一次（C563 第 4 处，用户 2026-10-22 定「甲」）：一次瞬时读坏不把做得成的退回变成拒；重读仍坏当没有根，交下面那一关判。
    let readable_roots_in_the_ring = readable_roots_rereading_every_bad_ring_slot_once(
        reader,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );
    // 不跳过一次末条读得出、根读不出、不被所选根的实例表抛弃的发布（攻方 `research/prompts/m3-batch37-43-code-r1-opus-output.md` 第二节 Y4，
    // 臂 `OPUS_ARM_F2_NO_SKIP=1`，用户 2026-10-08 定「两形都修」）：退到的那一版与所选根之间有这样一次发布，说明那一版之后还有一次写齐了记录的
    // 发布（多半已确认），只是这一刻它的根读不出——退过去就悄悄越过它（较新的系统配置槽也读回全零时 c_见证 落回它之前，下界护不住它）。
    // 照报原来的失败，等故障撤掉再读。只看内存里扫出的记录与读出的根，不另读盘。所选根的实例表读不出时滤不了，那次发布算不被抛弃
    // （条款没写这一格；照攻方那条臂取，交主 agent）。那次发布的根读得出不算（它在区间里读得出又不被抛弃就会被选中；读得出而被抛弃，跳过它本来就对）。
    // 所选根的实例表读不出时，见证到的那一版与所选根共用同一个实例表指针，才另收区间里与所选根同实例、读得出的根，当它不被抛弃
    // （C751（只读挂载在实例表读不出时越过已确认的版本），用户 2026-10-20 定「甲：共用实例表指针才收」；两个指针都在根记录里，不读表）。
    // 见证到的那一版的根读不出就比不了指针，不另收。
    let witnessed_version_shares_the_instance_table_pointer_of_the_chosen_root =
        readable_roots_in_the_ring.iter().any(|root| {
            order_of_the_root_choice(root) == witnessed_publish
                && root.instance_table == chosen_root.instance_table
        });
    let newest_root_in_the_range = readable_roots_in_the_ring
        .iter()
        .copied()
        .filter(|root| {
            let order = order_of_the_root_choice(root);
            let kept_by_the_instance_table_of_the_chosen_root =
                match &instance_table_of_the_chosen_root {
                    Some(table) => !root_is_abandoned_by_the_instance_table(root, table),
                    None => order == witnessed_publish
                        || (witnessed_version_shares_the_instance_table_pointer_of_the_chosen_root
                            && root.instance == chosen_root.instance),
                };
            witnessed_publish <= order
                && order < chosen
                && kept_by_the_instance_table_of_the_chosen_root
        })
        .max_by_key(order_of_the_root_choice)?;
    let skips_a_publish_whose_last_record_reads_while_its_root_does_not =
        records.values().any(|record| {
            let order = (record.checkpoint_txg, record.instance);
            let abandoned_by_the_instance_table_of_the_chosen_root =
                instance_table_of_the_chosen_root
                    .as_ref()
                    .is_some_and(|table| {
                        publish_is_abandoned_by_the_instance_table(
                            record.instance,
                            record.checkpoint_txg,
                            table,
                        )
                    });
            record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
                && order_of_the_root_choice(&newest_root_in_the_range) < order
                && order < chosen
                && !abandoned_by_the_instance_table_of_the_chosen_root
                && !readable_roots_in_the_ring
                    .iter()
                    .any(|root| order_of_the_root_choice(root) == order)
        });
    if skips_a_publish_whose_last_record_reads_while_its_root_does_not {
        return None;
    }
    Some(newest_root_in_the_range)
}
