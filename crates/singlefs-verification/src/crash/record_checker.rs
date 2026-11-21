//! 记录核对器：照持久集合核每次发布的记录还在不在，与它判复用、回退下限要读的几样。

use super::*;

/// 记录核对器（D13（验证路线） 已定项 7，故意不给它编号；入参 (崩溃前镜像, 记录流, 崩溃后镜像, 持久集合)）：崩溃前镜像是 `image.base`、
/// 记录流是 `image.writes`、崩溃后镜像是 `image` 本身、持久集合是 `image.persisted`（枚举器给的这个崩溃状态里哪些写落了盘，
/// 与 `image.writes` 逐条对应——四样都从同一个崩溃状态里取，对不上的组合写不出来）。崩溃点重放只跑只读恢复、不改盘，
/// 崩溃后镜像的两份（崩溃态、实现恢复后）是同一份。第一条判据里一处写「在盘上」= 崩溃后镜像那个位置上的字节与记录流里写下的
/// 逐字节相同；第二条判据按扇区、拿持久集合判一份单元副本缺不缺席（[`check_records_against`]）。
/// 不经恢复代码、不解析树。`effective_root` 取恢复报出来的实际走的那条根。
#[must_use]
pub fn check_records(
    image: &CrashImage<'_>,
    effective_root: Option<(InstanceGeneration, CheckpointTxg)>,
) -> RecordCheck {
    check_records_against(
        image,
        image,
        image.writes,
        &image.persisted,
        RecordStreamContinuity::OneRecording,
        effective_root,
    )
}

