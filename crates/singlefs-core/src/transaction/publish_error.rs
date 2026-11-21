//! 发布的错误：`PublishError` 与它的各种拒绝。

use super::*;

/// 发布能出的错：分配器给不出落点（哪一个单元、为什么）、多层码 2 树的形状算不出来、内容装不下、释放判定路径对不上，或底层块设备错。
/// 按 key 空间定形状的分配记录树与 extent 树不分裂、没有装不下这一格（D8（核心索引结构） 已定项 14），此前那两个成员
/// （分配记录装不进一个节点、extent 树要长内部节点）随之去掉。
#[derive(Debug)]
pub enum PublishError {
    /// 分配器拒了这个单元的落点，`refusal` 原样带着分配器的原因：每块盘上都没有合政策的落点（容量不够）是 `NoFreeSlotOnAnyDevice`；
    /// 小盘写满、各盘的落点不一致是第一版不支持的池形状（D2（RAID 条带策略） 已定项 2 ⚠️、已定项 10；D3（空间分配） 已定项 8），
    /// 不是容量不够。此前这四种一律报成 `NoSpaceFor`（C368（分配器落点只看盘 0，盘不等大时断言失败） 仍欠的那一半；
    /// 增补 3 第 2 件代码三方第一轮判决第三节第 2 条：胶水把它们都当成单元区墙）。
    PlacementRefused {
        unit: TransactionUnit,
        refusal: PlacementRefusal,
    },
    /// 空间准入不够（D28（挂载期承诺量） 已定项 1 的式子逐设备合取；C363 (b) 判决 `research/prompts/c363b-r1-main-verification.md`
    /// 第四节第 2 条把它接进发布路径）：这次发布有普通分配（用户数据单元、extent 树与 inode 树的节点，`admission::space_budget_of_role`），
    /// 它新写的全部角色在至少一块盘上多于可用(d)，带着不够的每一块。在算定这次发布的样子之后、读盘核被换下的单元与动分配器之前返回，
    /// 一个写都没发、盘上逐字节不变。同一个成员也报「这次的单元落不下」（C545（空间准入罩不住分裂与聚簇段层），用户 2026-09-27 定准入先拒）：
    /// 在走分配记录树的固定点之前，至少一块盘上段外成对的空槽少于这次的数据单元、或剩下没挡的槽少于这次提交内生块至多要的
    /// （`admission::admit_the_units_landing_on_every_device`；那一判里不够的每一块报的是短的那一种单元，见它的文档）。准入不够时先推空发布抬 F 再判（D16（发布语义） 已定项 1，C283（准入失败时不先推发布就报 ENOSPC））
    /// 归挂着的会话（`mounted_session`）：直接调发布路径的调用方收到的就是这一次的拒绝。
    SpaceAdmissionRefused(AdmissionRefusedOnSomeDevices),
    /// 合取 7 `全空聚簇段数 ≥ N₀` 不成立（D26（后台整理与放置回收） 已定项 3 的第二合取式；D3（空间分配） 已定项 7 登记表第 7 行，用户定只拦前台写入类发布）：
    /// 带着全空聚簇段少于 `N₀` 的每一块盘。与 `SpaceAdmissionRefused` 分开报，因为调用方要做的决定不同：这一条约束的是空间的形状
    /// （有没有整段空着），不是字节够不够——`df` 可以还有空闲，腾出整段要靠整理（或推抬 F 让 defer 里的段回收），不是删东西
    /// （`.claude/singlefs-ai-sop/rules/code-discipline.md`「错误」一节：成员按调用方要做的决定分）。在算定这次发布的样子之后、
    /// 式子那一判、读盘核被换下的单元与动分配器之前返回，一个写都没发、盘上逐字节不变。
    EmptyClusterSegmentsBelowTheFloor(EmptyClusterSegmentsBelowTheFloorOnSomeDevices),
    /// 记账树或中央映射树这次之后的形状算不出来（`code_two_tree::plan_the_tree_after_this_publish`）：要长到 257 层、
    /// 码 2 头的层级 1 字节写不下（多层之后映射树容量准入剩下的唯一一条，D19（块指针的结构与宽度预算） 已定项 5），
    /// 或上一版（从盘上重建的）树按分隔 key 走不到它自己叶里的一把 key。在动分配器、发任何一个写之前返回，盘上逐字节不变。
    /// 两棵树装不下一个节点时不再报错、照 D8（核心索引结构） 已定项 11 分裂（里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 28 行）。
    MultiLevelCodeTwoTreeRefused {
        tree: MultiLevelCodeTwoTree,
        refusal: CodeTwoTreeRefusal,
    },
    /// 释放判定路径在上一版的映射里查不到这个单元（D19（块指针的结构与宽度预算） 已定项 5 第 1 条：不按提示释放）。
    ReleaseNotInMapping { unit: TransactionUnit },
    /// 映射查出来的落点在**这块盘**的分配记录里没有条目。`device` 带着是哪块盘：`PoolAllocator::release` 对池里
    /// 每块盘都要求一条对得上的记录，而两块盘的分配记录树对不对称是**盘上读来的**、不是不变量（panic 面普查 R10）。
    ReleaseTargetNotAllocated {
        unit: TransactionUnit,
        device: DeviceIdentity,
        slot: SlotNumber,
    },
    /// 映射查出来的落点在**这块盘**上已经是已释放的（同一个落点释放两次）。
    ReleaseTargetAlreadyReleased {
        unit: TransactionUnit,
        device: DeviceIdentity,
        slot: SlotNumber,
    },
    /// 要释放的这个单元，上一版那条指针的两条位置条目落在不同的槽：槽号是**盘上读来的 6 字节**、可以是任何值，
    /// 而第一版的布局是两盘同槽（一个单元整个落在一列上、两盘各一份，D2（RAID 条带策略） 已定项 10 每个副本一条位置条目）
    /// ⇒ 这是一份坏镜像，不是我们的不变量被破坏，报错、不断言（panic 面普查 R5）。拒的是「盘上这条指针的两条位置条目不同槽」，
    /// 与 `PlacementRefusal` 那两条「各盘算出来的落点不同、第一版不支持」不是同一件事（那两条说的是写侧算出来的落点）。
    /// 与释放判定路径的其它几种一样：一个落点都不释放，在动分配器之前返回，盘上逐字节不变。
    ReleaseTargetLocationsOnDifferentSlots {
        unit: TransactionUnit,
        disagreement: LocationEntriesOnDifferentSlots,
    },
    /// **这块盘**的分配记录里的跨度与这个单元种类该有的跨度不符（映射给槽、记录给跨度，两边对不上）。
    ReleaseSpanMismatch {
        unit: TransactionUnit,
        device: DeviceIdentity,
        slot: SlotNumber,
        recorded_span: u64,
        expected_span: u64,
    },
    /// 上一版的映射节点里有一条条目窄于映射条目的字段表（55）：条目宽是映射树根自述的一个**盘上字段**，
    /// `parse_index_node` 只判了它 ≥ key 宽 27，切到偏移 55 之前要判一次（panic 面普查 R2）。
    /// 与 `ReleaseNotInMapping` 分开报：那一条说的是「节点好、里面没有这把 key」，这一条说的是「节点里的条目切不动」。
    MappingEntryNarrowerThanItsFieldTable {
        unit: TransactionUnit,
        entry_bytes: usize,
        field_table_bytes: usize,
    },
    /// 经映射释放的这个单元，映射条目的一条位置项指的盘不在这个池里（D19（块指针的结构与宽度预算） 已定项 5「硬规则 1 的读盘核读不出、
    /// 核出对不上时怎么办」：当映射条目损坏，用户 2026-09-25 定）。映射条目是盘上读来的字节，坏镜像、外来镜像才有。
    /// 在动分配器、读盘核任何一份、发任何一个写之前返回，盘上逐字节不变。
    MappingEntryLocationOnADeviceOutsideThePool {
        unit: TransactionUnit,
        device: DeviceIdentity,
        slot: SlotNumber,
    },
    /// 经映射释放的这个单元，映射条目的两条位置项指同一块盘（同上一条：当映射条目损坏）——另一块盘上那一份从来没被核过就会被释放。
    /// 在动分配器、读盘核任何一份、发任何一个写之前返回，盘上逐字节不变。`slot` 是第一条位置项的槽。
    MappingEntryLocationsOnTheSameDevice {
        unit: TransactionUnit,
        device: DeviceIdentity,
        slot: SlotNumber,
    },
    /// 用户给的内容装不进一个数据单元（32768 − 头 − 预留）：`publish_first_file` 与 `publish_overwrite` 按一个数据单元写
    /// （它们的契约），多单元的内容走 `publish_sequential_write`。
    ContentExceedsDataUnit { bytes: usize, capacity: usize },
    /// 分配记录树这次重写哪几个节点的固定点在 `rounds` 轮里没有定下来（`settle_the_allocation_record_tree`）。产品路径上走不到这里
    /// （释放不在这次发布里空出槽、多一个角色只把取落点那一串拉长，重写集与节点都单调）；只供测试的复用窗口置 0 开关下单调性推不出来，
    /// 条款没写那一格怎么办 ⇒ 第一版不支持：在任何落盘动作之前返回，盘上逐字节不变。
    AllocationRecordTreeRewriteSetDidNotSettle { rounds: usize },
    /// 这次发布要往 inode 树里写的记录，落法要的条款仓里还没定（[`InodeTreeWriteRefusal`] 的三格）：
    /// 在分配器、块设备都还没被碰过的时候返回，盘上逐字节不变。
    InodeTreeWriteRefused(InodeTreeWriteRefusal),
    /// `publish_first_file` 要建在上面的那一版的根与交给它的上一条记录说的不是同一版：记录解不出本池的记录（`None`），
    /// 或记录的 checkpoint_txg 与那条根的不同。新根的 txg 从那条根接着算、jsn 从那条记录接着算，两者对不上时接上去会盖在别的发布上。
    FirstFileVersionDoesNotFollowTheVersionItBuildsOn {
        version_to_build_on: CheckpointTxg,
        previous_record: Option<(CheckpointTxg, u64)>,
    },
    /// `publish_first_file` 要建在上面的那一版已经有过文件版本（它的树表不是 0 条，`tree_table_entries` 是条数）：
    /// 这条路径按「树还没建起来」写——`previous` 传 `None`、十二棵树从水位重新发号、inode 树与 extent 树从头建、
    /// 树表条目的诞生 txg 与 inode 1 的对象出生代都取这次发布的 txg。接在已经有文件的一版后面写出来的那条根，
    /// 会与同一条时间线上的旧根在 I-9.14（树表条目的诞生 txg 跨根不变） 与 I-9.10（对象出生代与 inode 记录相符） 上对不上
    /// （2026-09-23 崩溃注入默认批打中）。判的是树表条数，不是水位（C511（回退到无文件那一版之后诞生代怎么接） 第 3 步：
    /// 回退到树表 0 条的一版时水位带着根环里的 max，已经不是 mkfs 的 65536）。
    /// 同一个文件再写一版走 `publish_overwrite`。一个字节都不写。
    FirstFileVersionOnAVersionThatAlreadyHasAFile { tree_table_entries: usize },
    /// `publish_first_file` 要判「那一版有没有过文件版本」得读它的树表单元，而那一片两份都读不出或解不开
    /// （`failure` 原样带着恢复路径那一格的原因）：判不了就不写，一个字节都不写。
    TreeTableOfTheVersionToBuildOnUnreadable { failure: RecoveryFailure },
    /// `publish_first_file` 从那一版的树 ID 水位起连号发十二棵树的号，而水位（盘上读来的 8 字节）加十二个号越过 `u64::MAX`：
    /// 发不出号就不写，一个字节都不写。
    TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees(
        TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees,
    ),
    /// 分配器上冻结着一次落盘中途失败、还没原样重发的发布（`FrozenPublish`，D23（journal 的角色与格式） 已定项 14「这一版的失败处置」：
    /// 重发成功才建下一次发布）：先 `resend_the_frozen_publish`，成功之后在它交回的那一版上接着发。在任何读写之前返回，盘上逐字节不变。
    /// 带的是冻结的那一次的 checkpoint_txg 与实例代号。
    PublishFrozenAfterAWriteFailureIsNotResentYet {
        checkpoint_txg: CheckpointTxg,
        instance: InstanceGeneration,
    },
    /// 这次发布切出来的 journal 记录条数 `records_of_the_publish` 多于上限 `record_limit`（代码审阅第 26 条；
    /// [`journal_record_limit_of_one_publish`]：在飞记录数上限 = 环槽数 ÷ F，D23（journal 的角色与格式） 已定项 18，与 32 位序号取小）：
    /// 这样一次发布恢复施加不全、条数过了环槽数还会盖掉自己的记录。调用方要做的：把改动切成几次发布（顺序写分几次写）。
    /// 在算定这次发布的样子之后、准入、读盘核与动分配器之前返回，一个写都没发、盘上逐字节不变。
    JournalRecordsOfThePublishExceedTheLimit {
        records_of_the_publish: u64,
        record_limit: u64,
    },
    /// 要建在上面的那一版的 checkpoint_txg 已是 `u64::MAX`，这次发布的 txg（它加一）装不下（代码审阅第 36 条）：
    /// 那一版的 txg 是盘上读来的 8 字节，坏镜像、外来镜像才走得到。在任何读写之前返回，盘上逐字节不变。
    NextCheckpointTxgPastTheTopOfItsRange { version_to_build_on: CheckpointTxg },
    /// 这次发布要在墓碑树、谱系树或退役树里开一片新容器，而那棵树最右那片的号（盘上读来的 8 字节）已是 `u64::MAX`，下一个号装不下
    /// （C841，用户 2026-10-15 定「写路径报类型化错误」）：坏镜像、外来镜像才走得到。在算这次发布的样子那一步交回
    /// （[`PublishPlan::resolve`] 与它之前的普通分配那一半），分配器、块设备都还没被碰过，盘上逐字节不变。推抬 F 不会让号变小，不是空间不够。
    NextContainerNumberPastTheTopOfItsRange(NextContainerNumberPastTheTopOfItsRange),
    /// 覆盖写、顺序写要写的那个 inode 在上一版里是删除中的（nlink = 0：unlink 的意图已经发布，D8（核心索引结构） 已定项 16 第 4 条
    /// 「写入口拒 nlink = 0 的 inode」）：从意图那一版起它对读者已经是「没有文件」，再写一版就把 nlink 写回 1、复活一个正在回收的对象。
    /// 在任何读写之前返回，盘上逐字节不变。
    WriteToAnInodeBeingDeleted { inode: u64 },
    /// 覆盖写、顺序写要写的第一个文件在上一版里已经删完（unlink 最后一批删掉了 inode 1 那条记录，D8（核心索引结构） 已定项 16
    /// 第 4 条「现造记录的入口拒没有这条记录」；射程：删完之后不再建带数据的文件，C618）。在任何读写之前返回，盘上逐字节不变。
    WriteToAnInodeThatIsNotInTheTree { inode: u64 },
    /// 按 inode 号给一个在第一个文件删完之后才建的 inode 写数据：D8（核心索引结构） 已定项 16 射程「删完之后不再建带数据的文件
    /// （K-U5，记欠账）」，用户 2026-10-19 定「β 时序级」——第一个文件已经删完（inode 树里没有 inode 1 的记录）的一版上，目标的对象出生代
    /// `object_birth` 晚于 inode 1 那条整对象墓碑的死亡 txg `death_txg_of_the_first_file` 才拒；在那之前建的别的文件（原有的、带数据的
    /// 或 size 0 的）照常能写。依赖墓碑第一版不回收（D18（块里携带什么信息） 已定项 24，C618）。`inode` 是这次要写的号。
    /// 在任何读写之前返回，盘上逐字节不变。
    DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion {
        inode: u64,
        object_birth: CheckpointTxg,
        death_txg_of_the_first_file: CheckpointTxg,
    },
    /// unlink 要删的号不是一条活着的 inode 记录（树里没有、或已经在删除中）：在任何读写之前返回，盘上逐字节不变。
    UnlinkOfAnInodeThatIsNotALiveRecord { inode: u64 },
    /// `deleted_inodes` 根装不下这一版的号（`crate::deleted_inodes`，D8（核心索引结构） 已定项 16 第 6 条：类型化错误、不是空间不够）。
    /// 不是空间不够，不推抬 F。
    DeletedInodesQueueRefused(DeletedInodesQueueRefusal),
    /// 建快照那次发布（[`publish_create_snapshot`]）建不了（[`SnapshotCreationRefusal`]）：在任何读写之前返回，盘上逐字节不变。
    SnapshotCreationRefused(SnapshotCreationRefusal),
    /// 删快照的意图或某一批发不了（[`SnapshotDestructionRefusal`]）：在任何读写之前返回，盘上逐字节不变。
    SnapshotDestructionRefused(SnapshotDestructionRefusal),
    /// 删快照的一批要放的一条 deadlist 条目放不了（`refusal` 说为什么）：在任何落盘动作之前返回，盘上逐字节不变。
    DeadlistEntryNotReleasable {
        mapping_key: Vec<u8>,
        refusal: DeadlistEntryReleaseRefusal,
    },
    /// 截断或变长（[`publish_size_change`]、[`publish_truncation_step`]）做不了（[`TruncationRefusal`]）：在任何读写之前返回，盘上逐字节不变。
    TruncationRefused(TruncationRefusal),
    /// 块设备报的错。落盘阶段报的（带文件的一版、树表 0 条的一版上写行）：这次发布已冻结在分配器上等原样重发。
    BlockDevice(BlockDeviceError),
    /// 可写挂载取号之前预演续做（分配器拷贝上走删除队列那一趟，连同空间不够时只进内存的盘上走取号之后那一串与续做那一趟）报了错、
    /// 没走到续做的结局，而这一趟读盘时有一次读穿到底下那块盘、盘报了错（读错）：从头重走，重走到上限
    /// （岔路单 `research/prompts/m3-b44-code-r3-forks.md` 岔路二「没走完就重走」那一行，用户定）仍是这样，交最后那一趟最先的那一次读错，在取号之前拒
    /// （`MountError::Publish`，三份写账都是空的，两块盘逐字节不变；`mount` 的
    /// `refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range`）。
    /// 这次挂载读过的块预演与真做共用一份（`mounted_session::DeviceReadOnceInThisMount`），读错的那一块没记下：交给真做，真做会再读一次
    /// 底下那块盘，看到的可能与预演不同（读错是瞬时的），预演判的就不是真做走的那一串（岔路单 `research/prompts/m3-b44-code-r2-forks.md`
    /// 岔路一「读一次两边共用」那一行，用户定）。只在取号之前那一判里造，发布路径不报它。
    ReadFailedWhileRehearsingTheContinuationBeforeAcquisition(Box<ReadFailedOnTheDisk>),
    /// 同上一个成员的那一判，最后那一趟读盘没报过错、而有一次读读成了、读的一方核出自证不过（读坏：系统配置槽、单元校验和与位置项不符；
    /// 岔路单 `research/prompts/m3-b44-code-r3-forks.md` 岔路一「两样都做」那一行，用户定：读坏也记账，取号之前那一判照读错账与读坏账两本判）：
    /// 预演重走到上限仍没走到续做的结局，交最后那一趟最先的那一次读坏，在取号之前拒（`MountError::Publish`，三份写账都是空的，两块盘逐字节不变）。
    /// 读坏的那几块读坏那一刻已从这次挂载共用的那一份里丢掉（坏字节不留），交给真做的话真做会再读一次盘。只在取号之前那一判里造，发布路径不报它。
    ReadBackFailingSelfVerificationWhileRehearsingTheContinuationBeforeAcquisition(
        ReadFailingSelfVerification,
    ),
}

