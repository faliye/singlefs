//! 还没有文件的那一版上的发布：暖机、不带单元的发布、只写实例表的发布与行发布。

use super::*;

/// 暖机写出的东西：两代根、两条空记录、最后一条记录的字节（下一条的反向链要罩它）、每次空发布按结构种类的写。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WarmUpOutput {
    pub roots: Vec<RootRecord>,
    pub records: Vec<JournalRecord>,
    pub last_record_bytes: Vec<u8>,
    /// 与 `roots` 同序，一次空发布一份。
    pub writes: Vec<WritesByStructureKind>,
}

/// 暖机那两次空发布里有一次没做成：零单元发布报的错（块设备的错；写入口参数里的环在飞上限为 0 时的记录条数上限），
/// 连同失败之前已经落盘的那几次各自的写（按先后）。
/// 失败那一次落盘阶段已记的写照旧进调用方那个写入口的失败账（`PoolWriter::writes_of_failed_publishes`）；已经落盘的那几次的账
/// 只在这里——它们的输出随错一起丢掉，不交出来，这段暖机里设备一层数到的写与程序交得出的账就对不上（增补 2 收口表第 58 行）。
#[derive(Debug)]
pub struct WarmUpFailed {
    pub cause: PublishError,
    pub writes_of_persisted_publishes: Vec<WritesByStructureKind>,
}

/// mkfs 之后环里一条记录都没有：接着它数的 jsn 从 1 起（D23（journal 的角色与格式） 已定项 14 第 3 条，与可写挂载「环里没有记录时从 1 起」同一个起点）。
const LAST_JOURNAL_COUNTER_AFTER_MAKE_FILESYSTEM: u64 = 0;

/// 暖机（D16（发布语义） 已定项 8）：mkfs 之后第一次可写挂载的两次空发布，checkpoint_txg 走 1、2；环在 mkfs 之后是空的，
/// jsn 从 1 起。其余见 `warm_up_after_journal_counter`。
///
/// # Errors
/// 零单元发布报的错（[`publish_without_units`]），连同已经落盘的那几次的账（[`WarmUpFailed`]）。
pub fn warm_up<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    genesis: &RootRecord,
    instance: InstanceGeneration,
) -> Result<WarmUpOutput, WarmUpFailed> {
    warm_up_after_journal_counter(
        pool,
        genesis,
        instance,
        LAST_JOURNAL_COUNTER_AFTER_MAKE_FILESYSTEM,
    )
}

/// 暖机的两次空发布，接在环里 jsn 计数器为 `last_journal_counter` 的那条记录之后：checkpoint_txg 照格式常量走 1、2
/// （`WARM_UP_EMPTY_PUBLISHES`，D16（发布语义） 已定项 8），jsn 与系统配置里的 tail 按记录号接着数、不取 txg
/// （D23（journal 的角色与格式） 已定项 18：tail 存 jsn 的 48 位计数器；已定项 14 第 3 条：计数器全池接着走；C366（暖机路径把 txg 写进计数器与 tail））。
/// 空记录不点名任何单元、事务号 0、提交标记 1、新根段照 mkfs 的根（D16（发布语义） 已定项 9 / D23（journal 的角色与格式） 已定项 19），
/// 根记录只改 checkpoint_txg 与实例代号。今天只有 `warm_up` 调它，环是空的、两个量按构造相等；给不相等的起点才分得出它们。
///
/// # Errors
/// 零单元发布报的错（[`publish_without_units`]），连同已经落盘的那几次的账（[`WarmUpFailed`]）。
pub fn warm_up_after_journal_counter<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    genesis: &RootRecord,
    instance: InstanceGeneration,
    last_journal_counter: u64,
) -> Result<WarmUpOutput, WarmUpFailed> {
    let mut roots = Vec::new();
    let mut records = Vec::new();
    let mut writes = Vec::new();
    let mut previous_record_bytes: Option<Vec<u8>> = None;
    let mut previous_counter = last_journal_counter;
    for txg_number in 1..=WARM_UP_EMPTY_PUBLISHES {
        let output = publish_without_units(
            pool,
            genesis,
            ZeroUnitPublishPlan {
                txg: CheckpointTxg(txg_number),
                counter: previous_counter + 1,
                instance,
                back_chain: previous_record_bytes.as_deref().map_or(0, back_chain_of),
                rollback_floor: genesis.rollback_floor,
                // mkfs 同一个进程里那条流：mkfs 之后根环里只有 mkfs 的第 0 代根，暖机根照抄它（D16（发布语义） 已定项 9）。
                tree_identifier_watermark: genesis.tree_identifier_watermark,
            },
        )
        .map_err(|cause| WarmUpFailed {
            cause,
            writes_of_persisted_publishes: writes.clone(),
        })?;
        previous_counter = output.record.counter;
        roots.push(output.root);
        records.push(output.record);
        writes.push(output.writes);
        previous_record_bytes = Some(output.record_bytes);
    }
    Ok(WarmUpOutput {
        roots,
        records,
        last_record_bytes: previous_record_bytes.expect("暖机至少一次"),
        writes,
    })
}

/// 一次零单元发布的参数（D16（发布语义） 已定项 9：树表 0 条时空发布写零个单元）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZeroUnitPublishPlan {
    pub txg: CheckpointTxg,
    /// jsn 计数器，全池接着走（D23（journal 的角色与格式） 已定项 14 第 3 条）。
    pub counter: u64,
    pub instance: InstanceGeneration,
    /// 反向链：本实例的第一条恒 0（D23（journal 的角色与格式） 已定项 19 ②）。
    pub back_chain: u32,
    pub rollback_floor: CheckpointTxg,
    /// 这次写进根记录与记录新根段的树 ID 水位：max(根环里全部根记录的该字段, 本次发出的最高树 ID + 1)
    /// （D8（核心索引结构） 已定项 8 ②），零单元发布一个号都不发 ⇒ 就是根环里的 max。本实例的第一次发布由挂载按环算
    /// （回退到的那一版的水位可以低于环里被抛弃的根带的）；之后接着本会话现行那一版的根照抄。
    pub tree_identifier_watermark: u64,
}

