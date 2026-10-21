//! 崩溃点重放（checker 档，D13（验证路线） 已定项 4、15）：拿录制流在内存里重建镜像，屏障与 FUA 按设备切段，
//! 段内每次写各取它的几态、任意组合：原地覆写的写多一态「新旧都读不出」（[`TearableInPlaceOverwrites`]），
//! 其余写的撕裂态并进「没持久」；每个状态跑恢复、oracle、池级 checker 与记录核对器（D13（验证路线） 已定项 7，故意不给编号）。
//! 内存池、保留写、崩溃镜像与两个结果类型在 `singlefs_harness::memory_pool`。

use crate::crash_judge_conclusions::recovery_passes_differ;
use std::collections::BTreeMap;

use std::ops::Range;
use std::sync::mpsc;

use singlefs_checker::image::{ImageReader, InvariantVerdict};
use singlefs_checker::walk::{check_pool_image, InstanceTableOfRootRecord};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::recovery::{
    recover, JournalPolicy, PoolReader, RecoveryOutcome, RecoveryReport,
};
use singlefs_format::NODE_POINTER_BYTES;

use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_harness::memory_pool::{
    newest_persisted_root, publishes_in, root_identity_of_write, root_identity_written_by,
    CrashImage, MemoryPool, PublishWrites, PublishedVersion, PublishedVersionWithoutAFile,
    RecordCheck, RecordStreamContinuity, RetainedWrite, WrittenContents, SECTOR_BYTES,
};
use singlefs_harness::segments::StepKind;

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
/// 的定案判一份副本缺席：**它区间里有一个扇区在持久集合下不是它的字节，且这个扇区上没有一次更晚、在持久集合里、过了回收谓词的写**
/// （崩溃点重放规模第三轮攻方的 `SecPers513`，`research/prompts/m2-layer0-scale-r3-main-verification.md` 第四节用户定的乙）。
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

/// 崩溃点重放的一个崩溃状态按发布归到哪一格（里程碑「覆盖写、释放、回退与复用」步 6 验收第 1 条「每次发布各多少」）。
/// 归法按段：状态所在的那一段往后数，第一次根槽 FUA 写所在的那一段写出的根，就是这个状态归的那次发布——
/// 上一次发布的根槽写之后、这一次发布的根槽写为止，崩在中间的状态都归这一次。上一次发布的系统配置槽轮换与这一次的单元写
/// 之间没有屏障、并在同一段时，那一段整段归这一次：段是枚举的最小单位，一个状态落在哪一段是确定的，落在哪一次写上不是。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CrashPointReplayPublishOfState {
    /// 这次发布的根槽 FUA 写：写表里的下标（按它排就是录制流里的次序）与它写出的根的实例代号、checkpoint_txg。
    UpToTheRootOf {
        root_write_index: usize,
        instance: InstanceGeneration,
        checkpoint_txg: CheckpointTxg,
    },
    /// 写表里最后一次根槽写之后的段（最后那次发布的系统配置槽轮换）：后面没有根槽写可归。
    AfterTheLastRoot,
    /// 每一段都整段持久的那一个状态（闭式 1 + Σ(2^|段| − 1) 里的那个 1）。
    EveryWritePersisted,
}

impl CrashPointReplayPublishOfState {
    /// 计数行里的名字：`instance2_txg5`、`after_the_last_root`、`every_write_persisted`。
    #[must_use]
    pub fn name(self) -> String {
        match self {
            CrashPointReplayPublishOfState::UpToTheRootOf {
                instance,
                checkpoint_txg,
                ..
            } => format!("instance{}_txg{}", instance.0, checkpoint_txg.0),
            CrashPointReplayPublishOfState::AfterTheLastRoot => "after_the_last_root".to_string(),
            CrashPointReplayPublishOfState::EveryWritePersisted => {
                "every_write_persisted".to_string()
            }
        }
    }

    /// 发现日志里的名字：`instance2_txg5_root_write14`、`after_the_last_root`、`every_write_persisted`。
    /// 比 [`Self::name`] 多带根槽写的下标：两块盘各写一条同身份的根时，两格只差这一项，发现表的签名不许把它们并成一格。
    #[must_use]
    pub fn name_with_the_root_write_index(self) -> String {
        match self {
            CrashPointReplayPublishOfState::UpToTheRootOf {
                root_write_index, ..
            } => format!("{}_root_write{root_write_index}", self.name()),
            CrashPointReplayPublishOfState::AfterTheLastRoot
            | CrashPointReplayPublishOfState::EveryWritePersisted => self.name(),
        }
    }
}

/// 每一段归哪次发布（[`CrashPointReplayPublishOfState`] 的归法）：从最后一段往前走，记着「后面最近的那次根槽写」。
#[must_use]
pub fn publish_of_each_segment(
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
) -> Vec<CrashPointReplayPublishOfState> {
    let mut next_root: Option<CrashPointReplayPublishOfState> = None;
    let mut publish_of_segment =
        vec![CrashPointReplayPublishOfState::AfterTheLastRoot; segments.len()];
    for (segment_index, segment) in segments.iter().enumerate().rev() {
        if let Some(root_write_index) = segment.iter().rev().copied().find(|write_index| {
            writes
                .get(*write_index)
                .is_some_and(|write| write.kind == StepKind::RootRecordFua)
        }) {
            let (instance, checkpoint_txg) = root_identity_of_write(&writes[root_write_index]);
            next_root = Some(CrashPointReplayPublishOfState::UpToTheRootOf {
                root_write_index,
                instance,
                checkpoint_txg,
            });
        }
        publish_of_segment[segment_index] =
            next_root.unwrap_or(CrashPointReplayPublishOfState::AfterTheLastRoot);
    }
    publish_of_segment
}

/// 崩溃点重放的 oracle（[`oracle_violation_for_versions`]）判的违例是哪一类：崩溃点重放发现表按它分签名（两遍恢复各判一次）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CrashPointReplayOracleViolationKind {
    /// 恢复没择到根。
    NoRootChosen,
    /// 盘上已持久的最新根槽比恢复实际走的根新。
    RecoveredToAnOlderRoot,
    /// 走读失败。
    ReadFailed,
    /// 走到一个没有版本的根、报没有文件，而更旧的根下面有文件。
    NoFileWhereAnOlderRootHasOne,
    /// 走到的根下面有文件却报没有。
    NoFileWhereThisRootHasOne,
    /// 走到的根下面没有文件却读出了内容。
    FileReadWhereThisRootHasNone,
    /// 读回的内容与走到的根写出的那一版不同。
    WrongContent,
}

impl CrashPointReplayOracleViolationKind {
    /// 每一类各一个：[`Self::from_name`] 在里面找（新加一类要加进来，不然红例行里这一类按名字读不回）。
    pub const EVERY_KIND: [Self; 7] = [
        Self::NoRootChosen,
        Self::RecoveredToAnOlderRoot,
        Self::ReadFailed,
        Self::NoFileWhereAnOlderRootHasOne,
        Self::NoFileWhereThisRootHasOne,
        Self::FileReadWhereThisRootHasNone,
        Self::WrongContent,
    ];

    /// 发现日志与 `CRASH_POINT_REPLAY_FINDING` 行里的名字。
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::NoRootChosen => "no_root_chosen",
            Self::RecoveredToAnOlderRoot => "recovered_to_an_older_root",
            Self::ReadFailed => "read_failed",
            Self::NoFileWhereAnOlderRootHasOne => "no_file_where_an_older_root_has_one",
            Self::NoFileWhereThisRootHasOne => "no_file_where_this_root_has_one",
            Self::FileReadWhereThisRootHasNone => "file_read_where_this_root_has_none",
            Self::WrongContent => "wrong_content",
        }
    }

    /// [`Self::name`] 反过来：不是 [`Self::EVERY_KIND`] 里哪一类的名字交回 `None`。
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::EVERY_KIND
            .into_iter()
            .find(|kind| kind.name() == name)
    }
}

/// oracle 判的一处违例：哪一类、给人看的原因。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrashPointReplayOracleViolation {
    pub kind: CrashPointReplayOracleViolationKind,
    pub reason: String,
}

/// 崩溃点重放发现表里一个签名的一半：判红的是哪一遍、违了哪几条。一个状态几遍都红，就各进各的签名。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CrashPointReplayRedPass {
    /// 看 journal 那一遍恢复过 oracle 判违例（计进 [`CrashPointReplayTally::violations`]）。
    JournalConsultedOracle(CrashPointReplayOracleViolationKind),
    /// 不看 journal 那一遍恢复过 oracle 判违例（计进 [`CrashPointReplayTally::ignored_violations`]）。
    JournalIgnoredOracle(CrashPointReplayOracleViolationKind),
    /// 池级 checker 判违例的不变量，至少一条；次序照 `check_pool_image` 报的次序（它按 checker 的清单 `IMPLEMENTED_INVARIANTS` 报）。
    PoolChecker(Vec<&'static str>),
    /// 记录核对器：两条判据里至少一条成立。
    RecordChecker(RecordCheck),
}

/// 记录核对器第一条判据在发现日志里的名字。
pub const RECORD_CHECKER_ROOT_WITHOUT_RECORD: &str = "root_without_record";
/// 记录核对器第二条判据在发现日志里的名字。
pub const RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT: &str = "claimed_state_missing_unit";
/// 发现日志里 `pass=` 的四个值，依次是 [`CrashPointReplayRedPass`] 的四个成员。
pub const CRASH_POINT_REPLAY_RED_PASS_JOURNAL_CONSULTED_ORACLE: &str = "journal_consulted_oracle";
pub const CRASH_POINT_REPLAY_RED_PASS_JOURNAL_IGNORED_ORACLE: &str = "journal_ignored_oracle";
pub const CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER: &str = "pool_checker";
pub const CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER: &str = "record_checker";

impl CrashPointReplayRedPass {
    /// 发现日志里 `pass=` 的值。
    #[must_use]
    pub fn pass_name(&self) -> &'static str {
        match self {
            Self::JournalConsultedOracle(_) => CRASH_POINT_REPLAY_RED_PASS_JOURNAL_CONSULTED_ORACLE,
            Self::JournalIgnoredOracle(_) => CRASH_POINT_REPLAY_RED_PASS_JOURNAL_IGNORED_ORACLE,
            Self::PoolChecker(_) => CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER,
            Self::RecordChecker(_) => CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER,
        }
    }

    /// 发现日志里 `violated=` 的那几项（逗号拼起来就是值）。
    #[must_use]
    pub fn violated_names(&self) -> Vec<&'static str> {
        match self {
            Self::JournalConsultedOracle(kind) | Self::JournalIgnoredOracle(kind) => {
                vec![kind.name()]
            }
            Self::PoolChecker(invariants) => invariants.clone(),
            Self::RecordChecker(record_check) => {
                let mut names = Vec::new();
                if record_check.root_without_record {
                    names.push(RECORD_CHECKER_ROOT_WITHOUT_RECORD);
                }
                if record_check.claimed_state_missing_unit {
                    names.push(RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT);
                }
                names
            }
        }
    }
}

/// 一个状态在哪一段：段表里的下标，或最后那个每一段都整段持久的状态（不在任何一段里）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CrashPointReplaySegmentOfState {
    Segment(usize),
    AllPersisted,
}

impl CrashPointReplaySegmentOfState {
    /// 发现日志与 `CRASH_POINT_REPLAY_PROGRESS` 行里的写法：段号，或 `all_persisted`。
    #[must_use]
    pub fn name(self) -> String {
        match self {
            Self::Segment(segment_index) => segment_index.to_string(),
            Self::AllPersisted => "all_persisted".to_string(),
        }
    }
}

/// 崩溃点重放发现表的签名：判红的是哪一遍、违了哪几条、状态所在的段、状态归哪次发布。大小次序（先 pass 与违了哪几条，再段、再发布）
/// 只在同一个状态上出了几个签名时用来排号。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CrashPointReplayFindingSignature {
    pub red_pass: CrashPointReplayRedPass,
    pub segment: CrashPointReplaySegmentOfState,
    pub publish: CrashPointReplayPublishOfState,
}

/// 发现表里每个签名最多留几个样本（最先的几个状态）。
pub const CRASH_POINT_REPLAY_FINDING_SAMPLES_KEPT: usize = 3;

/// 签名下的一个样本：状态序号与那一条违例原文。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrashPointReplayFindingSample {
    pub state_ordinal: u64,
    pub violation: String,
}

/// 一个签名下判红的状态：几个，与最先的至多 [`CRASH_POINT_REPLAY_FINDING_SAMPLES_KEPT`] 个（序号从小到大，个数 = min(状态数, 3)）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrashPointReplayFinding {
    pub states: u64,
    pub earliest_samples: Vec<CrashPointReplayFindingSample>,
}

impl CrashPointReplayFinding {
    /// 最先那个状态：签名进表时就记了第一个样本。
    ///
    /// # Panics
    /// 一个样本都没有（不变量被破坏：进表的签名至少一个状态、至少一个样本）。
    #[must_use]
    pub fn first_sample(&self) -> &CrashPointReplayFindingSample {
        self.earliest_samples
            .first()
            .expect("进了发现表的签名至少一个状态，第一个状态进表时就记成了样本")
    }
}

/// 崩溃点重放的发现表：判红的状态按签名去重（[`CrashPointReplayFindingSignature`]），每个签名记状态数与最先几个样本；另记至少一遍判红的状态数
/// （一个状态几遍都红只算一次）。只有按段枚举的那几个入口记这一项（签名要状态所在的段与发布）；直接调
/// [`evaluate_state_for_versions`] 评手摆的状态时它是空的。判红的状态逐个经 [`CrashPointReplayFindings::record_red_state`] 记进来
/// （现判时在评那个状态的那一刻，从判定库读回时按库里存的红例行重建）；发现日志里的号按
/// [`CrashPointReplayFindings::signatures_in_first_state_order`] 排。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CrashPointReplayFindings {
    pub red_states: u64,
    pub by_signature: BTreeMap<CrashPointReplayFindingSignature, CrashPointReplayFinding>,
}

impl CrashPointReplayFindings {
    /// 记一个判红的状态在一个签名下；这个签名的样本还不满 [`CRASH_POINT_REPLAY_FINDING_SAMPLES_KEPT`] 个时才调 `violation` 取原文。
    /// 同一片里状态按序号从小到大评，样本自然从小到大。
    pub(crate) fn record_red_state(
        &mut self,
        signature: CrashPointReplayFindingSignature,
        state_ordinal: u64,
        violation: impl FnOnce() -> String,
    ) {
        let finding =
            self.by_signature
                .entry(signature)
                .or_insert_with(|| CrashPointReplayFinding {
                    states: 0,
                    earliest_samples: Vec::new(),
                });
        finding.states += 1;
        if finding.earliest_samples.len() < CRASH_POINT_REPLAY_FINDING_SAMPLES_KEPT {
            finding
                .earliest_samples
                .push(CrashPointReplayFindingSample {
                    state_ordinal,
                    violation: violation(),
                });
        }
    }

    /// 签名按最先那个状态的序号排（同一个状态上的几个签名按签名的大小次序）：发现日志里的号就是这个次序里的第几个（从 1 起）。
    #[must_use]
    pub fn signatures_in_first_state_order(
        &self,
    ) -> Vec<(&CrashPointReplayFindingSignature, &CrashPointReplayFinding)> {
        let mut ordered: Vec<(&CrashPointReplayFindingSignature, &CrashPointReplayFinding)> =
            self.by_signature.iter().collect();
        ordered.sort_by(
            |(left_signature, left_finding), (right_signature, right_finding)| {
                (left_finding.first_sample().state_ordinal, *left_signature)
                    .cmp(&(right_finding.first_sample().state_ordinal, *right_signature))
            },
        );
        ordered
    }
}

/// 崩溃点重放的计数：每个状态跑一遍看 journal 的恢复与一遍不看的，oracle 只判前者。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CrashPointReplayTally {
    pub states: u64,
    /// 枚举出来的状态按发布分（[`CrashPointReplayPublishOfState`] 的归法）：各格之和 = `states`。只有按段枚举的那几个入口
    /// （`enumerate_crash_states*`）记这一项；直接调 [`evaluate_state_for_versions`] 评手摆的状态时它是空的。
    pub states_by_publish: BTreeMap<CrashPointReplayPublishOfState, u64>,
    pub violations: u64,
    pub root_persisted_states: u64,
    pub no_file_states: u64,
    pub file_read_states: u64,
    pub failed_states: u64,
    /// 两遍恢复落到的根、结局码或失败点不同的状态数（`crash_judge_conclusions::recovery_passes_differ`：现判与从库里的恢复字重建按同一组字比）。
    pub journal_differing_states: u64,
    pub first_violation: Option<String>,
    /// 不看 journal 那一遍恢复（`JournalPolicy::Ignore`）过同一个 oracle 判违例的状态数，另计、不混进 `violations`
    /// （靶向对照里「根槽已持久而单元缺席」那一格两遍都违例）。发布 B 之后两遍恢复会读出两个不同的版本，这一遍此前没人判。
    pub ignored_violations: u64,
    pub first_ignored_violation: Option<String>,
    /// 记录核对器判「根在案而记录缺席」的状态数。
    pub record_root_without_record: u64,
    /// 记录核对器判「恢复自称新态而单元缺席」的状态数。
    pub record_claimed_state_missing_unit: u64,
    /// checker：每条不变量在几个状态上评估过（成立或违例）、在几个状态上判违例、第一处违例。
    pub checker_evaluated_states: BTreeMap<&'static str, u64>,
    pub checker_violated_states: BTreeMap<&'static str, u64>,
    pub checker_first_violation: BTreeMap<&'static str, String>,
    /// checker 报「不适用」（这条不变量判的代码在这个状态上没跑到）的状态数：与评估过的状态数分开报，阴性结果不与「没跑到」混在一起
    /// （里程碑「覆盖写、释放、回退与复用」步 6 验收第 3 条）；每条不变量的评估过 + 不适用 = `states`。
    pub checker_not_applicable_states: BTreeMap<&'static str, u64>,
    /// 观察者看过的状态数（没有观察者时是 0）：从库里复用的块不再给观察者看，判段按块从库里存的观察者结论重建这个数与 `observer_counts`；
    /// 有观察者时整条流跑完它必须等于 `states`（崩溃点重放规模第三轮判决 U3）。
    pub observed_states: u64,
    /// 观察者按名字累计的数（[`CrashPointReplayObserverCounts`]），按块存进库（观察者各自的结论种类）。
    pub observer_counts: CrashPointReplayObserverCounts,
    /// 判红的状态按签名去重的发现表（[`CrashPointReplayFindings`]），从库里存的红例行重建。
    pub findings: CrashPointReplayFindings,
}

/// 观察者在状态上记的数，按名字累计（名字只许小写字母、数字、`_`：它原样进库里的结论种类名与打印的计数行）。从库里复用的块不再给观察者看，
/// 观察者要接着用的累计都得记在这里，记在观察者自己的变量里的复用之后就少了那几块。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CrashPointReplayObserverCounts(BTreeMap<String, u64>);

impl CrashPointReplayObserverCounts {
    /// 名字能不能当观察者计数的名字：非空，只有小写字母、数字、`_`。
    #[must_use]
    pub fn is_counter_name(name: &str) -> bool {
        !name.is_empty()
            && name
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    }

    /// 给名字为 `name` 的计数加 `amount`。
    ///
    /// # Panics
    /// 名字不合 [`Self::is_counter_name`]。
    pub fn add(&mut self, name: &str, amount: u64) {
        assert!(
            Self::is_counter_name(name),
            "观察者计数的名字只许小写字母、数字、_：{name:?}"
        );
        *self.0.entry(name.to_string()).or_insert(0) += amount;
    }