/// 一次读读成了、读的一方核出自证不过：哪块盘、从哪个偏移起读多长（可写挂载这次挂载共用的那一份读，
/// `mounted_session::DeviceReadOnceInThisMount`，记它）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReadFailingSelfVerification {
    pub device: DeviceIdentity,
    pub offset: DeviceOffsetInBytes,
    pub length_in_bytes: u64,
}

/// 一次读穿到底下那块盘时读错了：哪块盘、从哪个偏移起读多长、盘报的错（可写挂载这次挂载共用的那一份读，
/// `mounted_session::DeviceReadOnceInThisMount`，记它）。
#[derive(Debug)]
pub struct ReadFailedOnTheDisk {
    pub device: DeviceIdentity,
    pub offset: DeviceOffsetInBytes,
    pub length_in_bytes: u64,
    pub error: BlockDeviceError,
}

/// 改 size（截断、变长、只改记录）为什么做不了（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1、K12）。
/// 封闭集合，`match` 不写通配臂。都在任何落盘动作之前返回。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TruncationRefusal {
    /// 要改的 inode 不是树里一条活着的记录（树里没有、或 nlink = 0 删除中）：K1「在任何写之前拒成类型化错误」。
    InodeIsNotALiveRecord { inode: u64 },
    /// ⌈N ÷ 净荷⌉ × 净荷装不进 u64（K12）。
    TargetSizeRoundedUpToAWholeDataUnitOverflows { target_size_in_bytes: u64 },
    /// 读—改—写要读回的旧数据单元读盘读不回或核不过（D4（校验和位置） 已定项 4：读不回就失败，不拿零填充凑一个合法的零；判决
    /// `research/prompts/m4-partial-write-r1-main-verification.md` 第二节 R3「读盘」）：按偏移部分写要读回的旧单元（[`publish_partial_write`]）
    /// 与截断意图、变长重写的尾单元（截断取它旧净荷的前一段、变长在它后面补零）都报这一个，`read_back` 说是哪一样没过。调用方要做的决定
    /// 两条路一样（这个单元盘上读不回，写不了），所以同一个成员；它住在改 size 的拒绝里、外层是 `PublishError::TruncationRefused`，是因为
    /// [`PublishError`] 被挂载与会话两处穷举 `match`，另立外层成员要动那两份文件（这一批不碰）。
    OldDataUnitToRewriteNotReadBack {
        inode: InodeNumber,
        unit: DataUnitIndexInFile,
        read_back: OldDataUnitReadBackFailure,
    },
    /// 续做截断时 logged_ops 里那一条截断条目点名的对象不是现行那一版里那条活记录（按条目点名的号找记录：没有、nlink = 0、对象出生代或
    /// size 对不上目标）：条款没写这一格怎么续，第一版拒。
    TruncationIntentDoesNotNameTheLiveRecord { intent: TruncationIntent },
    /// 续做截断时现行那一版没有截断条目：调用方（共用走空）只在有截断条目时调它。
    NoTruncationIntentToContinue,
}