/// 树表 0 条的一版上一次发布写出的东西：根、记录、记录的字节（下一条的反向链要罩它）、按结构种类的写，
/// 以及这次写出的单元与这一版的分配记录（对拍拿它们比这一版的实例表与分配代，代码审阅第 12 条；字段名同带文件那一版的 [`TransactionOutput`]）。
/// 零单元发布（暖机）与写行那次发布（重写实例表链与分配记录树）都交回它；下一次发布只用根、记录与记录的字节
/// （这一版没有记账树与映射树；分配记录树住分配器上，`PoolAllocator::allocation_record_tree_of_the_version_without_file`）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionWithoutFilePublishOutput {
    pub root: RootRecord,
    /// 这次发布的末条记录（下一条的反向链要罩它的头）。
    pub record: JournalRecord,
    pub record_bytes: Vec<u8>,
    /// 末条之前的那几条：写行那次发布点名项多于一条记录装得下时末条再跨记录（D23（journal 的角色与格式） 已定项 17）；
    /// 零单元发布与只有一条记录的写行是空的。
    pub earlier_records_of_this_publish: Vec<WrittenJournalRecord>,
    /// 这次发布真正写出的角色，按写出的次序（= 点名项的次序：实例表链各片尾片先，再分配记录树重写的节点先叶后根）；零单元发布是空的。
    pub rewritten: Vec<TransactionUnit>,
    /// 这次发布写出的单元，与 `rewritten` 逐项同序、同角色（就是交给设备的那几个单元写）：写行那次是整条实例表链的各片
    /// （写行 COW 重写整条链，D18（块里携带什么信息） 已定项 11）加分配记录树这次重写的节点；零单元发布是空的。
    /// 与带文件那一版的 `units` 不同，没重写的单元不照抄进来：这一版没重写的只有 mkfs 的树表单元与分配记录树没变的节点。
    pub units: Vec<PublishedUnit>,
    /// 这一版的分配记录（[`AllocationRecordsOfTheVersionWithoutFile`]）。
    pub allocation_records: AllocationRecordsOfTheVersionWithoutFile,
    pub writes: WritesByStructureKind,
}

/// 树表 0 条的一版上一次发布交出的这一版分配记录。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AllocationRecordsOfTheVersionWithoutFile {
    /// 写行那次发布：这一版的全部分配记录，按 (设备, 槽号) 升序——就是这次装进分配记录树的那一份（释放、取落点都做完之后）。
    WrittenIntoTheAllocationRecordTreeByThisPublish(Vec<AllocationRecord>),
    /// 零单元发布不经分配器、一个字节都不写：这一版的分配记录就是上一版的（根记录里分配记录树根指针照抄），这次发布手里没有那一份。
    SameAsThePreviousVersionNotHandedInByAZeroUnitPublish,
}

/// 零单元发布（D16（发布语义） 已定项 9「树表 0 条 ⇒ 零单元」）：屏障 → 空记录 → 屏障 → 根槽 FUA → 系统配置槽轮换 → 屏障；
/// 空记录不点名任何单元、事务号 0、提交标记 1，新根段照上一版的根，根记录照上一版的根、只换 checkpoint_txg、实例代号、回退下界
/// 与树 ID 水位（取计划里给的，D8（核心索引结构） 已定项 8 ②）。
/// 第一次可写挂载的暖机与「只做过 mkfs 的池」上的可写挂载都走它（新池新建文件的字节不变）。
///
/// # Errors
/// 这次发布的记录条数（恒一条）多于 [`journal_record_limit_of_one_publish`]（在飞上限为 0 的环）⇒
/// `PublishError::JournalRecordsOfThePublishExceedTheLimit`，在任何写之前（代码审阅第 26 条，与带文件的一版、写行那一版同一个判定）；
/// 块设备报的错 ⇒ `PublishError::BlockDevice`。
pub fn publish_without_units<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    previous_root: &RootRecord,
    plan: ZeroUnitPublishPlan,
) -> Result<VersionWithoutFilePublishOutput, PublishError> {
    let (writes, output) = prepare_the_publish_without_units(pool, previous_root, plan);
    // 条数上限（代码审阅第 26 条，实审 A2c 报告「停下交主 agent 的」第 4 条）：可写挂载择系统配置时已拒在飞上限为 0 的环
    // （`recovery::system_configuration_values_this_reader_accepts`，实审 A3a 第 38 条），mkfs 也拒；走到这里的只有写入口的参数
    // 由调用方自己给、带着这种环的时候（`PoolWriter::new` 不判参数）。
    refuse_a_publish_of_more_journal_records_than_the_limit(
        writes.records.len(),
        pool.parameters.geometry.journal_ring_bytes,
    )?;
    let writes_before_this_publish = pool.writes_by_structure_kind.clone();
    // 中途失败时这次已记的写要交出去（增补 2 第 20b 行）：失败在这里记账再把错原样交回。零单元发布不冻结（见 `FrozenPublish`）。
    if let Err(cause) = persist_publish_writes(pool, &writes) {
        pool.count_failed_publish(&writes_before_this_publish);
        return Err(PublishError::BlockDevice(cause));
    }
    let VersionWithoutFilePublishOutput {
        root,
        record,
        record_bytes,
        earlier_records_of_this_publish,
        rewritten,
        units,
        allocation_records,
        writes: _nothing_written_before_persisting,
    } = output;
    Ok(VersionWithoutFilePublishOutput {
        root,
        record,
        record_bytes,
        earlier_records_of_this_publish,
        rewritten,
        units,
        allocation_records,
        writes: pool
            .writes_by_structure_kind
            .since(&writes_before_this_publish),
    })
}

