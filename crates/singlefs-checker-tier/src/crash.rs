//! 层 0 崩溃点重放（checker 档，D13（验证路线） 已定项 4、15）：拿录制流在内存里重建镜像，屏障与 FUA 按设备切段，
//! 段内每次写各取它的几态、任意组合：原地覆写的写多一态「新旧都读不出」（[`TearableInPlaceOverwrites`]），
//! 其余写的撕裂态并进「没持久」；每个状态跑恢复、oracle、池级 checker 与记录核对器（D13（验证路线） 已定项 7，故意不给编号）。
//! 内存池、保留写、崩溃镜像与两个结果类型在 `singlefs_harness::memory_pool`。

use std::collections::BTreeMap;

use std::num::{NonZeroU64, NonZeroUsize};
use std::ops::Range;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::time::Instant;

use singlefs_checker::image::{ImageReader, InvariantVerdict};
use singlefs_checker::walk::{check_pool_image, InstanceTableOfRootRecord};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::recovery::{
    recover, JournalPolicy, PoolReader, RecoveryOutcome, RecoveryReport,
};
use singlefs_format::NODE_POINTER_BYTES;

use crate::layer0_progress::{
    findings_log_final_lines, findings_log_new_signature_line, findings_log_threshold_line,
    findings_standard_output_new_signature_line, findings_standard_output_summary_line,
    findings_standard_output_threshold_line, read_shard_ledgers_for_merge, shard_ledger_path,
    write_shard_ledger, DeleteTheProgressFileWhenPanicking, Layer0FindingsLog,
    Layer0FindingsLogBegin, Layer0FindingsLogSection, Layer0ProgressFile,
    Layer0ProgressFileAfterCompletion, Layer0ProgressPlan, Layer0Resume, Layer0ShardMerge,
    Layer0ShardOfShards, Layer0ShardWorkerThreads,
};
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_harness::memory_pool::{
    newest_persisted_root, publishes_in, root_identity_of_write, root_identity_written_by,
    CrashImage, MemoryPool, PublishWrites, PublishedVersion, RecordCheck, RecordStreamContinuity,
    RetainedWrite, WrittenContents, SECTOR_BYTES,
};
use singlefs_harness::segments::StepKind;

/// 记录核对器（D13（验证路线） 已定项 7，故意不给它编号；入参 (崩溃前镜像, 记录流, 崩溃后镜像, 持久集合)）：崩溃前镜像是 `image.base`、
/// 记录流是 `image.writes`、崩溃后镜像是 `image` 本身、持久集合是 `image.persisted`（枚举器给的这个崩溃状态里哪些写落了盘，
/// 与 `image.writes` 逐条对应——四样都从同一个崩溃状态里取，对不上的组合写不出来）。层 0 只跑只读恢复、不改盘，
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
/// 一条录制流，还是崩溃截断的流接上崩溃之后写出的流，[`publishes_in`] 按它分发布）。
///
/// 崩溃后镜像是两份（D13（验证路线） 已定项 7）：`crash_state_image` 是崩溃态镜像，判择根与前缀——第一条判据里根槽写与记录写
/// 「在盘上」看它；`recovered_image` 是实现恢复后的镜像，判在不在——第二条判据里单元副本缺不缺席、恢复落到的那一版的实例表
/// 都从它读（实现的恢复会改盘：可写挂载写行、暖机，恢复落到的那一版可以是恢复自己写出的，只在恢复后的镜像上）。
/// 只读恢复不改盘，两份是同一份。
///
/// 装置里各处交什么（用例另有直接调它的）：
/// - 层 0（[`check_records`]）：两份镜像同一份；写表是枚举用的那一张（录制流里的写在前，原地覆写的撕裂镜像与重放接在后面，
///   见 [`CrashImage`]），一条录制流。
/// - 崩溃注入第一截（增补 3 第 3 件，`crash_injection.rs`）：两份镜像都是「更早的段整段持久 + 当前段一个子集」那份崩溃镜像；
///   写表是这段历史的整条录制流，一条录制流——`persisted` 里更早的段整段持久、当前段按这个崩溃点的子集、更晚的段一个都没持久；
///   恢复落到由记录重建的一版时，那一版的根槽写在更晚的段里，也在这张表里。
/// - 崩溃注入第二截（崩溃后镜像上可写挂载、再发一次布之后的池）：崩溃态镜像是第一截那份崩溃镜像，恢复后的镜像是挂载与那次发布
///   之后的池；写表是整条历史录制流接上挂载那一段录制流、再接上那次发布的录制流，在历史与挂载的接缝处断开——历史那一段的持久集合
///   同第一截，挂载与那次发布全落了；`effective_root` 取挂载与那次发布写出的最新那条根（一条根都没写出时取第一次崩溃之后
///   恢复落到的那一版）。
/// - 崩溃注入第三截（挂载途中的二次崩溃）：两份镜像都是二次崩溃镜像；写表同第二截，挂载那一段的持久集合按二次崩溃——
///   更早的段整段持久、当前段按子集、更晚的段一个都没持久，之后那次发布一个都没写到。历史那几次发布在二次崩溃镜像上还在不在也核得到。
///
/// 第一条判据只在随机历史上才遇得到的限定：一次发布一条 journal 记录都没写过时不判「根在而记录一条都不在」（记录流本来就是空的，不是有洞）。
///
/// 第二条判据（恢复自称的 txg ≥ 某次发布、那次发布的某个单元全部副本都缺席）按 C561（记录核对器的复用豁免在复用只落一半时假红）
/// 的定案判一份副本缺席：**它区间里有一个扇区在持久集合下不是它的字节，且这个扇区上没有一次更晚、在持久集合里、过了回收谓词的写**
/// （层 0 规模第三轮攻方的 `SecPers513`，`research/prompts/m2-layer0-scale-r3-main-verification.md` 第四节用户定的乙）。
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
    continuity: RecordStreamContinuity,
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
/// 与这次根槽写带的是同一个）。根槽没落盘、由记录重建的那一版，它的根槽写在截断处或二次崩溃的当前段之后：层 0 与崩溃注入三截
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
fn checkpoint_txg_of_the_publish_that_made_the_write(
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
///   D16（发布语义） 已定项 1「生效」，大不过这些写带的全部 F 的最大值；C556（checker 与层 0 不读系统配置里的 F） 的层 0 那一半：
///   抬 F 先写系统配置那一步落了、带新 F 的根一条都没落时，下一次挂载按系统配置里的新 F 回收，写行那次发布就可能复用）。
/// - 环里最旧有效根 ≤ 这次写之前写到根环、还没被更晚的根槽写盖掉的那些根里最小的 txg。这里不按实例表剔被抛弃的根：
///   被抛弃的根的 txg 夹在恢复的所选根与恢复之后的第一条根之间，它比全部有效根都旧时，有效时间线上被复用的单元要么比它还旧
///   （下界过得了它）、要么比它新而它自己的根还在环里（本来就不许复用）。
///
/// 按这三个界判，合法的复用不会被判成过不了；漏的是释放代比 `earlier_publish_txg + 1` 晚得多、又晚过门槛的那一类。
/// 崩溃注入第二、三截的写表（崩溃截断的历史流接上崩溃之后写出的流，[`RecordStreamContinuity::ResumedAfterACrash`]）里，
/// 截断处之后那一截历史写没发生过，接缝之后的写取「这次写之前」时照样把它们算进去：那一截的根比截断处之前的 txg 都大、
/// 带的 F 不比之前的小，算进去只会把两个界往「过得了」那边推，不会把合法的复用判成过不了（推的，没造状态量过松了多少）。
/// 前提有两条，写表的来路都满足：基镜像里的根带的 F 为 0（层 0 的基是 mkfs 之后，崩溃注入的基是空池），
/// 写表里没有写失败的根槽写（录制器只录落下了的写；写失败时旧根留在槽里、环不再按 txg 连续，上面那条关于被抛弃根的论证就不成立）。
/// 写表里这次写之前一条根槽写都没有时，环里是什么只有基镜像知道，一律按过得了。
fn reuse_is_not_proven_illegal_by_the_reclaim_predicate(
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

/// 层 0 的一个崩溃状态按发布归到哪一格（里程碑「覆盖写、释放、回退与复用」步 6 验收第 1 条「每次发布各多少」）。
/// 归法按段：状态所在的那一段往后数，第一次根槽 FUA 写所在的那一段写出的根，就是这个状态归的那次发布——
/// 上一次发布的根槽写之后、这一次发布的根槽写为止，崩在中间的状态都归这一次。上一次发布的系统配置槽轮换与这一次的单元写
/// 之间没有屏障、并在同一段时，那一段整段归这一次：段是枚举的最小单位，一个状态落在哪一段是确定的，落在哪一次写上不是。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Layer0PublishOfState {
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

impl Layer0PublishOfState {
    /// 计数行里的名字：`instance2_txg5`、`after_the_last_root`、`every_write_persisted`。
    #[must_use]
    pub fn name(self) -> String {
        match self {
            Layer0PublishOfState::UpToTheRootOf {
                instance,
                checkpoint_txg,
                ..
            } => format!("instance{}_txg{}", instance.0, checkpoint_txg.0),
            Layer0PublishOfState::AfterTheLastRoot => "after_the_last_root".to_string(),
            Layer0PublishOfState::EveryWritePersisted => "every_write_persisted".to_string(),
        }
    }

    /// 发现日志里的名字：`instance2_txg5_root_write14`、`after_the_last_root`、`every_write_persisted`。
    /// 比 [`Self::name`] 多带根槽写的下标：两块盘各写一条同身份的根时，两格只差这一项，发现表的签名不许把它们并成一格。
    #[must_use]
    pub fn name_with_the_root_write_index(self) -> String {
        match self {
            Layer0PublishOfState::UpToTheRootOf {
                root_write_index, ..
            } => format!("{}_root_write{root_write_index}", self.name()),
            Layer0PublishOfState::AfterTheLastRoot | Layer0PublishOfState::EveryWritePersisted => {
                self.name()
            }
        }
    }
}

/// 每一段归哪次发布（[`Layer0PublishOfState`] 的归法）：从最后一段往前走，记着「后面最近的那次根槽写」。
#[must_use]
pub fn publish_of_each_segment(
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
) -> Vec<Layer0PublishOfState> {
    let mut next_root: Option<Layer0PublishOfState> = None;
    let mut publish_of_segment = vec![Layer0PublishOfState::AfterTheLastRoot; segments.len()];
    for (segment_index, segment) in segments.iter().enumerate().rev() {
        if let Some(root_write_index) = segment.iter().rev().copied().find(|write_index| {
            writes
                .get(*write_index)
                .is_some_and(|write| write.kind == StepKind::RootRecordFua)
        }) {
            let (instance, checkpoint_txg) = root_identity_of_write(&writes[root_write_index]);
            next_root = Some(Layer0PublishOfState::UpToTheRootOf {
                root_write_index,
                instance,
                checkpoint_txg,
            });
        }
        publish_of_segment[segment_index] =
            next_root.unwrap_or(Layer0PublishOfState::AfterTheLastRoot);
    }
    publish_of_segment
}

/// 层 0 的 oracle（[`oracle_violation_for_versions`]）判的违例是哪一类：层 0 发现表按它分签名（两遍恢复各判一次）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Layer0OracleViolationKind {
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

impl Layer0OracleViolationKind {
    /// 每一类各一个：[`Self::from_name`] 在里面找（新加一类要加进来，不然进度文件里这一类读不回、整份作废、从头跑）。
    pub const EVERY_KIND: [Self; 7] = [
        Self::NoRootChosen,
        Self::RecoveredToAnOlderRoot,
        Self::ReadFailed,
        Self::NoFileWhereAnOlderRootHasOne,
        Self::NoFileWhereThisRootHasOne,
        Self::FileReadWhereThisRootHasNone,
        Self::WrongContent,
    ];

    /// 发现日志与 `LAYER0_FINDING` 行里的名字。
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
pub struct Layer0OracleViolation {
    pub kind: Layer0OracleViolationKind,
    pub reason: String,
}

/// 层 0 发现表里一个签名的一半：判红的是哪一遍、违了哪几条。一个状态几遍都红，就各进各的签名。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Layer0RedPass {
    /// 看 journal 那一遍恢复过 oracle 判违例（计进 [`Layer0Tally::violations`]）。
    JournalConsultedOracle(Layer0OracleViolationKind),
    /// 不看 journal 那一遍恢复过 oracle 判违例（计进 [`Layer0Tally::ignored_violations`]）。
    JournalIgnoredOracle(Layer0OracleViolationKind),
    /// 池级 checker 判违例的不变量，至少一条；次序照 `check_pool_image` 报的次序（它按 checker 的清单 `IMPLEMENTED_INVARIANTS` 报）。
    PoolChecker(Vec<&'static str>),
    /// 记录核对器：两条判据里至少一条成立。
    RecordChecker(RecordCheck),
}

/// 记录核对器第一条判据在发现日志里的名字。
pub const RECORD_CHECKER_ROOT_WITHOUT_RECORD: &str = "root_without_record";
/// 记录核对器第二条判据在发现日志里的名字。
pub const RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT: &str = "claimed_state_missing_unit";
/// 发现日志里 `pass=` 的四个值，依次是 [`Layer0RedPass`] 的四个成员。
pub const LAYER0_RED_PASS_JOURNAL_CONSULTED_ORACLE: &str = "journal_consulted_oracle";
pub const LAYER0_RED_PASS_JOURNAL_IGNORED_ORACLE: &str = "journal_ignored_oracle";
pub const LAYER0_RED_PASS_POOL_CHECKER: &str = "pool_checker";
pub const LAYER0_RED_PASS_RECORD_CHECKER: &str = "record_checker";

impl Layer0RedPass {
    /// 发现日志里 `pass=` 的值。
    #[must_use]
    pub fn pass_name(&self) -> &'static str {
        match self {
            Self::JournalConsultedOracle(_) => LAYER0_RED_PASS_JOURNAL_CONSULTED_ORACLE,
            Self::JournalIgnoredOracle(_) => LAYER0_RED_PASS_JOURNAL_IGNORED_ORACLE,
            Self::PoolChecker(_) => LAYER0_RED_PASS_POOL_CHECKER,
            Self::RecordChecker(_) => LAYER0_RED_PASS_RECORD_CHECKER,
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
pub enum Layer0SegmentOfState {
    Segment(usize),
    AllPersisted,
}

impl Layer0SegmentOfState {
    /// 发现日志与 `LAYER0_PROGRESS` 行里的写法：段号，或 `all_persisted`。
    #[must_use]
    pub fn name(self) -> String {
        match self {
            Self::Segment(segment_index) => segment_index.to_string(),
            Self::AllPersisted => "all_persisted".to_string(),
        }
    }
}

/// 层 0 发现表的签名：判红的是哪一遍、违了哪几条、状态所在的段、状态归哪次发布。大小次序（先 pass 与违了哪几条，再段、再发布）
/// 只在同一个状态上出了几个签名时用来排号。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Layer0FindingSignature {
    pub red_pass: Layer0RedPass,
    pub segment: Layer0SegmentOfState,
    pub publish: Layer0PublishOfState,
}

/// 发现表里每个签名最多留几个样本（最先的几个状态）。
pub const LAYER0_FINDING_SAMPLES_KEPT: usize = 3;

/// 签名下的一个样本：状态序号与那一条违例原文。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layer0FindingSample {
    pub state_ordinal: u64,
    pub violation: String,
}

/// 一个签名下判红的状态：几个，与最先的至多 [`LAYER0_FINDING_SAMPLES_KEPT`] 个（序号从小到大，个数 = min(状态数, 3)）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layer0Finding {
    pub states: u64,
    pub earliest_samples: Vec<Layer0FindingSample>,
}

impl Layer0Finding {
    /// 最先那个状态：签名进表时就记了第一个样本。
    ///
    /// # Panics
    /// 一个样本都没有（不变量被破坏：进表的签名至少一个状态、至少一个样本）。
    #[must_use]
    pub fn first_sample(&self) -> &Layer0FindingSample {
        self.earliest_samples
            .first()
            .expect("进了发现表的签名至少一个状态，第一个状态进表时就记成了样本")
    }
}

/// 层 0 的发现表：判红的状态按签名去重（[`Layer0FindingSignature`]），每个签名记状态数与最先几个样本；另记至少一遍判红的状态数
/// （一个状态几遍都红只算一次）。只有按段枚举的那几个入口记这一项（签名要状态所在的段与发布）；直接调
/// [`evaluate_state_for_versions`] 评手摆的状态时它是空的。并片按状态序号从小到大（[`Layer0Findings::absorb_following_slice`]），
/// 与线程数、切法无关。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Layer0Findings {
    pub red_states: u64,
    pub by_signature: BTreeMap<Layer0FindingSignature, Layer0Finding>,
}

/// 发现日志的台阶：签名的状态数跨过 10、100、1000……各报一次（1 由「新签名」那一次报过）。
const LAYER0_FINDING_THRESHOLD_BASE: u64 = 10;

/// 并进一片时报的一件事。
enum Layer0FindingEvent<'findings> {
    /// 这个签名第一次出现：它最先那个状态（全流最先，因为片按序号从小到大并）。
    NewSignature {
        signature: &'findings Layer0FindingSignature,
        first_sample: &'findings Layer0FindingSample,
    },
    /// 这个签名的状态数跨过了一级台阶。
    ThresholdCrossed {
        signature: &'findings Layer0FindingSignature,
        states_at_least: u64,
    },
}