/// 建快照那次发布（[`publish_create_snapshot`]）为什么建不了。封闭集合，`match` 不写通配臂。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapshotCreationRefusal {
    /// 上一版树表里已经有 [`SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION`] 个只读快照（码 14 的条数 `snapshots`）：第一版同时活着的至多 10 个
    /// （用户 2026-10-13 定，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第七节岔路 1）。
    MoreSnapshotsThanTheFirstVersionAllows { snapshots: usize },
    /// 从上一版的树 ID 水位起连发两个号（快照 inode 根、快照 extent 根）越过 `u64::MAX`：水位是盘上读来的 8 字节，坏镜像、外来镜像才走得到。
    TreeIdentifierWatermarkLeavesNoRoomForTheSnapshotTrees { tree_identifier_watermark: u64 },
    /// 上一版里有快照 `being_destroyed` 还在销毁中：删快照串行，删完再接别的改动（判决 `research/prompts/m4-snapshot-r1-main-verification.md`
    /// 第七节岔路 2）。会话与挂载续做先走空（连销毁中的快照一起），走到这里的是直接调发布函数的调用方。
    SnapshotBeingDestroyed { being_destroyed: TreeIdentifier },
}

/// 删快照（[`publish_snapshot_destruction_intent`]、[`publish_snapshot_destruction_step`]）为什么发不了。封闭集合，`match` 不写通配臂。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapshotDestructionRefusal {
    /// 树表里没有树号是 `snapshot` 的码 14 条目（快照按码 14 的树号点名，判决第七节岔路 5）。
    NoSnapshotWithThisTree { snapshot: TreeIdentifier },
    /// 要写意图的快照已经带着「销毁中」：它的意图那次已经发过，接着走批（会话与挂载续做的走空做这件事）。
    SnapshotAlreadyBeingDestroyed { snapshot: TreeIdentifier },
    /// 另一个快照 `being_destroyed` 还在销毁中：删快照串行（用户 2026-10-13 定「删完再接别的改动」，判决第七节岔路 2），先把它走完。
    AnotherSnapshotIsBeingDestroyed { being_destroyed: TreeIdentifier },
    /// 要走批的快照不带「销毁中」：它的意图还没发。
    SnapshotNotBeingDestroyed { snapshot: TreeIdentifier },
}