/// 零单元发布落盘之前的全部：装好记录与根，交回要交给设备的字节与这一版（写账是空的）。发布路径（[`publish_without_units`]）
/// 与可写挂载取号之前的预演（`mount`）读的是同一份。
#[must_use]
pub fn prepare_the_publish_without_units<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
    previous_root: &RootRecord,
    plan: ZeroUnitPublishPlan,
) -> (PublishWrites, VersionWithoutFilePublishOutput) {
    let root = RootRecord {
        filesystem_identifier: previous_root.filesystem_identifier,
        // 零单元发布只有暖机与可写挂载取号之后那一串走：正常卸载在树表 0 条的一版上一个字节都不写
        // （`Unmounted::NothingWrittenOnAVersionWithoutFile`），走这里的根都不是卸载那一串写的。
        unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
        instance: plan.instance,
        checkpoint_txg: plan.txg,
        tree_table: previous_root.tree_table,
        tree_identifier_watermark: plan.tree_identifier_watermark,
        rollback_floor: plan.rollback_floor,
        instance_table: previous_root.instance_table,
        mapping_root: previous_root.mapping_root,
        // 零单元发布一个字节都不写：分配记录树照抄上一版的那一条指针，账也因此一条都没变。
        allocation_record_tree_root: previous_root.allocation_record_tree_root,
        // 零单元发布不建账、不经分配器记根（根落盘之后才记）：回收门槛 t 与钉槽 p 照抄上一版的（C773（读坏与全零根槽的回收门槛钉只在内存里），
        // 用户 2026-10-20 定；池级 checker 的 I-7.18、I-7.19 只判建账的发布，这一版下面没有记账树、报不适用）。
        reclaim_threshold: previous_root.reclaim_threshold,
        pinned_ring_slot: previous_root.pinned_ring_slot,
    };
    let record = JournalRecord {
        instance: plan.instance,
        counter: plan.counter,
        checkpoint_txg: plan.txg,
        transaction: 0,
        is_commit: true,
        // 空发布记录也写 1（D23（journal 的角色与格式） 已定项 4）：它一条就是一次发布。
        ordinal_within_publish: JournalRecordOrdinalWithinPublish::FIRST,
        // 它一条就是一次发布，也就是这次发布的末条（D23（journal 的角色与格式） 已定项 17：每次只有一条记录的发布，
        // 含空发布记录，那一条记录标志位 0 也写 1）。
        place_in_publish: JournalRecordPlaceInPublish::LastRecordOfThePublish,
        back_chain: plan.back_chain,
        filesystem_identifier: unit_filesystem_identifier(&pool.parameters.filesystem_identifier),
        // 新根段七项取这次发布写的根上的（D23（journal 的角色与格式） 已定项 15，用户 2026-10-02「回到甲」）。
        new_tree_table: root.tree_table,
        new_mapping_root: root.mapping_root,
        new_tree_identifier_watermark: root.tree_identifier_watermark,
        new_rollback_floor: root.rollback_floor,
        new_reclaim_threshold: root.reclaim_threshold,
        new_pinned_ring_slot: root.pinned_ring_slot,
        new_instance_table: root.instance_table,
        new_allocation_record_tree_root: root.allocation_record_tree_root,
        new_unmount_marker: root.unmount_marker,
        named: Vec::new(),
    };
    let record_bytes = record.to_bytes();
    // 零单元：屏障 → 空记录 → 屏障 → 根槽 FUA → 系统配置槽轮换 → 屏障（单元那一段是空的，头一道屏障前面没有写——上一次发布
    // 末尾那道或取号那道已经放行了——同一个写入口上不发它）。
    let writes = PublishWrites {
        units: Vec::new(),
        records: vec![JournalRecordWrite {
            counter: plan.counter,
            bytes: record_bytes.clone(),
        }],
        checkpoint_txg: plan.txg,
        root_slot: root.to_slot(pool.root_slot_bytes()),
        journal_tail: plan.counter,
        journal_instance: plan.instance,
    };
    (
        writes,
        VersionWithoutFilePublishOutput {
            root,
            record,
            record_bytes,
            earlier_records_of_this_publish: Vec::new(),
            rewritten: Vec::new(),
            units: Vec::new(),
            allocation_records:
                AllocationRecordsOfTheVersionWithoutFile::SameAsThePreviousVersionNotHandedInByAZeroUnitPublish,
            writes: WritesByStructureKind::NOTHING_WRITTEN,
        },
    )
}

/// 写行那次发布的参数，树表 0 条的一版上（D18（块里携带什么信息） 已定项 11「每次可写挂载都写行」）：身份字段同零单元发布，
/// 另带这次整条实例表链怎么重写（这次之后整张表的行——调用方在上一版那张表后面接上这次的行——与被换下的那条旧链）。
#[derive(Clone, Copy, Debug)]
pub struct InstanceTableOnlyPublishPlan<'records> {
    pub txg: CheckpointTxg,
    /// jsn 计数器，全池接着走（D23（journal 的角色与格式） 已定项 14 第 3 条）。
    pub counter: u64,
    pub instance: InstanceGeneration,
    /// 反向链：本实例的第一条恒 0（D23（journal 的角色与格式） 已定项 19 ②）。
    pub back_chain: u32,
    pub rollback_floor: CheckpointTxg,
    pub instance_table: &'records InstanceTableRewrite,
    /// 这次写进根记录与记录新根段的树 ID 水位，同 `ZeroUnitPublishPlan::tree_identifier_watermark`：写行那次发布一个树 ID 都不发，
    /// 就是根环里全部根记录的 max（D8（核心索引结构） 已定项 8 ②）——回退到树表 0 条的一版时它高于那一版自己带的。
    pub tree_identifier_watermark: u64,
}