    /// 名字为 `name` 的计数，没记过是 0。
    #[must_use]
    pub fn count_named(&self, name: &str) -> u64 {
        self.0.get(name).copied().unwrap_or(0)
    }

    /// 按名字排好的全部计数。
    pub fn iter(&self) -> impl Iterator<Item = (&str, u64)> {
        self.0.iter().map(|(name, count)| (name.as_str(), *count))
    }
}

impl CrashPointReplayTally {
    /// checker 那一半按不变量报成一段：`I-x.y=评估过/判违例/不适用`，次序照 checker 的清单。
    #[must_use]
    pub fn checker_counts_by_invariant(&self) -> String {
        let count_of = |counts: &BTreeMap<&'static str, u64>, invariant: &str| {
            counts.get(invariant).copied().unwrap_or(0)
        };
        singlefs_checker::image::IMPLEMENTED_INVARIANTS
            .iter()
            .map(|invariant| {
                format!(
                    "{invariant}={}/{}/{}",
                    count_of(&self.checker_evaluated_states, invariant),
                    count_of(&self.checker_violated_states, invariant),
                    count_of(&self.checker_not_applicable_states, invariant)
                )
            })
            .collect::<Vec<String>>()
            .join(" ")
    }

    /// 按发布分的状态数报成一段：`instance1_txg1=15 … after_the_last_root=3 every_write_persisted=1`，次序照录制流。
    #[must_use]
    pub fn states_by_publish_text(&self) -> String {
        self.states_by_publish
            .iter()
            .map(|(publish, states)| format!("{}={states}", publish.name()))
            .collect::<Vec<String>>()
            .join(" ")
    }
}

/// oracle（E77（发布的持久顺序） 判据 1）：读回的内容要对；根槽已持久就不许恢复到旧态；走读不许失败。
/// 单版本形态：整条流只有一次带文件的发布（新池新建文件）。
#[must_use]
pub fn oracle_violation(
    outcome: &RecoveryOutcome,
    root_persisted: bool,
    expected_content: &[u8],
) -> Option<String> {
    match outcome {
        RecoveryOutcome::FileRead { content, .. } => {
            (content != expected_content).then(|| "读回的内容不对".to_string())
        }
        RecoveryOutcome::NoFile { .. } => {
            root_persisted.then(|| "根槽已持久而恢复到旧态".to_string())
        }
        RecoveryOutcome::Failed { failure, .. } => Some(format!("走读失败：{failure:?}")),
    }
}

/// 多版本形态的 oracle：实际走的根是哪一代，读回的就得是那一代写出的内容；根下面没有文件的那几代（mkfs、暖机）只许报没有文件；
/// 盘上已持久的最新根槽是 (T, 实例 i)，恢复就不许落到按 (txg, 实例) 字典序比它旧的根上（D22（单元原子性怎么合成） 已定项 7：
/// 择新 txg 为主、平局按实例代号高者赢）；走读不许失败。交回给人看的原因；哪一类见 [`classified_oracle_violation_for_versions`]。
#[must_use]
pub fn oracle_violation_for_versions(
    outcome: &RecoveryOutcome,
    effective_root: Option<(InstanceGeneration, CheckpointTxg)>,
    newest_persisted_root: Option<(CheckpointTxg, InstanceGeneration)>,
    versions: &[PublishedVersion],
) -> Option<String> {
    classified_oracle_violation_for_versions(
        outcome,
        effective_root,
        newest_persisted_root,
        versions,
    )
    .map(|violation| violation.reason)
}

/// 崩溃点重放一条流的版本表：带文件的那几版（[`PublishedVersion`]）与文件从那一版起没有了的那几版（[`PublishedVersionWithoutAFile`]，
/// D8（核心索引结构） 已定项 16 第 4 条：删除意图那次发布起冷走读答没有文件）。表按根（实例，txg）认；落到的根不在表里时，
/// 照比它旧的最新那一项判（按 (txg, 实例) 字典序，同 D22（单元原子性怎么合成） 已定项 7 的择根序）。
#[derive(Clone, Copy, Debug)]
pub struct CrashPointReplayVersionTable<'table> {
    versions_with_a_file: &'table [PublishedVersion],
    versions_without_a_file: &'table [PublishedVersionWithoutAFile],
}

/// 恢复落到的根在版本表里是哪一项。
#[derive(Clone, Copy, Debug)]
enum CrashPointReplayVersionAtTheRoot<'table> {
    /// 这一版有文件，读回的要是它的内容。
    WithAFile(&'table PublishedVersion),
    /// 文件从这一版起没有了。
    WithoutAFile,
    /// 表里没有这个根。
    NotInTheTable,
}

impl<'table> CrashPointReplayVersionTable<'table> {
    /// 有文件的那几版（崩溃点重放过库时抄进录下的流）。
    #[must_use]
    pub(crate) fn versions_with_a_file(&self) -> &'table [PublishedVersion] {
        self.versions_with_a_file
    }

    /// 文件从那一版起没有了的那几版（崩溃点重放过库时抄进录下的流）。
    #[must_use]
    pub(crate) fn versions_without_a_file(&self) -> &'table [PublishedVersionWithoutAFile] {
        self.versions_without_a_file
    }

    /// 没有一版是「文件从这一版起没有了」：文件一旦写出就不消失的流（删除之前的每一条流）。
    #[must_use]
    pub fn with_a_file_only(versions_with_a_file: &'table [PublishedVersion]) -> Self {
        Self {
            versions_with_a_file,
            versions_without_a_file: &[],
        }
    }

    /// # Panics
    /// 同一个根（实例，txg）既在带文件的那几版里、又在没有文件的那几版里：版本表自相矛盾，是用例写错了。
    #[must_use]
    pub fn new(
        versions_with_a_file: &'table [PublishedVersion],
        versions_without_a_file: &'table [PublishedVersionWithoutAFile],
    ) -> Self {
        for without_a_file in versions_without_a_file {
            assert!(
                !versions_with_a_file.iter().any(|with_a_file| {
                    (with_a_file.instance, with_a_file.checkpoint_txg)
                        == (without_a_file.instance, without_a_file.checkpoint_txg)
                }),
                "实例 {} 第 {} 代既写成有文件、又写成文件从这一版起没有了",
                without_a_file.instance.0,
                without_a_file.checkpoint_txg.0
            );
        }
        Self {
            versions_with_a_file,
            versions_without_a_file,
        }
    }

    fn version_at_the_root(
        &self,
        instance: InstanceGeneration,
        checkpoint_txg: CheckpointTxg,
    ) -> CrashPointReplayVersionAtTheRoot<'table> {
        if let Some(with_a_file) = self.versions_with_a_file.iter().find(|version| {
            version.checkpoint_txg == checkpoint_txg && version.instance == instance
        }) {
            return CrashPointReplayVersionAtTheRoot::WithAFile(with_a_file);
        }
        if self
            .versions_without_a_file
            .iter()
            .any(|version| version.checkpoint_txg == checkpoint_txg && version.instance == instance)
        {
            return CrashPointReplayVersionAtTheRoot::WithoutAFile;
        }
        CrashPointReplayVersionAtTheRoot::NotInTheTable
    }

    /// 不在表里的根照比它旧的最新那一项：那一项有文件就交回它（报没有文件是违例），那一项是没有文件、或没有更旧的项就交回 `None`。
    /// 文件从不消失的表里（[`Self::with_a_file_only`]）这与「任一更旧版有文件」相同。
    fn newest_older_version_if_it_has_a_file(
        &self,
        instance: InstanceGeneration,
        checkpoint_txg: CheckpointTxg,
    ) -> Option<&'table PublishedVersion> {
        let newest_older_with_a_file = self
            .versions_with_a_file
            .iter()
            .filter(|version| {
                (version.checkpoint_txg, version.instance) < (checkpoint_txg, instance)
            })
            .max_by_key(|version| (version.checkpoint_txg, version.instance));
        let newest_older_without_a_file = self
            .versions_without_a_file
            .iter()
            .map(|version| (version.checkpoint_txg, version.instance))
            .filter(|root| *root < (checkpoint_txg, instance))
            .max();
        match (newest_older_with_a_file, newest_older_without_a_file) {
            (Some(with_a_file), Some(without_a_file)) => {
                ((with_a_file.checkpoint_txg, with_a_file.instance) > without_a_file)
                    .then_some(with_a_file)
            }
            (Some(with_a_file), None) => Some(with_a_file),
            (None, Some(_) | None) => None,
        }
    }
}

/// 同 [`oracle_violation_for_versions`]，另交回违例是哪一类（[`CrashPointReplayOracleViolationKind`]，崩溃点重放发现表按它分签名）。
/// 文件从不消失的流：版本表是 [`CrashPointReplayVersionTable::with_a_file_only`]。
#[must_use]
pub fn classified_oracle_violation_for_versions(
    outcome: &RecoveryOutcome,
    effective_root: Option<(InstanceGeneration, CheckpointTxg)>,
    newest_persisted_root: Option<(CheckpointTxg, InstanceGeneration)>,
    versions: &[PublishedVersion],
) -> Option<CrashPointReplayOracleViolation> {
    classified_oracle_violation_for_the_version_table(
        outcome,
        effective_root,
        newest_persisted_root,
        CrashPointReplayVersionTable::with_a_file_only(versions),
    )
}

/// 崩溃点重放的 oracle 对一张版本表（[`CrashPointReplayVersionTable`]）判：落到的根是带文件的一版，读回的得是它的内容；是「文件从这一版起没有了」
/// 的一版，只许报没有文件；不在表里，报没有文件时看比它旧的最新那一项——那一项有文件就是违例（文件从不消失的流上与「任一更旧版有文件」
/// 相同），不在表里却读出了内容一律是违例。根槽已持久而恢复到旧态、走读失败、没择到根同 [`oracle_violation_for_versions`]。
#[must_use]
pub fn classified_oracle_violation_for_the_version_table(
    outcome: &RecoveryOutcome,
    effective_root: Option<(InstanceGeneration, CheckpointTxg)>,
    newest_persisted_root: Option<(CheckpointTxg, InstanceGeneration)>,
    versions: CrashPointReplayVersionTable<'_>,
) -> Option<CrashPointReplayOracleViolation> {
    let violation = |kind: CrashPointReplayOracleViolationKind, reason: String| {
        CrashPointReplayOracleViolation { kind, reason }
    };
    let Some((effective_instance, effective_txg)) = effective_root else {
        return Some(violation(
            CrashPointReplayOracleViolationKind::NoRootChosen,
            "没择到根".to_string(),
        ));
    };
    if let Some((newest_txg, newest_instance)) = newest_persisted_root {
        if (effective_txg, effective_instance) < (newest_txg, newest_instance) {
            return Some(violation(
                CrashPointReplayOracleViolationKind::RecoveredToAnOlderRoot,
                format!(
                "根槽已持久而恢复到旧态（盘上最新的根槽是实例 {} 第 {} 代，走的是实例 {} 第 {} 代）",
                newest_instance.0, newest_txg.0, effective_instance.0, effective_txg.0
            ),
            ));
        }
    }
    match (
        outcome,
        versions.version_at_the_root(effective_instance, effective_txg),
    ) {
        (RecoveryOutcome::Failed { failure, .. }, _) => Some(violation(
            CrashPointReplayOracleViolationKind::ReadFailed,
            format!("走读失败：{failure:?}"),
        )),
        // 落在一个表里没有的更新的根上报「没有文件」：比它旧的最新一项下面有文件时是违例（第二轮攻方腿：此前一律放过）；
        // 那一项是「文件从这一版起没有了」（删除那几批落在意图之后）时不是。
        (RecoveryOutcome::NoFile { .. }, CrashPointReplayVersionAtTheRoot::NotInTheTable) => versions
            .newest_older_version_if_it_has_a_file(effective_instance, effective_txg)
            .map(|older_version| {
                violation(
                    CrashPointReplayOracleViolationKind::NoFileWhereAnOlderRootHasOne,
                    format!(
                    "走到实例 {} 第 {} 代根报没有文件，而没有这一代的版本、实例 {} 第 {} 代下面有文件",
                    effective_instance.0,
                    effective_txg.0,
                    older_version.instance.0,
                    older_version.checkpoint_txg.0
                ),
                )
            }),
        (RecoveryOutcome::NoFile { .. }, CrashPointReplayVersionAtTheRoot::WithoutAFile) => None,
        (RecoveryOutcome::NoFile { .. }, CrashPointReplayVersionAtTheRoot::WithAFile(_)) => Some(violation(
            CrashPointReplayOracleViolationKind::NoFileWhereThisRootHasOne,
            format!("第 {} 代根下面有文件却报没有", effective_txg.0),
        )),
        (
            RecoveryOutcome::FileRead { .. },
            CrashPointReplayVersionAtTheRoot::NotInTheTable | CrashPointReplayVersionAtTheRoot::WithoutAFile,
        ) => Some(violation(
            CrashPointReplayOracleViolationKind::FileReadWhereThisRootHasNone,
            format!("第 {} 代根下面没有文件却读出了内容", effective_txg.0),
        )),
        (RecoveryOutcome::FileRead { content, .. }, CrashPointReplayVersionAtTheRoot::WithAFile(version)) => {
            (*content != version.content).then(|| {
                violation(
                    CrashPointReplayOracleViolationKind::WrongContent,
                    format!("读回的内容不对（走的是第 {} 代根）", effective_txg.0),
                )
            })
        }
    }
}

/// 单版本形态：被判的那次根槽写出的那一代就是唯一带文件的版本。
pub fn evaluate_state(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    persisted: Vec<bool>,
    root_index: usize,
    expected_content: &[u8],
    tally: &mut CrashPointReplayTally,
) -> RecoveryReport {
    let (instance, checkpoint_txg) = root_identity_of_write(&writes[root_index]);
    let versions = [PublishedVersion {
        instance,
        checkpoint_txg,
        content: expected_content.to_vec(),
    }];
    evaluate_state_for_versions(base, writes, persisted, root_index, &versions, tally)
}

/// 评一个状态：跑两种 journal 政策的恢复，记进计数。`judged_root_index` 是被判的那次根槽 FUA 写（计「根槽已持久」的状态数用），
/// oracle 按 `versions` 判实际走的根该读出哪一版。手摆的状态不在任何一段里，发现表（[`CrashPointReplayTally::findings`]）不记。
pub fn evaluate_state_for_versions(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    persisted: Vec<bool>,
    judged_root_index: usize,
    versions: &[PublishedVersion],
    tally: &mut CrashPointReplayTally,
) -> RecoveryReport {
    evaluate_state_recording_findings(
        base,
        writes,
        persisted,
        judged_root_index,
        CrashPointReplayVersionTable::with_a_file_only(versions),
        None,
        tally,
    )
}

/// 崩溃点重放按段枚举时一个状态在哪：序号、所在的段、归哪次发布。发现表的签名要后两样、样本要序号。
#[derive(Clone, Copy, Debug)]
struct CrashStatePosition {
    ordinal: u64,
    segment: CrashPointReplaySegmentOfState,
    publish: CrashPointReplayPublishOfState,
}

impl CrashStatePosition {
    fn signature_of(self, red_pass: CrashPointReplayRedPass) -> CrashPointReplayFindingSignature {
        CrashPointReplayFindingSignature {
            red_pass,
            segment: self.segment,
            publish: self.publish,
        }
    }
}

/// 看 journal 那一遍 oracle 的违例原文：原因后面带这一状态里持久了的写的种类（[`CrashPointReplayTally::first_violation`] 与发现表的样本同一个写法）。
fn violation_with_the_persisted_write_kinds(reason: &str, image: &CrashImage<'_>) -> String {
    let persisted_kinds: Vec<&str> = image
        .persisted
        .iter()
        .zip(image.writes)
        .filter(|(is_persisted, _)| **is_persisted)
        .map(|(_, write)| write.kind.name())
        .collect();
    format!("{reason}（持久的写：{}）", persisted_kinds.join("|"))
}