/// 同一条判据，两份崩溃后镜像、被核的记录流与它的持久集合分开给（`persisted` 与 `writes` 逐条对应；`continuity` 说这张写表是
/// 一条录制流，还是在掉电、崩溃那几道接缝处断开的几截录制流，[`publishes_in`] 按它分发布）。
///
/// 崩溃后镜像是两份（D13（验证路线） 已定项 7）：`crash_state_image` 是崩溃态镜像，判择根与前缀——第一条判据里根槽写与记录写
/// 「在盘上」看它；`recovered_image` 是实现恢复后的镜像，判在不在——第二条判据里单元副本缺不缺席、恢复落到的那一版的实例表
/// 都从它读（实现的恢复会改盘：可写挂载写行、暖机，恢复落到的那一版可以是恢复自己写出的，只在恢复后的镜像上）。
/// 只读恢复不改盘，两份是同一份。
///
/// 装置里各处交什么（用例另有直接调它的）：
/// - 崩溃点重放（[`check_records`]）：两份镜像同一份；写表是枚举用的那一张（录制流里的写在前，原地覆写的撕裂镜像与重放接在后面，
///   见 [`CrashImage`]），一条录制流。
/// - 崩溃注入第一截（增补 3 第 3 件，`crash_injection.rs`）：两份镜像都是「更早的段整段持久 + 当前段一个子集」那份崩溃镜像；
///   写表是这段历史的整条录制流——`persisted` 里更早的段整段持久、当前段按这个崩溃点的子集、更晚的段一个都没持久；
///   恢复落到由记录重建的一版时，那一版的根槽写在更晚的段里，也在这张表里。这个崩溃状态之前掉过电的，在每道掉电接缝处断开
///   （`RecordStreamSeam::AfterAPowerLoss`：掉电之前那次改动落了盘、没有根的单元写不归掉电之后的第一次发布），没掉过电的是一条录制流。
/// - 崩溃注入第二截（崩溃后镜像上可写挂载、再发一次布之后的池）：崩溃态镜像是第一截那份崩溃镜像，恢复后的镜像是挂载与那次发布
///   之后的池；写表是整条历史录制流接上挂载那一段录制流、再接上那次发布的录制流，在历史与挂载的接缝处断开（第一截那几道掉电接缝照给）——历史那一段的持久集合
///   同第一截，挂载与那次发布全落了；恢复自称的那一版（`effective_root`）取挂载与那次发布写出的最新那条根，挂载一条根都没写出时
///   退回第一次崩溃落到的那一版（D13（验证路线） 已定项 7 的字面；`crate::crash_injection::newest_root_written_after_the_crash` 取它，
///   给定状态表那一路由表里挂载之后的池那个状态那一栏带到判段，`crate::crash_given_states::ClaimedVersionOfAGivenState`）。
/// - 崩溃注入第三截（挂载途中的二次崩溃）：两份镜像都是二次崩溃镜像；写表同第二截，挂载那一段的持久集合按二次崩溃——
///   更早的段整段持久、当前段按子集、更晚的段一个都没持久，之后那次发布一个都没写到。历史那几次发布在二次崩溃镜像上还在不在也核得到。
///
/// 第一条判据只在随机历史上才遇得到的限定：一次发布一条 journal 记录都没写过时不判「根在而记录一条都不在」（记录流本来就是空的，不是有洞）。
///
/// 第二条判据（恢复自称的 txg ≥ 某次发布、那次发布的某个单元全部副本都缺席）按 C561（记录核对器的复用豁免在复用只落一半时假红）
/// 的结论判一份副本缺席：**它区间里有一个扇区在持久集合下不是它的字节，且这个扇区上没有一次更晚、在持久集合里、过了回收谓词的写**
/// （`SecPers513`，`research/prompts/m2-layer0-scale-r3-main-verification.md` 第四节的乙）。
/// 「在持久集合下不是它的字节」是崩溃后镜像那个扇区上的字节与这份副本写下的不同（崩溃后镜像就是基线加持久集合里的写）；
/// 「解释」那个扇区的只认持久集合里的写，不认盘上字节恰好相同的写——只看盘上字节分不开「写没落、而那里恰好是零」与「写落了、
/// 露在外面的是它的零尾」（C561 那一行的状态 Y 与假红状态崩溃镜像逐字节相同）。
/// - 按扇区，不按整份：跨度不对齐的两代复用（两个 16K 节点 → 一个 32K 数据单元 → 两个 16K 节点）只落一半时，先一代副本的每个扇区
///   要么还是它的、要么被一次落了的后写合法盖住，这份副本不算缺席（C561 的假红）；整份在位的后写只盖住它一半、另一半不是它的
///   而又没有落了的写解释时算缺席（按整份开脱的那一版在这里假绿）。
/// - 更晚那次写没落盘时不解释（C507（记录核对器的复用豁免比登记的候选宽，把真洞变哑））；那次复用证得出违反回收谓词的也不解释
///   （C513（复用豁免不判那次复用合不合法），判据与剩下的盲点写在 `reuse_is_not_proven_illegal_by_the_reclaim_predicate` 上）。
///
/// 第二条判据不罩被抛弃时间线上的发布（实八，实七报告 (c)）：一次发布 (i, T) 按恢复落到的那一版的实例表判为被抛弃
/// （有行 (i, Ti, Wi) 且 T > Ti，D23（journal 的角色与格式） 已定项 14 回退段的候选集规则）时，它写出的单元不算「该在」——
/// 那条时间线被切掉之后，它的单元可以被合法复用。恢复落到的那一版的实例表按 [`instance_table_of_the_effective_root`] 读；
/// 读不出时一次发布都不算被抛弃（判据不放宽）。
///
/// # Panics
/// `persisted` 与 `writes` 不一样长；接缝的下标越过写表；单元副本的长度不是整扇区。
#[must_use]
pub fn check_records_against<
    CrashStateImage: PoolReader + ImageReader,
    RecoveredImage: PoolReader + ImageReader,