/// 写行那次发布，上一版树表 0 条：**重写实例表链与分配记录树**（链有几片就几个实例表单元，加分配记录树这次内容变了的节点），
/// 落盘顺序同别的发布（D16（发布语义） 已定项 7）——单元写 → 屏障 → journal 记录（点名这几个单元）→ 屏障 → 根槽 FUA → 系统配置槽轮换 → 屏障。
///
/// 为什么是这两样：树表 0 条 ⇒ 这一版没有记账树，D16（发布语义） 已定项 9 那五样（记账行、记账树节点、映射条目、
/// 树表单元、树表条目）一样都不写；而 D18（块里携带什么信息） 已定项 11 要求每次可写挂载都写行、写行 COW 重写整条链 ⇒ 实例表自己那几个单元
/// 必写。**分配记录树是第二样**：写行换下上一版那条实例表链，这条释放要有地方记，不然重开之后只能从根记录那两条指针重建账、
/// 那几片就成了空闲槽，而根环里 txg 更低的候选根还指着它们（2026-09-23 用户定案随 C512（树表 0 条的一版上被换下的单元记在哪））。
/// 它的根指针住**根记录**新加的那一项，不进树表——进树表 `tree_table_has_no_entries` 当场翻面，
/// 按 `PreviousVersion::WithoutFile` / `WithFile` 分流的每一处跟着变。这棵树照 D8（核心索引结构） 已定项 14 按绝对槽号按位置寻址，
/// 树号 0（这一版还没登记过任何树），节点豁免映射；重写哪几个节点照带文件那一路走到固定点
/// （[`settle_the_allocation_record_tree_of_a_row_publish_on_a_version_without_file`]），没变的节点照抄上一版的。
/// 落点照 D3（空间分配） 已定项 5 从聚簇段 bump、按已定项 10 ⑤ 各自那一档取（实例表各片最前、尾片先，分配记录树节点在后、先叶后根），
/// 与这几个角色在别的发布路径上走的是同一条规则。
///
/// 被换下的（上一版的整条实例表链、上一版那棵分配记录树这次重写的节点）在同一次发布里释放（释放先于分配，D3（空间分配） 已定项 7）：
/// 两样都豁免映射（实例表见 D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12；这一版的分配记录树根住根记录），
/// 实例表链的各片取计划带着的那条旧链（`instance_table_chain_to_release`），分配记录树的节点取分配器记着的那一版那棵树的指针，
/// 三样逐盘核过（`placement_to_release_after_checking_every_device`）。上一版是 mkfs 的第 0 代（分配记录树根指针全零）时只释放实例表。
///
/// 新根照上一版的根，只换 checkpoint_txg、实例代号、回退下界、实例表指针、分配记录树根指针与树 ID 水位（取计划里给的，
/// D8（核心索引结构） 已定项 8 ②）：树表、映射树根照抄（这一版的树表仍是 mkfs 那片 0 条的）。
///
/// # Errors
/// 分配器上冻结着一次没重发的发布（`PublishFrozenAfterAWriteFailureIsNotResentYet`）、
/// 被换下的任一片的三样核不过（`ReleaseTarget*` / `ReleaseSpanMismatch`）、落点取不到（`PlacementRefused`）、
/// 分配记录树重写集的固定点没定下来（`AllocationRecordTreeRewriteSetDidNotSettle`）、块设备报错。
/// 前几样在任何写之前返回，盘上逐字节不变；块设备错交回时分配器回到发布之前的样子、这次发布冻结在它上面等原样重发（[`FrozenPublish`]）。
///
/// # Panics
/// 计划带着的旧链第 0 片不是上一版根记录指着的那一片：调用方拼行与拼旧链读的是同一条根。
pub fn publish_instance_table_on_version_without_file<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous_root: &RootRecord,
    plan: InstanceTableOnlyPublishPlan<'_>,
) -> Result<VersionWithoutFilePublishOutput, PublishError> {
    refuse_while_a_publish_is_frozen(allocator)?;
    // 失败就换回去：取落点会动分配器（bump 指针、位图、记录），中途报错不留半新的池。
    let allocator_before_this_publish = allocator.clone();
    let built =
        prepare_the_row_publish_on_a_version_without_file(pool, allocator, previous_root, plan);
    let (writes, output) = match built {
        Ok(built) => built,
        Err(refusal) => {
            *allocator = allocator_before_this_publish;
            return Err(refusal);
        }
    };
    persist_the_publish_or_freeze_it(
        pool,
        allocator,
        allocator_before_this_publish,
        writes,
        PoolVersion::WithoutFile(output),
    )
    .map(|version| {
        version.into_version_without_file().expect(
            "交给落盘的是树表 0 条的一版，交回的就是它（`PoolVersion::with_writes` 不换成员）",
        )
    })
}

/// 树表 0 条那一版写行时这一次的样子：分配记录树这次之后的计划、这次重写的角色（实例表各片尾片先，再分配记录树重写的节点，先叶后根）、
/// 要释放的落点（整条旧链在前，再是分配记录树换下的节点）。
struct SettledRowPublishOnAVersionWithoutFile {
    allocation_record_tree: AllocationRecordTreePlan,
    rewritten_roles: Vec<TransactionUnit>,
    release: Vec<Placement>,
    /// 固定点走了几轮（定下来的那一轮算在内）。
    rounds_walked: usize,
}

/// 树表 0 条那一版写行时，分配记录树重写哪几个节点（固定点，同带文件那一路的 [`settle_the_allocation_record_tree`]）：上一版的树是分配器记着的
/// 那一棵（`PoolAllocator::allocation_record_tree_of_the_version_without_file`；mkfs 的第 0 代没有树，节点全新），换下的节点逐盘核三样
/// （`placement_to_release_after_checking_every_device`，只查不改）。在分配器的拷贝上走，不动 `allocator`。
///
/// # Errors
/// 旧链或换下的节点三样核不过（`ReleaseTarget*` / `ReleaseSpanMismatch`）、拷贝上取不到落点（`PlacementRefused`）、
/// 轮数用完（`AllocationRecordTreeRewriteSetDidNotSettle`）。
///
/// # Panics
/// 上一版根记录那一项不是全零，而分配器没记着那一版的分配记录树，或记着的那一棵的根不是那一项指的：分配器的这一项由同一进程的写行
/// 或挂载时整棵读回来记下（`mount`），与根记录说的是同一版。
fn settle_the_allocation_record_tree_of_a_row_publish_on_a_version_without_file(
    previous_root: &RootRecord,
    instance_table: &InstanceTableRewrite,
    txg: CheckpointTxg,
    allocator: &PoolAllocator,
) -> Result<SettledRowPublishOnAVersionWithoutFile, PublishError> {
    let geometry = AllocationRecordTreeGeometry::of_allocator(allocator);
    let previous_tree = if previous_root.allocation_record_tree_root == NodePointer::empty_root() {
        None
    } else {
        let tree = allocator
            .allocation_record_tree_of_the_version_without_file()
            .expect("根记录那一项不是全零时，分配器记着那一版的分配记录树（同一进程的写行或挂载时读回来的）");
        assert_eq!(
            tree.version.root_pointer(),
            previous_root.allocation_record_tree_root,
            "分配器记着的那一棵就是根记录那一项指的那一棵"
        );
        Some(tree)
    };
    let previous_nodes: BTreeSet<AllocationRecordTreeNode> = previous_tree
        .map(|tree| tree.version.node_set())
        .unwrap_or_default();
    let previous_records: &[AllocationRecord] =
        previous_tree.map_or(&[], |tree| tree.records.as_slice());
    let chain = instance_table_chain_to_release(&instance_table.replaced_chain, allocator)?;
    let instance_table_roles =
        instance_table_page_roles_in_bump_order(instance_table.pages_after_this_publish());
    let previous_version_of_the_tree = previous_tree.map(|tree| &tree.version);
    let mut rewritten: BTreeSet<AllocationRecordTreeNode> = BTreeSet::new();
    let mut nodes_after: BTreeSet<AllocationRecordTreeNode> = previous_nodes.clone();
    // 迭代上界与跨轮携带的同 `settle_the_allocation_record_tree`。
    for round_index in 0..ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT {
        let rewritten_among_the_nodes_after: BTreeSet<AllocationRecordTreeNode> =
            rewritten.intersection(&nodes_after).copied().collect();
        let allocation_plan = crate::allocation_record_tree::plan_the_tree_after_this_publish(
            &previous_nodes,
            &nodes_after,
            &rewritten_among_the_nodes_after,
        );
        let rewritten_by_the_plan: BTreeSet<AllocationRecordTreeNode> =
            allocation_plan.rewritten_nodes().into_iter().collect();
        let mut release = chain.clone();
        for node in &allocation_plan.replaced_previous_nodes {
            let pointer = previous_tree
                .and_then(|tree| tree.version.pointer_of(*node))
                .expect("换下的节点在上一版那棵树里");
            release.push(placement_to_release_after_checking_every_device(
                role_of_allocation_record_tree_node(*node),
                &pointer.locations,
                allocator,
            )?);
        }
        let rewritten_roles: Vec<TransactionUnit> = instance_table_roles
            .iter()
            .copied()
            .chain(
                allocation_plan
                    .rewritten_nodes()
                    .into_iter()
                    .map(role_of_allocation_record_tree_node),
            )
            .collect();
        let mut rehearsal = allocator.clone();
        for placement in &release {
            rehearsal.release(*placement, txg);
        }
        // 释放本身弄脏的叶先并进重写集再取落点（同 `settle_the_allocation_record_tree`）。
        let changed_by_the_release =
            nodes_whose_contents_changed(&geometry, previous_records, rehearsal.records());
        if !changed_by_the_release.is_subset(&rewritten_by_the_plan) {
            merge_into_the_rewrite_set_walking_the_release_chain(
                &mut rewritten,
                changed_by_the_release,
                previous_version_of_the_tree,
                &geometry,
            );
            nodes_after.extend(nodes_holding_records(&geometry, rehearsal.records()));
            continue;
        }
        let rewrite_set_leaves = rewrite_set_leaves_of_the_roles(&rewritten_roles, allocator);
        for identity in &rewritten_roles {
            allocate_placement_for_role(&mut rehearsal, *identity, txg, &rewrite_set_leaves)?;
        }
        let changed =
            nodes_whose_contents_changed(&geometry, previous_records, rehearsal.records());
        let holding_records = nodes_holding_records(&geometry, rehearsal.records());
        if holding_records == nodes_after && changed.is_subset(&rewritten_by_the_plan) {
            return Ok(SettledRowPublishOnAVersionWithoutFile {
                allocation_record_tree: allocation_plan,
                rewritten_roles,
                release,
                rounds_walked: round_index + 1,
            });
        }
        merge_into_the_rewrite_set_walking_the_release_chain(
            &mut rewritten,
            changed,
            previous_version_of_the_tree,
            &geometry,
        );
        nodes_after = holding_records;
    }
    Err(PublishError::AllocationRecordTreeRewriteSetDidNotSettle {
        rounds: ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT,
    })
}