/// 池级 checker 在一个状态上判违例的那几条拼成一条原文：`I-3.1：细节；I-7.1：细节`。
fn pool_checker_violation_text(violated_invariants: &[(&'static str, String)]) -> String {
    violated_invariants
        .iter()
        .map(|(invariant, detail)| format!("{invariant}：{detail}"))
        .collect::<Vec<String>>()
        .join("；")
}

/// 记录核对器在一个状态上成立的判据拼成一条原文。
fn record_checker_violation_text(record_check: RecordCheck) -> String {
    let mut criteria = Vec::new();
    if record_check.root_without_record {
        criteria.push("根槽写在盘上而那次发布的 journal 记录一份都不在");
    }
    if record_check.claimed_state_missing_unit {
        criteria.push("恢复自称的那一版该有的单元两份都缺席");
    }
    criteria.join("；")
}

/// 一个崩溃状态的全部判定：两种 journal 政策的恢复、两份 oracle、池级 checker 逐条、记录核对器。
/// 崩溃点重放的计数（[`evaluate_state_for_versions`] 一族）与崩溃放量流水线（`crash_amplification`）都从这一份取，不各判一遍。
#[derive(Clone, Debug)]
pub struct CrashStateJudgement {
    pub consulted: RecoveryReport,
    pub ignored: RecoveryReport,
    /// 被判的那次根槽写落没落（计「根槽已持久」的状态数用）。
    pub root_persisted: bool,
    pub consulted_oracle: Option<CrashPointReplayOracleViolation>,
    pub ignored_oracle: Option<CrashPointReplayOracleViolation>,
    pub pool_checker: Vec<(&'static str, InvariantVerdict)>,
    pub records: RecordCheck,
}

impl CrashStateJudgement {
    /// 判红的各项，按固定次序：两份 oracle 各记它的违例种类，池级 checker 记违反的不变量名，记录核对器记它的两种结论；
    /// 一项都没有就是绿。崩溃放量流水线按这串存「逐项原始判定」（读方现算红绿与计数）。
    #[must_use]
    pub fn red_items(&self) -> Vec<String> {
        let mut items = Vec::new();
        if let Some(violation) = &self.consulted_oracle {
            items.push(format!(
                "{CRASH_POINT_REPLAY_RED_PASS_JOURNAL_CONSULTED_ORACLE}:{}",
                violation.kind.name()
            ));
        }
        if let Some(violation) = &self.ignored_oracle {
            items.push(format!(
                "{CRASH_POINT_REPLAY_RED_PASS_JOURNAL_IGNORED_ORACLE}:{}",
                violation.kind.name()
            ));
        }
        for (invariant, verdict) in &self.pool_checker {
            match verdict {
                InvariantVerdict::Violated(_) => items.push(format!(
                    "{CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER}:{invariant}"
                )),
                InvariantVerdict::Holds
                | InvariantVerdict::NotApplicable(_)
                | InvariantVerdict::PartiallyJudged(_) => {}
            }
        }
        if self.records.root_without_record {
            items.push(format!(
                "{CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER}:{RECORD_CHECKER_ROOT_WITHOUT_RECORD}"
            ));
        }
        if self.records.claimed_state_missing_unit {
            items.push(format!(
                "{CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER}:{RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT}"
            ));
        }
        items
    }

    /// 违例正文：这个状态自己现算的（第三轮判决「正文逐状态」），不从类的代表取。绿的状态是空串。
    #[must_use]
    pub fn violation_text(&self, image: &CrashImage<'_>) -> String {
        let mut text = String::new();
        if let Some(violation) = &self.consulted_oracle {
            text.push_str(&violation_with_the_persisted_write_kinds(
                &violation.reason,
                image,
            ));
            text.push('\n');
        }
        if let Some(violation) = &self.ignored_oracle {
            text.push_str(&violation.reason);
            text.push('\n');
        }
        let violated: Vec<(&'static str, String)> = self
            .pool_checker
            .iter()
            .filter_map(|(invariant, verdict)| match verdict {
                InvariantVerdict::Violated(detail) => Some((*invariant, detail.clone())),
                InvariantVerdict::Holds
                | InvariantVerdict::NotApplicable(_)
                | InvariantVerdict::PartiallyJudged(_) => None,
            })
            .collect();
        if !violated.is_empty() {
            text.push_str(&pool_checker_violation_text(&violated));
            text.push('\n');
        }
        if self.records.root_without_record || self.records.claimed_state_missing_unit {
            text.push_str(&record_checker_violation_text(self.records));
            text.push('\n');
        }
        text
    }
}

/// 判一个崩溃镜像（写表是枚举用的那一张，撕裂镜像与重放接在后面）；`judged_root_index` 是被判的那次根槽 FUA 写。
/// 文件从不消失的流：版本表是 [`CrashPointReplayVersionTable::with_a_file_only`]。
#[must_use]
pub fn judge_crash_image(
    image: &CrashImage<'_>,
    judged_root_index: usize,
    versions: &[PublishedVersion],
) -> CrashStateJudgement {
    judge_crash_image_against_the_version_table(
        image,
        judged_root_index,
        CrashPointReplayVersionTable::with_a_file_only(versions),
    )
}

/// 同 [`judge_crash_image`]，oracle 对一张版本表判（[`classified_oracle_violation_for_the_version_table`]）。
#[must_use]
fn judge_crash_image_against_the_version_table(
    image: &CrashImage<'_>,
    judged_root_index: usize,
    versions: CrashPointReplayVersionTable<'_>,
) -> CrashStateJudgement {
    let root_persisted = image.persisted[judged_root_index];
    let newest_persisted = newest_persisted_root(image.writes, &image.persisted);
    let consulted = recover(image, JournalPolicy::Consult);
    let ignored = recover(image, JournalPolicy::Ignore);
    let consulted_oracle = classified_oracle_violation_for_the_version_table(
        &consulted.outcome,
        consulted.effective_root,
        newest_persisted,
        versions,
    );
    let ignored_oracle = classified_oracle_violation_for_the_version_table(
        &ignored.outcome,
        ignored.effective_root,
        newest_persisted,
        versions,
    );
    let pool_checker = check_pool_image(image);
    let records = check_records(image, consulted.effective_root);
    CrashStateJudgement {
        consulted,
        ignored,
        root_persisted,
        consulted_oracle,
        ignored_oracle,
        pool_checker,
        records,
    }
}

/// [`judge_crash_state_range_against_the_version_table_naming_the_publish`] 的本体：只给已经从判定库读回的流用（崩溃放量判段）。
///
/// # Panics
/// `ordinals` 越过整条流的状态数。
#[allow(
    clippy::too_many_arguments,
    reason = "与 judge_crash_state_range_against_the_version_table_naming_the_publish 同一组参数"
)]
#[must_use]
pub(crate) fn judge_crash_state_range_of_the_read_back_flow(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: CrashPointReplayVersionTable<'_>,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
    ordinals: Range<u64>,
    judge_state: &mut dyn FnMut(
        u64,
        CrashPointReplayPublishOfState,
        &CrashImage<'_>,
        &CrashStateJudgement,
    ),
) -> u64 {
    judge_crash_state_range_of_the_read_back_flow_with_slices(
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        expansion,
        &no_segment_slices(segments.len()),
        ordinals,
        judge_state,
    )
}

/// 同 [`judge_crash_state_range_of_the_read_back_flow`]，分片的段照 `segment_slices`（[`CrashPointReplaySegmentSlice`]，与段同序）只判那一截
/// （崩溃放量判段读回的流带着录入段定的片）。
///
/// # Panics
/// 同 [`judge_crash_state_range_of_the_read_back_flow`]；片越过了段的域（`CrashStatePlan::new`）。
#[allow(
    clippy::too_many_arguments,
    reason = "与 judge_crash_state_range_of_the_read_back_flow 同一组参数，另加每段的片"
)]
#[must_use]
pub(crate) fn judge_crash_state_range_of_the_read_back_flow_with_slices(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: CrashPointReplayVersionTable<'_>,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
    segment_slices: &[Option<CrashPointReplaySegmentSlice>],
    ordinals: Range<u64>,
    judge_state: &mut dyn FnMut(
        u64,
        CrashPointReplayPublishOfState,
        &CrashImage<'_>,
        &CrashStateJudgement,
    ),
) -> u64 {
    visit_crash_state_range_of_the_read_back_flow_with_slices(
        base,
        writes,
        segments,
        expansion,
        segment_slices,
        ordinals,
        &mut |ordinal, publish, image| {
            let judgement =
                judge_crash_image_against_the_version_table(image, judged_root_index, versions);
            judge_state(ordinal, publish, image, &judgement);
        },
    )
}

/// [`judge_crash_state_range_of_the_read_back_flow_with_slices`] 走状态的那一半（逐个交镜像、不判）：只给已经从判定库读回的流用（崩溃放量判段、过库之后），分片的段照 `segment_slices`
/// （与段同序，不分片的段是 `None`）只走那一截。
///
/// # Panics
/// `ordinals` 越过整条流的状态数；片越过了段的域（`CrashStatePlan::new`）。
#[must_use]
pub(crate) fn visit_crash_state_range_of_the_read_back_flow_with_slices(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
    segment_slices: &[Option<CrashPointReplaySegmentSlice>],
    ordinals: Range<u64>,
    visit_state: &mut dyn FnMut(u64, CrashPointReplayPublishOfState, &CrashImage<'_>),
) -> u64 {
    let tearable = TearableInPlaceOverwrites::of(base, writes);
    let writes_with_torn_images = WritesWithTornImages::of(base, writes, segments, &tearable);
    let plan = CrashStatePlan::new(
        writes,
        segments,
        expansion,
        segment_slices,
        &tearable,
        &writes_with_torn_images,
    );
    assert!(
        ordinals.end <= plan.state_count,
        "序号段 {ordinals:?} 越过整条流的 {} 个状态",
        plan.state_count
    );
    let enumerated_writes = writes_with_torn_images.writes.as_slice();
    for ordinal in ordinals {
        let image = CrashImage {
            base,
            writes: enumerated_writes,
            persisted: plan.persisted_writes_of_state(ordinal),
        };
        visit_state(ordinal, plan.publish_of_state(ordinal), &image);
    }
    plan.state_count
}

/// 每一段展开出来的状态序号区间（与 `segments` 同序，不展开的段是空区间），与整条流的状态数（最后那个全持久状态的序号是它减一）。
/// 崩溃放量流水线按它把状态分给崩溃点：一个点管它起头的那几段。
#[must_use]
pub fn crash_state_ranges_by_segment(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> (Vec<Range<u64>>, u64) {
    crash_state_ranges_by_segment_with_slices(
        base,
        writes,
        segments,
        expansion,
        &no_segment_slices(segments.len()),
    )
}

/// 同 [`crash_state_ranges_by_segment`]，分片的段的序号区间是它的片长（[`CrashPointReplaySegmentSlice`]）。
///
/// # Panics
/// 片越过了段的域（`CrashStatePlan::new`）。
#[must_use]
pub fn crash_state_ranges_by_segment_with_slices(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
    segment_slices: &[Option<CrashPointReplaySegmentSlice>],
) -> (Vec<Range<u64>>, u64) {
    let tearable = TearableInPlaceOverwrites::of(base, writes);
    let writes_with_torn_images = WritesWithTornImages::of(base, writes, segments, &tearable);
    let plan = CrashStatePlan::new(
        writes,
        segments,
        expansion,
        segment_slices,
        &tearable,
        &writes_with_torn_images,
    );
    (plan.state_ranges_by_segment.clone(), plan.state_count)
}

/// 崩溃点重放枚举用的几张表，给崩溃放量第 ① 段抽事实用（`crate::crash_facts`）：枚举写表（录制流里的写在前、撕裂镜像与重放接在后面）、
/// 哪几次写取三态、每次原地覆写的撕裂镜像与重放接在枚举写表的哪里、每段展开出来的状态序号区间、状态总数。
/// 与 [`judge_crash_state_range`] 按同一套 `CrashStatePlan` 算：事实那边按这几张表现算的掩码与这里的 `persisted_writes_of_state` 逐位相同。
/// 一次原地覆写的撕裂镜像接在枚举写表的哪里：写表下标、撕裂镜像在枚举写表里的下标、（重放在枚举写表里的下标，重放的是写表里哪一次写）。
pub struct TornImageTableRow {
    pub write_index: usize,
    pub torn_image_index: usize,
    pub replays: Vec<(usize, usize)>,
}

pub struct CrashStateEnumerationTables {
    pub enumerated_writes: Vec<RetainedWrite>,
    pub is_tearable_by_write: Vec<bool>,
    pub torn_images: Vec<TornImageTableRow>,
    pub state_ranges_by_segment: Vec<Range<u64>>,
    /// 每一段这一趟的片（与段同序，不分片的段是 `None`）：分片的段在 `state_ranges_by_segment` 里的区间是片长。
    pub segment_slices: Vec<Option<CrashPointReplaySegmentSlice>>,
    pub state_count: u64,
}

#[must_use]
pub fn crash_state_enumeration_tables(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> CrashStateEnumerationTables {
    crash_state_enumeration_tables_with_slices(
        base,
        writes,
        segments,
        expansion,
        &no_segment_slices(segments.len()),
    )
}

/// 同 [`crash_state_enumeration_tables`]，分片的段照 `segment_slices`。
///
/// # Panics
/// 片越过了段的域（`CrashStatePlan::new`）。
#[must_use]
pub fn crash_state_enumeration_tables_with_slices(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
    segment_slices: &[Option<CrashPointReplaySegmentSlice>],
) -> CrashStateEnumerationTables {
    crash_state_enumeration_tables_with_slices_choosing_the_torn_images(
        base,
        writes,
        segments,
        expansion,
        segment_slices,
        TornImagesOfTheInPlaceOverwrites::Appended,
    )
}

/// 枚举写表末尾接不接原地覆写的撕裂镜像（连同它们的重放）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TornImagesOfTheInPlaceOverwrites {
    /// 崩溃点重放的口径：每次原地覆写（[`TearableInPlaceOverwrites::of`]）取三态，撕裂镜像与重放接在录制流后面。
    Appended,
    /// 每次写只取两态（[`TearableInPlaceOverwrites::none`]），枚举写表就是录制流：给定状态表的持久掩码从不点名撕裂态时用
    /// （随机历史每步的镜像全是「前缀全落、之后一个都没落」）。接了也是从不落的写，只白占掩码的位与它罩住的落点的写数。
    LeftOut,
}

/// 同 [`crash_state_enumeration_tables_with_slices`]，撕裂镜像接不接照 `torn_images`。
///
/// # Panics
/// 同 [`crash_state_enumeration_tables_with_slices`]。
#[must_use]
pub fn crash_state_enumeration_tables_with_slices_choosing_the_torn_images(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
    segment_slices: &[Option<CrashPointReplaySegmentSlice>],
    torn_images: TornImagesOfTheInPlaceOverwrites,
) -> CrashStateEnumerationTables {
    let tearable = match torn_images {
        TornImagesOfTheInPlaceOverwrites::Appended => TearableInPlaceOverwrites::of(base, writes),
        TornImagesOfTheInPlaceOverwrites::LeftOut => TearableInPlaceOverwrites::none(writes.len()),
    };
    let writes_with_torn_images = WritesWithTornImages::of(base, writes, segments, &tearable);
    let plan = CrashStatePlan::new(
        writes,
        segments,
        expansion,
        segment_slices,
        &tearable,
        &writes_with_torn_images,
    );
    CrashStateEnumerationTables {
        enumerated_writes: writes_with_torn_images.writes.clone(),
        is_tearable_by_write: tearable.is_tearable_by_write.clone(),
        torn_images: writes_with_torn_images
            .torn_image_by_write
            .iter()
            .map(|(write_index, torn)| TornImageTableRow {
                write_index: *write_index,
                torn_image_index: torn.torn_image_index,
                replays: torn.replays.clone(),
            })
            .collect(),
        state_ranges_by_segment: plan.state_ranges_by_segment.clone(),
        segment_slices: segment_slices.to_vec(),
        state_count: plan.state_count,
    }
}

/// 整条流在这种展开下的状态数，与 [`judge_crash_state_range`] 交回的相同（不判任何状态）。
#[must_use]
pub fn crash_point_replay_plan_state_count(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> u64 {
    let tearable = TearableInPlaceOverwrites::of(base, writes);
    let writes_with_torn_images = WritesWithTornImages::of(base, writes, segments, &tearable);
    CrashStatePlan::new(
        writes,
        segments,
        expansion,
        &no_segment_slices(segments.len()),
        &tearable,
        &writes_with_torn_images,
    )
    .state_count
}

/// [`evaluate_state_for_versions`] 的本体：`position` 是这个状态在按段枚举里的位置（手摆的状态没有），有位置时把判红的每一遍
/// 按签名记进发现表（[`CrashPointReplayFindings`]），至少一遍判红的状态另记一个。
fn evaluate_state_recording_findings(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    persisted: Vec<bool>,
    judged_root_index: usize,
    versions: CrashPointReplayVersionTable<'_>,
    position: Option<CrashStatePosition>,
    tally: &mut CrashPointReplayTally,
) -> RecoveryReport {
    let image = CrashImage {
        base,
        writes,
        persisted,
    };
    let judgement =
        judge_crash_image_against_the_version_table(&image, judged_root_index, versions);
    let passes_differ = recovery_passes_differ(&judgement.consulted, &judgement.ignored);
    let CrashStateJudgement {
        consulted,
        ignored: _,
        root_persisted,
        consulted_oracle,
        ignored_oracle,
        pool_checker,
        records,
    } = judgement;
    tally.states += 1;
    if root_persisted {
        tally.root_persisted_states += 1;
    }
    if passes_differ {
        tally.journal_differing_states += 1;
    }
    match &consulted.outcome {
        RecoveryOutcome::NoFile { .. } => tally.no_file_states += 1,
        RecoveryOutcome::FileRead { .. } => tally.file_read_states += 1,
        RecoveryOutcome::Failed { .. } => tally.failed_states += 1,
    }
    let mut is_red_state = false;
    if let Some(violation) = consulted_oracle {
        tally.violations += 1;
        is_red_state = true;
        if tally.first_violation.is_none() {
            tally.first_violation = Some(violation_with_the_persisted_write_kinds(
                &violation.reason,
                &image,
            ));
        }
        if let Some(position) = position {
            tally.findings.record_red_state(
                position.signature_of(CrashPointReplayRedPass::JournalConsultedOracle(
                    violation.kind,
                )),
                position.ordinal,
                || violation_with_the_persisted_write_kinds(&violation.reason, &image),
            );
        }
    }
    if let Some(violation) = ignored_oracle {
        tally.ignored_violations += 1;
        is_red_state = true;
        if tally.first_ignored_violation.is_none() {
            tally.first_ignored_violation = Some(violation.reason.clone());
        }
        if let Some(position) = position {
            tally.findings.record_red_state(
                position.signature_of(CrashPointReplayRedPass::JournalIgnoredOracle(
                    violation.kind,
                )),
                position.ordinal,
                || violation.reason,
            );
        }
    }
    let mut violated_invariants: Vec<(&'static str, String)> = Vec::new();
    for (invariant, verdict) in pool_checker {
        match verdict {
            InvariantVerdict::Holds => {
                *tally.checker_evaluated_states.entry(invariant).or_insert(0) += 1
            }
            InvariantVerdict::Violated(detail) => {
                *tally.checker_evaluated_states.entry(invariant).or_insert(0) += 1;
                *tally.checker_violated_states.entry(invariant).or_insert(0) += 1;
                tally
                    .checker_first_violation
                    .entry(invariant)
                    .or_insert_with(|| detail.clone());
                violated_invariants.push((invariant, detail));
            }
            // 部分没判（`InvariantVerdict::PartiallyJudged`）不红、也不算评估过（判到的那一部分成立不等于整条成立）：计进不适用这一格，
            // 「评估过 + 不适用 = 状态数」照旧成立，库里存的计数也不多一格。
            InvariantVerdict::NotApplicable(_) | InvariantVerdict::PartiallyJudged(_) => {
                *tally
                    .checker_not_applicable_states
                    .entry(invariant)
                    .or_insert(0) += 1;
            }
        }
    }
    if !violated_invariants.is_empty() {
        is_red_state = true;
        if let Some(position) = position {
            tally.findings.record_red_state(
                position.signature_of(CrashPointReplayRedPass::PoolChecker(
                    violated_invariants
                        .iter()
                        .map(|(invariant, _detail)| *invariant)
                        .collect(),
                )),
                position.ordinal,
                || pool_checker_violation_text(&violated_invariants),
            );
        }
    }
    if records.root_without_record {
        tally.record_root_without_record += 1;
    }
    if records.claimed_state_missing_unit {
        tally.record_claimed_state_missing_unit += 1;
    }
    if records.root_without_record || records.claimed_state_missing_unit {
        is_red_state = true;
        if let Some(position) = position {
            tally.findings.record_red_state(
                position.signature_of(CrashPointReplayRedPass::RecordChecker(records)),
                position.ordinal,
                || record_checker_violation_text(records),
            );
        }
    }
    if is_red_state && position.is_some() {
        tally.findings.red_states += 1;
    }
    consulted
}

/// 态数的肢数：[`StateCount1280Bits`] 由这么多个 u128 拼成（用户 2026-10-13 定「10 个 u128 拼起来」：按段登记的段的域、片起点、
/// 片尾与进度都用这个定宽整数，算术全部 checked，溢出这个位宽才拒跑）。
pub const STATE_COUNT_LIMBS: usize = 10;

/// 一个 [`StateCount1280Bits`] 存成字节的长度（小端，低肢在前）：事实表的片起点与判定库的进度记录按它存。
pub const STATE_COUNT_BYTES: usize = STATE_COUNT_LIMBS * 16;

/// 半肢（u64）的个数：乘、除小数时按半肢做，中间量装得进 u128。
const STATE_COUNT_HALF_LIMBS: usize = 2 * STATE_COUNT_LIMBS;

/// 一次十进制分块的除数 10^19（装得进 u64 的最大的 10 的幂）与它的位数。
const DECIMAL_CHUNK_DIVISOR: u64 = 10_000_000_000_000_000_000;
const DECIMAL_CHUNK_DIGITS: usize = 19;

/// 1280 位（[`STATE_COUNT_LIMBS`] 个 u128）的无符号态数：按段登记的段整域的态数（3^290 ≈ 2^460 装得下）、片起点、片尾、进度。
/// 算术全部 checked（交 `None` 就是越过 1280 位或减成负数），不饱和、不截断。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StateCount1280Bits {
    /// 低肢在前。
    limbs: [u128; STATE_COUNT_LIMBS],
}

impl PartialOrd for StateCount1280Bits {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StateCount1280Bits {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.limbs.iter().rev().cmp(other.limbs.iter().rev())
    }
}

/// 一个 u128 的低 64 位。
fn low_sixty_four_bits(value: u128) -> u64 {
    u64::try_from(value & u128::from(u64::MAX)).expect("掩掉高 64 位之后装得进 u64")
}

/// 一个 u128 的高 64 位。
fn high_sixty_four_bits(value: u128) -> u64 {
    u64::try_from(value >> 64).expect("右移 64 位之后装得进 u64")
}

impl StateCount1280Bits {
    pub const ZERO: Self = Self {
        limbs: [0; STATE_COUNT_LIMBS],
    };

    #[must_use]
    pub fn from_u64(value: u64) -> Self {
        let mut limbs = [0u128; STATE_COUNT_LIMBS];
        limbs[0] = u128::from(value);
        Self { limbs }
    }

    /// 装得进 u64 就交回那个数。
    #[must_use]
    pub fn to_u64(&self) -> Option<u64> {
        if self.limbs[1..].iter().any(|limb| *limb != 0) {
            return None;
        }
        u64::try_from(self.limbs[0]).ok()
    }

    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.limbs.iter().all(|limb| *limb == 0)
    }

    fn halves(&self) -> [u64; STATE_COUNT_HALF_LIMBS] {
        let mut halves = [0u64; STATE_COUNT_HALF_LIMBS];
        for (limb_index, limb) in self.limbs.iter().enumerate() {
            halves[2 * limb_index] = low_sixty_four_bits(*limb);
            halves[2 * limb_index + 1] = high_sixty_four_bits(*limb);
        }
        halves
    }

    fn of_halves(halves: &[u64; STATE_COUNT_HALF_LIMBS]) -> Self {
        let mut limbs = [0u128; STATE_COUNT_LIMBS];
        for (limb_index, limb) in limbs.iter_mut().enumerate() {
            *limb =
                u128::from(halves[2 * limb_index]) | (u128::from(halves[2 * limb_index + 1]) << 64);
        }
        Self { limbs }
    }

    /// 两数之和；越过 1280 位交 `None`。
    #[must_use]
    pub fn checked_add(&self, other: &Self) -> Option<Self> {
        let mut limbs = [0u128; STATE_COUNT_LIMBS];
        let mut carry = false;
        for (limb_index, limb) in limbs.iter_mut().enumerate() {
            let (sum, first_overflow) =
                self.limbs[limb_index].overflowing_add(other.limbs[limb_index]);
            let (sum, second_overflow) = sum.overflowing_add(u128::from(carry));
            *limb = sum;
            carry = first_overflow || second_overflow;
        }
        (!carry).then_some(Self { limbs })
    }

    /// 加一个 u64；越过 1280 位交 `None`。
    #[must_use]
    pub fn checked_add_u64(&self, addend: u64) -> Option<Self> {
        self.checked_add(&Self::from_u64(addend))
    }

    /// 两数之差；`other` 比它大交 `None`。
    #[must_use]
    pub fn checked_sub(&self, other: &Self) -> Option<Self> {
        let mut limbs = [0u128; STATE_COUNT_LIMBS];
        let mut borrow = false;
        for (limb_index, limb) in limbs.iter_mut().enumerate() {
            let (difference, first_borrow) =
                self.limbs[limb_index].overflowing_sub(other.limbs[limb_index]);
            let (difference, second_borrow) = difference.overflowing_sub(u128::from(borrow));
            *limb = difference;
            borrow = first_borrow || second_borrow;
        }
        (!borrow).then_some(Self { limbs })
    }

    /// 乘一个 u64；越过 1280 位交 `None`。
    #[must_use]
    pub fn checked_mul_u64(&self, multiplier: u64) -> Option<Self> {
        let mut halves = self.halves();
        let mut carry = 0u64;
        for half in &mut halves {
            // 半肢乘 u64 再加一个 u64 的进位：(2^64 − 1)^2 + 2^64 − 1 < 2^128，装得进 u128
            let product = u128::from(*half) * u128::from(multiplier) + u128::from(carry);
            *half = low_sixty_four_bits(product);
            carry = high_sixty_four_bits(product);
        }
        (carry == 0).then(|| Self::of_halves(&halves))
    }

    /// 除以一个正的 u64：交回（商，余数）。
    ///
    /// # Panics
    /// 除数是 0。
    #[must_use]
    pub fn divided_by_u64(&self, divisor: u64) -> (Self, u64) {
        assert_ne!(divisor, 0, "除数不许是 0");
        let mut halves = self.halves();
        let mut remainder = 0u64;
        for half in halves.iter_mut().rev() {
            // 余数小于除数、小于 2^64：余数左移 64 位再拼上这一个半肢，装得进 u128
            let dividend = (u128::from(remainder) << 64) | u128::from(*half);
            *half = u64::try_from(dividend / u128::from(divisor))
                .expect("被除数的高 64 位（余数）小于除数，商装得进 u64");
            remainder = u64::try_from(dividend % u128::from(divisor)).expect("余数小于除数");
        }
        (Self::of_halves(&halves), remainder)
    }

    /// 拆成（右移 64 位之后的高位，低 64 位）：崩溃放量分片的段的块键里块起点只有 8 字节，高位进计划哈希那一格。
    #[must_use]
    pub fn split_at_sixty_four_bits(&self) -> (Self, u64) {
        let halves = self.halves();
        let mut shifted = [0u64; STATE_COUNT_HALF_LIMBS];
        shifted[..STATE_COUNT_HALF_LIMBS - 1].copy_from_slice(&halves[1..]);
        (Self::of_halves(&shifted), halves[0])
    }

    /// 十进制，不带前导 0（0 写 `0`）：驱动行与拒跑句照它写，不写浮点、不写近似。
    #[must_use]
    pub fn decimal_text(&self) -> String {
        if self.is_zero() {
            return "0".to_string();
        }
        let mut chunks_from_the_lowest: Vec<u64> = Vec::new();
        let mut rest = *self;
        while !rest.is_zero() {
            let (quotient, chunk) = rest.divided_by_u64(DECIMAL_CHUNK_DIVISOR);
            chunks_from_the_lowest.push(chunk);
            rest = quotient;
        }
        let mut text = String::new();
        for (position, chunk) in chunks_from_the_lowest.iter().rev().enumerate() {
            if position == 0 {
                text.push_str(&chunk.to_string());
            } else {
                text.push_str(&format!("{chunk:0width$}", width = DECIMAL_CHUNK_DIGITS));
            }
        }
        text
    }

    /// 小端字节（低肢在前、每肢 16 字节小端）。
    #[must_use]
    pub fn to_le_bytes(&self) -> [u8; STATE_COUNT_BYTES] {
        let mut bytes = [0u8; STATE_COUNT_BYTES];
        for (limb_index, limb) in self.limbs.iter().enumerate() {
            bytes[limb_index * 16..(limb_index + 1) * 16].copy_from_slice(&limb.to_le_bytes());
        }
        bytes
    }

    #[must_use]
    pub fn from_le_bytes(bytes: &[u8; STATE_COUNT_BYTES]) -> Self {
        let mut limbs = [0u128; STATE_COUNT_LIMBS];
        for (limb_index, limb) in limbs.iter_mut().enumerate() {
            *limb = u128::from_le_bytes(
                bytes[limb_index * 16..(limb_index + 1) * 16]
                    .try_into()
                    .expect("一肢 16 字节"),
            );
        }
        Self { limbs }
    }

    /// 按混合进位拆成各位（第 0 位最低，`radices` 是每一位的进制）；它不小于各进制之积（拆不下）交 `None`。
    ///
    /// # Panics
    /// 有一位的进制是 0。
    #[must_use]
    pub fn mixed_radix_digits(&self, radices: &[u8]) -> Option<Vec<u8>> {
        let mut rest = *self;
        let mut digits = Vec::with_capacity(radices.len());
        for radix in radices {
            let (quotient, digit) = rest.divided_by_u64(u64::from(*radix));
            digits.push(u8::try_from(digit).expect("一位上的数字小于它的进制，装得进 u8"));
            rest = quotient;
        }
        rest.is_zero().then_some(digits)
    }

    /// [`Self::mixed_radix_digits`] 的反方向；位数与进制数不同、有一位不小于它的进制、越过 1280 位交 `None`。
    #[must_use]
    pub fn of_mixed_radix_digits(digits: &[u8], radices: &[u8]) -> Option<Self> {
        if digits.len() != radices.len() {
            return None;
        }
        let mut value = Self::ZERO;
        for (digit, radix) in digits.iter().zip(radices).rev() {
            if digit >= radix {
                return None;
            }
            value = value
                .checked_mul_u64(u64::from(*radix))?
                .checked_add_u64(u64::from(*digit))?;
        }
        Some(value)
    }
}

/// 混合进位的各位上加一个 u64（第 0 位最低，逐位进位，不做宽整数除法：GPU 两个内核同一个算法、每一步只用 u32）：
/// 加完还有进位（越过各进制之积）交 `false`，这时各位是加到一半的样子、不能用。
///
/// # Panics
/// 位数与进制数不同；有一位的进制是 0。
#[must_use]
pub fn add_to_the_mixed_radix_digits(digits: &mut [u8], radices: &[u8], addend: u64) -> bool {
    assert_eq!(digits.len(), radices.len(), "每一位一个进制");
    let mut carry = addend;
    for (digit, radix) in digits.iter_mut().zip(radices) {
        if carry == 0 {
            break;
        }
        let radix = u64::from(*radix);
        // 进位先拆成「整进制的几倍」与余数，余数加到这一位上至多到 2 × 进制 − 2，不越过 u64
        let sum = u64::from(*digit) + carry % radix;
        *digit = u8::try_from(sum % radix).expect("一位上的数字小于它的进制，装得进 u8");
        carry = carry / radix + sum / radix;
    }
    carry == 0
}

/// 一段在这一趟里只判它的域（按它的展开方式展开出的全部状态，按混合进位编号、0 起；编号的次序见 [`CrashStatePlan::persisted_writes_of_state`]）
/// 里编号 [`Self::start_within_the_domain`, + `Self::state_count`) 那一截：按段登记的段整域装不进 u64 时分片枚举（C790），
/// 这一段在流里的序号区间就是这一截那么长，段内编号 = 片起点 + （序号 − 段在流里的起点）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CrashPointReplaySegmentSlice {
    pub start_within_the_domain: StateCount1280Bits,
    pub state_count: u64,
}

/// 每一段都不分片（与段同序）。
#[must_use]
pub fn no_segment_slices(segment_count: usize) -> Vec<Option<CrashPointReplaySegmentSlice>> {
    vec![None; segment_count]
}

/// 一段按它的展开方式拆编号时混合进位的一位：第 0 位最低。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CrashPointReplayDigitOfASegment {
    /// 甲二段里单元写那一组整组落不落（两进制，1 是全落）；只有单元写不空的甲二段有，排在最低位。
    CopyOnWriteGroup,
    /// 写表下标那一次写的落法（它能取几态就是几进制，[`WriteLandingChoices::landing_of_digit`]）。
    LandingOfAWrite(usize),
}