>(
    crash_state_image: &CrashStateImage,
    recovered_image: &RecoveredImage,
    writes: &[RetainedWrite],
    persisted: &[bool],
    continuity: RecordStreamContinuity<'_>,
    effective_root: Option<(InstanceGeneration, CheckpointTxg)>,
) -> RecordCheck {
    assert_eq!(
        persisted.len(),
        writes.len(),
        "持久集合与记录流逐条对应：第 k 项说的是记录流里第 k 次写落没落盘"
    );
    let in_place = |index: usize| {
        let write = &writes[index];
        let length = usize::try_from(write.length_in_bytes()).expect("写长装得进 usize");
        PoolReader::read(crash_state_image, write.device, write.offset, length)
            .is_some_and(|bytes| write.contents.still_on_disk(&bytes))
    };
    let copy_is_missing = |copy: usize| {
        unit_copy_is_missing_under_the_persisted_set(recovered_image, writes, persisted, copy)
    };
    let instance_table_of_the_landed_version =
        instance_table_of_the_effective_root(recovered_image, writes, effective_root);
    let abandoned_by_the_landed_version = |publish: &PublishWrites| {
        instance_table_of_the_landed_version
            .as_ref()
            .is_some_and(|table| table.abandons(publish.instance, publish.checkpoint_txg))
    };
    let mut check = RecordCheck::default();
    for publish in publishes_in(writes, persisted, continuity) {
        if !publish.records.is_empty()
            && in_place(publish.root)
            && !publish.records.iter().any(|record| in_place(*record))
        {
            check.root_without_record = true;
        }
        if effective_root.is_some_and(|(_, txg)| txg.0 >= publish.checkpoint_txg)
            && !abandoned_by_the_landed_version(&publish)
        {
            let mut copies_by_offset: BTreeMap<u64, Vec<usize>> = BTreeMap::new();
            for unit in &publish.units {
                copies_by_offset
                    .entry(writes[*unit].offset.0)
                    .or_default()
                    .push(*unit);
            }
            if copies_by_offset
                .values()
                .any(|copies| copies.iter().all(|copy| copy_is_missing(*copy)))
            {
                check.claimed_state_missing_unit = true;
            }
        }
    }
    check
}

/// 恢复落到的那一版（`effective_root`）的实例表，从 `reader`（实现恢复后的镜像）上沿链读：那一版的根记录取记录流里写出这个根身份的
/// 最后一次根槽写的字节（同一实例里实例表不重写，重建的根的实例表指针照所选根，D23（journal 的角色与格式） 已定项 15，
/// 与这次根槽写带的是同一个）。根槽没落盘、由记录重建的那一版，它的根槽写在截断处或二次崩溃的当前段之后：崩溃点重放与崩溃注入三截
/// 交的都是整条流（第二、三截是整条历史录制流接上整条挂载流），都在表里（见 [`check_records_against`]）。
/// 同一个根身份在表里出现两次时（第二、三截里历史截断处之后那一截与挂载撞了身份），挂载的接在后面，取到的是盘上那一条。
/// 记录流里没有这个根身份（基镜像里的根，如 mkfs 的第 0 代根，实例表一行都没有）、恢复没落到任何一版、链读不出，都是 None。
fn instance_table_of_the_effective_root<Reader: PoolReader + ImageReader>(
    reader: &Reader,
    writes: &[RetainedWrite],
    effective_root: Option<(InstanceGeneration, CheckpointTxg)>,
) -> Option<InstanceTableOfRootRecord> {
    let effective_root = effective_root?;
    let root_record = writes
        .iter()
        .rev()
        .filter(|write| write.kind == StepKind::RootRecordFua)
        .find(|write| root_identity_of_write(write) == effective_root)?
        .bytes()
        .expect("根槽 FUA 写是普通写，带着字节");
    InstanceTableOfRootRecord::read(reader, root_record)
}