/// 这次重写出去的实例表链上的一片：角色、单元字节、指着它的指针（第 0 片的进根记录，第 k 片的进第 k − 1 片的链指针记录）。
pub(super) struct InstanceTablePageUnit {
    pub(super) role: TransactionUnit,
    pub(super) bytes: Vec<u8>,
    pub(super) pointer: NodePointer,
}

/// 装一条实例表链（D18（块里携带什么信息） 已定项 11）：行按链上的次序一片写满 369 行再开下一片、最后一片装剩下的
/// （`instance_table_rows_of_each_page`，用户 2026-09-24 定案），每片末尾一条链指针记录——最后一片写「无下一片」，别的片写下一片的指针。
/// **尾片先装、先发出生序号**（D3（空间分配） 已定项 10 ⑤，用户 2026-09-24 定案：第 k 片当第 k + 1 片的父，照先叶后根——第 k 片的
/// 链指针记录要第 k + 1 片的落点、整单元校验和与出生序号，第 k + 1 片装好了它才装得出来）。落点由调用方先按同一个次序取好
/// （`instance_table_page_roles_in_bump_order`），这里按角色查。交回的各片按链上的次序，第 0 片在前。
/// 带文件的一版上写行（`publish_admitted`）与树表 0 条的一版上写行（`prepare_the_row_publish_on_a_version_without_file`）共用这一处：
/// 两处各装一份，片的切法与发号次序会分叉。
///
/// # Panics
/// 某一片的角色没取落点（`slot_of` 里的断言）：调用方按同一张行表的片数取的落点。
pub(super) fn build_instance_table_chain(
    rows: &[InstanceRow],
    slot_of: &dyn Fn(TransactionUnit) -> SlotNumber,
    txg: CheckpointTxg,
    write_order: WriteOrder,
    filesystem_identifier: &[u8; 16],
    device_identities: &[DeviceIdentity],
    sequences: &mut BirthSequenceAllocator,
) -> Vec<InstanceTablePageUnit> {
    let rows_of_each_page = instance_table_rows_of_each_page(rows);
    let mut pages_tail_first: Vec<InstanceTablePageUnit> =
        Vec::with_capacity(rows_of_each_page.len());
    // 迭代次数的上界是片数；跨轮带的只有后一片的指针（前一片的链指针记录要它），没有提前出口。
    let mut pointer_to_the_next_page: Option<NodePointer> = None;
    for (position, page_rows) in rows_of_each_page.iter().enumerate().rev() {
        let page = InstanceTablePageIndex(u64::try_from(position).expect("片序号"));
        let role = TransactionUnit::of_instance_table_page(page);
        let chain = match pointer_to_the_next_page {
            None => InstanceTableChainRecord::LastPage,
            Some(next_page) => InstanceTableChainRecord::NextPage(next_page),
        };
        let birth_sequence = sequences.next(
            TreeIdentifier(TREE_IDENTIFIER_NONE),
            txg,
            write_order.instance,
        );
        let bytes = build_packed_unit(
            page.packed_identity(),
            u16::try_from(INSTANCE_ROW_BYTES).expect("88"),
            &instance_table_page_records(page_rows, &chain),
            txg,
            filesystem_identifier,
            write_order,
            birth_sequence,
        );
        let pointer = NodePointer {
            head: PointerHead {
                birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
                birth_txg: txg,
            },
            locations: location_entries(device_identities, slot_of(role), &bytes),
            instance: write_order.instance,
            birth_sequence,
        };
        pointer_to_the_next_page = Some(pointer);
        pages_tail_first.push(InstanceTablePageUnit {
            role,
            bytes,
            pointer,
        });
    }
    pages_tail_first.reverse();
    pages_tail_first
}