/// 分片的段拆编号要的：混合进位的每一位是什么、几进制，片起点拆成的各位。
#[derive(Clone, Debug, PartialEq, Eq)]
struct DigitsOfASlicedSegment {
    positions: Vec<CrashPointReplayDigitOfASegment>,
    radices: Vec<u8>,
    slice_start_digits: Vec<u8>,
}

/// 一段怎么展开成崩溃状态（前面的段全持久、这一段落一部分、后面的段全不落）。
/// 段里每次写各取几态见 [`TearableInPlaceOverwrites`]：原地覆写的写取三态（没持久 / 新旧都读不出 / 持久），其余取两态。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CrashPointReplaySegmentExpansion {
    /// 不展开：只以整段持久进入后面的状态。
    NotExpanded,
    /// 段内每次写各取它的几态、任意组合，去掉整段全持久那一个（D13（验证路线） 已定项 4 的全量）：
    /// n 个写里 m 个是原地覆写时 3^m · 2^(n−m) − 1 个；m = 0 时就是任意真子集 2^n − 1 个。
    EveryProperSubset,
    /// 甲二（崩溃点重放规模三轮判完、用户 2026-09-26 定的平时快档）：段内原地写（单元写之外的写）各取它的几态、任意组合 × 单元写（COW）只取
    /// 全不落或全落，去掉整段全落那一个。原地写 k 个（其中 m 个是原地覆写）、单元写 c 个：3^m · 2^(k−m) ·（c > 0 时 2，否则 1）− 1 个，
    /// m = 0 时 c > 0 是 2^(k+1) − 1、c = 0 是 2^k − 1（与全量相同）。
    /// 它看不见「单元写只落一部分」才显出来的那一类（C561 那一形就是），替不了全量：提交时照旧全量。
    InPlaceSubsetsWithCopyOnWriteNoneOrAll,
}

impl CrashPointReplaySegmentExpansion {
    /// 进计划哈希的名字（崩溃放量的块键要分得开两种展开方式，哪怕状态数碰巧相同）。
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::NotExpanded => "not_expanded",
            Self::EveryProperSubset => "every_proper_subset",
            Self::InPlaceSubsetsWithCopyOnWriteNoneOrAll => {
                "in_place_subsets_with_copy_on_write_none_or_all"
            }
        }
    }

    /// 这一段按这种展开出几个状态；`tearable` 里的写取三态，其余取两态。
    ///
    /// # Panics
    /// 一段的状态数装不进 u64（段里的写太多：2 态的写六十几个就到了）。装不装得下要先问的调用方用 [`Self::checked_state_count`]。
    #[must_use]
    pub fn state_count(
        self,
        writes: &[RetainedWrite],
        segment: &[usize],
        tearable: &TearableInPlaceOverwrites,
    ) -> u64 {
        self.checked_state_count(writes, segment, tearable)
            .and_then(|state_count| state_count.to_u64())
            .expect("一段的状态数装得进 u64")
    }

    /// 同 [`Self::state_count`]，交回 1280 位的态数（[`StateCount1280Bits`]，装得进 u64 的照样取得出 u64）；越过 1280 位交 `None`、
    /// 不 panic（崩溃放量录入段排计划之前先用它定按段登记的段整域展不展得开，`crate::crash_amplification::states_of_the_flow_to_record`）。
    #[must_use]
    pub fn checked_state_count(
        self,
        writes: &[RetainedWrite],
        segment: &[usize],
        tearable: &TearableInPlaceOverwrites,
    ) -> Option<StateCount1280Bits> {
        match self {
            Self::NotExpanded => Some(StateCount1280Bits::ZERO),
            Self::EveryProperSubset => tearable
                .checked_landing_combinations_of(segment)?
                .checked_sub(&StateCount1280Bits::from_u64(1)),
            Self::InPlaceSubsetsWithCopyOnWriteNoneOrAll => {
                let (in_place_writes, copy_on_write_writes) =
                    in_place_and_copy_on_write_writes_of(writes, segment);
                let copy_on_write_choices: u64 = if copy_on_write_writes.is_empty() {
                    1
                } else {
                    2
                };
                tearable
                    .checked_landing_combinations_of(&in_place_writes)?
                    .checked_mul_u64(copy_on_write_choices)?
                    .checked_sub(&StateCount1280Bits::from_u64(1))
            }
        }
    }

    /// 这一段拆编号时混合进位的各位（第 0 位最低）与各位的进制：全量是段里每次写；甲二是原地写，单元写不空时最低一位是单元写那一组；
    /// 不展开的段一位都没有。编号的次序与 [`CrashStatePlan::persisted_writes_of_state`] 拆不分片的段内序号相同。
    fn digits_of_the_segment(
        self,
        segment: &[usize],
        in_place_and_copy_on_write: &(Vec<usize>, Vec<usize>),
        tearable: &TearableInPlaceOverwrites,
    ) -> Vec<(CrashPointReplayDigitOfASegment, u8)> {
        let radix_of_a_write = |write_index: usize| {
            u8::try_from(tearable.landing_choices_of(write_index).count())
                .expect("一次写至多三态，进制装得进 u8")
        };
        let landing_digits = |write_indexes: &[usize]| {
            write_indexes
                .iter()
                .map(|write_index| {
                    (
                        CrashPointReplayDigitOfASegment::LandingOfAWrite(*write_index),
                        radix_of_a_write(*write_index),
                    )
                })
                .collect::<Vec<_>>()
        };
        match self {
            Self::NotExpanded => Vec::new(),
            Self::EveryProperSubset => landing_digits(segment),
            Self::InPlaceSubsetsWithCopyOnWriteNoneOrAll => {
                let (in_place_writes, copy_on_write_writes) = in_place_and_copy_on_write;
                let mut digits = Vec::with_capacity(in_place_writes.len() + 1);
                if !copy_on_write_writes.is_empty() {
                    digits.push((CrashPointReplayDigitOfASegment::CopyOnWriteGroup, 2));
                }
                digits.extend(landing_digits(in_place_writes));
                digits
            }
        }
    }
}

/// 一次写在一个崩溃状态里怎么落（D13（验证路线） 已定项 4；第三态是代码审阅第 4 条，用户 2026-09-27 定「原地覆写补第三态、全量也跑」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WriteLandingInCrashState {
    /// 没持久：那个位置上是崩溃前的旧字节。
    NotPersisted,
    /// 新旧都读不出：只有原地覆写的写取这一态，镜像里那一段是 [`torn_image_of_in_place_overwrite`] 的字节。
    Torn,
    /// 整次写都持久了。
    Persisted,
}

/// 一次写在崩溃状态里能取哪几态。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WriteLandingChoices {
    NotPersistedOrPersisted,
    NotPersistedTornOrPersisted,
}

impl WriteLandingChoices {
    fn count(self) -> u64 {
        match self {
            Self::NotPersistedOrPersisted => 2,
            Self::NotPersistedTornOrPersisted => 3,
        }
    }

    /// 混合进制里这一位上的数字是哪一态。最大的那个数字是「持久」：一段里每个写都取最大时就是整段全持久（展开时去掉的那一个，
    /// 它排在这一段序号的最后），只取两态的写上数字就是今天子集掩码的那一位（0 没持久、1 持久），次序与补第三态之前逐个相同。
    fn landing_of_digit(self, digit: u64) -> WriteLandingInCrashState {
        match self {
            Self::NotPersistedOrPersisted => match digit {
                0 => WriteLandingInCrashState::NotPersisted,
                1 => WriteLandingInCrashState::Persisted,
                out_of_range => unreachable!("两态的写上数字只有 0、1：拿到 {out_of_range}"),
            },
            Self::NotPersistedTornOrPersisted => match digit {
                0 => WriteLandingInCrashState::NotPersisted,
                1 => WriteLandingInCrashState::Torn,
                2 => WriteLandingInCrashState::Persisted,
                out_of_range => unreachable!("三态的写上数字只有 0、1、2：拿到 {out_of_range}"),
            },
        }
    }
}

/// 写表里哪几次写是原地覆写、撕裂时出第三态「新旧都读不出」（代码审阅第 4 条，用户 2026-09-27 定「原地覆写补第三态、全量也跑」；
/// 认法照 `research/prompts/m2-rev-b3a-implementer-report.md` 第四节），三条都成立才算：
/// 1. 不是单元写：单元写是 COW，落在分配器交出来的槽上，那一槽的旧内容没有谁还要（回收谓词管着），撕裂与没持久一样，照旧两态；
/// 2. 长于一个扇区：崩溃点重放的撕裂粒度是 [`SECTOR_BYTES`]，一个扇区的写撕不开（根槽写是一个物理块，照旧两态）；
/// 3. 它罩住的范围里原来有东西：基镜像那一段有非零字节，或写表里更早有一次同一块盘上、带字节（不是整段清零或 discard）的写与它重叠。
///
/// discard 照这三条认（D13（验证路线） 已定项 4 原地覆写的字面，判决 `research/prompts/m4-batch2-code-r1-main-verification.md` 第二节 S6）：
/// 它的「新字节」是 0，撕裂镜像前一半清了、后一半还是旧的；不长于一个扇区、或罩住的范围里原来全 0 的只取两态。
/// 一次 discard 在一个崩溃状态里只取这几态之一，「段内部分落地」（几个扇区落了、别的没落的任意组合）不另枚举。
///
/// 撕裂态原来并进「没持久」（D13（验证路线） 已定项 4 原句）；对原地覆写不成立——撕裂时那一处的旧内容也没了，「没持久」留着完整的旧内容。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TearableInPlaceOverwrites {
    is_tearable_by_write: Vec<bool>,
}