/// 记录核对器第二条判据的缺席判定（判据见 [`check_records_against`]）：写表下标 `copy` 那份单元副本，在 `reader`（崩溃后镜像）与
/// `persisted`（持久集合）下缺不缺席。逐扇区看：扇区上还是它的字节就过；不是的，要有一次更晚、在持久集合里、落在这个扇区上、
/// 过了回收谓词的写解释它，一个扇区解释不了就是缺席。读不出这块盘（池里没有它）按每个扇区都不是它的字节算。
fn unit_copy_is_missing_under_the_persisted_set(
    reader: &dyn PoolReader,
    writes: &[RetainedWrite],
    persisted: &[bool],
    copy: usize,
) -> bool {
    let copy_write = &writes[copy];
    let length = usize::try_from(copy_write.length_in_bytes()).expect("写长装得进 usize");
    let sector_bytes = usize::try_from(SECTOR_BYTES).expect("512");
    assert!(
        length.is_multiple_of(sector_bytes),
        "单元副本是整扇区的写（写表下标 {copy}，{length} 字节）"
    );
    let on_disk = reader.read(copy_write.device, copy_write.offset, length);
    let earlier_publish_txg = checkpoint_txg_of_the_publish_that_made_the_write(writes, copy);
    // 每次更晚的写过不过回收谓词只算一次：谓词只看这次后写之前的写表与这份副本的发布，与扇区无关。
    let mut reuse_verdict_by_later_write: BTreeMap<usize, bool> = BTreeMap::new();
    for sector_start in (0..length).step_by(sector_bytes) {
        let sector_still_holds_the_copy = on_disk.as_ref().is_some_and(|bytes| {
            copy_write.contents.range_still_on_disk(
                sector_start,
                &bytes[sector_start..sector_start + sector_bytes],
            )
        });
        if sector_still_holds_the_copy {
            continue;
        }
        let sector_offset =
            copy_write.offset.0 + u64::try_from(sector_start).expect("扇区偏移装得进 u64");
        let explained_by_a_persisted_legal_later_write =
            (copy + 1..writes.len()).any(|later_index| {
                let later = &writes[later_index];
                let later_start = later.offset.0;
                let later_end = later_start + later.length_in_bytes();
                persisted[later_index]
                    && later.device == copy_write.device
                    && later_start < sector_offset + SECTOR_BYTES
                    && sector_offset < later_end
                    && *reuse_verdict_by_later_write
                        .entry(later_index)
                        .or_insert_with(|| {
                            reuse_is_not_proven_illegal_by_the_reclaim_predicate(
                                writes,
                                earlier_publish_txg,
                                later_index,
                            )
                        })
            });
        if !explained_by_a_persisted_legal_later_write {
            return true;
        }
    }
    false
}

/// 写表下标 `index` 那次写属于哪次发布：它后面第一条根槽 FUA 写写出的那条根的 checkpoint_txg（与 `publishes_in` 的归法相同）。
/// 后面没有根槽写（这次写属于写表里没做完的那次发布）时 `None`。
pub(crate) fn checkpoint_txg_of_the_publish_that_made_the_write(
    writes: &[RetainedWrite],
    index: usize,
) -> Option<CheckpointTxg> {
    writes[index + 1..]
        .iter()
        .find(|write| write.kind == StepKind::RootRecordFua)
        .map(|root_write| root_identity_of_write(root_write).1)
}

/// 根记录里回退下界 F 的偏移：magic 4 + fsid 16 + flags 4 + 实例代号 4 + checkpoint_txg 8，再过树表指针（节点指针 86）
/// 与树 ID 水位 8（D22（单元原子性怎么合成） 已定项 7 的字段表；`singlefs_checker::check_root_slot` 读的是同一个偏移）。
const ROOT_RECORD_ROLLBACK_FLOOR_OFFSET: u64 = 36 + NODE_POINTER_BYTES + 8;

/// 一次落在根环里的写写下的回退下界 F（8 字节，偏移见 [`ROOT_RECORD_ROLLBACK_FLOOR_OFFSET`]）。
fn rollback_floor_written_by(bytes: &[u8]) -> CheckpointTxg {
    let offset = usize::try_from(ROOT_RECORD_ROLLBACK_FLOOR_OFFSET).expect("130");
    CheckpointTxg(u64::from_le_bytes(
        bytes[offset..offset + 8]
            .try_into()
            .expect("根记录的回退下界 F 占 8 字节"),
    ))
}