/// 树表 0 条那一版上写行那次发布，落盘之前的全部：走分配记录树重写集的固定点、释放、取落点、装单元，交回装好、还没落盘的字节与这一版
/// （写账是空的，落盘由调用方做）。只动 `allocator`：失败时它停在中途，由调用方换回去——发布路径换回进来时那一份，
/// 可写挂载取号之前的预演拿的本来就是拷贝（`mount`，D18（块里携带什么信息） 已定项 11「可写挂载的顺序」第五个合取：预演里写行那次报错，
/// 同样判这次不能可写）。两处走的是同一段代码。
///
/// # Errors
/// 旧链或换下的分配记录树节点三样核不过、落点取不到、固定点没定下来（见 [`publish_instance_table_on_version_without_file`]）。
///
/// # Panics
/// 计划带着的旧链第 0 片不是上一版根记录指着的那一片：调用方拼行与拼旧链读的是同一条根。
pub fn prepare_the_row_publish_on_a_version_without_file<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous_root: &RootRecord,
    plan: InstanceTableOnlyPublishPlan<'_>,
) -> Result<(PublishWrites, VersionWithoutFilePublishOutput), PublishError> {
    assert_eq!(
        plan.instance_table.replaced_chain.first(),
        Some(&previous_root.instance_table),
        "计划带着的旧链是上一版根记录指着的那一条：调用方拼行与拼旧链读的是同一条根"
    );
    let txg = plan.txg;
    let instance = plan.instance;
    let filesystem_identifier = &pool.parameters.filesystem_identifier;
    let write_order = WriteOrder {
        instance,
        // 写行那次发布不承载事务（D23（journal 的角色与格式） 已定项 19 ①：空发布与写行的记录写事务号 0）。
        transaction: 0,
    };
    let settled = settle_the_allocation_record_tree_of_a_row_publish_on_a_version_without_file(
        previous_root,
        plan.instance_table,
        txg,
        allocator,
    )?;
    pool.rounds_of_the_last_allocation_record_tree_settling
        .set(settled.rounds_walked);
    // 这次发布切出来的记录条数在动分配器之前判（代码审阅第 26 条）：点名的就是这次重写的角色（下面 `named_roles`）。
    refuse_a_publish_of_more_journal_records_than_the_limit(
        roles_named_by_each_record_of_the_publish(
            &settled.rewritten_roles,
            pool.journal_record_named_entry_capacity(),
        )
        .len(),
        pool.parameters.geometry.journal_ring_bytes,
    )?;
    let previous_tree_pointers = allocator
        .allocation_record_tree_of_the_version_without_file()
        .filter(|_| previous_root.allocation_record_tree_root != NodePointer::empty_root())
        .map(|tree| tree.version.clone());
    for released in settled.release.iter().copied() {
        allocator.release(released, txg);
    }
    // 实例表链的各片最前、尾片先（D3（空间分配） 已定项 10 ⑤），分配记录树的节点在它们之后按位置先叶后根取：
    // 这些节点自己的分配记录也要进这棵树，所以落点都取完才装节点。
    let mut slots: BTreeMap<TransactionUnit, SlotNumber> = BTreeMap::new();
    let rewrite_set_leaves = rewrite_set_leaves_of_the_roles(&settled.rewritten_roles, allocator);
    for identity in &settled.rewritten_roles {
        let placement =
            allocate_placement_for_role(allocator, *identity, txg, &rewrite_set_leaves)?;
        slots.insert(*identity, placement.slot);
    }
    // 落点取完、这次不再分配：记下这条根盖掉的根环槽（同 `publish_admitted` 那一处；失败时分配器由调用方整个换回去），
    // 记之前照 E2 重读挂载时读不出的那一槽（`mount::record_root_written_by_this_process_rereading_root_ring_slots_unreadable_at_mount`）。
    crate::mount::record_root_written_by_this_process_rereading_root_ring_slots_unreadable_at_mount(
        &*pool.devices,
        pool.parameters,
        allocator,
        txg,
    );
    // 记完这条根、回收完之后的门槛与最小钉值就是根记录 t 与 p 的采样点（回收之后、装根之前，C773）。
    let reclaim_floor_when_the_root_was_recorded = allocator
        .reclaim_floor_when_the_latest_root_was_recorded()
        .expect(
            "上一句刚记过这条根：record_root_written_by_this_process 记完就记下回收门槛与最小钉值",
        );
    // 这一次发布的提交内生块（实例表链的各片、分配记录树的节点）都归树 0（这一版还没登记过任何树），在 (txg, 实例) 上连着发出生序号：
    // 实例表各片按 bump 次序（尾片先）在前，分配记录树的节点在后（先叶后根）。
    let mut sequences = BirthSequenceAllocator::default();
    let device_identities: Vec<DeviceIdentity> =
        pool.devices.iter().map(|(identity, _)| *identity).collect();
    let instance_table_pages = build_instance_table_chain(
        &plan.instance_table.rows,
        &|identity| slots[&identity],
        txg,
        write_order,
        filesystem_identifier,
        &device_identities,
        &mut sequences,
    );
    let instance_table_pointer = instance_table_pages
        .first()
        .expect("一条链至少一片：0 行也写出空的那一片")
        .pointer;
    // 分配记录树：树号 0（这一版还没登记过任何树，写分配记录树的号 65538 会当场判红 I-7.8（树 ID 水位不小于盘上出现过的最大树 ID）），
    // 由根记录独占持有，与树表单元、实例表单元同一个身份（D22（单元原子性怎么合成） 已定项 7 / 已定项 12）。
    let mut allocation_records: Vec<AllocationRecord> = allocator.records().to_vec();
    allocation_records.sort_by_key(AllocationRecord::sort_key);
    let allocation_geometry = AllocationRecordTreeGeometry::of_allocator(allocator);
    let allocation_built = build_allocation_record_tree(
        &settled.allocation_record_tree,
        &allocation_geometry,
        &allocation_records,
        &|node| {
            previous_tree_pointers
                .as_ref()
                .and_then(|version| version.pointer_of(node))
        },
        TreeIdentifier(TREE_IDENTIFIER_NONE),
        &MultiLevelTreeBuildContext {
            txg,
            filesystem_identifier,
            instance,
            device_identities: &device_identities,
        },
        &|identity| slots[&identity],
        &mut sequences,
    );
    let allocation_record_tree_root = allocation_built.version.root_pointer();
    // 这一版的分配记录树换成了刚装的这一棵：下一次发布（再写一次行，或在这一版上发第一个文件版本）要照抄或换下它的节点，
    // 而那时手里只有这个分配器——记的要是上一版那一棵（这次刚换下几个节点的那一棵），新写的节点就永远没人释放，
    // I-3.1（已分配统计对得上） 在抬 F 之后当场红。
    allocator.note_allocation_record_tree_of_the_version_without_file(
        crate::allocator::AllocationRecordTreeOfTheVersionWithoutFile {
            version: allocation_built.version.clone(),
            records: allocation_records.clone(),
        },
    );
    // 点名项（D23（journal 的角色与格式） 已定项 17）：这次重写的角色各一项，按 bump 次序——实例表链的各片（尾片先），分配记录树重写的节点。
    let named_roles: Vec<TransactionUnit> = settled.rewritten_roles.clone();
    let named_unit_of = |identity: TransactionUnit| -> NamedUnit {
        match identity {
            TransactionUnit::AllocationTreeNodeBelowTheRoot(_) | TransactionUnit::AllocationTree => {
                let unit = allocation_built
                    .rewritten_units
                    .iter()
                    .find(|unit| unit.identity == identity)
                    .expect("点名的分配记录树节点是这次重写的");
                NamedUnit {
                    locations: pool.location_entries(unit.slot, &unit.bytes),
                    unit_class: identity.unit_class(),
                    // 树 ID 0：这一版还没登记过任何树，这棵树由根记录独占持有。
                    birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
                    birth_txg: txg,
                    key_tail: node_key_tail(
                        instance,
                        allocation_built.birth_sequences_of_rewritten_nodes[&identity],
                    ),
                }
            }
            TransactionUnit::InstanceTable | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
                let page = instance_table_pages
                    .iter()
                    .find(|page| page.role == identity)
                    .expect("每个取了落点的实例表角色都装了一片");
                NamedUnit {
                    locations: page.pointer.locations,
                    unit_class: identity.unit_class(),
                    // 实例表单元不属于任何一棵树（树 0）；这一版还没发过树 ID，不按 `TransactionUnit::tree` 取。
                    birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
                    birth_txg: txg,
                    key_tail: node_key_tail(instance, page.pointer.birth_sequence),
                }
            }
            TransactionUnit::Data(_)
            | TransactionUnit::ExtentLowerNode(_)
            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
            | TransactionUnit::ExtentRoot
            | TransactionUnit::InodeLeafContainer(_)
            | TransactionUnit::InodeRoot
            | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
            | TransactionUnit::AccountingTree
            | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
            | TransactionUnit::DeadlistTree
            | TransactionUnit::DeletedInodesRoot
            | TransactionUnit::LoggedOperationsRoot
            | TransactionUnit::TombstoneLeafContainer(_)
            | TransactionUnit::TombstoneRoot
            | TransactionUnit::TombstoneNodeBelowTheRoot(_)
            | TransactionUnit::LineageLeafContainer(_)
            | TransactionUnit::LineageRoot
            | TransactionUnit::LineageNodeBelowTheRoot(_)
            | TransactionUnit::RetirementLeafContainer(_)
            | TransactionUnit::RetirementRoot
            | TransactionUnit::RetirementNodeBelowTheRoot(_)
            | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
            | TransactionUnit::MappingTree
            | TransactionUnit::DataOfAFileOtherThanTheFirst { .. }
            | TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { .. }
            | TransactionUnit::TreeTable => unreachable!(
                "树表 0 条的一版上写行只点名实例表链各片与分配记录树节点（`named_roles` 就是这两样）"
            ),
        }
    };
    // 这次发布切成几条记录（D23（journal 的角色与格式） 已定项 17「末条再跨记录」，与带文件那一路同一个切法：
    // `roles_named_by_each_record_of_the_publish`）：点名项多于一条记录装得下的（实例表长到 64 片起）就装满一条再开下一条。
    // 写行那次发布不承载事务，这几条都是事务号 0、同属一个事务（`transaction_offset_of_each_record_of_the_publish` 全是 0）：
    // 提交标记只在它的最后一条上（已定项 7），「本次发布末条」标志只在真正的最后一条上（已定项 17），本次发布内序号依次 1..N（已定项 4），
    // 反向链第一条接 `plan.back_chain`、之后每条接这次发布里前一条的头（已定项 8），每条都带整次发布的新根段（已定项 15）。
    let named_entry_capacity = pool.journal_record_named_entry_capacity();
    let roles_named_by_each_record =
        roles_named_by_each_record_of_the_publish(&named_roles, named_entry_capacity);
    let transaction_offset_of_each_record =
        transaction_offset_of_each_record_of_the_publish(&named_roles, named_entry_capacity);
    let root = RootRecord {
        filesystem_identifier: previous_root.filesystem_identifier,
        // 写行那次发布是新实例的第一次发布，不是卸载那一串。
        unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
        instance,
        checkpoint_txg: txg,
        tree_table: previous_root.tree_table,
        tree_identifier_watermark: plan.tree_identifier_watermark,
        rollback_floor: plan.rollback_floor,
        instance_table: instance_table_pointer,
        mapping_root: previous_root.mapping_root,
        allocation_record_tree_root,
        // 带单元的发布：t = max(这条根自己带的 F, 分配器记这条根时的回收门槛)，p = 记完之后还钉着的槽里最小的钉值（C773，用户 2026-10-20 定）。
        reclaim_threshold: reclaim_floor_when_the_root_was_recorded
            .reclaim_threshold_of_a_root_carrying(plan.rollback_floor),
        pinned_ring_slot: reclaim_floor_when_the_root_was_recorded.pinned_ring_slot(),
    };
    let mut written_records: Vec<WrittenJournalRecord> =
        Vec::with_capacity(roles_named_by_each_record.len());
    // 迭代次数的上界是这次的记录条数（≥ 1）；跨轮携带的只有已经装好的记录（下一条的反向链要罩前一条的头）。
    for (record_offset_in_this_publish, roles_of_this_record) in
        roles_named_by_each_record.iter().enumerate()
    {
        let back_chain = match written_records.last() {
            None => plan.back_chain,
            Some(previous_record_of_this_publish) => {
                back_chain_of(&previous_record_of_this_publish.bytes)
            }
        };
        let transaction_offset = transaction_offset_of_each_record[record_offset_in_this_publish];
        let transaction_of_the_next_record = transaction_offset_of_each_record
            .get(record_offset_in_this_publish + 1)
            .copied();
        let record = JournalRecord {
            instance,
            counter: plan.counter + u64::try_from(record_offset_in_this_publish).expect("记录序号"),
            checkpoint_txg: txg,
            // 写行那次发布不承载事务（D23（journal 的角色与格式） 已定项 19 ①）：事务号 0 加这条属于的事务序号（恒 0）。
            transaction: transaction_offset,
            is_commit: transaction_of_the_next_record != Some(transaction_offset),
            ordinal_within_publish: JournalRecordOrdinalWithinPublish::of_record_at_offset(
                record_offset_in_this_publish,
            ),
            place_in_publish: if transaction_of_the_next_record.is_none() {
                JournalRecordPlaceInPublish::LastRecordOfThePublish
            } else {
                JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow
            },
            back_chain,
            filesystem_identifier: unit_filesystem_identifier(filesystem_identifier),
            // 新根段七项取这次发布写的根上的（D23（journal 的角色与格式） 已定项 15，用户 2026-10-02「回到甲」）。
            new_tree_table: root.tree_table,
            new_mapping_root: root.mapping_root,
            new_tree_identifier_watermark: root.tree_identifier_watermark,
            new_rollback_floor: root.rollback_floor,
            new_reclaim_threshold: root.reclaim_threshold,
            new_pinned_ring_slot: root.pinned_ring_slot,
            new_instance_table: root.instance_table,
            new_allocation_record_tree_root: root.allocation_record_tree_root,
            new_unmount_marker: root.unmount_marker,
            named: roles_of_this_record
                .iter()
                .copied()
                .map(named_unit_of)
                .collect(),
        };
        let bytes = record.to_bytes();
        written_records.push(WrittenJournalRecord { record, bytes });
    }
    let last_counter_of_this_publish =
        plan.counter + u64::try_from(written_records.len() - 1).expect("一次发布至少一条记录");
    // 单元写按 bump 次序：实例表链的各片（尾片先），分配记录树重写的节点（先叶后根）最后。
    let units: Vec<PublishedUnit> = settled
        .rewritten_roles
        .iter()
        .filter(|identity| identity.is_a_page_of_the_instance_table())
        .map(|identity| {
            let page = instance_table_pages
                .iter()
                .find(|page| page.role == *identity)
                .expect("每个取了落点的实例表角色都装了一片");
            PublishedUnit {
                slot: slots[identity],
                identity: *identity,
                bytes: Arc::from(page.bytes.as_slice()),
            }
        })
        .chain(allocation_built.rewritten_units.iter().cloned())
        .collect();
    let writes = PublishWrites {
        units,
        records: written_records
            .iter()
            .map(|written_record| JournalRecordWrite {
                counter: written_record.record.counter,
                bytes: written_record.bytes.clone(),
            })
            .collect(),
        checkpoint_txg: txg,
        root_slot: root.to_slot(pool.root_slot_bytes()),
        // 系统配置里的 tail 存这次发布末条记录的 jsn 计数器（D23（journal 的角色与格式） 已定项 18）。
        journal_tail: last_counter_of_this_publish,
        journal_instance: instance,
    };
    let WrittenJournalRecord {
        record,
        bytes: record_bytes,
    } = written_records
        .pop()
        .expect("一次发布至少一条记录（`roles_named_by_each_record_of_the_publish` 至少给一项）");
    // 交出去的单元就是这次交给设备的那几个（与 `rewritten` 逐项同序）：对拍比的是实现写出的字节，不另装一份。
    let units_handed_in_with_the_version = writes.units.clone();
    Ok((
        writes,
        VersionWithoutFilePublishOutput {
            root,
            record,
            record_bytes,
            earlier_records_of_this_publish: written_records,
            rewritten: settled.rewritten_roles,
            units: units_handed_in_with_the_version,
            allocation_records:
                AllocationRecordsOfTheVersionWithoutFile::WrittenIntoTheAllocationRecordTreeByThisPublish(
                    allocation_records,
                ),
            writes: WritesByStructureKind::NOTHING_WRITTEN,
        },
    ))
}