impl TearableInPlaceOverwrites {
    /// 按上面三条逐次认。
    ///
    /// # Panics
    /// 一次根槽写长于一个扇区又罩住了旧内容：第一版不支持根槽写的第三态（它的撕裂镜像接在枚举用的写表末尾，会被择根与记录核对器
    /// 当成又一条根槽写）。根槽写是一个物理块，崩溃点重放的池都按 512 字节的物理块建，走不到。
    #[must_use]
    pub fn of(base: &MemoryPool, writes: &[RetainedWrite]) -> Self {
        let is_tearable_by_write: Vec<bool> = (0..writes.len())
            .map(|write_index| is_tearable_in_place_overwrite(base, writes, write_index))
            .collect();
        for (write, is_tearable) in writes.iter().zip(&is_tearable_by_write) {
            assert!(
                !(*is_tearable && write.kind == StepKind::RootRecordFua),
                "根槽写（盘 {} 偏移 {}、{} 字节）长于一个扇区又罩住旧内容：第一版不支持根槽写的第三态",
                write.device.0,
                write.offset.0,
                write.length_in_bytes()
            );
        }
        Self {
            is_tearable_by_write,
        }
    }

    /// 每次写都只取两态（补第三态之前的口径）。
    #[must_use]
    pub fn none(write_count: usize) -> Self {
        Self {
            is_tearable_by_write: vec![false; write_count],
        }
    }

    /// 写表下标 `write_index` 那次写是不是原地覆写。
    #[must_use]
    pub fn contains(&self, write_index: usize) -> bool {
        self.is_tearable_by_write[write_index]
    }

    /// 原地覆写的写，按写表下标从小到大。
    #[must_use]
    pub fn write_indexes(&self) -> Vec<usize> {
        self.is_tearable_by_write
            .iter()
            .enumerate()
            .filter(|(_, is_tearable)| **is_tearable)
            .map(|(write_index, _)| write_index)
            .collect()
    }

    fn landing_choices_of(&self, write_index: usize) -> WriteLandingChoices {
        if self.is_tearable_by_write[write_index] {
            WriteLandingChoices::NotPersistedTornOrPersisted
        } else {
            WriteLandingChoices::NotPersistedOrPersisted
        }
    }

    /// 这几次写各取它的几态、全部组合的个数；越过 1280 位交回 `None`。
    fn checked_landing_combinations_of(
        &self,
        write_indexes: &[usize],
    ) -> Option<StateCount1280Bits> {
        write_indexes.iter().try_fold(
            StateCount1280Bits::from_u64(1),
            |combinations, write_index| {
                combinations.checked_mul_u64(self.landing_choices_of(*write_index).count())
            },
        )
    }
}

/// [`TearableInPlaceOverwrites`] 的三条判据，逐次判。
fn is_tearable_in_place_overwrite(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    write_index: usize,
) -> bool {
    let write = &writes[write_index];
    let is_copy_on_write = match write.kind {
        StepKind::UnitWrite => true,
        // discard 是原地的范围动作（「范围写零」）：照三条认，长于一个扇区、罩住旧内容的取第三态（D13（验证路线） 已定项 4 原地覆写的字面，
        // 判决 `research/prompts/m4-batch2-code-r1-main-verification.md` 第二节 S6）。
        StepKind::Discard => false,
        StepKind::ZeroFill
        | StepKind::JournalRecord
        | StepKind::RootRecordFua
        | StepKind::SystemConfigurationSlot
        | StepKind::Barrier => false,
    };
    if is_copy_on_write || write.length_in_bytes() <= SECTOR_BYTES {
        return false;
    }
    let write_end = write.offset.0 + write.length_in_bytes();
    let base_holds_nonzero_bytes_there = base.devices.get(&write.device).is_some_and(|device| {
        device
            .written_sectors()
            .range(write.offset.0 / SECTOR_BYTES..write_end.div_ceil(SECTOR_BYTES))
            .any(|(_, bytes)| bytes.iter().any(|byte| *byte != 0))
    });
    let an_earlier_write_with_bytes_overlaps = writes[..write_index].iter().any(|earlier| {
        let earlier_has_bytes = match earlier.contents {
            WrittenContents::Bytes(_) => true,
            WrittenContents::Zeros { .. } => false,
        };
        earlier_has_bytes
            && earlier.device == write.device
            && earlier.offset.0 < write_end
            && write.offset.0 < earlier.offset.0 + earlier.length_in_bytes()
    });
    base_holds_nonzero_bytes_there || an_earlier_write_with_bytes_overlaps
}

/// 一次原地覆写撕裂时那一段的字节，「新旧都读不出」：新旧不同的那一截（第一个不同的字节到最后一个不同的字节）前一半是新的、
/// 后一半还是旧的，这一截之外新旧本来就相同。第一个不同的字节是新的、最后一个是旧的，所以新旧有两个以上的字节不同时它与新、旧都不同；
/// 整段校验和罩着的写（系统配置槽、journal 记录）新旧两份校验和都对不上（校验和碰撞不在模型里，D13（验证路线） 已定项 4 射程）。
/// 新旧只差一个字节时撕不出第三种内容，交回旧的；完全相同时交回新的（也就是旧的）——这一态的镜像与另外两态之一逐字节相同。
///
/// # Panics
/// `old` 与这次写不一样长。
#[must_use]
pub fn torn_image_of_in_place_overwrite(old: &[u8], new: &WrittenContents) -> Vec<u8> {
    let length = usize::try_from(new.length_in_bytes()).expect("写长装得进 usize");
    assert_eq!(old.len(), length, "旧字节与这次写一样长");
    let mut new_bytes = vec![0u8; length];
    new.copy_range_into(0, &mut new_bytes);
    let differs = |index: &usize| old[*index] != new_bytes[*index];
    let Some(first_differing_byte) = (0..length).find(differs) else {
        return new_bytes;
    };
    let last_differing_byte = (0..length)
        .rev()
        .find(differs)
        .expect("有第一个不同的字节就有最后一个");
    let differing_span_in_bytes = last_differing_byte - first_differing_byte + 1;
    let first_old_byte = first_differing_byte + differing_span_in_bytes / 2;
    let mut torn = new_bytes;
    torn[first_old_byte..].copy_from_slice(&old[first_old_byte..]);
    torn
}

/// 枚举用的写表：录制流里的写原样在前（下标不变），其后每个原地覆写的写接一条撕裂镜像（罩住那次写整个范围、字节是
/// [`torn_image_of_in_place_overwrite`] 按「这次写之前的写全落了」时的旧字节算的），再接同一段里在它之后、同一块盘上与它重叠的写
/// 各一份重放——那几次写落了时要盖在撕裂镜像上面，与录制流里的先后一致（[`CrashImage`] 按写表次序叠）。
/// 一个状态的持久集合按这张表给：撕裂那一态是「原来那次写没持久、它的撕裂镜像持久」，重放跟着被重放的那次写落不落。
struct WritesWithTornImages {
    writes: Vec<RetainedWrite>,
    /// 原地覆写的写（写表下标）→ 它的撕裂镜像与之后那几份重放在 `writes` 里的位置。
    torn_image_by_write: BTreeMap<usize, TornImageOfOneWrite>,
}

struct TornImageOfOneWrite {
    torn_image_index: usize,
    /// （重放在 `writes` 里的下标，重放的是写表里哪一次写）。
    replays: Vec<(usize, usize)>,
}

impl WritesWithTornImages {
    fn of(
        base: &MemoryPool,
        writes: &[RetainedWrite],
        segments: &[Vec<usize>],
        tearable: &TearableInPlaceOverwrites,
    ) -> Self {
        let mut segment_of_write: BTreeMap<usize, usize> = BTreeMap::new();
        for (segment_index, segment) in segments.iter().enumerate() {
            for write_index in segment {
                segment_of_write.insert(*write_index, segment_index);
            }
        }
        let mut writes_with_torn_images = writes.to_vec();
        let mut torn_image_by_write = BTreeMap::new();
        let tearable_write_indexes = tearable.write_indexes();
        if tearable_write_indexes.is_empty() {
            return Self {
                writes: writes_with_torn_images,
                torn_image_by_write,
            };
        }
        // 每次写之前的镜像：基镜像上按次序叠写表里更早的写（这次写之前的写全落了）。
        let mut image_before_the_write = base.clone();
        let mut next_write_to_apply = 0usize;
        for write_index in tearable_write_indexes {
            image_before_the_write.apply_writes(&writes[next_write_to_apply..write_index]);
            next_write_to_apply = write_index;
            let write = &writes[write_index];
            let old = PoolReader::read(
                &image_before_the_write,
                write.device,
                write.offset,
                usize::try_from(write.length_in_bytes()).expect("写长装得进 usize"),
            )
            .expect("录到的写落在池里的盘上、在盘内、按扇区对齐");
            let torn_image_index = writes_with_torn_images.len();
            writes_with_torn_images.push(RetainedWrite {
                device: write.device,
                kind: write.kind,
                is_force_unit_access: write.is_force_unit_access,
                offset: write.offset,
                contents: WrittenContents::Bytes(torn_image_of_in_place_overwrite(
                    &old,
                    &write.contents,
                )),
            });
            let write_end = write.offset.0 + write.length_in_bytes();
            let later_overlapping_writes_in_the_segment: Vec<usize> = segment_of_write
                .get(&write_index)
                .map(|segment_index| {
                    segments[*segment_index]
                        .iter()
                        .copied()
                        .filter(|later_index| {
                            let later = &writes[*later_index];
                            *later_index > write_index
                                && later.device == write.device
                                && later.offset.0 < write_end
                                && write.offset.0 < later.offset.0 + later.length_in_bytes()
                        })
                        .collect()
                })
                .unwrap_or_default();
            let replays = later_overlapping_writes_in_the_segment
                .into_iter()
                .map(|replayed_write_index| {
                    writes_with_torn_images.push(writes[replayed_write_index].clone());
                    (writes_with_torn_images.len() - 1, replayed_write_index)
                })
                .collect();
            torn_image_by_write.insert(
                write_index,
                TornImageOfOneWrite {
                    torn_image_index,
                    replays,
                },
            );
        }
        Self {
            writes: writes_with_torn_images,
            torn_image_by_write,
        }
    }
}

/// 一段里的原地写与单元写（COW），各按段内次序：分法只有一份，在 [`crate::crash_facts::in_place_and_copy_on_write_writes_of_the_segment`]
/// （GPU 判表与第 ② 段核对的拆法也照它），这里只把写表下标换回 `usize`。
fn in_place_and_copy_on_write_writes_of(
    writes: &[RetainedWrite],
    segment: &[usize],
) -> (Vec<usize>, Vec<usize>) {
    let write_index_of = |index: u32| usize::try_from(index).expect("写表下标装得进 usize");
    let (in_place_writes, copy_on_write_writes) =
        crate::crash_facts::in_place_and_copy_on_write_writes_of_the_segment(writes, segment);
    (
        in_place_writes.into_iter().map(write_index_of).collect(),
        copy_on_write_writes
            .into_iter()
            .map(write_index_of)
            .collect(),
    )
}

/// 按 `expansion` 展开、每次写只取两态（没持久 / 持久，补第三态之前的口径）时的状态数：各段展开出来的，再加全部持久那一个
/// （全量时就是 [`closed_form_state_count`] 的闭式）。崩溃点重放枚举出来的状态数是
/// [`crash_state_count_with_torn_in_place_overwrites`]：写表里没有原地覆写的写时两者相等。
#[must_use]
pub fn crash_state_count(
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> u64 {
    state_count_of_the_segments(
        writes,
        segments,
        expansion,
        &TearableInPlaceOverwrites::none(writes.len()),
    )
}

/// 崩溃点重放按 `expansion` 枚举出来的状态数（`enumerate_crash_states` 一族的 `states`）：原地覆写的写（[`TearableInPlaceOverwrites`]，要看基镜像）
/// 取三态、其余取两态，各段展开出来的，再加全部持久那一个。
#[must_use]
pub fn crash_state_count_with_torn_in_place_overwrites(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> u64 {
    state_count_of_the_segments(
        writes,
        segments,
        expansion,
        &TearableInPlaceOverwrites::of(base, writes),
    )
}

fn state_count_of_the_segments(
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
    tearable: &TearableInPlaceOverwrites,
) -> u64 {
    segments
        .iter()
        .enumerate()
        .map(|(segment_index, segment)| {
            expansion(segment_index, segment).state_count(writes, segment, tearable)
        })
        .try_fold(1u64, u64::checked_add)
        .expect("整条流的状态数装得进 u64")
}

/// 每一段都按甲二展开（平时快档）。
#[must_use]
pub fn quick_tier_expansion(
    _segment_index: usize,
    _segment: &[usize],
) -> CrashPointReplaySegmentExpansion {
    CrashPointReplaySegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll
}

/// 每一段都展开任意真子集（全量）。
#[must_use]
pub fn full_expansion(
    _segment_index: usize,
    _segment: &[usize],
) -> CrashPointReplaySegmentExpansion {
    CrashPointReplaySegmentExpansion::EveryProperSubset
}

/// 枚举次序里每个状态的序号怎么落到「第几段、段内每次写怎么落」：次序与单线程逐段逐个组合走相同——
/// 前面的段全持久 + 当前段的一个组合（全量时段内序号按混合进制拆成每次写的一态，数到组合数 − 2；甲二时见
/// [`Self::persisted_writes_of_state`]），最后再加全部持久那一个状态。
struct CrashStatePlan<'plan> {
    segments: &'plan [Vec<usize>],
    /// 每一段怎么展开，与 `segments` 同序。
    expansion_by_segment: Vec<CrashPointReplaySegmentExpansion>,
    /// 每一段展开出来的状态的序号区间，首尾相接、随段号递增；不展开的段是空区间。
    state_ranges_by_segment: Vec<Range<u64>>,
    /// 状态总数：各段展开出来的，再加最后全部持久那一个。
    state_count: u64,
    /// 每一段归哪次发布（[`publish_of_each_segment`]），与 `segments` 同序。
    publish_of_segment: Vec<CrashPointReplayPublishOfState>,
    /// 每一段的原地写与单元写（[`in_place_and_copy_on_write_writes_of`]），与 `segments` 同序。
    in_place_and_copy_on_write_by_segment: Vec<(Vec<usize>, Vec<usize>)>,
    /// 分片的段拆编号要的各位与片起点（[`CrashPointReplaySegmentSlice`]），与 `segments` 同序；不分片的段是 `None`。
    digits_of_each_sliced_segment: Vec<Option<DigitsOfASlicedSegment>>,
    /// 哪几次写取三态（[`TearableInPlaceOverwrites`]）。
    tearable: &'plan TearableInPlaceOverwrites,
    /// 枚举用的写表（录制流里的写在前，撕裂镜像与重放接在后面）：持久集合按它给。
    writes_with_torn_images: &'plan WritesWithTornImages,
}

impl<'plan> CrashStatePlan<'plan> {
    /// `expansion` 只在调用线程上逐段问一次，所以它不必能跨线程。`writes` 是录制流里的写（不带撕裂镜像）。
    /// `segment_slices` 与 `segments` 同序：分片的段在流里的序号区间是片长，不分片的段是它按展开方式展开出的全部状态。
    ///
    /// # Panics
    /// `segment_slices` 与段数不同；分片的段不展开、或片越过了它的域（片起点 + 片长大于整域的态数）、或整域越过 1280 位；
    /// 不分片的段的态数装不进 u64；整条流的状态数装不进 u64。
    fn new(
        writes: &[RetainedWrite],
        segments: &'plan [Vec<usize>],
        expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
        segment_slices: &[Option<CrashPointReplaySegmentSlice>],
        tearable: &'plan TearableInPlaceOverwrites,
        writes_with_torn_images: &'plan WritesWithTornImages,
    ) -> Self {
        assert_eq!(
            segment_slices.len(),
            segments.len(),
            "每一段一格片（不分片的段是 None）"
        );
        let expansion_by_segment: Vec<CrashPointReplaySegmentExpansion> = segments
            .iter()
            .enumerate()
            .map(|(segment_index, segment)| expansion(segment_index, segment))
            .collect();
        let in_place_and_copy_on_write_by_segment: Vec<(Vec<usize>, Vec<usize>)> = segments
            .iter()
            .map(|segment| in_place_and_copy_on_write_writes_of(writes, segment))
            .collect();
        let digits_of_each_sliced_segment: Vec<Option<DigitsOfASlicedSegment>> = segments
            .iter()
            .enumerate()
            .map(|(segment_index, segment)| {
                segment_slices[segment_index].map(|slice| {
                    let segment_expansion = expansion_by_segment[segment_index];
                    let domain = segment_expansion
                        .checked_state_count(writes, segment, tearable)
                        .expect("分片的段整域的态数装得进 1280 位：录入段在写库之前核过");
                    let slice_end = slice
                        .start_within_the_domain
                        .checked_add_u64(slice.state_count)
                        .expect("片尾装得进 1280 位：不超过整域");
                    assert!(
                        segment_expansion != CrashPointReplaySegmentExpansion::NotExpanded
                            && slice_end <= domain,
                        "段 {segment_index} 的片 [{}, {}) 越过了它按 {} 展开的域（{} 个状态）",
                        slice.start_within_the_domain.decimal_text(),
                        slice_end.decimal_text(),
                        segment_expansion.name(),
                        domain.decimal_text()
                    );
                    let (positions, radices): (Vec<CrashPointReplayDigitOfASegment>, Vec<u8>) =
                        segment_expansion
                            .digits_of_the_segment(
                                segment,
                                &in_place_and_copy_on_write_by_segment[segment_index],
                                tearable,
                            )
                            .into_iter()
                            .unzip();
                    let slice_start_digits = slice
                        .start_within_the_domain
                        .mixed_radix_digits(&radices)
                        .expect("片起点小于整域的态数，拆得进各位");
                    DigitsOfASlicedSegment {
                        positions,
                        radices,
                        slice_start_digits,
                    }
                })
            })
            .collect();
        let mut next_ordinal = 0u64;
        let state_ranges_by_segment = segments
            .iter()
            .zip(&expansion_by_segment)
            .zip(segment_slices)
            .map(|((segment, segment_expansion), slice)| {
                let first_ordinal = next_ordinal;
                let states_of_the_segment = match slice {
                    Some(slice) => slice.state_count,
                    None => segment_expansion.state_count(writes, segment, tearable),
                };
                next_ordinal = next_ordinal
                    .checked_add(states_of_the_segment)
                    .expect("整条流的状态数装得进 u64");
                first_ordinal..next_ordinal
            })
            .collect();
        Self {
            segments,
            expansion_by_segment,
            state_ranges_by_segment,
            state_count: next_ordinal + 1,
            publish_of_segment: publish_of_each_segment(writes, segments),
            in_place_and_copy_on_write_by_segment,
            digits_of_each_sliced_segment,
            tearable,
            writes_with_torn_images,
        }
    }

    /// 段内序号 `ordinal_within` 按混合进制拆到 `write_indexes` 这几次写上（第一次写是最低位，每位的进制是它能取几态）。
    ///
    /// # Panics
    /// 序号超出这几次写的组合数（计划算错了）。
    fn assign_landings(
        &self,
        landings: &mut [WriteLandingInCrashState],
        ordinal_within: u64,
        write_indexes: &[usize],
    ) {
        let mut remaining = ordinal_within;
        for write_index in write_indexes {
            let choices = self.tearable.landing_choices_of(*write_index);
            landings[*write_index] = choices.landing_of_digit(remaining % choices.count());
            remaining /= choices.count();
        }
        assert_eq!(
            remaining, 0,
            "段内序号 {ordinal_within} 超出这几次写（{write_indexes:?}）的组合数"
        );
    }

    /// 序号落在哪一段；等于段数说明是最后全部持久那一个状态。
    fn segment_of_state(&self, ordinal: u64) -> usize {
        self.state_ranges_by_segment
            .partition_point(|state_range| state_range.end <= ordinal)
    }

    /// 这个状态归哪次发布：所在那一段归的那一次；最后全部持久那一个状态单列一格。
    fn publish_of_state(&self, ordinal: u64) -> CrashPointReplayPublishOfState {
        self.publish_of_segment
            .get(self.segment_of_state(ordinal))
            .copied()
            .unwrap_or(CrashPointReplayPublishOfState::EveryWritePersisted)
    }

    /// 这个状态的持久集合，按枚举用的写表（[`WritesWithTornImages`]）给：所在的段之前每一段整段持久（展不展开都一样），
    /// 所在的段按段内序号 r（序号减去这一段的起点）拆到段里每次写上（[`Self::assign_landings`]，原地覆写的写三态、其余两态）：
    /// 全量时 r 拆到段里全部的写上；甲二时单元写一个都没有就同全量（只拆到原地写上），有单元写时 r / 2 拆到原地写上、
    /// r 除以 2 的余数是「单元写全落」——次序是原地写的每个组合先单元写全不落、再全落，整段全落那一个取不到。
    /// 每次写都只取两态时 r 就是补第三态之前的子集掩码，次序逐个相同。撕裂那一态记成「原来那次写没持久、它的撕裂镜像持久」，
    /// 撕裂镜像之后的重放跟着被重放的那次写落不落。
    fn persisted_writes_of_state(&self, ordinal: u64) -> Vec<bool> {
        let segment_of_state = self.segment_of_state(ordinal);
        let recorded_write_count = self.tearable.is_tearable_by_write.len();
        let mut landings = vec![WriteLandingInCrashState::NotPersisted; recorded_write_count];
        for segment in &self.segments[..segment_of_state] {
            for write_index in segment {
                landings[*write_index] = WriteLandingInCrashState::Persisted;
            }
        }
        if let (Some(_segment), Some(Some(digits_of_the_slice))) = (
            self.segments.get(segment_of_state),
            self.digits_of_each_sliced_segment.get(segment_of_state),
        ) {
            // 分片的段：段内编号 = 片起点 + 片内第几个；片起点拆成的各位加上片内序号、逐位进位，再照每一位落
            let ordinal_within_the_slice =
                ordinal - self.state_ranges_by_segment[segment_of_state].start;
            let mut digits = digits_of_the_slice.slice_start_digits.clone();
            assert!(
                add_to_the_mixed_radix_digits(
                    &mut digits,
                    &digits_of_the_slice.radices,
                    ordinal_within_the_slice
                ),
                "段 {segment_of_state} 片内第 {ordinal_within_the_slice} 个越过了它的域（计划算错了）"
            );
            for (position, digit) in digits_of_the_slice.positions.iter().zip(&digits) {
                match position {
                    CrashPointReplayDigitOfASegment::CopyOnWriteGroup => {
                        if *digit == 1 {
                            let (_in_place_writes, copy_on_write_writes) =
                                &self.in_place_and_copy_on_write_by_segment[segment_of_state];
                            for write_index in copy_on_write_writes {
                                landings[*write_index] = WriteLandingInCrashState::Persisted;
                            }
                        }
                    }
                    CrashPointReplayDigitOfASegment::LandingOfAWrite(write_index) => {
                        landings[*write_index] = self
                            .tearable
                            .landing_choices_of(*write_index)
                            .landing_of_digit(u64::from(*digit));
                    }
                }
            }
        } else if let Some(segment) = self.segments.get(segment_of_state) {
            let ordinal_within_the_segment =
                ordinal - self.state_ranges_by_segment[segment_of_state].start;
            match self.expansion_by_segment[segment_of_state] {
                CrashPointReplaySegmentExpansion::NotExpanded => {
                    unreachable!(
                        "不展开的段序号区间是空的，没有状态落在它里面（段 {segment_of_state}）"
                    )
                }
                CrashPointReplaySegmentExpansion::EveryProperSubset => {
                    self.assign_landings(&mut landings, ordinal_within_the_segment, segment);
                }
                CrashPointReplaySegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll => {
                    let (in_place_writes, copy_on_write_writes) =
                        &self.in_place_and_copy_on_write_by_segment[segment_of_state];
                    if copy_on_write_writes.is_empty() {
                        self.assign_landings(
                            &mut landings,
                            ordinal_within_the_segment,
                            in_place_writes,
                        );
                    } else {
                        self.assign_landings(
                            &mut landings,
                            ordinal_within_the_segment / 2,
                            in_place_writes,
                        );
                        if ordinal_within_the_segment % 2 == 1 {
                            for write_index in copy_on_write_writes {
                                landings[*write_index] = WriteLandingInCrashState::Persisted;
                            }
                        }
                    }
                }
            }
        }
        let mut persisted = vec![false; self.writes_with_torn_images.writes.len()];
        for (write_index, landing) in landings.iter().enumerate() {
            persisted[write_index] = match landing {
                WriteLandingInCrashState::Persisted => true,
                WriteLandingInCrashState::NotPersisted | WriteLandingInCrashState::Torn => false,
            };
        }
        for (write_index, torn_image) in &self.writes_with_torn_images.torn_image_by_write {
            match landings[*write_index] {
                WriteLandingInCrashState::Torn => {
                    persisted[torn_image.torn_image_index] = true;
                    for (replay_index, replayed_write_index) in &torn_image.replays {
                        persisted[*replay_index] = match landings[*replayed_write_index] {
                            WriteLandingInCrashState::Persisted => true,
                            WriteLandingInCrashState::NotPersisted
                            | WriteLandingInCrashState::Torn => false,
                        };
                    }
                }
                WriteLandingInCrashState::NotPersisted | WriteLandingInCrashState::Persisted => {}
            }
        }
        persisted
    }
}

/// 长跑的枚举与核对按挂钟至少隔这么久打一行进度（`.claude/rules/implementation-workflow.md`「测试与崩溃检测优先多线程」的进度间隔 5 分钟）：
/// 一片、一块跑得比它久时，等着收的那条线程照样隔这么久打一行，不至于几个小时零输出。
pub const PROGRESS_LINE_INTERVAL_IN_SECONDS: u64 = 300;

/// 上一次收到消息、或上一次到期打按挂钟那一行是在什么时候（从这一趟开跑算起），离下一行按挂钟的到期还要等多久。
/// 时刻由调用方的时钟给，用例换成假的时钟。收到消息记的是收到的时刻，不是调用方真打出它那一行的时刻：调用方收到一条就打一行时两者只差
/// 打那一行的工夫；收到不打的（崩溃放量 CPU 块那一支，行在核对线程发消息之前打）差一个排队的时间，下一行按挂钟的最多晚这么一截。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WallClockProgressLines {
    interval: std::time::Duration,
    last_message_or_wall_clock_line_at: std::time::Duration,
}