/// 写表下标 `later_index` 那次写盖掉 checkpoint_txg 为 `earlier_publish_txg` 的那次发布写下的一份单元，那次复用过不过得了
/// D16（发布语义） 已定项 1 的回收谓词（可再分配 ⟺ 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)）——**只在证得出过不了时交回 false**
/// （C513（复用豁免不判那次复用合不合法））。
///
/// 记录核对器不解析树，谓词里的三个量各取一个往「过得了」那边偏的界，都从写表里这次写之前的那一段取
/// （写这次单元的那一刻，发布是一次接一次做完的，更早的写在实现看来都已落下）：
/// - 释放代 ≥ `earlier_publish_txg + 1`：换下那份单元的发布以写下它的那次发布为祖先，txg 严格更大；从没释放过的比任何界都大。
/// - F_生效 ≤ 这次写之前写表里全部根槽写与系统配置槽写带的 F 的最大值（生效值取根上带的与系统配置里读得出的最大值，
///   D16（发布语义） 已定项 1「生效」，大不过这些写带的全部 F 的最大值；C556（checker 与崩溃点重放不读系统配置里的 F） 的崩溃点重放那一半：
///   抬 F 先写系统配置那一步落了、带新 F 的根一条都没落时，下一次挂载按系统配置里的新 F 回收，写行那次发布就可能复用）。
/// - 环里最旧有效根 ≤ 这次写之前写到根环、还没被更晚的根槽写盖掉的那些根里最小的 txg。这里不按实例表剔被抛弃的根：
///   被抛弃的根的 txg 夹在恢复的所选根与恢复之后的第一条根之间，它比全部有效根都旧时，有效时间线上被复用的单元要么比它还旧
///   （下界过得了它）、要么比它新而它自己的根还在环里（本来就不许复用）。
///
/// 按这三个界判，合法的复用不会被判成过不了；漏的是释放代比 `earlier_publish_txg + 1` 晚得多、又晚过门槛的那一类。
/// 崩溃注入第二、三截的写表（崩溃截断的历史流接上崩溃之后写出的流，`RecordStreamSeam::AfterACrash`）里，
/// 截断处之后那一截历史写没发生过，接缝之后的写取「这次写之前」时照样把它们算进去：那一截的根比截断处之前的 txg 都大、
/// 带的 F 不比之前的小，算进去只会把两个界往「过得了」那边推，不会把合法的复用判成过不了（推的，没造状态量过松了多少）。
/// 前提有两条，写表的来路都满足：基镜像里的根带的 F 为 0（崩溃点重放的基是 mkfs 之后，崩溃注入的基是空池），
/// 写表里没有写失败的根槽写（录制器只录落下了的写；写失败时旧根留在槽里、环不再按 txg 连续，上面那条关于被抛弃根的论证就不成立）。
/// 写表里这次写之前一条根槽写都没有时，环里是什么只有基镜像知道，一律按过得了。
pub(crate) fn reuse_is_not_proven_illegal_by_the_reclaim_predicate(
    writes: &[RetainedWrite],
    earlier_publish_txg: Option<CheckpointTxg>,
    later_index: usize,
) -> bool {
    let Some(earlier_publish_txg) = earlier_publish_txg else {
        return true;
    };
    let mut root_in_each_ring_slot: BTreeMap<(DeviceIdentity, DeviceOffsetInBytes), CheckpointTxg> =
        BTreeMap::new();
    let mut highest_rollback_floor = CheckpointTxg(0);
    for write in &writes[..later_index] {
        match write.kind {
            StepKind::RootRecordFua => {
                let bytes = write.bytes().expect("根槽 FUA 写是普通写，带着字节");
                let (_, checkpoint_txg) = root_identity_written_by(bytes);
                root_in_each_ring_slot.insert((write.device, write.offset), checkpoint_txg);
                highest_rollback_floor =
                    highest_rollback_floor.max(rollback_floor_written_by(bytes));
            }
            // 系统配置槽写是整槽 4096 字节：解得开就取它带的 F；解不开的（写的不是一份自证得过的槽）不抬这个界。
            StepKind::SystemConfigurationSlot => {
                let bytes = write.bytes().expect("系统配置槽写是普通写，带着字节");
                if let Ok(system_configuration) = SystemConfiguration::parse_slot(bytes) {
                    highest_rollback_floor =
                        highest_rollback_floor.max(system_configuration.quantities.rollback_floor);
                }
            }
            StepKind::ZeroFill
            | StepKind::Discard
            | StepKind::UnitWrite
            | StepKind::JournalRecord
            | StepKind::Barrier => {}
        }
    }
    let Some(oldest_root_in_the_ring) = root_in_each_ring_slot.values().min().copied() else {
        return true;
    };
    let release_generation_at_least = earlier_publish_txg.0.saturating_add(1);
    let reclaim_threshold_at_most = highest_rollback_floor.max(oldest_root_in_the_ring);
    release_generation_at_least <= reclaim_threshold_at_most.0
}