impl Layer0Findings {
    /// 记一个判红的状态在一个签名下；这个签名的样本还不满 [`LAYER0_FINDING_SAMPLES_KEPT`] 个时才调 `violation` 取原文。
    /// 同一片里状态按序号从小到大评，样本自然从小到大。
    fn record_red_state(
        &mut self,
        signature: Layer0FindingSignature,
        state_ordinal: u64,
        violation: impl FnOnce() -> String,
    ) {
        let finding = self
            .by_signature
            .entry(signature)
            .or_insert_with(|| Layer0Finding {
                states: 0,
                earliest_samples: Vec::new(),
            });
        finding.states += 1;
        if finding.earliest_samples.len() < LAYER0_FINDING_SAMPLES_KEPT {
            finding.earliest_samples.push(Layer0FindingSample {
                state_ordinal,
                violation: violation(),
            });
        }
    }

    /// 把紧跟在后面的那一片的发现表并进来：状态数相加；样本接在后面、只留最先的 [`LAYER0_FINDING_SAMPLES_KEPT`] 个。
    /// 按字段拆开写全：新加一个字段而这里没并，编译不过。
    ///
    /// # Panics
    /// 后面那一片的样本序号不大于前面已留的（不变量被破坏：片要按状态序号从小到大并）。
    pub fn absorb_following_slice(&mut self, following_slice: Layer0Findings) {
        let Layer0Findings {
            red_states,
            by_signature,
        } = following_slice;
        self.red_states += red_states;
        for (signature, following_finding) in by_signature {
            let Some(earlier_finding) = self.by_signature.get_mut(&signature) else {
                self.by_signature.insert(signature, following_finding);
                continue;
            };
            let Layer0Finding {
                states,
                earliest_samples,
            } = following_finding;
            earlier_finding.states += states;
            for sample in earliest_samples {
                if earlier_finding.earliest_samples.len() == LAYER0_FINDING_SAMPLES_KEPT {
                    break;
                }
                let last_kept_ordinal = earlier_finding
                    .earliest_samples
                    .last()
                    .expect("进了发现表的签名至少一个样本")
                    .state_ordinal;
                assert!(
                    sample.state_ordinal > last_kept_ordinal,
                    "并片要按状态序号从小到大：后面那一片的样本 {} 不大于前面已留的 {last_kept_ordinal}",
                    sample.state_ordinal
                );
                earlier_finding.earliest_samples.push(sample);
            }
        }
    }

    /// 签名按最先那个状态的序号排（同一个状态上的几个签名按签名的大小次序）：发现日志里的号就是这个次序里的第几个（从 1 起）。
    #[must_use]
    pub fn signatures_in_first_state_order(
        &self,
    ) -> Vec<(&Layer0FindingSignature, &Layer0Finding)> {
        let mut ordered: Vec<(&Layer0FindingSignature, &Layer0Finding)> =
            self.by_signature.iter().collect();
        ordered.sort_by(
            |(left_signature, left_finding), (right_signature, right_finding)| {
                (left_finding.first_sample().state_ordinal, *left_signature)
                    .cmp(&(right_finding.first_sample().state_ordinal, *right_signature))
            },
        );
        ordered
    }

    /// 紧跟在后面的那一片并进来会报的事：先是这一片里第一次出现的签名（按最先那个状态的序号、再按签名排），
    /// 再是跨过台阶的（签名之间按签名排，同一签名按台阶从小到大）。只看状态数与样本，不改表。
    fn events_of_absorbing<'following>(
        &self,
        following_slice: &'following Layer0Findings,
    ) -> Vec<Layer0FindingEvent<'following>> {
        let mut new_signatures: Vec<(&Layer0FindingSignature, &Layer0FindingSample)> = Vec::new();
        let mut thresholds_crossed: Vec<Layer0FindingEvent<'following>> = Vec::new();
        for (signature, following_finding) in &following_slice.by_signature {
            let states_before = self
                .by_signature
                .get(signature)
                .map_or(0, |earlier_finding| earlier_finding.states);
            if states_before == 0 {
                new_signatures.push((signature, following_finding.first_sample()));
            }
            let states_after = states_before + following_finding.states;
            thresholds_crossed.extend(
                std::iter::successors(Some(LAYER0_FINDING_THRESHOLD_BASE), |threshold| {
                    threshold.checked_mul(LAYER0_FINDING_THRESHOLD_BASE)
                })
                .take_while(|threshold| *threshold <= states_after)
                .filter(|threshold| *threshold > states_before)
                .map(|threshold| Layer0FindingEvent::ThresholdCrossed {
                    signature,
                    states_at_least: threshold,
                }),
            );
        }
        new_signatures.sort_by(
            |(left_signature, left_sample), (right_signature, right_sample)| {
                (left_sample.state_ordinal, *left_signature)
                    .cmp(&(right_sample.state_ordinal, *right_signature))
            },
        );
        new_signatures
            .into_iter()
            .map(
                |(signature, first_sample)| Layer0FindingEvent::NewSignature {
                    signature,
                    first_sample,
                },
            )
            .chain(thresholds_crossed)
            .collect()
    }
}

/// 层 0 的计数：每个状态跑一遍看 journal 的恢复与一遍不看的，oracle 只判前者。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Layer0Tally {
    pub states: u64,
    /// 枚举出来的状态按发布分（[`Layer0PublishOfState`] 的归法）：各格之和 = `states`。只有按段枚举的那几个入口
    /// （`enumerate_layer0*`）记这一项；直接调 [`evaluate_state_for_versions`] 评手摆的状态时它是空的。
    pub states_by_publish: BTreeMap<Layer0PublishOfState, u64>,
    pub violations: u64,
    pub root_persisted_states: u64,
    pub no_file_states: u64,
    pub file_read_states: u64,
    pub failed_states: u64,
    pub journal_differing_states: u64,
    pub verification_ran_states: u64,
    pub verification_failed_states: u64,
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
    /// 观察者看过的状态数（没有观察者时是 0）：续跑接上的片不再给观察者看，靠进度文件里记的这个数与 `observer_counts` 接上累计；
    /// 有观察者时整条流跑完它必须等于 `states`（层 0 规模第三轮判决 U3）。
    pub observed_states: u64,
    /// 观察者按名字累计的数（[`Layer0ObserverCounts`]），随片进进度文件。
    pub observer_counts: Layer0ObserverCounts,
    /// 判红的状态按签名去重的发现表（[`Layer0Findings`]），随片进进度文件与分片账本。
    pub findings: Layer0Findings,
}

/// 观察者在状态上记的数，按名字累计（名字只许小写字母、数字、`_`：它原样进进度文件）。续跑时已跑完的片不再给观察者看，
/// 观察者要接着用的累计都得记在这里，记在观察者自己的变量里的续跑之后就少了那几片。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Layer0ObserverCounts(BTreeMap<String, u64>);

impl Layer0ObserverCounts {
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

    fn absorb(&mut self, following: Layer0ObserverCounts) {
        for (name, count) in following.0 {
            *self.0.entry(name).or_insert(0) += count;
        }
    }
}

impl Layer0Tally {
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