impl WallClockProgressLines {
    #[must_use]
    pub fn new(interval: std::time::Duration, started_at: std::time::Duration) -> Self {
        Self {
            interval,
            last_message_or_wall_clock_line_at: started_at,
        }
    }

    /// 间隔取 [`PROGRESS_LINE_INTERVAL_IN_SECONDS`]。
    #[must_use]
    pub fn every_progress_line_interval(started_at: std::time::Duration) -> Self {
        Self::new(
            std::time::Duration::from_secs(PROGRESS_LINE_INTERVAL_IN_SECONDS),
            started_at,
        )
    }

    /// 离下一行到期还剩多久；已经到期是 0。
    #[must_use]
    pub fn wait_before_the_next_line_is_due(
        &self,
        now: std::time::Duration,
    ) -> std::time::Duration {
        self.interval
            .saturating_sub(now.saturating_sub(self.last_message_or_wall_clock_line_at))
    }

    /// 上一次收到消息或打按挂钟那一行之后，时钟走过了整个间隔。
    #[must_use]
    pub fn wall_clock_line_is_due(&self, now: std::time::Duration) -> bool {
        now.saturating_sub(self.last_message_or_wall_clock_line_at) >= self.interval
    }

    /// 记下 `now`：刚收到一条消息，或刚到期打了一行按挂钟的；下一行按挂钟的从这一刻起再等一个间隔。
    pub fn restart_the_interval_at_a_received_message_or_a_wall_clock_line(
        &mut self,
        now: std::time::Duration,
    ) {
        self.last_message_or_wall_clock_line_at = now;
    }
}

/// [`receive_until_the_senders_are_gone_with_wall_clock_lines`] 交给调用方的一件事：收到一条消息（调用方照旧按片、按块打它那一行），
/// 或者上一行之后过了整个间隔还没有新消息（调用方打一行按挂钟的进度）。
pub enum ReceivedOrWallClockLineDue<Message> {
    Received(Message),
    WallClockLineDue { elapsed: std::time::Duration },
}