/// 一个角色这次发布的落点：用户数据按政策函数，其余是提交内生块（码 3 容器按数据单元那一档，D3（空间分配） 已定项 10 ⑤）；
/// 提交内生块回落时先取 `rewrite_set_leaves`（这次发布分配记录树重写集里的叶）里的空槽，再取空槽最多的叶（D3（空间分配） 已定项 8 第 2 条）。
/// 取不到时带上是哪个角色、分配器给的原因。
///
/// # Errors
/// `PlacementRefused`。
pub(super) fn allocate_placement_for_role(
    allocator: &mut PoolAllocator,
    identity: TransactionUnit,
    txg: CheckpointTxg,
    rewrite_set_leaves: &RewriteSetLeaves,
) -> Result<Placement, PublishError> {
    let placement = match identity.placement() {
        PlacementRule::UserData => allocator.try_allocate_user_data(txg),
        PlacementRule::CommitGenerated(footprint) => allocator
            .try_allocate_commit_generated_preferring_the_rewrite_set(
                footprint,
                txg,
                rewrite_set_leaves,
            ),
    };
    placement.map_err(|refusal| PublishError::PlacementRefused {
        unit: identity,
        refusal,
    })
}

/// 这次发布重写的角色里分配记录树的叶（层级 0），池里每块盘上都重写的那几片的序号（[`RewriteSetLeaves::present_on_every_device`]）：
/// 提交内生块回落先取这些叶里的空槽（[`PoolAllocator::try_allocate_commit_generated_preferring_the_rewrite_set`]）。
/// 固定点的演练与真发读的是同一串重写角色（定下来那一轮的角色就是真发的角色），所以两边得同一个集合、取同一串落点。
pub(super) fn rewrite_set_leaves_of_the_roles(
    rewritten_roles: &[TransactionUnit],
    allocator: &PoolAllocator,
) -> RewriteSetLeaves {
    let leaf_indices_per_device: Vec<BTreeSet<AllocationRecordTreeLeafIndex>> = allocator
        .devices
        .iter()
        .map(|device_map| {
            rewritten_roles
                .iter()
                .filter_map(|role| match allocation_record_tree_node_of_role(*role) {
                    Some(AllocationRecordTreeNode::BelowTheRoot(position))
                        if position.level == 0 && position.device == device_map.device =>
                    {
                        Some(AllocationRecordTreeLeafIndex(position.index_in_device))
                    }
                    Some(
                        AllocationRecordTreeNode::BelowTheRoot(_) | AllocationRecordTreeNode::Root,
                    )
                    | None => None,
                })
                .collect()
        })
        .collect();
    RewriteSetLeaves::present_on_every_device(&leaf_indices_per_device)
}