    /// 把紧跟在后面的那一片的计数并进来：计数逐项相加；「第一处」只在前面各片都没有时取这一片的。各片按状态序号从小到大并，
    /// 「第一处」就是序号最小的那一处，与单线程逐个跑逐项相同。按字段拆开写全：新加一个字段而这里没并，编译不过。
    fn absorb_following_slice(&mut self, following_slice: Layer0Tally) {
        let Layer0Tally {
            states,
            states_by_publish,
            violations,
            root_persisted_states,
            no_file_states,
            file_read_states,
            failed_states,
            journal_differing_states,
            verification_ran_states,
            verification_failed_states,
            first_violation,
            ignored_violations,
            first_ignored_violation,
            record_root_without_record,
            record_claimed_state_missing_unit,
            checker_evaluated_states,
            checker_violated_states,
            checker_first_violation,
            checker_not_applicable_states,
            observed_states,
            observer_counts,
            findings,
        } = following_slice;
        self.states += states;
        self.observed_states += observed_states;
        self.observer_counts.absorb(observer_counts);
        self.findings.absorb_following_slice(findings);
        for (publish, publish_states) in states_by_publish {
            *self.states_by_publish.entry(publish).or_insert(0) += publish_states;
        }
        self.violations += violations;
        self.root_persisted_states += root_persisted_states;
        self.no_file_states += no_file_states;
        self.file_read_states += file_read_states;
        self.failed_states += failed_states;
        self.journal_differing_states += journal_differing_states;
        self.verification_ran_states += verification_ran_states;
        self.verification_failed_states += verification_failed_states;
        if self.first_violation.is_none() {
            self.first_violation = first_violation;
        }
        self.ignored_violations += ignored_violations;
        if self.first_ignored_violation.is_none() {
            self.first_ignored_violation = first_ignored_violation;
        }
        self.record_root_without_record += record_root_without_record;
        self.record_claimed_state_missing_unit += record_claimed_state_missing_unit;
        for (invariant, evaluated_states) in checker_evaluated_states {
            *self.checker_evaluated_states.entry(invariant).or_insert(0) += evaluated_states;
        }
        for (invariant, violated_states) in checker_violated_states {
            *self.checker_violated_states.entry(invariant).or_insert(0) += violated_states;
        }
        for (invariant, detail) in checker_first_violation {
            self.checker_first_violation
                .entry(invariant)
                .or_insert(detail);
        }
        for (invariant, not_applicable_states) in checker_not_applicable_states {
            *self
                .checker_not_applicable_states
                .entry(invariant)
                .or_insert(0) += not_applicable_states;
        }
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

/// 同 [`oracle_violation_for_versions`]，另交回违例是哪一类（[`Layer0OracleViolationKind`]，层 0 发现表按它分签名）。
#[must_use]
pub fn classified_oracle_violation_for_versions(
    outcome: &RecoveryOutcome,
    effective_root: Option<(InstanceGeneration, CheckpointTxg)>,
    newest_persisted_root: Option<(CheckpointTxg, InstanceGeneration)>,
    versions: &[PublishedVersion],
) -> Option<Layer0OracleViolation> {
    let violation =
        |kind: Layer0OracleViolationKind, reason: String| Layer0OracleViolation { kind, reason };
    let Some((effective_instance, effective_txg)) = effective_root else {
        return Some(violation(
            Layer0OracleViolationKind::NoRootChosen,
            "没择到根".to_string(),
        ));
    };
    if let Some((newest_txg, newest_instance)) = newest_persisted_root {
        if (effective_txg, effective_instance) < (newest_txg, newest_instance) {
            return Some(violation(
                Layer0OracleViolationKind::RecoveredToAnOlderRoot,
                format!(
                "根槽已持久而恢复到旧态（盘上最新的根槽是实例 {} 第 {} 代，走的是实例 {} 第 {} 代）",
                newest_instance.0, newest_txg.0, effective_instance.0, effective_txg.0
            ),
            ));
        }
    }
    let version = versions.iter().find(|version| {
        version.checkpoint_txg == effective_txg && version.instance == effective_instance
    });
    match (outcome, version) {
        (RecoveryOutcome::Failed { failure, .. }, _) => Some(violation(
            Layer0OracleViolationKind::ReadFailed,
            format!("走读失败：{failure:?}"),
        )),
        // 落在一个没发布过的更新的根上报「没有文件」：更旧的根下面有文件时是违例（第二轮攻方腿：此前一律放过）。
        (RecoveryOutcome::NoFile { .. }, None) => versions
            .iter()
            .find(|candidate_version| {
                (candidate_version.checkpoint_txg, candidate_version.instance)
                    < (effective_txg, effective_instance)
            })
            .map(|older_version| {
                violation(
                    Layer0OracleViolationKind::NoFileWhereAnOlderRootHasOne,
                    format!(
                    "走到实例 {} 第 {} 代根报没有文件，而没有这一代的版本、实例 {} 第 {} 代下面有文件",
                    effective_instance.0,
                    effective_txg.0,
                    older_version.instance.0,
                    older_version.checkpoint_txg.0
                ),
                )
            }),
        (RecoveryOutcome::NoFile { .. }, Some(_)) => Some(violation(
            Layer0OracleViolationKind::NoFileWhereThisRootHasOne,
            format!("第 {} 代根下面有文件却报没有", effective_txg.0),
        )),
        (RecoveryOutcome::FileRead { .. }, None) => Some(violation(
            Layer0OracleViolationKind::FileReadWhereThisRootHasNone,
            format!("第 {} 代根下面没有文件却读出了内容", effective_txg.0),
        )),
        (RecoveryOutcome::FileRead { content, .. }, Some(version)) => {
            (*content != version.content).then(|| {
                violation(
                    Layer0OracleViolationKind::WrongContent,
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
    tally: &mut Layer0Tally,
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
/// oracle 按 `versions` 判实际走的根该读出哪一版。手摆的状态不在任何一段里，发现表（[`Layer0Tally::findings`]）不记。
pub fn evaluate_state_for_versions(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    persisted: Vec<bool>,
    judged_root_index: usize,
    versions: &[PublishedVersion],
    tally: &mut Layer0Tally,
) -> RecoveryReport {
    evaluate_state_recording_findings(
        base,
        writes,
        persisted,
        judged_root_index,
        versions,
        None,
        tally,
    )
}

/// 层 0 按段枚举时一个状态在哪：序号、所在的段、归哪次发布。发现表的签名要后两样、样本要序号。
#[derive(Clone, Copy, Debug)]
struct Layer0StatePosition {
    ordinal: u64,
    segment: Layer0SegmentOfState,
    publish: Layer0PublishOfState,
}

impl Layer0StatePosition {
    fn signature_of(self, red_pass: Layer0RedPass) -> Layer0FindingSignature {
        Layer0FindingSignature {
            red_pass,
            segment: self.segment,
            publish: self.publish,
        }
    }
}

/// 看 journal 那一遍 oracle 的违例原文：原因后面带这一状态里持久了的写的种类（[`Layer0Tally::first_violation`] 与发现表的样本同一个写法）。
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
/// 层 0 的计数（[`evaluate_state_for_versions`] 一族）与崩溃放量流水线（`crash_amplification`）都从这一份取，不各判一遍。
#[derive(Clone, Debug)]
pub struct Layer0StateJudgement {
    pub consulted: RecoveryReport,
    pub ignored: RecoveryReport,
    /// 被判的那次根槽写落没落（计「根槽已持久」的状态数用）。
    pub root_persisted: bool,
    pub consulted_oracle: Option<Layer0OracleViolation>,
    pub ignored_oracle: Option<Layer0OracleViolation>,
    pub pool_checker: Vec<(&'static str, InvariantVerdict)>,
    pub records: RecordCheck,
}

impl Layer0StateJudgement {
    /// 判红的各项，按固定次序：两份 oracle 各记它的违例种类，池级 checker 记违反的不变量名，记录核对器记它的两种结论；
    /// 一项都没有就是绿。崩溃放量流水线按这串存「逐项原始判定」（读方现算红绿与计数）。
    #[must_use]
    pub fn red_items(&self) -> Vec<String> {
        let mut items = Vec::new();
        if let Some(violation) = &self.consulted_oracle {
            items.push(format!(
                "{LAYER0_RED_PASS_JOURNAL_CONSULTED_ORACLE}:{}",
                violation.kind.name()
            ));
        }
        if let Some(violation) = &self.ignored_oracle {
            items.push(format!(
                "{LAYER0_RED_PASS_JOURNAL_IGNORED_ORACLE}:{}",
                violation.kind.name()
            ));
        }
        for (invariant, verdict) in &self.pool_checker {
            match verdict {
                InvariantVerdict::Violated(_) => {
                    items.push(format!("{LAYER0_RED_PASS_POOL_CHECKER}:{invariant}"))
                }
                InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => {}
            }
        }
        if self.records.root_without_record {
            items.push(format!(
                "{LAYER0_RED_PASS_RECORD_CHECKER}:{RECORD_CHECKER_ROOT_WITHOUT_RECORD}"
            ));
        }
        if self.records.claimed_state_missing_unit {
            items.push(format!(
                "{LAYER0_RED_PASS_RECORD_CHECKER}:{RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT}"
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
                InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
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
#[must_use]
pub fn judge_crash_image(
    image: &CrashImage<'_>,
    judged_root_index: usize,
    versions: &[PublishedVersion],
) -> Layer0StateJudgement {
    let root_persisted = image.persisted[judged_root_index];
    let newest_persisted = newest_persisted_root(image.writes, &image.persisted);
    let consulted = recover(image, JournalPolicy::Consult);
    let ignored = recover(image, JournalPolicy::Ignore);
    let consulted_oracle = classified_oracle_violation_for_versions(
        &consulted.outcome,
        consulted.effective_root,
        newest_persisted,
        versions,
    );
    let ignored_oracle = classified_oracle_violation_for_versions(
        &ignored.outcome,
        ignored.effective_root,
        newest_persisted,
        versions,
    );
    let pool_checker = check_pool_image(image);
    let records = check_records(image, consulted.effective_root);
    Layer0StateJudgement {
        consulted,
        ignored,
        root_persisted,
        consulted_oracle,
        ignored_oracle,
        pool_checker,
        records,
    }
}

/// 同一条流、同一种展开，按层 0 的状态序号判 `ordinals` 这一段：每个状态现算持久集合、造镜像、[`judge_crash_image`]，
/// 交给 `judge_state`（序号、镜像、判定）。交回整条流的状态数（与 [`enumerate_layer0_in_state_slices`] 数的相同）。
/// 崩溃放量流水线按块调它：块就是一段序号，镜像到核对时现算，库里不存持久集合。
///
/// # Panics
/// `ordinals` 越过整条流的状态数。
#[allow(
    clippy::too_many_arguments,
    reason = "前六个与枚举同一条流的参数相同，多出的是要判的序号段与收判定的回调"
)]
#[must_use]
pub fn judge_layer0_state_range(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: &[PublishedVersion],
    expansion: &dyn Fn(usize, &[usize]) -> Layer0SegmentExpansion,
    ordinals: Range<u64>,
    judge_state: &mut dyn FnMut(u64, &CrashImage<'_>, &Layer0StateJudgement),
) -> u64 {
    let tearable = TearableInPlaceOverwrites::of(base, writes);
    let writes_with_torn_images = WritesWithTornImages::of(base, writes, segments, &tearable);
    let plan = Layer0StatePlan::new(
        writes,
        segments,
        expansion,
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
        let judgement = judge_crash_image(&image, judged_root_index, versions);
        judge_state(ordinal, &image, &judgement);
    }
    plan.state_count
}

/// 每一段展开出来的状态序号区间（与 `segments` 同序，不展开的段是空区间），与整条流的状态数（最后那个全持久状态的序号是它减一）。
/// 崩溃放量流水线按它把状态分给崩溃点：一个点管它起头的那几段。
#[must_use]
pub fn layer0_state_ranges_by_segment(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    expansion: &dyn Fn(usize, &[usize]) -> Layer0SegmentExpansion,
) -> (Vec<Range<u64>>, u64) {
    let tearable = TearableInPlaceOverwrites::of(base, writes);
    let writes_with_torn_images = WritesWithTornImages::of(base, writes, segments, &tearable);
    let plan = Layer0StatePlan::new(
        writes,
        segments,
        expansion,
        &tearable,
        &writes_with_torn_images,
    );
    (plan.state_ranges_by_segment.clone(), plan.state_count)
}

/// 层 0 枚举用的几张表，给崩溃放量第 ① 段抽事实用（`crate::crash_facts`）：枚举写表（录制流里的写在前、撕裂镜像与重放接在后面）、
/// 哪几次写取三态、每次原地覆写的撕裂镜像与重放接在枚举写表的哪里、每段展开出来的状态序号区间、状态总数。
/// 与 [`judge_layer0_state_range`] 按同一套 `Layer0StatePlan` 算：事实那边按这几张表现算的掩码与这里的 `persisted_writes_of_state` 逐位相同。
/// 一次原地覆写的撕裂镜像接在枚举写表的哪里：写表下标、撕裂镜像在枚举写表里的下标、（重放在枚举写表里的下标，重放的是写表里哪一次写）。
pub struct TornImageTableRow {
    pub write_index: usize,
    pub torn_image_index: usize,
    pub replays: Vec<(usize, usize)>,
}

pub struct Layer0EnumerationTables {
    pub enumerated_writes: Vec<RetainedWrite>,
    pub is_tearable_by_write: Vec<bool>,
    pub torn_images: Vec<TornImageTableRow>,
    pub state_ranges_by_segment: Vec<Range<u64>>,
    pub state_count: u64,
}

#[must_use]
pub fn layer0_enumeration_tables(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    expansion: &dyn Fn(usize, &[usize]) -> Layer0SegmentExpansion,
) -> Layer0EnumerationTables {
    let tearable = TearableInPlaceOverwrites::of(base, writes);
    let writes_with_torn_images = WritesWithTornImages::of(base, writes, segments, &tearable);
    let plan = Layer0StatePlan::new(
        writes,
        segments,
        expansion,
        &tearable,
        &writes_with_torn_images,
    );
    Layer0EnumerationTables {
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
        state_count: plan.state_count,
    }
}

/// 整条流在这种展开下的状态数，与 [`judge_layer0_state_range`] 交回的相同（不判任何状态）。
#[must_use]
pub fn layer0_plan_state_count(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    expansion: &dyn Fn(usize, &[usize]) -> Layer0SegmentExpansion,
) -> u64 {
    let tearable = TearableInPlaceOverwrites::of(base, writes);
    let writes_with_torn_images = WritesWithTornImages::of(base, writes, segments, &tearable);
    Layer0StatePlan::new(
        writes,
        segments,
        expansion,
        &tearable,
        &writes_with_torn_images,
    )
    .state_count
}

/// [`evaluate_state_for_versions`] 的本体：`position` 是这个状态在按段枚举里的位置（手摆的状态没有），有位置时把判红的每一遍
/// 按签名记进发现表（[`Layer0Findings`]），至少一遍判红的状态另记一个。
fn evaluate_state_recording_findings(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    persisted: Vec<bool>,
    judged_root_index: usize,
    versions: &[PublishedVersion],
    position: Option<Layer0StatePosition>,
    tally: &mut Layer0Tally,
) -> RecoveryReport {
    let image = CrashImage {
        base,
        writes,
        persisted,
    };
    let judgement = judge_crash_image(&image, judged_root_index, versions);
    let Layer0StateJudgement {
        consulted,
        ignored,
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
    if consulted.outcome != ignored.outcome {
        tally.journal_differing_states += 1;
    }
    if consulted.journal.verification_passed + consulted.journal.verification_failed > 0 {
        tally.verification_ran_states += 1;
    }
    if consulted.journal.verification_failed > 0 {
        tally.verification_failed_states += 1;
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
                position.signature_of(Layer0RedPass::JournalConsultedOracle(violation.kind)),
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
                position.signature_of(Layer0RedPass::JournalIgnoredOracle(violation.kind)),
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
            InvariantVerdict::NotApplicable(_) => {
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
                position.signature_of(Layer0RedPass::PoolChecker(
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
                position.signature_of(Layer0RedPass::RecordChecker(records)),
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

/// 单版本形态的枚举。
#[must_use]
pub fn enumerate_layer0_selecting(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    root_index: usize,
    expected_content: &[u8],
    expand: &dyn Fn(usize, &[usize]) -> bool,
) -> Layer0Tally {
    let (instance, checkpoint_txg) = root_identity_of_write(&writes[root_index]);
    let versions = [PublishedVersion {
        instance,
        checkpoint_txg,
        content: expected_content.to_vec(),
    }];
    enumerate_layer0_selecting_versions(base, writes, segments, root_index, &versions, expand)
}

/// 层 0 枚举的工作线程数从这个环境变量取（十进制正整数）；没设就取 [`std::thread::available_parallelism`]。门禁 54 号显式传进来。
pub const LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_LAYER0_THREADS";

/// 默认切法的片数下限：线程数为 1 时全量用例也按片报进度。
const LAYER0_MINIMUM_SLICE_COUNT: u64 = 64;
/// 默认切法里每个工作线程摊到的片数：越往后的状态历史越长、越贵，片切得比线程多，先跑完的线程接着领下一片。
const LAYER0_SLICES_PER_WORKER_THREAD: u64 = 16;
/// 默认切法里每片的状态数下限：平时 `cargo test` 里几十个状态的枚举只切成几片，进度行不刷屏。
const LAYER0_MINIMUM_STATES_PER_SLICE: u64 = 16;

/// 工作线程数是从哪来的：与实际起的线程数一起打进 `LAYER0_PARALLEL_START` / `LAYER0_PARALLEL_FINISHED` 两行（门禁 54 号拿实际起的线程数判「没显式设成 1、机器多于 1 核却只用了 1 个线程」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer0WorkerThreadsSource {
    /// 环境变量 [`LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE`] 显式给的。
    EnvironmentVariable,
    /// 没设环境变量，取 `available_parallelism`。
    AvailableParallelism,
    /// 没设环境变量，`available_parallelism` 也报不出来：只用 1 个线程，照实报出来。
    AvailableParallelismUnknown,
    /// 调用方在代码里直接给的（用例拿不同切法对拍）。
    GivenByCaller,
}

impl Layer0WorkerThreadsSource {
    fn name(self) -> &'static str {
        match self {
            Self::EnvironmentVariable => "environment_variable",
            Self::AvailableParallelism => "available_parallelism",
            Self::AvailableParallelismUnknown => "available_parallelism_unknown",
            Self::GivenByCaller => "given_by_caller",
        }
    }
}

/// 每片几个状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer0SliceLength {
    /// 片数取 max(64, 16 × 工作线程数)，每片至少 16 个状态。
    ScaledToWorkerThreads,
    /// 每片固定这么多个状态（用例把切法推到两头：每片 1 个，或整条流 1 片）。
    StatesPerSlice(NonZeroU64),
}

/// 层 0 按状态序号区间切片、多线程跑：几个工作线程、每片几个状态。切法与线程数只影响跑得多快，不影响计数与「第一处违例」。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layer0Parallelism {
    pub worker_threads: NonZeroUsize,
    pub worker_threads_source: Layer0WorkerThreadsSource,
    pub slice_length: Layer0SliceLength,
}

impl Layer0Parallelism {
    /// 线程数取环境变量 [`LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE`]，没设就取 `available_parallelism`；片长按线程数定。
    ///
    /// # Panics
    /// 环境变量设了却不是正整数：配错了就停，不悄悄退回单线程。
    #[must_use]
    pub fn from_environment() -> Self {
        Self::from_environment_value(
            std::env::var(LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE),
            std::thread::available_parallelism,
        )
    }

    /// [`Self::from_environment`] 的判定本身：环境变量读到什么、`available_parallelism` 报什么都由调用方给（用例不改进程的环境变量）。
    ///
    /// # Panics
    /// 环境变量设了却不是正整数（含 0、空串、非 UTF-8）。
    #[must_use]
    fn from_environment_value(
        environment_value: Result<String, std::env::VarError>,
        available_parallelism: impl FnOnce() -> std::io::Result<NonZeroUsize>,
    ) -> Self {
        let (worker_threads, worker_threads_source) = match environment_value {
            Ok(text) => (
                text.parse::<NonZeroUsize>().unwrap_or_else(|error| {
                    panic!(
                        "{LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE} 要是正整数，读到 {text:?}：{error}"
                    )
                }),
                Layer0WorkerThreadsSource::EnvironmentVariable,
            ),
            Err(std::env::VarError::NotPresent) => match available_parallelism() {
                Ok(available) => (available, Layer0WorkerThreadsSource::AvailableParallelism),
                Err(_unavailable) => (
                    NonZeroUsize::MIN,
                    Layer0WorkerThreadsSource::AvailableParallelismUnknown,
                ),
            },
            Err(std::env::VarError::NotUnicode(raw)) => panic!(
                "{LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE} 要是正整数，读到的不是 UTF-8：{raw:?}"
            ),
        };
        Self {
            worker_threads,
            worker_threads_source,
            slice_length: Layer0SliceLength::ScaledToWorkerThreads,
        }
    }
}

/// 一段怎么展开成崩溃状态（前面的段全持久、这一段落一部分、后面的段全不落）。
/// 段里每次写各取几态见 [`TearableInPlaceOverwrites`]：原地覆写的写取三态（没持久 / 新旧都读不出 / 持久），其余取两态。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Layer0SegmentExpansion {
    /// 不展开：只以整段持久进入后面的状态。
    NotExpanded,
    /// 段内每次写各取它的几态、任意组合，去掉整段全持久那一个（D13（验证路线） 已定项 4 的全量）：
    /// n 个写里 m 个是原地覆写时 3^m · 2^(n−m) − 1 个；m = 0 时就是任意真子集 2^n − 1 个。
    EveryProperSubset,
    /// 甲二（层 0 规模三轮判完、用户 2026-09-26 定的平时快档）：段内原地写（单元写之外的写）各取它的几态、任意组合 × 单元写（COW）只取
    /// 全不落或全落，去掉整段全落那一个。原地写 k 个（其中 m 个是原地覆写）、单元写 c 个：3^m · 2^(k−m) ·（c > 0 时 2，否则 1）− 1 个，
    /// m = 0 时 c > 0 是 2^(k+1) − 1、c = 0 是 2^k − 1（与全量相同）。
    /// 它看不见「单元写只落一部分」才显出来的那一类（C561 那一形就是），替不了全量：提交时照旧全量。
    InPlaceSubsetsWithCopyOnWriteNoneOrAll,
}

impl Layer0SegmentExpansion {
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
    /// 一段的状态数装不进 u64（段里的写太多：2 态的写六十几个就到了）。
    #[must_use]
    pub fn state_count(
        self,
        writes: &[RetainedWrite],
        segment: &[usize],
        tearable: &TearableInPlaceOverwrites,
    ) -> u64 {
        match self {
            Self::NotExpanded => 0,
            Self::EveryProperSubset => tearable.landing_combinations_of(segment) - 1,
            Self::InPlaceSubsetsWithCopyOnWriteNoneOrAll => {
                let (in_place_writes, copy_on_write_writes) =
                    in_place_and_copy_on_write_writes_of(writes, segment);
                let copy_on_write_choices: u64 = if copy_on_write_writes.is_empty() {
                    1
                } else {
                    2
                };
                tearable
                    .landing_combinations_of(&in_place_writes)
                    .checked_mul(copy_on_write_choices)
                    .expect("一段的状态数装得进 u64")
                    - 1
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
/// 2. 长于一个扇区：层 0 的撕裂粒度是 [`SECTOR_BYTES`]，一个扇区的写撕不开（根槽写是一个物理块，照旧两态）；
/// 3. 它罩住的范围里原来有东西：基镜像那一段有非零字节，或写表里更早有一次同一块盘上、带字节（不是整段清零）的写与它重叠。
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
    /// 当成又一条根槽写）。根槽写是一个物理块，层 0 的池都按 512 字节的物理块建，走不到。
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

    /// 这几次写各取它的几态、全部组合的个数。
    fn landing_combinations_of(&self, write_indexes: &[usize]) -> u64 {
        write_indexes
            .iter()
            .try_fold(1u64, |combinations, write_index| {
                combinations.checked_mul(self.landing_choices_of(*write_index).count())
            })
            .expect("一段的状态数装得进 u64")
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

/// 一段里的原地写与单元写（COW），各按段内次序：单元写落在新分出来的槽上，其余（系统配置槽、journal 记录、根槽、清零）都是原地覆盖。
fn in_place_and_copy_on_write_writes_of(
    writes: &[RetainedWrite],
    segment: &[usize],
) -> (Vec<usize>, Vec<usize>) {
    segment
        .iter()
        .copied()
        .partition(|write_index| match writes[*write_index].kind {
            StepKind::UnitWrite => false,
            StepKind::ZeroFill
            | StepKind::SystemConfigurationSlot
            | StepKind::JournalRecord
            | StepKind::RootRecordFua
            | StepKind::Barrier => true,
        })
}

/// 按 `expansion` 展开、每次写只取两态（没持久 / 持久，补第三态之前的口径）时的状态数：各段展开出来的，再加全部持久那一个
/// （全量时就是 [`closed_form_state_count`] 的闭式）。层 0 枚举出来的状态数是
/// [`layer0_state_count_with_torn_in_place_overwrites`]：写表里没有原地覆写的写时两者相等。
#[must_use]
pub fn layer0_state_count(
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    expansion: &dyn Fn(usize, &[usize]) -> Layer0SegmentExpansion,
) -> u64 {
    state_count_of_the_segments(
        writes,
        segments,
        expansion,
        &TearableInPlaceOverwrites::none(writes.len()),
    )
}

/// 层 0 按 `expansion` 枚举出来的状态数（`enumerate_layer0` 一族的 `states`）：原地覆写的写（[`TearableInPlaceOverwrites`]，要看基镜像）
/// 取三态、其余取两态，各段展开出来的，再加全部持久那一个。
#[must_use]
pub fn layer0_state_count_with_torn_in_place_overwrites(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    expansion: &dyn Fn(usize, &[usize]) -> Layer0SegmentExpansion,
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
    expansion: &dyn Fn(usize, &[usize]) -> Layer0SegmentExpansion,
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
pub fn quick_tier_expansion(_segment_index: usize, _segment: &[usize]) -> Layer0SegmentExpansion {
    Layer0SegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll
}

/// 每一段都展开任意真子集（全量）。
#[must_use]
pub fn full_expansion(_segment_index: usize, _segment: &[usize]) -> Layer0SegmentExpansion {
    Layer0SegmentExpansion::EveryProperSubset
}

/// 枚举次序里每个状态的序号怎么落到「第几段、段内每次写怎么落」：次序与单线程逐段逐个组合走相同——
/// 前面的段全持久 + 当前段的一个组合（全量时段内序号按混合进制拆成每次写的一态，数到组合数 − 2；甲二时见
/// [`Self::persisted_writes_of_state`]），最后再加全部持久那一个状态。
struct Layer0StatePlan<'plan> {
    segments: &'plan [Vec<usize>],
    /// 每一段怎么展开，与 `segments` 同序。
    expansion_by_segment: Vec<Layer0SegmentExpansion>,
    /// 每一段展开出来的状态的序号区间，首尾相接、随段号递增；不展开的段是空区间。
    state_ranges_by_segment: Vec<Range<u64>>,
    /// 状态总数：各段展开出来的，再加最后全部持久那一个。
    state_count: u64,
    /// 每一段归哪次发布（[`publish_of_each_segment`]），与 `segments` 同序。
    publish_of_segment: Vec<Layer0PublishOfState>,
    /// 每一段的原地写与单元写（[`in_place_and_copy_on_write_writes_of`]），与 `segments` 同序。
    in_place_and_copy_on_write_by_segment: Vec<(Vec<usize>, Vec<usize>)>,
    /// 哪几次写取三态（[`TearableInPlaceOverwrites`]）。
    tearable: &'plan TearableInPlaceOverwrites,
    /// 枚举用的写表（录制流里的写在前，撕裂镜像与重放接在后面）：持久集合按它给。
    writes_with_torn_images: &'plan WritesWithTornImages,
}

impl<'plan> Layer0StatePlan<'plan> {
    /// `expansion` 只在调用线程上逐段问一次，所以它不必能跨线程。`writes` 是录制流里的写（不带撕裂镜像）。
    fn new(
        writes: &[RetainedWrite],
        segments: &'plan [Vec<usize>],
        expansion: &dyn Fn(usize, &[usize]) -> Layer0SegmentExpansion,
        tearable: &'plan TearableInPlaceOverwrites,
        writes_with_torn_images: &'plan WritesWithTornImages,
    ) -> Self {
        let expansion_by_segment: Vec<Layer0SegmentExpansion> = segments
            .iter()
            .enumerate()
            .map(|(segment_index, segment)| expansion(segment_index, segment))
            .collect();
        let mut next_ordinal = 0u64;
        let state_ranges_by_segment = segments
            .iter()
            .zip(&expansion_by_segment)
            .map(|(segment, segment_expansion)| {
                let first_ordinal = next_ordinal;
                next_ordinal = next_ordinal
                    .checked_add(segment_expansion.state_count(writes, segment, tearable))
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
            in_place_and_copy_on_write_by_segment: segments
                .iter()
                .map(|segment| in_place_and_copy_on_write_writes_of(writes, segment))
                .collect(),
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
    fn publish_of_state(&self, ordinal: u64) -> Layer0PublishOfState {
        self.publish_of_segment
            .get(self.segment_of_state(ordinal))
            .copied()
            .unwrap_or(Layer0PublishOfState::EveryWritePersisted)
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
        if let Some(segment) = self.segments.get(segment_of_state) {
            let ordinal_within_the_segment =
                ordinal - self.state_ranges_by_segment[segment_of_state].start;
            match self.expansion_by_segment[segment_of_state] {
                Layer0SegmentExpansion::NotExpanded => {
                    unreachable!(
                        "不展开的段序号区间是空的，没有状态落在它里面（段 {segment_of_state}）"
                    )
                }
                Layer0SegmentExpansion::EveryProperSubset => {
                    self.assign_landings(&mut landings, ordinal_within_the_segment, segment);
                }
                Layer0SegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll => {
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

    /// [`Self::segment_of_state`] 交回的段号换成发现表与进度行里的段：最后全部持久那一个状态不在任何一段里。
    fn segment_of_state_for_findings(&self, segment_index: usize) -> Layer0SegmentOfState {
        if segment_index < self.segments.len() {
            Layer0SegmentOfState::Segment(segment_index)
        } else {
            Layer0SegmentOfState::AllPersisted
        }
    }

    /// 进度行里的段号：最后全部持久那一个状态不在任何一段里，报成 `all_persisted`。
    fn segment_label(&self, segment_index: usize) -> String {
        self.segment_of_state_for_findings(segment_index).name()
    }
}

/// 把 [0, `state_count`) 按序号切成首尾相接的区间。
fn state_slices(state_count: u64, parallelism: &Layer0Parallelism) -> Vec<Range<u64>> {
    let states_per_slice = match parallelism.slice_length {
        Layer0SliceLength::ScaledToWorkerThreads => {
            let worker_threads =
                u64::try_from(parallelism.worker_threads.get()).expect("线程数装得进 u64");
            let slice_count = LAYER0_MINIMUM_SLICE_COUNT
                .max(worker_threads.saturating_mul(LAYER0_SLICES_PER_WORKER_THREAD));
            state_count
                .div_ceil(slice_count)
                .max(LAYER0_MINIMUM_STATES_PER_SLICE)
        }
        Layer0SliceLength::StatesPerSlice(states_per_slice) => states_per_slice.get(),
    };
    (0..state_count.div_ceil(states_per_slice))
        .map(|slice_index| {
            let first_ordinal = slice_index * states_per_slice;
            let end_ordinal = first_ordinal
                .saturating_add(states_per_slice)
                .min(state_count);
            first_ordinal..end_ordinal
        })
        .collect()
}

/// 续跑时的切法不随线程数变（层 0 规模第一轮判决 R2）：片数取这么多，每片至少 [`LAYER0_MINIMUM_STATES_PER_SLICE`] 个状态。
/// 第二条流全量五十多亿个状态时每片约八万五千个（32 线程每秒约两万多个时一片一分钟上下，推的），被杀时最多丢每个线程手上那一片。
const LAYER0_RESUMABLE_SLICE_COUNT: u64 = 65_536;

/// 续跑时每片几个状态：只看状态数，不看线程数——换了线程数续跑，片方案照旧对得上。
fn states_per_slice_independent_of_worker_threads(state_count: u64) -> NonZeroU64 {
    NonZeroU64::new(
        state_count
            .div_ceil(LAYER0_RESUMABLE_SLICE_COUNT)
            .max(LAYER0_MINIMUM_STATES_PER_SLICE),
    )
    .expect("每片至少 16 个状态，不是 0")
}

/// 这一趟按什么切片：续跑、分片跑一片、merge 时给的是「按线程数定片长」就改按状态数定（R2：换了线程数续跑，片方案照旧对得上；
/// 分片时 n 台线程数各不相同，切法也要逐片相同，merge 核 n 份账本的切法相同）；调用方给了固定片长、或不续跑，照给的切。
fn slicing_of_the_run(
    parallelism: &Layer0Parallelism,
    resume: &Layer0Resume,
    state_count: u64,
) -> Layer0Parallelism {
    let slicing_rule = match resume {
        Layer0Resume::KeepProgressFile(_)
        | Layer0Resume::RunOneShardKeepingProgressFile(_)
        | Layer0Resume::MergeShardLedgers(_) => SlicingRule::IndependentOfWorkerThreads,
        Layer0Resume::NoProgressFile => SlicingRule::AsGiven,
    };
    Layer0Parallelism {
        worker_threads: parallelism.worker_threads,
        worker_threads_source: parallelism.worker_threads_source,
        slice_length: match (slicing_rule, parallelism.slice_length) {
            (SlicingRule::IndependentOfWorkerThreads, Layer0SliceLength::ScaledToWorkerThreads) => {
                Layer0SliceLength::StatesPerSlice(states_per_slice_independent_of_worker_threads(
                    state_count,
                ))
            }
            (
                SlicingRule::IndependentOfWorkerThreads,
                Layer0SliceLength::StatesPerSlice(states_per_slice),
            )
            | (SlicingRule::AsGiven, Layer0SliceLength::StatesPerSlice(states_per_slice)) => {
                Layer0SliceLength::StatesPerSlice(states_per_slice)
            }
            (SlicingRule::AsGiven, Layer0SliceLength::ScaledToWorkerThreads) => {
                Layer0SliceLength::ScaledToWorkerThreads
            }
        },
    }
}

/// 片长按线程数定的那一档要不要换成按状态数定。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SlicingRule {
    /// 续跑、分片跑一片、merge：片方案不随线程数变。
    IndependentOfWorkerThreads,
    /// 不续跑：照给的切。
    AsGiven,
}

/// 逐状态的观察者：拿到崩溃镜像、看 journal 那一遍恢复的报告，与这一片的观察者计数（要随进度文件续跑的累计记在这里）。
pub type Layer0StateObserver<'observer> =
    &'observer mut dyn FnMut(&CrashImage<'_>, &RecoveryReport, &mut Layer0ObserverCounts);

/// 工作线程评状态时 panic，就把旗子立起来：别的工作线程看到旗子不再领新片，调用线程不必等整条流跑完才知道红了。
struct RaiseFlagWhenPanicking<'flag>(&'flag AtomicBool);

impl Drop for RaiseFlagWhenPanicking<'_> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            self.0.store(true, Ordering::Relaxed);
        }
    }
}

/// 工作线程要不要把每个状态的持久集合与看 journal 那一遍恢复的报告带回调用线程。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StateReportRetention {
    /// 有观察者：带回去，由调用线程按序号交给它。
    HandEachStateToObserver,
    /// 没有观察者：只带计数回去（全量流上两百多万个报告不必留）。
    CountOnly,
}

/// 一片跑完交回调用线程的东西。
struct FinishedSlice {
    slice_index: usize,
    tally: Layer0Tally,
    /// 按序号排好的（持久集合，看 journal 那一遍恢复的报告）；`CountOnly` 时是空的。
    observed_states: Vec<(Vec<bool>, RecoveryReport)>,
}

/// 在工作线程上跑一片：每个状态自己建崩溃镜像、自己记进这一片的计数；基线与写表只读、各线程共用。
/// `writes` 是枚举用的写表（[`WritesWithTornImages`]：录制流里的写在前，撕裂镜像与重放接在后面），与计划给的持久集合逐条对应。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数各是一样东西：基线、写表、状态计划、这一片的区间、被判的根、版本表、要不要带回报告"
)]
fn evaluate_state_slice(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    plan: &Layer0StatePlan<'_>,
    slice_index: usize,
    slice: Range<u64>,
    judged_root_index: usize,
    versions: &[PublishedVersion],
    retention: StateReportRetention,
) -> FinishedSlice {
    let mut tally = Layer0Tally::default();
    let mut observed_states = Vec::new();
    for ordinal in slice {
        let position = Layer0StatePosition {
            ordinal,
            segment: plan.segment_of_state_for_findings(plan.segment_of_state(ordinal)),
            publish: plan.publish_of_state(ordinal),
        };
        *tally.states_by_publish.entry(position.publish).or_insert(0) += 1;
        let persisted = plan.persisted_writes_of_state(ordinal);
        match retention {
            StateReportRetention::HandEachStateToObserver => {
                let consulted_report = evaluate_state_recording_findings(
                    base,
                    writes,
                    persisted.clone(),
                    judged_root_index,
                    versions,
                    Some(position),
                    &mut tally,
                );
                observed_states.push((persisted, consulted_report));
            }
            StateReportRetention::CountOnly => {
                evaluate_state_recording_findings(
                    base,
                    writes,
                    persisted,
                    judged_root_index,
                    versions,
                    Some(position),
                    &mut tally,
                );
            }
        }
    }
    FinishedSlice {
        slice_index,
        tally,
        observed_states,
    }
}

/// 枚举：前面的段全持久 + 当前段每次写各取它的几态的任意组合（去掉整段全持久；原地覆写的写三态，[`TearableInPlaceOverwrites`]），
/// 最后再加全部持久那一个状态；`expand` 决定哪一段展开
/// （不展开的段只以整段持久进入后面的状态，平时 `cargo test` 里跳过 18 个写那一段就靠它）。`judged_root_index` 是被判的那次根槽 FUA 写。
/// 按状态序号区间切片、多线程跑，线程数见 [`Layer0Parallelism::from_environment`]。
#[must_use]
pub fn enumerate_layer0_selecting_versions(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: &[PublishedVersion],
    expand: &dyn Fn(usize, &[usize]) -> bool,
) -> Layer0Tally {
    enumerate_layer0_in_state_slices(
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        &|segment_index, segment| expansion_of_the_yes_or_no_choice(expand(segment_index, segment)),
        Layer0Parallelism::from_environment(),
        None,
        &Layer0Resume::NoProgressFile,
    )
}

/// 「展不展开」这一种选法换成 [`Layer0SegmentExpansion`]：展开就是任意真子集。
fn expansion_of_the_yes_or_no_choice(is_expanded: bool) -> Layer0SegmentExpansion {
    if is_expanded {
        Layer0SegmentExpansion::EveryProperSubset
    } else {
        Layer0SegmentExpansion::NotExpanded
    }
}

/// 平时快档（甲二，[`Layer0SegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll`]）：每一段都按甲二展开、多版本，不留进度文件。
#[must_use]
pub fn enumerate_layer0_quick_tier_versions(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: &[PublishedVersion],
) -> Layer0Tally {
    enumerate_layer0_in_state_slices(
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        &quick_tier_expansion,
        Layer0Parallelism::from_environment(),
        None,
        &Layer0Resume::NoProgressFile,
    )
}

/// 同 [`enumerate_layer0_selecting_versions`]，每个状态评完之后把这个崩溃镜像与看 journal 那一遍恢复的报告交给 `observe_state`：
/// 用例按自己独立算的谓词逐状态核恢复（预置的残留记录该不该施加、改坏 tail 之后终态与没改坏的是否逐项相等）。
/// 观察者在调用线程上、按状态序号从小到大调用，次序与单线程逐个跑相同，所以它不必能跨线程。不留进度文件：
/// 这个观察者的累计记在它自己的变量里，续跑接不上（要续跑的用 [`enumerate_layer0_in_state_slices`]，累计记进观察者计数）。
#[must_use]
pub fn enumerate_layer0_selecting_versions_observing_each_state(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: &[PublishedVersion],
    expand: &dyn Fn(usize, &[usize]) -> bool,
    observe_state: &mut dyn FnMut(&CrashImage<'_>, &RecoveryReport),
) -> Layer0Tally {
    let mut observe_without_counting =
        |image: &CrashImage<'_>, report: &RecoveryReport, _counts: &mut Layer0ObserverCounts| {
            observe_state(image, report);
        };
    enumerate_layer0_in_state_slices(
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        &|segment_index, segment| expansion_of_the_yes_or_no_choice(expand(segment_index, segment)),
        Layer0Parallelism::from_environment(),
        Some(&mut observe_without_counting),
        &Layer0Resume::NoProgressFile,
    )
}

/// 一段展开方式在计划哈希里的名字。
fn expansion_name(expansion: Layer0SegmentExpansion) -> &'static str {
    match expansion {
        Layer0SegmentExpansion::NotExpanded => "not_expanded",
        Layer0SegmentExpansion::EveryProperSubset => "every_proper_subset",
        Layer0SegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll => {
            "in_place_subsets_with_copy_on_write_none_or_all"
        }
    }
}

/// 往计划哈希的消息里追加一段：先写长度再写字节，两段之间不会串。
fn append_length_prefixed(message: &mut Vec<u8>, bytes: &[u8]) {
    message.extend_from_slice(
        &u64::try_from(bytes.len())
            .expect("长度装得进 u64")
            .to_le_bytes(),
    );
    message.extend_from_slice(bytes);
}

/// 枚举计划哈希（进度文件分格的第三样，层 0 规模第三轮判决 U1）：基线的每个扇区、写表（盘、种类、FUA、偏移、内容；枚举用的那一张，
/// 撕裂镜像与重放在内：补第三态之后同一条流的计划哈希变了，旧进度文件不再相认）、段与每段怎么展开、
/// 被判的根、版本表（实例、txg、内容）、状态数、片方案、有没有观察者，逐项长度前缀拼起来取 SHA-256。
/// 同一条流、同一组状态只换了版本表（或换了展开方式、切法）就是另一个哈希，两次枚举的进度互不相认。
/// 分片跑一片时末尾另拼 `shard <i>/<n>`：分片的进度文件与单机的、与别的片的互不相认；不分片时不拼，单机的哈希与分片之前的相同。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数各是计划里的一样：基线、写表、段计划、被判的根、版本表、片方案、有没有观察者、第几片"
)]
fn layer0_plan_hash(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    plan: &Layer0StatePlan<'_>,
    judged_root_index: usize,
    versions: &[PublishedVersion],
    slices: &[Range<u64>],
    has_observer: bool,
    shard: Option<Layer0ShardOfShards>,
) -> String {
    let mut message: Vec<u8> = Vec::new();
    append_length_prefixed(&mut message, b"singlefs layer0 enumeration plan 1");
    message.extend_from_slice(&base.device_size_in_bytes.to_le_bytes());
    for (device, sparse_device) in &base.devices {
        message.extend_from_slice(&device.0.to_le_bytes());
        message.extend_from_slice(
            &u64::try_from(sparse_device.written_sectors().len())
                .expect("扇区数装得进 u64")
                .to_le_bytes(),
        );
        for (sector, bytes) in sparse_device.written_sectors() {
            message.extend_from_slice(&sector.to_le_bytes());
            append_length_prefixed(&mut message, bytes);
        }
    }
    for write in writes {
        message.extend_from_slice(&write.device.0.to_le_bytes());
        append_length_prefixed(&mut message, write.kind.name().as_bytes());
        message.push(u8::from(write.is_force_unit_access));
        message.extend_from_slice(&write.offset.0.to_le_bytes());
        match &write.contents {
            WrittenContents::Bytes(bytes) => {
                message.push(0);
                append_length_prefixed(&mut message, bytes);
            }
            WrittenContents::Zeros { length } => {
                message.push(1);
                message.extend_from_slice(&length.to_le_bytes());
            }
        }
    }
    for (segment, expansion) in plan.segments.iter().zip(&plan.expansion_by_segment) {
        append_length_prefixed(&mut message, expansion_name(*expansion).as_bytes());
        message.extend_from_slice(&u64::try_from(segment.len()).expect("段长").to_le_bytes());
        for write_index in segment {
            message.extend_from_slice(&u64::try_from(*write_index).expect("下标").to_le_bytes());
        }
    }
    message.extend_from_slice(
        &u64::try_from(judged_root_index)
            .expect("下标")
            .to_le_bytes(),
    );
    for version in versions {
        message.extend_from_slice(&version.instance.0.to_le_bytes());
        message.extend_from_slice(&version.checkpoint_txg.0.to_le_bytes());
        append_length_prefixed(&mut message, &version.content);
    }
    message.extend_from_slice(&plan.state_count.to_le_bytes());
    for slice in slices {
        message.extend_from_slice(&slice.start.to_le_bytes());
        message.extend_from_slice(&slice.end.to_le_bytes());
    }
    message.push(u8::from(has_observer));
    if let Some(shard) = shard {
        append_length_prefixed(&mut message, format!("shard {}", shard.text()).as_bytes());
    }
    singlefs_harness::sha256::sha256_hexadecimal(&message)
}

/// 并片时报发现：每并进一片之前，这一片带来的新签名、跨台阶各打一行 `LAYER0_FINDING`（标准输出），开了发现日志的另追加进这一节；
/// 跑完打一行 `LAYER0_FINDINGS`，把发现日志里这一节换成定稿（`layer0_progress::findings_log_final_lines`）。
struct Layer0FindingsReporter {
    /// 报过「新签名」的签名与它的号：从 1 起，按报的次序。
    number_by_signature: BTreeMap<Layer0FindingSignature, usize>,
    log_section: Option<(Layer0FindingsLogSection, Layer0FindingsLogBegin)>,
}

impl Layer0FindingsReporter {
    /// 开跑、并任何一片之前调：开了发现日志的，在文件末尾开一节。
    fn begin(findings_log: &Layer0FindingsLog, begin: Layer0FindingsLogBegin) -> Self {
        let log_section = match findings_log {
            Layer0FindingsLog::NotWritten => None,
            Layer0FindingsLog::AppendedTo(path) => {
                Some((Layer0FindingsLogSection::begin(path, &begin), begin))
            }
        };
        Self {
            number_by_signature: BTreeMap::new(),
            log_section,
        }
    }

    /// 紧跟在后面的一片并进 `merged` 之前调：报这一片带来的事（[`Layer0Findings::events_of_absorbing`]）。
    fn report_before_absorbing(
        &mut self,
        merged: &Layer0Findings,
        following_slice: &Layer0Findings,
    ) {
        for event in merged.events_of_absorbing(following_slice) {
            let (standard_output_line, log_line) = match event {
                Layer0FindingEvent::NewSignature {
                    signature,
                    first_sample,
                } => {
                    let number = self.number_by_signature.len() + 1;
                    self.number_by_signature.insert(signature.clone(), number);
                    (
                        findings_standard_output_new_signature_line(
                            number,
                            signature,
                            first_sample,
                        ),
                        findings_log_new_signature_line(number, signature, first_sample),
                    )
                }
                Layer0FindingEvent::ThresholdCrossed {
                    signature,
                    states_at_least,
                } => {
                    let number = *self.number_by_signature.get(signature).expect(
                        "跨台阶的签名先报过新签名：同一片里新签名排在跨台阶前面，更早的片里报过的记在表里",
                    );
                    (
                        findings_standard_output_threshold_line(number, signature, states_at_least),
                        findings_log_threshold_line(number, signature, states_at_least),
                    )
                }
            };
            println!("{standard_output_line}");
            if let Some((section, _begin)) = self.log_section.as_mut() {
                section.append_line(&log_line);
            }
        }
    }

    /// 这一趟并完：打一行 `LAYER0_FINDINGS`（`shard_fields` 是分片跑一片时接在后面的那一串，别的时候空），
    /// 开了发现日志的把这一节换成定稿。`states_of_this_run` 是这一趟评过的状态数（分片跑一片时是这一片的）。
    ///
    /// # Panics
    /// 跑的过程中报的号与定稿里的号对不上（不变量被破坏：片没按状态序号从小到大并）。
    fn finish(self, findings: &Layer0Findings, states_of_this_run: u64, shard_fields: &str) {
        // 号从 1 起，没报过的记成 0：对不上时一眼看得出是哪个签名没报过。
        let reported_numbers_in_first_state_order: Vec<usize> = findings
            .signatures_in_first_state_order()
            .into_iter()
            .map(|(signature, _finding)| {
                self.number_by_signature
                    .get(signature)
                    .copied()
                    .unwrap_or(0)
            })
            .collect();
        assert_eq!(
            (
                reported_numbers_in_first_state_order,
                self.number_by_signature.len()
            ),
            (
                (1..=findings.by_signature.len()).collect::<Vec<usize>>(),
                findings.by_signature.len()
            ),
            "跑的过程中报的号就是定稿里按最先那个状态排的次序"
        );
        println!(
            "{}{shard_fields}",
            findings_standard_output_summary_line(findings, states_of_this_run)
        );
        if let Some((section, begin)) = self.log_section {
            section.replace_with_final_lines(&findings_log_final_lines(
                &begin,
                findings,
                states_of_this_run,
            ));
        }
    }
}

/// 调用线程上等着按片号次序并进来的一片：这一趟跑完的，或从进度文件读回来的。
enum SliceWaitingToBeMerged {
    FreshlyRun(FinishedSlice),
    RestoredFromTheProgressFile(Layer0Tally),
}

/// 层 0 枚举的本体：把 [0, 状态数) 按 `parallelism` 切成首尾相接的序号区间，工作线程按片号从小到大领片、各自跑完交回；
/// 调用线程收到一片就打一行 `LAYER0_PROGRESS`（片号、序号区间、段号、已跑完的片数与状态数），再按片号从小到大并计数、调观察者。
/// 计数按片的次序相加，「第一处违例」取序号最小的那一处（`Layer0Tally` 的并片），结果与线程数、切法无关。
/// 开跑与跑完各打一行 `LAYER0_PARALLEL_START` / `LAYER0_PARALLEL_FINISHED`（状态数、片数、实际起的工作线程数、线程数从哪来、
/// 续跑接上的片数与这一趟跑的片数、耗时）。`expansion` 决定每一段怎么展开（全量、甲二、不展开）。
///
/// 续跑（`resume` 是 [`Layer0Resume::KeepProgressFile`]，层 0 规模三轮判决 R1–R5、U1–U3）：
/// - 切法不随线程数变（`parallelism` 给的是按线程数定片长时，改按状态数定，[`states_per_slice_independent_of_worker_threads`]）；
/// - 进度文件按输入指纹、流名与计划哈希（[`layer0_plan_hash`]）分格，开跑时读回核过的片不再跑（打一行 `LAYER0_RESUME`）；
/// - 一片在观察者看完、没 panic 之后才写进进度文件，观察者在这一片上的计数随片行一起写；按片号次序写；
/// - 这一趟有工作线程或观察者 panic（判红），删掉进度文件；
/// - 并完之后核「读回的片 + 这一趟跑的片 = 总片数」，有观察者时核它看过的状态数（连读回那几片记的）等于状态数；
/// - 整条流跑完删掉进度文件（只供测试强制进入的那一档留着）。
///
/// 双机分片（`resume` 是 [`Layer0Resume::RunOneShardKeepingProgressFile`] 或 [`Layer0Resume::MergeShardLedgers`]，里程碑三第六项）见
/// [`enumerate_layer0_in_state_slices_or_one_shard`]；这个入口要整条流的计数，分片跑一片的 `resume` 不收。
///
/// # Panics
/// 某个工作线程在评状态时 panic（恢复或 checker 里的断言：别的线程不再领新片，手上那一片跑完就退）；观察者 panic；
/// 有一片领了却没交回、读回的片加跑的片不等于总片数、观察者看过的状态数不等于状态数（不变量被破坏）；进度文件读写失败；
/// `resume` 是分片跑一片（这个入口交不出整条流的计数：分片跑的用例调 [`enumerate_layer0_in_state_slices_or_one_shard`]）；merge 的账本核不齐。
#[allow(
    clippy::too_many_arguments,
    reason = "前六个与单线程时的枚举相同，多出来的是切法、观察者与续跑"
)]
#[must_use]
pub fn enumerate_layer0_in_state_slices(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: &[PublishedVersion],
    expansion: &dyn Fn(usize, &[usize]) -> Layer0SegmentExpansion,
    parallelism: Layer0Parallelism,
    observe_state: Option<Layer0StateObserver<'_>>,
    resume: &Layer0Resume,
) -> Layer0Tally {
    match resume {
        Layer0Resume::RunOneShardKeepingProgressFile(shard_run) => panic!(
            "分片跑一片（第 {} 片）交不出整条流的计数：这条用例要分片跑，改调 enumerate_layer0_in_state_slices_or_one_shard、拿到 OneShardWrittenToItsLedger 就收工",
            shard_run.shard.text()
        ),
        Layer0Resume::NoProgressFile
        | Layer0Resume::KeepProgressFile(_)
        | Layer0Resume::MergeShardLedgers(_) => {}
    }
    match enumerate_layer0_in_state_slices_or_one_shard(
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        expansion,
        parallelism,
        observe_state,
        resume,
    ) {
        Layer0EnumerationOutcome::WholeStream(tally) => tally,
        Layer0EnumerationOutcome::OneShardWrittenToItsLedger(written) => unreachable!(
            "开头已经挡掉分片跑一片的 resume，这里只会交回整条流的计数：{:?}",
            written.shard
        ),
    }
}

/// 一趟层 0 枚举的结局。
#[derive(Debug)]
pub enum Layer0EnumerationOutcome {
    /// 整条流的计数：不分片跑完全部切片，或 merge 把 n 份账本核齐、按切片序号并完。
    WholeStream(Layer0Tally),
    /// 只跑了分给这一片的切片、计数写进了这一片的账本（`SINGLEFS_LAYER0_SHARD=<i>/<n>`）：整条流的计数要等 merge，
    /// 用例拿到它就收工，不拿它判用例钉死的数。
    OneShardWrittenToItsLedger(Layer0ShardLedgerWritten),
}

/// 分片跑完的一片：第几片、账本在哪、这一片的切片按序号并起来的计数（不是整条流的）。
#[derive(Debug)]
pub struct Layer0ShardLedgerWritten {
    pub shard: Layer0ShardOfShards,
    pub ledger_path: std::path::PathBuf,
    pub tally_of_this_shard: Layer0Tally,
}

/// 同 [`enumerate_layer0_in_state_slices`]，另收双机分片的两种 `resume`（里程碑三第六项，`.claude/kb/milestone/03-third-txn.md` 第六节）：
/// - [`Layer0Resume::RunOneShardKeepingProgressFile`]：只跑切片序号 `slice_index % n == i` 的切片（片方案不随线程数变，
///   [`slicing_of_the_run`]），进度文件照续跑走（名字、文件头、计划哈希另带这一片）；这一片全部切片并完之后写账本
///   （`layer0_progress::write_shard_ledger`），再删进度文件，交回 [`Layer0EnumerationOutcome::OneShardWrittenToItsLedger`]。
///   开跑打一行 `LAYER0_SHARD mode=run`；`LAYER0_PARALLEL_START` / `FINISHED` 两行的 `states=` / `slices=` 是这一片的，另带 `shard=`。
/// - [`Layer0Resume::MergeShardLedgers`]：不枚举、不调观察者；读 n 份账本、核齐（`layer0_progress::read_shard_ledgers_for_merge`），
///   按切片序号从小到大并（与单机并片的次序相同，「第一处」取序号最小的），交回整条流的计数。打一行 `LAYER0_SHARD mode=merge`，
///   `LAYER0_PARALLEL_START` / `FINISHED` 两行报 n 片的工作线程之和与逐片的数（`shard_worker_threads=` 等，逗号分隔、按第几片排）。
///
/// 发现表（[`Layer0Findings`]）：并片时每一片带来的新签名、跨台阶各打一行 `LAYER0_FINDING`，跑完打一行 `LAYER0_FINDINGS`；
/// 发现日志写不写、写到哪取环境变量 `SINGLEFS_LAYER0_FINDINGS_FILE`（`layer0_progress::Layer0FindingsLog::from_environment`），
/// 写法见 [`enumerate_layer0_in_state_slices_or_one_shard_with_findings_log`]。
///
/// # Panics
/// 同 [`enumerate_layer0_in_state_slices`]；另有账本写失败，merge 时缺账本、账本文件头与这一趟不同（输入指纹、切片方案、工具链……）、
/// 两份账本同一片、账本里的切片不归它或缺切片（逐条说清是第几片、哪一处）；发现日志的环境变量设了却是空串。
#[allow(
    clippy::too_many_arguments,
    reason = "前六个与单线程时的枚举相同，多出来的是切法、观察者与续跑"
)]
#[must_use]
pub fn enumerate_layer0_in_state_slices_or_one_shard(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: &[PublishedVersion],
    expansion: &dyn Fn(usize, &[usize]) -> Layer0SegmentExpansion,
    parallelism: Layer0Parallelism,
    observe_state: Option<Layer0StateObserver<'_>>,
    resume: &Layer0Resume,
) -> Layer0EnumerationOutcome {
    enumerate_layer0_in_state_slices_or_one_shard_with_findings_log(
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        expansion,
        parallelism,
        observe_state,
        resume,
        &Layer0FindingsLog::from_environment(),
    )
}

/// 同 [`enumerate_layer0_in_state_slices_or_one_shard`]，发现日志由调用方给（用例不改进程的环境变量）。
/// `findings_log` 是 [`Layer0FindingsLog::AppendedTo`] 时这一趟往那个文件末尾追加一节：开跑先写 begin 行，每并进一片把它带来的
/// 新签名、跨台阶各追加一行并落盘；跑完把这一节换成定稿（每个签名一行、一行汇总），定稿与线程数、切法、续跑与否无关，
/// merge 那一趟写的定稿与单机跑同一条流的逐字节相同。一个进程里同一时刻只开一节（别的带发现日志的枚举排队）。
///
/// # Panics
/// 同 [`enumerate_layer0_in_state_slices_or_one_shard`]；另有发现日志建、写、落盘失败。
#[allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "前六个与单线程时的枚举相同，多出来的是切法、观察者、续跑与发现日志；续跑的读回、并片、落盘、收尾是同一件事的几步，拆开要把可变状态来回传"
)]
#[must_use]
pub fn enumerate_layer0_in_state_slices_or_one_shard_with_findings_log(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: &[PublishedVersion],
    expansion: &dyn Fn(usize, &[usize]) -> Layer0SegmentExpansion,
    parallelism: Layer0Parallelism,
    mut observe_state: Option<Layer0StateObserver<'_>>,
    resume: &Layer0Resume,
    findings_log: &Layer0FindingsLog,
) -> Layer0EnumerationOutcome {
    let tearable = TearableInPlaceOverwrites::of(base, writes);
    let writes_with_torn_images = WritesWithTornImages::of(base, writes, segments, &tearable);
    let plan = Layer0StatePlan::new(
        writes,
        segments,
        expansion,
        &tearable,
        &writes_with_torn_images,
    );
    // 从这里往下，评状态、交给观察者的崩溃镜像、计划哈希用的都是枚举用的写表：录制流里的写下标不变，撕裂镜像与重放接在后面。
    let enumerated_writes = writes_with_torn_images.writes.as_slice();
    let slices = state_slices(
        plan.state_count,
        &slicing_of_the_run(&parallelism, resume, plan.state_count),
    );
    let has_observer = observe_state.is_some();
    let (progress_settings, shard_run) = match resume {
        Layer0Resume::NoProgressFile => (None, None),
        Layer0Resume::KeepProgressFile(settings) => (Some(settings), None),
        Layer0Resume::RunOneShardKeepingProgressFile(shard_run) => {
            (Some(&shard_run.progress), Some(shard_run))
        }
        Layer0Resume::MergeShardLedgers(merge) => {
            let whole_plan = Layer0ProgressPlan {
                plan_hash: layer0_plan_hash(
                    base,
                    enumerated_writes,
                    &plan,
                    judged_root_index,
                    versions,
                    &slices,
                    has_observer,
                    None,
                ),
                state_count: plan.state_count,
                slices: slices.clone(),
                has_observer,
                shard: None,
            };
            return Layer0EnumerationOutcome::WholeStream(merge_the_shard_ledgers(
                &whole_plan,
                merge,
                findings_log,
            ));
        }
    };
    let shard = shard_run.map(|run| run.shard);
    // 这一趟负责的切片：不分片是全部，分片跑一片是序号模 n 等于 i 的那些（交错分）。
    let slices_of_this_run: Vec<usize> = (0..slices.len())
        .filter(|slice_index| shard.is_none_or(|shard| shard.owns_slice(*slice_index)))
        .collect();
    let states_of_this_run: u64 = slices_of_this_run
        .iter()
        .map(|slice_index| slices[*slice_index].end - slices[*slice_index].start)
        .sum();
    if let Some(run) = shard_run {
        println!(
            "LAYER0_SHARD mode=run shard={} owned_slices={} all_slices={} states={states_of_this_run} all_states={} ledger={}",
            run.shard.text(),
            slices_of_this_run.len(),
            slices.len(),
            plan.state_count,
            shard_ledger_path(&run.progress.directory, &run.progress.stream_name, run.shard)
                .display()
        );
    }
    let (mut progress_file, restored_slices) = match progress_settings {
        None => (None, BTreeMap::new()),
        Some(settings) => {
            let progress_plan = Layer0ProgressPlan {
                plan_hash: layer0_plan_hash(
                    base,
                    enumerated_writes,
                    &plan,
                    judged_root_index,
                    versions,
                    &slices,
                    has_observer,
                    shard,
                ),
                state_count: plan.state_count,
                slices: slices.clone(),
                has_observer,
                shard,
            };
            let opened = Layer0ProgressFile::open(settings, &progress_plan);
            println!(
                "LAYER0_RESUME progress_file={} restored_slices={}/{} half_lines_dropped={} discarded_because={}",
                opened.file.path().display(),
                opened.restored_slices.len(),
                slices.len(),
                opened.half_lines_dropped,
                opened.discarded_because.as_deref().unwrap_or("none")
            );
            (Some(opened.file), opened.restored_slices)
        }
    };
    let _delete_the_progress_file_if_this_run_goes_red =
        progress_file
            .as_ref()
            .map(|file| DeleteTheProgressFileWhenPanicking {
                path: file.path().to_path_buf(),
            });
    let slices_to_run: Vec<usize> = slices_of_this_run
        .iter()
        .copied()
        .filter(|slice_index| !restored_slices.contains_key(slice_index))
        .collect();
    let restored_slice_count = restored_slices.len();
    let spawned_worker_threads = parallelism.worker_threads.get().min(slices_to_run.len());
    let retention = match observe_state {
        Some(_) => StateReportRetention::HandEachStateToObserver,
        None => StateReportRetention::CountOnly,
    };
    // 分片跑一片时两行 LAYER0_PARALLEL_* 的 states= / slices= 是这一片的，另带整条流的数；不分片时什么都不加，与分片之前逐字相同。
    let shard_fields = shard.map_or_else(String::new, |shard| {
        format!(
            " shard={} all_states={} all_slices={}",
            shard.text(),
            plan.state_count,
            slices.len()
        )
    });
    let started = Instant::now();
    println!(
        "LAYER0_PARALLEL_START states={states_of_this_run} slices={} states_per_slice={} worker_threads={spawned_worker_threads} configured_worker_threads={} worker_threads_source={} resumed_slices={restored_slice_count} freshly_run_slices={}{shard_fields}",
        slices_of_this_run.len(),
        slices.first().map_or(0, |slice| slice.end - slice.start),
        parallelism.worker_threads,
        parallelism.worker_threads_source.name(),
        slices_to_run.len()
    );
    let mut findings_reporter = Layer0FindingsReporter::begin(
        findings_log,
        Layer0FindingsLogBegin {
            stream_name: progress_settings
                .map(|settings| settings.stream_name.as_str().to_string()),
            whole_stream_states: plan.state_count,
            shard: shard.map(|shard_of_this_run| (shard_of_this_run, states_of_this_run)),
        },
    );
    let next_position_in_slices_to_run = AtomicUsize::new(0);
    let some_worker_thread_panicked = AtomicBool::new(false);
    let merged_slices_of_this_run = std::thread::scope(|scope| {
        // 收发两端都建在这个闭包里：观察者 panic 时接收端随闭包一起丢掉，工作线程下一次交片就发不出去、随即退出。
        let (finished_slice_sender, finished_slice_receiver) = mpsc::channel::<FinishedSlice>();
        for _ in 0..spawned_worker_threads {
            let finished_slice_sender = finished_slice_sender.clone();
            let plan = &plan;
            let slices = &slices;
            let slices_to_run = &slices_to_run;
            let next_position_in_slices_to_run = &next_position_in_slices_to_run;
            let some_worker_thread_panicked = &some_worker_thread_panicked;
            scope.spawn(move || loop {
                let _raise_the_flag_if_this_thread_panics =
                    RaiseFlagWhenPanicking(some_worker_thread_panicked);
                if some_worker_thread_panicked.load(Ordering::Relaxed) {
                    break;
                }
                let position = next_position_in_slices_to_run.fetch_add(1, Ordering::Relaxed);
                let Some(slice_index) = slices_to_run.get(position).copied() else {
                    break;
                };
                let finished = evaluate_state_slice(
                    base,
                    enumerated_writes,
                    plan,
                    slice_index,
                    slices[slice_index].clone(),
                    judged_root_index,
                    versions,
                    retention,
                );
                // 发不出去说明调用线程已经不收了（观察者 panic）：不再领新的片。
                if finished_slice_sender.send(finished).is_err() {
                    break;
                }
            });
        }
        // 只留工作线程手里的发送端：它们都退出之后，下面的接收循环才结束。
        drop(finished_slice_sender);
        let mut merged = Layer0Tally::default();
        let mut waiting_for_earlier_slices: BTreeMap<usize, SliceWaitingToBeMerged> =
            restored_slices
                .into_iter()
                .map(|(slice_index, tally)| {
                    (
                        slice_index,
                        SliceWaitingToBeMerged::RestoredFromTheProgressFile(tally),
                    )
                })
                .collect();
        // 这一趟负责的切片里下一个该并的是第几个（不分片时就是片号）。
        let mut next_position_in_slices_of_this_run = 0usize;
        let mut merged_fresh_slice_count = 0usize;
        // 分片跑一片时每片的计数另留一份，全部并完之后写进账本；不分片时不留。
        let mut slice_tallies_for_the_ledger: BTreeMap<usize, Layer0Tally> = BTreeMap::new();
        let mut keep_for_the_ledger = |slice_index: usize, slice_tally: &Layer0Tally| {
            if shard.is_some() {
                slice_tallies_for_the_ledger.insert(slice_index, slice_tally.clone());
            }
        };
        // 收下一片（`None` 是开跑前只并读回的片），再按片号次序并：读回的片直接并；这一趟跑的片先给观察者看、
        // 把它的计数记进这一片，再写进进度文件，再并。
        let mut take_and_merge_the_slices_now_in_order =
            |arrived: Option<(usize, SliceWaitingToBeMerged)>| {
                if let Some((slice_index, slice_waiting)) = arrived {
                    waiting_for_earlier_slices.insert(slice_index, slice_waiting);
                }
                while let Some((slice_index, in_order)) = slices_of_this_run
                    .get(next_position_in_slices_of_this_run)
                    .and_then(|slice_index| {
                        waiting_for_earlier_slices
                            .remove(slice_index)
                            .map(|in_order| (*slice_index, in_order))
                    })
                {
                    match in_order {
                        SliceWaitingToBeMerged::RestoredFromTheProgressFile(restored_tally) => {
                            keep_for_the_ledger(slice_index, &restored_tally);
                            findings_reporter.report_before_absorbing(
                                &merged.findings,
                                &restored_tally.findings,
                            );
                            merged.absorb_following_slice(restored_tally);
                        }
                        SliceWaitingToBeMerged::FreshlyRun(finished) => {
                            let mut slice_tally = finished.tally;
                            if let Some(observe) = observe_state.as_mut() {
                                for (persisted, consulted_report) in finished.observed_states {
                                    observe(
                                        &CrashImage {
                                            base,
                                            writes: enumerated_writes,
                                            persisted,
                                        },
                                        &consulted_report,
                                        &mut slice_tally.observer_counts,
                                    );
                                    slice_tally.observed_states += 1;
                                }
                            }
                            if let Some(file) = progress_file.as_mut() {
                                file.append_finished_slice(
                                    finished.slice_index,
                                    &slices[finished.slice_index],
                                    &slice_tally,
                                );
                            }
                            keep_for_the_ledger(slice_index, &slice_tally);
                            findings_reporter
                                .report_before_absorbing(&merged.findings, &slice_tally.findings);
                            merged.absorb_following_slice(slice_tally);
                            merged_fresh_slice_count += 1;
                        }
                    }
                    next_position_in_slices_of_this_run += 1;
                }
            };
        take_and_merge_the_slices_now_in_order(None);
        let mut finished_states = 0u64;
        for (finished_slice_count, finished) in finished_slice_receiver.iter().enumerate() {
            let slice = &slices[finished.slice_index];
            finished_states += slice.end - slice.start;
            println!(
                "LAYER0_PROGRESS slice={}/{} states=[{},{}) segments={}..={} finished_slices={}/{} finished_states={finished_states}/{} resumed_slices={restored_slice_count} elapsed_seconds={:.1}",
                finished.slice_index + 1,
                slices.len(),
                slice.start,
                slice.end,
                plan.segment_label(plan.segment_of_state(slice.start)),
                plan.segment_label(plan.segment_of_state(slice.end - 1)),
                finished_slice_count + 1,
                slices_to_run.len(),
                states_of_this_run,
                started.elapsed().as_secs_f64()
            );
            take_and_merge_the_slices_now_in_order(Some((
                finished.slice_index,
                SliceWaitingToBeMerged::FreshlyRun(finished),
            )));
        }
        MergedSlicesOfThisRun {
            tally: merged,
            merged_slice_count: next_position_in_slices_of_this_run,
            merged_fresh_slice_count,
            slice_tallies_for_the_ledger,
        }
    });
    let MergedSlicesOfThisRun {
        tally,
        merged_slice_count,
        merged_fresh_slice_count,
        slice_tallies_for_the_ledger,
    } = merged_slices_of_this_run;
    assert_eq!(
        merged_slice_count,
        slices_of_this_run.len(),
        "每一片都按次序并进来了：少了说明有工作线程领了片却没交回"
    );
    assert_eq!(
        restored_slice_count + merged_fresh_slice_count,
        slices_of_this_run.len(),
        "读回的片 + 这一趟跑的片 = 这一趟负责的片数（层 0 规模第二轮判决 R3；分片时是归这一片的片数）"
    );
    if has_observer {
        assert_eq!(
            tally.observed_states, states_of_this_run,
            "观察者看过的状态数（连续跑接上的片记的）等于这一趟负责的状态数（层 0 规模第三轮判决 U3）"
        );
    }
    let ledger_written = shard_run.map(|run| {
        let whole_plan = Layer0ProgressPlan {
            plan_hash: layer0_plan_hash(
                base,
                enumerated_writes,
                &plan,
                judged_root_index,
                versions,
                &slices,
                has_observer,
                None,
            ),
            state_count: plan.state_count,
            slices: slices.clone(),
            has_observer,
            shard: None,
        };
        // 账本写在删进度文件之前：两步之间被杀，下一趟从进度文件读回全部片、重写同一份账本。
        let ledger_path = write_shard_ledger(
            run,
            &whole_plan,
            &Layer0ShardWorkerThreads {
                spawned_worker_threads,
                configured_worker_threads: parallelism.worker_threads.get(),
                worker_threads_source: parallelism.worker_threads_source.name().to_string(),
                available_parallelism: std::thread::available_parallelism()
                    .map_or(0, NonZeroUsize::get),
                resumed_slices: restored_slice_count,
                freshly_run_slices: merged_fresh_slice_count,
                elapsed_milliseconds: u64::try_from(started.elapsed().as_millis())
                    .unwrap_or(u64::MAX),
            },
            &slice_tallies_for_the_ledger,
        );
        (run.shard, ledger_path)
    });
    findings_reporter.finish(&tally.findings, states_of_this_run, &shard_fields);
    let progress_file_after_completion = progress_file.map(Layer0ProgressFile::finish);
    println!(
        "LAYER0_PARALLEL_FINISHED states={states_of_this_run} slices={} worker_threads={spawned_worker_threads} configured_worker_threads={} worker_threads_source={} resumed_slices={restored_slice_count} freshly_run_slices={merged_fresh_slice_count} progress_file_after_completion={} elapsed_seconds={:.1}{shard_fields}",
        slices_of_this_run.len(),
        parallelism.worker_threads,
        parallelism.worker_threads_source.name(),
        match progress_file_after_completion {
            None => "none",
            Some(Layer0ProgressFileAfterCompletion::Deleted) => "deleted",
            Some(Layer0ProgressFileAfterCompletion::KeptForTheTestThatInspectsIt) => "kept",
        },
        started.elapsed().as_secs_f64()
    );
    match ledger_written {
        Some((written_shard, ledger_path)) => {
            Layer0EnumerationOutcome::OneShardWrittenToItsLedger(Layer0ShardLedgerWritten {
                shard: written_shard,
                ledger_path,
                tally_of_this_shard: tally,
            })
        }
        None => Layer0EnumerationOutcome::WholeStream(tally),
    }
}

/// 调用线程并完这一趟负责的切片之后交回的东西。
struct MergedSlicesOfThisRun {
    tally: Layer0Tally,
    /// 按次序并进来的片数（读回的 + 这一趟跑的）。
    merged_slice_count: usize,
    merged_fresh_slice_count: usize,
    /// 分片跑一片时每片的计数（按片号）；不分片时是空的。
    slice_tallies_for_the_ledger: BTreeMap<usize, Layer0Tally>,
}

/// 逐片报的一样数：`shard_<名>=a,b,…`，按第几片排。
fn per_shard_field(
    name: &str,
    worker_threads_by_shard: &[Layer0ShardWorkerThreads],
    value_of: impl Fn(&Layer0ShardWorkerThreads) -> String,
) -> String {
    let values: Vec<String> = worker_threads_by_shard.iter().map(value_of).collect();
    format!("shard_{name}={}", values.join(","))
}

/// merge：读 n 份账本、核齐，按切片序号从小到大并（「第一处」取序号最小的，与单机并片同一个次序）。打一行 `LAYER0_SHARD mode=merge`
/// 与两行 `LAYER0_PARALLEL_*`：工作线程、读回与跑过的片是 n 片之和（读回 + 跑过 = 总片数，与单机的判法对得上），另逐片报出来。
/// 发现表照单机并片报（`LAYER0_FINDING` / `LAYER0_FINDINGS`），开了发现日志的写一节定稿：与单机跑同一条流的那一节逐字节相同。
///
/// # Panics
/// 账本核不齐（缺哪一片、哪一片的哪一处不同）；并完之后状态数、观察者看过的状态数对不上（不变量被破坏）；发现日志写失败。
fn merge_the_shard_ledgers(
    whole_plan: &Layer0ProgressPlan,
    merge: &Layer0ShardMerge,
    findings_log: &Layer0FindingsLog,
) -> Layer0Tally {
    let started = Instant::now();
    let ledgers = read_shard_ledgers_for_merge(merge, whole_plan)
        .unwrap_or_else(|problem| panic!("merge {} 片的账本：{problem}", merge.shard_count));
    println!(
        "LAYER0_SHARD mode=merge shards={} ledgers={}",
        merge.shard_count,
        ledgers
            .ledger_paths
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<String>>()
            .join(",")
    );
    let by_shard = &ledgers.worker_threads_by_shard;
    let sum_of = |value_of: fn(&Layer0ShardWorkerThreads) -> usize| -> usize {
        by_shard.iter().map(value_of).sum()
    };
    let spawned_worker_threads = sum_of(|threads| threads.spawned_worker_threads);
    let configured_worker_threads = sum_of(|threads| threads.configured_worker_threads);
    let resumed_slices = sum_of(|threads| threads.resumed_slices);
    let freshly_run_slices = sum_of(|threads| threads.freshly_run_slices);
    let shard_fields = [
        format!("shards={}", merge.shard_count),
        per_shard_field("worker_threads", by_shard, |threads| {
            threads.spawned_worker_threads.to_string()
        }),
        per_shard_field("configured_worker_threads", by_shard, |threads| {
            threads.configured_worker_threads.to_string()
        }),
        per_shard_field("worker_threads_sources", by_shard, |threads| {
            threads.worker_threads_source.clone()
        }),
        per_shard_field("available_parallelism", by_shard, |threads| {
            threads.available_parallelism.to_string()
        }),
        per_shard_field("resumed_slices", by_shard, |threads| {
            threads.resumed_slices.to_string()
        }),
        per_shard_field("freshly_run_slices", by_shard, |threads| {
            threads.freshly_run_slices.to_string()
        }),
        per_shard_field("elapsed_milliseconds", by_shard, |threads| {
            threads.elapsed_milliseconds.to_string()
        }),
    ]
    .join(" ");
    println!(
        "LAYER0_PARALLEL_START states={} slices={} states_per_slice={} worker_threads={spawned_worker_threads} configured_worker_threads={configured_worker_threads} worker_threads_source=shard_ledgers resumed_slices={resumed_slices} freshly_run_slices={freshly_run_slices} {shard_fields}",
        whole_plan.state_count,
        whole_plan.slices.len(),
        whole_plan.slices.first().map_or(0, |slice| slice.end - slice.start),
    );
    let mut findings_reporter = Layer0FindingsReporter::begin(
        findings_log,
        Layer0FindingsLogBegin {
            stream_name: Some(merge.stream_name.as_str().to_string()),
            whole_stream_states: whole_plan.state_count,
            shard: None,
        },
    );
    let mut tally = Layer0Tally::default();
    for (_slice_index, slice_tally) in ledgers.slice_tallies {
        findings_reporter.report_before_absorbing(&tally.findings, &slice_tally.findings);
        tally.absorb_following_slice(slice_tally);
    }
    assert_eq!(
        tally.states, whole_plan.state_count,
        "n 份账本并起来的状态数等于整条流的状态数"
    );
    if whole_plan.has_observer {
        assert_eq!(
            tally.observed_states, whole_plan.state_count,
            "n 片的观察者合起来看过每一个状态（层 0 规模第三轮判决 U3）"
        );
    }
    findings_reporter.finish(&tally.findings, whole_plan.state_count, "");
    println!(
        "LAYER0_PARALLEL_FINISHED states={} slices={} worker_threads={spawned_worker_threads} configured_worker_threads={configured_worker_threads} worker_threads_source=shard_ledgers resumed_slices={resumed_slices} freshly_run_slices={freshly_run_slices} progress_file_after_completion=none elapsed_seconds={:.1} {shard_fields}",
        whole_plan.state_count,
        whole_plan.slices.len(),
        started.elapsed().as_secs_f64()
    );
    tally
}

/// 全量、多版本：每一段都展开。
#[must_use]
pub fn enumerate_layer0_versions(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: &[PublishedVersion],
) -> Layer0Tally {
    enumerate_layer0_selecting_versions(
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        &|_segment_index, _segment| true,
    )
}

/// 全量：每一段都展开。
#[must_use]
pub fn enumerate_layer0(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    root_index: usize,
    expected_content: &[u8],
) -> Layer0Tally {
    enumerate_layer0_selecting(
        base,
        writes,
        segments,
        root_index,
        expected_content,
        &|_segment_index, _segment| true,
    )
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

    /// 单线程逐段逐个子集走（并行之前 `enumerate_layer0_selecting_versions_observing_each_state` 的次序）给出的持久集合，按次序排好。
    fn persisted_sets_walking_segment_by_segment(
        segments: &[Vec<usize>],
        write_count: usize,
        expand: &dyn Fn(usize, &[usize]) -> bool,
    ) -> Vec<Vec<bool>> {
        let mut persisted_sets = Vec::new();
        let mut persisted_before = vec![false; write_count];
        for (segment_index, segment) in segments.iter().enumerate() {
            if expand(segment_index, segment) {
                for subset_mask in 0..(1u64 << segment.len()) - 1 {
                    let mut persisted = persisted_before.clone();
                    for (bit, write_index) in segment.iter().enumerate() {
                        if subset_mask & (1 << bit) != 0 {
                            persisted[*write_index] = true;
                        }
                    }
                    persisted_sets.push(persisted);
                }
            }
            for write_index in segment {
                persisted_before[*write_index] = true;
            }
        }
        persisted_sets.push(persisted_before);
        persisted_sets
    }

    /// 按序号取状态（并行切片靠它）与逐段逐个子集走，给出同一串持久集合：展开的段夹着不展开的段、不展开的段在头上和尾上都算。
    #[test]
    fn the_state_plan_hands_out_the_same_persisted_sets_in_the_same_order_as_walking_segment_by_segment(
    ) {
        let segments = vec![
            vec![0, 1],
            vec![2],
            vec![3, 4, 5],
            vec![6, 7],
            vec![8, 9, 10, 11],
            vec![12],
        ];
        let write_count = 13;
        let every_segment = |_segment_index: usize, _segment: &[usize]| true;
        let skip_three_write_segment_and_the_ends = |segment_index: usize, segment: &[usize]| {
            segment.len() != 3 && segment_index != 0 && segment_index != 5
        };
        let only_the_four_write_segment =
            |_segment_index: usize, segment: &[usize]| segment.len() == 4;
        // 这条只核序号与持久集合的对应，写表里没有根槽写：每一段都归「最后一次根槽写之后」。
        let writes = writes_of_kinds(&[StepKind::UnitWrite; 13]);
        let tearable = TearableInPlaceOverwrites::none(writes.len());
        let writes_with_torn_images =
            WritesWithTornImages::of(&MemoryPool::default(), &writes, &segments, &tearable);
        let assert_plan_matches_the_walk = |expand: &dyn Fn(usize, &[usize]) -> bool| {
            let walked = persisted_sets_walking_segment_by_segment(&segments, write_count, expand);
            let plan = Layer0StatePlan::new(
                &writes,
                &segments,
                &|segment_index, segment| {
                    expansion_of_the_yes_or_no_choice(expand(segment_index, segment))
                },
                &tearable,
                &writes_with_torn_images,
            );
            assert_eq!(
                plan.state_count,
                u64::try_from(walked.len()).expect("状态数"),
                "状态数与逐段走的相同"
            );
            let by_ordinal: Vec<Vec<bool>> = (0..plan.state_count)
                .map(|ordinal| plan.persisted_writes_of_state(ordinal))
                .collect();
            assert_eq!(by_ordinal, walked, "第 k 个状态就是逐段走到的第 k 个");
        };
        assert_plan_matches_the_walk(&every_segment);
        assert_plan_matches_the_walk(&skip_three_write_segment_and_the_ends);
        assert_plan_matches_the_walk(&only_the_four_write_segment);
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
        let plan = Layer0StatePlan::new(
            &writes,
            &segments,
            &quick_tier_expansion,
            &tearable,
            &writes_with_torn_images,
        );
        // [2 原地 + 3 单元] 7、[2 原地] 3、[1 原地] 1、[2 单元] 1、[1 原地 + 1 单元] 3，再加全部持久那一个。
        assert_eq!(plan.state_count, 7 + 3 + 1 + 1 + 3 + 1);
        assert_eq!(
            layer0_state_count(&writes, &segments, &quick_tier_expansion),
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
            layer0_state_count(&writes, &segments, &full_expansion),
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
        let plan = Layer0StatePlan::new(
            &writes,
            &segments,
            &full_expansion,
            &tearable,
            &writes_with_torn_images,
        );
        // 第一段 3 · 2 · 3 − 1、第二段 2 − 1、第三段 3 − 1，再加全部持久那一个。
        assert_eq!(plan.state_count, 17 + 1 + 2 + 1);
        assert_eq!(
            layer0_state_count_with_torn_in_place_overwrites(
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

    /// 切片首尾相接、从 0 起、到状态数止、一片都不空：丢一片或两片重叠，这里与用例里「状态数等于闭式」的断言都红。
    /// 默认切法下全量两条流切出来的片数不少于线程数（每个线程都领得到片）。
    #[test]
    fn state_slices_cover_every_state_exactly_once_in_ordinal_order() {
        let thread_counts = [1usize, 3, 32, 200];
        let fixed_lengths = [1u64, 7, 16, u64::MAX];
        for state_count in [1u64, 2, 15, 16, 17, 22, 108, 1000, 262_165, 2_104_413] {
            let mut parallelisms: Vec<Layer0Parallelism> = thread_counts
                .iter()
                .map(|worker_threads| Layer0Parallelism {
                    worker_threads: NonZeroUsize::new(*worker_threads).expect("非 0"),
                    worker_threads_source: Layer0WorkerThreadsSource::GivenByCaller,
                    slice_length: Layer0SliceLength::ScaledToWorkerThreads,
                })
                .collect();
            parallelisms.extend(
                fixed_lengths
                    .iter()
                    .map(|states_per_slice| Layer0Parallelism {
                        worker_threads: NonZeroUsize::MIN,
                        worker_threads_source: Layer0WorkerThreadsSource::GivenByCaller,
                        slice_length: Layer0SliceLength::StatesPerSlice(
                            NonZeroU64::new(*states_per_slice).expect("非 0"),
                        ),
                    }),
            );
            for parallelism in parallelisms {
                let slices = state_slices(state_count, &parallelism);
                let mut next_ordinal = 0u64;
                for slice in &slices {
                    assert_eq!(
                        slice.start, next_ordinal,
                        "{state_count} 个状态、{parallelism:?}：片首接上一片的尾"
                    );
                    assert!(slice.end > slice.start, "{parallelism:?}：没有空片");
                    next_ordinal = slice.end;
                }
                assert_eq!(
                    next_ordinal, state_count,
                    "{parallelism:?}：最后一片止于状态数"
                );
                if state_count >= 262_165 {
                    assert!(
                        slices.len() >= parallelism.worker_threads.get()
                            || matches!(
                                parallelism.slice_length,
                                Layer0SliceLength::StatesPerSlice(_)
                            ),
                        "{state_count} 个状态、{parallelism:?}：片数 {} 不少于线程数",
                        slices.len()
                    );
                }
            }
        }
    }

    /// 续跑的片方案不随线程数变（R2）：第二条流全量那么多个状态，1、8、32 个线程按「线程数定片长」给进来，续跑时切出来的片逐个相同、
    /// 片数不超过 [`LAYER0_RESUMABLE_SLICE_COUNT`]；不续跑时照旧按线程数切（1 个与 32 个线程切得不同）。
    #[test]
    fn a_resumable_run_slices_independently_of_the_worker_threads() {
        let state_count = 6_649_413_746u64;
        let resume =
            Layer0Resume::KeepProgressFile(crate::layer0_progress::Layer0ProgressFileSettings {
                directory: std::path::PathBuf::from("/nonexistent"),
                input_fingerprint: crate::layer0_progress::Layer0ProgressFileNamePart::new(
                    "fingerprint0",
                )
                .expect("合法"),
                stream_name: crate::layer0_progress::Layer0ProgressFileNamePart::new("stream_a")
                    .expect("合法"),
                start: crate::layer0_progress::Layer0ResumeStart::ResumeFromTheProgressFile,
                after_completion: Layer0ProgressFileAfterCompletion::Deleted,
            });
        let scaled_to = |worker_threads: usize| Layer0Parallelism {
            worker_threads: NonZeroUsize::new(worker_threads).expect("不是 0"),
            worker_threads_source: Layer0WorkerThreadsSource::GivenByCaller,
            slice_length: Layer0SliceLength::ScaledToWorkerThreads,
        };
        let resumable_slices = |worker_threads: usize| {
            state_slices(
                state_count,
                &slicing_of_the_run(&scaled_to(worker_threads), &resume, state_count),
            )
        };
        let on_one_thread = resumable_slices(1);
        assert_eq!(on_one_thread, resumable_slices(8));
        assert_eq!(on_one_thread, resumable_slices(32));
        assert!(
            u64::try_from(on_one_thread.len()).expect("片数") <= LAYER0_RESUMABLE_SLICE_COUNT,
            "片数 {}",
            on_one_thread.len()
        );
        let without_resume = |worker_threads: usize| {
            state_slices(
                state_count,
                &slicing_of_the_run(
                    &scaled_to(worker_threads),
                    &Layer0Resume::NoProgressFile,
                    state_count,
                ),
            )
            .len()
        };
        assert_ne!(without_resume(1), without_resume(32), "不续跑时按线程数切");
    }

    /// 双机分片的片方案不随线程数变：第二条流全量那么多个状态，分片跑一片与 merge 按「线程数定片长」给进来，1、8、32 个线程切出来的
    /// 片逐个相同，也与单机续跑的相同（两台线程数不同，账本的切法照样对得上）。
    #[test]
    fn a_shard_run_and_a_merge_slice_independently_of_the_worker_threads() {
        use crate::layer0_progress::{
            Layer0ProgressFileNamePart, Layer0ProgressFileSettings, Layer0ResumeStart,
            Layer0ShardMerge, Layer0ShardRun, Layer0ToolchainIdentity,
        };
        let state_count = 6_649_413_746u64;
        let progress = Layer0ProgressFileSettings {
            directory: std::path::PathBuf::from("/nonexistent"),
            input_fingerprint: Layer0ProgressFileNamePart::new("fingerprint0").expect("合法"),
            stream_name: Layer0ProgressFileNamePart::new("stream_a").expect("合法"),
            start: Layer0ResumeStart::ResumeFromTheProgressFile,
            after_completion: Layer0ProgressFileAfterCompletion::Deleted,
        };
        let toolchain = Layer0ToolchainIdentity {
            rustc_version_lines: "rustc".to_string(),
            cargo_version: "cargo".to_string(),
            target_triple: "target".to_string(),
        };
        let shard_count = std::num::NonZeroU32::new(2).expect("不是 0");
        let shard_run = Layer0Resume::RunOneShardKeepingProgressFile(Layer0ShardRun {
            progress: progress.clone(),
            shard: Layer0ShardOfShards::new(1, shard_count).expect("1 < 2"),
            toolchain: toolchain.clone(),
        });
        let merge = Layer0Resume::MergeShardLedgers(Layer0ShardMerge {
            directory: progress.directory.clone(),
            input_fingerprint: progress.input_fingerprint.clone(),
            stream_name: progress.stream_name.clone(),
            shard_count,
            toolchain,
        });
        let single_machine = Layer0Resume::KeepProgressFile(progress);
        let slices_on = |resume: &Layer0Resume, worker_threads: usize| {
            state_slices(
                state_count,
                &slicing_of_the_run(
                    &Layer0Parallelism {
                        worker_threads: NonZeroUsize::new(worker_threads).expect("不是 0"),
                        worker_threads_source: Layer0WorkerThreadsSource::GivenByCaller,
                        slice_length: Layer0SliceLength::ScaledToWorkerThreads,
                    },
                    resume,
                    state_count,
                ),
            )
        };
        let single_machine_slices = slices_on(&single_machine, 1);
        for resume in [&shard_run, &merge] {
            for worker_threads in [1, 8, 32] {
                assert!(
                    slices_on(resume, worker_threads) == single_machine_slices,
                    "{resume:?} 在 {worker_threads} 个线程上的片方案与单机续跑的相同"
                );
            }
        }
    }

    /// 线程数：环境变量设了就用它（不再问 `available_parallelism`）；没设取 `available_parallelism`；那个也报不出来只用 1 个线程、照实报来源。
    #[test]
    fn worker_threads_come_from_the_environment_variable_before_available_parallelism() {
        let thirty_two = || Ok(NonZeroUsize::new(32).expect("非 0"));
        let from_variable = Layer0Parallelism::from_environment_value(Ok("4".to_string()), || {
            panic!("环境变量设了就不该再问 available_parallelism")
        });
        assert_eq!(
            (
                from_variable.worker_threads.get(),
                from_variable.worker_threads_source
            ),
            (4, Layer0WorkerThreadsSource::EnvironmentVariable)
        );
        let explicit_single =
            Layer0Parallelism::from_environment_value(Ok("1".to_string()), thirty_two);
        assert_eq!(
            (
                explicit_single.worker_threads.get(),
                explicit_single.worker_threads_source
            ),
            (1, Layer0WorkerThreadsSource::EnvironmentVariable),
            "显式设成 1 就是 1"
        );
        let unset = Layer0Parallelism::from_environment_value(
            Err(std::env::VarError::NotPresent),
            thirty_two,
        );
        assert_eq!(
            (unset.worker_threads.get(), unset.worker_threads_source),
            (32, Layer0WorkerThreadsSource::AvailableParallelism)
        );
        let unknown =
            Layer0Parallelism::from_environment_value(Err(std::env::VarError::NotPresent), || {
                Err(std::io::Error::other("平台报不出核数"))
            });
        assert_eq!(
            (unknown.worker_threads.get(), unknown.worker_threads_source),
            (1, Layer0WorkerThreadsSource::AvailableParallelismUnknown)
        );
        assert_eq!(unset.slice_length, Layer0SliceLength::ScaledToWorkerThreads);
    }

    /// 环境变量设成 0：停下，不悄悄退回单线程。不写成 `#[should_panic]`：那样 libtest 的输出行带「- should panic」，门禁 59 号认不出这条测试红没红。
    #[test]
    fn zero_worker_threads_in_the_environment_variable_stops_instead_of_falling_back() {
        let outcome = std::panic::catch_unwind(|| {
            Layer0Parallelism::from_environment_value(Ok("0".to_string()), || {
                Ok(NonZeroUsize::new(32).expect("非 0"))
            })
        });
        let panic_payload = outcome.expect_err("设成 0 要停下，不许退回 1 个线程接着跑");
        let panic_message = panic_payload
            .downcast_ref::<String>()
            .cloned()
            .unwrap_or_default();
        assert!(
            panic_message.contains("SINGLEFS_LAYER0_THREADS 要是正整数"),
            "停下时说清是哪个环境变量配错了：{panic_message}"
        );
    }

    /// 并片：计数相加，「第一处」取前面那一片的；前面那一片没有才取后面的。
    #[test]
    fn absorbing_a_following_slice_adds_counts_and_keeps_the_earlier_first_violation() {
        let first_publish = Layer0PublishOfState::UpToTheRootOf {
            root_write_index: 4,
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(1),
        };
        let mut earlier = Layer0Tally {
            states: 3,
            states_by_publish: BTreeMap::from([(first_publish, 3)]),
            violations: 1,
            root_persisted_states: 0,
            no_file_states: 3,
            file_read_states: 0,
            failed_states: 0,
            journal_differing_states: 0,
            verification_ran_states: 0,
            verification_failed_states: 0,
            first_violation: Some("前面那一片的".to_string()),
            ignored_violations: 0,
            first_ignored_violation: None,
            record_root_without_record: 0,
            record_claimed_state_missing_unit: 0,
            checker_evaluated_states: BTreeMap::from([("I-1.1", 3)]),
            checker_violated_states: BTreeMap::from([("I-1.1", 1)]),
            checker_first_violation: BTreeMap::from([("I-1.1", "前面那一片的 I-1.1".to_string())]),
            checker_not_applicable_states: BTreeMap::from([("I-3.1", 3)]),
            observed_states: 3,
            observer_counts: observer_counts_of(&[("read_through_the_record", 2)]),
            findings: Layer0Findings::default(),
        };
        let following = Layer0Tally {
            states: 5,
            states_by_publish: BTreeMap::from([
                (first_publish, 2),
                (Layer0PublishOfState::AfterTheLastRoot, 2),
                (Layer0PublishOfState::EveryWritePersisted, 1),
            ]),
            violations: 2,
            root_persisted_states: 5,
            no_file_states: 0,
            file_read_states: 5,
            failed_states: 0,
            journal_differing_states: 1,
            verification_ran_states: 1,
            verification_failed_states: 0,
            first_violation: Some("后面那一片的".to_string()),
            ignored_violations: 1,
            first_ignored_violation: Some("后面那一片的 Ignore".to_string()),
            record_root_without_record: 1,
            record_claimed_state_missing_unit: 1,
            checker_evaluated_states: BTreeMap::from([("I-1.1", 5), ("I-3.1", 5)]),
            checker_violated_states: BTreeMap::from([("I-1.1", 2), ("I-3.1", 1)]),
            checker_first_violation: BTreeMap::from([
                ("I-1.1", "后面那一片的 I-1.1".to_string()),
                ("I-3.1", "后面那一片的 I-3.1".to_string()),
            ]),
            checker_not_applicable_states: BTreeMap::new(),
            observed_states: 5,
            observer_counts: observer_counts_of(&[
                ("read_through_the_record", 1),
                ("read_through_the_root", 4),
            ]),
            findings: Layer0Findings::default(),
        };
        earlier.absorb_following_slice(following);
        assert_eq!(
            (earlier.observed_states, earlier.observer_counts.clone()),
            (
                8,
                observer_counts_of(&[("read_through_the_record", 3), ("read_through_the_root", 4)])
            ),
            "观察者看过的状态数与按名字的计数逐项相加（续跑接上的片靠它们接累计）"
        );
        assert_eq!(
            (
                earlier.states,
                earlier.violations,
                earlier.ignored_violations,
                earlier.no_file_states,
                earlier.file_read_states,
                earlier.root_persisted_states,
                earlier.journal_differing_states,
                earlier.verification_ran_states,
                earlier.record_root_without_record,
                earlier.record_claimed_state_missing_unit,
            ),
            (8, 3, 1, 3, 5, 5, 1, 1, 1, 1),
            "计数逐项相加"
        );
        assert_eq!(
            earlier.states_by_publish,
            BTreeMap::from([
                (first_publish, 5),
                (Layer0PublishOfState::AfterTheLastRoot, 2),
                (Layer0PublishOfState::EveryWritePersisted, 1),
            ]),
            "按发布分的状态数逐格相加"
        );
        assert_eq!(earlier.first_violation.as_deref(), Some("前面那一片的"));
        assert_eq!(
            earlier.first_ignored_violation.as_deref(),
            Some("后面那一片的 Ignore"),
            "前面那一片没有，取后面的"
        );
        assert_eq!(
            earlier.checker_evaluated_states,
            BTreeMap::from([("I-1.1", 8), ("I-3.1", 5)])
        );
        assert_eq!(
            earlier.checker_violated_states,
            BTreeMap::from([("I-1.1", 3), ("I-3.1", 1)])
        );
        assert_eq!(
            earlier.checker_not_applicable_states,
            BTreeMap::from([("I-3.1", 3)])
        );
        assert_eq!(
            earlier.checker_first_violation,
            BTreeMap::from([
                ("I-1.1", "前面那一片的 I-1.1".to_string()),
                ("I-3.1", "后面那一片的 I-3.1".to_string()),
            ]),
            "每条不变量的第一处各取最早有的那一片"
        );
    }

    /// 一片只在「不看 journal 那一遍」上有违例的计数：其余计数与第一处都空着，并片时只看这两项。
    fn slice_with_only_ignored_violations(
        states: u64,
        ignored_violations: u64,
        first_ignored_violation: &str,
    ) -> Layer0Tally {
        Layer0Tally {
            states,
            states_by_publish: BTreeMap::new(),
            violations: 0,
            root_persisted_states: 0,
            no_file_states: states,
            file_read_states: 0,
            failed_states: 0,
            journal_differing_states: 0,
            verification_ran_states: 0,
            verification_failed_states: 0,
            first_violation: None,
            ignored_violations,
            first_ignored_violation: Some(first_ignored_violation.to_string()),
            record_root_without_record: 0,
            record_claimed_state_missing_unit: 0,
            checker_evaluated_states: BTreeMap::new(),
            checker_violated_states: BTreeMap::new(),
            checker_first_violation: BTreeMap::new(),
            checker_not_applicable_states: BTreeMap::new(),
            observed_states: 0,
            observer_counts: Layer0ObserverCounts::default(),
            findings: Layer0Findings::default(),
        }
    }

    fn observer_counts_of(entries: &[(&str, u64)]) -> Layer0ObserverCounts {
        let mut counts = Layer0ObserverCounts::default();
        for (name, amount) in entries {
            counts.add(name, *amount);
        }
        counts
    }

    /// 并片：两片都带「不看 journal 那一遍的第一处违例」时取前面那一片的，再并第三片也不换（代码三方第一轮 Y5m2：
    /// 上一条用例里前面那一片这一项是空的，改成「后面那一片有就取后面的」照样绿）。
    #[test]
    fn absorbing_a_following_slice_keeps_the_earlier_first_ignored_violation_when_both_slices_have_one(
    ) {
        let mut merged = slice_with_only_ignored_violations(4, 1, "第一片的 Ignore");
        merged.absorb_following_slice(slice_with_only_ignored_violations(4, 2, "第二片的 Ignore"));
        assert_eq!(
            (merged.states, merged.ignored_violations),
            (8, 3),
            "计数逐项相加"
        );
        assert_eq!(
            merged.first_ignored_violation.as_deref(),
            Some("第一片的 Ignore"),
            "两片都有时取前面那一片的"
        );
        merged.absorb_following_slice(slice_with_only_ignored_violations(2, 1, "第三片的 Ignore"));
        assert_eq!(
            merged.first_ignored_violation.as_deref(),
            Some("第一片的 Ignore"),
            "再并一片也不换"
        );
        assert_eq!(merged.first_violation, None, "看 journal 那一遍没有违例");
    }

    /// 并片：「走读失败」与「journal 验证失败」两项也逐项相加（代码审阅第 14 条：上面那条用例里两片这两项都是 0，
    /// 删掉相加照样绿，层 0 用例 `failed_states == 0` 的断言对第二片以后的失败状态就没了判别力）。两片各取不同的非 0 值，
    /// 相加的和与任一片单独的值都不同。
    #[test]
    fn absorbing_a_following_slice_adds_the_failed_and_the_verification_failed_states_of_both_slices(
    ) {
        let mut earlier = slice_with_only_ignored_violations(6, 0, "前面那一片的 Ignore");
        earlier.failed_states = 2;
        earlier.verification_ran_states = 4;
        earlier.verification_failed_states = 3;
        let mut following = slice_with_only_ignored_violations(9, 0, "后面那一片的 Ignore");
        following.failed_states = 5;
        following.verification_ran_states = 8;
        following.verification_failed_states = 7;
        earlier.absorb_following_slice(following);
        assert_eq!(
            (
                earlier.states,
                earlier.failed_states,
                earlier.verification_ran_states,
                earlier.verification_failed_states
            ),
            (15, 7, 12, 10),
            "走读失败 2 + 5、验证跑过 4 + 8、验证失败 3 + 7：两片逐项相加"
        );
    }

    fn finding_signature(red_pass: Layer0RedPass, segment_index: usize) -> Layer0FindingSignature {
        Layer0FindingSignature {
            red_pass,
            segment: Layer0SegmentOfState::Segment(segment_index),
            publish: Layer0PublishOfState::AfterTheLastRoot,
        }
    }

    /// 发现表：每项是（签名、状态数、样本序号），样本原文写成「状态 <序号>」。
    fn findings_with(
        red_states: u64,
        entries: &[(&Layer0FindingSignature, u64, &[u64])],
    ) -> Layer0Findings {
        Layer0Findings {
            red_states,
            by_signature: entries
                .iter()
                .map(|(signature, states, sample_ordinals)| {
                    (
                        (*signature).clone(),
                        Layer0Finding {
                            states: *states,
                            earliest_samples: sample_ordinals
                                .iter()
                                .map(|ordinal| Layer0FindingSample {
                                    state_ordinal: *ordinal,
                                    violation: format!("状态 {ordinal}"),
                                })
                                .collect(),
                        },
                    )
                })
                .collect(),
        }
    }

    /// 发现表并片：同一签名状态数相加、样本接在后面只留最先 3 个；只在一片里有的签名原样并进来；判红的状态数相加。
    #[test]
    fn absorbing_a_following_slice_adds_the_states_of_each_signature_and_keeps_its_earliest_three_samples(
    ) {
        let wrong_content = finding_signature(
            Layer0RedPass::JournalConsultedOracle(Layer0OracleViolationKind::WrongContent),
            2,
        );
        let pool_checker = finding_signature(Layer0RedPass::PoolChecker(vec!["I-3.1"]), 5);
        let record_checker = finding_signature(
            Layer0RedPass::RecordChecker(RecordCheck {
                root_without_record: true,
                claimed_state_missing_unit: false,
            }),
            5,
        );
        let mut earlier =
            findings_with(3, &[(&wrong_content, 2, &[1, 4]), (&pool_checker, 1, &[7])]);
        earlier.absorb_following_slice(findings_with(
            6,
            &[
                (&wrong_content, 4, &[10, 11, 12]),
                (&record_checker, 2, &[13, 14]),
            ],
        ));
        assert_eq!(
            earlier,
            findings_with(
                9,
                &[
                    (&wrong_content, 6, &[1, 4, 10]),
                    (&pool_checker, 1, &[7]),
                    (&record_checker, 2, &[13, 14]),
                ]
            ),
            "状态数 2 + 4、样本 1、4 之后只接上 10；另两个签名原样；判红的状态数 3 + 6"
        );
    }

    /// 并进一片之前报的事：这一片里第一次出现的签名按最先那个状态排在前面（不按签名的大小次序），再是跨过的台阶（一下跨两级报两件）；
    /// 更早的片里有过的签名不再报「新」，早就跨过的台阶不再报。
    #[test]
    fn absorbing_a_slice_reports_new_signatures_by_their_first_state_and_every_threshold_crossed() {
        let wrong_content = finding_signature(
            Layer0RedPass::JournalConsultedOracle(Layer0OracleViolationKind::WrongContent),
            2,
        );
        let no_root_chosen = finding_signature(
            Layer0RedPass::JournalIgnoredOracle(Layer0OracleViolationKind::NoRootChosen),
            0,
        );
        let pool_checker = finding_signature(Layer0RedPass::PoolChecker(vec!["I-3.1"]), 1);
        let record_checker = finding_signature(
            Layer0RedPass::RecordChecker(RecordCheck {
                root_without_record: false,
                claimed_state_missing_unit: true,
            }),
            1,
        );
        let merged = findings_with(
            21,
            &[
                (&wrong_content, 9, &[1, 2, 3]),
                (&no_root_chosen, 12, &[4, 5, 6]),
            ],
        );
        let following = findings_with(
            118,
            &[
                (&wrong_content, 1, &[20]),
                (&no_root_chosen, 5, &[30, 31, 32]),
                (&pool_checker, 110, &[40, 41, 42]),
                (&record_checker, 2, &[25, 26]),
            ],
        );
        let reported: Vec<String> = merged
            .events_of_absorbing(&following)
            .iter()
            .map(|event| match event {
                Layer0FindingEvent::NewSignature {
                    signature,
                    first_sample,
                } => format!(
                    "new {} {}",
                    signature.red_pass.pass_name(),
                    first_sample.state_ordinal
                ),
                Layer0FindingEvent::ThresholdCrossed {
                    signature,
                    states_at_least,
                } => format!(
                    "threshold {} {states_at_least}",
                    signature.red_pass.pass_name()
                ),
            })
            .collect();
        assert_eq!(
            reported,
            vec![
                "new record_checker 25",
                "new pool_checker 40",
                "threshold journal_consulted_oracle 10",
                "threshold pool_checker 10",
                "threshold pool_checker 100",
            ],
            "记录核对器最先在 25、池级 checker 在 40（签名次序反过来）；看 journal 那一遍 9 → 10 跨一级；池级 checker 0 → 110 跨两级；\
             不看 journal 那一遍 12 → 17 早就跨过 10、不报"
        );
    }
}

#[cfg(test)]
mod oracle_instance_tests {
    use super::{oracle_violation_for_versions, PublishedVersion};
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