/// 一直收到发送端全部丢掉为止：每收到一条交给 `handle`（调用方照它的约定按片、按块打一行或不打），记下收到的时刻；等的时候最多等到下一行到期，
/// 到期还没收到就交一件 [`ReceivedOrWallClockLineDue::WallClockLineDue`]，让调用方打一行按挂钟的进度。`clock` 交回从这一趟开跑算起的时刻。
/// `handle` panic 时原样往外传（接收端随调用方的栈一起丢掉，发送的线程下一次发不出去就退）。
pub fn receive_until_the_senders_are_gone_with_wall_clock_lines<Message>(
    receiver: &mpsc::Receiver<Message>,
    progress: &mut WallClockProgressLines,
    clock: &dyn Fn() -> std::time::Duration,
    handle: &mut dyn FnMut(ReceivedOrWallClockLineDue<Message>),
) {
    loop {
        let wait = progress.wait_before_the_next_line_is_due(clock());
        match receiver.recv_timeout(wait) {
            Ok(message) => {
                handle(ReceivedOrWallClockLineDue::Received(message));
                progress.restart_the_interval_at_a_received_message_or_a_wall_clock_line(clock());
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                let now = clock();
                if progress.wall_clock_line_is_due(now) {
                    handle(ReceivedOrWallClockLineDue::WallClockLineDue { elapsed: now });
                    progress.restart_the_interval_at_a_received_message_or_a_wall_clock_line(now);
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use singlefs_harness::memory_pool::{closed_form_state_count, SparseDevice};

    #[test]
    fn sparse_device_reads_zero_for_holes_and_flip_byte_only_touches_one_sector() {
        let mut device = SparseDevice::default();
        device.write(DeviceOffsetInBytes(1024), &[7u8; 1024]);
        assert_eq!(device.read(DeviceOffsetInBytes(512), 512), vec![0u8; 512]);
        assert_eq!(device.read(DeviceOffsetInBytes(1024), 512), vec![7u8; 512]);
        assert_eq!(
            device.written_sectors_in(DeviceOffsetInBytes(0), 4096),
            vec![2, 3]
        );
        let mut pool = MemoryPool::with_devices(&[DeviceIdentity(0)], 1 << 20);
        pool.devices
            .get_mut(&DeviceIdentity(0))
            .expect("盘")
            .write(DeviceOffsetInBytes(0), &[1u8; 1024]);
        pool.flip_byte(DeviceIdentity(0), DeviceOffsetInBytes(0), 600);
        let read_back =
            PoolReader::read(&pool, DeviceIdentity(0), DeviceOffsetInBytes(0), 1024).expect("读");
        assert_eq!(read_back[600], 0xfe);
        assert!(read_back
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 600 || *byte == 1));
        assert_eq!(closed_form_state_count(&[vec![0, 1], vec![2]]), 1 + 3 + 1);
    }

    /// 写表里一串按次序摆的写，种类照给的：盘 0、每次一个扇区、内容全 0（只核计划的用例不读内容）。
    fn writes_of_kinds(kinds: &[StepKind]) -> Vec<RetainedWrite> {
        kinds
            .iter()
            .enumerate()
            .map(|(index, kind)| RetainedWrite {
                device: DeviceIdentity(0),
                kind: *kind,
                is_force_unit_access: false,
                offset: DeviceOffsetInBytes(
                    u64::try_from(index).expect("下标装得进 u64") * SECTOR_BYTES,
                ),
                contents: WrittenContents::Bytes(vec![0u8; 512]),
            })
            .collect()
    }

    /// 甲二那一串持久集合的参照走法（与计划分开写）：逐段，段内先按原地写的子集掩码从 0 数到 2^k − 1，每个掩码先「单元写全不落」、
    /// 有单元写时再「单元写全落」，去掉整段全落那一个；每段走完整段持久。
    fn persisted_sets_walking_the_quick_tier(
        writes: &[RetainedWrite],
        segments: &[Vec<usize>],
    ) -> Vec<Vec<bool>> {
        let mut persisted_sets = Vec::new();
        let mut persisted_before = vec![false; writes.len()];
        for segment in segments {
            let in_place: Vec<usize> = segment
                .iter()
                .copied()
                .filter(|write_index| writes[*write_index].kind != StepKind::UnitWrite)
                .collect();
            let copy_on_write: Vec<usize> = segment
                .iter()
                .copied()
                .filter(|write_index| writes[*write_index].kind == StepKind::UnitWrite)
                .collect();
            for in_place_mask in 0..1u64 << in_place.len() {
                let mut with_in_place_subset = persisted_before.clone();
                for (bit, write_index) in in_place.iter().enumerate() {
                    if in_place_mask & (1 << bit) != 0 {
                        with_in_place_subset[*write_index] = true;
                    }
                }
                let every_in_place_write = in_place_mask == (1 << in_place.len()) - 1;
                if !(copy_on_write.is_empty() && every_in_place_write) {
                    persisted_sets.push(with_in_place_subset.clone());
                }
                if !copy_on_write.is_empty() && !every_in_place_write {
                    let mut with_every_copy_on_write = with_in_place_subset;
                    for write_index in &copy_on_write {
                        with_every_copy_on_write[*write_index] = true;
                    }
                    persisted_sets.push(with_every_copy_on_write);
                }
            }
            for write_index in segment {
                persisted_before[*write_index] = true;
            }
        }
        persisted_sets.push(persisted_before);
        persisted_sets
    }

    /// 甲二：计划按序号给出的持久集合与参照走法逐个相同，状态数等于闭式（有单元写的段 2^(k+1) − 1、没有的 2^k − 1，再加 1）；
    /// 全是单元写的段只有「全不落」一个状态，全是原地写的段同全量。
    #[test]
    fn the_quick_tier_plan_takes_every_in_place_subset_with_the_copy_on_write_writes_none_or_all() {
        let kinds = [
            StepKind::SystemConfigurationSlot,
            StepKind::SystemConfigurationSlot,
            StepKind::UnitWrite,
            StepKind::UnitWrite,
            StepKind::UnitWrite,
            StepKind::JournalRecord,
            StepKind::JournalRecord,
            StepKind::RootRecordFua,
            StepKind::UnitWrite,
            StepKind::UnitWrite,
            StepKind::SystemConfigurationSlot,
            StepKind::UnitWrite,
        ];
        let writes = writes_of_kinds(&kinds);
        let segments = vec![
            vec![0, 1, 2, 3, 4],
            vec![5, 6],
            vec![7],
            vec![8, 9],
            vec![10, 11],
        ];
        let walked = persisted_sets_walking_the_quick_tier(&writes, &segments);
        let tearable = TearableInPlaceOverwrites::none(writes.len());
        let writes_with_torn_images =
            WritesWithTornImages::of(&MemoryPool::default(), &writes, &segments, &tearable);
        let plan = CrashStatePlan::new(
            &writes,
            &segments,
            &quick_tier_expansion,
            &no_segment_slices(segments.len()),
            &tearable,
            &writes_with_torn_images,
        );
        // [2 原地 + 3 单元] 7、[2 原地] 3、[1 原地] 1、[2 单元] 1、[1 原地 + 1 单元] 3，再加全部持久那一个。
        assert_eq!(plan.state_count, 7 + 3 + 1 + 1 + 3 + 1);
        assert_eq!(
            crash_state_count(&writes, &segments, &quick_tier_expansion),
            plan.state_count
        );
        assert_eq!(
            plan.state_count,
            u64::try_from(walked.len()).expect("状态数"),
            "状态数与参照走法的相同"
        );
        let by_ordinal: Vec<Vec<bool>> = (0..plan.state_count)
            .map(|ordinal| plan.persisted_writes_of_state(ordinal))
            .collect();
        assert_eq!(by_ordinal, walked, "第 k 个状态就是参照走法的第 k 个");
        assert_eq!(
            crash_state_count(&writes, &segments, &full_expansion),
            closed_form_state_count(&segments),
            "全量的状态数就是闭式"
        );
    }

    /// 一次手摆的写：`length_in_bytes` 个 `fill` 字节。
    fn write_filled_with(
        device: u32,
        kind: StepKind,
        offset: u64,
        length_in_bytes: usize,
        fill: u8,
    ) -> RetainedWrite {
        RetainedWrite {
            device: DeviceIdentity(device),
            kind,
            is_force_unit_access: false,
            offset: DeviceOffsetInBytes(offset),
            contents: WrittenContents::Bytes(vec![fill; length_in_bytes]),
        }
    }

    /// 撕裂镜像的字节：新旧不同的那一截前一半新、后一半旧，与新、旧都不同；只差一个字节时撕不出第三种、交回旧的；完全相同交回新的；
    /// 整段清零照同一条算（新的是全 0）。
    #[test]
    fn the_torn_image_keeps_the_first_half_of_the_differing_bytes_new_and_the_rest_old() {
        let old = [1u8, 1, 1, 1, 1, 1, 1, 1];
        let new = WrittenContents::Bytes(vec![1, 2, 2, 2, 2, 2, 2, 1]);
        let torn = torn_image_of_in_place_overwrite(&old, &new);
        assert_eq!(torn, vec![1, 2, 2, 2, 1, 1, 1, 1]);
        assert_ne!(torn.as_slice(), old.as_slice(), "与旧的不同");
        assert_ne!(Some(torn.as_slice()), new.as_bytes(), "与新的不同");
        assert_eq!(
            torn_image_of_in_place_overwrite(
                &old,
                &WrittenContents::Bytes(vec![1, 1, 1, 9, 1, 1, 1, 1])
            ),
            old.to_vec(),
            "只差一个字节：撕不出第三种内容，交回旧的"
        );
        assert_eq!(
            torn_image_of_in_place_overwrite(&old, &WrittenContents::Bytes(old.to_vec())),
            old.to_vec(),
            "完全相同"
        );
        assert_eq!(
            torn_image_of_in_place_overwrite(&old, &WrittenContents::Zeros { length: 8 }),
            vec![0, 0, 0, 0, 1, 1, 1, 1],
            "整段清零：前一半清了、后一半还是旧的"
        );
    }

    /// 补第三态之后计划给出的持久集合：一段里原地覆写的写三态、其余两态，按混合进制（第一次写是最低位，没持久 0、撕裂 1、持久 2）
    /// 逐个组合各出一次、整段全持久那一个不出；撕裂那一态是「原来那次写没持久、它的撕裂镜像持久」，同一段里在它之后、与它重叠的写
    /// 落了时它的重放也持久（盖在撕裂镜像上面）。参照走法是三层循环，与计划的算法分开写。
    #[test]
    fn the_state_plan_with_torn_in_place_overwrites_hands_out_every_landing_combination_once() {
        const SYSTEM_CONFIGURATION_SLOT: u64 = 4096;
        const JOURNAL_SLOT: u64 = 16 << 20;
        const UNIT_SLOT: u64 = 784 << 20;
        let mut base = MemoryPool::with_devices(&[DeviceIdentity(0)], 1 << 30);
        base.devices
            .get_mut(&DeviceIdentity(0))
            .expect("盘 0")
            .write(
                DeviceOffsetInBytes(SYSTEM_CONFIGURATION_SLOT),
                &[0x5Au8; 4096],
            );
        let writes = vec![
            write_filled_with(
                0,
                StepKind::SystemConfigurationSlot,
                SYSTEM_CONFIGURATION_SLOT,
                4096,
                1,
            ),
            write_filled_with(0, StepKind::UnitWrite, UNIT_SLOT, 512, 2),
            write_filled_with(0, StepKind::JournalRecord, JOURNAL_SLOT, 4096, 3),
            write_filled_with(
                0,
                StepKind::SystemConfigurationSlot,
                SYSTEM_CONFIGURATION_SLOT,
                4096,
                4,
            ),
            write_filled_with(0, StepKind::JournalRecord, JOURNAL_SLOT, 4096, 5),
        ];
        // 写 0（基镜像那一槽有旧内容）、写 3（与更早的写 0 重叠）、写 4（与更早的写 2 重叠）是原地覆写；写 3 与写 0 同段、在它之后。
        let segments = vec![vec![0, 1, 3], vec![2], vec![4]];
        let tearable = TearableInPlaceOverwrites::of(&base, &writes);
        assert_eq!(tearable.write_indexes(), vec![0, 3, 4]);
        let writes_with_torn_images =
            WritesWithTornImages::of(&base, &writes, &segments, &tearable);
        // 写表后面接：写 0 的撕裂镜像（5）、写 3 的重放（6）、写 3 的撕裂镜像（7）、写 4 的撕裂镜像（8）。
        assert_eq!(writes_with_torn_images.writes.len(), 9);
        assert_eq!(
            writes_with_torn_images.writes[5].contents,
            WrittenContents::Bytes(torn_image_of_in_place_overwrite(
                &[0x5Au8; 4096],
                &writes[0].contents
            )),
            "写 0 的撕裂镜像按基镜像上的旧字节算"
        );
        assert_eq!(writes_with_torn_images.writes[6], writes[3], "写 3 的重放");
        assert_eq!(
            writes_with_torn_images.writes[7].contents,
            WrittenContents::Bytes(torn_image_of_in_place_overwrite(
                &[1u8; 4096],
                &writes[3].contents
            )),
            "写 3 的撕裂镜像按「写 0 已落」时的旧字节算"
        );
        let plan = CrashStatePlan::new(
            &writes,
            &segments,
            &full_expansion,
            &no_segment_slices(segments.len()),
            &tearable,
            &writes_with_torn_images,
        );
        // 第一段 3 · 2 · 3 − 1、第二段 2 − 1、第三段 3 − 1，再加全部持久那一个。
        assert_eq!(plan.state_count, 17 + 1 + 2 + 1);
        assert_eq!(
            crash_state_count_with_torn_in_place_overwrites(
                &base,
                &writes,
                &segments,
                &full_expansion
            ),
            plan.state_count
        );
        use WriteLandingInCrashState::{NotPersisted, Persisted, Torn};
        let torn_image_index_of = |write_index: usize| match write_index {
            0 => Some(5),
            3 => Some(7),
            4 => Some(8),
            _ => None,
        };
        let landing_of = |persisted: &[bool], write_index: usize| {
            if persisted[write_index] {
                Persisted
            } else if torn_image_index_of(write_index).is_some_and(|index| persisted[index]) {
                Torn
            } else {
                NotPersisted
            }
        };
        let mut expected: Vec<[WriteLandingInCrashState; 5]> = Vec::new();
        for landing_of_write_three in [NotPersisted, Torn, Persisted] {
            for landing_of_write_one in [NotPersisted, Persisted] {
                for landing_of_write_zero in [NotPersisted, Torn, Persisted] {
                    expected.push([
                        landing_of_write_zero,
                        landing_of_write_one,
                        NotPersisted,
                        landing_of_write_three,
                        NotPersisted,
                    ]);
                }
            }
        }
        expected.pop();
        expected.push([Persisted, Persisted, NotPersisted, Persisted, NotPersisted]);
        expected.push([Persisted, Persisted, Persisted, Persisted, NotPersisted]);
        expected.push([Persisted, Persisted, Persisted, Persisted, Torn]);
        expected.push([Persisted; 5]);
        let mut by_ordinal: Vec<[WriteLandingInCrashState; 5]> = Vec::new();
        for ordinal in 0..plan.state_count {
            let persisted = plan.persisted_writes_of_state(ordinal);
            assert_eq!(persisted.len(), 9, "持久集合与枚举用的写表逐条对应");
            let landings = [0, 1, 2, 3, 4].map(|write_index| landing_of(&persisted, write_index));
            assert_eq!(
                persisted[6],
                landings[0] == Torn && landings[3] == Persisted,
                "写 3 的重放只在写 0 撕裂、写 3 落了时持久（第 {ordinal} 个状态）"
            );
            by_ordinal.push(landings);
        }
        assert_eq!(by_ordinal, expected, "第 k 个状态就是参照走法的第 k 个");
        // 写 0 撕裂、写 3 落了：那一槽读出来是写 3 的字节（重放盖在撕裂镜像上面）；写 3 没落：读出来是写 0 的撕裂镜像。
        let slot_in_state = |landing_of_write_three: WriteLandingInCrashState| {
            let ordinal = by_ordinal
                .iter()
                .position(|landings| {
                    landings[0] == Torn
                        && landings[1] == NotPersisted
                        && landings[3] == landing_of_write_three
                })
                .expect("有这个状态");
            let image = CrashImage {
                base: &base,
                writes: &writes_with_torn_images.writes,
                persisted: plan.persisted_writes_of_state(u64::try_from(ordinal).expect("序号")),
            };
            PoolReader::read(
                &image,
                DeviceIdentity(0),
                DeviceOffsetInBytes(SYSTEM_CONFIGURATION_SLOT),
                4096,
            )
            .expect("读得出")
        };
        assert_eq!(slot_in_state(Persisted), vec![4u8; 4096]);
        assert_eq!(
            Some(slot_in_state(NotPersisted).as_slice()),
            writes_with_torn_images.writes[5].contents.as_bytes()
        );
    }
    /// 1280 位态数（用户 2026-10-13 定的位宽）：3^290 − 1（位置寻址树第三条流记录段 290 次原地写那一形的整域上界）按十进制逐字写出，
    /// 不饱和、不写成幂；拆成 290 位三进制每位都是 2、再拼回来是同一个数；装得进 u64 的照样取得出 u64。
    #[test]
    fn three_to_the_two_hundred_ninetieth_minus_one_is_written_in_decimal_exactly() {
        let mut power = StateCount1280Bits::from_u64(1);
        for _ in 0..290 {
            power = power
                .checked_mul_u64(3)
                .expect("3^290 ≈ 2^460，装得进 1280 位");
        }
        let domain = power
            .checked_sub(&StateCount1280Bits::from_u64(1))
            .expect("3^290 不小于 1");
        assert_eq!(
            domain.decimal_text(),
            "2318269218083089908234280468459894595445540070540355238386428052606631732586480323016481331702297098820479338186895528168360391078014797448"
        );
        assert_eq!(domain.to_u64(), None, "3^290 − 1 装不进 u64");
        let radices = vec![3u8; 290];
        let digits = domain.mixed_radix_digits(&radices).expect("小于 3^290");
        assert!(
            digits.iter().all(|digit| *digit == 2),
            "3^290 − 1 的每一位三进制都是 2"
        );
        assert_eq!(
            StateCount1280Bits::of_mixed_radix_digits(&digits, &radices),
            Some(domain)
        );
        assert_eq!(
            power.mixed_radix_digits(&radices),
            None,
            "3^290 本身拆不进 290 位三进制"
        );
        assert_eq!(
            StateCount1280Bits::from_u64(u64::MAX).to_u64(),
            Some(u64::MAX)
        );
        assert_eq!(
            StateCount1280Bits::from_u64(u64::MAX)
                .checked_add_u64(1)
                .map(|value| value.decimal_text()),
            Some("18446744073709551616".to_string())
        );
        assert_eq!(StateCount1280Bits::ZERO.decimal_text(), "0");
        assert_eq!(
            StateCount1280Bits::from_le_bytes(&domain.to_le_bytes()),
            domain,
            "小端字节往返不丢"
        );
    }

    /// 越过 1280 位交 `None`、不饱和：2^1279 再乘 2、1280 位全 1 再加 1、0 减 1 都拒。
    #[test]
    fn a_state_count_past_1280_bits_is_refused_not_saturated() {
        let mut highest_bit = StateCount1280Bits::from_u64(1);
        for _ in 0..1279 {
            highest_bit = highest_bit
                .checked_mul_u64(2)
                .expect("2^1279 装得进 1280 位");
        }
        assert_eq!(
            highest_bit.checked_mul_u64(2),
            None,
            "2^1280 装不进 1280 位"
        );
        let every_bit = highest_bit
            .checked_sub(&StateCount1280Bits::from_u64(1))
            .and_then(|below| below.checked_add(&highest_bit))
            .expect("2^1280 − 1 装得进");
        assert_eq!(
            every_bit.checked_add_u64(1),
            None,
            "1280 位全 1 再加 1 越过位宽"
        );
        assert_eq!(
            StateCount1280Bits::ZERO.checked_sub(&StateCount1280Bits::from_u64(1)),
            None
        );
        assert!(highest_bit > StateCount1280Bits::from_u64(u64::MAX));
    }

    /// 甲二一段：两次原地覆写（三态）与一组两次单元写（整组落或不落），3·3·2 − 1 = 17 个态。分片的计划按片起点 + 片内序号拆出的持久集合，
    /// 与不分片的计划在段内同一个编号上拆出的逐个相同；几片首尾相接罩住整域时，各片拆出的态合起来与整域逐个相同、一个不多一个不少（编号到态是双射）。
    #[test]
    fn slices_of_an_in_place_subsets_segment_decode_the_same_states_as_the_whole_segment() {
        const SYSTEM_CONFIGURATION_SLOT: u64 = 4096;
        const SECOND_SYSTEM_CONFIGURATION_SLOT: u64 = 8 << 20;
        const UNIT_SLOT: u64 = 784 << 20;
        let mut base = MemoryPool::with_devices(&[DeviceIdentity(0)], 1 << 30);
        for slot in [SYSTEM_CONFIGURATION_SLOT, SECOND_SYSTEM_CONFIGURATION_SLOT] {
            base.devices
                .get_mut(&DeviceIdentity(0))
                .expect("盘 0")
                .write(DeviceOffsetInBytes(slot), &[0x5Au8; 4096]);
        }
        let writes = vec![
            write_filled_with(
                0,
                StepKind::SystemConfigurationSlot,
                SYSTEM_CONFIGURATION_SLOT,
                4096,
                1,
            ),
            write_filled_with(0, StepKind::UnitWrite, UNIT_SLOT, 512, 2),
            write_filled_with(
                0,
                StepKind::SystemConfigurationSlot,
                SECOND_SYSTEM_CONFIGURATION_SLOT,
                4096,
                3,
            ),
            write_filled_with(0, StepKind::UnitWrite, UNIT_SLOT + 16384, 512, 4),
        ];
        let segments = vec![vec![0, 1, 2, 3]];
        let tearable = TearableInPlaceOverwrites::of(&base, &writes);
        assert_eq!(
            tearable.write_indexes(),
            vec![0, 2],
            "两次系统配置槽写都罩住旧内容、取三态"
        );
        let writes_with_torn_images =
            WritesWithTornImages::of(&base, &writes, &segments, &tearable);
        let whole = CrashStatePlan::new(
            &writes,
            &segments,
            &quick_tier_expansion,
            &no_segment_slices(1),
            &tearable,
            &writes_with_torn_images,
        );
        assert_eq!(whole.state_count, 17 + 1);
        let mut states_of_the_slices = Vec::new();
        for (start, state_count) in [(0u64, 5u64), (5, 7), (12, 5)] {
            let slices = vec![Some(CrashPointReplaySegmentSlice {
                start_within_the_domain: StateCount1280Bits::from_u64(start),
                state_count,
            })];
            let sliced = CrashStatePlan::new(
                &writes,
                &segments,
                &quick_tier_expansion,
                &slices,
                &tearable,
                &writes_with_torn_images,
            );
            assert_eq!(
                sliced.state_count,
                state_count + 1,
                "片长加最后全部持久那一个"
            );
            for ordinal_within_the_slice in 0..state_count {
                let persisted = sliced.persisted_writes_of_state(ordinal_within_the_slice);
                assert_eq!(
                    persisted,
                    whole.persisted_writes_of_state(start + ordinal_within_the_slice),
                    "片 [{start}, {}) 第 {ordinal_within_the_slice} 个是段内编号 {} 的那个态",
                    start + state_count,
                    start + ordinal_within_the_slice
                );
                states_of_the_slices.push(persisted);
            }
        }
        let states_of_the_whole_segment: Vec<Vec<bool>> = (0..17)
            .map(|ordinal| whole.persisted_writes_of_state(ordinal))
            .collect();
        assert_eq!(states_of_the_slices, states_of_the_whole_segment);
        let distinct: std::collections::BTreeSet<Vec<bool>> =
            states_of_the_slices.iter().cloned().collect();
        assert_eq!(distinct.len(), 17, "17 个编号拆出 17 个不同的态");
    }

    /// 全量展开的一段（三次两态写，2^3 − 1 = 7 个态）同上：从段内编号 3 起的一片与整段在同一个编号上拆出的态逐个相同。
    #[test]
    fn a_slice_of_an_every_proper_subset_segment_decodes_the_same_states_as_the_whole_segment() {
        let writes = writes_of_kinds(&[
            StepKind::JournalRecord,
            StepKind::JournalRecord,
            StepKind::JournalRecord,
        ]);
        let segments = vec![vec![0, 1, 2]];
        let tearable = TearableInPlaceOverwrites::none(writes.len());
        let writes_with_torn_images =
            WritesWithTornImages::of(&MemoryPool::default(), &writes, &segments, &tearable);
        let whole = CrashStatePlan::new(
            &writes,
            &segments,
            &full_expansion,
            &no_segment_slices(1),
            &tearable,
            &writes_with_torn_images,
        );
        let slices = vec![Some(CrashPointReplaySegmentSlice {
            start_within_the_domain: StateCount1280Bits::from_u64(3),
            state_count: 4,
        })];
        let sliced = CrashStatePlan::new(
            &writes,
            &segments,
            &full_expansion,
            &slices,
            &tearable,
            &writes_with_torn_images,
        );
        for ordinal_within_the_slice in 0..4 {
            assert_eq!(
                sliced.persisted_writes_of_state(ordinal_within_the_slice),
                whole.persisted_writes_of_state(3 + ordinal_within_the_slice),
                "片内第 {ordinal_within_the_slice} 个"
            );
        }
    }

    /// 片越过段的域（片起点 + 片长大于整域的态数）：建计划时停下，不拆出域外的态。
    #[test]
    #[should_panic(expected = "越过了它按 every_proper_subset 展开的域")]
    fn a_slice_past_the_domain_of_its_segment_is_refused_when_the_plan_is_built() {
        let writes = writes_of_kinds(&[StepKind::JournalRecord, StepKind::JournalRecord]);
        let segments = vec![vec![0, 1]];
        let tearable = TearableInPlaceOverwrites::none(writes.len());
        let writes_with_torn_images =
            WritesWithTornImages::of(&MemoryPool::default(), &writes, &segments, &tearable);
        let slices = vec![Some(CrashPointReplaySegmentSlice {
            start_within_the_domain: StateCount1280Bits::from_u64(2),
            state_count: 2,
        })];
        let _plan = CrashStatePlan::new(
            &writes,
            &segments,
            &full_expansion,
            &slices,
            &tearable,
            &writes_with_torn_images,
        );
    }

    /// 混合进位逐位进位地加：[2, 1]（三进制、两进制，值 2 + 3 = 5）加 1 得 [0, 0] 带出进位（越过 6），加 0 不动；
    /// [0, 0, 0] 加 13（三、二、三进制）得 [1, 0, 2]。
    #[test]
    fn adding_to_mixed_radix_digits_carries_digit_by_digit() {
        let mut digits = vec![2u8, 1];
        assert!(
            !add_to_the_mixed_radix_digits(&mut digits, &[3, 2], 1),
            "5 + 1 = 6 越过 3 × 2"
        );
        let mut unchanged = vec![2u8, 1];
        assert!(add_to_the_mixed_radix_digits(&mut unchanged, &[3, 2], 0));
        assert_eq!(unchanged, vec![2, 1]);
        let mut thirteen = vec![0u8, 0, 0];
        assert!(add_to_the_mixed_radix_digits(&mut thirteen, &[3, 2, 3], 13));
        assert_eq!(thirteen, vec![1, 0, 2], "13 = 1 + 3 × (0 + 2 × 2)");
    }

    /// 单元区里一个槽的起点（崩溃点重放的池按 512 字节物理块建，单元区从 784 MiB 起）。
    const DISCARDED_UNIT_SLOT: u64 = 784 << 20;
    /// 一次 discard 罩住的字节数：一个 16 KiB 槽。
    const DISCARD_OF_ONE_SLOT_BYTES: u64 = 16384;

    /// 一次 discard（盘 0、`offset` 起 `length_in_bytes` 字节），写表里按「范围写零」记。
    fn discard_write(offset: u64, length_in_bytes: u64) -> RetainedWrite {
        RetainedWrite {
            device: DeviceIdentity(0),
            kind: StepKind::Discard,
            is_force_unit_access: false,
            offset: DeviceOffsetInBytes(offset),
            contents: WrittenContents::Zeros {
                length: length_in_bytes,
            },
        }
    }

    /// 基镜像：盘 0 上 `DISCARDED_UNIT_SLOT` 起一整槽是旧的非零字节（discard 罩住的范围里原来有东西）。
    fn base_with_old_bytes_in_the_discarded_slot() -> MemoryPool {
        let mut base = MemoryPool::with_devices(&[DeviceIdentity(0)], 1 << 30);
        base.devices
            .get_mut(&DeviceIdentity(0))
            .expect("盘 0")
            .write(DeviceOffsetInBytes(DISCARDED_UNIT_SLOT), &[0x6Bu8; 16384]);
        base
    }

    /// 一次 16 KiB 的 discard 罩住非零旧内容：照 D13（验证路线） 已定项 4 原地覆写的字面取三态（判决 m4-batch2-code-r1 S6），
    /// 全量与快档都是 1 + (3 − 1) 个状态；撕裂那一态的镜像那一槽前一半是 0、后一半还是旧字节，没持久那一态整槽是旧字节，
    /// 持久那一态整槽是 0。判据改回「discard 只取两态」时这一条红在「是原地覆写」那一句。
    #[test]
    fn a_discard_longer_than_a_sector_over_old_contents_enumerates_a_torn_third_state() {
        let base = base_with_old_bytes_in_the_discarded_slot();
        let writes = vec![discard_write(
            DISCARDED_UNIT_SLOT,
            DISCARD_OF_ONE_SLOT_BYTES,
        )];
        let segments = vec![vec![0]];
        let tearable = TearableInPlaceOverwrites::of(&base, &writes);
        assert!(
            tearable.contains(0),
            "16 KiB 的 discard 罩住非零旧内容：是原地覆写，多取第三态"
        );
        for expansion in [
            &full_expansion as &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
            &quick_tier_expansion,
        ] {
            assert_eq!(
                crash_state_count_with_torn_in_place_overwrites(
                    &base, &writes, &segments, expansion
                ),
                3,
                "没持久、撕裂两个，再加全部持久那一个（全量与快档相同）"
            );
        }
        let tables = crash_state_enumeration_tables(&base, &writes, &segments, &full_expansion);
        assert_eq!(tables.torn_images.len(), 1, "这次 discard 接一条撕裂镜像");
        let torn_image_index = tables.torn_images[0].torn_image_index;
        let mut expected_torn_bytes = vec![0x6Bu8; 16384];
        expected_torn_bytes[..8192].fill(0);
        assert_eq!(
            tables.enumerated_writes[torn_image_index].contents,
            WrittenContents::Bytes(expected_torn_bytes.clone()),
            "撕裂镜像：前一半清了、后一半还是旧字节"
        );
        let writes_with_torn_images =
            WritesWithTornImages::of(&base, &writes, &segments, &tearable);
        let plan = CrashStatePlan::new(
            &writes,
            &segments,
            &full_expansion,
            &no_segment_slices(segments.len()),
            &tearable,
            &writes_with_torn_images,
        );
        let torn_ordinal = (0..plan.state_count)
            .find(|ordinal| {
                let persisted = plan.persisted_writes_of_state(*ordinal);
                !persisted[0] && persisted[torn_image_index]
            })
            .expect("有一个状态是这次 discard 撕裂");
        let torn_state = CrashImage {
            base: &base,
            writes: &writes_with_torn_images.writes,
            persisted: plan.persisted_writes_of_state(torn_ordinal),
        };
        assert_eq!(
            PoolReader::read(
                &torn_state,
                DeviceIdentity(0),
                DeviceOffsetInBytes(DISCARDED_UNIT_SLOT),
                16384
            ),
            Some(expected_torn_bytes),
            "撕裂那一态的镜像真的生成出来：那一槽是撕裂镜像的字节"
        );
    }

    /// 不长于一个扇区的 discard、罩住的范围里原来全 0 的 discard 都只取两态（D13（验证路线） 已定项 4：不是原地覆写的写，撕裂并进「没持久」）：
    /// 两段各 2 − 1 个，再加全部持久那一个；写表后面不接撕裂镜像。
    #[test]
    fn a_discard_of_one_sector_or_over_zeros_takes_only_two_states() {
        let base = base_with_old_bytes_in_the_discarded_slot();
        let writes = vec![
            discard_write(DISCARDED_UNIT_SLOT, SECTOR_BYTES),
            discard_write(DISCARDED_UNIT_SLOT + (1 << 20), DISCARD_OF_ONE_SLOT_BYTES),
        ];
        let segments = vec![vec![0], vec![1]];
        let tearable = TearableInPlaceOverwrites::of(&base, &writes);
        assert_eq!(
            tearable.write_indexes(),
            Vec::<usize>::new(),
            "一个扇区的 discard 撕不开；罩住全 0 的 discard 原来没东西：都不是原地覆写"
        );
        assert_eq!(
            crash_state_count_with_torn_in_place_overwrites(
                &base,
                &writes,
                &segments,
                &full_expansion
            ),
            1 + 1 + 1
        );
        assert_eq!(
            crash_state_enumeration_tables(&base, &writes, &segments, &full_expansion)
                .enumerated_writes
                .len(),
            2,
            "写表后面不接撕裂镜像"
        );
    }
}

#[cfg(test)]
mod oracle_instance_tests {
    use super::{
        classified_oracle_violation_for_the_version_table,
        classified_oracle_violation_for_versions, oracle_violation_for_versions,
        CrashPointReplayOracleViolationKind, CrashPointReplayVersionTable, PublishedVersion,
        PublishedVersionWithoutAFile,
    };
    use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
    use singlefs_core::recovery::RecoveryOutcome;

    /// 两条同 txg 不同实例的版本（设备失而复得、或回退实例与被抛弃实例同时在环里会造出来）。
    fn two_instances_at_txg_seven() -> Vec<PublishedVersion> {
        vec![
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(7),
                content: b"written by instance one".to_vec(),
            },
            PublishedVersion {
                instance: InstanceGeneration(2),
                checkpoint_txg: CheckpointTxg(7),
                content: b"written by instance two".to_vec(),
            },
        ]
    }

    /// D22（单元原子性怎么合成） 已定项 7：txg 平局按实例代号高者赢。实例 2 的第 7 代根已持久而恢复落在实例 1 的第 7 代根上是一次退代。
    #[test]
    fn landing_on_the_lower_instance_of_the_same_txg_is_a_violation() {
        let outcome = RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(7)),
            content: b"written by instance one".to_vec(),
        };
        let violation = oracle_violation_for_versions(
            &outcome,
            Some((InstanceGeneration(1), CheckpointTxg(7))),
            Some((CheckpointTxg(7), InstanceGeneration(2))),
            &two_instances_at_txg_seven(),
        );
        assert!(
            violation.is_some(),
            "只按 txg 比会把实例 1 的第 7 代当成最新的"
        );
    }

    /// 走到一个没有版本的更新的根上报「没有文件」，而更旧的根下面有文件：违例，不许因为查不到版本就放过。
    #[test]
    fn newer_root_without_any_version_reporting_no_file_is_violation() {
        let outcome = RecoveryOutcome::NoFile {
            root: (InstanceGeneration(1), CheckpointTxg(9)),
        };
        assert!(oracle_violation_for_versions(
            &outcome,
            Some((InstanceGeneration(1), CheckpointTxg(9))),
            Some((CheckpointTxg(7), InstanceGeneration(2))),
            &two_instances_at_txg_seven(),
        )
        .is_some());
        // 比每条版本都旧的根（暖机）报没有文件照旧不是违例。
        let warm_up = RecoveryOutcome::NoFile {
            root: (InstanceGeneration(1), CheckpointTxg(2)),
        };
        assert_eq!(
            oracle_violation_for_versions(
                &warm_up,
                Some((InstanceGeneration(1), CheckpointTxg(2))),
                Some((CheckpointTxg(2), InstanceGeneration(1))),
                &two_instances_at_txg_seven(),
            ),
            None
        );
    }

    /// 多记录流（`crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs`）的版本表：实例 1 的
    /// A、B、C 三次带文件的发布落在第 3、4、5 代（第 1、2 代是两次暖机，根下面没有文件）。
    fn the_multi_record_publish_stream_versions() -> Vec<PublishedVersion> {
        [
            (3, b"A".as_slice()),
            (4, b"B".as_slice()),
            (5, b"C".as_slice()),
        ]
        .into_iter()
        .map(|(checkpoint_txg, content)| PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(checkpoint_txg),
            content: content.to_vec(),
        })
        .collect()
    }

    /// 改「根不在表里只看比它旧的最新一版」之前的判法（逐字照那一版的分支写）：报没有文件、根不在表里，任一更旧版有文件就是违例。
    fn kind_judged_by_any_older_version_with_a_file(
        outcome: &RecoveryOutcome,
        effective_root: (InstanceGeneration, CheckpointTxg),
        newest_persisted_root: Option<(CheckpointTxg, InstanceGeneration)>,
        versions: &[PublishedVersion],
    ) -> Option<CrashPointReplayOracleViolationKind> {
        let (effective_instance, effective_txg) = effective_root;
        if newest_persisted_root.is_some_and(|newest| (effective_txg, effective_instance) < newest)
        {
            return Some(CrashPointReplayOracleViolationKind::RecoveredToAnOlderRoot);
        }
        let version = versions.iter().find(|version| {
            version.checkpoint_txg == effective_txg && version.instance == effective_instance
        });
        match (outcome, version) {
            (RecoveryOutcome::Failed { .. }, _) => {
                Some(CrashPointReplayOracleViolationKind::ReadFailed)
            }
            (RecoveryOutcome::NoFile { .. }, None) => versions
                .iter()
                .any(|candidate_version| {
                    (candidate_version.checkpoint_txg, candidate_version.instance)
                        < (effective_txg, effective_instance)
                })
                .then_some(CrashPointReplayOracleViolationKind::NoFileWhereAnOlderRootHasOne),
            (RecoveryOutcome::NoFile { .. }, Some(_)) => {
                Some(CrashPointReplayOracleViolationKind::NoFileWhereThisRootHasOne)
            }
            (RecoveryOutcome::FileRead { .. }, None) => {
                Some(CrashPointReplayOracleViolationKind::FileReadWhereThisRootHasNone)
            }
            (RecoveryOutcome::FileRead { content, .. }, Some(version)) => (*content
                != version.content)
                .then_some(CrashPointReplayOracleViolationKind::WrongContent),
        }
    }

    /// 文件从不消失的流上，「根不在表里只看比它旧的最新一版」与「任一更旧版有文件」判得逐项相同：多记录流与两实例同 txg 两张版本表，
    /// 实例 0–2、第 0–8 代的每个根，报没有文件、读出表里每一版的内容、读出表里没有的内容，盘上最新根槽没有或就是这个根，两种判法交回的违例类相同。
    #[test]
    fn a_table_where_the_file_never_vanishes_judges_a_root_not_in_it_like_any_older_version_with_a_file(
    ) {
        for versions in [
            the_multi_record_publish_stream_versions(),
            two_instances_at_txg_seven(),
        ] {
            let mut judged_pairs = 0usize;
            for instance in 0..=2 {
                for checkpoint_txg in 0..=8 {
                    let root = (InstanceGeneration(instance), CheckpointTxg(checkpoint_txg));
                    let mut outcomes = vec![RecoveryOutcome::NoFile { root }];
                    outcomes.extend(
                        versions
                            .iter()
                            .map(|version| version.content.clone())
                            .chain([b"not written by any version".to_vec()])
                            .map(|content| RecoveryOutcome::FileRead { root, content }),
                    );
                    for outcome in &outcomes {
                        for newest_persisted_root in [None, Some((root.1, root.0))] {
                            assert_eq!(
                                classified_oracle_violation_for_versions(
                                    outcome,
                                    Some(root),
                                    newest_persisted_root,
                                    &versions,
                                )
                                .map(|violation| violation.kind),
                                kind_judged_by_any_older_version_with_a_file(
                                    outcome,
                                    root,
                                    newest_persisted_root,
                                    &versions,
                                ),
                                "根 {root:?}、{outcome:?}、盘上最新根槽 {newest_persisted_root:?}：两种判法应当相同"
                            );
                            judged_pairs += 1;
                        }
                    }
                }
            }
            assert_eq!(
                judged_pairs,
                3 * 9 * (versions.len() + 2) * 2,
                "每个根、每种结局、两种最新根槽都判到了"
            );
        }
    }

    /// 多记录流之后 unlink 的版本表（`crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs`）：
    /// A、B、C 有文件，第 6 代（删除意图那次）起没有文件，之后两批与最后一批（第 7–9 代）不进表。
    fn versions_without_a_file_from_txg_six() -> Vec<PublishedVersionWithoutAFile> {
        vec![PublishedVersionWithoutAFile {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(6),
        }]
    }

    fn kind_judged_against_the_table(
        outcome: &RecoveryOutcome,
        effective_root: (InstanceGeneration, CheckpointTxg),
        versions: CrashPointReplayVersionTable<'_>,
    ) -> Option<CrashPointReplayOracleViolationKind> {
        classified_oracle_violation_for_the_version_table(
            outcome,
            Some(effective_root),
            Some((effective_root.1, effective_root.0)),
            versions,
        )
        .map(|violation| violation.kind)
    }

    /// D8（核心索引结构） 已定项 16 第 4 条：意图那次起冷走读答没有文件。落在意图那一版、或落在它之后不在表里的根（各批）上报没有文件不是违例；
    /// 在那几个根上读出内容、落在 C 上报没有文件照旧是违例。
    #[test]
    fn no_file_from_the_version_without_a_file_on_is_not_a_violation_and_a_file_read_there_is() {
        let with_a_file = the_multi_record_publish_stream_versions();
        let without_a_file = versions_without_a_file_from_txg_six();
        let table = CrashPointReplayVersionTable::new(&with_a_file, &without_a_file);
        for checkpoint_txg in 6..=9 {
            let root = (InstanceGeneration(1), CheckpointTxg(checkpoint_txg));
            assert_eq!(
                kind_judged_against_the_table(&RecoveryOutcome::NoFile { root }, root, table),
                None,
                "第 {checkpoint_txg} 代（意图那次或之后的一批）报没有文件不是违例"
            );
            assert_eq!(
                kind_judged_against_the_table(
                    &RecoveryOutcome::FileRead {
                        root,
                        content: b"C".to_vec(),
                    },
                    root,
                    table
                ),
                Some(CrashPointReplayOracleViolationKind::FileReadWhereThisRootHasNone),
                "第 {checkpoint_txg} 代起没有文件，读出 C 的内容是违例"
            );
        }
        let root_of_c = (InstanceGeneration(1), CheckpointTxg(5));
        assert_eq!(
            kind_judged_against_the_table(
                &RecoveryOutcome::NoFile { root: root_of_c },
                root_of_c,
                table
            ),
            Some(CrashPointReplayOracleViolationKind::NoFileWhereThisRootHasOne),
            "C 那一版有文件，报没有是违例"
        );
    }

    /// 根不在表里只看比它旧的最新那一项：文件没了之后又有一版带文件的（第 8 代），落在它后面的第 9 代报没有文件是违例、点名第 8 代；
    /// 落在没有文件那一版与第 8 代之间的第 7 代报没有文件不是违例。
    #[test]
    fn a_root_not_in_the_table_is_judged_by_the_newest_older_entry_only() {
        let with_a_file = vec![
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(3),
                content: b"A".to_vec(),
            },
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(8),
                content: b"written after the file vanished".to_vec(),
            },
        ];
        let without_a_file = versions_without_a_file_from_txg_six();
        let table = CrashPointReplayVersionTable::new(&with_a_file, &without_a_file);
        let root_after_the_newer_file = (InstanceGeneration(1), CheckpointTxg(9));
        let violation = classified_oracle_violation_for_the_version_table(
            &RecoveryOutcome::NoFile {
                root: root_after_the_newer_file,
            },
            Some(root_after_the_newer_file),
            None,
            table,
        )
        .expect("比第 9 代旧的最新一项是第 8 代、有文件：报没有文件是违例");
        assert_eq!(
            violation.kind,
            CrashPointReplayOracleViolationKind::NoFileWhereAnOlderRootHasOne
        );
        assert!(
            violation.reason.contains("实例 1 第 8 代下面有文件"),
            "违例点名比它旧的最新那一版：{}",
            violation.reason
        );
        let root_between = (InstanceGeneration(1), CheckpointTxg(7));
        assert_eq!(
            kind_judged_against_the_table(
                &RecoveryOutcome::NoFile { root: root_between },
                root_between,
                table
            ),
            None,
            "比第 7 代旧的最新一项是第 6 代、没有文件"
        );
    }

    /// 同一个根既写成有文件、又写成没有文件：版本表自相矛盾，构造时停下。
    #[test]
    #[should_panic(expected = "既写成有文件、又写成文件从这一版起没有了")]
    fn a_root_both_with_and_without_a_file_is_refused_when_the_table_is_built() {
        let with_a_file = the_multi_record_publish_stream_versions();
        let without_a_file = vec![PublishedVersionWithoutAFile {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(5),
        }];
        let _table = CrashPointReplayVersionTable::new(&with_a_file, &without_a_file);
    }

    /// 同 txg 的两条版本是两条版本：落在实例 2 上读出实例 2 的内容不是违例。
    #[test]
    fn the_same_txg_from_two_instances_are_two_versions() {
        let outcome = RecoveryOutcome::FileRead {
            root: (InstanceGeneration(2), CheckpointTxg(7)),
            content: b"written by instance two".to_vec(),
        };
        assert_eq!(
            oracle_violation_for_versions(
                &outcome,
                Some((InstanceGeneration(2), CheckpointTxg(7))),
                Some((CheckpointTxg(7), InstanceGeneration(2))),
                &two_instances_at_txg_seven(),
            ),
            None
        );
    }
}