/// 删快照的一批要放的一条 deadlist 条目为什么放不了。
#[derive(Debug)]
pub enum DeadlistEntryReleaseRefusal {
    /// 它留存的映射条目在上一版里没有（C300（deadlist 条目按 key 查映射查不到没有条款）：第一版不处置）。
    KeptMappingEntryMissing,
    /// 它映射 key 的类标签不是数据单元、码 2 节点、码 3 容器之一（盘上读来的一字节）：跨度判不了。
    UnitClassOutsideTheUserVisibleOnes { unit_class: u8 },
    /// 落点核不过（两条位置项不同槽、某块盘的记录不在册 / 已释放 / 跨度不对、位置项指池外的盘或两条指同一块盘）：`cause` 原样是
    /// 释放判定路径的那一种，里面的角色是按类标签取的那一类用户可见角色（[`user_visible_role_standing_for_a_unit_class`]）。
    PlacementNotReleasable(Box<PublishError>),
}

impl From<BlockDeviceError> for PublishError {
    fn from(error: BlockDeviceError) -> Self {
        PublishError::BlockDevice(error)
    }
}

impl From<InodeTreeWriteRefusal> for PublishError {
    fn from(refusal: InodeTreeWriteRefusal) -> Self {
        PublishError::InodeTreeWriteRefused(refusal)
    }
}