#[cfg(test)]
mod wall_clock_progress_line_tests {
    use super::{
        receive_until_the_senders_are_gone_with_wall_clock_lines, ReceivedOrWallClockLineDue,
        WallClockProgressLines,
    };
    use std::cell::{Cell, RefCell};
    use std::sync::mpsc;
    use std::time::Duration;

    const INTERVAL: Duration = Duration::from_secs(300);
    /// 假时钟每读一次走这么久：比间隔长，每一次等都已经到期，不真等。
    const CLOCK_STEP_LONGER_THAN_THE_INTERVAL: Duration = Duration::from_secs(301);
    /// 假时钟读到这么多次就丢掉发送端，接收循环随之收尾（间隔判断坏了也收得了尾，用例红而不是卡住）。
    const CLOCK_READS_BEFORE_THE_SENDER_IS_DROPPED: u32 = 12;

    /// 一条消息都没来、假时钟每读一次走过一个间隔以上：每一次等完都要交一行按挂钟的进度，相邻两行之间假时钟走过的不超过两次读数。
    #[test]
    fn a_wall_clock_line_is_due_whenever_a_whole_interval_passes_without_a_message() {
        let (sender, receiver) = mpsc::channel::<u64>();
        let sender_still_open = RefCell::new(Some(sender));
        let clock_reads = Cell::new(0u32);
        let clock = || {
            let reads = clock_reads.get() + 1;
            clock_reads.set(reads);
            if reads >= CLOCK_READS_BEFORE_THE_SENDER_IS_DROPPED {
                sender_still_open.borrow_mut().take();
            }
            CLOCK_STEP_LONGER_THAN_THE_INTERVAL * reads
        };
        let mut progress = WallClockProgressLines::new(INTERVAL, Duration::ZERO);
        let mut lines_at: Vec<Duration> = Vec::new();
        receive_until_the_senders_are_gone_with_wall_clock_lines(
            &receiver,
            &mut progress,
            &clock,
            &mut |event| match event {
                ReceivedOrWallClockLineDue::Received(message) => {
                    panic!("没有发过消息，却收到了 {message}")
                }
                ReceivedOrWallClockLineDue::WallClockLineDue { elapsed } => lines_at.push(elapsed),
            },
        );
        assert!(
            lines_at.len() >= 5,
            "假时钟走过 {} 次间隔以上，按挂钟的进度只交了 {} 行：{lines_at:?}",
            CLOCK_READS_BEFORE_THE_SENDER_IS_DROPPED,
            lines_at.len()
        );
        let mut previous = Duration::ZERO;
        for line_at in &lines_at {
            assert!(
                line_at.saturating_sub(previous) <= CLOCK_STEP_LONGER_THAN_THE_INTERVAL * 2,
                "两行之间假时钟走了 {:?}，过了间隔却没有一行：{lines_at:?}",
                line_at.saturating_sub(previous)
            );
            previous = *line_at;
        }
    }

    /// 消息接连到、时钟不走：每条消息交给调用方（它按片、按块打行），不多交一行按挂钟的进度。
    #[test]
    fn no_wall_clock_line_is_due_while_messages_arrive_within_the_interval() {
        let (sender, receiver) = mpsc::channel::<u64>();
        for message in 0..3 {
            sender.send(message).expect("接收端还在");
        }
        drop(sender);
        let clock = || Duration::from_secs(5);
        let mut progress = WallClockProgressLines::new(INTERVAL, Duration::ZERO);
        let mut received = Vec::new();
        let mut wall_clock_lines = 0usize;
        receive_until_the_senders_are_gone_with_wall_clock_lines(
            &receiver,
            &mut progress,
            &clock,
            &mut |event| match event {
                ReceivedOrWallClockLineDue::Received(message) => received.push(message),
                ReceivedOrWallClockLineDue::WallClockLineDue { .. } => wall_clock_lines += 1,
            },
        );
        assert_eq!(received, vec![0, 1, 2], "每条消息按次序交给调用方");
        assert_eq!(wall_clock_lines, 0, "间隔没过，不该有按挂钟的行");
    }
}
